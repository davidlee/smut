//! Markdown rendering of a [`Selection`] (DEC-009).
//!
//! Entities in selection order, one blank line between them; a heading from a
//! selected non-empty string field; the body verbatim; other fields only on
//! request; identity as a safely fenced code span, and the visibility
//! fallback. The completeness notice opens the document, because stored bodies
//! are emitted verbatim and could otherwise absorb it.
//!
//! The document is a flat list of blocks joined by one blank line — the notice
//! block, then each entity's blocks. "One blank line between entities" and "the
//! notice is the first block" are therefore the same join.

use serde_json::{Map, Value};

use crate::entity::ENTITY_ID;
use crate::id::Id;
use crate::select::Selection;

/// The field a heading comes from unless `--heading-field` says otherwise.
pub const DEFAULT_HEADING_FIELD: &str = "title";

/// The level a heading renders at unless `--heading-level` says otherwise.
pub const DEFAULT_HEADING_LEVEL: u8 = 2;

/// The field emitted as its own verbatim block.
const BODY_FIELD: &str = "body";

/// The rendering choices a command's flags produce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The field a heading comes from.
    pub heading_field: String,
    /// The heading's level, repeated as `#` that many times.
    pub heading_level: u8,
    /// Whether fields that are neither the heading nor the body are shown.
    pub show_fields: bool,
    /// Whether identity is shown even when the entity has visible content.
    pub show_ids: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            heading_field: DEFAULT_HEADING_FIELD.to_owned(),
            heading_level: DEFAULT_HEADING_LEVEL,
            show_fields: false,
            show_ids: false,
        }
    }
}

/// Renders `selection` as a Markdown document.
///
/// The document ends with one newline when it has any bytes, and is empty when
/// there is nothing to render. Body bytes reach the document untouched.
pub fn render(selection: &Selection<'_>, options: &Options) -> String {
    let mut blocks = Vec::new();

    // Nothing stored can absorb the notice, because nothing stored precedes it.
    if let Some(notice) = completeness_notice(selection) {
        blocks.push(notice);
    }

    for id in &selection.ids {
        blocks.extend(entity_blocks(selection, options, id));
    }

    finish(blocks)
}

/// One blank line between blocks, and one trailing newline for a non-empty
/// document.
fn finish(blocks: Vec<String>) -> String {
    let mut document = blocks.join("\n\n");

    if !document.is_empty() {
        document.push('\n');
    }

    document
}

fn entity_blocks(selection: &Selection<'_>, options: &Options, id: &Id) -> Vec<String> {
    let entity = entity_of(selection, id);
    let heading = heading(entity, selection, options);
    let body = body(entity, selection, options);

    let mut rendered_specially = Vec::new();

    if heading.is_some() {
        rendered_specially.push(options.heading_field.as_str());
    }

    if body.is_some() {
        rendered_specially.push(BODY_FIELD);
    }

    let attributes = attribute_lines(entity, selection, options, &rendered_specially);
    let visible = heading.is_some()
        || body.as_deref().is_some_and(|text| !text.trim().is_empty())
        || !attributes.is_empty();
    let mut blocks = Vec::new();

    // Identity leads the entity, and is also what the visibility fallback
    // renders: a bodyless entity never silently vanishes.
    if options.show_ids || !visible {
        blocks.push(code_span(id.as_str()));
    }

    if let Some(heading) = heading {
        blocks.push(heading);
    }

    if let Some(body) = body {
        blocks.push(body);
    }

    if !attributes.is_empty() {
        blocks.push(attributes.join("\n"));
    }

    blocks
}

/// The selected entity, which the selection's ids are drawn from.
fn entity_of<'a>(selection: &Selection<'a>, id: &Id) -> &'a Map<String, Value> {
    selection
        .store
        .get(id)
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("a selection's ids come from its store, and its entities are objects: {id}")
        })
}

/// The heading block, when the heading field is selected, a string and
/// non-empty. Newlines inside heading text are structural, so they collapse.
fn heading(
    entity: &Map<String, Value>,
    selection: &Selection<'_>,
    options: &Options,
) -> Option<String> {
    let field = options.heading_field.as_str();

    if !is_selected(selection, field) {
        return None;
    }

    let text = entity.get(field)?.as_str()?;

    if text.is_empty() {
        return None;
    }

    Some(format!(
        "{} {}",
        "#".repeat(usize::from(options.heading_level)),
        collapse_newlines(text)
    ))
}

