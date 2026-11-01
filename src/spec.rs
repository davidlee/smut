//! The 0.1 query specification: four accepted keys, and two kinds of refusal.
//!
//! `from`, `select`, `order` and `limit` are the whole surface (DEC-011). Keys a
//! later release will define are refused *by name* with that release, so the
//! refusal is a promise a user can act on; keys nobody planned are refused
//! plainly as unknown, because naming a release for them would be a guess
//! presented as a promise. `query {}` is valid and selects everything subject to
//! the default limit.
//!
//! The parsed [`Limit`] has no "unlimited" spelling: `get` bypasses the default
//! through the crate-internal [`Limit::Unlimited`], which no input can reach
//! (DEC-001 rule 4). That is what keeps `get` from being a defaulted spec.

use serde_json::Value;

use crate::error::Reject;
use crate::id::Id;
use crate::order::{Direction, SortKey};

/// The result cap when a specification supplies none (§6.2).
pub const DEFAULT_LIMIT: u64 = 100;

/// The keys a later release will accept, and the release to name when refusing
/// them.
const FUTURE_KEYS: [&str; 5] = ["where", "follow", "backlinks", "depth", "budget"];

/// How many results a selection may return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// No `limit` key: [`DEFAULT_LIMIT`] applies.
    Default,
    /// A positive, explicit cap.
    Capped(u64),
    /// No cap at all. Only `get` asks for this, and `parse` cannot produce it.
    Unlimited,
}

/// A parsed query specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The explicit roots in their own array order; `None` means every id in ID
    /// order, and `Some([])` means nothing.
    pub from: Option<Vec<Id>>,
    /// The fields to project; `None` means all of them.
    pub select: Option<Vec<String>>,
    /// The sort keys in array order; empty means discovery order.
    pub order: Vec<SortKey>,
    /// The result cap.
    pub limit: Limit,
}

/// Parses a query specification, rejecting anything 0.1 does not define.
///
/// Keys are examined in the spec's own (lexical) order, so a spec with more than
/// one defect always reports the same one.
pub fn parse(input: &Value) -> Result<Spec, Reject> {
    let fields = input.as_object().ok_or(Reject::NotASpecification)?;

    let mut spec = Spec {
        from: None,
        select: None,
        order: Vec::new(),
        limit: Limit::Default,
    };

    for (key, value) in fields {
        match key.as_str() {
            "from" => spec.from = Some(parse_roots(value)?),
            "select" => spec.select = Some(parse_select(value)?),
            "order" => spec.order = parse_order(value)?,
            "limit" => spec.limit = Limit::Capped(parse_limit(value)?),
            key if FUTURE_KEYS.contains(&key) => {
                return Err(Reject::FutureSpecificationKey {
                    key: key.to_owned(),
                });
            }
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

    #[test]
    fn query_accepts_exactly_from_select_order_and_limit() {
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
        assert_eq!(parsed.limit, Limit::Capped(5));
    }

    #[test]
    fn an_empty_spec_selects_everything_subject_to_the_default_limit() {
        let parsed = spec(json!({})).expect("an empty spec is valid");

        assert_eq!(
            parsed,
            Spec {
                from: None,
                select: None,
                order: Vec::new(),
                limit: Limit::Default,
            }
        );
        assert_eq!(DEFAULT_LIMIT, 100);
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
    fn each_named_future_key_is_rejected_by_name_citing_the_release() {
        for key in ["where", "follow", "backlinks", "depth", "budget"] {
            let rejection = rejection(json!({key: {}}));

            assert_eq!(
                rejection,
                Reject::FutureSpecificationKey {
                    key: key.to_owned(),
                },
                "{key}"
            );

            let message = rejection.to_string();
            assert!(message.contains(key), "{message}");
            assert!(message.contains("0.2"), "{message}");
        }
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
            Limit::Capped(1)
        );
        assert_eq!(
            spec(json!({"limit": u64::MAX}))
                .expect("the largest integer a JSON number holds")
                .limit,
            Limit::Capped(u64::MAX)
        );
    }

    #[test]
    fn a_parsed_specification_never_asks_for_an_unlimited_result() {
        for input in [json!({}), json!({"limit": 1}), json!({"limit": u64::MAX})] {
            assert_ne!(
                spec(input).expect("a valid spec").limit,
                Limit::Unlimited,
                "only get may bypass the cap"
            );
        }
    }

    #[test]
    fn the_acceptance_set_is_exactly_the_four_keys() {
        for key in ["from", "select", "order", "limit"] {
            assert!(
                spec(json!({key: match key {
                    "from" => json!("smut"),
                    "select" => json!([]),
                    "order" => json!([]),
                    _ => json!(1),
                }}))
                .is_ok(),
                "{key} is accepted"
            );
        }
    }
}
