//! The selection layer's integration home (§11.1, DEC-001).
//!
//! These tests drive the public surface a command will drive — spec parsing,
//! selection, projection and the envelope — through the lib target, so the layer
//! is exercised across module boundaries rather than only inside each unit. No
//! filesystem is involved, because none of these modules touch one.

use std::collections::BTreeSet;

use serde_json::{Value, json};
use smut::declare::Entities;
use smut::id::Id;
use smut::select::{Selection, envelope, get, project, select};
use smut::spec;

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
    (0..count)
        .map(|index| {
            let id = format!("id-{index:03}");
            (Id::new(&id), json!({"id": id}))
        })
        .collect()
}

fn parse(input: Value) -> spec::Spec {
    spec::parse(&input).expect("the fixture specification is valid")
}

fn selected<'a>(selection: &'a Selection<'a>) -> Vec<&'a str> {
    selection.ids.iter().map(|id| id.as_str()).collect()
}

/// An envelope's top-level key set, so a conditional member cannot hide.
fn keys(document: &Value) -> BTreeSet<&str> {
    document
        .as_object()
        .expect("an envelope is an object")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn roots_order_limit_and_the_envelope_are_one_pipeline() {
    let store = store(&[
        json!({"id": "a", "title": "alpha", "priority": 2}),
        json!({"id": "b", "title": "beta", "priority": 1}),
        json!({"id": "c", "title": "gamma", "priority": 3}),
    ]);
    let specification = parse(json!({
        "from": ["c", "b", "gone"],
        "select": ["title"],
        "order": [{"field": "priority"}],
        "limit": 2
    }));

    let selection = select(&store, &specification).expect("every sort value is a number");

    // Exactly `limit` rows is not an omission, so the envelope reports no
    // truncation even though a cap was in force.
    assert_eq!(selected(&selection), ["b", "c"]);
    assert_eq!(
        envelope(&selection),
        json!({
            "objects": [
                {"id": "b", "title": "beta"},
                {"id": "c", "title": "gamma"}
            ],
            "missing": ["gone"],
            "dangling": [],
            "truncated": false,
            "truncation_reasons": []
        })
    );
}

#[test]
fn missing_roots_are_reported_without_consuming_the_limit() {
    let store = store(&[json!({"id": "a"}), json!({"id": "b"})]);
    let specification = parse(json!({"from": ["a", "gone", "b"], "limit": 1}));

    let selection = select(&store, &specification).expect("nothing to sort");

    assert_eq!(selected(&selection), ["a"]);
    assert_eq!(
        selection
            .missing
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["gone"]
    );

    let document = envelope(&selection);
    assert_eq!(document["objects"], json!([{"id": "a"}]));
    assert_eq!(document["missing"], json!(["gone"]));
    assert_eq!(document["truncated"], json!(true));
}

#[test]
fn a_zero_entity_selection_still_carries_every_diagnostics_field() {
    let store = store(&[json!({"id": "a"})]);
    let specification = parse(json!({"from": []}));

    let selection = select(&store, &specification).expect("an empty root list is valid");

    assert!(selection.ids.is_empty());
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
fn get_through_the_lib_target_never_truncates() {
    let store = numbered(150);
    let roots: Vec<Id> = store.keys().cloned().collect();

    let selection = get(&store, &roots);

    assert_eq!(selection.ids.len(), 150);
    assert!(!selection.truncated);
    assert!(selection.reasons.is_empty());
    assert_eq!(project(&selection).len(), 150);
    assert_eq!(envelope(&selection)["truncated"], json!(false));
}

#[test]
fn the_public_spec_edge_names_the_release_for_a_key_it_will_accept() {
    let rejection =
        spec::parse(&json!({"where": {"type": "question"}})).expect_err("`where` is 0.2, not 0.1");

    assert!(rejection.to_string().contains("0.2"), "{rejection}");
}

// ── VT-9: the traversal member's scope split (§8.1-§8.3) ────────────────────

#[test]
fn a_traversal_selection_carries_a_traversal_member_that_get_never_has() {
    // §8.1 presence keys on the *request*: a supplied `follow` adds `traversal`.
    // The root reaches a child which reaches a grandchild, so a depth-1 walk
    // discovers the child at the boundary and never meets the grandchild.
    let store = store(&[
        json!({"id": "root", "next": {"id": "child"}}),
        json!({"id": "child", "next": {"id": "grandchild"}}),
        json!({"id": "grandchild"}),
    ]);
    let specification = parse(json!({"from": "root", "follow": ["next"], "depth": 1}));

    let selection = select(&store, &specification).expect("every discovered id is stored");
    let document = envelope(&selection);

    assert_eq!(
        keys(&document),
        BTreeSet::from([
            "dangling",
            "missing",
            "objects",
            "traversal",
            "truncated",
            "truncation_reasons",
        ]),
        "the five 0.1 members plus exactly one conditional member: {document}"
    );

    let traversal = &document["traversal"];
    assert_eq!(
        traversal["depths"],
        json!({"root": 0, "child": 1}),
        "depths is result-scoped, so it names only returned ids: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "root", "field": "next", "target": "child"}]),
        "the induced subgraph of the returned objects: {traversal}"
    );
    assert_eq!(
        traversal["frontier"],
        json!(["child"]),
        "the child sits at the depth boundary: discovered, never expanded: {traversal}"
    );

    // `get` supplies no specification at all, so it can never traverse; 150
    // entities make sure a fabricated cap or walk would show.
    let many = numbered(150);
    let roots: Vec<Id> = many.keys().cloned().collect();
    let fetched = envelope(&get(&many, &roots));

    assert!(
        !fetched
            .as_object()
            .expect("an envelope is an object")
            .contains_key("traversal"),
        "get never traverses, so it has no traversal member: {fetched}"
    );
    assert_eq!(
        keys(&fetched),
        BTreeSet::from([
            "dangling",
            "missing",
            "objects",
            "truncated",
            "truncation_reasons",
        ]),
        "get's envelope is still the 0.1 five: {fetched}"
    );
    assert_eq!(fetched["truncated"], json!(false));
}

#[test]
fn depths_and_edges_are_result_scoped_while_frontier_and_dangling_are_walk_scoped() {
    // One query, all four scopes. The walk reaches `middle` and then `leaf`, and
    // `middle` names an absent target; the limit then cuts the result to the
    // root, so `leaf` is discovered but not returned and the dangling source is
    // an id the caller never sees.
    let store = store(&[
        json!({"id": "root", "next": {"id": "middle"}}),
        json!({"id": "middle", "next": [{"id": "leaf"}, {"id": "gone"}]}),
        json!({"id": "leaf"}),
    ]);
    let specification = parse(json!({
        "from": "root",
        "follow": ["next"],
        "depth": 2,
        "limit": 1
    }));

    let selection = select(&store, &specification).expect("every discovered id is stored");
    let document = envelope(&selection);
    let traversal = &document["traversal"];

    assert_eq!(selected(&selection), ["root"], "the limit cut the result");
    assert_eq!(
        traversal["frontier"],
        json!(["leaf"]),
        "frontier is walk-scoped, so the result limit cannot cut it: {traversal}"
    );
    assert_eq!(
        traversal["depths"],
        json!({"root": 0}),
        "depths is result-scoped: an omitted id has no depth: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([]),
        "edges are induced among returned objects, so a cut edge is not reported: {traversal}"
    );
    assert_eq!(
        document["dangling"],
        json!([{"source": "middle", "field": "next", "target": "gone"}]),
        "dangling is walk-scoped: the source was cut by the limit and is still reported: {document}"
    );
    assert_eq!(document["truncated"], json!(true));
}

#[test]
fn truncation_reasons_stays_unique_and_names_only_the_result_limit() {
    // §8.3 orders reasons as they can occur: `traversal_budget` first, then
    // `result_limit`. This store can only truncate by the result limit, so the
    // earlier position is out of reach here; PHASE-02's
    // `both_reasons_appear_once_in_the_order_they_can_occur` in
    // `tests/traversal.rs` asserts the pair and their order. DEF-1, recorded in
    // the slice notes: PHASE-01 could not assert a reason the budget key could
    // not yet produce, so the re-cut keeps the reachable half rather than
    // deleting the test.
    let store = store(&[
        json!({"id": "root", "next": [{"id": "one"}, {"id": "two"}]}),
        json!({"id": "one"}),
        json!({"id": "two"}),
    ]);
    let specification = parse(json!({"from": "root", "follow": ["next"], "limit": 1}));

    let selection = select(&store, &specification).expect("every discovered id is stored");
    let document = envelope(&selection);
    let reasons = document["truncation_reasons"]
        .as_array()
        .expect("truncation_reasons is an array")
        .iter()
        .map(|reason| reason.as_str().expect("a reason is a string"))
        .collect::<Vec<_>>();

    assert_eq!(reasons, ["result_limit"], "{document}");
    assert_eq!(
        reasons.iter().collect::<BTreeSet<_>>().len(),
        reasons.len(),
        "each reason appears once: {reasons:?}"
    );
}
