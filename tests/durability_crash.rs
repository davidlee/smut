#![cfg(feature = "fault-injection")]
// The double underscore separates the obligation from the boundary it reaches —
// `interrupted_at__<boundary>`, `recovery_interrupted__<step>` — so each
// obligation is greppable and `the_suite_covers_every_registered_boundary` can
// check it against the registry. rustc's casing lint does not know that
// convention, and it is the only casing in this file that trips it.
#![allow(non_snake_case)]

//! The journal's interruption evidence (§4.3–§4.5, DEC-003).
//!
//! Every interruption here is at a named boundary from
//! [`smut::store::boundary::Boundary`], armed through the fault seam: an injected
//! failure for a step that returns an error, and a real abort — a child process
//! of this test binary — for a step that stops the process. The parent then
//! inspects what the crash left behind and requires the next command to converge
//! on the whole batch or on the pre-commit state, never on a mixture.
//!
//! **Code-path ordering is asserted by injection; power-loss durability is
//! assumed, not measured.** Injection proves this code issues those calls in that
//! order and reacts as the design says when they fail; it cannot prove that the
//! kernel or the disk honours them, and nothing here claims it does.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitStatus};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use smut::declare::WriteSet;
use smut::entity::{Form, serialise};
use smut::error::Reject;
use smut::id::Id;
use smut::store::boundary::Boundary;
use smut::store::fault;
use smut::store::index::Index;
use smut::store::journal::{COMMIT_STATE, DONE_STATE, MANIFEST_FORMAT_VERSION, PREPARE_STATE};
use smut::store::{
    Command, Computed, INDEX_BASENAME, LOCK_BASENAME, MANIFEST_BASENAME, STORE_DIRECTORY, Snapshot,
    Store,
};

/// The signal a real abort ends a process with.
const SIGABRT: i32 = 6;

/// The environment variable that names the scenario the child runs.
const CHILD_SCENARIO: &str = "SMUT_TEST_SCENARIO";

/// The environment variable that carries the store path to the child.
const CHILD_STORE: &str = "SMUT_TEST_STORE";

/// The pre-commit store's two entity files: `alpha` and `beta`.
const ALPHA_FILE: &str = "Ab3.json";
const BETA_FILE: &str = "Zq7.json";

/// The uid the hand-built staging fixtures use for the entity the batch adds.
const STAGED_UID: &str = "Fresh.json";

fn entity(id: &str, body: &str) -> Value {
    json!({ "id": id, "body": body })
}

fn entity_bytes(value: &Value) -> Vec<u8> {
    serialise(value, Form::File).into_bytes()
}

fn index_bytes(objects: &[(&str, &str)]) -> Vec<u8> {
    let mut index = Index::empty();

    for (id, basename) in objects {
        index
            .insert(Id::new(*id), (*basename).to_owned())
            .expect("a safe basename");
    }

    index.to_bytes().into_bytes()
}

/// The batch every interruption test drives: update `alpha`, add `gamma`, delete
/// `beta` — one put on a mapped file, one on a fresh uid, one removal.
fn the_batch() -> WriteSet {
    WriteSet {
        put: vec![entity("alpha", "changed"), entity("gamma", "new")],
        remove: vec![Id::new("beta")],
    }
}

