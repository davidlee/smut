//! The journal: the commit protocol of §4.3, recovery of §4.4, and the write-set
//! translation that ties a declaration batch to staged bytes.
//!
//! The state is a directory name (DEC-003): `journal.prepare` means staged and
//! not yet decided, `journal.commit` means decided and must roll forward, and
//! `journal.done` means applied and being cleared. Every transition is one
//! rename, and every durable step goes through [`crate::store::fs`], so nothing
//! here open-codes an fsync.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

use crate::declare::WriteSet;
use crate::entity::{ENTITY_ID, Form, serialise_document, serialise_entity};
use crate::error::StoreError;
use crate::id::Id;
use crate::store::boundary::Boundary;
use crate::store::fs as durable;
use crate::store::index::Index;
use crate::store::{INDEX_BASENAME, JOURNAL_STATES, MANIFEST_BASENAME, Snapshot};

/// The staging state: staged bytes, commitment unknown (§4.3).
pub const PREPARE_STATE: &str = JOURNAL_STATES[0];

/// The commit state: decided and durable, replayed to completion (§4.3).
pub const COMMIT_STATE: &str = JOURNAL_STATES[1];

/// The done state: applied, being cleared (§4.4).
pub const DONE_STATE: &str = JOURNAL_STATES[2];

/// The manifest format this build writes and reads (§4.7).
pub const MANIFEST_FORMAT_VERSION: u64 = 1;

const FORMAT_VERSION_KEY: &str = "format_version";
const PUTS_KEY: &str = "puts";
const REMOVES_KEY: &str = "removes";

/// The suffix of every entity file's basename (§4.1).
const UID_SUFFIX: &str = ".json";

/// The temporary suffix §4.1 reserves, whether or not the protocol generates the
/// name.
const TEMPORARY_SUFFIX: &str = ".tmp";

/// The write set, named in basename space: what the staged directory holds.
///
/// The manifest **is** the write set (§4.3), so the staged directory and this
/// document can only disagree by corruption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    puts: Vec<String>,
    removes: Vec<String>,
}

impl Manifest {
    /// A manifest of `puts` and `removes`, in the order they were decided.
    pub fn new(
        puts: impl IntoIterator<Item = String>,
        removes: impl IntoIterator<Item = String>,
    ) -> Self {
        Manifest {
            puts: puts.into_iter().collect(),
            removes: removes.into_iter().collect(),
        }
    }

    /// The basenames the commit publishes, `index.json` last.
    pub fn puts(&self) -> &[String] {
        &self.puts
    }

    /// The basenames the commit unlinks.
    pub fn removes(&self) -> &[String] {
        &self.removes
    }

    /// The manifest's canonical file bytes (§4.2).
    pub fn to_bytes(&self) -> String {
        let names =
            |names: &[String]| Value::Array(names.iter().cloned().map(Value::String).collect());
        let document = Value::Object(Map::from_iter([
            (
                FORMAT_VERSION_KEY.to_owned(),
                Value::from(MANIFEST_FORMAT_VERSION),
            ),
            (PUTS_KEY.to_owned(), names(&self.puts)),
            (REMOVES_KEY.to_owned(), names(&self.removes)),
        ]));

        serialise_document(&document, Form::File)
    }

