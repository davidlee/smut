//! The store shell: the protocol's boundaries, its durable writes, and the fault
//! seam the interruption suite arms.

pub mod boundary {
    //! The single list of the store protocol's boundaries (§4.3, §4.4).
    //!
    //! One variant per boundary, grouped by the track that reaches it. This is
    //! the only enumeration anywhere: `src/store/journal.rs` consumes it and may
    //! append names here, and no second list may exist beside it. A boundary
    //! names the moment a step has completed — the moment at which an
    //! interruption is meaningful — so a boundary is reached once per repetition
    //! of the step it names (once per staged file, per applied file, per cleanup
    //! removal).

    /// Which protocol track reaches a boundary.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Track {
        /// The commit path of §4.3: stage, decide, apply, clear.
        Commit,
        /// The recovery path of §4.4: discard, replay, complete, clear.
        Recovery,
    }

    /// One moment in the commit or recovery protocol.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Boundary {
        /// `.smut/`'s parent directory was fsynced, making the store directory's
        /// own entry durable (§5.1, DEC-003). A new store's first commit cannot
        /// begin before this completes.
        InitialisationParentSynced,
        /// A staged entity file was written to completion.
        StagedPutWritten,
        /// A staged entity file was fsynced.
        StagedPutSynced,
        /// The staged `index.json` was written to completion.
        StagedIndexWritten,
        /// The staged `index.json` was fsynced.
        StagedIndexSynced,
        /// The staged `manifest.json` was written to completion.
        StagedManifestWritten,
        /// The staged `manifest.json` was fsynced.
        StagedManifestSynced,
        /// `journal.prepare/` was fsynced, making its entries durable.
        StagingDirectorySynced,
        /// `journal.prepare/` was renamed to `journal.commit/`: the commit decision.
        CommitRenamed,
        /// `.smut/` was fsynced after the commit rename.
        ParentDirectorySynced,
        /// The puts were applied.
        PutsApplied,
        /// The index was applied.
        IndexApplied,
        /// The removals were applied.
        RemovalsApplied,
        /// `journal.commit/` was renamed to `journal.done/`.
        DoneRenamed,
        /// One journal entry was unlinked during cleanup.
        CleanupRemoved,
        /// One protocol-owned temporary outside a journal was removed.
        RecoveryTemporariesRemoved,
        /// A `prepare` state was discarded.
        RecoveryPrepareDiscarded,
        /// A `commit` manifest was validated completely before replay.
        RecoveryCommitValidated,
        /// One staged put was replayed.
        RecoveryPutReplayed,
        /// One removal was replayed.
        RecoveryRemovalReplayed,
        /// A replayed `commit` state was renamed to `done`.
        RecoveryDoneRenamed,
        /// One journal entry was unlinked while recovery cleared its state.
        RecoveryCleanupRemoved,
    }

    impl Boundary {
        /// The commit path's boundaries, in the order the protocol reaches them.
        pub const COMMIT_PATH: [Boundary; 15] = [
            Boundary::InitialisationParentSynced,
            Boundary::StagedPutWritten,
            Boundary::StagedPutSynced,
            Boundary::StagedIndexWritten,
            Boundary::StagedIndexSynced,
            Boundary::StagedManifestWritten,
            Boundary::StagedManifestSynced,
            Boundary::StagingDirectorySynced,
            Boundary::CommitRenamed,
            Boundary::ParentDirectorySynced,
            Boundary::PutsApplied,
            Boundary::IndexApplied,
            Boundary::RemovalsApplied,
            Boundary::DoneRenamed,
            Boundary::CleanupRemoved,
        ];

        /// The recovery path's boundaries, in the order recovery reaches them.
        pub const RECOVERY_PATH: [Boundary; 7] = [
            Boundary::RecoveryTemporariesRemoved,
            Boundary::RecoveryPrepareDiscarded,
            Boundary::RecoveryCommitValidated,
            Boundary::RecoveryPutReplayed,
            Boundary::RecoveryRemovalReplayed,
            Boundary::RecoveryDoneRenamed,
            Boundary::RecoveryCleanupRemoved,
        ];

        /// Every boundary, the commit path first.
        pub fn all() -> impl Iterator<Item = Boundary> {
            Boundary::COMMIT_PATH
                .into_iter()
                .chain(Boundary::RECOVERY_PATH)
        }

        /// The name the fault seam's environment variables match.
        pub const fn as_str(self) -> &'static str {
            match self {
                Boundary::InitialisationParentSynced => "initialisation_parent_synced",
                Boundary::StagedPutWritten => "staged_put_written",
                Boundary::StagedPutSynced => "staged_put_synced",
                Boundary::StagedIndexWritten => "staged_index_written",
                Boundary::StagedIndexSynced => "staged_index_synced",
                Boundary::StagedManifestWritten => "staged_manifest_written",
                Boundary::StagedManifestSynced => "staged_manifest_synced",
                Boundary::StagingDirectorySynced => "staging_directory_synced",
                Boundary::CommitRenamed => "commit_renamed",
                Boundary::ParentDirectorySynced => "parent_directory_synced",
                Boundary::PutsApplied => "puts_applied",
                Boundary::IndexApplied => "index_applied",
                Boundary::RemovalsApplied => "removals_applied",
                Boundary::DoneRenamed => "done_renamed",
                Boundary::CleanupRemoved => "cleanup_removed",
                Boundary::RecoveryTemporariesRemoved => "recovery_temporaries_removed",
                Boundary::RecoveryPrepareDiscarded => "recovery_prepare_discarded",
                Boundary::RecoveryCommitValidated => "recovery_commit_validated",
                Boundary::RecoveryPutReplayed => "recovery_put_replayed",
                Boundary::RecoveryRemovalReplayed => "recovery_removal_replayed",
                Boundary::RecoveryDoneRenamed => "recovery_done_renamed",
                Boundary::RecoveryCleanupRemoved => "recovery_cleanup_removed",
            }
        }

        /// The boundary registered under `name`, if the registry knows it.
        pub fn from_name(name: &str) -> Option<Boundary> {
            Boundary::all().find(|boundary| boundary.as_str() == name)
        }

        /// Which track reaches this boundary.
        pub const fn track(self) -> Track {
            match self {
                Boundary::InitialisationParentSynced
                | Boundary::StagedPutWritten
                | Boundary::StagedPutSynced
                | Boundary::StagedIndexWritten
                | Boundary::StagedIndexSynced
                | Boundary::StagedManifestWritten
                | Boundary::StagedManifestSynced
                | Boundary::StagingDirectorySynced
                | Boundary::CommitRenamed
                | Boundary::ParentDirectorySynced
                | Boundary::PutsApplied
                | Boundary::IndexApplied
                | Boundary::RemovalsApplied
                | Boundary::DoneRenamed
                | Boundary::CleanupRemoved => Track::Commit,
                Boundary::RecoveryTemporariesRemoved
                | Boundary::RecoveryPrepareDiscarded
                | Boundary::RecoveryCommitValidated
                | Boundary::RecoveryPutReplayed
                | Boundary::RecoveryRemovalReplayed
                | Boundary::RecoveryDoneRenamed
                | Boundary::RecoveryCleanupRemoved => Track::Recovery,
            }
        }
    }
}

