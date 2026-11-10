//! The store's refusal behaviour (§5.2, DEC-004), through the public surface.
//!
//! Every corruption the design names is refused with the offending entry named.
//! Nothing is repaired, an index-less store is never initialised over, and a
//! symlinked entry is refused rather than followed. An identity nested anywhere
//! below the entity root is validated too: one that is present and not a
//! nonempty string is refused with its path named, while the root keeps its own
//! refusal. A healthy fixture loads in the same file, so a blanket refusal
//! cannot pass for refusal behaviour.
//!
//! Two later boundaries sit beside those refusals. The index here is format 2,
//! and a format-1 index is refused by version alone, before any entity or
//! envelope is read. Ordinary `id` is data at every depth: the same five values
//! (null, empty string, number, array, object) that are corruption under
//! `smut_id` load untouched under `id`, both as a nested object value and as an
//! array element.
//!
//! The structural keys are spelled with the literal wire spelling `"smut_id"`
//! instead of going through `ENTITY_ID`, exactly as PHASE-01 spelled them with
//! the then-current literal `"id"`. The fixture states the contract it is
//! testing rather than following the constant, so a fixture cannot flip silently
//! when the key moves. PHASE-02 respelled the literal spellings here as a
//! reviewed act, in the same change that moved the key and the format version.

use std::fs;

use serde_json::{Value, json};

use smut::error::{Error, StoreError};
use smut::id::Id;
use smut::store::Snapshot;

mod common;
use common::{Scratch, smut};

/// A healthy index: one public id, mapped to one safe basename.
const INDEX: &str =
    "{\n  \"format_version\": 2,\n  \"objects\": {\n    \"alpha\": \"Ab3.json\"\n  }\n}\n";

const ENTITY: &str = "{\"smut_id\":\"alpha\",\"body\":\"a thought\"}";

/// The same one-entity store, declared through the format this build refuses.
const VERSION_1_INDEX: &str = "{\"format_version\":1,\"objects\":{\"alpha\":\"Ab3.json\"}}";

/// An entity that reads as `alpha` under either identity spelling, so the only
/// defect a version-1 fixture carries is its version.
const ENTITY_UNDER_EITHER_KEY: &str = r#"{"smut_id":"alpha","id":"alpha","body":"a thought"}"#;

/// This file's store-writing fixture; the scratch is `tests/common`'s.
impl Scratch {
    fn write(&self, name: &str, contents: &str) {
        fs::create_dir_all(self.store_dir()).expect("the store directory");
        fs::write(self.store_dir().join(name), contents).expect("a store file");
    }
}

/// Build a fixture out of `files` and require that loading it is refused.
fn refuse(files: &[(&str, &str)]) -> StoreError {
    let scratch = Scratch::new();

    for (name, contents) in files {
        scratch.write(name, contents);
    }

    scratch
        .store()
        .load()
        .expect_err("the fixture must be refused")
}

/// Build a fixture out of `files` and require that it loads.
fn loads(files: &[(&str, &str)]) -> Snapshot {
    let scratch = Scratch::new();

    for (name, contents) in files {
        scratch.write(name, contents);
    }

    scratch.store().load().expect("the fixture must load")
}

#[test]
fn a_healthy_store_loads_and_reports_every_entity() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write("Ab3.json", ENTITY);

    let snapshot = scratch.store().load().expect("a healthy store");

    assert_eq!(snapshot.entities().len(), 1);
    assert_eq!(snapshot.index().len(), 1);
}

#[test]
fn the_stores_own_names_and_journal_states_are_explained() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write("Ab3.json", ENTITY);
    scratch.write("lock", "");
    scratch.write("index.json.tmp", "{}");
    fs::create_dir(scratch.store_dir().join("journal.done")).expect("a journal state");

    let snapshot = scratch
        .store()
        .load()
        .expect("everything here is explained");

    assert_eq!(snapshot.entities().len(), 1);
}

