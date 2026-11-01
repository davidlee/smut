//! The store index: `format_version` 1, public ids mapped to safe basenames.
//!
//! The index is the only thing that explains a file in `.smut/`, so it is read
//! strictly (§4.1, §5.2): an unknown format version, a malformed document, a
//! duplicated mapping or an unsafe basename is refused rather than repaired.
//!
//! The basename rule holds on both sides. [`Index::parse`] applies it to what a
//! document claims; [`Index::new`] and [`Index::insert`] apply it to what a
//! caller builds, so no construction can serialise a basename the reader would
//! refuse — the write half is a constructor's rule, not only a parser's.
//!
//! Nothing here writes. Reading is the whole of this module's filesystem work:
//! publication belongs to the commit path, so exactly one code path creates an
//! index (§5.1).

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value};

use crate::entity::{Form, serialise_document};
use crate::error::StoreError;
use crate::id::Id;
use crate::store::fs;
use crate::store::{INDEX_BASENAME, JOURNAL_STATES, LOCK_BASENAME, MANIFEST_BASENAME, read_entry};

/// The index format this build reads and writes.
pub const FORMAT_VERSION: u64 = 1;

/// The temporary suffix §4.1 reserves, whether or not the protocol generates the
/// name: `fs::is_protocol_temporary` is the narrower namespace it writes.
const TEMPORARY_SUFFIX: &str = ".tmp";

const FORMAT_VERSION_KEY: &str = "format_version";
const OBJECTS_KEY: &str = "objects";

/// Why `basename` is not one safe path component inside `.smut/`, if it is not.
///
/// Safe means: a single component, not `.` or `..`, not one of the store's own
/// names, and not in the protocol's temporary namespace (§4.1).
pub fn unsafe_basename(basename: &str) -> Option<&'static str> {
    if basename.is_empty() {
        return Some("a basename cannot be empty");
    }
    if basename.contains(['/', '\\']) {
        return Some("a basename cannot hold a path separator");
    }
    if basename == "." || basename == ".." {
        return Some("a basename cannot be `.` or `..`");
    }
    if basename == LOCK_BASENAME {
        return Some("the store's lock is a reserved name");
    }
    if basename == INDEX_BASENAME {
        return Some("the store index is a reserved name");
    }
    if basename == MANIFEST_BASENAME {
        return Some("a journal manifest is a reserved name");
    }
    if JOURNAL_STATES.contains(&basename) {
        return Some("a journal state is a reserved name");
    }
    if fs::is_protocol_temporary(basename) || basename.ends_with(TEMPORARY_SUFFIX) {
        return Some("the protocol's temporary namespace is reserved");
    }

    None
}

/// The committed index: every public id, and the basename that holds it.
///
/// Ordered by public id, so the index's bytes never come from `readdir`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index {
    objects: BTreeMap<Id, String>,
}

impl Index {
    /// An index that maps nothing: a store whose first commit has not happened.
    pub fn empty() -> Self {
        Index::default()
    }

    /// The index of `objects`, refusing any basename the reader would refuse.
    pub fn new(objects: impl IntoIterator<Item = (Id, String)>) -> Result<Self, StoreError> {
        let mut index = Index::empty();

        for (id, basename) in objects {
            index.insert(id, basename)?;
        }

        Ok(index)
    }

    /// Read and parse `directory`'s index, or the empty index when there is none.
    pub fn read(directory: &Path) -> Result<Self, StoreError> {
        match read_entry(directory, INDEX_BASENAME)? {
            Some(bytes) => Index::parse(&bytes),
            None => Ok(Index::empty()),
        }
    }