/// A scratch directory holding one store, removed when it is dropped.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let root = std::env::temp_dir().join(format!(
            "smut-durability-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("a scratch directory");

        Self { root }
    }

    /// A store with two indexed entities, published and valid.
    fn seeded() -> Self {
        let scratch = Self::new();
        seed(&scratch.root);

        scratch
    }

    fn directory(&self) -> PathBuf {
        self.root.join(STORE_DIRECTORY)
    }

    fn store(&self) -> Store {
        Store::in_directory(&self.root)
    }

    /// Every entry under `.smut/`, keyed by its relative path, with its bytes.
    fn tree(&self) -> BTreeMap<String, Vec<u8>> {
        tree(&self.directory())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn seed(root: &Path) {
    let directory = root.join(STORE_DIRECTORY);
    fs::create_dir_all(&directory).expect("the store directory");
    fs::write(directory.join(LOCK_BASENAME), b"").expect("the lock");
    fs::write(
        directory.join(ALPHA_FILE),
        entity_bytes(&entity("alpha", "first")),
    )
    .expect("alpha's file");
    fs::write(
        directory.join(BETA_FILE),
        entity_bytes(&entity("beta", "second")),
    )
    .expect("beta's file");
    fs::write(
        directory.join(INDEX_BASENAME),
        index_bytes(&[("alpha", ALPHA_FILE), ("beta", BETA_FILE)]),
    )
    .expect("the index");
}

fn tree(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(directory: &Path, prefix: &str, into: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .expect("a store directory")
            .map(|entry| entry.expect("a directory entry"))
            .collect();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let key = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };

            if path.is_dir() {
                walk(&path, &key, into);
            } else {
                into.insert(key, fs::read(&path).expect("a store entry"));
            }
        }
    }

    let mut into = BTreeMap::new();

    if directory.is_dir() {
        walk(directory, "", &mut into);
    }

    into
}

/// The bytes the canonical batch stages, in the order the commit stages them.
fn staged_files() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (ALPHA_FILE, entity_bytes(&entity("alpha", "changed"))),
        (STAGED_UID, entity_bytes(&entity("gamma", "new"))),
        (
            INDEX_BASENAME,
            index_bytes(&[("alpha", ALPHA_FILE), ("gamma", STAGED_UID)]),
        ),
    ]
}

/// The manifest the canonical batch stages, written by hand so the reader is
/// tested against the format rather than against its own writer.
fn staged_manifest() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "format_version": MANIFEST_FORMAT_VERSION,
        "puts": [ALPHA_FILE, STAGED_UID, INDEX_BASENAME],
        "removes": [BETA_FILE],
    }))
    .expect("a manifest")
}

fn write_state(scratch: &Scratch, state: &str, manifest: &[u8], files: &[(&str, Vec<u8>)]) {
    let path = scratch.directory().join(state);
    fs::create_dir_all(&path).expect("a journal state");
    fs::write(path.join(MANIFEST_BASENAME), manifest).expect("the manifest");

    for (name, bytes) in files {
        fs::write(path.join(name), bytes).expect("a staged file");
    }
}

fn stage_the_batch(scratch: &Scratch, state: &str) {
    write_state(scratch, state, &staged_manifest(), &staged_files());
}

/// Publish the batch by hand: the state a store is in once apply has finished.
fn publish_the_batch(scratch: &Scratch) {
    for (name, bytes) in staged_files() {
        fs::write(scratch.directory().join(name), bytes).expect("a published file");
    }

    fs::remove_file(scratch.directory().join(BETA_FILE)).expect("the removal");
}

/// A command whose recomputation is the canonical batch, so the pipeline can be
/// driven without the declaration core.
struct Batch {
    write_set: WriteSet,
}

impl Batch {
    fn new() -> Self {
        Batch {
            write_set: the_batch(),
        }
    }

    fn of(write_set: WriteSet) -> Self {
        Batch { write_set }
    }
}

impl Command for Batch {
    type Value = ();

    fn creates_store(&self) -> bool {
        true
    }

    fn recompute(&mut self, _snapshot: &Snapshot) -> Result<Computed<()>, Reject> {
        Ok(Computed {
            write_set: self.write_set.clone(),
            value: (),
        })
    }

    fn prepare_output(&self, (): ()) -> String {
        String::from("{}\n")
    }
}

/// The read every recovery is reached by: it holds the lock, recovers, then
/// reads, in DEC-004's order.
struct Read;

impl Command for Read {
    type Value = usize;

