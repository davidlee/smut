//! Selection: roots → where → walk → order → limit, and the projection and
//! envelope downstream.
//!
//! A [`Selection`] borrows the snapshot, so it carries full entities while the
//! ordered id list stays the only thing a renderer may walk (DEC-007). The same
//! engine serves `query` and `get`: `get` is selection with explicit roots, no
//! fields, no keys, no cap at all and no traversal, which is why it cannot
//! truncate (DEC-001 rules 4 and 8) and never carries a [`Walk`].
//!
//! The walk — when the request supplied `follow` or `backlinks` — runs on the
//! snapshot's full entities, after the roots are resolved and filtered by
//! `where`, and before ordering. That is why a `select` hiding a reference field
//! cannot erase connectivity, and why the result limit cannot widen what the walk
//! discovered (§4, §7.1).
//! The envelope's `dangling` member is now the walk's diagnostics and the
//! conditional `traversal` member is its provenance; both are derived here,
//! after the limit, and both read references only through
//! [`crate::entity::scan_refs`].
//!
//! Nothing here touches the filesystem: the envelope is a pure function of a
//! selection, so query and get tests need no store on disk.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::declare::Entities;
use crate::entity::{ENTITY_ID, scan_refs};
use crate::error::Reject;
use crate::id::Id;
use crate::order::{self, SortKey};
use crate::predicate::Predicate;
use crate::spec::{DEFAULT_BUDGET, DEFAULT_DEPTH, Limit, Spec};
use crate::walk::{self, Fields, Walk};

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
    /// The walk this selection came from, `Some` exactly when the request
    /// supplied `follow` or `backlinks`.
    ///
    /// A walk is provenance, never an order: [`Self::ids`] stays the only list a
    /// renderer may walk, and the limit applies after ordering, so carrying a
    /// walk never widens the result past `limit`.
    ///
    /// The envelope and the Markdown notice are pure functions of the selection
    /// (DEC-012), so carrying the walk is what lets either say what a traversal
    /// did; `get` never carries one.
    pub walk: Option<Walk<'a>>,
}

/// Why a selection is incomplete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The traversal budget declined a permitted expansion, so discovery
    /// stopped (§7.3, §8.3).
    TraversalBudget,
    /// The result cap omitted rows.
    ResultLimit,
}

impl Reason {
    /// The envelope's spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::TraversalBudget => "traversal_budget",
            Reason::ResultLimit => "result_limit",
        }
    }
}

/// Selects from `store` as `spec` asks.
pub fn select<'a>(store: &'a Entities, spec: &'a Spec) -> Result<Selection<'a>, Reject> {
    run(
        store,
        Request {
            roots: spec.from.as_deref(),
            predicate: spec.r#where.as_ref(),
            fields: spec.select.as_deref(),
            keys: &spec.order,
            limit: spec.limit,
            follow: spec.follow.as_ref(),
            backlinks: spec.backlinks.as_ref(),
            depth: spec.depth,
            budget: spec.budget,
        },
    )
}

/// `get`'s constructor: the explicit roots, in order and deduped, with no
/// projection, no ordering, no traversal and no cap at all.
///
/// It runs the same engine as [`select`]; what it does not do is start from a
/// parsed spec, because a spec can ask for the default limit and `get` must
/// never truncate (DEC-001 rule 4).
pub fn get<'a>(store: &'a Entities, ids: &'a [Id]) -> Selection<'a> {
    run(
        store,
        Request {
            roots: Some(ids),
            // `get` takes explicit roots and never filters them.
            predicate: None,
            fields: None,
            keys: &[],
            limit: Limit::unlimited(),
            // `get` never traverses: no direction is followed, so no walk runs
            // whatever the depth says.
            follow: None,
            backlinks: None,
            depth: DEFAULT_DEPTH,
            // `get` passes the default budget for coherence only: no walk runs,
            // so no expansion can spend it.
            budget: DEFAULT_BUDGET,
        },
    )
    .expect("get supplies no sort keys, so no sort value is validated")
}

/// Everything one selection request carries.
///
/// One value rather than positional parameters: PHASE-02 added the traversal
/// budget as a field here and PHASE-03 added the root predicate to **this same
/// value** (EX-13), so clippy's `too_many_arguments` is never the design
/// constraint.
/// Every field borrows from the specification the caller holds, so a request
/// copies no request data.
struct Request<'a> {
    roots: Option<&'a [Id]>,
    predicate: Option<&'a Predicate>,
    fields: Option<&'a [String]>,
    keys: &'a [SortKey],
    limit: Limit,
    follow: Option<&'a Fields>,
    backlinks: Option<&'a Fields>,
    depth: u64,
    budget: u64,
}