    /// Parse a manifest's bytes, refusing every document §4.7 calls unreadable.
    pub fn parse(bytes: &[u8]) -> Result<Self, StoreError> {
        let document: Value = serde_json::from_slice(bytes)
            .map_err(|error| corrupt(format!("malformed JSON: {error}")))?;
        let fields = document
            .as_object()
            .ok_or_else(|| corrupt("a manifest must be a JSON object"))?;

        match fields.get(FORMAT_VERSION_KEY) {
            Some(version) if version.as_u64() == Some(MANIFEST_FORMAT_VERSION) => {}
            Some(version) => {
                return Err(corrupt(format!(
                    "format_version {version} is not the version this build reads \
                     ({MANIFEST_FORMAT_VERSION})"
                )));
            }
            None => return Err(corrupt("the manifest has no format_version")),
        }

        let names = |key: &str| -> Result<Vec<String>, StoreError> {
            let entries = fields
                .get(key)
                .and_then(Value::as_array)
                .ok_or_else(|| corrupt(format!("the manifest must hold a `{key}` array")))?;

            entries
                .iter()
                .map(|entry| {
                    entry.as_str().map(str::to_owned).ok_or_else(|| {
                        corrupt(format!("`{key}` holds something other than a string"))
                    })
                })
                .collect()
        };

        Ok(Manifest {
            puts: names(PUTS_KEY)?,
            removes: names(REMOVES_KEY)?,
        })
    }

    /// Validate the names §4.4 requires before anything is replayed: safe, not
    /// reserved, and unique across both lists — which is disjointness too.
    pub fn validate(&self) -> Result<(), StoreError> {
        let mut seen = BTreeSet::new();

        for basename in self.puts.iter().chain(self.removes.iter()) {
            if let Some(reason) = reserved_basename(basename) {
                return Err(StoreError::UnsafeBasename {
                    basename: basename.clone(),
                    reason,
                });
            }
            if !seen.insert(basename) {
                return Err(corrupt(format!(
                    "{basename} appears more than once in the write set"
                )));
            }
        }

        Ok(())
    }
}

/// The material one commit stages: the manifest, and the bytes of every file it
/// puts, `index.json` last.
#[derive(Debug)]
pub struct Staged {
    manifest: Manifest,
    files: Vec<(String, Vec<u8>)>,
}

impl Staged {
    /// The write set this material realises (§4.3: the manifest *is* the write
    /// set).
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The bytes to stage, in staging order: every put, `index.json` last.
    pub fn files(&self) -> &[(String, Vec<u8>)] {
        &self.files
    }
}

/// Translate `write_set` against `snapshot` into the material the commit stages.
///
/// An existing public id keeps the basename the index already maps it to, since
/// uids are stable until deletion; a new one draws a free filesystem-safe uid
/// (§3.4). Removals become the basenames the index maps them to, and the
/// successor index is the old one with the puts overwritten and the removals
/// deleted.
pub fn translate(
    directory: &Path,
    snapshot: &Snapshot,
    write_set: &WriteSet,
) -> Result<Staged, StoreError> {
    let mut taken: BTreeSet<String> = snapshot.index().basenames().map(str::to_owned).collect();
    let removed: BTreeSet<&Id> = write_set.remove.iter().collect();
    let mut files = Vec::new();
    let mut objects = Vec::new();
    let mut puts = Vec::new();

    for entity in &write_set.put {
        let id = entity_id(entity)?;

        if removed.contains(&id) {
            return Err(invalid(format!("{id} is both put and removed")));
        }

        let basename = match snapshot.index().basename(&id) {
            Some(mapped) => mapped.to_owned(),
            None => fresh_basename(directory, &taken)?,
        };

        taken.insert(basename.clone());
        puts.push(basename.clone());
        files.push((
            basename.clone(),
            serialise_entity(entity, Form::File).into_bytes(),
        ));
        objects.push((id, basename));
    }

    let mut removes = Vec::new();

    for id in &write_set.remove {
        let basename = snapshot.index().basename(id).ok_or_else(|| {
            invalid(format!(
                "the write set removes {id}, which the index does not map"
            ))
        })?;

        removes.push(basename.to_owned());
    }

    for (id, basename) in snapshot.index().entries() {
        if removed.contains(id) {
            continue;
        }

        objects.push((id.clone(), basename.to_owned()));
    }

    let successor = Index::new(objects)?;

    puts.push(INDEX_BASENAME.to_owned());
    files.push((INDEX_BASENAME.to_owned(), successor.to_bytes().into_bytes()));

    Ok(Staged {
        manifest: Manifest::new(puts, removes),
        files,
    })
}

