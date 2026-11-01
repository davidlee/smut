//! The `where` predicate: a closed interpreter over **one entity's top-level
//! fields** (§6).
//!
//! A predicate runs on roots before traversal and never sees a discovered
//! entity unless a caller passes one back as a root, so the whole surface here
//! is one JSON [`Value`] in and a `bool` out. There is no coercion, regex,
//! JSONPath, nested-field syntax or executable predicate: a field name addresses
//! a top-level member of the object, and the five operators plus the three
//! clauses are the whole language.
//!
//! This module and everything it reaches touch neither the store, nor the clock,
//! nor the filesystem (VA-1): there is no `crate::store`, no `std::fs` and no
//! `SystemTime`/`Instant` below this point, which is what lets a predicate be
//! decided against an entity a test built in memory.
//!
//! The AST is the design's (§6.1) and the parsing is the design's (§6.2),
//! including the disambiguation rule that makes `$` reserved inside a condition
//! object: a literal object carrying operator-shaped keys is reachable only as
//! an explicit `$eq` operand.

use serde_json::Value;

use crate::error::Reject;

/// A `where` predicate over one entity's top-level fields (§6.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicate {
    /// One top-level field, with every condition ANDed.
    Field(String, Vec<Condition>),
    /// Every predicate holds; an empty list is true.
    And(Vec<Predicate>),
    /// At least one predicate holds; an empty list is false.
    Or(Vec<Predicate>),
    /// The predicate does not hold.
    Not(Box<Predicate>),
}

/// One condition on one top-level field (§6.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Present and structurally equal to the operand.
    Eq(Value),
    /// Present and not structurally equal to the operand.
    Ne(Value),
    /// Present and equal to one of the operand array's values.
    In(Vec<Value>),
    /// Present, or absent, as the operand boolean says.
    Exists(bool),
    /// A string field containing the operand substring, or an array field
    /// holding a structurally equal member.
    Contains(Value),
}

/// Parses a `where` value into a [`Predicate`] (§6.2).
///
/// Keys are examined in the object's own (lexical) order, and sibling keys are
/// ANDed, so a `where` object with more than one defect always reports the same
/// one.
pub fn parse(value: &Value) -> Result<Predicate, Reject> {
    let members = value
        .as_object()
        .ok_or_else(|| invalid("where", "a predicate object"))?;

    let mut clauses = Vec::new();

    for (key, value) in members {
        match key.as_str() {
            "$and" => clauses.push(Predicate::And(parse_all("$and", value)?)),
            "$or" => clauses.push(Predicate::Or(parse_all("$or", value)?)),
            "$not" => clauses.push(parse_not(value)?),
            operator if operator.starts_with('$') => {
                return Err(Reject::UnknownPredicateOperator {
                    operator: operator.to_owned(),
                });
            }
            field => clauses.push(parse_field(field, value)?),
        }
    }

    // `$and` of nothing is true, so an empty object is the identity predicate.
    Ok(if clauses.len() == 1 {
        clauses.pop().expect("exactly one clause")
    } else {
        Predicate::And(clauses)
    })
}

/// Parses `$and` or `$or`: an array of predicate objects.
fn parse_all(operator: &str, value: &Value) -> Result<Vec<Predicate>, Reject> {
    const EXPECTED: &str = "an array of predicate objects";

    let items = value
        .as_array()
        .ok_or_else(|| invalid_predicate(operator, EXPECTED))?;

    items
        .iter()
        .map(|item| {
            if !item.is_object() {
                return Err(invalid_predicate(operator, EXPECTED));
            }

            parse(item)
        })
        .collect()
}

/// Parses `$not`: exactly one predicate object, negated.
fn parse_not(value: &Value) -> Result<Predicate, Reject> {
    const OPERATOR: &str = "$not";

    if !value.is_object() {
        return Err(invalid_predicate(OPERATOR, "a predicate object"));
    }

    Ok(Predicate::Not(Box::new(parse(value)?)))
}

