//! The seam neither half of the crate tests alone (§11.2, DEC-001, DEC-003).
//!
//! `normalize → write set → commit → reload`: the declaration core produces a
//! write set, the store shell translates it into files and a fresh index, and only
//! a round trip proves the entities survive that translation. Every declaration
//! here goes through the library's real path — [`cli::parse`] and then
//! [`Request::run`] — rather than a hand-built write set, and every reload is a
//! *fresh* [`Store::load`], which validates the index, the mapped files and the
//! agreement between each file's `id` and its index key. A `load()` that succeeds
//! is part of the proof.
//!
//! The file is deliberately small: this layer exists to catch a write-set
//! translation that loses a field, a generated id or a removal, not to re-test the
//! declaration algebra.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use smut::cli::{self, Invocation};
use smut::declare::Entities;
use smut::id::Id;
use smut::store::{Snapshot, Store};

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-roundtrip-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a scratch directory");

        Self { path }
    }

    fn store(&self) -> Store {
        Store::in_directory(&self.path)
    }

    /// The store as a *later* process would find it: read and validated from
    /// disk, sharing nothing with the command that wrote it.
    fn reload(&self) -> Snapshot {
        self.store()
            .load()
            .expect("the store reloads and validates")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Runs one request exactly as the binary does: DEC-004's step 1 through
/// [`cli::parse`], and steps 2–11 through [`Request::run`], returning the bytes
/// the shell would write.
fn run(scratch: &Scratch, arguments: &[&str]) -> String {
    let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let mut stdin: &[u8] = b"";

    let invocation = cli::parse(&arguments, false, &mut stdin)
        .unwrap_or_else(|error| panic!("{arguments:?} is a valid request: {error}"));

    match invocation {
        Invocation::Request(request) => request
            .run(&scratch.store())
            .unwrap_or_else(|error| panic!("{arguments:?} runs: {error}")),
        _ => panic!("{arguments:?} is a request, not a bare answer"),
    }
}

/// The same, with the response parsed back from the bytes a caller would see.
fn run_json(scratch: &Scratch, arguments: &[&str]) -> Value {
    let stdout = run(scratch, arguments);

    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} writes one JSON value ({error}): {stdout:?}"))
}

fn statuses(response: &Value) -> Vec<&str> {
    response["changes"]
        .as_array()
        .expect("changes is an array")
        .iter()
        .map(|change| change["status"].as_str().expect("a status"))
        .collect()
}

fn ids_of(entities: &Entities) -> BTreeSet<&str> {
    entities.keys().map(Id::as_str).collect()
}

// ── VT-1: the round-trip seam ────────────────────────────────────────────────

#[test]
fn roundtrip_reload_keeps_a_generated_id_and_every_declared_field() {
    let scratch = Scratch::new();

    // One entity with an explicit id, one without: the second is the case unit
    // tests can only reach with a scripted id source, and here the real one
    // runs.
    let response = run_json(
        &scratch,
        &[
            "declare",
            r#"[{"id":"alpha","title":"Alpha","body":"First."},{"title":"Generated","kind":"note"}]"#,
        ],
    );

    assert_eq!(response["ids"][0], json!("alpha"));
    let generated = response["ids"][1]
        .as_str()
        .expect("the id-less entity was given an id")
        .to_owned();
    assert_ne!(generated, "alpha");
    assert_eq!(statuses(&response), ["created", "created"]);
    for change in response["changes"].as_array().expect("an array") {
        assert_eq!(change["previous"], Value::Null, "{change}");
    }

    let snapshot = scratch.reload();
    let entities = snapshot.entities();

    assert_eq!(entities.len(), 2, "{entities:?}");
    assert_eq!(
        entities.get("alpha"),
        Some(&json!({"id":"alpha","title":"Alpha","body":"First."}))
    );
    assert_eq!(
        entities.get(generated.as_str()),
        Some(&json!({"id":generated,"title":"Generated","kind":"note"})),
        "the generated id is stored under itself, with the field set it was declared with"
    );

    // The response described exactly the entities the store holds. The response's
    // own order is declaration order and the store's is ID order (a `BTreeMap`,
    // never `readdir`), so the two agree as sets rather than element by element.
    let reported: BTreeSet<&str> = response["ids"]
        .as_array()
        .expect("an array")
        .iter()
        .map(|id| id.as_str().expect("a string"))
        .collect();
    assert_eq!(reported, ids_of(entities));
}