/// The store beside one directory: `<root>/.smut`, and never a search upwards.
#[derive(Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// The store beside `root`. Nothing is created until a command asks for it.
    pub fn in_directory(root: impl Into<PathBuf>) -> Self {
        Store { root: root.into() }
    }

    /// The `.smut/` directory this store lives in.
    pub fn directory(&self) -> PathBuf {
        self.root.join(STORE_DIRECTORY)
    }

    /// Whether the store directory is there.
    pub fn exists(&self) -> bool {
        self.directory().is_dir()
    }

    /// Read and validate the store: every entity the index explains, and the
    /// index itself (§5.2). Nothing is written.
    pub fn load(&self) -> Result<Snapshot, StoreError> {
        let directory = self.directory();

        if !directory.is_dir() {
            return Err(StoreError::Absent { path: directory });
        }

        let index = Index::read(&directory)?;
        refuse_unexplained_entries(&directory, &index)?;

        let mut entities = Entities::new();

        for (id, basename) in index.entries() {
            let bytes = read_entry(&directory, basename)?.ok_or_else(|| StoreError::Corrupt {
                entry: basename.to_owned(),
                reason: "the index maps it, and the file is not there".to_owned(),
            })?;
            let entity: Value =
                serde_json::from_slice(&bytes).map_err(|error| StoreError::Corrupt {
                    entry: basename.to_owned(),
                    reason: format!("malformed JSON: {error}"),
                })?;

            if !entity.is_object() {
                return Err(StoreError::Corrupt {
                    entry: basename.to_owned(),
                    reason: "an entity file holds a JSON object".to_owned(),
                });
            }
            let found = entity.get(ENTITY_ID);
            if found.and_then(Value::as_str) != Some(id.as_str()) {
                return Err(StoreError::IdDisagreement {
                    entry: basename.to_owned(),
                    key: id.as_str().to_owned(),
                    found: describe(found),
                });
            }

            validate_identity_tree(basename, &entity)?;

            entities.insert(id.clone(), entity);
        }

        Ok(Snapshot { entities, index })
    }

    /// Take the store's exclusive lock, waiting for it if another holder has it.
    ///
    /// The lock covers reads as well as writes, because a read may recover,
    /// which mutates (DEC-004).
    pub fn lock(&self) -> Result<StoreLock, StoreError> {
        let path = self.lock_path();
        let file = open_lock(&path)?;

        FileExt::lock(&file).map_err(|source| failure("lock", &path, source))?;

        Ok(StoreLock { file })
    }

    /// Take the store's exclusive lock if it is free, without waiting: the
    /// variant a test can assert exclusivity with.
    pub fn try_lock(&self) -> Result<StoreLock, StoreError> {
        let path = self.lock_path();
        let file = open_lock(&path)?;

        match FileExt::try_lock(&file) {
            Ok(()) => Ok(StoreLock { file }),
            Err(TryLockError::WouldBlock) => Err(StoreError::Locked { path }),
            Err(TryLockError::Error(source)) => Err(failure("lock", &path, source)),
        }
    }

    /// Run one command through DEC-004's order and return the bytes to write.
    ///
    /// Steps 1 and 11 are the caller's: the request is parsed before this is
    /// called, and the returned output is written after it returns, which is
    /// after the lock is released (§5.3).
    pub fn run<C: Command>(&self, command: &mut C) -> Result<String, Error> {
        if !self.exists() {
            if !command.creates_store() {
                return Err(StoreError::Absent {
                    path: self.directory(),
                }
                .into());
            }

            // The pre-check runs against an empty snapshot so that a request
            // with nothing to write never creates a store (DEC-004 step 2).
            let precomputed = command.recompute(&Snapshot::empty())?;
            if precomputed.write_set.is_empty() {
                return Ok(command.prepare_output(precomputed.value));
            }

            self.initialise()?;
        }

        let lock = self.lock()?;
        self.recover()?;
        let snapshot = self.load()?;
        let computed = command.recompute(&snapshot)?;

        if !computed.write_set.is_empty() {
            self.commit(&snapshot, &computed.write_set)?;
        }

        let output = command.prepare_output(computed.value);
        drop(lock);

        Ok(output)
    }

    fn lock_path(&self) -> PathBuf {
        self.directory().join(LOCK_BASENAME)
    }

    /// Create `.smut/` and make its entry durable, tolerating a lost race.
    ///
    /// A process using a possibly new `.smut/` syncs the parent entry itself
    /// rather than trusting the process that won `mkdir` (DEC-004). The fsync
    /// goes through [`durable::sync_directory`], the crate's only fsync site, and
    /// carries [`Boundary::InitialisationParentSynced`] so an interruption here is
    /// told apart from the commit path's own parent fsync.
    fn initialise(&self) -> Result<(), StoreError> {
        let directory = self.directory();

        match std::fs::create_dir(&directory) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(failure("create_store_directory", &directory, source)),
        }

        durable::sync_directory(&self.root, Boundary::InitialisationParentSynced)
    }

    /// Step 5: recovery, strictly before validation (DEC-004).
    ///
    /// Recovery runs under the lock and never publishes an index of its own; it
    /// discards a staging state, replays a decided commit, or finishes clearing
    /// a done state (§4.4).
    fn recover(&self) -> Result<(), StoreError> {
        journal::recover(&self.directory())
    }

    /// Step 8: publish the write set as one durable batch.
    ///
    /// The journal stages every entity file and the fresh `index.json`, makes
    /// them durable, renames the staging state to the commit state — the commit
    /// decision — and only then applies (§4.3). This is the one code path that
    /// creates an index.
    fn commit(&self, snapshot: &Snapshot, write_set: &WriteSet) -> Result<(), StoreError> {
        journal::commit(&self.directory(), snapshot, write_set)
    }
}