    fn creates_store(&self) -> bool {
        false
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<usize>, Reject> {
        Ok(Computed {
            write_set: WriteSet::default(),
            value: snapshot.entities().len(),
        })
    }

    fn prepare_output(&self, count: usize) -> String {
        format!("{count}\n")
    }
}

fn is_child() -> bool {
    std::env::var_os(CHILD_SCENARIO).is_some()
}

/// Run the scenario the armed seam is about to abort.
///
/// This never returns on success: the child either aborts at the boundary the
/// parent armed or panics, and the parent tells those two exits apart.
fn run_child() -> ! {
    let root = PathBuf::from(std::env::var(CHILD_STORE).expect("the store path"));
    let store = Store::in_directory(&root);
    let scenario = std::env::var(CHILD_SCENARIO).expect("the scenario");

    match scenario.as_str() {
        "declare" => {
            let _ = store.run(&mut Batch::new());
        }
        "read" => {
            let _ = store.run(&mut Read);
        }
        other => panic!("unknown scenario: {other}"),
    }

    panic!("the seam did not abort at the armed boundary");
}

/// Spawn this test binary as a child that runs `scenario` against `root` with the
/// seam armed to abort at `boundary`, and require the abort.
fn abort_at(test: &str, boundary: Boundary, root: &Path, scenario: &str) {
    let status: ExitStatus = Process::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(fault::ABORT_VARIABLE, boundary.as_str())
        .env(CHILD_STORE, root)
        .env(CHILD_SCENARIO, scenario)
        .status()
        .expect("the child test binary runs");

    assert_eq!(
        status.code(),
        None,
        "{test}: the child was not killed by a signal"
    );
    assert_eq!(
        status.signal(),
        Some(SIGABRT),
        "{test}: the child did not abort at {}",
        boundary.as_str()
    );
}

/// Run one command against the store, which recovers whatever the crash left.
fn recover(scratch: &Scratch) {
    scratch
        .store()
        .run(&mut Read)
        .expect("the next command recovers the store");
}

/// The state the same fixture settles in when one command recovers it without
/// interruption: what an interrupted recovery has to converge on.
fn settled(seed_fixture: impl Fn(&Scratch)) -> BTreeMap<String, Vec<u8>> {
    let twin = Scratch::new();
    seed_fixture(&twin);
    recover(&twin);

    twin.tree()
}

/// The whole batch, and nothing else, is published.
fn assert_whole_batch(scratch: &Scratch) {
    let snapshot = scratch.store().load().expect("a healthy store");
    let uid = snapshot
        .index()
        .basename(&Id::new("gamma"))
        .expect("the added entity is indexed")
        .to_owned();

    assert_eq!(snapshot.entities().len(), 2, "the batch is whole");
    assert_eq!(
        snapshot.entities()[&Id::new("alpha")]["body"],
        json!("changed"),
        "the update is published"
    );
    assert_eq!(
        snapshot.entities()[&Id::new("gamma")]["body"],
        json!("new"),
        "the addition is published"
    );
    assert_eq!(
        snapshot.index().basename(&Id::new("beta")),
        None,
        "the removal is published"
    );

    let expected = BTreeMap::from([
        (LOCK_BASENAME.to_owned(), Vec::new()),
        (
            INDEX_BASENAME.to_owned(),
            index_bytes(&[("alpha", ALPHA_FILE), ("gamma", &uid)]),
        ),
        (
            ALPHA_FILE.to_owned(),
            entity_bytes(&entity("alpha", "changed")),
        ),
        (uid, entity_bytes(&entity("gamma", "new"))),
    ]);

    assert_eq!(
        scratch.tree(),
        expected,
        "the store holds the whole batch and no journal state"
    );
}

fn assert_pre_commit(scratch: &Scratch) {
    assert_eq!(
        scratch.tree(),
        Scratch::seeded().tree(),
        "the store is exactly as it was before the batch"
    );
}

/// What a crash at a boundary is allowed to leave behind.
enum Crash {
    /// Nothing published: the staging state is on disk and the commit decision
    /// has not been taken.
    BeforeDecision,
    /// The decision is on disk and the batch must roll forward.
    AfterDecision,
}

fn assert_crash_state(scratch: &Scratch, crash: Crash) {
    let tree = scratch.tree();
    let has = |name: &str| {
        tree.keys()
            .any(|key| key == name || key.starts_with(&format!("{name}/")))
    };

    match crash {
        Crash::BeforeDecision => {
            assert!(has(PREPARE_STATE), "the staging state is what is left");
            assert!(!has(COMMIT_STATE), "the decision was not taken");
            assert!(!has(DONE_STATE));

            let published: BTreeMap<&String, &Vec<u8>> = tree
                .iter()
                .filter(|(key, _)| !key.starts_with("journal."))
                .collect();
            let before = Scratch::seeded().tree();

            assert_eq!(
                published.len(),
                before.len(),
                "an interruption before the decision published something: {published:?}"
            );
            for (name, bytes) in &before {
                assert_eq!(published.get(name), Some(&bytes), "{name} changed");
            }
        }
        Crash::AfterDecision => {
            assert!(!has(PREPARE_STATE), "the staging state is gone");
            assert!(
                has(COMMIT_STATE) || has(DONE_STATE),
                "the decision is on disk: {:?}",
                tree.keys().collect::<Vec<_>>()
            );
        }
    }
}

// ── the commit path, interrupted at every named boundary (§4.3, EX-5) ─────────

#[test]
fn interrupted_at__initialisation_parent_synced() {
    if is_child() {
        run_child();
    }

    let scratch = Scratch::new();

    abort_at(
        "interrupted_at__initialisation_parent_synced",
        Boundary::InitialisationParentSynced,
        &scratch.root,
        "declare",
    );

    assert!(
        scratch.directory().is_dir(),
        "the directory was created before its parent fsync"
    );
    assert_eq!(
        scratch.tree(),
        BTreeMap::new(),
        "an interrupted initialisation published nothing"
    );

    scratch
        .store()
        .run(&mut Batch::of(WriteSet {
            put: vec![entity("gamma", "new")],
            remove: Vec::new(),
        }))
        .expect("the next declare initialises and commits");

    let snapshot = scratch.store().load().expect("a healthy store");
    let uid = snapshot
        .index()
        .basename(&Id::new("gamma"))
        .expect("the first declaration's entity is indexed")
        .to_owned();
    let expected = BTreeMap::from([
        (LOCK_BASENAME.to_owned(), Vec::new()),
        (INDEX_BASENAME.to_owned(), index_bytes(&[("gamma", &uid)])),
        (uid, entity_bytes(&entity("gamma", "new"))),
    ]);

    assert_eq!(
        scratch.tree(),
        expected,
        "the first declaration published its whole batch"
    );
}

/// An interruption before the decision: nothing was published, so the store is
/// exactly as it was before the batch.
fn interruption_before_the_decision(test: &str, boundary: Boundary) {
    if is_child() {
        run_child();
    }

    let scratch = Scratch::seeded();

    abort_at(test, boundary, &scratch.root, "declare");

    assert_crash_state(&scratch, Crash::BeforeDecision);
    recover(&scratch);
    assert_pre_commit(&scratch);
}

/// An interruption after the decision: the batch must roll forward, whole.
fn interruption_after_the_decision(test: &str, boundary: Boundary) {
    if is_child() {
        run_child();
    }

    let scratch = Scratch::seeded();

    abort_at(test, boundary, &scratch.root, "declare");

    assert_crash_state(&scratch, Crash::AfterDecision);
    recover(&scratch);
    assert_whole_batch(&scratch);
}

#[test]
fn interrupted_at__staged_put_written() {
    interruption_before_the_decision(
        "interrupted_at__staged_put_written",
        Boundary::StagedPutWritten,
    );
}

#[test]
fn interrupted_at__staged_put_synced() {
    interruption_before_the_decision(
        "interrupted_at__staged_put_synced",
        Boundary::StagedPutSynced,
    );
}

#[test]
fn interrupted_at__staged_index_written() {
    interruption_before_the_decision(
        "interrupted_at__staged_index_written",
        Boundary::StagedIndexWritten,
    );
}

#[test]
fn interrupted_at__staged_index_synced() {
    interruption_before_the_decision(
        "interrupted_at__staged_index_synced",
        Boundary::StagedIndexSynced,
    );
}

#[test]
fn interrupted_at__staged_manifest_written() {
    interruption_before_the_decision(
        "interrupted_at__staged_manifest_written",
        Boundary::StagedManifestWritten,
    );
}

#[test]
fn interrupted_at__staged_manifest_synced() {
    interruption_before_the_decision(
        "interrupted_at__staged_manifest_synced",
        Boundary::StagedManifestSynced,
    );
}

#[test]
fn interrupted_at__staging_directory_synced() {
    interruption_before_the_decision(
        "interrupted_at__staging_directory_synced",
        Boundary::StagingDirectorySynced,
    );
}

#[test]
fn interrupted_at__commit_renamed() {
    interruption_after_the_decision("interrupted_at__commit_renamed", Boundary::CommitRenamed);
}

#[test]
fn interrupted_at__parent_directory_synced() {
    interruption_after_the_decision(
        "interrupted_at__parent_directory_synced",
        Boundary::ParentDirectorySynced,
    );
}

#[test]
fn interrupted_at__puts_applied() {
    interruption_after_the_decision("interrupted_at__puts_applied", Boundary::PutsApplied);
}

#[test]
fn interrupted_at__index_applied() {
    interruption_after_the_decision("interrupted_at__index_applied", Boundary::IndexApplied);
}

#[test]
fn interrupted_at__removals_applied() {
    interruption_after_the_decision(
        "interrupted_at__removals_applied",
        Boundary::RemovalsApplied,
    );
}

#[test]
fn interrupted_at__done_renamed() {
    interruption_after_the_decision("interrupted_at__done_renamed", Boundary::DoneRenamed);
}

#[test]
fn interrupted_at__cleanup_removed() {
    interruption_after_the_decision("interrupted_at__cleanup_removed", Boundary::CleanupRemoved);
}

// ── injected faults: each barrier member separately, and the stated reactions ─

/// Arm the calling thread's seam until the guard drops.
struct Armed;

impl Armed {
    fn failing(boundary: Boundary) -> Self {
        fault::install(fault::Faults::failing_at(boundary));

        Armed
    }

