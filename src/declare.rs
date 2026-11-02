//! The declaration algebra: what a declare request means.
//!
//! One module, because normalisation, the conflict fold and patch application are
//! one semantic unit (§2). Everything here is a function over an in-memory
//! snapshot: no path, no clock, no entropy, and the only error it can raise is
//! [`Reject`], which is user error and always exit 2 (DEC-001).
//!
//! Equality is `serde_json::Value`'s structural equality with no coercion, and
//! object key order is irrelevant because the maps are `BTreeMap`-backed: `1` and
//! `1.0` are distinct, arrays are ordered, and `null` is an ordinary value that
//! conflicts with a drop (DEC-002).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::entity::{Decl, ENTITY_ID, Op, extract, reference_id, scan_refs};
use crate::error::Reject;
use crate::id::{Id, IdSource, Occupied};

/// The validated snapshot the core works over: every entity, keyed by public id.
///
/// The map is ordered, so iteration order is the id order and never `readdir`'s.
/// Its values are entities the shell has already validated; the core cannot
/// express a storage failure, so it does not re-check them.
pub type Entities = BTreeMap<Id, Value>;

/// A normalised declaration request: what the request says, before it meets the
/// snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    /// The top-level input order, repeats included, with generated ids in place.
    pub ids: Vec<Id>,
    /// Every declaration the request contains, in preorder, one entry per
    /// occurrence.
    pub declarations: Vec<Decl>,
    /// Unresolved references, unique and in encounter order, against the
    /// post-batch id set.
    pub missing: Vec<Id>,
}

/// What a declaration did to one entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The entity did not exist before the batch.
    Created,
    /// The patch leaves the entity structurally different.
    Updated,
    /// The patch leaves the entity structurally equal.
    Unchanged,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Created => "created",
            Status::Updated => "updated",
            Status::Unchanged => "unchanged",
        }
    }
}

/// One entity's entry in a declaration response.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    /// The declared entity.
    pub id: Id,
    /// What happened to it.
    pub status: Status,
    /// The entity as it stood before the whole batch: `Some(Value::Null)` for a
    /// created entity, and `None` where the response omits `previous` (an
    /// unchanged entity).
    pub previous: Option<Value>,
}

/// The declaration response of the brief's contract, in both of its forms.
#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    /// The top-level input order, repeats included.
    pub ids: Vec<Id>,
    /// Each declared entity once, in first-encounter order.
    pub changes: Vec<Change>,
    /// Unresolved references, unique and in encounter order.
    pub missing: Vec<Id>,
}

impl Response {
    /// The response as JSON. `compact` (the `--compact` form) omits `previous`
    /// everywhere; status, ids and `missing` are unchanged.
    pub fn to_json(&self, compact: bool) -> Value {
        Value::Object(Map::from_iter([
            ("ids".to_owned(), id_list(&self.ids)),
            (
                "changes".to_owned(),
                Value::Array(
                    self.changes
                        .iter()
                        .map(|change| change.to_json(compact))
                        .collect(),
                ),
            ),
            ("missing".to_owned(), id_list(&self.missing)),
        ]))
    }
}

impl Change {
    fn to_json(&self, compact: bool) -> Value {
        let mut entry = Map::new();
        entry.insert(
            ENTITY_ID.to_owned(),
            Value::String(self.id.as_str().to_owned()),
        );
        entry.insert(
            "status".to_owned(),
            Value::String(self.status.as_str().to_owned()),
        );

        if !compact && let Some(previous) = &self.previous {
            entry.insert("previous".to_owned(), previous.clone());
        }

        Value::Object(entry)
    }
}

/// The delete response of the brief's contract.
#[derive(Clone, Debug, PartialEq)]
pub struct DeleteResponse {
    /// The ids removed, deduped in first-occurrence order.
    pub deleted: Vec<Id>,
    /// The ids that were not there, deduped in first-occurrence order.
    pub missing: Vec<Id>,
}

impl DeleteResponse {
    /// The response as JSON.
    pub fn to_json(&self) -> Value {
        Value::Object(Map::from_iter([
            ("deleted".to_owned(), id_list(&self.deleted)),
            ("missing".to_owned(), id_list(&self.missing)),
        ]))
    }
}

/// What the shell must write for one batch, in public-id space (DEC-001 rule 6).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WriteSet {
    /// The complete entities to store, in first-encounter order.
    pub put: Vec<Value>,
    /// The public ids to remove, in the order they were decided.
    pub remove: Vec<Id>,
}

