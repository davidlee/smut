//! Concurrent initialisation, termination before the first lock acquisition, and
//! one process recovering another's transaction (§5.4, DEC-003, DEC-004).
//!
//! Every scenario here asserts invariants that hold for *all* interleavings,
//! never one expected interleaving: a store created once, no batch partially
//! visible, no process observing an unrecovered journal. Nothing is synchronised
//! by sleeping — where ordering matters the test takes the lock itself, so the
//! child is *guaranteed* to be on one side of it. The one bounded retry is the
//! first-declaration race, where the only way to vary the timing is to run it
//! again; each attempt asserts the identical set, so a flaky result is a failure
//! rather than a retry until green.

use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Output, Stdio};

use serde_json::{Value, json};

use smut::declare::{Entities, WriteSet};
use smut::error::Reject;
use smut::id::{Id, IdSource, Occupied};
use smut::store::journal::{COMMIT_STATE, MANIFEST_FORMAT_VERSION};
use smut::store::{
    Command, Computed, INDEX_BASENAME, JOURNAL_STATES, LOCK_BASENAME, MANIFEST_BASENAME,
    STORE_DIRECTORY, Snapshot,
};

mod common;
use common::{Scratch, entity, entity_bytes, index_bytes, listing};

/// How many times the first-declaration race is run. Each attempt is a fresh
/// store and asserts the whole invariant set; an attempt that fails fails the
/// test. The repetition varies the timing of two processes racing to create a
/// store — it is not a retry until green.
const ATTEMPTS: usize = 3;

/// The signal a killed child ends with.
const SIGKILL: i32 = 9;

/// The id the spawned child commits first, so the parent's precheck candidate is
/// stale by the time the authoritative pass runs.
const COLLIDING: &str = "shared";

/// The id the authoritative pass generates once the colliding candidate is
/// refused.
const FREE: &str = "parent-second";

/// This file's listings and reload; the scratch is `tests/common`'s.
impl Scratch {
    /// Every entry under `.smut/`, sorted.
    fn listing(&self) -> Vec<String> {
        listing(&self.store_dir())
    }

    /// Every entry beside the store, sorted.
    fn root_listing(&self) -> Vec<String> {
        listing(self.path())
    }

    /// The store as a later process would find it: read and validated from disk.
    fn load(&self) -> Snapshot {
        self.store().load().expect("the store loads and validates")
    }
}

/// Spawns the real binary beside `scratch`'s store, with its streams captured
/// and no stdin, so a killed child can never be waiting on one.
fn spawn_in(scratch: &Scratch, arguments: &[&str]) -> Child {
    scratch
        .command(scratch.path())
        .args(arguments)
        .stdin(Stdio::null())
        .spawn()
        .expect("the binary starts")
}

/// Runs the real binary beside `scratch`'s store to completion.
fn run_in(scratch: &Scratch, arguments: &[&str]) -> Output {
    spawn_in(scratch, arguments)
        .wait_with_output()
        .expect("the binary exits")
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn envelope(output: &Output) -> Value {
    serde_json::from_str(&stdout(output))
        .unwrap_or_else(|error| panic!("one JSON value on stdout ({error}): {:?}", stderr(output)))
}

fn named(entries: &[String], name: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| *entry == name)
        .cloned()
        .collect()
}

// ── VT-2 · scenario 1: two simultaneous first declarations ───────────────────