    /// Parse the index's bytes, refusing every document §5.2 calls corrupt.
    pub fn parse(bytes: &[u8]) -> Result<Self, StoreError> {
        let document: Value = serde_json::from_slice(bytes)
            .map_err(|error| corrupt(format!("malformed JSON: {error}")))?;
        let fields = document
            .as_object()
            .ok_or_else(|| corrupt("the index must be a JSON object"))?;

        match fields.get(FORMAT_VERSION_KEY) {
            Some(version) if version.as_u64() == Some(FORMAT_VERSION) => {}
            Some(version) => {
                return Err(StoreError::UnknownFormatVersion {
                    found: version.to_string(),
                    expected: FORMAT_VERSION,
                });
            }
            None => return Err(corrupt("the index has no format_version")),
        }

        let objects = fields
            .get(OBJECTS_KEY)
            .and_then(Value::as_object)
            .ok_or_else(|| corrupt("the index must hold an `objects` object"))?;
        let mut index = Index::empty();

        for (id, basename) in objects {
            if id.is_empty() {
                return Err(corrupt("`objects` holds an empty id"));
            }
            let basename = basename.as_str().ok_or_else(|| {
                corrupt(format!(
                    "`objects` maps {id} to something other than a string"
                ))
            })?;

            index.insert(Id::new(id), basename.to_owned())?;
        }

        Ok(index)
    }

    /// The index's canonical file bytes (§4.2): pretty-printed, keys lexical,
    /// ids in their own order.
    pub fn to_bytes(&self) -> String {
        let objects = Map::from_iter(
            self.objects
                .iter()
                .map(|(id, basename)| (id.as_str().to_owned(), Value::String(basename.clone()))),
        );
        let document = Value::Object(Map::from_iter([
            (FORMAT_VERSION_KEY.to_owned(), Value::from(FORMAT_VERSION)),
            (OBJECTS_KEY.to_owned(), Value::Object(objects)),
        ]));

        serialise_document(&document, Form::File)
    }

    /// Map `id` to `basename`, refusing an unsafe name or one already taken.
    pub fn insert(&mut self, id: Id, basename: impl Into<String>) -> Result<(), StoreError> {
        let basename = basename.into();

        if let Some(reason) = unsafe_basename(&basename) {
            return Err(StoreError::UnsafeBasename { basename, reason });
        }
        if let Some((other, _)) = self
            .objects
            .iter()
            .find(|(other, mapped)| other.as_str() != id.as_str() && mapped.as_str() == basename)
        {
            return Err(StoreError::DuplicatePathMapping {
                basename,
                key: id.as_str().to_owned(),
                other: other.as_str().to_owned(),
            });
        }

        self.objects.insert(id, basename);

        Ok(())
    }

    /// Stop mapping `id`, returning the basename it held.
    pub fn remove(&mut self, id: &Id) -> Option<String> {
        self.objects.remove(id)
    }

    /// Every mapping, in public-id order.
    pub fn entries(&self) -> impl Iterator<Item = (&Id, &str)> {
        self.objects
            .iter()
            .map(|(id, basename)| (id, basename.as_str()))
    }

    /// The basename that holds `id`.
    pub fn basename(&self, id: &Id) -> Option<&str> {
        self.objects.get(id).map(String::as_str)
    }

    /// Every basename the index explains.
    pub fn basenames(&self) -> impl Iterator<Item = &str> {
        self.objects.values().map(String::as_str)
    }

    /// Whether the index maps nothing.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// How many entities the index maps.
    pub fn len(&self) -> usize {
        self.objects.len()
    }
}