impl WriteSet {
    /// Whether the batch asks for no write at all.
    pub fn is_empty(&self) -> bool {
        self.put.is_empty() && self.remove.is_empty()
    }
}

/// A batch's result: the response to print and the write set to commit, and
/// nothing else.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// What the command reports.
    pub response: Response,
    /// What the command writes.
    pub write_set: WriteSet,
}

/// Normalises `input` against `store`'s occupied ids, drawing ids from `source`.
///
/// Shape, identity and hoisting are settled here: a top-level object without `id`
/// receives a generated id unless it is empty; every entity node is hoisted with
/// its occurrence replaced by an id-only reference; a top-level `"DROP"` becomes an
/// operation. References are resolved against the post-batch id set afterwards, so
/// a forward reference is not missing.
///
/// A generated candidate is drawn until it is outside the occupied set — the
/// snapshot's keys, every explicit id anywhere in the request, and every id
/// generated earlier in this batch — so a collision can never become an upsert
/// (DEC-001 rule 3).
pub fn normalize(
    store: &Entities,
    input: &Value,
    source: &mut dyn IdSource,
) -> Result<Batch, Reject> {
    let roots = declaration_roots(input)?;

    let mut occupied = Occupied::of(store.keys());
    occupy_explicit_ids(input, &mut occupied);

    let mut ids = Vec::new();
    let mut declarations = Vec::new();
    let mut encounters = Vec::new();

    for root in roots {
        if !has_id(root) && root.as_object().is_some_and(Map::is_empty) {
            continue;
        }

        let generated;
        let value = if has_id(root) {
            root
        } else {
            let id = occupied.generate(source);
            generated = seeded(root, &id);
            &generated
        };

        let (_, found) = extract(value)?;
        let id = Id::new(
            reference_id(value)?.expect("a declaration root is an entity node or was seeded"),
        );
        ids.push(id.clone());

        if found.is_empty() {
            // Nothing was hoisted, so the root was an id-only reference.
            encounters.push(id);
        } else {
            for declaration in &found {
                for (_, target) in scan_refs(&declaration.entity()) {
                    encounters.push(Id::new(target));
                }
            }
            declarations.extend(found);
        }
    }

    let known: BTreeSet<&str> = store
        .keys()
        .map(Id::as_str)
        .chain(
            declarations
                .iter()
                .map(|declaration| declaration.id.as_str()),
        )
        .collect();
    let mut seen = BTreeSet::new();
    let mut missing = Vec::new();

    for encounter in encounters {
        if known.contains(encounter.as_str()) {
            continue;
        }
        if seen.insert(encounter.clone()) {
            missing.push(encounter);
        }
    }

    Ok(Batch {
        ids,
        declarations,
        missing,
    })
}

/// Folds `batch`'s declarations and applies them against `store`.
///
/// Disjoint fields of one id combine and identical assignments coalesce; a
/// field given two incompatible operations refuses the whole batch. The fold runs
/// before anything else is computed, so a refusal leaves the snapshot untouched
/// and produces no `previous` values to report (DEC-002). `previous` is read from
/// the pre-batch snapshot: the full entity for an update, `null` for a creation,
/// absent for an unchanged entity.
pub fn apply(store: &Entities, batch: Batch) -> Result<Outcome, Reject> {
    let folded = fold(batch.declarations)?;
    let mut changes = Vec::new();
    let mut write_set = WriteSet::default();

    for declaration in folded {
        let existing = store.get(&declaration.id);
        let mut fields = match existing {
            Some(Value::Object(fields)) => fields.clone(),
            Some(_) => panic!("apply requires a validated snapshot: a stored entity is an object"),
            None => Map::new(),
        };
        fields.insert(
            ENTITY_ID.to_owned(),
            Value::String(declaration.id.as_str().to_owned()),
        );

        for (name, op) in &declaration.ops {
            match op {
                Op::Set(value) => {
                    fields.insert(name.clone(), value.clone());
                }
                Op::Drop => {
                    fields.remove(name);
                }
            }
        }

        let entity = Value::Object(fields);
        let (status, previous) = match existing {
            None => {
                write_set.put.push(entity);
                (Status::Created, Some(Value::Null))
            }
            Some(old) if *old == entity => (Status::Unchanged, None),
            Some(old) => {
                write_set.put.push(entity);
                (Status::Updated, Some(old.clone()))
            }
        };

        changes.push(Change {
            id: declaration.id,
            status,
            previous,
        });
    }

    Ok(Outcome {
        response: Response {
            ids: batch.ids,
            changes,
            missing: batch.missing,
        },
        write_set,
    })
}