/// One run of the race: two processes, both declaring an id-less entity into a
/// directory with no `.smut/`, started so they overlap.
fn race_attempt(attempt: usize) {
    let scratch = Scratch::new();
    let bodies = ["the first process's", "the second process's"];
    let declarations: Vec<String> = bodies
        .iter()
        .map(|body| format!(r#"{{"title":"Raced","body":"{body}"}}"#))
        .collect();

    // Both are spawned before either is waited on, so the second is already
    // running while the first creates the store.
    let children: Vec<Child> = declarations
        .iter()
        .map(|declaration| spawn_in(&scratch, &["declare", declaration]))
        .collect();
    let outputs: Vec<Output> = children
        .into_iter()
        .map(|child| child.wait_with_output().expect("the binary exits"))
        .collect();

    for output in &outputs {
        assert_eq!(code(output), 0, "attempt {attempt}: {}", stderr(output));
        assert!(
            output.stderr.is_empty(),
            "attempt {attempt}: {}",
            stderr(output)
        );
    }

    // The store exists exactly once, and nothing else was written beside it.
    assert_eq!(
        scratch.root_listing(),
        [STORE_DIRECTORY],
        "attempt {attempt}: one store, nothing beside it"
    );
    let entries = scratch.listing();
    assert_eq!(
        named(&entries, LOCK_BASENAME).len(),
        1,
        "attempt {attempt}: one lock inode: {entries:?}"
    );
    assert_eq!(
        named(&entries, INDEX_BASENAME).len(),
        1,
        "attempt {attempt}: one index: {entries:?}"
    );
    let lock = fs::symlink_metadata(scratch.store_dir().join(LOCK_BASENAME))
        .expect("attempt: the lock exists");
    assert!(
        lock.file_type().is_file(),
        "attempt {attempt}: the lock is one regular file"
    );
    for state in JOURNAL_STATES {
        assert!(
            !entries.iter().any(|entry| entry == state),
            "attempt {attempt}: {state} was left behind: {entries:?}"
        );
    }

    let snapshot = scratch.load();
    assert_eq!(
        snapshot.entities().len(),
        2,
        "attempt {attempt}: {snapshot:?}"
    );

    // Each process reported its own entity created — never `updated`, which is
    // what a generated id landing on the other process's entity would look like.
    let mut reported = Vec::new();
    for (output, body) in outputs.iter().zip(bodies) {
        let envelope = envelope(output);
        assert_eq!(
            envelope["ids"].as_array().map(Vec::len),
            Some(1),
            "attempt {attempt}: {envelope}"
        );
        assert_eq!(
            envelope["changes"][0]["status"],
            json!("created"),
            "attempt {attempt}: {envelope}"
        );
        assert_eq!(
            envelope["changes"][0]["previous"],
            Value::Null,
            "attempt {attempt}: {envelope}"
        );

        let id = envelope["ids"][0]
            .as_str()
            .expect("a generated id")
            .to_owned();
        let stored = snapshot
            .entities()
            .get(id.as_str())
            .unwrap_or_else(|| panic!("attempt {attempt}: {id} is stored"));
        assert_eq!(
            stored["body"],
            json!(body),
            "attempt {attempt}: {id} holds its own process's body"
        );
        reported.push(id);
    }
    assert_ne!(
        reported[0], reported[1],
        "attempt {attempt}: the two declarations did not generate the same id"
    );

    // Every stored id maps to a distinct file, and every file's `smut_id` is
    // its index key. (`load()` refused any disagreement on the way here.)
    let mut files = Vec::new();
    for (id, basename) in snapshot.index().entries() {
        assert!(
            !files.iter().any(|seen| seen == basename),
            "attempt {attempt}: {basename} is mapped twice"
        );
        files.push(basename.to_owned());

        let bytes = fs::read(scratch.store_dir().join(basename)).expect("a mapped file");
        let stored: Value = serde_json::from_slice(&bytes).expect("an entity file");
        assert_eq!(
            stored["smut_id"],
            json!(id.as_str()),
            "attempt {attempt}: {basename}"
        );
    }
    assert_eq!(files.len(), 2, "attempt {attempt}");
}

#[test]
fn two_simultaneous_first_declarations() {
    for attempt in 1..=ATTEMPTS {
        race_attempt(attempt);
    }
}

// ── VT-2 · scenario 2: termination before the first lock acquisition ─────────

#[test]
fn termination_before_the_first_lock_acquisition_leaves_nothing_half_written() {
    let scratch = Scratch::new();

    // The gate is deterministic: the parent takes the lock before the child is
    // spawned, so the child cannot have acquired it, however far it got.
    fs::create_dir_all(scratch.store_dir()).expect("the store directory");
    let lock = scratch.store().lock().expect("the parent's lock");

    let mut child = spawn_in(
        &scratch,
        &[
            "declare",
            r#"[{"smut_id":"killed-one","body":"one"},{"smut_id":"killed-two","body":"two"}]"#,
        ],
    );
    child.kill().expect("the child is killed");
    let output = child.wait_with_output().expect("the child is reaped");
    drop(lock);

    assert_eq!(
        output.status.signal(),
        Some(SIGKILL),
        "the child died of the kill, not of its own exit"
    );
    assert_eq!(
        stdout(&output),
        "",
        "a batch that never committed printed no response"
    );

    // No partial batch is visible: neither entity of the killed batch exists.
    let snapshot = scratch.load();
    assert!(
        snapshot.entities().is_empty(),
        "nothing of the killed batch is visible: {snapshot:?}"
    );
    for id in ["killed-one", "killed-two"] {
        assert_eq!(snapshot.entities().get(id), None, "{id} was published");
    }

    // Nothing half-written: only the lock the parent made, no journal state, no
    // temporary.
    let entries = scratch.listing();
    assert_eq!(entries, [LOCK_BASENAME], "the store holds only its lock");
    for state in JOURNAL_STATES {
        assert!(!entries.iter().any(|entry| entry == state), "{state}");
    }
    for entry in &entries {
        assert!(!entry.ends_with(".tmp"), "{entry} is a leftover temporary");
    }

    // The store converged: the next declaration succeeds and publishes.
    let survivor = run_in(
        &scratch,
        &["declare", r#"{"smut_id":"survivor","body":"survives"}"#],
    );
    assert_eq!(
        code(&survivor),
        0,
        "a declaration after the kill succeeds: {}",
        stderr(&survivor)
    );
    assert_eq!(
        scratch.load().entities(),
        &Entities::from([(
            Id::new("survivor"),
            json!({"smut_id":"survivor","body":"survives"})
        )])
    );
}

// ── VT-2 · scenario 3: one process recovering another's transaction ──────────

/// The pre-decision store's two entity files, and the uid the fabricated batch
/// adds.
const ALPHA_FILE: &str = "Ab3.json";
const BETA_FILE: &str = "Zq7.json";
const GAMMA_FILE: &str = "Fresh.json";
/// A store holding `alpha` and `beta`, with a *decided but unapplied* batch in
/// `journal.commit/`: alpha amended, gamma added, beta removed.
///
/// Fabricating the decided state is how a "died after the decision" store is
/// produced deterministically without fault injection — the injection seam is
/// gated behind a test feature these suites must not need, and the bytes below
/// are exactly what the commit path would have staged and made durable before
/// taking the decision (§4.3).
fn decided_but_unapplied(scratch: &Scratch) {
    let directory = scratch.store_dir();
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
    .expect("the pre-decision index");

    let staged = directory.join(COMMIT_STATE);
    fs::create_dir_all(&staged).expect("a decided journal state");
    let manifest = json!({
        "format_version": MANIFEST_FORMAT_VERSION,
        "puts": [ALPHA_FILE, GAMMA_FILE, INDEX_BASENAME],
        "removes": [BETA_FILE],
    });
    fs::write(
        staged.join(MANIFEST_BASENAME),
        serde_json::to_vec(&manifest).expect("a manifest"),
    )
    .expect("the manifest");
    fs::write(
        staged.join(ALPHA_FILE),
        entity_bytes(&entity("alpha", "changed")),
    )
    .expect("the staged alpha");
    fs::write(
        staged.join(GAMMA_FILE),
        entity_bytes(&entity("gamma", "new")),
    )
    .expect("the staged gamma");
    fs::write(
        staged.join(INDEX_BASENAME),
        index_bytes(&[("alpha", ALPHA_FILE), ("gamma", GAMMA_FILE)]),
    )
    .expect("the staged index");
}

#[test]
fn a_process_recovering_anothers_transaction_observes_one_coherent_state() {
    let scratch = Scratch::new();
    decided_but_unapplied(&scratch);

    // The fixture really is unrecovered, so what follows is the spawned process's
    // recovery rather than the fixture's own shape: a direct read — which by
    // design performs no recovery — still shows the pre-decision state.
    assert_eq!(
        scratch
            .store()
            .load()
            .expect("the pre-decision state")
            .entities(),
        &Entities::from([
            (Id::new("alpha"), entity("alpha", "first")),
            (Id::new("beta"), entity("beta", "second")),
        ])
    );

    // A real process, not a test harness: `query` recovers before it validates
    // and before it serves anything (§4.4, DEC-004 step 4).
    let first = run_in(&scratch, &["query", r#"{"from":["alpha","beta","gamma"]}"#]);
    assert_eq!(code(&first), 0, "{}", stderr(&first));
    let observed = envelope(&first);

    // The whole batch at once — both staged puts, the staged index, and the
    // removal — never a mixture of before and after.
    assert_eq!(
        observed["objects"],
        json!([entity("alpha", "changed"), entity("gamma", "new")]),
        "{observed}"
    );
    assert_eq!(observed["missing"], json!(["beta"]), "{observed}");
    assert_eq!(observed["dangling"], json!([]), "{observed}");
    assert_eq!(observed["truncated"], json!(false), "{observed}");

    // The recovery was completed: no journal state and no temporary remain.
    let entries = scratch.listing();
    for state in JOURNAL_STATES {
        assert!(
            !entries.iter().any(|entry| entry == state),
            "{state} survived recovery: {entries:?}"
        );
    }
    for entry in &entries {
        assert!(!entry.ends_with(".tmp"), "{entry} is a leftover temporary");
    }

    // The replayed store is coherent, and it loads.
    let recovered = scratch.load();
    assert_eq!(
        recovered.entities(),
        &Entities::from([
            (Id::new("alpha"), entity("alpha", "changed")),
            (Id::new("gamma"), entity("gamma", "new")),
        ])
    );
    assert_eq!(recovered.index().len(), 2);

    // A second process observes the same state: recovering a store that is
    // already recovered is idempotent, and it changes nothing.
    let second = run_in(&scratch, &["query", r#"{"from":["alpha","beta","gamma"]}"#]);
    assert_eq!(code(&second), 0, "{}", stderr(&second));
    assert_eq!(envelope(&second), observed, "the second spawn agrees");
    assert_eq!(scratch.load().entities(), recovered.entities());
    assert_eq!(scratch.listing(), entries, "the second spawn wrote nothing");
}

// ── VT-2 · scenario 4: a stale empty-snapshot precheck ──────────────────────

/// An [`IdSource`] that offers its script in order.
struct Script(Vec<&'static str>);

impl IdSource for Script {
    fn candidate(&mut self) -> String {
        if self.0.is_empty() {
            // A source that runs dry fails the test by generating nothing free.
            return String::new();
        }

        self.0.remove(0).to_owned()
    }
}

/// The stale precheck, built at the seam the criterion names.
///
/// The real `Declare` command cannot be used here: its generated ids come from
/// `nanoid` inside the binary, and an integration test cannot script that
/// entropy. What it *can* script is [`Store::run`]'s own contract — `recompute`
/// is called once against an empty snapshot (step 2, the precheck) and once
/// against the loaded one (step 6, the authoritative pass) — so this command uses
/// the first call to let a second process create the store and commit the
/// colliding id, and the second call to generate past it.
struct StalePrecheck<'a> {
    scratch: &'a Scratch,
    passes: Vec<&'static str>,
    generated: Vec<String>,
}

impl Command for StalePrecheck<'_> {
    type Value = String;

    fn creates_store(&self) -> bool {
        true
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<String>, Reject> {
        if self.passes.is_empty() {
            // Step 2: the store does not exist yet, and this snapshot is empty.
            assert!(
                snapshot.entities().is_empty(),
                "the precheck runs against an empty snapshot"
            );
            self.passes.push("precheck");

            // The child creates the store and commits the id this pass would
            // have been free to use — the precheck is now provably stale.
            let declaration = format!(r#"{{"smut_id":"{COLLIDING}","body":"the child's"}}"#);
            let output = run_in(self.scratch, &["declare", &declaration]);
            assert_eq!(code(&output), 0, "{}", stderr(&output));
            assert_eq!(envelope(&output)["ids"], json!([COLLIDING]));

            return Ok(Computed {
                write_set: WriteSet {
                    put: vec![json!({ "smut_id": COLLIDING, "body": "the parent's" })],
                    remove: Vec::new(),
                },
                value: format!("precheck:{COLLIDING}"),
            });
        }

        // Step 6: the authoritative pass, against the snapshot the child's
        // commit is now part of. Generation must draw past the taken candidate.
        self.passes.push("authoritative");
        let mut occupied = Occupied::of(snapshot.entities().keys());
        let generated = occupied.generate(&mut Script(vec![COLLIDING, FREE]));
        let id = generated.as_str().to_owned();
        self.generated.push(id.clone());

        Ok(Computed {
            write_set: WriteSet {
                put: vec![json!({ "smut_id": id, "body": "the parent's" })],
                remove: Vec::new(),
            },
            value: format!("authoritative:{id}"),
        })
    }

    fn prepare_output(&self, value: String) -> String {
        value
    }
}

#[test]
fn stale_empty_snapshot_precheck() {
    let scratch = Scratch::new();
    let mut command = StalePrecheck {
        scratch: &scratch,
        passes: Vec::new(),
        generated: Vec::new(),
    };

    let answer = scratch
        .store()
        .run(&mut command)
        .expect("the declaration runs against a store the child created");

    // The precheck ran, and then the authoritative pass did: two calls, in order.
    assert_eq!(command.passes, ["precheck", "authoritative"]);
    // Generation drew past the stale candidate instead of accepting it.
    assert_eq!(command.generated, [FREE]);
    // The published entity is step 6's decision, not step 2's.
    assert_eq!(answer, format!("authoritative:{FREE}"));

    let snapshot = scratch.load();
    assert_eq!(
        snapshot.entities(),
        &Entities::from([
            (
                Id::new(COLLIDING),
                json!({ "smut_id": COLLIDING, "body": "the child's" })
            ),
            (
                Id::new(FREE),
                json!({ "smut_id": FREE, "body": "the parent's" })
            ),
        ])
    );
    assert_eq!(
        snapshot.entities()[&Id::new(COLLIDING)]["body"],
        json!("the child's"),
        "the precheck's candidate was never persisted: no accidental upsert"
    );
    assert_eq!(snapshot.index().len(), 2);
}