    fn short_writing(boundary: Boundary) -> Self {
        fault::install(fault::Faults::short_writing_at(boundary));

        Armed
    }
}

impl Drop for Armed {
    fn drop(&mut self) {
        fault::reset();
    }
}

fn failed_batch(boundary: Boundary) -> (Scratch, smut::error::Error) {
    let scratch = Scratch::seeded();
    let error = {
        let _armed = Armed::failing(boundary);

        scratch
            .store()
            .run(&mut Batch::new())
            .expect_err("the armed step fails")
    };

    (scratch, error)
}

/// A failure at `boundary` before the decision: exit 1 and the store is exactly
/// as it was.
fn failed_batch_is_pre_commit(test: &str, boundary: Boundary) {
    let (scratch, error) = failed_batch(boundary);

    assert!(
        error.to_string().contains(boundary.as_str()),
        "{test}: {error}"
    );
    assert_eq!(error.exit_code(), 1, "{test}");
    assert_pre_commit(&scratch);
}

#[test]
fn a_failed_staged_put_fsync_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staged_put_fsync_is_pre_commit",
        Boundary::StagedPutSynced,
    );
}

#[test]
fn a_failed_staged_index_fsync_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staged_index_fsync_is_pre_commit",
        Boundary::StagedIndexSynced,
    );
}

