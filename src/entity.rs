//! The canonical serialisation boundary: one entity ordering, two byte forms.
//!
//! §4.2 gives the store one ordering and two byte forms. The ordering is applied
//! here, at the last moment before bytes exist, by walking the value and writing
//! each object's keys in the order §4.2 states. Nothing serialises and then
//! re-parses to arrange keys: `serde_json::Value` would erase the arrangement.

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

use crate::error::Reject;

/// The entity's public key. The entity root carries it first.
pub const ENTITY_ID: &str = "id";

/// The single definition of an entity node (DEC-005): an object with a member
/// `id` whose value is a nonempty string. Presence is the whole test; field count
/// and names are irrelevant.
///
/// `Ok(None)` means the value is not an object carrying `id` at all. An `id` that
/// is present but not a nonempty string is rejected, never read as "no id". The
/// error's class is the caller's: the request path reports it as user error, and
/// the store's loader reports it as corruption (DEC-005).
pub fn reference_id(value: &Value) -> Result<Option<&str>, Reject> {
    let Some(fields) = value.as_object() else {
        return Ok(None);
    };
    let Some(id) = fields.get(ENTITY_ID) else {
        return Ok(None);
    };

    match id {
        Value::String(text) if !text.is_empty() => Ok(Some(text)),
        other => Err(Reject::InvalidId {
            found: describe_id(other),
        }),
    }
}

/// `true` exactly when the value is an entity node: the boolean projection of
/// [`reference_id`], for callers that only branch on it.
pub fn is_reference(value: &Value) -> bool {
    matches!(reference_id(value), Ok(Some(_)))
}

/// Every reference occurrence reachable from an entity's non-`id` fields, as
/// `(label, target)` in encounter order.
///
/// The label is the entity's own field name, carried through embedded objects and
/// arrays; an occurrence that is itself an entity node is emitted and not
/// descended into, because the entity it names is scanned in its own right
/// (DEC-005). Objects iterate lexically, arrays positionally.
///
/// The input must be a validated, normalised entity: an entity node's `id` is
/// always a nonempty string, and a malformed one here is corruption the loader
/// should have refused.
pub fn scan_refs(entity: &Value) -> impl Iterator<Item = (&str, &str)> {
    let mut found = Vec::new();

    if let Some(fields) = entity.as_object() {
        for (label, child) in fields {
            if label != ENTITY_ID {
                collect_refs(label, child, &mut found);
            }
        }
    }

    found.into_iter()
}

fn collect_refs<'a>(label: &'a str, value: &'a Value, found: &mut Vec<(&'a str, &'a str)>) {
    match value {
        Value::Object(fields) => {
            match fields.get(ENTITY_ID) {
                Some(Value::String(target)) if !target.is_empty() => {
                    found.push((label, target));
                    return;
                }
                Some(_) => panic!(
                    "scan_refs requires a validated entity: an `id` that is not a nonempty string is corruption"
                ),
                None => {}
            }

            for (_, child) in fields {
                collect_refs(label, child, found);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_refs(label, item, found);
            }
        }
        _ => {}
    }
}

fn describe_id(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(_) => "a boolean".to_owned(),
        Value::Number(_) => "a number".to_owned(),
        Value::String(_) => "an empty string".to_owned(),
        Value::Array(_) => "an array".to_owned(),
        Value::Object(_) => "an object".to_owned(),
    }
}

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

#[cfg(test)]
mod reference_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_id_only_object_is_a_reference() {
        let value = json!({"id": "a"});

        assert_eq!(reference_id(&value), Ok(Some("a")));
        assert!(is_reference(&value));
    }

    #[test]
    fn an_id_plus_fields_is_an_entity_node_too() {
        let value = json!({"id": "a", "body": "text"});

        assert_eq!(reference_id(&value), Ok(Some("a")));
        assert!(is_reference(&value));
    }

    #[test]
    fn an_object_without_an_id_is_not_an_entity_node() {
        let value = json!({"body": "text", "nested": {"id": "b"}});

        assert_eq!(reference_id(&value), Ok(None));
        assert!(!is_reference(&value));
    }

    #[test]
    fn a_scalar_is_not_an_entity_node() {
        for value in [json!(null), json!(1), json!("a"), json!([{"id": "a"}])] {
            assert_eq!(reference_id(&value), Ok(None), "{value}");
            assert!(!is_reference(&value), "{value}");
        }
    }

    #[test]
    fn a_malformed_id_is_rejected_rather_than_treated_as_absent() {
        for (value, found) in [
            (json!({"id": null}), "null"),
            (json!({"id": 5}), "a number"),
            (json!({"id": ""}), "an empty string"),
            (json!({"id": true}), "a boolean"),
            (json!({"id": ["a"]}), "an array"),
            (json!({"id": {"x": 1}}), "an object"),
        ] {
            assert_eq!(
                reference_id(&value),
                Err(Reject::InvalidId {
                    found: found.to_owned()
                }),
                "{value}"
            );
        }
    }

    #[test]
    fn a_whitespace_only_id_is_a_valid_opaque_id() {
        assert_eq!(reference_id(&json!({"id": " "})), Ok(Some(" ")));
    }

    #[test]
    fn scan_refs_labels_each_reference_with_the_containing_entitys_field() {
        let entity = json!({"id": "a", "refs": [{"id": "b"}], "note": {"id": "c"}});

        let found: Vec<(&str, &str)> = scan_refs(&entity).collect();

        assert_eq!(found, vec![("note", "c"), ("refs", "b")]);
    }

    #[test]
    fn scan_refs_carries_the_label_through_arrays_and_embedded_objects() {
        let entity = json!({"id": "a", "links": [{"to": {"id": "b"}}, {"id": "c"}, "plain"]});

        let found: Vec<(&str, &str)> = scan_refs(&entity).collect();

        assert_eq!(found, vec![("links", "b"), ("links", "c")]);
    }

    #[test]
    fn scan_refs_stops_at_a_reference_occurrence() {
        let entity = json!({"id": "a", "refs": {"id": "b", "nested": {"id": "c"}}});

        let found: Vec<(&str, &str)> = scan_refs(&entity).collect();

        assert_eq!(found, vec![("refs", "b")]);
    }

    #[test]
    fn scan_refs_ignores_the_entitys_own_id_and_embedded_values() {
        let entity = json!({"id": "a", "body": "text", "meta": {"x": 1, "y": [1, 2]}});

        assert_eq!(scan_refs(&entity).count(), 0);
        assert_eq!(scan_refs(&json!({"id": "a"})).count(), 0);
    }
}
