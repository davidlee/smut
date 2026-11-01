//! The Markdown renderer across the lib boundary (§11.1, DEC-009).
//!
//! These tests drive `Selection` → document the way a command will, and they
//! are the home of the notice's **position**: the completeness notice has to be
//! the document's first block, because bodies are emitted verbatim and a body
//! ending in an unclosed comment or fence absorbs everything after it. A
//! guarantee that stored content can hide is not a guarantee, so the assertions
//! here read the document's first block rather than searching for the notice
//! somewhere in the bytes.

use serde_json::{Value, json};
use smut::declare::Entities;
use smut::id::Id;
use smut::render_markdown::{Options, render};
use smut::select::{Selection, select};
use smut::spec;

const TRUNCATED: &str = "> truncated: result_limit; additional entities omitted";

fn store(entities: &[Value]) -> Entities {
    entities
        .iter()
        .map(|entity| {
            let id = entity["id"].as_str().expect("every fixture names its id");
            (Id::new(id), entity.clone())
        })
        .collect()
}

fn rendered(store: &Entities, specification: Value, options: &Options) -> String {
    let specification = spec::parse(&specification).expect("the fixture specification is valid");
    let selection = select(store, &specification).expect("the fixture selects");

    render(&selection, options)
}

fn selected<'a>(store: &'a Entities, specification: &'a spec::Spec) -> Selection<'a> {
    select(store, specification).expect("the fixture selects")
}

/// The document's first block — where the notice has to be for nothing stored
/// to be able to swallow it. A one-block document ends in the document's own
/// trailing newline, which is not part of the block.
fn first_block(document: &str) -> &str {
    document
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .trim_end_matches('\n')
}

/// The notice is the first block, it is the only one, and no stored byte
/// precedes it.
fn assert_notice_leads(document: &str, notice: &str, body_bytes: &str) {
    assert_eq!(first_block(document), notice, "{document:?}");
    assert!(document.starts_with(notice), "{document:?}");
    assert_eq!(
        document.matches(notice).count(),
        1,
        "the notice renders once, first: {document:?}"
    );
    assert!(
        document
            .find(body_bytes)
            .is_some_and(|stored| document.find(notice).is_some_and(|notice| notice < stored)),
        "every stored byte follows the notice: {document:?}"
    );
}

#[test]
fn the_notice_leads_the_document_above_a_body_ending_in_an_unclosed_comment() {
    let store = store(&[
        json!({
            "id": "hostile",
            "title": "Hostile",
            "body": "Kept raw.\n\n<!-- unclosed comment"
        }),
        json!({"id": "later", "title": "Later", "body": "after"}),
    ]);

    let document = rendered(&store, json!({"limit": 1}), &Options::default());

    assert_notice_leads(&document, TRUNCATED, "<!-- unclosed comment");
    assert!(
        document.contains("<!-- unclosed comment"),
        "the body is still verbatim: {document:?}"
    );
}

#[test]
fn the_notice_leads_the_document_above_a_body_ending_in_an_unclosed_fence() {
    let store = store(&[
        json!({
            "id": "hostile",
            "title": "Hostile",
            "body": "Kept raw.\n\n```\nunclosed fence"
        }),
        json!({"id": "second", "title": "Second", "body": "absorbed"}),
        json!({"id": "third", "title": "Third", "body": "also absorbed"}),
    ]);

    let document = rendered(&store, json!({"limit": 2}), &Options::default());

    assert_notice_leads(&document, TRUNCATED, "```\nunclosed fence");
    assert!(
        document.contains("```\nunclosed fence"),
        "the body is still verbatim: {document:?}"
    );
}

#[test]
fn a_zero_entity_selection_emits_the_missing_notice_and_no_entity_output() {
    let store = store(&[
        json!({"id": "a", "title": "Alpha", "body": "unselected"}),
        json!({"id": "b", "title": "Beta", "body": "unselected too"}),
    ]);
    let specification = spec::parse(&json!({"from": ["gone", "also-gone"]}))
        .expect("the fixture specification is valid");
    let selection = selected(&store, &specification);

    assert!(selection.ids.is_empty());

    let document = render(&selection, &Options::default());
    let missing = "> missing: `gone`, `also-gone`";

    assert_eq!(document, format!("{missing}\n"));
    assert_eq!(first_block(&document), missing);
    assert!(
        !document.contains("Alpha") && !document.contains("unselected"),
        "no entity output: {document:?}"
    );
}

#[test]
fn a_full_selection_renders_identity_heading_body_and_fields_in_order() {
    let store = store(&[
        json!({
            "id": "design",
            "title": "Query design",
            "body": "Keep selection small.",
            "type": "question",
            "tags": ["storage"]
        }),
        json!({"id": "later", "title": "Later"}),
    ]);
    let options = Options {
        show_ids: true,
        show_fields: true,
        ..Options::default()
    };

    assert_eq!(
        rendered(&store, json!({"from": ["design", "later"]}), &options),
        "`design`\n\n\
         ## Query design\n\n\
         Keep selection small.\n\n\
         - **tags:** [\"storage\"]\n\
         - **type:** question\n\n\
         `later`\n\n\
         ## Later\n"
    );
}

#[test]
fn truncation_and_missing_share_one_leading_block() {
    let store = store(&[
        json!({"id": "a", "title": "Alpha"}),
        json!({"id": "b", "title": "Beta"}),
    ]);
    let document = rendered(
        &store,
        json!({"from": ["gone", "a", "b"], "limit": 1}),
        &Options::default(),
    );

    assert_eq!(
        document,
        format!("{TRUNCATED}\n> missing: `gone`\n\n## Alpha\n")
    );
}
