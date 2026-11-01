//! The query specification: nine accepted keys, and one kind of refusal.
//!
//! `from`, `where`, `select`, `order`, `limit`, `follow`, `backlinks`, `depth`
//! and `budget` are the whole surface (DEC-011, §5). `where` is the predicate
//! object of §6, parsed here so a malformed predicate is refused before any
//! store access. A traversal key is either the wildcard `"*"` or an array of
//! top-level field names; the wildcard has that one spelling, because a mixed
//! array would need a precedence rule between query order and lexical order that
//! the brief does not give. The traversal keys imply neither each other nor a
//! traversal: `depth` or `budget` without `follow` or `backlinks` parses and is
//! inert.
//!
//! Keys nobody planned are refused plainly as unknown, because naming a release
//! for them would be a guess presented as a promise. `query {}` is valid and
//! selects everything subject to the default limit.
//!
//! The parsed [`Limit`] has no "unlimited" spelling a user can reach: `get`
//! bypasses the default through [`Limit::unlimited`], which is crate-internal
//! and which `parse` never produces (DEC-001 rule 4). That is what keeps `get`
//! from being a defaulted spec.

use serde_json::Value;

use crate::error::Reject;
use crate::id::Id;
use crate::order::{Direction, SortKey};
use crate::predicate::{self, Predicate};
use crate::walk::Fields;

/// The result cap when a specification supplies none (§6.2).
pub const DEFAULT_LIMIT: u64 = 100;

/// The traversal depth when a specification supplies none (§5).
pub const DEFAULT_DEPTH: u64 = 1;

/// The traversal budget when a specification supplies none (§5).
pub const DEFAULT_BUDGET: u64 = 10000;

/// How many results a selection may return.
///
/// The representation is private so that the unlimited case has no public
/// constructor: a parsed specification can ask for an explicit cap or for the
/// default, and only this crate's `get` can lift the cap entirely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(LimitKind);

/// [`Limit`]'s states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LimitKind {
    Default,
    Capped(u64),
    Unlimited,
}

impl Limit {
    /// No `limit` key: [`DEFAULT_LIMIT`] applies.
    pub const DEFAULT: Limit = Limit(LimitKind::Default);

    /// An explicit, positive cap.
    pub const fn capped(results: u64) -> Self {
        Limit(LimitKind::Capped(results))
    }

    /// No cap at all: `get`'s constructor, unreachable from a parsed spec.
    pub(crate) const fn unlimited() -> Self {
        Limit(LimitKind::Unlimited)
    }

    /// The number of results this limit allows, or `None` when nothing caps it.
    pub fn cap(self) -> Option<u64> {
        match self.0 {
            LimitKind::Default => Some(DEFAULT_LIMIT),
            LimitKind::Capped(results) => Some(results),
            LimitKind::Unlimited => None,
        }
    }
}

/// A parsed query specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The explicit roots in their own array order; `None` means every id in ID
    /// order, and `Some([])` means nothing.
    pub from: Option<Vec<Id>>,
    /// The root predicate of §6; `None` means no filtering. It filters roots
    /// only, before traversal (§7.1, §10.1 inv. 2).
    pub r#where: Option<Predicate>,
    /// The fields to project; `None` means all of them.
    pub select: Option<Vec<String>>,
    /// The sort keys in array order; empty means discovery order.
    pub order: Vec<SortKey>,
    /// The result cap.
    pub limit: Limit,
    /// The outgoing fields a traversal follows; `None` means no outgoing
    /// traversal, while `Some` of an empty list is a supplied traversal that
    /// follows nothing.
    pub follow: Option<Fields>,
    /// The incoming fields a traversal follows; `None` means no incoming
    /// traversal, read with the same distinction as [`Spec::follow`].
    pub backlinks: Option<Fields>,
    /// The traversal depth; `0` is roots only, and a depth without `follow` or
    /// `backlinks` starts no traversal.
    pub depth: u64,
    /// The traversal budget: the entity expansions a walk may perform, checked
    /// before each one (§7.3). A budget without `follow` or `backlinks` starts
    /// no traversal and is inert.
    pub budget: u64,
}