/// The body block: emitted verbatim, never escaped and never trimmed. A body
/// that is not a string is not a body block at all; a body named as the heading
/// field was already rendered once.
fn body(
    entity: &Map<String, Value>,
    selection: &Selection<'_>,
    options: &Options,
) -> Option<String> {
    if options.heading_field.as_str() == BODY_FIELD || !is_selected(selection, BODY_FIELD) {
        return None;
    }

    let text = entity.get(BODY_FIELD)?.as_str()?;

    if text.is_empty() {
        return None;
    }

    Some(text.to_owned())
}

/// Whether `field` is part of what the selection asked for. `id` is always
/// selected, because identity is not a projected field (DEC-001 rule 8).
fn is_selected(selection: &Selection<'_>, field: &str) -> bool {
    if field == ENTITY_ID {
        return true;
    }

    match selection.fields {
        None => true,
        Some(fields) => fields.iter().any(|name| name == field),
    }
}

/// The `- **name:** value` lines, in the entity's own (lexical, §4.2) field
/// order. `rendered_specially` names the fields already emitted as something
/// other than an attribute, so a body that is not a string stays displayable.
fn attribute_lines(
    entity: &Map<String, Value>,
    selection: &Selection<'_>,
    options: &Options,
    rendered_specially: &[&str],
) -> Vec<String> {
    if !options.show_fields {
        return Vec::new();
    }

    entity
        .iter()
        .filter(|(field, _)| *field != ENTITY_ID && is_selected(selection, field))
        .filter(|(field, _)| !rendered_specially.contains(&field.as_str()))
        .map(|(field, value)| format!("- **{}:** {}", escape_label(field), attribute_value(value)))
        .collect()
}

/// Labels are generated names, not stored content: every ASCII punctuation
/// character is backslash-escaped, which CommonMark renders literally, so a
/// field name cannot end the bold marker or open a block of its own.
fn escape_label(label: &str) -> String {
    let mut escaped = String::with_capacity(label.len());

    for character in label.chars() {
        match character {
            '\n' | '\r' => escaped.push(' '),
            punctuation if punctuation.is_ascii_punctuation() => {
                escaped.push('\\');
                escaped.push(punctuation);
            }
            other => escaped.push(other),
        }
    }

    escaped
}

/// A string value is its own raw text; everything else is compact JSON.
fn attribute_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => {
            serde_json::to_string(other).expect("serialising a JSON value into memory cannot fail")
        }
    }
}

fn collapse_newlines(text: &str) -> String {
    text.replace("\r\n", " ").replace(['\r', '\n'], " ")
}

/// A CommonMark code span: a fence one backtick longer than the longest run
/// inside `content`, padded when the content's own edges would otherwise be
/// stripped by the code-span rule. Public ids are opaque, so a fixed
/// single-backtick span is not a delimiter.
fn code_span(content: &str) -> String {
    let fence = "`".repeat(longest_backtick_run(content) + 1);
    let pad = content.starts_with('`')
        || content.ends_with('`')
        || (content.starts_with(' ')
            && content.ends_with(' ')
            && !content.chars().all(|character| character == ' '));

    if pad {
        format!("{fence} {content} {fence}")
    } else {
        format!("{fence}{content}{fence}")
    }
}

fn longest_backtick_run(content: &str) -> usize {
    content
        .split(|character: char| character != '`')
        .map(str::len)
        .max()
        .unwrap_or_default()
}