#[test]
fn a_failed_staged_manifest_fsync_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staged_manifest_fsync_is_pre_commit",
        Boundary::StagedManifestSynced,
    );
}

#[test]
fn a_failed_staging_directory_fsync_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staging_directory_fsync_is_pre_commit",
        Boundary::StagingDirectorySynced,
    );
}

#[test]
fn a_failed_staged_put_write_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staged_put_write_is_pre_commit",
        Boundary::StagedPutWritten,
    );
}

#[test]
fn a_failed_staged_index_write_is_pre_commit() {
    failed_batch_is_pre_commit(
        "a_failed_staged_index_write_is_pre_commit",
        Boundary::StagedIndexWritten,
    );
}

#[test]
fn a_failed_commit_rename_is_pre_commit_and_discards_the_staging_state() {
    failed_batch_is_pre_commit(
        "a_failed_commit_rename_is_pre_commit_and_discards_the_staging_state",
        Boundary::CommitRenamed,
    );
}

/// The decision is the rename, so what `journal.commit/` holds is exactly the
/// material every barrier step produced before it (§4.3).
#[test]
fn the_barrier_stages_every_put_and_the_manifest_before_the_decision() {
    if is_child() {
        run_child();
    }

    let scratch = Scratch::seeded();

    abort_at(
        "the_barrier_stages_every_put_and_the_manifest_before_the_decision",
        Boundary::CommitRenamed,
        &scratch.root,
        "declare",
    );

    let staged: BTreeMap<String, Vec<u8>> = scratch
        .tree()
        .into_iter()
        .filter(|(key, _)| key.starts_with(&format!("{COMMIT_STATE}/")))
        .collect();
    let manifest = smut::store::journal::Manifest::parse(
        &staged[&format!("{COMMIT_STATE}/{MANIFEST_BASENAME}")],
    )
    .expect("the staged manifest");

    assert_eq!(
        manifest.removes(),
        [BETA_FILE],
        "the removal is in the write set"
    );
    assert_eq!(manifest.puts().len(), 3, "two entities and the index");
    assert_eq!(
        manifest.puts().last().map(String::as_str),
        Some(INDEX_BASENAME),
        "the index is staged last, so it is applied after the puts"
    );
    for basename in manifest.puts() {
        assert!(
            staged.contains_key(&format!("{COMMIT_STATE}/{basename}")),
            "{basename} is listed and not staged"
        );
    }
    assert_eq!(
        staged.len(),
        4,
        "the manifest and every put, and nothing else"
    );
}

