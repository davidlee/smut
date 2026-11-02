//! Ordering: the comparator DEC-006 specifies, and the validation that must
//! precede it.
//!
//! Per key, in array order: both missing continues to the next key; exactly one
//! missing sorts that record later, whatever the direction; both present compares
//! the values and applies the direction only to a non-equal result. All keys
//! tying breaks on ID ascending, so the order is total without depending on sort
//! stability or discovery order.
//!
//! Missing-last is a per-key rule, never a final partition: two records that both
//! lack `priority` are still ordered by the next key. And because a comparator
//! only ever sees pairs, an array or object in an order field would be discovered
//! late, or not at all, so every candidate's every sort value is validated before
//! sorting begins.

use std::cmp::Ordering;

use serde_json::{Map, Value};

use crate::declare::Entities;
use crate::error::Reject;
use crate::id::Id;
use crate::numeric;

/// Which way one `order` entry reads its field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Smallest first: the default.
    Ascending,
    /// Largest first.
    Descending,
}

/// One `order` entry: a top-level field and the direction to read it in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortKey {
    /// The top-level field to sort by.
    pub field: String,
    /// Which way to read it.
    pub direction: Direction,
}

impl SortKey {
    /// An `asc` entry.
    pub fn ascending(field: impl Into<String>) -> Self {
        SortKey {
            field: field.into(),
            direction: Direction::Ascending,
        }
    }

    /// A `desc` entry.
    pub fn descending(field: impl Into<String>) -> Self {
        SortKey {
            field: field.into(),
            direction: Direction::Descending,
        }
    }
}

/// Sorts `ids` by `keys`, rejecting any sort value that has no ordering.
///
/// Validation runs over every candidate and every key first, so the rejection
/// does not depend on how many candidates there are or on which key would decide
/// a comparison.
pub fn sort_ids(ids: &mut [&Id], store: &Entities, keys: &[SortKey]) -> Result<(), Reject> {
    if keys.is_empty() {
        return Ok(());
    }

    for id in ids.iter() {
        let entity = entity_of(store, id);

        for key in keys {
            if let Some(value) = entity.get(key.field.as_str())
                && let Some(found) = unsortable(value)
            {
                return Err(Reject::UnsortableValue {
                    field: key.field.clone(),
                    found,
                });
            }
        }
    }

    ids.sort_by(|left, right| compare(store, left, right, keys));

    Ok(())
}

/// The comparator: every key in array order, then the ID tiebreak.
fn compare(store: &Entities, left: &Id, right: &Id, keys: &[SortKey]) -> Ordering {
    let left_fields = entity_of(store, left);
    let right_fields = entity_of(store, right);

    for key in keys {
        let left_value = left_fields.get(key.field.as_str());
        let right_value = right_fields.get(key.field.as_str());

        match (left_value, right_value) {
            (None, None) => continue,
            // A missing value sorts last whatever the direction, so the
            // direction never reaches this arm.
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(left_value), Some(right_value)) => {
                let ordering = compare_values(left_value, right_value);

                if ordering != Ordering::Equal {
                    return match key.direction {
                        Direction::Ascending => ordering,
                        Direction::Descending => ordering.reverse(),
                    };
                }
            }
        }
    }

    left.cmp(right)
}

/// The scalar type order, then natural comparison within one type.
fn compare_values(left: &Value, right: &Value) -> Ordering {
    let left_rank = rank(left);
    let right_rank = rank(right);

    if left_rank != right_rank {
        return left_rank.cmp(&right_rank);
    }

    match (left, right) {
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Bool(left), Value::Bool(right)) => left.cmp(right),
        (Value::Number(left), Value::Number(right)) => numeric::compare(left, right),
        (Value::String(left), Value::String(right)) => left.as_bytes().cmp(right.as_bytes()),
        _ => unreachable!("sort values are validated before any comparison"),
    }
}