/// Roots → where → walk → order → limit. Both public constructors are this one
/// call.
fn run<'a>(store: &'a Entities, request: Request<'a>) -> Result<Selection<'a>, Reject> {
    let mut ids: Vec<&'a Id> = Vec::new();
    let mut missing: Vec<&'a Id> = Vec::new();
    let mut seen = BTreeSet::new();

    match request.roots {
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

    // §7.1, §10.1 inv. 2: `where` filters ROOTS ONLY, and it runs here — after
    // the roots are resolved and before the walk, so a filtered-out root starts
    // no traversal. A supplied root that is filtered out is omitted and is NOT
    // `missing`, because `missing` was recorded above. Nothing filters
    // `walked.order`, and this runs once, never again after the walk.
    if let Some(predicate) = request.predicate {
        ids.retain(|id| store.get(*id).is_some_and(|entity| predicate.holds(entity)));
    }

    // A supplied direction is a traversal even when it names nothing to follow,
    // so `follow: []` still walks — and therefore still reports.
    let walk = if request.follow.is_some() || request.backlinks.is_some() {
        let walked = walk::walk(
            store,
            &ids,
            request.follow,
            request.backlinks,
            request.depth,
            request.budget,
        );
        ids = walked.order.clone();

        Some(walked)
    } else {
        None
    };

    order::sort_ids(&mut ids, store, request.keys)?;

    let cap = request.limit.cap();

    let mut truncated = false;
    let mut reasons = Vec::new();

    // §8.3 fixes the order reasons can occur: discovery stopped first, then
    // ordered rows omitted. A walk that is not exhausted truncated nothing, and
    // a depth boundary is never a reason.
    if let Some(walk) = &walk
        && walk.exhausted
    {
        truncated = true;
        reasons.push(Reason::TraversalBudget);
    }

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
        fields: request.fields,
        missing,
        truncated,
        reasons,
        walk,
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
///
/// The five 0.1 members are always there; `traversal` is the one conditional
/// member, present exactly when the selection came from a walk (§8.1).
pub fn envelope(selection: &Selection<'_>) -> Value {
    let mut members = Map::from_iter([
        ("objects".to_owned(), Value::Array(project(selection))),
        ("missing".to_owned(), id_list(&selection.missing)),
        // Structurally empty without a walk, so widening it in 0.2 changed the
        // content and never the shape.
        ("dangling".to_owned(), dangling(selection)),
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
    ]);

    if let Some(traversal) = traversal(selection) {
        members.insert("traversal".to_owned(), traversal);
    }

    Value::Object(members)
}

/// The walk's diagnostics: one triple per requested-field reference to an
/// entity the store does not hold, in encounter order; `[]` when no walk ran.
fn dangling(selection: &Selection<'_>) -> Value {
    let Some(walk) = &selection.walk else {
        return Value::Array(Vec::new());
    };

    Value::Array(
        walk.dangling
            .iter()
            .map(|found| {
                Value::Object(Map::from_iter([
                    (
                        "source".to_owned(),
                        Value::String(found.source.as_str().to_owned()),
                    ),
                    ("field".to_owned(), Value::String(found.field.to_owned())),
                    ("target".to_owned(), Value::String(found.target.to_owned())),
                ]))
            })
            .collect(),
    )
}

/// The conditional `traversal` member: `None` exactly when the request started
/// no walk, which is why `get`, a from/select/order/limit query, a depth-only
/// query and a budget-only query all keep the five 0.1 keys (§8.1).
///
/// Its three views have three different scopes. `depths` and `edges` are
/// **result-scoped** — they are derived from the returned objects after the
/// limit, so an id the limit omitted has no entry — while `frontier` is
/// **walk-scoped**, the walk's own account, which the limit cannot cut (§8.2).
fn traversal<'a>(selection: &Selection<'a>) -> Option<Value> {
    let walk = selection.walk.as_ref()?;

    Some(Value::Object(Map::from_iter([
        ("depths".to_owned(), Value::Object(depths(selection, walk))),
        ("edges".to_owned(), Value::Array(edges(selection))),
        ("frontier".to_owned(), id_list(&walk.frontier)),
    ])))
}

/// One integer per **returned** id: the walk's minimum discovery depth, roots 0.
fn depths<'a>(selection: &Selection<'a>, walk: &Walk<'a>) -> Map<String, Value> {
    selection
        .ids
        .iter()
        .map(|id| {
            // A silently omitted depth would be an envelope lying about a
            // returned object.
            let depth = walk.depths.get(id).copied().unwrap_or_else(|| {
                panic!("every returned id was discovered, so the walk recorded its depth: {id}")
            });

            (id.as_str().to_owned(), Value::from(depth))
        })
        .collect()
}