#[test]
fn a_failed_initialisation_parent_fsync_publishes_nothing() {
    let scratch = Scratch::new();
    let error = {
        let _armed = Armed::failing(Boundary::InitialisationParentSynced);

        scratch
            .store()
            .run(&mut Batch::new())
            .expect_err("the store's parent fsync fails")
    };

    assert!(
        error.to_string().contains("initialisation_parent_synced"),
        "{error}"
    );
    assert_eq!(error.exit_code(), 1);
    assert_eq!(
        scratch.tree(),
        BTreeMap::new(),
        "a failed initialisation published nothing"
    );
}

/// A failure at `boundary` after the decision: exit 1, the commit state stays,
/// and the next command completes the batch (§4.5).
fn failed_batch_rolls_forward(test: &str, boundary: Boundary) {
    let (scratch, error) = failed_batch(boundary);

    assert!(
        error.to_string().contains(boundary.as_str()),
        "{test}: {error}"
    );
    assert_eq!(error.exit_code(), 1, "{test}");
    assert_crash_state(&scratch, Crash::AfterDecision);

    recover(&scratch);
    assert_whole_batch(&scratch);
}

#[test]
fn a_failed_parent_fsync_after_the_decision_rolls_forward() {
    failed_batch_rolls_forward(
        "a_failed_parent_fsync_after_the_decision_rolls_forward",
        Boundary::ParentDirectorySynced,
    );
}

#[test]
fn a_failed_put_apply_rolls_forward() {
    failed_batch_rolls_forward("a_failed_put_apply_rolls_forward", Boundary::PutsApplied);
}

#[test]
fn a_failed_index_apply_rolls_forward() {
    failed_batch_rolls_forward("a_failed_index_apply_rolls_forward", Boundary::IndexApplied);
}

#[test]
fn a_failed_removal_unlink_rolls_forward() {
    failed_batch_rolls_forward(
        "a_failed_removal_unlink_rolls_forward",
        Boundary::RemovalsApplied,
    );
}