/// Plans the delete of `ids` against `store`: the response the command prints and
/// the write set that removes them, both deduped in first-occurrence order.
///
/// Incoming references are left untouched, so a delete never cascades: an entity
/// that referenced a removed id keeps the reference, and whatever reads it next
/// reports the target as missing.
pub fn plan_delete(store: &Entities, ids: &[Id]) -> (DeleteResponse, WriteSet) {
    let mut deleted = Vec::new();
    let mut missing = Vec::new();
    let mut seen = BTreeSet::new();

    for id in ids {
        if !seen.insert(id.clone()) {
            continue;
        }
        if store.contains_key(id) {
            deleted.push(id.clone());
        } else {
            missing.push(id.clone());
        }
    }

    let write_set = WriteSet {
        put: Vec::new(),
        remove: deleted.clone(),
    };

    (DeleteResponse { deleted, missing }, write_set)
}

/// Folds every declaration of one id into one declaration per id, in
/// first-encounter order, refusing a field that is given incompatible operations.
fn fold(declarations: Vec<Decl>) -> Result<Vec<Decl>, Reject> {
    let mut order: Vec<Id> = Vec::new();
    let mut folded: BTreeMap<Id, BTreeMap<String, Op>> = BTreeMap::new();

    for Decl { id, ops } in declarations {
        if !folded.contains_key(&id) {
            order.push(id.clone());
        }
        let entry = folded.entry(id.clone()).or_default();

        for (field, op) in ops {
            let merged = match entry.get(&field) {
                None => op,
                Some(left) => {
                    fold_operations(left, &op).ok_or_else(|| Reject::ConflictingDeclaration {
                        id: id.to_string(),
                        field: field.clone(),
                    })?
                }
            };

            entry.insert(field, merged);
        }
    }

    Ok(order
        .into_iter()
        .map(|id| {
            let ops = folded.remove(&id).expect("every ordered id was folded");

            Decl { id, ops }
        })
        .collect())
}

/// The fold's per-field rule: equal assignments and repeated drops coalesce;
/// everything else is a conflict (DEC-002).
fn fold_operations(left: &Op, right: &Op) -> Option<Op> {
    match (left, right) {
        (Op::Set(value), Op::Set(other)) if value == other => Some(Op::Set(value.clone())),
        (Op::Drop, Op::Drop) => Some(Op::Drop),
        _ => None,
    }
}

/// The request's roots: one object, or an array of objects.
fn declaration_roots(input: &Value) -> Result<Vec<&Value>, Reject> {
    match input {
        Value::Object(_) => Ok(vec![input]),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Object(_) => Ok(item),
                _ => Err(Reject::NotADeclaration),
            })
            .collect(),
        _ => Err(Reject::NotADeclaration),
    }
}

fn has_id(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|fields| fields.contains_key(ENTITY_ID))
}

/// Every explicit id in the request, wherever it appears, so a candidate drawn
/// before a later root is read still avoids it (DEC-001 rule 3).
fn occupy_explicit_ids(value: &Value, occupied: &mut Occupied) {
    match value {
        Value::Object(fields) => {
            if let Some(Value::String(id)) = fields.get(ENTITY_ID)
                && !id.is_empty()
            {
                occupied.occupy(Id::new(id.clone()));
            }

            for (_, child) in fields {
                occupy_explicit_ids(child, occupied);
            }
        }
        Value::Array(items) => {
            for item in items {
                occupy_explicit_ids(item, occupied);
            }
        }
        _ => {}
    }
}

fn seeded(root: &Value, id: &Id) -> Value {
    let mut fields = root
        .as_object()
        .cloned()
        .expect("a declaration root is an object");
    fields.insert(ENTITY_ID.to_owned(), Value::String(id.as_str().to_owned()));

    Value::Object(fields)
}

