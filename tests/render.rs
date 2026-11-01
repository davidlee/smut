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

/// The dangling line's one spelling: a `> dangling: ` prefix, then three
/// single-backtick spans with no spaces around the `.` or around the `->`.
const DANGLING: &str = "> dangling: `root`.`link` -> `gone`";

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

// ── VT-8: the notice's trigger and its dangling lines (§8.3, EX-14) ─────────

#[test]
fn a_dangling_only_result_still_carries_the_notice() {
    // The trigger is truncated **or** a non-empty `missing` **or** a non-empty
    // dangling list: a walk that reported a dangling reference is an incomplete
    // answer even though nothing was omitted from the result.
    let store = store(&[json!({"id": "root", "title": "Root", "link": {"id": "gone"}})]);

    let document = rendered(
        &store,
        json!({"from": "root", "follow": ["link"], "depth": 1}),
        &Options::default(),
    );

    assert_eq!(first_block(&document), DANGLING, "{document:?}");
    assert_notice_leads(&document, DANGLING, "## Root");
}

#[test]
fn the_notice_orders_truncated_missing_and_dangling_lines() {
    // All three triggers at once: the lines appear in the order truncated,
    // missing, dangling, and the whole block is asserted as one string so a
    // reordering cannot pass.
    let store = store(&[
        json!({"id": "root", "title": "Root", "link": {"id": "gone"}}),
        json!({"id": "other", "title": "Other"}),
    ]);

    let document = rendered(
        &store,
        json!({
            "from": ["miss", "root", "other"],
            "limit": 1,
            "follow": ["link"],
            "depth": 1
        }),
        &Options::default(),
    );

    let notice = format!("{TRUNCATED}\n> missing: `miss`\n{DANGLING}");

    assert_eq!(first_block(&document), notice, "{document:?}");
    assert_eq!(
        first_block(&document).lines().count(),
        3,
        "one line per kind, in order: {document:?}"
    );
    assert!(
        document.contains("## Root"),
        "the entity's blocks follow the notice: {document:?}"
    );
}

#[test]
fn each_dangling_triple_prints_once_in_encounter_order() {
    // Two absent targets in one requested array field are two triples, so two
    // lines, in the order the walk met them.
    let store = store(&[json!({
        "id": "root",
        "title": "Root",
        "link": [{"id": "gone-one"}, {"id": "gone-two"}],
    })]);

    let document = rendered(
        &store,
        json!({"from": "root", "follow": ["link"], "depth": 1}),
        &Options::default(),
    );

    let notice = "> dangling: `root`.`link` -> `gone-one`\n\
                  > dangling: `root`.`link` -> `gone-two`";

    assert_eq!(first_block(&document), notice, "{document:?}");
    assert_eq!(
        document.matches("> dangling: ").count(),
        2,
        "one line per triple: {document:?}"
    );
}

#[test]
fn a_complete_traversal_result_carries_no_notice() {
    // A walk that finished inside its depth bound and found every target is a
    // complete answer, so the document opens with the entity's heading.
    let store = store(&[
        json!({"id": "root", "title": "Root", "next": {"id": "child"}}),
        json!({"id": "child", "title": "Child"}),
    ]);

    let document = rendered(
        &store,
        json!({"from": "root", "follow": ["next"], "depth": 2}),
        &Options::default(),
    );

    assert_eq!(
        first_block(&document),
        "## Root",
        "the document starts with the entity's heading: {document:?}"
    );
    assert!(
        !document.contains("dangling")
            && !document.contains("missing")
            && !document.contains("truncated"),
        "no notice at all: {document:?}"
    );
}

// ── PHASE-02 VT-4: the budget reason beside the dangling lines (§8.3) ───────

/// §8.3's truncation line when a walk stopped on its budget: one line, the
/// reason's own spelling, and no row count (DEC-009).
const TRUNCATED_BUDGET: &str = "> truncated: traversal_budget; additional entities omitted";

#[test]
fn a_budget_truncated_walk_leads_with_its_reason_and_keeps_every_dangling_line() {
    // §8.3: the reason joins the notice's truncation line, and the dangling
    // lines follow it, one per triple, in the same safe delimiters. The budget of
    // one expands the root alone, so the walk declines `left` while `right` is
    // still queued, and the root's two absent targets are both reported.
    let entities = store(&[
        json!({
            "id": "root",
            "title": "Root",
            "kids": [{"id": "left"}, {"id": "right"}],
            "link": {"id": "gone-one"},
            "side": {"id": "gone-two"}
        }),
        json!({"id": "left", "title": "Left"}),
        json!({"id": "right", "title": "Right"}),
    ]);

    let document = rendered(
        &entities,
        json!({
            "from": "root",
            "follow": ["kids", "link", "side"],
            "depth": 2,
            "budget": 1
        }),
        &Options::default(),
    );

    let notice = format!(
        "{TRUNCATED_BUDGET}\n\
         > dangling: `root`.`link` -> `gone-one`\n\
         > dangling: `root`.`side` -> `gone-two`"
    );

    assert_eq!(first_block(&document), notice, "{document:?}");
    assert_notice_leads(&document, &notice, "## Root");
    assert_eq!(
        document.matches("> truncated: ").count(),
        1,
        "one truncation line, and it is the notice's first: {document:?}"
    );
    assert_eq!(
        document.matches("> dangling: ").count(),
        2,
        "one line per triple: {document:?}"
    );
    assert!(
        !document.contains("result_limit"),
        "the only reason is the budget: {document:?}"
    );
}