#[test]
fn a_failed_done_rename_rolls_forward() {
    failed_batch_rolls_forward("a_failed_done_rename_rolls_forward", Boundary::DoneRenamed);
}

#[test]
fn a_failed_cleanup_unlink_rolls_forward() {
    failed_batch_rolls_forward(
        "a_failed_cleanup_unlink_rolls_forward",
        Boundary::CleanupRemoved,
    );
}

#[test]
fn a_short_write_is_retried_until_the_staged_file_is_whole() {
    let scratch = Scratch::seeded();

    {
        let _armed = Armed::short_writing(Boundary::StagedPutWritten);

        scratch
            .store()
            .run(&mut Batch::new())
            .expect("a short write is retried, not taken for completion");
    }

    assert_whole_batch(&scratch);
}

// ── recovery itself, interrupted (§4.4, EX-6, EX-7) ──────────────────────────

#[test]
fn a_failed_recovery_put_replay_leaves_the_commit_state_for_the_next_command() {
    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    {
        let _armed = Armed::failing(Boundary::RecoveryPutReplayed);
        let error = scratch
            .store()
            .run(&mut Read)
            .expect_err("the replay's publish fails");

        assert!(
            error.to_string().contains("recovery_put_replayed"),
            "{error}"
        );
    }

    assert_crash_state(&scratch, Crash::AfterDecision);
    recover(&scratch);
    assert_whole_batch(&scratch);
}

#[test]
fn a_failed_recovery_removal_replay_leaves_the_commit_state_for_the_next_command() {
    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    {
        let _armed = Armed::failing(Boundary::RecoveryRemovalReplayed);
        let error = scratch
            .store()
            .run(&mut Read)
            .expect_err("the removal replay fails");

        assert!(
            error.to_string().contains("recovery_removal_replayed"),
            "{error}"
        );
    }

    assert_crash_state(&scratch, Crash::AfterDecision);
    recover(&scratch);
    assert_whole_batch(&scratch);
}

#[test]
fn a_failed_recovery_done_rename_leaves_the_commit_state_for_the_next_command() {
    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    {
        let _armed = Armed::failing(Boundary::RecoveryDoneRenamed);
        let error = scratch
            .store()
            .run(&mut Read)
            .expect_err("the replay's state move fails");

        assert!(
            error.to_string().contains("recovery_done_renamed"),
            "{error}"
        );
    }

    assert_crash_state(&scratch, Crash::AfterDecision);
    recover(&scratch);
    assert_whole_batch(&scratch);
}

#[test]
fn a_failed_recovery_temporary_removal_leaves_the_store_recoverable() {
    let scratch = Scratch::seeded();
    fs::write(scratch.directory().join("index.json.tmp"), b"{}").expect("a temporary");

    {
        let _armed = Armed::failing(Boundary::RecoveryTemporariesRemoved);
        let error = scratch
            .store()
            .run(&mut Read)
            .expect_err("the temporary cannot be removed");

        assert!(
            error.to_string().contains("recovery_temporaries_removed"),
            "{error}"
        );
    }

    assert!(scratch.directory().join("index.json.tmp").exists());

    recover(&scratch);
    assert_pre_commit(&scratch);
}