fn id_list(ids: &[Id]) -> Value {
    Value::Array(
        ids.iter()
            .map(|id| Value::String(id.as_str().to_owned()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Reject;
    use crate::id::Id;
    use crate::id::scripted::ScriptedSource;
    use serde_json::{Value, json};

    fn snapshot(entities: &[(&str, Value)]) -> Entities {
        entities
            .iter()
            .map(|(id, entity)| (Id::new(*id), entity.clone()))
            .collect()
    }

    fn texts(ids: &[Id]) -> Vec<String> {
        ids.iter().map(Id::to_string).collect()
    }

    fn change_texts(changes: &[Change]) -> Vec<String> {
        changes
            .iter()
            .map(|change| format!("{}:{:?}", change.id, change.status))
            .collect()
    }

    fn normalised(store: &Entities, input: Value, script: &[&str]) -> Batch {
        let mut source = ScriptedSource::new(script);

        normalize(store, &input, &mut source).expect("the request normalises")
    }

    fn applied(store: &Entities, input: Value, script: &[&str]) -> Outcome {
        let batch = normalised(store, input, script);

        apply(store, batch).expect("the batch applies")
    }

    fn rejected_by_apply(store: &Entities, input: Value, script: &[&str]) -> Reject {
        let batch = normalised(store, input, script);

        apply(store, batch).expect_err("the batch is refused")
    }

    #[test]
    fn a_lone_object_is_one_declaration_with_a_generated_id() {
        let outcome = applied(&Entities::new(), json!({"body": "hello"}), &["gen"]);

        assert_eq!(texts(&outcome.response.ids), vec!["gen"]);
        assert_eq!(change_texts(&outcome.response.changes), vec!["gen:Created"]);
        assert!(outcome.response.missing.is_empty());
        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "gen", "body": "hello"})]
        );
        assert!(outcome.write_set.remove.is_empty());
    }

    #[test]
    fn an_array_of_declarations_is_applied_in_input_order() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "body": "1"}, {"body": "2"}]),
            &["gen"],
        );

        assert_eq!(texts(&outcome.response.ids), vec!["a", "gen"]);
        assert_eq!(
            outcome.write_set.put,
            vec![
                json!({"smut_id": "a", "body": "1"}),
                json!({"smut_id": "gen", "body": "2"}),
            ]
        );
    }

    #[test]
    fn omitted_fields_are_preserved_and_supplied_fields_take_effect() {
        let store = snapshot(&[(
            "a",
            json!({"smut_id": "a", "body": "old", "title": "keep", "meta": {"x": 1}}),
        )]);

        let outcome = applied(&store, json!({"smut_id": "a", "body": "new"}), &[]);

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Updated"]);
        assert_eq!(
            outcome.response.changes[0].previous,
            Some(store[&Id::new("a")].clone())
        );
        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "a", "body": "new", "title": "keep", "meta": {"x": 1}})]
        );
    }

    #[test]
    fn an_array_or_embedded_object_replaces_wholesale() {
        let store = snapshot(&[(
            "a",
            json!({"smut_id": "a", "tags": ["a", "b"], "meta": {"x": 1, "y": 2}}),
        )]);

        let outcome = applied(
            &store,
            json!({"smut_id": "a", "tags": ["c"], "meta": {"y": 9}}),
            &[],
        );

        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "a", "tags": ["c"], "meta": {"y": 9}})]
        );
    }

    #[test]
    fn drop_removes_a_field() {
        let store = snapshot(&[("a", json!({"smut_id": "a", "body": "b", "title": "t"}))]);

        let outcome = applied(&store, json!({"smut_id": "a", "title": "DROP"}), &[]);

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Updated"]);
        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "a", "body": "b"})]
        );
    }

    #[test]
    fn dropping_an_absent_field_is_a_no_op_and_is_not_updated() {
        let store = snapshot(&[("a", json!({"smut_id": "a", "body": "b"}))]);

        let outcome = applied(&store, json!({"smut_id": "a", "title": "DROP"}), &[]);

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Unchanged"]);
        assert_eq!(outcome.response.changes[0].previous, None);
        assert!(outcome.write_set.is_empty());
    }

    #[test]
    fn assigning_a_field_its_current_value_is_unchanged() {
        let store = snapshot(&[("a", json!({"smut_id": "a", "body": "b"}))]);

        let outcome = applied(&store, json!({"smut_id": "a", "body": "b"}), &[]);

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Unchanged"]);
        assert!(outcome.write_set.is_empty());
    }

    #[test]
    fn an_explicit_id_with_only_drops_creates_an_id_only_entity() {
        let outcome = applied(
            &Entities::new(),
            json!({"smut_id": "a", "status": "DROP"}),
            &[],
        );

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Created"]);
        assert_eq!(outcome.response.changes[0].previous, Some(Value::Null));
        assert_eq!(outcome.write_set.put, vec![json!({"smut_id": "a"})]);
    }

    #[test]
    fn a_bare_id_only_object_creates_nothing_and_is_reported_missing() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a"}, {"smut_id": "a"}]),
            &[],
        );

        assert_eq!(texts(&outcome.response.ids), vec!["a", "a"]);
        assert!(outcome.response.changes.is_empty());
        assert_eq!(texts(&outcome.response.missing), vec!["a"]);
        assert!(outcome.write_set.is_empty());
    }

    #[test]
    fn a_reference_is_not_missing_when_the_batch_declares_or_holds_it() {
        let store = snapshot(&[("stored", json!({"smut_id": "stored"}))]);
        let outcome = applied(
            &store,
            json!([{"smut_id": "stored"}, {"smut_id": "a", "refs": [{"smut_id": "b"}]}, {"smut_id": "b", "body": "y"}]),
            &[],
        );

        assert!(
            outcome.response.missing.is_empty(),
            "{:?}",
            outcome.response.missing
        );
        assert_eq!(
            change_texts(&outcome.response.changes),
            vec!["a:Created", "b:Created"]
        );
    }

    #[test]
    fn a_nested_declaration_is_hoisted_and_applied() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "refs": [{"smut_id": "b", "note": {"smut_id": "c", "body": "x"}}]}]),
            &[],
        );

        assert_eq!(
            change_texts(&outcome.response.changes),
            vec!["a:Created", "b:Created", "c:Created"]
        );
        assert_eq!(
            outcome.write_set.put,
            vec![
                json!({"smut_id": "a", "refs": [{"smut_id": "b"}]}),
                json!({"smut_id": "b", "note": {"smut_id": "c"}}),
                json!({"smut_id": "c", "body": "x"}),
            ]
        );
    }

    #[test]
    fn missing_is_unique_and_in_encounter_order() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "x": {"smut_id": "m1"}, "y": [{"smut_id": "m2"}, {"smut_id": "m1"}]}]),
            &[],
        );

        assert_eq!(texts(&outcome.response.missing), vec!["m1", "m2"]);
    }

    #[test]
    fn conflicting_assignments_reject_the_whole_batch() {
        let store = snapshot(&[("a", json!({"smut_id": "a"}))]);

        let reject = rejected_by_apply(
            &store,
            json!([{"smut_id": "a", "n": 1}, {"smut_id": "a", "n": 1.0}]),
            &[],
        );

        assert_eq!(
            reject,
            Reject::ConflictingDeclaration {
                id: "a".to_owned(),
                field: "n".to_owned(),
            },
            "1 and 1.0 are distinct values"
        );
    }

    #[test]
    fn assigning_and_dropping_the_same_field_rejects() {
        let reject = rejected_by_apply(
            &Entities::new(),
            json!([{"smut_id": "a", "body": "x"}, {"smut_id": "a", "body": "DROP"}]),
            &[],
        );

        assert_eq!(
            reject,
            Reject::ConflictingDeclaration {
                id: "a".to_owned(),
                field: "body".to_owned(),
            }
        );
    }

    #[test]
    fn identical_assignments_coalesce() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "body": "x"}, {"smut_id": "a", "body": "x"}]),
            &[],
        );

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Created"]);
        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "a", "body": "x"})]
        );
    }

    #[test]
    fn disjoint_fields_of_one_id_combine() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "body": "x"}, {"smut_id": "a", "title": "t"}]),
            &[],
        );

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Created"]);
        assert_eq!(
            outcome.write_set.put,
            vec![json!({"smut_id": "a", "body": "x", "title": "t"})]
        );
    }

    #[test]
    fn object_key_order_does_not_matter_for_equality() {
        let outcome = applied(
            &Entities::new(),
            json!([
                {"smut_id": "a", "meta": {"p": 1, "q": 2}},
                {"smut_id": "a", "meta": {"q": 2, "p": 1}},
            ]),
            &[],
        );

        assert_eq!(change_texts(&outcome.response.changes), vec!["a:Created"]);
    }

    #[test]
    fn the_two_spellings_of_one_declaration_coalesce() {
        // DEC-002's worked example: the nested declaration and the top-level one
        // normalise to the same thing, so the repeated `refs` assignments agree.
        let outcome = applied(
            &Entities::new(),
            json!([
                {"smut_id": "q", "refs": [{"smut_id": "t", "label": "x"}]},
                {"smut_id": "q", "refs": [{"smut_id": "t"}]},
                {"smut_id": "t", "label": "x"},
            ]),
            &[],
        );

        assert_eq!(
            change_texts(&outcome.response.changes),
            vec!["q:Created", "t:Created"]
        );
        assert_eq!(
            outcome.write_set.put,
            vec![
                json!({"smut_id": "q", "refs": [{"smut_id": "t"}]}),
                json!({"smut_id": "t", "label": "x"})
            ]
        );
    }

    #[test]
    fn a_nested_declaration_inside_a_replacement_array_is_hoisted() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a", "tags": [{"smut_id": "b", "label": "x"}]}, {"smut_id": "b"}]),
            &[],
        );

        assert!(outcome.response.missing.is_empty());
        assert_eq!(
            outcome.write_set.put,
            vec![
                json!({"smut_id": "a", "tags": [{"smut_id": "b"}]}),
                json!({"smut_id": "b", "label": "x"})
            ]
        );
    }

    #[test]
    fn the_response_echoes_top_level_order_and_lists_each_entity_once() {
        let outcome = applied(
            &Entities::new(),
            json!([{"smut_id": "a"}, {"smut_id": "b", "body": "1"}, {"smut_id": "a"}, {"body": "g"}]),
            &["gen"],
        );

        assert_eq!(texts(&outcome.response.ids), vec!["a", "b", "a", "gen"]);
        assert_eq!(
            change_texts(&outcome.response.changes),
            vec!["b:Created", "gen:Created"]
        );
    }

    #[test]
    fn the_compact_form_omits_previous_while_status_ids_and_missing_hold() {
        let store = snapshot(&[("a", json!({"smut_id": "a", "body": "old"}))]);
        let outcome = applied(
            &store,
            json!([{"smut_id": "a", "body": "new"}, {"smut_id": "n", "body": "x"}, {"smut_id": "gone"}]),
            &[],
        );

        let full = outcome.response.to_json(false);
        let compact = outcome.response.to_json(true);

        assert_eq!(full["ids"], json!(["a", "n", "gone"]));
        assert_eq!(full["missing"], json!(["gone"]));
        assert_eq!(
            full["changes"][0]["previous"],
            json!({"smut_id": "a", "body": "old"})
        );
        assert_eq!(full["changes"][1]["previous"], Value::Null);
        assert_eq!(full["changes"][0]["status"], json!("updated"));
        assert_eq!(full["changes"][1]["status"], json!("created"));

        for entry in compact["changes"].as_array().expect("changes is an array") {
            assert!(
                entry.get("previous").is_none(),
                "compact omits previous everywhere: {entry}"
            );
        }
        assert_eq!(compact["ids"], full["ids"]);
        assert_eq!(compact["missing"], full["missing"]);
        assert_eq!(
            compact["changes"][0]["smut_id"],
            full["changes"][0]["smut_id"]
        );
        assert_eq!(
            compact["changes"][0]["status"],
            full["changes"][0]["status"]
        );
    }

    #[test]
    fn an_unchanged_entry_omits_previous_in_the_full_form_too() {
        let store = snapshot(&[("a", json!({"smut_id": "a", "body": "b"}))]);
        let outcome = applied(&store, json!({"smut_id": "a", "body": "b"}), &[]);

        let full = outcome.response.to_json(false);

        assert_eq!(full["changes"][0]["status"], json!("unchanged"));
        assert!(full["changes"][0].get("previous").is_none());
    }

    #[test]
    fn an_empty_declaration_is_a_no_op() {
        for input in [json!([]), json!({})] {
            let outcome = applied(&Entities::new(), input.clone(), &[]);

            assert_eq!(
                outcome.response.to_json(false),
                json!({"ids": [], "changes": [], "missing": []}),
                "{input}"
            );
            assert!(outcome.write_set.is_empty(), "{input}");
        }
    }

    #[test]
    fn a_declaration_that_is_not_an_object_is_rejected() {
        for input in [
            json!(null),
            json!(5),
            json!("x"),
            json!([5]),
            json!([["a"]]),
        ] {
            let mut source = ScriptedSource::new(&["gen"]);

            let result = normalize(&Entities::new(), &input, &mut source);

            assert_eq!(result.err(), Some(Reject::NotADeclaration), "{input}");
            assert_eq!(
                source.draws(),
                0,
                "nothing is generated for a refusal: {input}"
            );
        }
    }

    #[test]
    fn an_invalid_id_is_rejected_and_never_generated() {
        for (input, found) in [
            (json!({"smut_id": null}), "null"),
            (json!({"smut_id": 5}), "a number"),
            (json!({"smut_id": ""}), "an empty string"),
            (
                json!([{"smut_id": "a", "meta": {"smut_id": false}}]),
                "a boolean",
            ),
        ] {
            let mut source = ScriptedSource::new(&["gen", "gen"]);

            let result = normalize(&Entities::new(), &input, &mut source);

            assert_eq!(
                result.err(),
                Some(Reject::InvalidId {
                    found: found.to_owned()
                }),
                "{input}"
            );
            assert_eq!(source.draws(), 0, "an invalid id never draws: {input}");
        }
    }

    #[test]
    fn a_generated_id_avoids_every_id_the_snapshot_holds() {
        let store = snapshot(&[("stored", json!({"smut_id": "stored", "body": "b"}))]);
        let mut source = ScriptedSource::new(&["stored", "fresh"]);

        let batch = normalize(&store, &json!({"body": "x"}), &mut source).expect("normalises");

        assert_eq!(texts(&batch.ids), vec!["fresh"]);
        assert_eq!(source.draws(), 2, "the occupancy forced a second draw");
    }

    #[test]
    fn a_generated_id_avoids_an_explicit_id_later_in_the_request() {
        let mut source = ScriptedSource::new(&["taken", "fresh"]);

        let batch = normalize(
            &Entities::new(),
            &json!([{"body": "x"}, {"smut_id": "taken"}]),
            &mut source,
        )
        .expect("normalises");

        assert_eq!(texts(&batch.ids), vec!["fresh", "taken"]);
        assert_eq!(
            source.draws(),
            2,
            "the explicit id was pre-scanned before the first draw"
        );
    }

    #[test]
    fn a_generated_id_avoids_an_earlier_generated_id() {
        let mut source = ScriptedSource::new(&["same", "same", "fresh"]);

        let batch = normalize(
            &Entities::new(),
            &json!([{"body": "1"}, {"body": "2"}]),
            &mut source,
        )
        .expect("normalises");

        assert_eq!(texts(&batch.ids), vec!["same", "fresh"]);
    }

    #[test]
    fn normalisation_is_deterministic_for_a_given_id_source() {
        let input = json!([{"smut_id": "a", "refs": [{"body": "x"}]}, {"smut_id": "b"}]);

        let mut first_source = ScriptedSource::new(&["gen"]);
        let mut second_source = ScriptedSource::new(&["gen"]);

        let first = normalize(&Entities::new(), &input, &mut first_source).expect("normalises");
        let second = normalize(&Entities::new(), &input, &mut second_source).expect("normalises");

        assert_eq!(first, second);
    }

    #[test]
    fn plan_delete_dedupes_in_first_occurrence_order() {
        let store = snapshot(&[
            ("a", json!({"smut_id": "a"})),
            ("b", json!({"smut_id": "b", "refs": [{"smut_id": "a"}]})),
        ]);
        let requested = ["b", "gone", "a", "b"]
            .iter()
            .map(|id| Id::new(*id))
            .collect::<Vec<_>>();

        let (response, write_set) = plan_delete(&store, &requested);

        assert_eq!(texts(&response.deleted), vec!["b", "a"]);
        assert_eq!(texts(&response.missing), vec!["gone"]);
        assert_eq!(texts(&write_set.remove), vec!["b", "a"]);
        assert!(write_set.put.is_empty());
    }

    #[test]
    fn plan_delete_of_nothing_is_empty() {
        let (response, write_set) = plan_delete(&Entities::new(), &[]);

        assert_eq!(response.to_json(), json!({"deleted": [], "missing": []}));
        assert!(write_set.is_empty());
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use crate::id::Id;
    use crate::id::scripted::ScriptedSource;
    use proptest::prelude::*;
    use serde_json::{Value, json};

    /// A field value from the shapes the algebra treats differently: `null`, an
    /// integer, a float that equals it numerically, strings, the drop sentinel, a
    /// nested reference and a nested declaration.
    fn field_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            Just(Value::Null),
            Just(json!(1)),
            Just(json!(1.0)),
            "[a-z]{0,3}".prop_map(|text| json!(text)),
            Just(json!("DROP")),
            Just(json!([{"smut_id": "b"}])),
            Just(json!({"smut_id": "c", "body": "nested"})),
            Just(json!({"k": [1, 2]})),
        ]
    }

    fn field_name() -> impl Strategy<Value = &'static str> {
        prop_oneof![Just("body"), Just("title"), Just("refs")]
    }

    fn declaration() -> impl Strategy<Value = Value> {
        (
            prop_oneof![Just("a"), Just("b"), Just("c")],
            prop::collection::btree_map(field_name(), field_value(), 0..3),
        )
            .prop_map(|(id, fields)| {
                let mut object = Map::new();
                object.insert(ENTITY_ID.to_owned(), json!(id));

                for (name, value) in fields {
                    object.insert(name.to_owned(), value);
                }

                Value::Object(object)
            })
    }

    fn request() -> impl Strategy<Value = Value> {
        prop::collection::vec(declaration(), 0..4).prop_map(Value::Array)
    }

    fn normalised(input: &Value) -> Batch {
        let mut source = ScriptedSource::new(&["g1", "g2", "g3", "g4"]);

        normalize(&Entities::new(), input, &mut source).expect("a generated request normalises")
    }

    proptest! {
        #[test]
        fn normalisation_is_deterministic_for_a_given_id_source(input in request()) {
            let mut first_source = ScriptedSource::new(&["g1", "g2", "g3", "g4"]);
            let mut second_source = ScriptedSource::new(&["g1", "g2", "g3", "g4"]);

            let first = normalize(&Entities::new(), &input, &mut first_source)
                .expect("a generated request normalises");
            let second = normalize(&Entities::new(), &input, &mut second_source)
                .expect("a generated request normalises");

            prop_assert_eq!(first, second, "the same request and source give the same batch");
        }

        #[test]
        fn the_fold_coalesces_repeated_declarations(input in request()) {
            let batch = normalised(&input);
            let mut repeated = batch.clone();
            repeated.declarations.extend(batch.declarations.iter().cloned());

            match (apply(&Entities::new(), batch), apply(&Entities::new(), repeated)) {
                (Ok(once), Ok(twice)) => prop_assert_eq!(once, twice),
                (Err(once), Err(twice)) => prop_assert_eq!(once, twice),
                (once, twice) => prop_assert!(
                    false,
                    "repeating the declarations changed the verdict: {once:?} then {twice:?}"
                ),
            }
        }

        #[test]
        fn the_fold_is_idempotent(
            fields in prop::collection::btree_map(field_name(), field_value(), 0..3),
        ) {
            let ops: BTreeMap<String, Op> = fields
                .iter()
                .map(|(name, value)| {
                    let op = if value == &json!("DROP") {
                        Op::Drop
                    } else {
                        Op::Set(value.clone())
                    };

                    ((*name).to_owned(), op)
                })
                .collect();

            let once = fold(vec![Decl { id: Id::new("a"), ops: ops.clone() }])
                .expect("one declaration cannot conflict");
            let repeated = fold(
                (0..3)
                    .map(|_| Decl { id: Id::new("a"), ops: ops.clone() })
                    .collect(),
            )
            .expect("repeats of equal operations coalesce");

            prop_assert_eq!(once, repeated);
        }

        #[test]
        fn reapplying_a_batch_leaves_every_entity_unchanged(input in request()) {
            let batch = normalised(&input);
            let Ok(outcome) = apply(&Entities::new(), batch.clone()) else {
                return Ok(());
            };
            let stored: Entities = outcome
                .write_set
                .put
                .iter()
                .map(|entity| {
                    let id = Id::new(
                        entity[ENTITY_ID]
                            .as_str()
                            .expect("a written entity carries its public id"),
                    );

                    (id, entity.clone())
                })
                .collect();
            let again = apply(&stored, batch).expect("a batch that applied applies again");

            for change in &again.response.changes {
                prop_assert_eq!(change.status, Status::Unchanged, "{} stayed changed", change.id);
            }
            prop_assert!(again.write_set.is_empty(), "re-applying wrote {:?}", again.write_set);
        }
    }
}
