//! Ingress: the one JSON operand, or stdin, and the strict parse behind it.

use std::cell::RefCell;
use std::fmt;
use std::io::Read;
use std::path::PathBuf;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use crate::error::{Error, Reject, StoreError};

/// What the shell does about a command's one JSON input (DEC-008).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action<'a> {
    /// Parse this operand.
    Operand(&'a str),
    /// Read the whole of stdin, then parse it.
    Stdin,
    /// Refuse: there is no operand and stdin is an interactive terminal.
    Refuse,
}

/// The pure operand/tty decision, so every branch is testable without a
/// terminal.
///
/// `-` means stdin even on a terminal, because the user asked for it explicitly.
pub fn decide(operand: Option<&str>, stdin_is_terminal: bool) -> Action<'_> {
    match operand {
        Some("-") => Action::Stdin,
        Some(text) => Action::Operand(text),
        None if stdin_is_terminal => Action::Refuse,
        None => Action::Stdin,
    }
}

/// Reads stdin to its end.
///
/// A read that fails is a runtime failure (exit 1); bytes that are not text are
/// invalid input (exit 2), because no JSON value can be recovered from them.
pub fn read_stdin(reader: &mut dyn Read) -> Result<String, Error> {
    let mut bytes = Vec::new();

    reader
        .read_to_end(&mut bytes)
        .map_err(|source| StoreError::FileSystem {
            step: "read_stdin",
            path: PathBuf::from("<stdin>"),
            source,
        })?;

    String::from_utf8(bytes).map_err(|_| {
        Reject::MalformedJson {
            reason: "the input is not UTF-8".to_owned(),
        }
        .into()
    })
}

/// Parses one JSON value from `text`, refusing a duplicate key as it is read.
///
/// The duplicate check is inside the visitor, not a scan of the built value: each
/// object's keys are checked as `next_key` yields them, so the second assignment
/// is refused before any `Value` holds the first one. It recurses, because every
/// nested value is deserialized with the same seed.
pub fn parse(text: &str) -> Result<Value, Reject> {
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_str(text);

    let seed = Strict {
        duplicate: &duplicate,
    };

    let value = match seed.deserialize(&mut deserializer) {
        Ok(value) => value,
        Err(error) => {
            return Err(match duplicate.into_inner() {
                Some(key) => Reject::DuplicateKey { key },
                None => Reject::MalformedJson {
                    reason: error.to_string(),
                },
            });
        }
    };

    deserializer.end().map_err(|_| Reject::TrailingInput)?;

    Ok(value)
}

/// A JSON value's deserializer, carrying the object key that turned out to be a
/// duplicate out of serde's error type.
struct Strict<'a> {
    duplicate: &'a RefCell<Option<String>>,
}