/// The induced subgraph of the returned objects, in **result order**.
///
/// Derived after the limit, from each returned id's full **stored** entity:
/// every reference [`scan_refs`] reports is a candidate, and a pair is kept only
/// when its target is a returned id too, so dangling targets are never edges. A
/// returned node that was never expanded still contributes, and an edge in a
/// field the traversal never named counts when both endpoints are returned:
/// because the scan reads full entities, a `select` that hides the reference
/// field cannot erase connectivity. Pairs are deduped by
/// `(source, field, target)`, keeping the first occurrence, so one target
/// reachable twice through one field is one edge.
///
/// The trap this order guards (DEC-014): **discovery order is not result
/// order**. `selection.ids` is post-order and post-limit, and `edges` is an
/// account of what was returned, never a second account of the walk.
fn edges<'a>(selection: &Selection<'a>) -> Vec<Value> {
    let returned: BTreeSet<&'a str> = selection.ids.iter().map(|id| id.as_str()).collect();
    let mut edges = Vec::new();
    let mut seen: BTreeSet<(&'a str, &'a str, &'a str)> = BTreeSet::new();

    for id in &selection.ids {
        let entity = selection
            .store
            .get(id.as_str())
            .unwrap_or_else(|| panic!("a selection's ids come from its store: {id}"));

        for (field, target) in scan_refs(entity) {
            if !returned.contains(target) {
                continue;
            }

            if seen.insert((id.as_str(), field, target)) {
                edges.push(Value::Object(Map::from_iter([
                    ("source".to_owned(), Value::String(id.as_str().to_owned())),
                    ("field".to_owned(), Value::String(field.to_owned())),
                    ("target".to_owned(), Value::String(target.to_owned())),
                ])));
            }
        }
    }

    edges
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

    use crate::spec;

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

    /// `count` entities named `id-000`, `id-001`, …
    fn numbered(count: u32) -> Entities {
        let entities: Vec<Value> = (0..count)
            .map(|index| json!({"smut_id": format!("id-{index:03}")}))
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
            json!({"smut_id": "a", "title": "alpha", "priority": 3}),
            json!({"smut_id": "b", "title": "beta", "priority": 1}),
            json!({"smut_id": "c", "title": "gamma", "priority": 2}),
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
        let store = store(&[json!({"smut_id": "a", "tags": ["storage"]})]);
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
    fn project_gives_smut_id_only_for_an_empty_select_and_naming_smut_id_is_a_no_op() {
        let store = store(&[json!({"smut_id": "a", "title": "alpha", "body": "text"})]);

        for fields in [json!([]), json!(["smut_id"])] {
            let spec = parse(json!({"select": fields}));
            let selection = select(&store, &spec).expect("nothing to sort");

            assert_eq!(projected(&selection), [json!({"smut_id": "a"})]);
        }

        let spec = parse(json!({"select": ["title", "smut_id"]}));
        let selection = select(&store, &spec).expect("nothing to sort");

        assert_eq!(
            projected(&selection),
            [json!({"smut_id": "a", "title": "alpha"})]
        );
    }

    #[test]
    fn a_selected_field_the_entity_lacks_is_omitted_not_null() {
        let store = sample();
        let spec = parse(json!({"from": "a", "select": ["title", "body"]}));
        let selection = select(&store, &spec).expect("nothing to sort");
        let objects = projected(&selection);

        assert_eq!(objects, [json!({"smut_id": "a", "title": "alpha"})]);
        assert!(
            !objects[0]
                .as_object()
                .expect("a projected entity is an object")
                .contains_key("body")
        );
    }

    #[test]
    fn the_select_set_never_determines_the_objects_key_order() {
        let store = store(&[json!({"smut_id": "a", "zebra": 1, "alpha": 2, "title": "t"})]);
        let spec = parse(json!({"select": ["zebra", "alpha"]}));
        let selection = select(&store, &spec).expect("nothing to sort");
        let objects = projected(&selection);
        let keys: Vec<&str> = objects[0]
            .as_object()
            .expect("a projected entity is an object")
            .keys()
            .map(String::as_str)
            .collect();

        assert_eq!(keys, ["alpha", "smut_id", "zebra"]);
    }

    #[test]
    fn projection_is_flat_and_never_inlines_a_referenced_entity() {
        let store = store(&[
            json!({"smut_id": "a", "title": "alpha", "questions": [{"smut_id": "b"}]}),
            json!({"smut_id": "b", "title": "beta"}),
        ]);
        let spec = parse(json!({"from": "a"}));
        let selection = select(&store, &spec).expect("nothing to sort");

        assert_eq!(
            projected(&selection),
            [json!({"smut_id": "a", "title": "alpha", "questions": [{"smut_id": "b"}]})]
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
                "objects": [{"smut_id": "a", "title": "alpha", "priority": 3}],
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

    #[test]
    fn the_walks_edges_follow_result_order_not_discovery_order() {
        // DEC-014: the walk discovers the roots in `from` order — `a` then `b` —
        // but `order` returns `b` first, and `edges` is the induced subgraph of
        // the returned objects in *result* order, not a second account of the
        // walk.
        let store = store(&[
            json!({"smut_id": "a", "next": {"smut_id": "c"}, "title": "Zulu"}),
            json!({"smut_id": "b", "next": {"smut_id": "c"}, "title": "Able"}),
            json!({"smut_id": "c", "title": "Middle"}),
        ]);
        let spec = parse(json!({
            "from": ["a", "b"],
            "follow": ["next"],
            "order": [{"field": "title"}]
        }));
        let selection = select(&store, &spec).expect("strings sort");

        assert_eq!(selected(&selection), ["b", "c", "a"]);
        assert_eq!(
            envelope(&selection)["traversal"]["edges"],
            json!([
                {"source": "b", "field": "next", "target": "c"},
                {"source": "a", "field": "next", "target": "c"},
            ])
        );
    }
}
