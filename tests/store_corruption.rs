//! The store's refusal behaviour (§5.2, DEC-004), through the public surface.
//!
//! Every corruption the design names is refused with the offending entry named.
//! Nothing is repaired, an index-less store is never initialised over, and a
//! symlinked entry is refused rather than followed. An identity nested anywhere
//! below the entity root is validated too: one that is present and not a
//! nonempty string is refused with its path named, while the root keeps its own
//! refusal. A healthy fixture loads in the same file, so a blanket refusal
//! cannot pass for refusal behaviour.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use smut::error::{Error, StoreError};
use smut::store::Store;

/// A healthy index: one public id, mapped to one safe basename.
const INDEX: &str =
    "{\n  \"format_version\": 1,\n  \"objects\": {\n    \"alpha\": \"Ab3.json\"\n  }\n}\n";

const ENTITY: &str = "{\"id\":\"alpha\",\"body\":\"a thought\"}";

/// A scratch directory holding one store, removed when it is dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-corruption-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a scratch directory");

        Self { path }
    }

    fn store(&self) -> Store {
        Store::in_directory(&self.path)
    }

    fn directory(&self) -> PathBuf {
        self.store().directory()
    }

    fn write(&self, name: &str, contents: &str) {
        fs::create_dir_all(self.directory()).expect("the store directory");
        fs::write(self.directory().join(name), contents).expect("a store file");
    }

    fn listing(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.directory())
            .expect("the store directory")
            .map(|entry| {
                entry
                    .expect("a directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();

        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
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
    fs::create_dir(scratch.directory().join("journal.done")).expect("a journal state");

    let snapshot = scratch
        .store()
        .load()
        .expect("everything here is explained");

    assert_eq!(snapshot.entities().len(), 1);
}

#[test]
fn a_malformed_index_is_refused_naming_it() {
    let error = refuse(&[("index.json", "{\"format_version\":1,")]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("index.json"), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn an_unknown_format_version_is_refused_without_migration() {
    let error = refuse(&[("index.json", "{\"format_version\":2,\"objects\":{}}")]);

    assert!(
        matches!(error, StoreError::UnknownFormatVersion { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("format_version"), "{error}");
    assert!(error.to_string().contains('2'), "{error}");
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_duplicate_path_mapping_is_refused_naming_both_ids_and_the_file() {
    let error = refuse(&[(
        "index.json",
        "{\"format_version\":1,\"objects\":{\"alpha\":\"Ab3.json\",\"beta\":\"Ab3.json\"}}",
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
    let error = refuse(&[("index.json", INDEX), ("Ab3.json", "{\"id\":\"beta\"}")]);

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
        "{\"format_version\":1,\"objects\":{\"alpha\":\"../escape.json\"}}",
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
        ("Zq7.json", "{\"id\":\"stray\"}"),
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
        scratch.listing(),
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
    let outside = scratch.path.join("outside.json");
    fs::write(&outside, ENTITY).expect("an entity outside the store");
    std::os::unix::fs::symlink(&outside, scratch.directory().join("Ab3.json"))
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
    let outside = scratch.path.join("outside-index.json");
    fs::write(&outside, INDEX).expect("an index outside the store");
    std::os::unix::fs::symlink(&outside, scratch.directory().join("index.json"))
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
        ("Ab3.json", r#"{"id":"alpha","payload":{"id":null}}"#),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"]["id"] must be a nonempty string"#),
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
            r#"{"id":"alpha","payload":{"tags":["a","b",{"id":""}]}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"]["tags"][2]["id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);

    // Element 0 is an object with no identity, so it is descended past without
    // changing the position the refusal reports.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"id":"alpha","items":[{"name":"first"},{"id":7}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["items"][1]["id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_valid_reference_shaped_object_does_not_stop_the_walk() {
    // `{"id":"opaque"}` is what `reference_id` reads as an entity node, so it is
    // the shape that could plausibly be treated as a leaf. It must still be
    // descended into: the malformed identity deeper inside is what is refused.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"id":"alpha","payload":{"id":"opaque","meta":{"id":null}}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"]["meta"]["id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);

    // The same below the root, with the valid reference-shaped object at array
    // position 0 and the malformed identity at position 1: an `id` that is
    // itself a container is already invalid at the object carrying it, so the
    // reachable shape is descent past a valid string id.
    let error = refuse(&[
        ("index.json", INDEX),
        (
            "Ab3.json",
            r#"{"id":"alpha","payload":[{"id":"opaque"},{"meta":{"id":[]}}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["payload"][1]["meta"]["id"] must be a nonempty string"#),
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
            r#"{"id":"alpha",".":{"[x]":{"a\"b":{"a\\b":{"\u0001":{"":[{"id":null}]}}}}}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(error.to_string().contains("Ab3.json"), "{error}");
    assert!(
        error.to_string().contains(
            r#"identity at $["."]["[x]"]["a\"b"]["a\\b"]["\u0001"][""][0]["id"] must be a nonempty string"#,
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
            r#"{"id":"alpha","zzz":{"id":""},"aaa":{"id":[]}}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["aaa"]["id"] must be a nonempty string"#),
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
            r#"{"id":"alpha","tags":["a",{"id":null},"b",{"id":""}]}"#,
        ),
    ]);

    assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains(r#"identity at $["tags"][1]["id"] must be a nonempty string"#),
        "{error}"
    );
    assert_eq!(Error::Store(error).exit_code(), 1);
}

#[test]
fn a_refusal_leaves_the_index_and_entity_bytes_untouched() {
    let scratch = Scratch::new();
    scratch.write("index.json", INDEX);
    scratch.write("Ab3.json", r#"{"id":"alpha","payload":{"id":null}}"#);
    let index_path = scratch.directory().join("index.json");
    let entity_path = scratch.directory().join("Ab3.json");
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
        scratch.listing(),
        ["Ab3.json", "index.json"],
        "the refusal wrote nothing"
    );
}

#[test]
fn a_malformed_root_identity_is_still_an_id_disagreement() {
    let error = refuse(&[
        ("index.json", INDEX),
        ("Ab3.json", r#"{"id":null,"body":"a thought"}"#),
    ]);

    assert!(
        matches!(error, StoreError::IdDisagreement { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("Ab3.json"), "{error}");
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
        r#"{"id":"alpha","payload":{"id":"opaque","meta":{"id":"deeper"}}}"#,
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
        r#"{"id":"alpha","payload":{"tags":["a","b"],"meta":{"count":3,"flag":true,"none":null}}}"#,
    );

    let snapshot = scratch
        .store()
        .load()
        .expect("an object without an identity member is ordinary data");

    assert_eq!(snapshot.entities().len(), 1);
}