impl<'de> DeserializeSeed<'de> for Strict<'_> {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Strict<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("a JSON number is finite"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let duplicate = self.duplicate;
        let mut items = Vec::new();

        while let Some(item) = access.next_element_seed(Strict { duplicate })? {
            items.push(item);
        }

        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let duplicate = self.duplicate;
        let mut fields = Map::new();

        while let Some(key) = access.next_key::<String>()? {
            if fields.contains_key(&key) {
                *duplicate.borrow_mut() = Some(key);

                return Err(de::Error::custom("a duplicate key"));
            }

            let value = access.next_value_seed(Strict { duplicate })?;
            fields.insert(key, value);
        }

        Ok(Value::Object(fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn an_operand_is_parsed_directly() {
        assert_eq!(
            decide(Some("{\"id\":\"a\"}"), false),
            Action::Operand("{\"id\":\"a\"}")
        );
    }

    #[test]
    fn an_operand_is_parsed_even_on_a_terminal() {
        assert_eq!(decide(Some("{}"), true), Action::Operand("{}"));
    }

    #[test]
    fn a_dash_reads_stdin() {
        assert_eq!(decide(Some("-"), false), Action::Stdin);
    }

    #[test]
    fn a_dash_reads_stdin_even_on_a_terminal() {
        assert_eq!(decide(Some("-"), true), Action::Stdin);
    }

    #[test]
    fn an_absent_operand_on_a_terminal_is_refused() {
        assert_eq!(decide(None, true), Action::Refuse);
    }

    #[test]
    fn an_absent_operand_on_a_pipe_reads_stdin() {
        assert_eq!(decide(None, false), Action::Stdin);
    }

    #[test]
    fn one_json_value_parses_with_its_types_intact() {
        let value =
            parse("{\"id\":\"a\",\"n\":1,\"f\":1.0,\"b\":true,\"z\":null,\"l\":[1,\"x\",{}]}")
                .expect("the fixture is one JSON value");

        assert_eq!(
            value,
            json!({"id": "a", "n": 1, "f": 1.0, "b": true, "z": null, "l": [1, "x", {}]})
        );
        assert_ne!(
            parse("1").expect("a number"),
            parse("1.0").expect("a number")
        );
    }

    #[test]
    fn surrounding_whitespace_is_not_trailing_input() {
        assert_eq!(
            parse(" \n\t{\"id\": \"a\"}\n \t").expect("whitespace around one value"),
            json!({"id": "a"})
        );
    }

    #[test]
    fn a_duplicate_key_at_the_top_level_is_refused_naming_it() {
        assert_eq!(
            parse("{\"id\":\"a\",\"title\":\"first\",\"title\":\"second\"}"),
            Err(Reject::DuplicateKey {
                key: "title".to_owned()
            })
        );
    }

    #[test]
    fn a_duplicate_key_inside_an_array_item_is_refused() {
        assert_eq!(
            parse("[{\"id\":\"a\",\"title\":\"first\",\"title\":\"second\"}]"),
            Err(Reject::DuplicateKey {
                key: "title".to_owned()
            })
        );
    }

    #[test]
    fn a_duplicate_key_inside_an_embedded_object_is_refused() {
        assert_eq!(
            parse("{\"id\":\"a\",\"meta\":{\"k\":1,\"k\":2}}"),
            Err(Reject::DuplicateKey {
                key: "k".to_owned()
            })
        );
    }

    #[test]
    fn distinct_keys_at_distinct_depths_are_not_duplicates() {
        assert!(
            parse("{\"id\":\"a\",\"meta\":{\"id\":\"b\"},\"list\":[{\"id\":\"c\"}]}").is_ok(),
            "the same key in a different object is not a duplicate"
        );
    }

    #[test]
    fn trailing_input_after_the_value_is_refused() {
        for text in ["{\"id\":\"a\"}{}", "{\"id\":\"a\"}x", "{} {}", "[] []"] {
            assert_eq!(parse(text), Err(Reject::TrailingInput), "{text}");
        }
    }

    #[test]
    fn no_json_value_at_all_is_malformed() {
        for text in ["", "   ", "\n"] {
            assert!(
                matches!(parse(text), Err(Reject::MalformedJson { .. })),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_broken_value_is_malformed() {
        for text in ["nope", "{", "{\"id\":}", "[1,]", "{\"a\" 1}"] {
            assert!(
                matches!(parse(text), Err(Reject::MalformedJson { .. })),
                "{text}"
            );
        }
    }

    #[test]
    fn stdin_is_read_to_its_end() {
        let mut source: &[u8] = b"{\"id\":\"a\"}\n";

        assert_eq!(
            read_stdin(&mut source).expect("a readable pipe"),
            "{\"id\":\"a\"}\n"
        );
    }

    #[test]
    fn stdin_that_is_not_text_is_malformed_rather_than_a_runtime_failure() {
        let mut source: &[u8] = &[0x7b, 0xff, 0x7d];
        let error = read_stdin(&mut source).expect_err("invalid UTF-8");

        assert!(
            matches!(error, Error::Reject(Reject::MalformedJson { .. })),
            "{error}"
        );
    }
}