/// The store's exclusive lock, held for as long as the value lives.
#[derive(Debug)]
#[must_use = "the lock is released when it is dropped"]
pub struct StoreLock {
    file: File,
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// The loaded, validated store: its entities, and the index that explains them.
#[derive(Debug)]
pub struct Snapshot {
    entities: Entities,
    index: Index,
}

impl Snapshot {
    /// An empty snapshot, for a pre-check that runs before the store is read.
    pub fn empty() -> Self {
        Snapshot {
            entities: Entities::new(),
            index: Index::empty(),
        }
    }

    /// Every entity the store holds, keyed and ordered by public id.
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    /// The index that explains the store's files.
    pub fn index(&self) -> &Index {
        &self.index
    }
}

/// What one recomputation produced: the write set to commit, and the value the
/// command turns into output.
pub struct Computed<Value> {
    /// The complete entities to store and the public ids to remove, in public-id
    /// space. Empty means the command writes nothing and the commit step is
    /// skipped.
    pub write_set: WriteSet,
    /// What only the command understands, handed back to it at step 9.
    pub value: Value,
}

/// One command's contribution to DEC-004's order.
///
/// A command supplies what only it knows — whether it may create a store, how to
/// compute against a snapshot, and how to turn that into bytes — and nothing
/// about the order, which the seam owns.
pub trait Command {
    /// What output preparation needs from the recomputation.
    type Value;

    /// Whether this command may create a store that is not there. Only `declare`
    /// answers yes; a read or a delete against an absent store is store-not-found
    /// and creates nothing (DEC-004).
    fn creates_store(&self) -> bool;

    /// Steps 2 and 7: compute against a snapshot. Step 2 runs against an empty
    /// snapshot and is never authoritative; step 7 runs against the validated one
    /// and is.
    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<Self::Value>, Reject>;

    /// Step 9: the bytes this command writes to stdout, prepared in full while
    /// the lock is still held (§5.3).
    fn prepare_output(&self, value: Self::Value) -> String;
}

/// Read one store entry, refusing to resolve a symlink.
///
/// `Ok(None)` means the entry is not there. A symlink is refused rather than
/// followed, so no entry can lead outside `.smut/` (§4.1, §5.2).
fn read_entry(directory: &Path, basename: &str) -> Result<Option<Vec<u8>>, StoreError> {
    let path = directory.join(basename);
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(failure("read_entry", &path, source)),
    };

    if metadata.file_type().is_symlink() {
        return Err(StoreError::SymlinkedEntry {
            entry: basename.to_owned(),
        });
    }

    std::fs::read(&path)
        .map(Some)
        .map_err(|source| failure("read_entry", &path, source))
}

/// Refuse any entry in `.smut/` that the store does not explain (§5.2).
fn refuse_unexplained_entries(directory: &Path, index: &Index) -> Result<(), StoreError> {
    let listing =
        std::fs::read_dir(directory).map_err(|source| failure("list_store", directory, source))?;

    for entry in listing {
        let entry = entry.map_err(|source| failure("list_store", directory, source))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(StoreError::UnexplainedEntry {
                entry: name.to_string_lossy().into_owned(),
            });
        };

        if !is_explained(name, index) {
            return Err(StoreError::UnexplainedEntry {
                entry: name.to_owned(),
            });
        }
    }

    Ok(())
}

/// Whether the store explains `name`: its own names, its journal states, its
/// protocol temporaries, or one of the index's mapped basenames.
fn is_explained(name: &str, index: &Index) -> bool {
    name == LOCK_BASENAME
        || name == INDEX_BASENAME
        || JOURNAL_STATES.contains(&name)
        || durable::is_protocol_temporary(name)
        || index.basenames().any(|basename| basename == name)
}

/// Open or create the lock inode, never truncating it and never replacing it.
fn open_lock(path: &Path) -> Result<File, StoreError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|source| failure("open_lock", path, source))
}

fn failure(step: &'static str, path: &Path, source: io::Error) -> StoreError {
    StoreError::FileSystem {
        step,
        path: path.to_path_buf(),
        source,
    }
}

fn describe(id: Option<&Value>) -> String {
    match id {
        None => "nothing".to_owned(),
        Some(value) => value.to_string(),
    }
}

/// Refuse a loaded entity that `scan_refs` could not read: every object in it
/// that carries `smut_id` must carry a nonempty string (§3, DEC-005).
///
/// This establishes `scan_refs`' precondition — documented there as "a
/// validated, normalised entity" — once, at the load boundary, instead of
/// threading a `Result` through a traversal that returns an iterator. A
/// malformed identity therefore reaches no caller of `scan_refs` at all: it is
/// refused as `Corrupt`, with the path to the offending member, by the load
/// that met it. The walk does not check reference targets, repair anything or
/// treat `smut_id` as data to validate; it asks `reference_id` for the one
/// identity rule and reports what that rule rejects.
///
/// Every object value and every array element is descended into, including
/// through an object whose own identity is valid, because a nested entity node
/// is not a leaf. Objects iterate lexically and arrays positionally, so the
/// first malformed identity is deterministic.
///
/// The cost is one linear pass over the value `load` has already parsed: no
/// second read, no second parse, no clone or re-serialisation of the data, and
/// one `String` path pushed and truncated along the way.
fn validate_identity_tree(entry: &str, value: &Value) -> Result<(), StoreError> {
    let mut path = String::from("$");

    walk_identities(entry, value, &mut path)
}

