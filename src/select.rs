//! Selection: roots → order → limit, and the projection and envelope downstream.
//!
//! A [`Selection`] borrows the snapshot, so it carries full entities while the
//! ordered id list stays the only thing a renderer may walk (DEC-007). The same
//! engine serves `query` and `get`: `get` is selection with explicit roots, no
//! fields, no keys and no cap at all, which is why it cannot truncate
//! (DEC-001 rules 4 and 8).
//!
//! Nothing here touches the filesystem: the envelope is a pure function of a
//! selection, so query and get tests need no store on disk.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::declare::Entities;
use crate::entity::ENTITY_ID;
use crate::error::Reject;
use crate::id::Id;
use crate::order::{self, SortKey};
use crate::spec::{self, Limit, Spec};

/// An ordered, deduped, post-limit view of the snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct Selection<'a> {
    /// The selected ids: ordered, deduped, already truncated to the limit.
    pub ids: Vec<&'a Id>,
    /// The snapshot itself, for full entities; it never widens [`Self::ids`].
    pub store: &'a Entities,
    /// The fields to project; `None` means every field.
    pub fields: Option<&'a [String]>,
    /// Roots that named no stored entity, in `from` order and unique.
    pub missing: Vec<&'a Id>,
    /// Whether rows were actually omitted.
    pub truncated: bool,
    /// Why rows were omitted, unique and ordered.
    pub reasons: Vec<Reason>,
}

/// Why a selection is incomplete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The result cap omitted rows.
    ResultLimit,
}

impl Reason {
    /// The envelope's spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::ResultLimit => "result_limit",
        }
    }
}

/// Selects from `store` as `spec` asks.
pub fn select<'a>(store: &'a Entities, spec: &'a Spec) -> Result<Selection<'a>, Reject> {
    run(
        store,
        spec.from.as_deref(),
        spec.select.as_deref(),
        &spec.order,
        spec.limit,
    )
}

/// `get`'s constructor: the explicit roots, in order and deduped, with no
/// projection, no ordering and no cap at all.
///
/// It runs the same engine as [`select`]; what it does not do is start from a
/// parsed spec, because a spec can ask for the default limit and `get` must
/// never truncate (DEC-001 rule 4).
pub fn get<'a>(store: &'a Entities, ids: &'a [Id]) -> Selection<'a> {
    run(store, Some(ids), None, &[], Limit::Unlimited)
        .expect("get supplies no sort keys, so no sort value is validated")
}

/// Roots → order → limit. Both public constructors are this one call.
fn run<'a>(
    store: &'a Entities,
    roots: Option<&'a [Id]>,
    fields: Option<&'a [String]>,
    keys: &[SortKey],
    limit: Limit,
) -> Result<Selection<'a>, Reject> {
    let mut ids: Vec<&'a Id> = Vec::new();
    let mut missing: Vec<&'a Id> = Vec::new();
    let mut seen = BTreeSet::new();

    match roots {
        None => ids.extend(store.keys()),
        Some(roots) => {
            for root in roots {
                if !seen.insert(root.as_str()) {
                    continue;
                }

                match store.get_key_value(root) {
                    Some((id, _)) => ids.push(id),
                    None => missing.push(root),
                }
            }
        }
    }

    order::sort_ids(&mut ids, store, keys)?;

    let cap = match limit {
        Limit::Default => Some(spec::DEFAULT_LIMIT),
        Limit::Capped(cap) => Some(cap),
        Limit::Unlimited => None,
    };

    let mut truncated = false;
    let mut reasons = Vec::new();

    if let Some(cap) = cap
        && ids.len() as u64 > cap
    {
        ids.truncate(cap as usize);
        truncated = true;
        reasons.push(Reason::ResultLimit);
    }

    Ok(Selection {
        ids,
        store,
        fields,
        missing,
        truncated,
        reasons,
    })
}

/// Projects each selected entity to a flat, field-filtered object.
///
/// `id` is present whatever `select` says, a selected field the entity lacks is
/// omitted rather than nulled, and a referenced entity is never inlined. The
/// objects are ordinary values: the canonical key order is applied at the byte
/// boundary, so the `select` set cannot influence it.
pub fn project(selection: &Selection<'_>) -> Vec<Value> {
    selection
        .ids
        .iter()
        .map(|id| project_one(selection, id))
        .collect()
}