#[test]
fn a_malformed_index_is_refused_naming_it() {
    let error = refuse(&[("index.json", "{\"format_version\":2,")]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("index.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_unknown_format_version_is_refused_without_migration() {
    // 3 is not a version this build reads; 2 is. The case no longer uses 2 as
    // its unknown value, because 2 is the format this build reads and writes
    // (PHASE-02/EX-3).
    let error = refuse(&[("index.json", "{\"format_version\":3,\"objects\":{}}")]);

    match &error {
        StoreError::UnknownFormatVersion { found, expected } => {
            assert_eq!(found, "3", "{error}");
            assert_eq!(*expected, 2, "{error}");
        }
        other => panic!("an unknown version is refused by version: {other}"),
    }
    assert!(error.to_string().contains("format_version 3"), "{error}");
    assert!(error.to_string().contains("only 2"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_version_1_index_is_refused_by_version_and_writes_no_envelope() {
    let scratch = Scratch::new();
    scratch.write("index.json", VERSION_1_INDEX);
    scratch.write("Ab3.json", ENTITY_UNDER_EITHER_KEY);

    let error = scratch
        .store()
        .load()
        .expect_err("a version-1 store is refused by its version");

    match &error {
        StoreError::UnknownFormatVersion { found, expected } => {
            assert_eq!(found, "1", "{error}");
            assert_eq!(*expected, 2, "{error}");
        }
        other => panic!("the version-1 index is refused by its version: {other}"),
    }
    let message = error.to_string();
    assert!(message.contains("format_version 1"), "{message}");
    assert!(message.contains("only 2"), "{message}");
    assert_eq!(Error::Store(error).exit_code(), 1);

    // The caller's half: the command fails, naming the version it found, and no
    // success envelope is written (§3, "no success envelope is emitted on
    // failure").
    let ran = smut(&scratch, &["get", "alpha"]);

    assert_eq!(ran.code, 1, "the refusal is exit 1");
    assert_eq!(ran.stdout, "", "no success envelope is written on failure");
    assert!(
        ran.stderr.contains("format_version 1"),
        "the refusal names the version it found: {}",
        ran.stderr
    );
}

#[test]
fn a_duplicate_path_mapping_is_refused_naming_both_ids_and_the_file() {
    let error = refuse(&[(
        "index.json",
        "{\"format_version\":2,\"objects\":{\"alpha\":\"Ab3.json\",\"beta\":\"Ab3.json\"}}",
    )]);

    assert!(
        matches!(error, StoreError::DuplicatePathMapping { .. }),
        "{error}"
    );
    for named in ["Ab3.json", "alpha", "beta"] {
        assert!(error.to_string().contains(named), "{error}");
    }
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_missing_indexed_file_is_refused_naming_it() {
    let error = refuse(&[("index.json", INDEX)]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_id_that_disagrees_with_its_index_key_is_refused_naming_the_file() {
    let error = refuse(&[
        ("index.json", INDEX),
        ("Ab3.json", "{\"smut_id\":\"beta\"}"),
    ]);

    assert!(
        matches!(error, StoreError::IdDisagreement { .. }),
        "{error}"
    );
    for named in ["Ab3.json", "alpha", "beta"] {
        assert!(error.to_string().contains(named), "{error}");
    }
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_unsafe_index_basename_is_refused_naming_it() {
    let error = refuse(&[(
        "index.json",
        "{\"format_version\":2,\"objects\":{\"alpha\":\"../escape.json\"}}",
    )]);

    assert!(
        matches!(error, StoreError::UnsafeBasename { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("../escape.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_unexplained_entry_is_refused_naming_it() {
    let error = refuse(&[
        ("index.json", INDEX),
        ("Ab3.json", ENTITY),
        ("Zq7.json", "{\"smut_id\":\"stray\"}"),
    ]);

    assert!(
        matches!(error, StoreError::UnexplainedEntry { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Zq7.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_index_less_store_that_holds_entity_files_is_refused_not_initialised_over() {
    let scratch = Scratch::new();
    scratch.write("Ab3.json", ENTITY);
    scratch.write("lock", "");

    let error = scratch
        .store()
        .load()
        .expect_err("an index-less store holding an entity is not a fresh store");

    assert!(
        matches!(error, StoreError::UnexplainedEntry { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert_eq!(
        scratch.store_entries(),
        ["Ab3.json", "lock"],
        "the refusal created no index and adopted nothing"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[cfg(unix)]
#[test]
fn a_symlinked_entity_file_is_refused_rather_than_followed() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    let outside = scratch.path().join("outside.json");
    fs::write(&outside, ENTITY).expect("an entity outside the store");
    std::os::unix::fs::symlink(&outside, scratch.store_dir().join("Ab3.json"))
        .expect("a symlinked entity entry");

    let error = scratch
        .store()
        .load()
        .expect_err("a symlink is refused, not resolved");

    assert!(
        matches!(error, StoreError::SymlinkedEntry { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[cfg(unix)]
#[test]
fn a_symlinked_index_is_refused_rather_than_followed() {
    let scratch = Scratch::new();
    scratch.write("Ab3.json", ENTITY);
    let outside = scratch.path().join("outside-index.json");
    fs::write(&outside, INDEX).expect("an index outside the store");
    std::os::unix::fs::symlink(&outside, scratch.store_dir().join("index.json"))
        .expect("a symlinked index");

    let error = scratch
        .store()
        .load()
        .expect_err("a symlinked index is refused, not resolved");

    assert!(
        matches!(error, StoreError::SymlinkedEntry { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("index.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

// ── VT-1: an identity below the root is validated eagerly ────────────────────
//
// Every object below the root is asked `reference_id`, all the way down: an
// identity that is present and is not a nonempty string is corruption, and the
// refusal names the path of the first one. The path is `$` followed by
// JSON-quoted bracket components for object keys and numeric ones for array
// positions. The root keeps its own check and its own refusal, and a store whose
// nested data is entirely valid still loads.

#[test]
fn a_nested_identity_under_an_object_is_refused_naming_its_path() {
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","payload":{"smut_id":null}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"]["smut_id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_nested_identity_under_an_array_is_refused_naming_its_position() {
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","payload":{"tags":["a","b",{"smut_id":""}]}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error.to_string().contains(
            r#"identity at $["payload"]["tags"][2]["smut_id"] must be a nonempty string"#
        ),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);

    // Element 0 is an object with no identity, so it is descended past without
    // changing the position the refusal reports.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","items":[{"name":"first"},{"smut_id":7}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["items"][1]["smut_id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_valid_reference_shaped_object_does_not_stop_the_walk() {
    // `{"smut_id":"opaque"}` is what `reference_id` reads as a reference, so it
    // is the shape that could plausibly be treated as a leaf. It must still be
    // descended into: the malformed identity deeper inside is what is refused.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","payload":{"smut_id":"opaque","meta":{"smut_id":null}}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"]["meta"]["smut_id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);

    // The same below the root, with the valid reference-shaped object at array
    // position 0 and the malformed identity at position 1: a `smut_id` that is
    // itself a container is already invalid at the object carrying it, so the
    // reachable shape is descent past a valid string identity.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","payload":[{"smut_id":"opaque"},{"meta":{"smut_id":[]}}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error.to_string().contains(
            r#"identity at $["payload"][1]["meta"]["smut_id"] must be a nonempty string"#
        ),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_identity_path_escapes_object_keys_and_numbers_array_positions() {
    // Each key here is a distinct escaping case: a dot, a bracket pair, a double
    // quote, a backslash, a control character and the empty key. The path below
    // pins all six components and one numeric array component verbatim.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha",".":{"[x]":{"a\"b":{"a\\b":{"\u0001":{"":[{"smut_id":null}]}}}}}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error.to_string().contains(
            r#"identity at $["."]["[x]"]["a\"b"]["a\\b"]["\u0001"][""][0]["smut_id"] must be a nonempty string"#,
        ),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn the_lexically_first_object_key_reports_the_identity() {
    // Both members are malformed. The fixture writes `zzz` first, so only the
    // map's own lexical iteration order can report `aaa`.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","zzz":{"smut_id":""},"aaa":{"smut_id":[]}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["aaa"]["smut_id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn the_earliest_array_position_reports_the_identity() {
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"smut_id":"alpha","tags":["a",{"smut_id":null},"b",{"smut_id":""}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["tags"][1]["smut_id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_refusal_leaves_the_index_and_entity_bytes_untouched() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write(
        "Ab3.json",
        r#"{"smut_id":"alpha","payload":{"smut_id":null}}"#,
    );
    let index_path = scratch.store_dir().join("index.json");
    let entity_path = scratch.store_dir().join("Ab3.json");
    let index_before = fs::read(&index_path).expect("the index bytes");
    let entity_before = fs::read(&entity_path).expect("the entity bytes");

    scratch
        .store()
        .load()
        .expect_err("the malformed nested identity is refused");

    assert_eq!(
        fs::read(&index_path).expect("the index bytes after the refusal"),
        index_before,
        "the refusal repaired nothing in the index"
    );
    assert_eq!(
        fs::read(&entity_path).expect("the entity bytes after the refusal"),
        entity_before,
        "the refusal repaired nothing in the entity"
    );
    assert_eq!(
        scratch.store_entries(),
        ["Ab3.json", "index.json"],
        "the refusal wrote nothing"
    );
}

#[test]
fn a_malformed_root_identity_is_still_an_id_disagreement() {
    let error = refuse(&[
        ("index.json", INDEX),
        ("Ab3.json", r#"{"smut_id":null,"body":"a thought"}"#),
    ]);

    assert!(
        matches!(error, StoreError::IdDisagreement { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error.to_string().contains("null"),
        "the root's own check reads the root identity member: {error}"
    );
    assert!(
        !error.to_string().contains("identity at"),
        "the root's own check speaks for the root, not the nested rule: {error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_non_object_entity_file_is_still_refused_as_not_an_object() {
    let error = refuse(&[("index.json", INDEX), ("Ab3.json", "[]")]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains("an entity file holds a JSON object"),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_valid_nested_identity_loads() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write(
        "Ab3.json",
        r#"{"smut_id":"alpha","payload":{"smut_id":"opaque","meta":{"smut_id":"deeper"}}}"#,
    );

    let snapshot = scratch
        .store()
        .load()
        .expect("every identity below the root is a nonempty string");

    assert_eq!(snapshot.entities().len(), 1);
    assert_eq!(snapshot.index().len(), 1);
}

#[test]
fn a_nested_object_without_an_identity_loads() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write(
        "Ab3.json",
        r#"{"smut_id":"alpha","payload":{"tags":["a","b"],"meta":{"count":3,"flag":true,"none":null}}}"#,
    );

    let snapshot = scratch
        .store()
        .load()
        .expect("an object without an identity member is ordinary data");

    assert_eq!(snapshot.entities().len(), 1);
}

// ── PHASE-02/VT-3: `smut_id` is structural and ordinary `id` is data ─────────
//
// One entity file, one index, the same five values in the same two positions.
// Under `smut_id` each value is corruption, refused with the bracketed path of
// the member that carries it; under ordinary `id` the same value is data that
// loads and keeps what it held. The path notation is the loader's own: a `$`
// root marker, then a JSON-quoted key component per object step and a numeric
// one per array step, ending at the identity member.

/// The five values a structural identity must refuse: null, empty string,
/// number, array and object.
const MALFORMED_IDENTITY_VALUES: &[&str] = &["null", "\"\"", "7", "[1, 2]", "{\"nested\": true}"];

#[test]
fn an_ordinary_id_at_depth_carries_any_of_those_values_as_data() {
    for literal in MALFORMED_IDENTITY_VALUES {
        let value: Value = serde_json::from_str(literal).expect("a JSON value");

        // `$["payload"]["id"]`: an ordinary id as a nested object's value.
        let entity = json!({"smut_id": "alpha", "payload": {"id": value.clone()}}).to_string();
        let snapshot = loads(&[("index.json", INDEX), ("Ab3.json", entity.as_str())]);
        assert_eq!(
            snapshot.entities().len(),
            1,
            "ordinary id `{literal}` is not identity"
        );
        let stored = snapshot
            .entities()
            .get(&Id::new("alpha"))
            .expect("alpha is stored");
        assert_eq!(
            stored["payload"]["id"], value,
            "the ordinary id keeps its value: {literal}"
        );

        // `$["payload"]["tags"][2]["id"]`: an ordinary id inside an array
        // element, one position past the array's scalar elements.
        let entity = json!({
            "smut_id": "alpha",
            "payload": {"tags": ["a", "b", {"id": value.clone()}]}
        })
        .to_string();
        let snapshot = loads(&[("index.json", INDEX), ("Ab3.json", entity.as_str())]);
        assert_eq!(
            snapshot.entities().len(),
            1,
            "ordinary id `{literal}` is not identity"
        );
        let stored = snapshot
            .entities()
            .get(&Id::new("alpha"))
            .expect("alpha is stored");
        assert_eq!(
            stored["payload"]["tags"][2]["id"], value,
            "the ordinary id keeps its value: {literal}"
        );
    }
}

#[test]
fn a_structural_identity_carrying_any_of_those_values_is_corruption() {
    for literal in MALFORMED_IDENTITY_VALUES {
        let value: Value = serde_json::from_str(literal).expect("a JSON value");

        // The same nested object position as the control above, spelled with
        // the identity key: the identical value is now corruption.
        let entity = json!({"smut_id": "alpha", "payload": {"smut_id": value.clone()}}).to_string();
        let error = refuse(&[("index.json", INDEX), ("Ab3.json", entity.as_str())]);
        assert!(
            matches!(error, StoreError::Corrupt { .. }),
            "{literal}: {error}"
        );
        assert!(error.to_string().contains("Ab3.json"), "{literal}: {error}");
        assert!(
            error
                .to_string()
                .contains(r#"identity at $["payload"]["smut_id"] must be a nonempty string"#),
            "{literal}: {error}"
        );
        assert_eq!(Error::Store(error).exit_code(), 1, "{literal}");

        // The same array element position: the path numbers the element.
        let entity = json!({
            "smut_id": "alpha",
            "payload": {"tags": ["a", "b", {"smut_id": value.clone()}]}
        })
        .to_string();
        let error = refuse(&[("index.json", INDEX), ("Ab3.json", entity.as_str())]);
        assert!(
            matches!(error, StoreError::Corrupt { .. }),
            "{literal}: {error}"
        );
        assert!(
            error.to_string().contains(
                r#"identity at $["payload"]["tags"][2]["smut_id"] must be a nonempty string"#
            ),
            "{literal}: {error}"
        );
        assert_eq!(Error::Store(error).exit_code(), 1, "{literal}");
    }
}

#[test]
fn an_ordinary_id_beside_a_structural_identity_is_not_validated_as_identity() {
    // The root and one nested object both carry a valid `smut_id` and an
    // ordinary `id` whose value is malformed as an identity. Only the
    // structural member is validated, so the entity loads and both ordinary
    // values are kept.
    let entity = json!({
        "smut_id": "alpha",
        "id": {"nested": [null]},
        "payload": {"smut_id": "opaque", "id": ""}
    })
    .to_string();
    let snapshot = loads(&[("index.json", INDEX), ("Ab3.json", entity.as_str())]);

    assert_eq!(snapshot.entities().len(), 1);
    let stored = snapshot
        .entities()
        .get(&Id::new("alpha"))
        .expect("alpha is stored");
    assert_eq!(stored["id"], json!({"nested": [null]}));
    assert_eq!(stored["payload"]["id"], json!(""));
    assert_eq!(stored["payload"]["smut_id"], json!("opaque"));
}