/// [`validate_identity_tree`]'s walk, carrying the path to the value it stands
/// on: one component is pushed on the way down and truncated on the way back
/// up, so a whole entity costs one `String` however deep it nests.
fn walk_identities(entry: &str, value: &Value, path: &mut String) -> Result<(), StoreError> {
    match value {
        Value::Object(fields) => {
            if reference_id(value).is_err() {
                let container = path.len();
                push_key_component(path, ENTITY_ID);
                let reason = format!("identity at {path} must be a nonempty string");
                path.truncate(container);

                return Err(StoreError::Corrupt {
                    entry: entry.to_owned(),
                    reason,
                });
            }

            for (key, child) in fields {
                let container = path.len();
                push_key_component(path, key);
                walk_identities(entry, child, path)?;
                path.truncate(container);
            }

            Ok(())
        }
        Value::Array(items) => {
            for (position, item) in items.iter().enumerate() {
                let container = path.len();
                path.push('[');
                path.push_str(&position.to_string());
                path.push(']');
                walk_identities(entry, item, path)?;
                path.truncate(container);
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

/// Append one object-key component, JSON-quoted, so that dots, brackets, quotes,
/// backslashes, control characters and the empty key cannot be read as the
/// notation's own separators.
fn push_key_component(path: &mut String, key: &str) {
    path.push('[');
    path.push_str(&serde_json::to_string(key).expect("a string is always representable as JSON"));
    path.push(']');
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_json::Value;

    use super::boundary::{Boundary, Track};
    use crate::error::StoreError;

    fn all_boundaries() -> Vec<Boundary> {
        Boundary::all().collect()
    }

    #[test]
    fn the_registry_is_not_empty_and_both_tracks_are_represented() {
        assert!(!Boundary::COMMIT_PATH.is_empty());
        assert!(!Boundary::RECOVERY_PATH.is_empty());
        assert_eq!(
            all_boundaries().len(),
            Boundary::COMMIT_PATH.len() + Boundary::RECOVERY_PATH.len()
        );
    }

    #[test]
    fn the_registry_lists_each_boundary_once() {
        let all = all_boundaries();
        let unique: HashSet<Boundary> = all.iter().copied().collect();

        assert_eq!(unique.len(), all.len());
    }

    #[test]
    fn every_boundary_name_is_unique_and_round_trips() {
        let all = all_boundaries();
        let names: HashSet<&str> = all.iter().map(|boundary| boundary.as_str()).collect();

        assert_eq!(names.len(), all.len());

        for boundary in all {
            assert_eq!(Boundary::from_name(boundary.as_str()), Some(boundary));
        }
    }

    #[test]
    fn every_name_is_lowercase_snake_case() {
        for boundary in all_boundaries() {
            let name = boundary.as_str();

            assert!(!name.is_empty());
            assert!(
                name.chars()
                    .all(|character| character.is_ascii_lowercase() || character == '_'),
                "{name}"
            );
            assert!(!name.starts_with('_') && !name.ends_with('_'), "{name}");
        }
    }

    #[test]
    fn the_two_groups_agree_with_the_track_each_boundary_reports() {
        for boundary in Boundary::COMMIT_PATH {
            assert_eq!(boundary.track(), Track::Commit, "{boundary:?}");
        }

        for boundary in Boundary::RECOVERY_PATH {
            assert_eq!(boundary.track(), Track::Recovery, "{boundary:?}");
        }
    }

    #[test]
    fn the_barrier_apply_and_recovery_boundaries_are_registered_by_name() {
        for name in [
            "initialisation_parent_synced",
            "staged_put_synced",
            "staged_index_synced",
            "staged_manifest_synced",
            "staging_directory_synced",
            "commit_renamed",
            "parent_directory_synced",
            "puts_applied",
            "index_applied",
            "removals_applied",
            "done_renamed",
            "cleanup_removed",
            "recovery_temporaries_removed",
            "recovery_prepare_discarded",
            "recovery_put_replayed",
            "recovery_done_renamed",
            "recovery_cleanup_removed",
        ] {
            assert!(
                Boundary::from_name(name).is_some(),
                "{name} is not registered"
            );
        }
    }

    #[test]
    fn an_unregistered_name_is_not_a_boundary() {
        assert_eq!(Boundary::from_name("commit_rename"), None);
        assert_eq!(Boundary::from_name(""), None);
    }

    // ── PHASE-01/VT-3/VA-3: the loaded-identity walk ─────────────────────────
    //
    // `validate_identity_tree` is the load boundary's identity walk. It is given
    // an already-parsed entity, asks `crate::entity::reference_id` about every
    // object it meets, and turns that predicate's `Err` into `StoreError::Corrupt`
    // naming where the malformed identity sits. The path is `$` followed by one
    // JSON-quoted bracket component per object key and one numeric component per
    // array position, ending at the offending object's identity member; the
    // reason reads exactly `identity at <path> must be a nonempty string`. Every
    // object value and array element is descended into — including through an
    // object whose own identity is valid — and scalars are ignored. Objects
    // iterate lexically and arrays positionally, so the first refusal is
    // deterministic.
    //
    // These tests drive the helper over parsed values only: no store is opened
    // and nothing touches the filesystem.

    /// The exact refusal a malformed identity earns: `Corrupt` naming the entry
    /// and the position.
    fn refusal_reason(fixture: &str) -> String {
        let value: Value = serde_json::from_str(fixture).expect("the fixture parses");

        match super::validate_identity_tree("Ab3.json", &value)
            .expect_err("a malformed identity is refused")
        {
            StoreError::Corrupt { entry, reason } => {
                assert_eq!(entry, "Ab3.json", "the refusal names the entity file");
                reason
            }
            other => panic!("a malformed identity is corrupt, not {other}"),
        }
    }

    #[test]
    fn a_parsed_entity_with_a_valid_identity_is_accepted() {
        let value: Value = serde_json::from_str(r#"{"smut_id":"alpha","body":"a thought"}"#)
            .expect("the fixture parses");

        assert!(
            super::validate_identity_tree("Ab3.json", &value).is_ok(),
            "an entity whose only identity is a nonempty string is a valid tree"
        );
    }

    #[test]
    fn nested_identities_inside_objects_and_arrays_are_accepted() {
        // Identities at depth, arrays of identity-bearing and identity-free
        // objects, and scalars beside them: the shapes a load must accept.
        let value: Value = serde_json::from_str(
            r#"{"smut_id":"alpha","payload":{"smut_id":"opaque","meta":{"smut_id":"deeper"}},"tags":[{"smut_id":"one"},{"name":"two"},{"smut_id":"three","nested":[{"smut_id":"four"},7,"a",null,true]}]}"#,
        )
        .expect("the fixture parses");

        assert!(
            super::validate_identity_tree("Ab3.json", &value).is_ok(),
            "a tree whose every identity is a nonempty string is valid"
        );
    }

    #[test]
    fn a_whitespace_only_identity_is_still_a_nonempty_string() {
        for fixture in [
            r#"{"smut_id":" "}"#,
            r#"{"smut_id":"alpha","payload":{"smut_id":"\t"}}"#,
        ] {
            let value: Value = serde_json::from_str(fixture).expect("the fixture parses");

            assert!(
                super::validate_identity_tree("Ab3.json", &value).is_ok(),
                "'a nonempty string' is the whole rule: {fixture}"
            );
        }
    }

    #[test]
    fn scalars_below_the_root_are_ignored() {
        let value: Value = serde_json::from_str(
            r#"{"smut_id":"alpha","n":1,"f":1.5,"t":true,"z":null,"s":"x","arr":[1,"two",null,false],"deep":[[[{"note":"no identity here"}]]]}"#,
        )
        .expect("the fixture parses");

        assert!(
            super::validate_identity_tree("Ab3.json", &value).is_ok(),
            "scalars carry no identity, so there is nothing to refuse"
        );
    }

    #[test]
    fn a_malformed_identity_below_the_root_names_its_path_and_the_rule() {
        // One path, five shapes: the refusal is the same rule whatever stands
        // where an identity belongs.
        for identity in ["null", r#""""#, "0", "[]", "{}"] {
            let fixture = format!(r#"{{"smut_id":"alpha","payload":{{"smut_id":{identity}}}}}"#);

            assert_eq!(
                refusal_reason(&fixture),
                r#"identity at $["payload"]["smut_id"] must be a nonempty string"#,
                "the identity {identity} is malformed"
            );
        }
    }

    #[test]
    fn an_identity_inside_an_array_reports_its_position() {
        assert_eq!(
            refusal_reason(r#"{"smut_id":"alpha","payload":{"tags":["a","b",{"smut_id":""}]}}"#),
            r#"identity at $["payload"]["tags"][2]["smut_id"] must be a nonempty string"#
        );
    }

    #[test]
    fn the_root_object_is_walked_too() {
        // `Store::load`'s own root check owns this case in the real loader, but
        // the walk is handed the root object and validates it like any other.
        for fixture in [r#"{"smut_id":""}"#, r#"{"smut_id":null}"#] {
            assert_eq!(
                refusal_reason(fixture),
                r#"identity at $["smut_id"] must be a nonempty string"#,
                "{fixture}"
            );
        }
    }

    #[test]
    fn a_valid_reference_shaped_object_does_not_stop_the_walk() {
        // `{"smut_id":"opaque"}` is what `reference_id` reads as an entity node, so
        // it is the shape a walk could mistake for a leaf. The malformed
        // identity deeper inside is what must be refused.
        assert_eq!(
            refusal_reason(
                r#"{"smut_id":"alpha","payload":{"smut_id":"opaque","meta":{"smut_id":null}}}"#,
            ),
            r#"identity at $["payload"]["meta"]["smut_id"] must be a nonempty string"#
        );
    }

    #[test]
    fn identity_paths_escape_object_keys_and_number_array_positions() {
        // Each key is a distinct escaping case: a dot, a bracket pair, a double
        // quote, a backslash, a control character, and the empty key. The six
        // object components and the numeric array component are pinned verbatim.
        assert_eq!(
            refusal_reason(
                r#"{"smut_id":"alpha",".":{"[x]":{"a\"b":{"a\\b":{"\u0001":{"":[{"smut_id":null}]}}}}}}"#,
            ),
            r#"identity at $["."]["[x]"]["a\"b"]["a\\b"]["\u0001"][""][0]["smut_id"] must be a nonempty string"#
        );
    }

    #[test]
    fn the_lexically_first_object_key_reports_the_identity() {
        // Both members are malformed, and the fixture writes `zzz` first, so only
        // the map's lexical iteration order can report `aaa`.
        assert_eq!(
            refusal_reason(r#"{"smut_id":"alpha","zzz":{"smut_id":""},"aaa":{"smut_id":[]}}"#),
            r#"identity at $["aaa"]["smut_id"] must be a nonempty string"#
        );
    }

    #[test]
    fn the_earliest_array_position_reports_the_identity() {
        assert_eq!(
            refusal_reason(
                r#"{"smut_id":"alpha","tags":["a",{"smut_id":null},"b",{"smut_id":""}]}"#
            ),
            r#"identity at $["tags"][1]["smut_id"] must be a nonempty string"#
        );
    }

    #[test]
    fn a_deep_chain_reports_the_whole_traversal_path() {
        // Forty alternating wrappers: the reported path carries every component
        // the walk descended through, which a path built along the walk gives and
        // a subtree rebuilt at the failure would not.
        let mut fixture = String::from(r#"{"smut_id":null}"#);
        let mut steps: Vec<&str> = Vec::new();
        for depth in 0..40 {
            if depth % 2 == 0 {
                fixture = format!(r#"{{"n":{fixture}}}"#);
                steps.push(r#"["n"]"#);
            } else {
                fixture = format!("[{fixture}]");
                steps.push("[0]");
            }
        }

        let mut path = String::from("$");
        for step in steps.iter().rev() {
            path.push_str(step);
        }
        path.push_str(r#"["smut_id"]"#);

        assert_eq!(
            refusal_reason(&fixture),
            format!("identity at {path} must be a nonempty string")
        );
    }

    #[test]
    fn the_walk_asks_the_shared_identity_predicate_rather_than_reimplementing_it() {
        // Each fixture carries its identity at the root, so the predicate and the
        // walk judge the same object: a walk that kept its own rule would drift
        // away from `reference_id` on at least one row.
        let candidates = [
            r#"{"smut_id":"ok"}"#,
            r#"{"smut_id":" "}"#,
            r#"{"smut_id":""}"#,
            r#"{"smut_id":null}"#,
            r#"{"smut_id":0}"#,
            r#"{"smut_id":[]}"#,
            r#"{"smut_id":{}}"#,
            "{}",
            r#"{"other":1}"#,
        ];

        let mut refused = 0;
        for fixture in candidates {
            let value: Value = serde_json::from_str(fixture).expect("the fixture parses");
            let refused_by_predicate = crate::entity::reference_id(&value).is_err();
            let refused_by_walk = super::validate_identity_tree("Ab3.json", &value).is_err();

            assert_eq!(refused_by_predicate, refused_by_walk, "{fixture}");

            if refused_by_walk {
                refused += 1;
            }
        }

        assert_eq!(refused, 5, "the table must exercise both answers, not one");
    }

    #[test]
    fn a_large_nested_array_of_identity_bearing_objects_is_accepted() {
        // A blanket refusal, or a walk that gave up after a few levels, fails
        // here: every identity in this tree is a nonempty string.
        let items: Vec<String> = (0..200)
            .map(|index| {
                format!(
                    r#"{{"smut_id":"e{index}","tags":[{{"smut_id":"t{index}"}},{{"note":"n{index}"}}]}}"#
                )
            })
            .collect();
        let fixture = format!(r#"{{"smut_id":"alpha","items":[{}]}}"#, items.join(","));
        let value: Value = serde_json::from_str(&fixture).expect("the fixture parses");

        assert!(
            super::validate_identity_tree("Ab3.json", &value).is_ok(),
            "a store-shaped entity with many identity-bearing objects is valid"
        );
    }
}

/// The store directory beside the working directory (§4.1).
pub const STORE_DIRECTORY: &str = ".smut";

/// The permanent lock inode: created once, never replaced, renamed or unlinked.
pub const LOCK_BASENAME: &str = "lock";

/// The committed index, the only entry that explains an entity file.
pub const INDEX_BASENAME: &str = "index.json";

/// A journal state's manifest, named inside the state directory.
pub const MANIFEST_BASENAME: &str = "manifest.json";

/// The three journal states; the state is the directory name (DEC-003).
pub const JOURNAL_STATES: [&str; 3] = ["journal.prepare", "journal.commit", "journal.done"];

#[cfg(feature = "fault-injection")]
pub mod fault;
pub mod fs;
pub mod index;
pub mod journal;

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use fs4::{FileExt, TryLockError};
use serde_json::Value;

use crate::declare::{Entities, WriteSet};
use crate::entity::{ENTITY_ID, reference_id};
use crate::error::{Error, Reject, StoreError};
use crate::store::boundary::Boundary;
use crate::store::fs as durable;
use crate::store::index::Index;

#[cfg(test)]
mod store_tests {
    use super::*;

    use std::cell::RefCell;
    use std::fs;

    use crate::id::Id;

    /// One entity and its index entry: the smallest healthy store.
    const ENTITY: &str = "{\"smut_id\":\"alpha\",\"body\":\"a thought\"}";

    fn fresh_index() -> String {
        Index::new([(Id::new("alpha"), "Ab3.json".to_owned())])
            .expect("a safe basename")
            .to_bytes()
    }

    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

            let path = std::env::temp_dir().join(format!(
                "smut-store-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("a scratch directory");

            Self { path }
        }

        fn directory(&self) -> PathBuf {
            self.path.join(STORE_DIRECTORY)
        }

        fn store(&self) -> Store {
            Store::in_directory(&self.path)
        }

        fn create_store(&self) {
            fs::create_dir(self.directory()).expect("the store directory");
        }

        fn write(&self, name: &str, contents: &str) {
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

        /// A store with one indexed entity, ready to load.
        fn healthy(&self) {
            self.create_store();
            self.write(INDEX_BASENAME, &fresh_index());
            self.write("Ab3.json", ENTITY);
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn the_lock_is_exclusive_and_is_released_when_it_is_dropped() {
        let scratch = Scratch::new();
        scratch.create_store();

        let held = scratch.store().lock().expect("the first lock");
        let refusal = scratch
            .store()
            .try_lock()
            .expect_err("the lock is already held");

        assert!(matches!(refusal, StoreError::Locked { .. }), "{refusal}");

        drop(held);

        let afterwards = scratch.store().try_lock().expect("the lock is free");
        drop(afterwards);
    }

    #[test]
    fn the_lock_file_is_created_once_and_kept() {
        let scratch = Scratch::new();
        scratch.create_store();

        let held = scratch.store().lock().expect("the lock");
        drop(held);
        let again = scratch.store().lock().expect("the same lock");
        drop(again);

        assert_eq!(scratch.listing(), ["lock"]);
    }

    #[test]
    fn a_store_that_is_not_there_is_refused_rather_than_read_as_empty() {
        let scratch = Scratch::new();

        let error = scratch.store().load().expect_err("there is no store");

        assert!(matches!(error, StoreError::Absent { .. }), "{error}");
        assert!(!scratch.directory().exists(), "the loader created nothing");
    }

    #[test]
    fn the_loader_returns_every_entity_the_index_explains() {
        let scratch = Scratch::new();
        scratch.healthy();

        let snapshot = scratch.store().load().expect("a healthy store");

        assert_eq!(snapshot.entities().len(), 1);
        assert_eq!(
            snapshot.entities()[&Id::new("alpha")]["body"],
            serde_json::json!("a thought")
        );
        assert_eq!(
            snapshot.index().basename(&Id::new("alpha")),
            Some("Ab3.json")
        );
    }

    #[test]
    fn the_loader_performs_no_writes() {
        let scratch = Scratch::new();
        scratch.healthy();

        let index_path = scratch.directory().join(INDEX_BASENAME);
        let index_bytes = fs::read(&index_path).expect("the index");
        let before = scratch.listing();

        durable::call_record::take();
        let snapshot = scratch.store().load().expect("a healthy store");
        let calls = durable::call_record::take();

        assert_eq!(snapshot.entities().len(), 1);
        assert!(calls.is_empty(), "load-and-validate wrote: {calls:?}");
        assert_eq!(fs::read(&index_path).expect("the index"), index_bytes);
        assert_eq!(scratch.listing(), before);
    }

    #[test]
    fn an_entry_the_index_does_not_explain_is_refused_naming_it() {
        let scratch = Scratch::new();
        scratch.healthy();
        scratch.write("Zq7.json", "{\"id\":\"stray\"}");

        let error = scratch
            .store()
            .load()
            .expect_err("Zq7.json is not explained");

        assert!(
            matches!(error, StoreError::UnexplainedEntry { .. }),
            "{error}"
        );
        assert!(error.to_string().contains("Zq7.json"), "{error}");
    }

    #[test]
    fn a_top_level_manifest_is_not_an_explained_entry() {
        let scratch = Scratch::new();
        scratch.healthy();
        scratch.write(MANIFEST_BASENAME, "{\"format_version\":1}");

        let error = scratch
            .store()
            .load()
            .expect_err("a manifest belongs inside a journal state");

        assert!(
            matches!(error, StoreError::UnexplainedEntry { .. }),
            "{error}"
        );
        assert!(error.to_string().contains(MANIFEST_BASENAME), "{error}");
    }

    #[test]
    fn the_journal_states_and_the_temporary_namespace_are_explained() {
        let scratch = Scratch::new();
        scratch.healthy();
        fs::create_dir(scratch.directory().join("journal.commit")).expect("a journal state");
        scratch.write("Zq7.json.tmp", "{}");

        let snapshot = scratch.store().load().expect("a store mid-recovery");

        assert_eq!(snapshot.entities().len(), 1);
    }

    #[test]
    fn an_index_less_store_holding_entity_files_is_refused_not_read_as_empty() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write("Ab3.json", ENTITY);

        let error = scratch
            .store()
            .load()
            .expect_err("an unindexed entity file is never adopted");

        assert!(
            matches!(error, StoreError::UnexplainedEntry { .. }),
            "{error}"
        );
        assert!(error.to_string().contains("Ab3.json"), "{error}");
        assert_eq!(scratch.listing(), ["Ab3.json"], "nothing was repaired");
    }

    #[test]
    fn an_index_less_store_holding_only_its_lock_is_an_empty_store() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write(LOCK_BASENAME, "");

        let snapshot = scratch.store().load().expect("an empty store");

        assert!(snapshot.entities().is_empty());
        assert!(snapshot.index().is_empty());
    }

    #[test]
    fn an_id_that_disagrees_with_its_index_key_names_the_file_and_both_ids() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write(INDEX_BASENAME, &fresh_index());
        scratch.write("Ab3.json", "{\"smut_id\":\"beta\"}");

        let error = scratch.store().load().expect_err("the file says beta");

        assert!(
            matches!(error, StoreError::IdDisagreement { .. }),
            "{error}"
        );
        for named in ["Ab3.json", "alpha", "beta"] {
            assert!(error.to_string().contains(named), "{error}");
        }
    }

    #[test]
    fn a_malformed_entity_file_is_refused_rather_than_repaired() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write(INDEX_BASENAME, &fresh_index());
        scratch.write("Ab3.json", "{\"id\":");

        let error = scratch.store().load().expect_err("the file is torn");

        assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
        assert!(error.to_string().contains("Ab3.json"), "{error}");
    }

    #[test]
    fn an_entity_file_that_is_not_an_object_is_refused() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write(INDEX_BASENAME, &fresh_index());
        scratch.write("Ab3.json", "[]");

        let error = scratch.store().load().expect_err("an entity is an object");

        assert!(matches!(error, StoreError::Corrupt { .. }), "{error}");
        assert!(error.to_string().contains("Ab3.json"), "{error}");
    }

    #[test]
    #[cfg(unix)]
    fn a_symlinked_entity_file_is_refused_rather_than_followed() {
        let scratch = Scratch::new();
        scratch.create_store();
        scratch.write(INDEX_BASENAME, &fresh_index());
        let outside = scratch.path.join("outside.json");
        fs::write(&outside, ENTITY).expect("an entity outside the store");
        std::os::unix::fs::symlink(&outside, scratch.directory().join("Ab3.json"))
            .expect("a symlinked entry");

        let error = scratch.store().load().expect_err("the entry is a symlink");

        assert!(
            matches!(error, StoreError::SymlinkedEntry { .. }),
            "{error}"
        );
        assert!(error.to_string().contains("Ab3.json"), "{error}");
    }

    #[test]
    #[cfg(unix)]
    fn a_symlinked_index_is_refused_rather_than_followed() {
        let scratch = Scratch::new();
        scratch.create_store();
        let outside = scratch.path.join("outside-index.json");
        fs::write(&outside, fresh_index()).expect("an index outside the store");
        std::os::unix::fs::symlink(&outside, scratch.directory().join(INDEX_BASENAME))
            .expect("a symlinked index");

        let error = scratch.store().load().expect_err("the index is a symlink");

        assert!(
            matches!(error, StoreError::SymlinkedEntry { .. }),
            "{error}"
        );
        assert!(error.to_string().contains(INDEX_BASENAME), "{error}");
    }

    /// A command whose recomputation only observes the seam around it.
    struct Probe<'a> {
        store: &'a Store,
        creates: bool,
        write_set: WriteSet,
        /// Whether the store's lock was held each time recompute ran.
        lock_held: RefCell<Vec<bool>>,
        /// The entity count recompute saw, once per pass.
        counts: RefCell<Vec<usize>>,
    }

    impl<'a> Probe<'a> {
        fn new(store: &'a Store, creates: bool, write_set: WriteSet) -> Self {
            Probe {
                store,
                creates,
                write_set,
                lock_held: RefCell::new(Vec::new()),
                counts: RefCell::new(Vec::new()),
            }
        }

        fn probe(&self, snapshot: &Snapshot) -> Computed<usize> {
            self.counts.borrow_mut().push(snapshot.entities().len());
            self.lock_held.borrow_mut().push(matches!(
                self.store.try_lock(),
                Err(StoreError::Locked { .. })
            ));

            Computed {
                write_set: self.write_set.clone(),
                value: snapshot.entities().len(),
            }
        }
    }

    impl Command for Probe<'_> {
        type Value = usize;

        fn creates_store(&self) -> bool {
            self.creates
        }

        fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<usize>, Reject> {
            Ok(self.probe(snapshot))
        }

        fn prepare_output(&self, value: usize) -> String {
            format!("{value} entities\n")
        }
    }

    fn one_put() -> WriteSet {
        WriteSet {
            put: vec![serde_json::json!({"smut_id": "alpha"})],
            remove: Vec::new(),
        }
    }

    #[test]
    fn a_declaration_with_nothing_to_write_never_creates_the_store() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let mut probe = Probe::new(&store, true, WriteSet::default());

        let output = store.run(&mut probe).expect("no write, no store");

        assert_eq!(output, "0 entities\n");
        assert!(!scratch.directory().exists(), "the store was never created");
        assert_eq!(
            *probe.counts.borrow(),
            [0],
            "one pass, against an empty snapshot"
        );
    }

    #[test]
    fn a_read_against_an_absent_store_is_store_not_found_and_creates_nothing() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let mut probe = Probe::new(&store, false, WriteSet::default());

        let error = store
            .run(&mut probe)
            .expect_err("there is no store to read");

        assert!(
            matches!(error, Error::Store(StoreError::Absent { .. })),
            "{error}"
        );
        assert_eq!(error.exit_code(), 1);
        assert!(!scratch.directory().exists(), "nothing was created");
    }

    #[test]
    fn the_seam_holds_the_lock_from_recovery_through_output_preparation() {
        let scratch = Scratch::new();
        scratch.healthy();
        let store = scratch.store();
        let mut probe = Probe::new(&store, false, WriteSet::default());

        let output = store.run(&mut probe).expect("a read");

        assert_eq!(output, "1 entities\n");
        assert_eq!(
            *probe.counts.borrow(),
            [1],
            "recompute saw the loaded store"
        );
        assert_eq!(*probe.lock_held.borrow(), [true], "the lock covers reads");

        let released = scratch
            .store()
            .try_lock()
            .expect("the seam released the lock");
        drop(released);
    }

    #[test]
    fn a_write_set_is_published_as_one_durable_batch() {
        let scratch = Scratch::new();
        scratch.healthy();
        let store = scratch.store();
        let mut probe = Probe::new(&store, false, one_put());

        let output = store.run(&mut probe).expect("the batch commits");

        assert_eq!(output, "1 entities\n");

        let snapshot = store.load().expect("the committed store");
        assert_eq!(
            snapshot.entities()[&Id::new("alpha")],
            serde_json::json!({"smut_id": "alpha"}),
            "the put replaced the stored entity"
        );
        assert_eq!(
            snapshot.index().basename(&Id::new("alpha")),
            Some("Ab3.json"),
            "an existing id keeps its uid"
        );
        assert_eq!(
            scratch.listing(),
            ["Ab3.json", "index.json", "lock"],
            "the journal cleared itself and published nothing else"
        );
    }

    #[cfg(feature = "fault-injection")]
    #[test]
    fn initialisation_reaches_its_own_parent_directory_boundary() {
        use crate::store::fault::{Faults, install, reset};

        let scratch = Scratch::new();
        let store = scratch.store();

        install(Faults::failing_at(Boundary::InitialisationParentSynced));
        let refused = store.initialise();
        reset();

        let error = refused.expect_err("the parent fsync is armed to fail");
        assert!(
            error.to_string().contains("initialisation_parent_synced"),
            "{error}"
        );

        install(Faults::failing_at(Boundary::ParentDirectorySynced));
        let second = store.initialise();
        reset();

        second.expect("initialisation does not use the commit path's fsync");
    }

    #[test]
    fn a_first_declaration_initialises_the_store_and_publishes_its_first_index() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let mut probe = Probe::new(&store, true, one_put());

        let output = store.run(&mut probe).expect("the first batch commits");

        assert_eq!(output, "0 entities\n");

        let snapshot = store.load().expect("the committed store");
        assert_eq!(snapshot.entities().len(), 1);
        let basename = snapshot
            .index()
            .basename(&Id::new("alpha"))
            .expect("a fresh uid")
            .to_owned();
        assert!(basename.ends_with(".json"), "{basename}");

        let mut expected = vec![
            basename,
            INDEX_BASENAME.to_owned(),
            LOCK_BASENAME.to_owned(),
        ];
        expected.sort();
        assert_eq!(scratch.listing(), expected);
        assert_eq!(
            *probe.counts.borrow(),
            [0, 0],
            "the pre-check ran against an empty snapshot, the authoritative pass after"
        );
        assert_eq!(
            *probe.lock_held.borrow(),
            [false, true],
            "the pre-check runs without a lock, the authoritative pass under it"
        );
    }

    #[test]
    fn initialisation_creates_the_store_directory_and_no_index() {
        let scratch = Scratch::new();
        let store = scratch.store();

        store.initialise().expect("the store directory");

        assert!(scratch.directory().is_dir());
        assert_eq!(
            scratch.listing(),
            Vec::<String>::new(),
            "initialisation publishes nothing; the first commit stages the index"
        );
    }
}
