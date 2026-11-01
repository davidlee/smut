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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            heading_field: DEFAULT_HEADING_FIELD.to_owned(),
            heading_level: DEFAULT_HEADING_LEVEL,
        }
    }
}

/// Renders `selection` as a Markdown document.
///
/// The document ends with one newline when it has any bytes, and is empty when
/// there is nothing to render. Body bytes reach the document untouched.
pub fn render(selection: &Selection<'_>, options: &Options) -> String {
    let mut blocks = Vec::new();

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
    let mut blocks = Vec::new();

    if let Some(heading) = heading(entity, selection, options) {
        blocks.push(heading);
    }

    if let Some(body) = body(entity, selection, options) {
        blocks.push(body);
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

fn collapse_newlines(text: &str) -> String {
    text.replace("\r\n", " ").replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::declare::Entities;
    use crate::select::get;

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

        assert_eq!(rendered(&store, &["empty"], &Options::default()), "");
        assert_eq!(rendered(&store, &["number"], &Options::default()), "");
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
}
