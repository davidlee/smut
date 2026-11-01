//! The selection layer's integration home (§11.1, DEC-001).
//!
//! These tests drive the public surface a command will drive — spec parsing,
//! selection, projection and the envelope — through the lib target, so the layer
//! is exercised across module boundaries rather than only inside each unit. No
//! filesystem is involved, because none of these modules touch one.

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