/// Parses one field's value into its conditions (§6.2's disambiguation rule).
///
/// A value with no `$`-prefixed key is a literal and means `$eq`; a value with
/// only `$`-prefixed keys is an operator object whose operators are all ANDed; a
/// value mixing the two is refused, because the design gives it no behaviour.
fn parse_field(field: &str, value: &Value) -> Result<Predicate, Reject> {
    let Some(members) = value.as_object() else {
        return Ok(Predicate::Field(
            field.to_owned(),
            vec![Condition::Eq(value.clone())],
        ));
    };

    let operators = members.keys().filter(|key| key.starts_with('$')).count();

    if operators == 0 {
        // `{}` has no `$` key, so it is the literal `Eq({})` and not an empty
        // operator object that cannot exist.
        return Ok(Predicate::Field(
            field.to_owned(),
            vec![Condition::Eq(value.clone())],
        ));
    }

    if operators != members.len() {
        return Err(Reject::MixedPredicateObject {
            field: field.to_owned(),
        });
    }

    let conditions = members
        .iter()
        .map(|(operator, operand)| parse_condition(operator, operand))
        .collect::<Result<Vec<Condition>, Reject>>()?;

    Ok(Predicate::Field(field.to_owned(), conditions))
}

/// Parses one operator inside a field condition. The three clauses are
/// predicate-level keys, so inside a condition object they are unknown
/// operators.
fn parse_condition(operator: &str, operand: &Value) -> Result<Condition, Reject> {
    match operator {
        "$eq" => Ok(Condition::Eq(operand.clone())),
        "$ne" => Ok(Condition::Ne(operand.clone())),
        "$in" => operand
            .as_array()
            .map(|values| Condition::In(values.clone()))
            .ok_or_else(|| invalid_predicate("$in", "an array of values")),
        "$exists" => operand
            .as_bool()
            .map(Condition::Exists)
            .ok_or_else(|| invalid_predicate("$exists", "a boolean")),
        "$contains" => Ok(Condition::Contains(operand.clone())),
        other => Err(Reject::UnknownPredicateOperator {
            operator: other.to_owned(),
        }),
    }
}

fn invalid(key: &str, expected: &str) -> Reject {
    Reject::InvalidSpecificationValue {
        key: key.to_owned(),
        expected: expected.to_owned(),
    }
}

fn invalid_predicate(operator: &str, expected: &str) -> Reject {
    Reject::InvalidPredicateValue {
        operator: operator.to_owned(),
        expected: expected.to_owned(),
    }
}

impl Predicate {
    /// Whether `entity` — one full stored entity, `id` included — satisfies this
    /// predicate (§6.3).
    ///
    /// A non-object entity is treated as carrying no field at all, so every
    /// field condition sees an absent value.
    pub fn holds(&self, entity: &Value) -> bool {
        match self {
            Predicate::Field(field, conditions) => {
                let value = entity.as_object().and_then(|members| members.get(field));

                conditions.iter().all(|condition| condition.holds(value))
            }
            Predicate::And(predicates) => {
                predicates.iter().all(|predicate| predicate.holds(entity))
            }
            Predicate::Or(predicates) => predicates.iter().any(|predicate| predicate.holds(entity)),
            Predicate::Not(predicate) => !predicate.holds(entity),
        }
    }
}