/// Parses a query specification, rejecting anything it does not define.
///
/// Keys are examined in the spec's own (lexical) order, so a spec with more than
/// one defect always reports the same one.
pub fn parse(input: &Value) -> Result<Spec, Reject> {
    let fields = input.as_object().ok_or(Reject::NotASpecification)?;

    let mut spec = Spec {
        from: None,
        r#where: None,
        select: None,
        order: Vec::new(),
        limit: Limit::DEFAULT,
        follow: None,
        backlinks: None,
        depth: DEFAULT_DEPTH,
        budget: DEFAULT_BUDGET,
    };

    for (key, value) in fields {
        match key.as_str() {
            "from" => spec.from = Some(parse_roots(value)?),
            "where" => spec.r#where = Some(predicate::parse(value)?),
            "select" => spec.select = Some(parse_select(value)?),
            "order" => spec.order = parse_order(value)?,
            "limit" => spec.limit = Limit::capped(parse_limit(value)?),
            "follow" => spec.follow = Some(parse_fields("follow", value)?),
            "backlinks" => spec.backlinks = Some(parse_fields("backlinks", value)?),
            "depth" => spec.depth = parse_depth(value)?,
            "budget" => spec.budget = parse_budget(value)?,
            key => {
                return Err(Reject::UnknownSpecificationKey {
                    key: key.to_owned(),
                });
            }
        }
    }

    Ok(spec)
}

fn parse_roots(value: &Value) -> Result<Vec<Id>, Reject> {
    match value {
        Value::String(id) => Ok(vec![Id::new(id.clone())]),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(id) => Ok(Id::new(id.clone())),
                _ => Err(invalid("from", "a string or an array of strings")),
            })
            .collect(),
        _ => Err(invalid("from", "a string or an array of strings")),
    }
}

fn parse_select(value: &Value) -> Result<Vec<String>, Reject> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(field) => Ok(field.clone()),
                _ => Err(invalid("select", "an array of strings")),
            })
            .collect(),
        _ => Err(invalid("select", "an array of strings")),
    }
}

fn parse_order(value: &Value) -> Result<Vec<SortKey>, Reject> {
    match value {
        Value::Array(items) => items.iter().map(parse_order_entry).collect(),
        _ => Err(invalid("order", "an array of {field, direction?} entries")),
    }
}

fn parse_order_entry(entry: &Value) -> Result<SortKey, Reject> {
    let members = entry
        .as_object()
        .ok_or_else(|| invalid("order", "an array of {field, direction?} entries"))?;

    let mut field = None;
    let mut direction = Direction::Ascending;

    for (member, value) in members {
        match member.as_str() {
            "field" => {
                let name = value.as_str().ok_or_else(|| invalid("field", "a string"))?;
                field = Some(name.to_owned());
            }
            "direction" => direction = parse_direction(value)?,
            key => {
                return Err(Reject::UnknownOrderMember {
                    key: key.to_owned(),
                });
            }
        }
    }

    let field = field.ok_or_else(|| invalid("field", "a string"))?;

    Ok(SortKey { field, direction })
}

fn parse_direction(value: &Value) -> Result<Direction, Reject> {
    match value.as_str() {
        Some("asc") => Ok(Direction::Ascending),
        Some("desc") => Ok(Direction::Descending),
        _ => Err(invalid("direction", "`asc` or `desc`")),
    }
}

fn parse_limit(value: &Value) -> Result<u64, Reject> {
    match value.as_u64() {
        Some(limit) if limit > 0 => Ok(limit),
        _ => Err(invalid("limit", "a positive integer")),
    }
}

fn parse_budget(value: &Value) -> Result<u64, Reject> {
    match value.as_u64() {
        Some(budget) if budget > 0 => Ok(budget),
        _ => Err(invalid("budget", "a positive integer")),
    }
}

/// Parses a traversal key's value: the wildcard or a list of field names.
fn parse_fields(key: &str, value: &Value) -> Result<Fields, Reject> {
    match value {
        Value::String(field) if field == "*" => Ok(Fields::All),
        Value::Array(items) => {
            let mut labels: Vec<String> = Vec::with_capacity(items.len());

            for item in items {
                let label = item
                    .as_str()
                    .ok_or_else(|| invalid(key, "`*` or an array of top-level field names"))?;

                if label == "*" {
                    return Err(invalid(
                        key,
                        "an array of field names, because the wildcard is the string `*` on its own",
                    ));
                }

                // Duplicates are a set for edge selection and must never reorder
                // the first occurrence (§5).
                if !labels.iter().any(|asked| asked == label) {
                    labels.push(label.to_owned());
                }
            }

            Ok(Fields::Named(labels))
        }
        _ => Err(invalid(key, "`*` or an array of top-level field names")),
    }
}