/// `null` < booleans < numbers < strings.
fn rank(value: &Value) -> u8 {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        _ => unreachable!("sort values are validated before any comparison"),
    }
}

/// Why a value cannot be ordered, if it cannot be.
fn unsortable(value: &Value) -> Option<String> {
    match value {
        Value::Array(_) => Some("an array".to_owned()),
        Value::Object(_) => Some("an object".to_owned()),
        _ => None,
    }
}

/// One candidate's fields. The ids handed to [`sort_ids`] come from the same
/// snapshot, which the shell has already validated.
fn entity_of<'a>(store: &'a Entities, id: &Id) -> &'a Map<String, Value> {
    store
        .get(id)
        .unwrap_or_else(|| panic!("sort_ids is given ids drawn from the store: {id} is absent"))
        .as_object()
        .unwrap_or_else(|| panic!("the snapshot is validated: {id} is not an entity object"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::id::Id;

    fn store(entities: &[Value]) -> Entities {
        entities
            .iter()
            .map(|entity| {
                let id = entity["smut_id"]
                    .as_str()
                    .expect("every fixture names its smut_id");
                (Id::new(id), entity.clone())
            })
            .collect()
    }

    fn ordered(store: &Entities, keys: &[SortKey]) -> Result<Vec<String>, Reject> {
        let mut ids: Vec<&Id> = store.keys().collect();
        sort_ids(&mut ids, store, keys)?;

        Ok(ids.iter().map(|id| id.to_string()).collect())
    }

    fn ascending(fields: &[&str]) -> Vec<SortKey> {
        fields
            .iter()
            .map(|field| SortKey::ascending(*field))
            .collect()
    }

    #[test]
    fn missing_values_sort_last_per_key_and_never_as_a_final_partition() {
        let store = store(&[
            json!({"smut_id": "a", "priority": 1, "title": "b"}),
            json!({"smut_id": "b", "title": "a"}),
            json!({"smut_id": "c", "priority": 2, "title": "a"}),
            json!({"smut_id": "d", "title": "z"}),
        ]);

        let order = ordered(&store, &ascending(&["priority", "title"]))
            .expect("every sort value is a scalar");

        // `b` and `d` both lack `priority`, so the next key still orders them.
        assert_eq!(order, ["a", "c", "b", "d"]);
    }

    #[test]
    fn the_cmp_puts_a_missing_value_last_whichever_the_direction() {
        let store = store(&[
            json!({"smut_id": "a", "priority": 1}),
            json!({"smut_id": "b"}),
            json!({"smut_id": "c", "priority": 2}),
        ]);

        let ascending_order = ordered(&store, &ascending(&["priority"])).expect("scalar values");
        let descending_order =
            ordered(&store, &[SortKey::descending("priority")]).expect("scalar values");

        assert_eq!(ascending_order, ["a", "c", "b"]);
        assert_eq!(descending_order, ["c", "a", "b"]);
    }

    #[test]
    fn direction_reverses_present_values_but_never_the_id_tiebreak() {
        let store = store(&[
            json!({"smut_id": "b", "title": "same"}),
            json!({"smut_id": "a", "title": "same"}),
            json!({"smut_id": "c", "title": "same"}),
        ]);

        let order = ordered(&store, &[SortKey::descending("title")]).expect("scalar values");

        assert_eq!(order, ["a", "b", "c"]);
    }

    #[test]
    fn equal_sort_values_fall_back_to_id_ascending() {
        let store = store(&[
            json!({"smut_id": "m", "priority": 1}),
            json!({"smut_id": "z", "priority": 1}),
            json!({"smut_id": "a", "priority": 1}),
        ]);

        assert_eq!(
            ordered(&store, &ascending(&["priority"])).expect("scalar values"),
            ["a", "m", "z"]
        );
    }

    #[test]
    fn the_scalar_type_order_is_null_then_booleans_then_numbers_then_strings() {
        let store = store(&[
            json!({"smut_id": "string", "value": "text"}),
            json!({"smut_id": "number", "value": 1}),
            json!({"smut_id": "true", "value": true}),
            json!({"smut_id": "false", "value": false}),
            json!({"smut_id": "null", "value": null}),
        ]);

        let order = ordered(&store, &ascending(&["value"])).expect("scalar values");

        assert_eq!(order, ["null", "false", "true", "number", "string"]);
    }

    #[test]
    fn strings_compare_byte_wise_with_no_collation_or_normalisation() {
        // Locale collation would interleave `Ä` with `A`; byte order puts every
        // ASCII letter first, then the two-byte sequences by their bytes.
        let store = store(&[
            json!({"smut_id": "a", "value": "a"}),
            json!({"smut_id": "upper", "value": "Z"}),
            json!({"smut_id": "umlaut", "value": "Ä"}),
            json!({"smut_id": "acute", "value": "é"}),
            json!({"smut_id": "capital", "value": "A"}),
        ]);

        let order = ordered(&store, &ascending(&["value"])).expect("scalar values");

        assert_eq!(order, ["capital", "upper", "a", "umlaut", "acute"]);
    }

    #[test]
    fn smut_id_is_an_ordinary_order_field() {
        let store = store(&[
            json!({"smut_id": "a"}),
            json!({"smut_id": "b"}),
            json!({"smut_id": "c"}),
        ]);

        assert_eq!(
            ordered(&store, &[SortKey::descending("smut_id")]).expect("scalar values"),
            ["c", "b", "a"]
        );
    }

    #[test]
    fn order_naming_a_field_absent_from_every_entity_ties_to_id_order() {
        let store = store(&[
            json!({"smut_id": "c", "title": "third"}),
            json!({"smut_id": "a", "title": "first"}),
            json!({"smut_id": "b", "title": "second"}),
        ]);

        assert_eq!(
            ordered(&store, &ascending(&["nowhere"])).expect("nothing to validate"),
            ["a", "b", "c"]
        );
    }

    #[test]
    fn one_and_one_point_zero_tie_for_ordering_and_break_on_id() {
        let store = store(&[
            json!({"smut_id": "float", "value": 1.0}),
            json!({"smut_id": "integer", "value": 1}),
        ]);

        assert_eq!(
            ordered(&store, &ascending(&["value"])).expect("scalar values"),
            ["float", "integer"]
        );
    }

    #[test]
    fn an_array_in_an_order_field_is_rejected_even_with_a_single_candidate() {
        let store = store(&[json!({"smut_id": "only", "tags": ["storage"]})]);

        let rejection = ordered(&store, &ascending(&["tags"])).expect_err("an array cannot sort");

        assert_eq!(
            rejection,
            Reject::UnsortableValue {
                field: "tags".to_owned(),
                found: "an array".to_owned(),
            }
        );
    }

    #[test]
    fn validation_reaches_a_later_key_that_no_comparison_would_consult() {
        let store = store(&[
            json!({"smut_id": "first", "priority": 1, "tags": {"kind": "storage"}}),
            json!({"smut_id": "second", "priority": 2, "tags": {"kind": "graph"}}),
        ]);

        // `priority` decides every comparison, so a lazily validated comparator
        // would never look at `tags` at all.
        let rejection =
            ordered(&store, &ascending(&["priority", "tags"])).expect_err("an object cannot sort");
        assert_eq!(
            rejection,
            Reject::UnsortableValue {
                field: "tags".to_owned(),
                found: "an object".to_owned(),
            }
        );
    }

    #[test]
    fn null_is_a_value_that_sorts_first_and_absence_still_sorts_last() {
        let store = store(&[
            json!({"smut_id": "absent"}),
            json!({"smut_id": "null", "priority": null}),
            json!({"smut_id": "one", "priority": 1}),
        ]);

        assert_eq!(
            ordered(&store, &ascending(&["priority"])).expect("scalar values"),
            ["null", "one", "absent"]
        );
    }
}