/// The public id of one put entity. The core guarantees this shape (§3.1); the
/// shell refuses a write set that contradicts it rather than inventing an id.
fn entity_id(entity: &Value) -> Result<Id, StoreError> {
    entity
        .get(ENTITY_ID)
        .and_then(Value::as_str)
        .map(Id::new)
        .ok_or_else(|| invalid("a put entity names no string `id`"))
}

/// A basename no mapping, no other staged file and no file in the store claims.
fn fresh_basename(directory: &Path, taken: &BTreeSet<String>) -> Result<String, StoreError> {
    fresh_basename_from(directory, taken, || {
        format!("{}{UID_SUFFIX}", nanoid::nanoid!())
    })
}

/// [`fresh_basename`] over a candidate source, so the redraw path is testable
/// without pretending entropy can be predicted.
fn fresh_basename_from(
    directory: &Path,
    taken: &BTreeSet<String>,
    mut candidate: impl FnMut() -> String,
) -> Result<String, StoreError> {
    loop {
        let basename = candidate();

        if taken.contains(&basename) {
            continue;
        }

        match std::fs::symlink_metadata(directory.join(&basename)) {
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(basename),
            Ok(_) => continue,
            Err(source) => return Err(failure("fresh_uid", directory, source)),
        }
    }
}

/// Commit `write_set` as one batch: stage, decide, apply, clear (§4.3).
///
/// The `prepare → commit` rename is the commit decision. A failure before it is
/// pre-commit: the staging state is discarded and the store is left exactly as
/// it was. A failure after it is roll-forward-only: the commit state stays for
/// the next command to replay (§4.5).
pub fn commit(
    directory: &Path,
    snapshot: &Snapshot,
    write_set: &WriteSet,
) -> Result<(), StoreError> {
    let staged = translate(directory, snapshot, write_set)?;
    let state = directory.join(PREPARE_STATE);

    std::fs::create_dir(&state)
        .map_err(|source| failure("create_journal_state", &state, source))?;

    if let Err(error) = fill_staging(&state, &staged) {
        let _ = discard_prepare(directory);

        return Err(error);
    }
    if let Err(error) = decide(directory) {
        let _ = discard_prepare(directory);

        return Err(error);
    }

    durable::sync_directory(directory, Boundary::ParentDirectorySynced)?;
    apply(directory, &staged)?;
    finish(directory)
}

/// The barrier: every staged put file including `index.json`, and the manifest,
/// written to completion and fsynced individually, then the staging directory
/// fsynced so its entries are durable (§4.3).
fn fill_staging(state: &Path, staged: &Staged) -> Result<(), StoreError> {
    for (basename, bytes) in &staged.files {
        let (written, synced) = if basename == INDEX_BASENAME {
            (Boundary::StagedIndexWritten, Boundary::StagedIndexSynced)
        } else {
            (Boundary::StagedPutWritten, Boundary::StagedPutSynced)
        };
        let path = state.join(basename);

        durable::write_file(&path, bytes, written)?;
        durable::sync_file(&path, synced)?;
    }

    let manifest = state.join(MANIFEST_BASENAME);

    durable::write_file(
        &manifest,
        staged.manifest.to_bytes().as_bytes(),
        Boundary::StagedManifestWritten,
    )?;
    durable::sync_file(&manifest, Boundary::StagedManifestSynced)?;
    durable::sync_directory(state, Boundary::StagingDirectorySynced)
}

/// The commit decision: one rename, and the only step that makes the batch
/// committed rather than staged.
fn decide(directory: &Path) -> Result<(), StoreError> {
    durable::rename(
        &directory.join(PREPARE_STATE),
        &directory.join(COMMIT_STATE),
        Boundary::CommitRenamed,
    )
}