#[test]
fn recovery_interrupted__recovery_temporaries_removed() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        fs::write(scratch.directory().join("index.json.tmp"), b"{}").expect("a temporary");
        fs::write(scratch.directory().join("Ab3.json.tmp"), b"{}").expect("a temporary");
    });

    let scratch = Scratch::new();
    seed(&scratch.root);
    fs::write(scratch.directory().join("index.json.tmp"), b"{}").expect("a temporary");
    fs::write(scratch.directory().join("Ab3.json.tmp"), b"{}").expect("a temporary");

    abort_at(
        "recovery_interrupted__recovery_temporaries_removed",
        Boundary::RecoveryTemporariesRemoved,
        &scratch.root,
        "read",
    );

    assert!(
        scratch.tree().keys().any(|key| key.ends_with(".tmp")),
        "the interruption landed inside the removal"
    );

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_prepare_discarded() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        stage_the_batch(scratch, PREPARE_STATE);
    });

    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, PREPARE_STATE);

    abort_at(
        "recovery_interrupted__recovery_prepare_discarded",
        Boundary::RecoveryPrepareDiscarded,
        &scratch.root,
        "read",
    );

    assert!(
        scratch.directory().join(PREPARE_STATE).is_dir(),
        "the discard stopped part-way"
    );

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_commit_validated() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        stage_the_batch(scratch, COMMIT_STATE);
    });

    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    abort_at(
        "recovery_interrupted__recovery_commit_validated",
        Boundary::RecoveryCommitValidated,
        &scratch.root,
        "read",
    );

    assert_crash_state(&scratch, Crash::AfterDecision);

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_put_replayed() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        stage_the_batch(scratch, COMMIT_STATE);
    });

    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    abort_at(
        "recovery_interrupted__recovery_put_replayed",
        Boundary::RecoveryPutReplayed,
        &scratch.root,
        "read",
    );

    assert_crash_state(&scratch, Crash::AfterDecision);
    assert!(
        scratch.directory().join(ALPHA_FILE).is_file(),
        "the interruption landed after the first replayed put"
    );

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_removal_replayed() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        stage_the_batch(scratch, COMMIT_STATE);
    });

    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    abort_at(
        "recovery_interrupted__recovery_removal_replayed",
        Boundary::RecoveryRemovalReplayed,
        &scratch.root,
        "read",
    );

    assert!(
        !scratch.directory().join(BETA_FILE).exists(),
        "the removal ran"
    );

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_done_renamed() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        stage_the_batch(scratch, COMMIT_STATE);
    });

    let scratch = Scratch::seeded();
    stage_the_batch(&scratch, COMMIT_STATE);

    abort_at(
        "recovery_interrupted__recovery_done_renamed",
        Boundary::RecoveryDoneRenamed,
        &scratch.root,
        "read",
    );

    assert_crash_state(&scratch, Crash::AfterDecision);

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

#[test]
fn recovery_interrupted__recovery_cleanup_removed() {
    if is_child() {
        run_child();
    }

    let expected = settled(|scratch| {
        seed(&scratch.root);
        publish_the_batch(scratch);
        stage_the_batch(scratch, DONE_STATE);
    });

    let scratch = Scratch::seeded();
    publish_the_batch(&scratch);
    stage_the_batch(&scratch, DONE_STATE);

    abort_at(
        "recovery_interrupted__recovery_cleanup_removed",
        Boundary::RecoveryCleanupRemoved,
        &scratch.root,
        "read",
    );

    assert!(
        scratch.directory().join(DONE_STATE).is_dir(),
        "the clear stopped part-way"
    );

    recover(&scratch);
    assert_eq!(
        scratch.tree(),
        expected,
        "the next command's recovery converged"
    );
}

// ── non-vacuity (EX-9) ───────────────────────────────────────────────────────

#[test]
fn the_suite_covers_every_registered_boundary() {
    let source = include_str!("durability_crash.rs");

    assert!(
        Boundary::all().count() > 0,
        "an empty registry would make this guard vacuous"
    );

    for boundary in Boundary::COMMIT_PATH {
        let expected = format!("fn interrupted_at__{}(", boundary.as_str());

        assert!(
            source.contains(&expected),
            "no interruption test for {}: this suite owes `{expected}`",
            boundary.as_str()
        );
    }

    for boundary in Boundary::RECOVERY_PATH {
        let expected = format!("fn recovery_interrupted__{}(", boundary.as_str());

        assert!(
            source.contains(&expected),
            "no repeated-recovery test for {}: this suite owes `{expected}`",
            boundary.as_str()
        );
    }
}

#[test]
fn an_armed_failure_is_cleared_when_the_guard_drops() {
    {
        let _armed = Armed::failing(Boundary::CleanupRemoved);

        fault::checkpoint(Boundary::CleanupRemoved).expect_err("the seam is armed");
    }

    fault::checkpoint(Boundary::CleanupRemoved).expect("the seam is disarmed");
}