/// The completeness notice, as the document's first block: truncation has no row
/// count (DEC-009), and missing ids use the same safe delimiters as identity.
fn completeness_notice(selection: &Selection<'_>) -> Option<String> {
    let mut lines = Vec::new();

    if selection.truncated {
        assert!(
            !selection.reasons.is_empty(),
            "a truncated selection names why rows were omitted"
        );

        let reasons: Vec<&str> = selection
            .reasons
            .iter()
            .map(|reason| reason.as_str())
            .collect();

        lines.push(format!(
            "> truncated: {}; additional entities omitted",
            reasons.join(", ")
        ));
    }

    if !selection.missing.is_empty() {
        let spans: Vec<String> = selection
            .missing
            .iter()
            .map(|id| code_span(id.as_str()))
            .collect();

        lines.push(format!("> missing: {}", spans.join(", ")));
    }

    (!lines.is_empty()).then(|| lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::declare::Entities;
    use crate::select::{get, select};
    use crate::spec;

    fn store(entities: &[Value]) -> Entities {
        entities
            .iter()
            .map(|entity| {
                let id = entity["id"].as_str().expect("every fixture names its id");
                (Id::new(id), entity.clone())
            })
            .collect()
    }

    fn rendered(store: &Entities, ids: &[&str], options: &Options) -> String {
        let roots: Vec<Id> = ids.iter().map(|id| Id::new(*id)).collect();
        let selection = get(store, &roots);

        render(&selection, options)
    }

    /// The entity DEC-009's own example renders.
    fn notebook() -> Entities {
        store(&[json!({
            "id": "design",
            "title": "Query design",
            "body": "Keep selection small.",
            "type": "question",
            "tags": ["storage"]
        })])
    }

    #[test]
    fn a_heading_and_a_verbatim_body_are_the_default_layout() {
        let store = notebook();

        assert_eq!(
            rendered(&store, &["design"], &Options::default()),
            "## Query design\n\nKeep selection small.\n"
        );
    }

    #[test]
    fn entities_render_in_selection_order_separated_by_exactly_one_blank_line() {
        let store = store(&[
            json!({"id": "a", "title": "Alpha"}),
            json!({"id": "b", "title": "Beta", "body": "second"}),
            json!({"id": "c", "title": "Gamma"}),
        ]);

        assert_eq!(
            rendered(&store, &["c", "a", "b"], &Options::default()),
            "## Gamma\n\n## Alpha\n\n## Beta\n\nsecond\n"
        );
    }

    #[test]
    fn the_heading_level_repeats_the_hash() {
        let store = notebook();
        let mut options = Options::default();

        for level in 1..=6 {
            options.heading_level = level;

            assert!(
                rendered(&store, &["design"], &options)
                    .starts_with(&format!("{} Query design", "#".repeat(usize::from(level)))),
                "level {level}"
            );
        }
    }

    #[test]
    fn the_heading_field_follows_the_option() {
        let store = store(&[json!({"id": "a", "title": "Alpha", "name": "Named", "body": "text"})]);
        let options = Options {
            heading_field: "name".to_owned(),
            ..Options::default()
        };

        assert_eq!(rendered(&store, &["a"], &options), "## Named\n\ntext\n");
    }

    #[test]
    fn a_heading_field_the_selection_did_not_ask_for_renders_no_heading() {
        let store = notebook();
        let roots = [Id::new("design")];
        let fields = ["body".to_owned()];
        let mut selection = get(&store, &roots);
        selection.fields = Some(&fields);

        assert_eq!(
            render(&selection, &Options::default()),
            "Keep selection small.\n"
        );
    }

    #[test]
    fn a_newline_inside_heading_text_collapses_to_one_space() {
        let store = store(&[json!({"id": "a", "title": "one\ntwo\r\nthree\rfour"})]);

        assert_eq!(
            rendered(&store, &["a"], &Options::default()),
            "## one two three four\n"
        );
    }

    #[test]
    fn a_heading_needs_a_selected_non_empty_string_field() {
        let store = store(&[
            json!({"id": "empty", "title": ""}),
            json!({"id": "number", "title": 42}),
            json!({"id": "absent", "body": "bodyless heading"}),
        ]);

        assert_eq!(
            rendered(&store, &["empty"], &Options::default()),
            "`empty`\n"
        );
        assert_eq!(
            rendered(&store, &["number"], &Options::default()),
            "`number`\n"
        );
        assert_eq!(
            rendered(&store, &["absent"], &Options::default()),
            "bodyless heading\n"
        );
    }

    #[test]
    fn the_body_is_emitted_verbatim_never_trimmed_and_never_escaped() {
        let store = store(&[json!({
            "id": "a",
            "title": "T",
            "body": "  *raw* [link](x) <b>\n\n**untouched**  "
        })]);

        assert_eq!(
            rendered(&store, &["a"], &Options::default()),
            "## T\n\n  *raw* [link](x) <b>\n\n**untouched**  \n"
        );
    }

    #[test]
    fn a_body_ending_in_newlines_keeps_them() {
        let store = store(&[json!({"id": "a", "title": "T", "body": "text\n\n"})]);

        assert_eq!(
            rendered(&store, &["a"], &Options::default()),
            "## T\n\ntext\n\n\n"
        );
    }

    #[test]
    fn a_non_string_body_is_not_a_body_block() {
        let store = store(&[json!({"id": "a", "title": "T", "body": 42})]);

        assert_eq!(rendered(&store, &["a"], &Options::default()), "## T\n");
    }

    fn fields_shown(options: Options) -> Options {
        Options {
            show_fields: true,
            ..options
        }
    }

    #[test]
    fn other_fields_appear_only_under_show_fields() {
        let store = notebook();

        assert_eq!(
            rendered(&store, &["design"], &Options::default()),
            "## Query design\n\nKeep selection small.\n"
        );
        assert_eq!(
            rendered(&store, &["design"], &fields_shown(Options::default())),
            "## Query design\n\nKeep selection small.\n\n- **tags:** [\"storage\"]\n- **type:** question\n"
        );
    }

    #[test]
    fn a_string_value_is_raw_and_anything_else_is_compact_json() {
        let store = store(&[json!({
            "id": "a",
            "flag": true,
            "list": [1, 2],
            "n": 1.5,
            "nested": {"b": 1},
            "nothing": null,
            "text": "a *raw* string"
        })]);

        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "- **flag:** true\n\
             - **list:** [1,2]\n\
             - **n:** 1.5\n\
             - **nested:** {\"b\":1}\n\
             - **nothing:** null\n\
             - **text:** a *raw* string\n"
        );
    }

    #[test]
    fn a_non_string_body_stays_displayable_as_an_attribute() {
        let store = store(&[json!({"id": "a", "body": 42})]);

        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "- **body:** 42\n"
        );
    }

    #[test]
    fn the_heading_field_is_excluded_only_when_it_rendered_a_heading() {
        let store = store(&[json!({"id": "a", "title": "", "type": "x"})]);

        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "- **title:** \n- **type:** x\n"
        );
    }

    #[test]
    fn the_body_named_as_the_heading_field_renders_once() {
        let store = store(&[json!({
            "id": "a",
            "title": "ignored",
            "body": "B text",
            "type": "x"
        })]);
        let options = fields_shown(Options {
            heading_field: BODY_FIELD.to_owned(),
            ..Options::default()
        });

        assert_eq!(
            rendered(&store, &["a"], &options),
            "## B text\n\n- **title:** ignored\n- **type:** x\n"
        );
    }

    #[test]
    fn field_labels_are_escaped() {
        let store = store(&[json!({
            "id": "a",
            "*bold*": "x",
            "a[b]`c`": "y",
            "two\nlines": "z"
        })]);

        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "- **\\*bold\\*:** x\n\
             - **a\\[b\\]\\`c\\`:** y\n\
             - **two lines:** z\n"
        );
    }

    #[test]
    fn attributes_honour_the_selections_fields() {
        let store = store(&[json!({"id": "a", "title": "T", "type": "x", "body": "text"})]);
        let roots = [Id::new("a")];
        let fields = ["type".to_owned()];
        let mut selection = get(&store, &roots);
        selection.fields = Some(&fields);

        assert_eq!(
            render(&selection, &fields_shown(Options::default())),
            "- **type:** x\n"
        );
    }

    #[test]
    fn the_id_is_never_an_attribute() {
        let store = store(&[json!({"id": "a", "type": "x"})]);

        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "- **type:** x\n"
        );
    }

    fn ids_shown(options: Options) -> Options {
        Options {
            show_ids: true,
            ..options
        }
    }

    #[test]
    fn show_ids_renders_identity_in_addition_to_the_content() {
        let store = store(&[json!({"id": "a", "title": "T", "body": "b", "type": "x"})]);

        assert_eq!(
            rendered(&store, &["a"], &ids_shown(Options::default())),
            "`a`\n\n## T\n\nb\n"
        );
        assert_eq!(
            rendered(&store, &["a"], &ids_shown(fields_shown(Options::default()))),
            "`a`\n\n## T\n\nb\n\n- **type:** x\n"
        );
    }

    #[test]
    fn the_visibility_fallback_renders_identity_when_nothing_else_would_be_visible() {
        let store = store(&[json!({"id": "a"})]);

        assert_eq!(rendered(&store, &["a"], &Options::default()), "`a`\n");
        assert_eq!(
            rendered(&store, &["a"], &fields_shown(Options::default())),
            "`a`\n"
        );
    }

    #[test]
    fn an_empty_or_whitespace_only_body_cannot_suppress_the_fallback() {
        let store = store(&[
            json!({"id": "empty", "body": ""}),
            json!({"id": "blank", "body": "   \n\t "}),
            json!({"id": "titled", "title": "T", "body": "  "}),
        ]);

        assert_eq!(
            rendered(&store, &["empty"], &Options::default()),
            "`empty`\n"
        );
        assert_eq!(
            rendered(&store, &["blank"], &Options::default()),
            "`blank`\n\n   \n\t \n"
        );
        assert_eq!(
            rendered(&store, &["titled"], &Options::default()),
            "## T\n\n  \n"
        );
    }

    #[test]
    fn an_id_with_backticks_gets_a_fence_longer_than_its_longest_run() {
        let store = store(&[
            json!({"id": "a`b"}),
            json!({"id": "a``b"}),
            json!({"id": "`lead"}),
            json!({"id": "trail`"}),
            json!({"id": " spaced ` middle "}),
        ]);

        assert_eq!(rendered(&store, &["a`b"], &Options::default()), "``a`b``\n");
        assert_eq!(
            rendered(&store, &["a``b"], &Options::default()),
            "```a``b```\n"
        );
        assert_eq!(
            rendered(&store, &["`lead"], &Options::default()),
            "`` `lead ``\n"
        );
        assert_eq!(
            rendered(&store, &["trail`"], &Options::default()),
            "`` trail` ``\n"
        );
        assert_eq!(
            rendered(&store, &[" spaced ` middle "], &Options::default()),
            "``  spaced ` middle  ``\n"
        );
    }

    #[test]
    fn each_entity_gets_its_own_span() {
        let store = store(&[json!({"id": "a"}), json!({"id": "b`c"})]);

        assert_eq!(
            rendered(&store, &["a", "b`c"], &Options::default()),
            "`a`\n\n``b`c``\n"
        );
    }

    fn sample() -> Entities {
        store(&[
            json!({"id": "a", "title": "Alpha", "body": "first"}),
            json!({"id": "b", "title": "Beta", "body": "second"}),
            json!({"id": "c", "title": "Gamma", "body": "third"}),
        ])
    }

    #[test]
    fn truncation_leads_the_document_without_a_row_count() {
        let store = sample();
        let specification = spec::parse(&json!({"limit": 1})).expect("the limit is valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");
        let document = render(&selection, &Options::default());

        assert_eq!(
            document,
            "> truncated: result_limit; additional entities omitted\n\n## Alpha\n\nfirst\n"
        );
        assert!(
            !document
                .lines()
                .next()
                .unwrap_or_default()
                .contains(|character: char| { character.is_ascii_digit() }),
            "the notice counts no rows: {document:?}"
        );
    }

    #[test]
    fn the_missing_notice_lists_each_id_as_its_own_span() {
        let store = sample();
        let specification =
            spec::parse(&json!({"from": ["gone", "gone`two"]})).expect("roots are valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");

        assert_eq!(
            render(&selection, &Options::default()),
            "> missing: `gone`, ``gone`two``\n"
        );
    }

    #[test]
    fn both_notices_are_one_leading_block_with_truncation_first() {
        let store = sample();
        let specification = spec::parse(&json!({"from": ["gone", "a", "b"], "limit": 1}))
            .expect("the roots and limit are valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");

        assert_eq!(
            render(&selection, &Options::default()),
            "> truncated: result_limit; additional entities omitted\n\
             > missing: `gone`\n\
             \n\
             ## Alpha\n\
             \n\
             first\n"
        );
    }

    #[test]
    fn a_missing_notice_leads_entity_content() {
        let store = sample();
        let specification = spec::parse(&json!({"from": ["a", "gone"]})).expect("roots are valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");

        assert_eq!(
            render(&selection, &Options::default()),
            "> missing: `gone`\n\n## Alpha\n\nfirst\n"
        );
    }

    #[test]
    fn a_zero_entity_selection_emits_the_missing_notice_and_no_entity_output() {
        let store = sample();
        let specification =
            spec::parse(&json!({"from": ["gone", "also-gone"]})).expect("roots are valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");

        assert_eq!(
            render(&selection, &Options::default()),
            "> missing: `gone`, `also-gone`\n"
        );
    }

    #[test]
    fn a_zero_entity_selection_with_nothing_to_report_renders_nothing() {
        let store = sample();
        let specification = spec::parse(&json!({"from": []})).expect("an empty root list is valid");
        let selection = select(&store, &specification).expect("the fixture specification is valid");

        assert_eq!(render(&selection, &Options::default()), "");
    }
}