/// Apply the batch in §4.3's order — puts, then the index, then removals — and
/// make the applied state durable before the state moves again.
fn apply(directory: &Path, staged: &Staged) -> Result<(), StoreError> {
    for (basename, bytes) in &staged.files {
        let at = if basename == INDEX_BASENAME {
            Boundary::IndexApplied
        } else {
            Boundary::PutsApplied
        };

        publish(directory, basename, bytes, at)?;
    }

    for basename in staged.manifest.removes() {
        remove_if_present(directory, basename, Boundary::RemovalsApplied)?;
    }

    durable::sync_directory(directory, Boundary::RemovalsApplied)
}

/// Publish `bytes` as `basename`: write its temporary, fsync it, rename it over
/// the target. The directory fsync is the caller's, once per phase.
fn publish(directory: &Path, basename: &str, bytes: &[u8], at: Boundary) -> Result<(), StoreError> {
    let temporary = directory.join(durable::temporary_name(basename));

    durable::write_file(&temporary, bytes, at)?;
    durable::sync_file(&temporary, at)?;
    durable::rename(&temporary, &directory.join(basename), at)
}

/// Move the decided state to `done`, make that name durable, then clear it — in
/// that order, so a power loss cannot restore `commit` after its replay material
/// is gone (§4.3).
fn finish(directory: &Path) -> Result<(), StoreError> {
    durable::rename(
        &directory.join(COMMIT_STATE),
        &directory.join(DONE_STATE),
        Boundary::DoneRenamed,
    )?;
    durable::sync_directory(directory, Boundary::DoneRenamed)?;
    clear(directory, DONE_STATE, Boundary::CleanupRemoved)
}

/// Unlink every entry of `directory`'s state, then the state directory itself.
fn clear(directory: &Path, state: &str, at: Boundary) -> Result<(), StoreError> {
    let path = directory.join(state);

    for name in names(&path)? {
        durable::remove_file(&path.join(&name), at)?;
    }

    durable::remove_directory(&path, at)
}

/// Discard a staging state, best effort (§4.3).
///
/// The error the caller already has is the one the command reports, and a
/// staging state this could not remove is discarded by the next command's
/// recovery.
fn discard_prepare(directory: &Path) -> Result<(), StoreError> {
    match std::fs::symlink_metadata(directory.join(PREPARE_STATE)) {
        Ok(metadata) if metadata.is_dir() => {
            clear(directory, PREPARE_STATE, Boundary::RecoveryPrepareDiscarded)
        }
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(failure("discard_prepare", directory, source)),
    }
}

/// Unlink `basename` if it is there: a replay must not depend on what an earlier
/// attempt already did.
fn remove_if_present(directory: &Path, basename: &str, at: Boundary) -> Result<(), StoreError> {
    let path = directory.join(basename);

    match std::fs::symlink_metadata(&path) {
        Ok(_) => durable::remove_file(&path, at),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(failure(at.as_str(), &path, source)),
    }
}

/// The entry names `directory` holds, in lexical order, so nothing here depends
/// on `readdir` order.
fn names(directory: &Path) -> Result<Vec<String>, StoreError> {
    let listing = std::fs::read_dir(directory)
        .map_err(|source| failure("list_journal", directory, source))?;
    let mut names = Vec::new();

    for entry in listing {
        let entry = entry.map_err(|source| failure("list_journal", directory, source))?;

        names.push(entry.file_name().to_string_lossy().into_owned());
    }

    names.sort();

    Ok(names)
}

/// Why `basename` may not appear in a manifest (§4.1, §4.4).
fn reserved_basename(basename: &str) -> Option<&'static str> {
    if basename.is_empty() {
        return Some("a basename cannot be empty");
    }
    if basename.contains(['/', '\\']) {
        return Some("a basename cannot hold a path separator");
    }
    if basename == "." || basename == ".." {
        return Some("a basename cannot be `.` or `..`");
    }
    if basename == crate::store::LOCK_BASENAME {
        return Some("the store's lock is a reserved name");
    }
    if basename == MANIFEST_BASENAME {
        return Some("a journal manifest is a reserved name");
    }
    if JOURNAL_STATES.contains(&basename) {
        return Some("a journal state is a reserved name");
    }
    if basename.ends_with(TEMPORARY_SUFFIX) {
        return Some("the protocol's temporary namespace is reserved");
    }

    None
}