fn parse_depth(value: &Value) -> Result<u64, Reject> {
    value
        .as_u64()
        .ok_or_else(|| invalid("depth", "a nonnegative integer"))
}

fn invalid(key: &str, expected: &str) -> Reject {
    Reject::InvalidSpecificationValue {
        key: key.to_owned(),
        expected: expected.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    fn spec(input: Value) -> Result<Spec, Reject> {
        parse(&input)
    }

    fn rejection(input: Value) -> Reject {
        parse(&input).expect_err("the specification should have been rejected")
    }

    /// An integer JSON number beyond `u64`: `18446744073709551616`, i.e. 2^64.
    fn beyond_u64() -> Value {
        serde_json::from_str("18446744073709551616").expect("a JSON number beyond u64")
    }

    #[test]
    fn query_accepts_the_0_1_keys() {
        // Re-cut from `query_accepts_exactly_from_select_order_and_limit`: the
        // name claimed an exhaustive surface and the surface has widened to
        // seven keys, so only the claim is dropped and the body is unchanged.
        let parsed = spec(json!({
            "from": "smut",
            "select": ["title", "body"],
            "order": [{"field": "priority", "direction": "desc"}],
            "limit": 5
        }))
        .expect("every key is one 0.1 accepts");

        assert_eq!(parsed.from, Some(vec![Id::new("smut")]));
        assert_eq!(
            parsed.select,
            Some(vec!["title".to_owned(), "body".to_owned()])
        );
        assert_eq!(parsed.order, vec![SortKey::descending("priority")]);
        assert_eq!(parsed.limit, Limit::capped(5));
    }

    #[test]
    fn an_empty_spec_selects_everything_subject_to_the_default_limit() {
        // Re-cut because the whole `Spec` is compared: the traversal keys add
        // their own defaults, `None` for both directions, `DEFAULT_DEPTH` and
        // `DEFAULT_BUDGET`.
        let parsed = spec(json!({})).expect("an empty spec is valid");

        assert_eq!(
            parsed,
            Spec {
                from: None,
                r#where: None,
                select: None,
                order: Vec::new(),
                limit: Limit::DEFAULT,
                follow: None,
                backlinks: None,
                depth: DEFAULT_DEPTH,
                budget: DEFAULT_BUDGET,
            }
        );
        assert_eq!(DEFAULT_LIMIT, 100);
        assert_eq!(parsed.limit.cap(), Some(100));
    }

    #[test]
    fn from_takes_a_string_or_an_array_of_strings_in_their_own_order() {
        assert_eq!(
            spec(json!({"from": "smut"})).expect("a string root").from,
            Some(vec![Id::new("smut")])
        );
        assert_eq!(
            spec(json!({"from": ["b", "a", "b"]}))
                .expect("an array keeps its order, repeats included")
                .from,
            Some(vec![Id::new("b"), Id::new("a"), Id::new("b")])
        );
        assert_eq!(
            spec(json!({"from": []})).expect("an empty root list").from,
            Some(Vec::new())
        );
    }

    #[test]
    fn select_takes_an_array_of_strings_and_an_empty_array_is_id_only() {
        assert_eq!(
            spec(json!({"select": []}))
                .expect("an empty field list")
                .select,
            Some(Vec::new())
        );
        assert!(spec(json!({"select": "title"})).is_err());
        assert!(spec(json!({"select": ["title", 1]})).is_err());
        assert!(spec(json!({"from": [1]})).is_err());
        assert!(spec(json!({"from": 1})).is_err());
    }

    #[test]
    fn an_order_entry_defaults_to_ascending_and_an_empty_order_equals_absent() {
        assert_eq!(
            spec(json!({"order": [{"field": "priority"}]}))
                .expect("direction is optional")
                .order,
            vec![SortKey::ascending("priority")]
        );
        assert_eq!(
            spec(json!({"order": []})).expect("an empty order").order,
            Vec::new()
        );
        assert_eq!(
            spec(json!({"order": [{"field": "title", "direction": "asc"}]}))
                .expect("asc is explicit")
                .order,
            vec![SortKey::ascending("title")]
        );
    }

    #[test]
    fn an_order_entry_rejects_an_unknown_member() {
        assert_eq!(
            rejection(json!({"order": [{"field": "title", "weight": 1}]})),
            Reject::UnknownOrderMember {
                key: "weight".to_owned(),
            }
        );
    }

    #[test]
    fn an_order_entry_needs_a_string_field_and_a_known_direction() {
        for entry in [
            json!({}),
            json!({"field": 1}),
            json!({"direction": "asc"}),
            json!({"field": "title", "direction": "upward"}),
            json!({"field": "title", "direction": null}),
        ] {
            assert!(
                matches!(
                    rejection(json!({"order": [entry]})),
                    Reject::InvalidSpecificationValue { .. }
                ),
                "{entry} should be refused for its value"
            );
        }

        assert!(matches!(
            rejection(json!({"order": {"field": "title"}})),
            Reject::InvalidSpecificationValue { .. }
        ));
    }

    #[test]
    fn an_unknown_key_is_reported_plainly_as_unknown_and_names_no_release() {
        let rejection = rejection(json!({"sort": [{"field": "title"}]}));

        assert_eq!(
            rejection,
            Reject::UnknownSpecificationKey {
                key: "sort".to_owned(),
            }
        );

        let message = rejection.to_string();
        assert!(message.contains("unknown"), "{message}");
        assert!(message.contains("sort"), "{message}");
        assert!(!message.contains("0.2"), "{message}");
    }

    #[test]
    fn a_non_object_spec_is_rejected() {
        for input in [
            json!([]),
            json!("select"),
            json!(1),
            json!(null),
            json!(true),
        ] {
            assert_eq!(rejection(input), Reject::NotASpecification);
        }

        assert!(
            Reject::NotASpecification
                .to_string()
                .contains("must be a JSON object")
        );
    }

    #[test]
    fn limit_must_be_a_positive_integer_and_zero_is_rejected() {
        for input in [
            json!({"limit": 0}),
            json!({"limit": -1}),
            json!({"limit": 1.5}),
            json!({"limit": 1.0}),
            json!({"limit": "5"}),
            json!({"limit": null}),
        ] {
            assert!(
                matches!(
                    rejection(input.clone()),
                    Reject::InvalidSpecificationValue { .. }
                ),
                "{input} should be refused"
            );
        }

        assert_eq!(
            spec(json!({"limit": 1})).expect("one result").limit,
            Limit::capped(1)
        );
        assert_eq!(
            spec(json!({"limit": u64::MAX}))
                .expect("the largest integer a JSON number holds")
                .limit,
            Limit::capped(u64::MAX)
        );
    }

    #[test]
    fn a_parsed_specification_never_asks_for_an_unlimited_result() {
        for input in [json!({}), json!({"limit": 1}), json!({"limit": 1000000})] {
            assert!(
                spec(input).expect("a valid spec").limit.cap().is_some(),
                "only get may bypass the cap"
            );
        }

        assert_eq!(
            Limit::unlimited().cap(),
            None,
            "get's constructor caps nothing"
        );
    }

    #[test]
    fn the_nine_keys_are_each_accepted() {
        // Named for what it does. The exclusivity half of "exactly" is not
        // asserted here: `an_unknown_key_is_reported_plainly_as_unknown_and_names_no_release`
        // (this module) and the CLI's `an_unknown_key_stays_unknown_and_names_no_release`
        // own it, so a tenth accepted key would fail there. The old name
        // claimed it twice over and could not fail on the thing it claimed.
        //
        // Re-cut from `the_acceptance_set_is_exactly_the_four_keys`, whose name
        // and body the widened surface falsified: each accepted key is exercised
        // with one value its own grammar allows. Re-cut again in PHASE-02
        // (DEC-019) when `budget` became real: the name claimed seven keys and
        // the loop omitted `budget`, so both the claim and the list grow by one.
        // (This re-cut was named by no criterion — reported as a plan defect.)
        // Re-cut a third time in PHASE-03 (DEC-019) when `where` became real: the
        // acceptance surface is complete now, so the claim and the list grow once
        // more and this is the last key the release adds.
        for (key, value) in [
            ("from", json!("smut")),
            ("where", json!({})),
            ("select", json!([])),
            ("order", json!([])),
            ("limit", json!(1)),
            ("follow", json!("*")),
            ("backlinks", json!(["questions"])),
            ("depth", json!(0)),
            ("budget", json!(1)),
        ] {
            assert!(spec(json!({key: value})).is_ok(), "{key} is accepted");
        }
    }

    #[test]
    fn where_takes_a_predicate_object_and_refuses_a_value_that_is_not_one() {
        // Rule: `where` is the predicate object of §6, so a field condition
        // parses into the AST and a non-object is refused before any store
        // access, naming the key and what it accepts.
        assert_eq!(
            spec(json!({"where": {"type": "question"}}))
                .expect("a predicate object")
                .r#where,
            Some(Predicate::Field(
                "type".to_owned(),
                vec![predicate::Condition::Eq(json!("question"))]
            ))
        );
        assert_eq!(
            spec(json!({"where": {}}))
                .expect("the identity predicate")
                .r#where,
            Some(Predicate::And(Vec::new()))
        );

        for value in [json!([]), json!("x"), json!(1)] {
            let rejection = rejection(json!({"where": value.clone()}));

            assert_eq!(
                rejection,
                Reject::InvalidSpecificationValue {
                    key: "where".to_owned(),
                    expected: "a predicate object".to_owned(),
                },
                "{value}"
            );

            assert!(
                rejection.to_string().contains("predicate object"),
                "{rejection}"
            );
        }
    }

    #[test]
    fn a_traversal_key_takes_the_wildcard_or_an_array_of_field_names() {
        // Rule: `follow`/`backlinks` are `"*"` or an array of top-level field
        // names, with an empty array a supplied traversal that follows nothing.
        assert_eq!(
            spec(json!({"follow": "*"})).expect("the wildcard").follow,
            Some(Fields::All)
        );
        assert_eq!(
            spec(json!({"backlinks": "*"}))
                .expect("the wildcard")
                .backlinks,
            Some(Fields::All)
        );
        assert_eq!(
            spec(json!({"follow": ["a", "b"]}))
                .expect("field names in query order")
                .follow,
            Some(Fields::Named(vec!["a".to_owned(), "b".to_owned()]))
        );
        assert_eq!(
            spec(json!({"backlinks": ["questions"]}))
                .expect("one field name")
                .backlinks,
            Some(Fields::Named(vec!["questions".to_owned()]))
        );
        assert_eq!(
            spec(json!({"follow": []}))
                .expect("an empty field list")
                .follow,
            Some(Fields::Named(Vec::new()))
        );
    }

    #[test]
    fn the_traversal_keys_are_absent_and_depth_is_one_by_default() {
        // Rule: absent `follow`/`backlinks` mean no traversal in that direction,
        // and absent `depth` is `DEFAULT_DEPTH`.
        let parsed = spec(json!({})).expect("an empty spec is valid");

        assert_eq!(parsed.follow, None);
        assert_eq!(parsed.backlinks, None);
        assert_eq!(parsed.depth, DEFAULT_DEPTH);
        assert_eq!(DEFAULT_DEPTH, 1);
    }

    #[test]
    fn depth_accepts_zero_and_any_integer_a_json_number_holds() {
        // Rule: `depth` is a nonnegative integer, so `0` (roots only) is valid
        // and nothing caps it below `u64::MAX`.
        assert_eq!(spec(json!({"depth": 0})).expect("roots only").depth, 0);
        assert_eq!(spec(json!({"depth": 5})).expect("five hops").depth, 5);
        assert_eq!(
            spec(json!({"depth": u64::MAX}))
                .expect("the largest integer a JSON number holds")
                .depth,
            u64::MAX
        );
    }

    #[test]
    fn depth_without_a_traversal_key_is_accepted_and_inert() {
        // Rule: the new keys do not imply each other or a traversal, so `depth`
        // alone parses and leaves both directions absent.
        let parsed = spec(json!({"depth": 3})).expect("depth alone is valid");

        assert_eq!(parsed.depth, 3);
        assert_eq!(parsed.follow, None);
        assert_eq!(parsed.backlinks, None);
    }

    #[test]
    fn the_wildcard_has_one_spelling_so_an_array_element_star_is_refused() {
        // Rule: the wildcard is the string `*` on its own; an array holds field
        // names only, never a wildcard element.
        for key in ["follow", "backlinks"] {
            for value in [json!(["*"]), json!(["a", "*"])] {
                let rejection = rejection(json!({key: value.clone()}));

                assert_eq!(
                    rejection,
                    Reject::InvalidSpecificationValue {
                        key: key.to_owned(),
                        expected: "an array of field names, because the wildcard is the string `*` on its own"
                            .to_owned(),
                    },
                    "{key}: {value}"
                );

                let message = rejection.to_string();
                assert!(message.contains(key), "{message}");
                assert!(message.contains('*'), "{message}");
                assert!(message.contains("string"), "{message}");
                assert!(!message.contains("0.2"), "{message}");
            }
        }
    }

    #[test]
    fn a_traversal_key_refuses_a_shape_outside_its_grammar() {
        // Rule: only `"*"` and an array are traversal values, so a bare
        // non-wildcard string, a number, null and an object are all refused.
        for key in ["follow", "backlinks"] {
            for value in [json!("questions"), json!(1), json!(null), json!({"a": "b"})] {
                assert_eq!(
                    rejection(json!({key: value.clone()})),
                    Reject::InvalidSpecificationValue {
                        key: key.to_owned(),
                        expected: "`*` or an array of top-level field names".to_owned(),
                    },
                    "{key}: {value}"
                );
            }
        }
    }

    #[test]
    fn a_follow_array_element_must_be_a_field_name() {
        // Rule: the array grammar is top-level field names only, so a non-string
        // element is refused rather than coerced.
        for value in [json!([1]), json!(["a", 1]), json!([null])] {
            assert_eq!(
                rejection(json!({"follow": value.clone()})),
                Reject::InvalidSpecificationValue {
                    key: "follow".to_owned(),
                    expected: "`*` or an array of top-level field names".to_owned(),
                },
                "{value}"
            );
        }
    }

    #[test]
    fn budget_is_a_positive_integer_and_starts_no_traversal_by_itself() {
        // Rules: `budget` counts entity expansions and is a positive integer
        // (§5, §7.3), and without `follow` or `backlinks` it is inert: it
        // implies no traversal, exactly as `depth` does.
        let parsed = spec(json!({"budget": 5})).expect("a positive budget");

        assert_eq!(parsed.budget, 5);
        assert_eq!(parsed.follow, None);
        assert_eq!(parsed.backlinks, None);

        for value in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!(1.0),
            json!("5"),
            beyond_u64(),
        ] {
            assert_eq!(
                rejection(json!({"budget": value.clone()})),
                Reject::InvalidSpecificationValue {
                    key: "budget".to_owned(),
                    expected: "a positive integer".to_owned(),
                },
                "{value}"
            );
        }
    }

    #[test]
    fn depth_refuses_a_negative_fractional_string_or_beyond_u64_value() {
        // Rule: `value.as_u64()` is the test, so `-1`, `1.5`, `1.0`, a string
        // and an integer beyond u64 are refused and never defaulted to 1.
        for value in [json!(-1), json!(1.5), json!(1.0), json!("2"), beyond_u64()] {
            assert_eq!(
                rejection(json!({"depth": value.clone()})),
                Reject::InvalidSpecificationValue {
                    key: "depth".to_owned(),
                    expected: "a nonnegative integer".to_owned(),
                },
                "{value}"
            );
        }
    }

    #[test]
    fn duplicate_labels_collapse_to_their_first_occurrence() {
        // Rule: duplicates are a set for edge selection and must never reorder
        // the first occurrence (§5), so `["b", "a", "b"]` keeps `b` first.
        assert_eq!(
            spec(json!({"follow": ["b", "a", "b"]}))
                .expect("a repeated label")
                .follow,
            Some(Fields::Named(vec!["b".to_owned(), "a".to_owned()]))
        );
        assert_eq!(
            spec(json!({"backlinks": ["a", "b", "a", "c", "b"]}))
                .expect("later duplicates of earlier labels")
                .backlinks,
            Some(Fields::Named(vec![
                "a".to_owned(),
                "b".to_owned(),
                "c".to_owned(),
            ]))
        );
    }
}
