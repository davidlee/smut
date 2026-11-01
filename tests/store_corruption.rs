//! The store's refusal behaviour (§5.2, DEC-004), through the public surface.
//!
//! Every corruption the design names is refused with the offending entry named.
//! Nothing is repaired, an index-less store is never initialised over, and a
//! symlinked entry is refused rather than followed. A healthy fixture loads in
//! the same file, so a blanket refusal cannot pass for refusal behaviour.

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