fn corrupt(reason: impl Into<String>) -> StoreError {
    StoreError::Corrupt {
        entry: MANIFEST_BASENAME.to_owned(),
        reason: reason.into(),
    }
}

fn invalid(reason: impl Into<String>) -> StoreError {
    StoreError::InvalidWriteSet {
        reason: reason.into(),
    }
}

fn failure(step: &'static str, path: &Path, source: io::Error) -> StoreError {
    StoreError::FileSystem {
        step,
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::json;

    use super::*;
    use crate::declare::Entities;
    use crate::entity::{Form, serialise_entity};
    use crate::store::Snapshot;

    /// A scratch directory holding one empty store directory.
    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);

            let path = std::env::temp_dir().join(format!(
                "smut-journal-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(path.join(crate::store::STORE_DIRECTORY)).expect("a scratch store");

            Self { path }
        }

        fn directory(&self) -> PathBuf {
            self.path.join(crate::store::STORE_DIRECTORY)
        }

        fn store(&self) -> crate::store::Store {
            crate::store::Store::in_directory(&self.path)
        }

        /// A store holding two indexed entities: `alpha` and `beta`.
        fn seeded() -> Self {
            let scratch = Scratch::new();
            let directory = scratch.directory();

            fs::write(directory.join(crate::store::LOCK_BASENAME), b"").expect("the lock");
            fs::write(
                directory.join("Ab3.json"),
                entity_bytes(json!({"id": "alpha", "body": "first"})),
            )
            .expect("alpha's file");
            fs::write(
                directory.join("Zq7.json"),
                entity_bytes(json!({"id": "beta", "body": "second"})),
            )
            .expect("beta's file");
            fs::write(
                directory.join(INDEX_BASENAME),
                Index::new([
                    (Id::new("alpha"), "Ab3.json".to_owned()),
                    (Id::new("beta"), "Zq7.json".to_owned()),
                ])
                .expect("safe basenames")
                .to_bytes(),
            )
            .expect("the index");

            scratch
        }

        fn loaded(&self) -> Snapshot {
            self.store().load().expect("a healthy store")
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

    /// A snapshot holding two mapped ids and nothing else.
    fn snapshot() -> Snapshot {
        Snapshot {
            entities: Entities::new(),
            index: Index::new([
                (Id::new("alpha"), "Ab3.json".to_owned()),
                (Id::new("beta"), "Zq7.json".to_owned()),
            ])
            .expect("safe basenames"),
        }
    }

    fn write_set(puts: Vec<Value>, removes: &[&str]) -> WriteSet {
        WriteSet {
            put: puts,
            remove: removes.iter().map(|id| Id::new(*id)).collect(),
        }
    }

    fn entity_bytes(entity: Value) -> Vec<u8> {
        serialise_entity(&entity, Form::File).into_bytes()
    }

    #[test]
    fn the_manifest_names_the_write_set_in_the_designs_form() {
        let manifest = Manifest::new(
            ["Ab3.json".to_owned(), "index.json".to_owned()],
            ["Zq7.json".to_owned()],
        );

        assert_eq!(
            manifest.to_bytes(),
            "{\n  \"format_version\": 1,\n  \"puts\": [\n    \"Ab3.json\",\n    \"index.json\"\n  ],\n  \"removes\": [\n    \"Zq7.json\"\n  ]\n}\n"
        );
        assert_eq!(
            Manifest::parse(manifest.to_bytes().as_bytes()).expect("its own bytes"),
            manifest
        );
    }

    #[test]
    fn a_manifest_that_writes_nothing_round_trips() {
        let manifest = Manifest::new(Vec::new(), Vec::new());

        assert_eq!(
            manifest.to_bytes(),
            "{\n  \"format_version\": 1,\n  \"puts\": [],\n  \"removes\": []\n}\n"
        );
        assert_eq!(
            Manifest::parse(manifest.to_bytes().as_bytes()).expect("its own bytes"),
            manifest
        );
    }

    #[test]
    fn a_manifest_of_another_format_version_is_refused() {
        for document in [
            r#"{"format_version":2,"puts":[],"removes":[]}"#,
            r#"{"format_version":"1","puts":[],"removes":[]}"#,
            r#"{"puts":[],"removes":[]}"#,
        ] {
            let error = Manifest::parse(document.as_bytes()).expect_err(document);

            assert!(
                matches!(error, StoreError::Corrupt { .. }),
                "{document}: {error}"
            );
            assert!(error.to_string().contains("manifest.json"), "{error}");
            assert!(error.to_string().contains("format_version"), "{error}");
        }
    }

    #[test]
    fn a_document_that_is_not_a_manifest_is_refused_rather_than_repaired() {
        for document in [
            "not json",
            "[]",
            r#"{"format_version":1}"#,
            r#"{"format_version":1,"puts":{},"removes":[]}"#,
            r#"{"format_version":1,"puts":[1],"removes":[]}"#,
            r#"{"format_version":1,"puts":[]}"#,
        ] {
            let error = Manifest::parse(document.as_bytes()).expect_err(document);

            assert!(
                matches!(error, StoreError::Corrupt { .. }),
                "{document}: {error}"
            );
            assert!(error.to_string().contains("manifest.json"), "{error}");
        }
    }

    #[test]
    fn a_manifest_refuses_a_basename_the_store_reserves() {
        for basename in [
            "",
            ".",
            "..",
            "a/b",
            "a\\b",
            "lock",
            "manifest.json",
            "journal.prepare",
            "journal.commit",
            "journal.done",
            "Ab3.json.tmp",
            "a.tmp.tmp",
        ] {
            let manifest = Manifest::new([basename.to_owned()], []);

            let error = manifest.validate().expect_err(basename);

            assert!(
                matches!(error, StoreError::UnsafeBasename { .. }),
                "{basename}: {error}"
            );
            assert!(error.to_string().contains(basename), "{error}");
        }
    }

    #[test]
    fn a_manifest_refuses_a_duplicate_and_an_overlapping_entry() {
        let duplicated = Manifest::new(["Ab3.json".to_owned(), "Ab3.json".to_owned()], []);
        let overlapping = Manifest::new(["Ab3.json".to_owned()], ["Ab3.json".to_owned()]);

        for manifest in [duplicated, overlapping] {
            let error = manifest.validate().expect_err("the entries are not a set");

            assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
            assert!(error.to_string().contains("Ab3.json"), "{error}");
        }
    }

    #[test]
    fn the_index_is_a_legal_put_because_every_commit_stages_one() {
        Manifest::new(["index.json".to_owned()], [])
            .validate()
            .expect("the staged index is part of the write set");
    }

    #[test]
    fn translation_reuses_a_mapped_basename_and_stages_the_fresh_index() {
        let scratch = Scratch::new();
        let staged = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(vec![json!({"id": "alpha", "body": "changed"})], &["beta"]),
        )
        .expect("a consistent write set");

        assert_eq!(staged.manifest().puts(), ["Ab3.json", INDEX_BASENAME]);
        assert_eq!(staged.manifest().removes(), ["Zq7.json"]);
        assert_eq!(staged.files().len(), 2);
        assert_eq!(staged.files()[0].0, "Ab3.json");
        assert_eq!(
            staged.files()[0].1,
            serialise_entity(&json!({"id": "alpha", "body": "changed"}), Form::File).into_bytes()
        );

        let successor = Index::parse(&staged.files()[1].1).expect("a parseable index");
        assert_eq!(successor.basename(&Id::new("alpha")), Some("Ab3.json"));
        assert_eq!(successor.basename(&Id::new("beta")), None);
    }

    #[test]
    fn every_commit_stages_a_fresh_index_even_when_no_mapping_changes() {
        let scratch = Scratch::new();
        let staged = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(vec![json!({"id": "alpha"})], &[]),
        )
        .expect("a consistent write set");

        assert_eq!(staged.manifest().puts(), ["Ab3.json", INDEX_BASENAME]);
        assert_eq!(
            staged.files()[1].1,
            snapshot().index().to_bytes().into_bytes(),
            "the staged index is the unchanged one, and it is still staged"
        );
    }

    #[test]
    fn a_new_public_id_gets_a_free_filesystem_safe_uid() {
        let scratch = Scratch::new();
        let staged = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(vec![json!({"id": "gamma"})], &[]),
        )
        .expect("a consistent write set");

        let basename = staged.manifest().puts()[0].clone();
        assert!(basename.ends_with(".json"), "{basename}");
        assert!(!basename.contains(['/', '\\']), "{basename}");
        assert!(!scratch.directory().join(&basename).exists(), "{basename}");

        let successor = Index::parse(&staged.files()[1].1).expect("a parseable index");
        assert_eq!(
            successor.basename(&Id::new("gamma")),
            Some(basename.as_str())
        );
        assert_eq!(successor.basename(&Id::new("alpha")), Some("Ab3.json"));
    }

    #[test]
    fn a_fresh_basename_is_redrawn_until_it_is_free() {
        let scratch = Scratch::new();
        let taken = BTreeSet::from(["Mapped.json".to_owned()]);
        fs::write(scratch.directory().join("Drawn.json"), b"{}").expect("a file in the way");

        let mut candidates = ["Mapped.json", "Drawn.json", "Free.json"].into_iter();
        let basename = fresh_basename_from(&scratch.directory(), &taken, || {
            candidates.next().expect("a scripted candidate").to_owned()
        })
        .expect("a free name");

        assert_eq!(basename, "Free.json");
    }

    #[test]
    fn a_write_set_that_contradicts_the_snapshot_is_refused() {
        let scratch = Scratch::new();

        let anonymous = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(vec![json!({"body": "no id"})], &[]),
        )
        .expect_err("a put names its entity");
        assert!(
            matches!(anonymous, StoreError::InvalidWriteSet { .. }),
            "{anonymous}"
        );

        let unmapped = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(Vec::new(), &["gone"]),
        )
        .expect_err("the index does not map that id");
        assert!(
            matches!(unmapped, StoreError::InvalidWriteSet { .. }),
            "{unmapped}"
        );

        let both = translate(
            &scratch.directory(),
            &snapshot(),
            &write_set(vec![json!({"id": "beta"})], &["beta"]),
        )
        .expect_err("one id cannot be put and removed");
        assert!(matches!(both, StoreError::InvalidWriteSet { .. }), "{both}");
    }

    #[test]
    fn the_commit_orders_the_barrier_the_apply_and_the_clear_steps() {
        use crate::store::fs::call_record::{Call, take};

        let scratch = Scratch::seeded();
        let snapshot = scratch.loaded();

        take();
        commit(
            &scratch.directory(),
            &snapshot,
            &write_set(vec![json!({"id": "alpha", "body": "changed"})], &["beta"]),
        )
        .expect("the batch commits");
        let calls = take();

        let directory = scratch.directory();
        let prepare = directory.join(PREPARE_STATE);
        let commit_state = directory.join(COMMIT_STATE);
        let done = directory.join(DONE_STATE);
        let temporary = |basename: &str| directory.join(crate::store::fs::temporary_name(basename));

        assert_eq!(
            calls,
            vec![
                Call::Write(prepare.join("Ab3.json")),
                Call::SyncFile(prepare.join("Ab3.json")),
                Call::Write(prepare.join(INDEX_BASENAME)),
                Call::SyncFile(prepare.join(INDEX_BASENAME)),
                Call::Write(prepare.join(MANIFEST_BASENAME)),
                Call::SyncFile(prepare.join(MANIFEST_BASENAME)),
                Call::SyncDirectory(prepare.clone()),
                Call::Rename {
                    from: prepare,
                    to: commit_state.clone(),
                },
                Call::SyncDirectory(directory.clone()),
                Call::Write(temporary("Ab3.json")),
                Call::SyncFile(temporary("Ab3.json")),
                Call::Rename {
                    from: temporary("Ab3.json"),
                    to: directory.join("Ab3.json"),
                },
                Call::Write(temporary(INDEX_BASENAME)),
                Call::SyncFile(temporary(INDEX_BASENAME)),
                Call::Rename {
                    from: temporary(INDEX_BASENAME),
                    to: directory.join(INDEX_BASENAME),
                },
                Call::Remove(directory.join("Zq7.json")),
                Call::SyncDirectory(directory.clone()),
                Call::Rename {
                    from: commit_state,
                    to: done.clone(),
                },
                Call::SyncDirectory(directory.clone()),
                Call::Remove(done.join("Ab3.json")),
                Call::Remove(done.join(INDEX_BASENAME)),
                Call::Remove(done.join(MANIFEST_BASENAME)),
                Call::RemoveDirectory(done),
            ]
        );
    }

    #[test]
    fn a_commit_publishes_every_put_and_removal_and_leaves_no_journal_behind() {
        let scratch = Scratch::seeded();
        let snapshot = scratch.loaded();

        commit(
            &scratch.directory(),
            &snapshot,
            &write_set(vec![json!({"id": "alpha", "body": "changed"})], &["beta"]),
        )
        .expect("the batch commits");

        let reloaded = scratch.loaded();
        assert_eq!(reloaded.entities().len(), 1);
        assert_eq!(
            reloaded.entities()[&Id::new("alpha")]["body"],
            json!("changed")
        );
        assert_eq!(reloaded.index().basename(&Id::new("beta")), None);
        assert_eq!(scratch.listing(), ["Ab3.json", "index.json", "lock"]);
    }

    #[cfg(feature = "fault-injection")]
    #[test]
    fn a_barrier_failure_before_the_decision_discards_the_staging_state() {
        use crate::store::fault::{Faults, arm};

        let scratch = Scratch::seeded();
        let snapshot = scratch.loaded();
        let before = scratch.listing();

        {
            let _armed = arm(Faults::failing_at(Boundary::StagedManifestSynced));
            let error = commit(
                &scratch.directory(),
                &snapshot,
                &write_set(vec![json!({"id": "alpha", "body": "changed"})], &["beta"]),
            )
            .expect_err("the barrier fails");

            assert!(
                error.to_string().contains("staged_manifest_synced"),
                "{error}"
            );
        }

        assert_eq!(
            scratch.listing(),
            before,
            "a pre-commit failure left the store exactly as it was"
        );
    }

    #[cfg(feature = "fault-injection")]
    #[test]
    fn a_failure_after_the_decision_leaves_the_commit_state_behind() {
        use crate::store::fault::{Faults, arm};

        let scratch = Scratch::seeded();
        let snapshot = scratch.loaded();

        {
            let _armed = arm(Faults::failing_at(Boundary::ParentDirectorySynced));
            let error = commit(
                &scratch.directory(),
                &snapshot,
                &write_set(vec![json!({"id": "alpha", "body": "changed"})], &["beta"]),
            )
            .expect_err("the decision's fsync fails");

            assert!(
                error.to_string().contains("parent_directory_synced"),
                "{error}"
            );
        }

        assert!(scratch.listing().contains(&COMMIT_STATE.to_owned()));
        assert!(!scratch.listing().contains(&PREPARE_STATE.to_owned()));
    }
}