impl Condition {
    /// Whether this condition holds of `field`: that field's value, or `None`
    /// when the entity does not carry it.
    ///
    /// Missing is distinct from present-`null`: only `$exists` can hold of a
    /// missing field, and its boolean decides. Equality is `serde_json`'s, so no
    /// comparator is introduced here.
    fn holds(&self, field: Option<&Value>) -> bool {
        match self {
            Condition::Eq(operand) => field.is_some_and(|value| value == operand),
            Condition::Ne(operand) => field.is_some_and(|value| value != operand),
            Condition::In(operands) => {
                field.is_some_and(|value| operands.iter().any(|operand| operand == value))
            }
            Condition::Exists(want) => field.is_some() == *want,
            Condition::Contains(operand) => match (field, operand) {
                (Some(Value::String(text)), Value::String(fragment)) => {
                    text.contains(fragment.as_str())
                }
                (Some(Value::Array(items)), operand) => items.iter().any(|item| item == operand),
                _ => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    fn parsed(input: Value) -> Predicate {
        parse(&input).expect("the fixture predicate is valid")
    }

    fn refused(input: Value) -> Reject {
        parse(&input).expect_err("the fixture predicate should have been refused")
    }

    fn field(name: &str, conditions: Vec<Condition>) -> Predicate {
        Predicate::Field(name.to_owned(), conditions)
    }

    #[test]
    fn parse_builds_an_eq_condition_from_a_literal_value() {
        // §6.2: a value with no `$` key is a literal and means `$eq`.
        assert_eq!(
            parsed(json!({"type": "question"})),
            field("type", vec![Condition::Eq(json!("question"))])
        );
        assert_eq!(
            parsed(json!({"tags": ["open", 1]})),
            field("tags", vec![Condition::Eq(json!(["open", 1]))])
        );
        // An object without a `$` key is one literal, `{}` included.
        assert_eq!(
            parsed(json!({"meta": {"a": 1}})),
            field("meta", vec![Condition::Eq(json!({"a": 1}))])
        );
        assert_eq!(
            parsed(json!({"meta": {}})),
            field("meta", vec![Condition::Eq(json!({}))])
        );
    }

    #[test]
    fn parse_ands_the_operators_of_one_operator_object() {
        // §6.2: a value with only `$` keys is an operator object, and its
        // operators are ANDed in the object's own (lexical) order.
        assert_eq!(
            parsed(json!({"type": {"$in": ["question", "decision"], "$exists": true}})),
            field(
                "type",
                vec![
                    Condition::Exists(true),
                    Condition::In(vec![json!("question"), json!("decision")]),
                ]
            )
        );
        // `$` is reserved inside a condition object, so a literal object that
        // carries `$` keys is reachable only as an explicit `$eq` operand.
        assert_eq!(
            parsed(json!({"meta": {"$eq": {"$ref": "x"}}})),
            field("meta", vec![Condition::Eq(json!({"$ref": "x"}))])
        );
    }

    #[test]
    fn parse_refuses_a_mixed_object_naming_the_field() {
        // §6.2: a value mixing a literal key with an operator key names no
        // behaviour, so it is refused rather than guessed at.
        let rejection = refused(json!({"meta": {"$eq": 1, "x": 2}}));

        assert_eq!(
            rejection,
            Reject::MixedPredicateObject {
                field: "meta".to_owned(),
            }
        );

        let message = rejection.to_string();
        assert!(message.contains("meta"), "{message}");
        assert!(message.contains("mix"), "{message}");
    }

    #[test]
    fn parse_builds_the_clauses_and_ands_sibling_keys() {
        // §6.2: `$and` and `$or` take arrays of predicate objects, `$not` takes
        // one, and a field condition beside a clause is one more conjunct.
        assert_eq!(parsed(json!({"$and": []})), Predicate::And(Vec::new()));
        assert_eq!(parsed(json!({"$or": []})), Predicate::Or(Vec::new()));
        assert_eq!(
            parsed(json!({"$not": {"status": "open"}})),
            Predicate::Not(Box::new(field(
                "status",
                vec![Condition::Eq(json!("open"))]
            )))
        );
        assert_eq!(
            parsed(json!({"$not": {"$not": {"status": "open"}}})),
            Predicate::Not(Box::new(Predicate::Not(Box::new(field(
                "status",
                vec![Condition::Eq(json!("open"))]
            )))))
        );
        // `{}` has no key to AND, and `$and` of nothing is true, so an empty
        // `where` parses to the identity predicate.
        assert_eq!(parsed(json!({})), Predicate::And(Vec::new()));
        assert_eq!(
            parsed(json!({"type": "question", "$or": [{"status": "open"}]})),
            Predicate::And(vec![
                Predicate::Or(vec![field("status", vec![Condition::Eq(json!("open"))])]),
                field("type", vec![Condition::Eq(json!("question"))]),
            ])
        );
    }

    #[test]
    fn parse_refuses_a_wrong_operand_or_an_unknown_operator_naming_it() {
        // §6.2: `$and`/`$or` take an array of predicate objects, `$not` one
        // predicate object, `$in` an array and `$exists` a boolean; every
        // refusal names the operator it is about.
        for (input, operator) in [
            (json!({"$and": 1}), "$and"),
            (json!({"$and": [1]}), "$and"),
            (json!({"$or": "x"}), "$or"),
            (json!({"$or": [{"a": 1}, []]}), "$or"),
            (json!({"$not": []}), "$not"),
        ] {
            assert_eq!(
                refused(input.clone()),
                Reject::InvalidPredicateValue {
                    operator: operator.to_owned(),
                    expected: if operator == "$not" {
                        "a predicate object".to_owned()
                    } else {
                        "an array of predicate objects".to_owned()
                    },
                },
                "{input}"
            );
        }

        assert_eq!(
            refused(json!({"x": {"$in": 1}})),
            Reject::InvalidPredicateValue {
                operator: "$in".to_owned(),
                expected: "an array of values".to_owned(),
            }
        );
        assert_eq!(
            refused(json!({"x": {"$exists": null}})),
            Reject::InvalidPredicateValue {
                operator: "$exists".to_owned(),
                expected: "a boolean".to_owned(),
            }
        );
    }

    #[test]
    fn parse_refuses_an_unknown_operator_at_either_level() {
        // §6.2: at the predicate level any `$` key but the three clauses is an
        // unknown operator, and so is any condition-level key but the five
        // operators — the clauses included, because a clause is not an operator.
        for (input, operator) in [
            (json!({"$foo": 1}), "$foo"),
            (json!({"x": {"$foo": 1}}), "$foo"),
            (json!({"x": {"$and": []}}), "$and"),
            (json!({"x": {"$or": []}}), "$or"),
            (json!({"x": {"$not": {}}}), "$not"),
        ] {
            assert_eq!(
                refused(input.clone()),
                Reject::UnknownPredicateOperator {
                    operator: operator.to_owned(),
                },
                "{input}"
            );
        }
    }

    #[test]
    fn the_clauses_evaluate_as_the_identities_say() {
        // §6.3: `And` of nothing is true, `Or` of nothing is false, and `Not`
        // negates its one predicate.
        let entity = json!({"id": "a", "status": "open"});
        let open = field("status", vec![Condition::Eq(json!("open"))]);

        assert!(Predicate::And(Vec::new()).holds(&entity));
        assert!(!Predicate::Or(Vec::new()).holds(&entity));
        assert!(Predicate::And(vec![open.clone()]).holds(&entity));
        assert!(Predicate::Or(vec![Predicate::Or(Vec::new()), open.clone()]).holds(&entity));
        assert!(!Predicate::Not(Box::new(open.clone())).holds(&entity));
        assert!(Predicate::Not(Box::new(Predicate::Not(Box::new(open)))).holds(&entity));
    }

    #[test]
    fn each_operator_holds_exactly_where_the_table_says() {
        // §6.3's rows, including `$ne`'s "a missing field does not match" and
        // `$contains`'s "other combinations do not match".
        let entity = json!({
            "id": "a",
            "type": "question",
            "tags": ["open", {"k": 2}],
            "count": 3,
            "body": null,
        });

        assert!(field("type", vec![Condition::Eq(json!("question"))]).holds(&entity));
        assert!(!field("type", vec![Condition::Eq(json!("decision"))]).holds(&entity));

        assert!(field("type", vec![Condition::Ne(json!("decision"))]).holds(&entity));
        assert!(!field("type", vec![Condition::Ne(json!("question"))]).holds(&entity));
        // `$ne` needs the field present: absent is not "not equal".
        assert!(!field("absent", vec![Condition::Ne(json!(null))]).holds(&entity));

        assert!(
            field(
                "type",
                vec![Condition::In(vec![json!("decision"), json!("question")])]
            )
            .holds(&entity)
        );
        assert!(!field("type", vec![Condition::In(vec![json!("decision")])]).holds(&entity));

        assert!(field("body", vec![Condition::Exists(true)]).holds(&entity));
        assert!(!field("body", vec![Condition::Exists(false)]).holds(&entity));
        assert!(field("absent", vec![Condition::Exists(false)]).holds(&entity));
        assert!(!field("absent", vec![Condition::Exists(true)]).holds(&entity));

        // `$contains` on a string field is a substring test.
        assert!(field("type", vec![Condition::Contains(json!("uest"))]).holds(&entity));
        assert!(!field("type", vec![Condition::Contains(json!("answer"))]).holds(&entity));
        // On an array field it is structural membership, object members included.
        assert!(field("tags", vec![Condition::Contains(json!({"k": 2}))]).holds(&entity));
        assert!(!field("tags", vec![Condition::Contains(json!("closed"))]).holds(&entity));
        // Every other combination does not match, and does not refuse either: a
        // number field answers no `$contains`, and a string field with a
        // non-string operand answers none — `$contains` never coerces.
        assert!(!field("count", vec![Condition::Contains(json!("3"))]).holds(&entity));
        assert!(!field("count", vec![Condition::Contains(json!(3))]).holds(&entity));
        assert!(!field("type", vec![Condition::Contains(json!(3))]).holds(&entity));
        assert!(!field("body", vec![Condition::Contains(json!(null))]).holds(&entity));
    }

    #[test]
    fn missing_and_present_null_are_different_fields() {
        // §6.3: present `null` is present, and presence is what `$exists`
        // decides; a missing field fails every operator but `$exists`.
        let absent = json!({"id": "a"});
        let null = json!({"id": "a", "f": null});

        assert!(field("f", vec![Condition::Exists(true)]).holds(&null));
        assert!(field("f", vec![Condition::Exists(false)]).holds(&absent));
        assert!(field("f", vec![Condition::Eq(json!(null))]).holds(&null));
        assert!(!field("f", vec![Condition::Eq(json!(null))]).holds(&absent));
        assert!(!field("f", vec![Condition::Ne(json!(null))]).holds(&null));
        assert!(!field("f", vec![Condition::Ne(json!(null))]).holds(&absent));
    }

    #[test]
    fn structural_equality_is_serde_json_equality() {
        // §6.3: no new comparator. Object key order is irrelevant, and an
        // integer is distinct from its float spelling, so `1` does not match
        // `1.0`.
        let object = serde_json::from_str::<Value>(r#"{"b": 2, "a": 1}"#).expect("a JSON object");

        assert!(
            field("meta", vec![Condition::Eq(json!({"a": 1, "b": 2}))])
                .holds(&json!({"meta": object}))
        );
        assert!(field("n", vec![Condition::Eq(json!(1))]).holds(&json!({"n": 1})));
        assert!(!field("n", vec![Condition::Eq(json!(1.0))]).holds(&json!({"n": 1})));
        assert!(field("n", vec![Condition::Eq(json!(1.0))]).holds(&json!({"n": 1.0})));
    }

    #[test]
    fn a_non_object_entity_carries_no_field() {
        // §6.3: fields are looked up among the entity object's members, so a
        // non-object entity treats every field as absent.
        for entity in [json!([]), json!("a"), json!(1), json!(null)] {
            assert!(field("f", vec![Condition::Exists(false)]).holds(&entity));
            assert!(!field("f", vec![Condition::Exists(true)]).holds(&entity));
            assert!(Predicate::And(Vec::new()).holds(&entity));
        }
    }
}