fn project_one(selection: &Selection<'_>, id: &Id) -> Value {
    let entity = selection
        .store
        .get(id)
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("a selection's ids come from its store, and its entities are objects: {id}")
        });

    let mut object = Map::new();
    object.insert(ENTITY_ID.to_owned(), Value::String(id.as_str().to_owned()));

    match selection.fields {
        None => {
            for (field, value) in entity {
                if field != ENTITY_ID {
                    object.insert(field.clone(), value.clone());
                }
            }
        }
        Some(fields) => {
            for field in fields {
                if field == ENTITY_ID {
                    continue;
                }

                if let Some(value) = entity.get(field) {
                    object.insert(field.clone(), value.clone());
                }
            }
        }
    }

    Value::Object(object)
}

/// The query/get response envelope: the projected objects plus every diagnostic,
/// present even when empty or false.
pub fn envelope(selection: &Selection<'_>) -> Value {
    Value::Object(Map::from_iter([
        ("objects".to_owned(), Value::Array(project(selection))),
        ("missing".to_owned(), id_list(&selection.missing)),
        // Structurally empty in 0.1, so widening it later changes the content
        // and never the shape.
        ("dangling".to_owned(), Value::Array(Vec::new())),
        ("truncated".to_owned(), Value::Bool(selection.truncated)),
        (
            "truncation_reasons".to_owned(),
            Value::Array(
                selection
                    .reasons
                    .iter()
                    .map(|reason| Value::String(reason.as_str().to_owned()))
                    .collect(),
            ),
        ),
    ]))
}