fn corrupt(reason: impl Into<String>) -> StoreError {
    StoreError::Corrupt {
        entry: INDEX_BASENAME.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn an_index() -> Index {
        Index::new([
            (Id::new("alpha"), "Ab3.json".to_owned()),
            (Id::new("beta"), "Zq7.json".to_owned()),
        ])
        .expect("safe basenames")
    }

    #[test]
    fn the_index_round_trips_through_its_canonical_file_bytes() {
        let index = an_index();
        let bytes = index.to_bytes();

        assert_eq!(
            bytes,
            "{\n  \"format_version\": 1,\n  \"objects\": {\n    \"alpha\": \"Ab3.json\",\n    \"beta\": \"Zq7.json\"\n  }\n}\n"
        );
        assert_eq!(
            Index::parse(bytes.as_bytes()).expect("its own bytes"),
            index
        );
    }

    #[test]
    fn an_empty_index_round_trips_and_maps_nothing() {
        let index = Index::empty();
        let bytes = index.to_bytes();

        assert_eq!(bytes, "{\n  \"format_version\": 1,\n  \"objects\": {}\n}\n");
        assert_eq!(
            Index::parse(bytes.as_bytes()).expect("its own bytes"),
            index
        );
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn an_unknown_format_version_is_refused_and_never_migrated() {
        for document in [
            r#"{"format_version":2,"objects":{}}"#,
            r#"{"format_version":0,"objects":{}}"#,
            r#"{"format_version":"1","objects":{}}"#,
            r#"{"format_version":1.5,"objects":{}}"#,
        ] {
            let error = Index::parse(document.as_bytes()).expect_err(document);

            assert!(
                matches!(error, StoreError::UnknownFormatVersion { .. }),
                "{document}: {error}"
            );
            assert!(error.to_string().contains("format_version"), "{error}");
        }
    }

    #[test]
    fn a_document_that_is_not_an_index_is_refused_rather_than_repaired() {
        for document in [
            "not json",
            "[]",
            "{}",
            r#"{"format_version":1}"#,
            r#"{"format_version":1,"objects":[]}"#,
            r#"{"format_version":1,"objects":{"alpha":1}}"#,
            r#"{"format_version":1,"objects":{"":"Ab3.json"}}"#,
        ] {
            let error = Index::parse(document.as_bytes()).expect_err(document);

            assert!(
                matches!(error, StoreError::Corrupt { .. }),
                "{document}: {error}"
            );
            assert!(error.to_string().contains("index.json"), "{error}");
        }
    }

    #[test]
    fn a_duplicate_path_mapping_is_refused_on_construction_and_on_read() {
        let constructed = Index::new([
            (Id::new("alpha"), "Ab3.json".to_owned()),
            (Id::new("beta"), "Ab3.json".to_owned()),
        ])
        .expect_err("two ids cannot share one file");

        assert!(
            matches!(constructed, StoreError::DuplicatePathMapping { .. }),
            "{constructed}"
        );
        assert!(
            constructed.to_string().contains("Ab3.json"),
            "{constructed}"
        );

        let read = Index::parse(
            br#"{"format_version":1,"objects":{"alpha":"Ab3.json","beta":"Ab3.json"}}"#,
        )
        .expect_err("the same duplicate came from the document");

        assert!(
            matches!(read, StoreError::DuplicatePathMapping { .. }),
            "{read}"
        );
    }

    #[test]
    fn an_unsafe_basename_is_refused_on_write_and_on_read() {
        for basename in [
            "",
            ".",
            "..",
            "a/b",
            "a\\b",
            "lock",
            "index.json",
            "manifest.json",
            "journal.prepare",
            "journal.commit",
            "journal.done",
            "Ab3.json.tmp",
            "a.tmp.tmp",
        ] {
            let written =
                Index::new([(Id::new("alpha"), basename.to_owned())]).expect_err(basename);

            assert!(
                matches!(written, StoreError::UnsafeBasename { .. }),
                "{basename}: {written}"
            );
            assert!(written.to_string().contains(basename), "{written}");

            let document = format!(
                r#"{{"format_version":1,"objects":{{"alpha":"{}"}}}}"#,
                basename.replace('\\', "\\\\")
            );
            let read = Index::parse(document.as_bytes()).expect_err(basename);

            assert!(
                matches!(read, StoreError::UnsafeBasename { .. }),
                "{basename}: {read}"
            );
        }
    }

    #[test]
    fn the_ordinary_uid_shape_is_a_safe_basename() {
        for basename in ["Ab3xK.json", "a.json", "_we-ird.json"] {
            assert_eq!(unsafe_basename(basename), None, "{basename}");
        }
    }

    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);

            let path = std::env::temp_dir().join(format!(
                "smut-index-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("a scratch directory");

            Self { path }
        }

        fn directory(&self) -> PathBuf {
            self.path.join(crate::store::STORE_DIRECTORY)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn a_mapped_file_that_is_not_there_is_refused_naming_it() {
        use crate::store::Store;

        let scratch = Scratch::new();
        fs::create_dir(scratch.directory()).expect("the store directory");
        fs::write(
            scratch.directory().join(INDEX_BASENAME),
            an_index().to_bytes(),
        )
        .expect("the index");

        let error = Store::in_directory(&scratch.path)
            .load()
            .expect_err("Ab3.json is mapped and absent");

        assert!(error.to_string().contains("Ab3.json"), "{error}");
    }
}