#[test]
fn roundtrip_reload_of_a_patch_and_a_drop_then_a_re_add() {
    let scratch = Scratch::new();

    run_json(
        &scratch,
        &[
            "declare",
            r#"{"id":"alpha","title":"Alpha","body":"First.","tags":["one"]}"#,
        ],
    );

    // Patch one field and DROP another in the same declaration.
    let patched = run_json(
        &scratch,
        &[
            "declare",
            r#"{"id":"alpha","body":"Revised.","tags":"DROP"}"#,
        ],
    );
    assert_eq!(statuses(&patched), ["updated"]);
    assert_eq!(
        patched["changes"][0]["previous"],
        json!({"id":"alpha","title":"Alpha","body":"First.","tags":["one"]}),
        "previous is the whole pre-batch entity"
    );

    let after_drop = scratch.reload();
    assert_eq!(
        after_drop.entities().get("alpha"),
        Some(&json!({"id":"alpha","title":"Alpha","body":"Revised."})),
        "the dropped field is gone, the fields not supplied are intact"
    );

    // A third declaration re-adds the dropped field: DROP removed the field, not
    // the id's ability to hold it again.
    let restored = run_json(
        &scratch,
        &["declare", r#"{"id":"alpha","tags":["two","three"]}"#],
    );
    assert_eq!(statuses(&restored), ["updated"]);

    let after_re_add = scratch.reload();
    assert_eq!(
        after_re_add.entities().get("alpha"),
        Some(&json!({"id":"alpha","title":"Alpha","body":"Revised.","tags":["two","three"]}))
    );
}

#[test]
fn roundtrip_reload_of_a_delete_leaves_the_incoming_reference_untouched() {
    let scratch = Scratch::new();

    // A nested declaration is hoisted into an entity of its own, and its
    // appearance becomes an id-only reference (the entity conventions).
    run_json(
        &scratch,
        &[
            "declare",
            r#"[{"id":"source","body":"A","points_at":{"id":"target","body":"B"}}]"#,
        ],
    );

    let reference = json!({"id":"source","body":"A","points_at":{"id":"target"}});
    assert_eq!(
        scratch.reload().entities().get("source"),
        Some(&reference),
        "the nested declaration was replaced by an id-only reference"
    );

    let deleted = run_json(&scratch, &["delete", "target"]);
    assert_eq!(deleted, json!({"deleted":["target"],"missing":[]}));

    let after_delete = scratch.reload();
    assert_eq!(after_delete.entities().get("target"), None);
    assert_eq!(
        after_delete.entities().get("source"),
        Some(&reference),
        "the delete does not cascade: the incoming reference survives verbatim"
    );
    assert_eq!(after_delete.entities().len(), 1);
}

#[test]
fn roundtrip_reload_matches_what_the_responses_described() {
    let scratch = Scratch::new();

    run_json(
        &scratch,
        &[
            "declare",
            r#"{"id":"kept","title":"Kept","body":"Untouched."}"#,
        ],
    );
    run_json(&scratch, &["declare", r#"{"id":"gone","title":"Gone"}"#]);

    // One batch that creates, amends and merely references: the third object is
    // an id-only reference, so it is named in `ids` and produces no change.
    let batch = run_json(
        &scratch,
        &[
            "declare",
            r#"[{"id":"fresh","title":"Fresh"},{"id":"kept","body":"Amended."},{"id":"gone"}]"#,
        ],
    );
    assert_eq!(batch["ids"], json!(["fresh", "kept", "gone"]));
    assert_eq!(statuses(&batch), ["created", "updated"]);

    let snapshot = scratch.reload();
    let expected: Entities = BTreeMap::from([
        (Id::new("fresh"), json!({"id":"fresh","title":"Fresh"})),
        (
            Id::new("kept"),
            json!({"id":"kept","title":"Kept","body":"Amended."}),
        ),
        (Id::new("gone"), json!({"id":"gone","title":"Gone"})),
    ]);

    assert_eq!(
        snapshot.entities(),
        &expected,
        "no silent extra or missing entity: the store is exactly the batch's result"
    );

    // `load()` refused any id/file disagreement on the way here; naming the
    // mapping makes the check the reload passed visible rather than implied.
    assert_eq!(snapshot.index().len(), snapshot.entities().len());
    for (id, basename) in snapshot.index().entries() {
        assert_eq!(
            snapshot.entities()[id]["id"],
            json!(id.as_str()),
            "{basename} holds another id"
        );
    }
}
