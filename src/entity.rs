//! The canonical serialisation boundary: one entity ordering, two byte forms.
//!
//! §4.2 gives the store one ordering and two byte forms. The ordering is applied
//! here, at the last moment before bytes exist, by walking the value and writing
//! each object's keys in the order §4.2 states. Nothing serialises and then
//! re-parses to arrange keys: `serde_json::Value` would erase the arrangement.

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

/// The entity's public key. The entity root carries it first.
const ENTITY_ID: &str = "id";

/// Which of §4.2's two byte forms a surface needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// Two-space pretty-printed with a trailing newline: entity files,
    /// `index.json` and the journal manifest.
    File,
    /// One compact line with a trailing newline: command stdout.
    Stdout,
}

/// The canonical bytes of one entity for `form`: its root names `id` first.
pub fn serialise_entity(entity: &Value, form: Form) -> String {
    render(entity, form, true)
}

/// The canonical bytes of a store document for `form`: with no entity root, its
/// keys stay lexical.
pub fn serialise_document(document: &Value, form: Form) -> String {
    render(document, form, false)
}

fn render(value: &Value, form: Form, id_first: bool) -> String {
    let canonical = Canonical { value, id_first };

    let rendered = match form {
        Form::File => serde_json::to_string_pretty(&canonical),
        Form::Stdout => serde_json::to_string(&canonical),
    };

    let mut rendered = rendered.expect("serialising a JSON value into memory cannot fail");
    rendered.push('\n');
    rendered
}

/// A value that serialises with §4.2's ordering.
struct Canonical<'a> {
    value: &'a Value,
    /// Only the entity root hoists `id`; embedded ordinary objects stay lexical.
    id_first: bool,
}

impl Serialize for Canonical<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.value {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(flag) => serializer.serialize_bool(*flag),
            Value::Number(number) => number.serialize(serializer),
            Value::String(text) => serializer.serialize_str(text),
            Value::Array(items) => {
                let mut sequence = serializer.serialize_seq(Some(items.len()))?;

                for item in items {
                    sequence.serialize_element(&Canonical {
                        value: item,
                        id_first: false,
                    })?;
                }

                sequence.end()
            }
            Value::Object(fields) => {
                let mut keys: Vec<&str> = fields.keys().map(String::as_str).collect();
                keys.sort_unstable();

                if self.id_first
                    && let Some(position) = keys.iter().position(|key| *key == ENTITY_ID)
                {
                    let id = keys.remove(position);
                    keys.insert(0, id);
                }

                let mut object = serializer.serialize_map(Some(keys.len()))?;

                for key in keys {
                    object.serialize_entry(
                        key,
                        &Canonical {
                            value: &fields[key],
                            id_first: false,
                        },
                    )?;
                }

                object.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const PRETTY_ENTITY: &str =
        "{\n  \"id\": \"smut\",\n  \"body\": \"hello\",\n  \"title\": \"first\"\n}\n";
    const COMPACT_ENTITY: &str = "{\"id\":\"smut\",\"body\":\"hello\",\"title\":\"first\"}\n";

    fn sample() -> Value {
        // Lexically `body` precedes `id`; the root rule has to move `id` ahead of it.
        json!({"title": "first", "body": "hello", "id": "smut"})
    }

    #[test]
    fn the_file_form_is_pretty_and_puts_the_entity_id_first() {
        assert_eq!(serialise_entity(&sample(), Form::File), PRETTY_ENTITY);
    }

    #[test]
    fn the_stdout_form_is_compact_and_puts_the_entity_id_first() {
        assert_eq!(serialise_entity(&sample(), Form::Stdout), COMPACT_ENTITY);
    }

    #[test]
    fn an_embedded_ordinary_object_is_lexical_and_is_not_hoisted() {
        let entity = json!({"id": "a", "note": {"zebra": true, "id": "inner", "alpha": 1}});

        assert_eq!(
            serialise_entity(&entity, Form::Stdout),
            "{\"id\":\"a\",\"note\":{\"alpha\":1,\"id\":\"inner\",\"zebra\":true}}\n"
        );
    }

    #[test]
    fn arrays_keep_their_order() {
        let entity = json!({"id": "a", "tags": ["zulu", "alpha", "mike"]});

        assert_eq!(
            serialise_entity(&entity, Form::Stdout),
            "{\"id\":\"a\",\"tags\":[\"zulu\",\"alpha\",\"mike\"]}\n"
        );
    }

    #[test]
    fn the_root_ordering_holds_in_the_pretty_form_too() {
        let entity = json!({"id": "a", "meta": {"zebra": true, "alpha": 1}, "body": "b"});

        assert_eq!(
            serialise_entity(&entity, Form::File),
            "{\n  \"id\": \"a\",\n  \"body\": \"b\",\n  \"meta\": {\n    \"alpha\": 1,\n    \"zebra\": true\n  }\n}\n"
        );
    }

    #[test]
    fn integers_and_floats_keep_their_own_bytes() {
        let entity = json!({"id": "a", "one": 1, "float": 1.0});

        assert_eq!(
            serialise_entity(&entity, Form::Stdout),
            "{\"id\":\"a\",\"float\":1.0,\"one\":1}\n"
        );
    }

    #[test]
    fn a_document_has_no_entity_root() {
        let index = json!({"objects": {"smut": "Ab3.json"}, "format_version": 1});

        assert_eq!(
            serialise_document(&index, Form::File),
            "{\n  \"format_version\": 1,\n  \"objects\": {\n    \"smut\": \"Ab3.json\"\n  }\n}\n"
        );

        let with_id = json!({"id": "x", "alpha": 1});
        assert_eq!(
            serialise_document(&with_id, Form::Stdout),
            "{\"alpha\":1,\"id\":\"x\"}\n"
        );
    }

    #[test]
    fn a_value_without_an_id_is_rendered_lexically() {
        let value = json!({"beta": 2, "alpha": 1});

        assert_eq!(
            serialise_entity(&value, Form::Stdout),
            "{\"alpha\":1,\"beta\":2}\n"
        );
    }

    #[test]
    fn the_bytes_parse_back_to_the_same_value() {
        let entity = json!({
            "id": "a",
            "body": "line\nquote\" tab\t",
            "nested": [{"z": 1, "a": 2}, null],
            "flag": false,
        });

        for form in [Form::File, Form::Stdout] {
            let rendered = serialise_entity(&entity, form);
            let parsed: Value = serde_json::from_str(&rendered).expect("canonical bytes parse");

            assert_eq!(parsed, entity);
        }
    }

    #[test]
    fn each_form_ends_with_exactly_one_newline() {
        for form in [Form::File, Form::Stdout] {
            let rendered = serialise_entity(&sample(), form);

            assert!(rendered.ends_with("}\n"), "{rendered:?}");
            assert!(!rendered.ends_with("}\n\n"), "{rendered:?}");
        }
    }

    #[test]
    fn an_empty_object_is_still_a_document() {
        assert_eq!(serialise_document(&json!({}), Form::File), "{}\n");
        assert_eq!(serialise_document(&json!({}), Form::Stdout), "{}\n");
    }
}