fn id_list(ids: &[&Id]) -> Value {
    Value::Array(
        ids.iter()
            .map(|id| Value::String(id.as_str().to_owned()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    fn store(entities: &[Value]) -> Entities {
        entities
            .iter()
            .map(|entity| {
                let id = entity["id"].as_str().expect("every fixture names its id");
                (Id::new(id), entity.clone())
            })
            .collect()
    }

    /// `count` entities named `id-000`, `id-001`, …
    fn numbered(count: u32) -> Entities {
        let entities: Vec<Value> = (0..count)
            .map(|index| json!({"id": format!("id-{index:03}")}))
            .collect();

        store(&entities)
    }

    fn parse(input: Value) -> Spec {
        spec::parse(&input).expect("the fixture specification is valid")
    }

    fn selected(selection: &Selection<'_>) -> Vec<String> {
        selection.ids.iter().map(|id| id.to_string()).collect()
    }

    fn absent(selection: &Selection<'_>) -> Vec<String> {
        selection.missing.iter().map(|id| id.to_string()).collect()
    }

    fn projected(selection: &Selection<'_>) -> Vec<Value> {
        project(selection)
    }

    fn sample() -> Entities {
        store(&[
            json!({"id": "a", "title": "alpha", "priority": 3}),
            json!({"id": "b", "title": "beta", "priority": 1}),
            json!({"id": "c", "title": "gamma", "priority": 2}),
        ])
    }

    #[test]
    fn roots_follow_from_in_its_own_order_and_dedupe_keeps_the_first_occurrence() {
        let store = sample();
        let spec = parse(json!({"from": ["c", "a", "c", "b"]}));
        let selection = select(&store, &spec).expect("every root exists");

        assert_eq!(selected(&selection), ["c", "a", "b"]);
        assert!(selection.missing.is_empty());
    }

    #[test]
    fn an_absent_from_roots_every_id_in_id_order() {
        let store = sample();
        let spec = parse(json!({}));
        let selection = select(&store, &spec).expect("nothing to sort");

        assert_eq!(selected(&selection), ["a", "b", "c"]);
    }

    #[test]
    fn an_empty_from_selects_nothing_without_error() {
        let store = sample();
        let spec = parse(json!({"from": []}));
        let selection = select(&store, &spec).expect("an empty root list is valid");

        assert!(selection.ids.is_empty());
        assert!(!selection.truncated);
        assert_eq!(
            envelope(&selection),
            json!({
                "objects": [],
                "missing": [],
                "dangling": [],
                "truncated": false,
                "truncation_reasons": []
            })
        );
    }

    #[test]
    fn missing_roots_are_excluded_reported_in_from_order_and_unique() {
        let store = sample();
        let spec = parse(json!({"from": ["gone", "a", "also-gone", "gone", "b"]}));
        let selection = select(&store, &spec).expect("the present roots select");

        assert_eq!(selected(&selection), ["a", "b"]);
        assert_eq!(absent(&selection), ["gone", "also-gone"]);
        assert_eq!(
            envelope(&selection)["missing"],
            json!(["gone", "also-gone"])
        );
    }

    #[test]
    fn missing_roots_never_consume_the_limit() {
        let store = sample();
        let spec = parse(json!({"from": ["gone", "a", "b"], "limit": 2}));
        let selection = select(&store, &spec).expect("the present roots select");

        assert_eq!(selected(&selection), ["a", "b"]);
        assert!(!selection.truncated);
        assert!(selection.reasons.is_empty());
    }

    #[test]
    fn order_replaces_discovery_order_wholesale_and_an_empty_order_keeps_it() {
        let store = sample();
        let discovery = parse(json!({"from": ["c", "a", "b"]}));
        let sorted = parse(json!({"from": ["c", "a", "b"], "order": [{"field": "title"}]}));
        let empty = parse(json!({"from": ["c", "a", "b"], "order": []}));

        assert_eq!(
            selected(&select(&store, &sorted).expect("strings sort")),
            ["a", "b", "c"]
        );
        assert_eq!(
            selected(&select(&store, &empty).expect("nothing to sort")),
            ["c", "a", "b"]
        );
        assert_eq!(
            selected(&select(&store, &discovery).expect("nothing to sort")),
            ["c", "a", "b"]
        );
    }

    #[test]
    fn the_limit_is_applied_after_ordering() {
        let store = sample();
        let spec = parse(json!({"order": [{"field": "priority"}], "limit": 2}));
        let selection = select(&store, &spec).expect("numbers sort");

        assert_eq!(selected(&selection), ["b", "c"]);
        assert!(selection.truncated);
        assert_eq!(selection.reasons, vec![Reason::ResultLimit]);
    }

    #[test]
    fn the_default_limit_is_one_hundred_and_only_a_real_omission_truncates() {
        let exactly = numbered(100);
        let overflow = numbered(101);
        let spec = parse(json!({}));

        let fitting = select(&exactly, &spec).expect("nothing to sort");
        assert_eq!(fitting.ids.len(), 100);
        assert!(!fitting.truncated);
        assert!(fitting.reasons.is_empty());

        let capped = select(&overflow, &spec).expect("nothing to sort");
        assert_eq!(capped.ids.len(), 100);
        assert!(capped.truncated);
        assert_eq!(capped.reasons, vec![Reason::ResultLimit]);
        assert_eq!(
            envelope(&capped)["truncation_reasons"],
            json!(["result_limit"])
        );
        assert_eq!(envelope(&fitting)["truncated"], json!(false));
    }

    #[test]
    fn a_selection_borrows_the_stores_ids_rather_than_copying_them() {
        let store = sample();
        let spec = parse(json!({"from": "b"}));
        let selection = select(&store, &spec).expect("the root exists");
        let stored = store.keys().next().expect("the fixture has entities");

        assert!(!std::ptr::eq(selection.ids[0], stored));
        assert!(std::ptr::eq(
            selection.ids[0],
            store.get_key_value("b").expect("b is stored").0
        ));
    }

    #[test]
    fn an_unsortable_order_value_is_rejected_through_select() {
        let store = store(&[json!({"id": "a", "tags": ["storage"]})]);
        let spec = parse(json!({"order": [{"field": "tags"}]}));

        assert_eq!(
            select(&store, &spec),
            Err(Reject::UnsortableValue {
                field: "tags".to_owned(),
                found: "an array".to_owned(),
            })
        );
    }

    #[test]
    fn project_gives_id_only_for_an_empty_select_and_naming_id_is_a_no_op() {
        let store = store(&[json!({"id": "a", "title": "alpha", "body": "text"})]);

        for fields in [json!([]), json!(["id"])] {
            let spec = parse(json!({"select": fields}));
            let selection = select(&store, &spec).expect("nothing to sort");

            assert_eq!(projected(&selection), [json!({"id": "a"})]);
        }

        let spec = parse(json!({"select": ["title", "id"]}));
        let selection = select(&store, &spec).expect("nothing to sort");

        assert_eq!(
            projected(&selection),
            [json!({"id": "a", "title": "alpha"})]
        );
    }

    #[test]
    fn a_selected_field_the_entity_lacks_is_omitted_not_null() {
        let store = sample();
        let spec = parse(json!({"from": "a", "select": ["title", "body"]}));
        let selection = select(&store, &spec).expect("nothing to sort");
        let objects = projected(&selection);

        assert_eq!(objects, [json!({"id": "a", "title": "alpha"})]);
        assert!(
            !objects[0]
                .as_object()
                .expect("a projected entity is an object")
                .contains_key("body")
        );
    }

    #[test]
    fn the_select_set_never_determines_the_objects_key_order() {
        let store = store(&[json!({"id": "a", "zebra": 1, "alpha": 2, "title": "t"})]);
        let spec = parse(json!({"select": ["zebra", "alpha"]}));
        let selection = select(&store, &spec).expect("nothing to sort");
        let objects = projected(&selection);
        let keys: Vec<&str> = objects[0]
            .as_object()
            .expect("a projected entity is an object")
            .keys()
            .map(String::as_str)
            .collect();

        assert_eq!(keys, ["alpha", "id", "zebra"]);
    }

    #[test]
    fn projection_is_flat_and_never_inlines_a_referenced_entity() {
        let store = store(&[
            json!({"id": "a", "title": "alpha", "questions": [{"id": "b"}]}),
            json!({"id": "b", "title": "beta"}),
        ]);
        let spec = parse(json!({"from": "a"}));
        let selection = select(&store, &spec).expect("nothing to sort");

        assert_eq!(
            projected(&selection),
            [json!({"id": "a", "title": "alpha", "questions": [{"id": "b"}]})]
        );
    }

    #[test]
    fn the_envelope_always_carries_every_diagnostics_field() {
        let store = sample();
        let spec = parse(json!({"from": ["a", "gone"]}));
        let selection = select(&store, &spec).expect("the present root selects");

        assert_eq!(
            envelope(&selection),
            json!({
                "objects": [{"id": "a", "title": "alpha", "priority": 3}],
                "missing": ["gone"],
                "dangling": [],
                "truncated": false,
                "truncation_reasons": []
            })
        );
    }

    #[test]
    fn get_bypasses_the_default_limit_and_never_truncates() {
        let store = numbered(150);
        let roots: Vec<Id> = store.keys().cloned().collect();
        let selection = get(&store, &roots);

        assert_eq!(selection.ids.len(), 150);
        assert!(!selection.truncated);
        assert!(selection.reasons.is_empty());
        assert_eq!(projected(&selection).len(), 150);
    }

    #[test]
    fn get_dedupes_its_roots_and_reports_the_absent_ones() {
        let store = sample();
        let roots = ["c", "a", "c", "gone"].map(Id::new);
        let selection = get(&store, &roots);

        assert_eq!(selected(&selection), ["c", "a"]);
        assert_eq!(absent(&selection), ["gone"]);
        assert!(!selection.truncated);
    }

    #[test]
    fn get_is_select_with_an_explicit_from_and_no_projection() {
        let store = sample();
        let spec = parse(json!({"from": ["c", "a"]}));
        let selected_by_spec = select(&store, &spec).expect("the roots exist");
        let roots = [Id::new("c"), Id::new("a")];
        let selected_by_get = get(&store, &roots);

        assert_eq!(
            envelope(&selected_by_get),
            envelope(&selected_by_spec),
            "one engine, one envelope"
        );
    }
}
