//! The 0.1 invocations that must keep working when the spec surface widens in
//! 0.2 (§7.3, DEC-011).
//!
//! These are pins, not tests of new behaviour: each one fixes what a caller of
//! this release sees, so a later release cannot change it silently. Each pin
//! carries its exit code and its stream discipline — JSON on stdout, nothing on
//! stderr, one compact line — and, for JSON, the *structure* of the result.
//!
//! **Why structure and not bytes.** An envelope's top-level keys are lexical
//! (DEC-007) but whether an entity root inside `objects` is id-first is an open
//! `/audit` question, so a byte pin there would pre-empt that review. Parsed
//! `Value` equality plus an explicit key set fixes everything a caller can rely
//! on without deciding key order. `render`'s output *is* settled: it is pinned as
//! exact text.
//!
//! The centre of the file is the contrast in
//! [`where_is_refused_by_name_as_a_0_2_key_and_the_pinned_query_still_works`]: a
//! 0.1 query keeps returning its pinned result immediately after a 0.2 key is
//! refused by name.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-compatibility-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a scratch directory");

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn store(&self) -> PathBuf {
        self.path.join(".smut")
    }

    fn has_store(&self) -> bool {
        self.store().is_dir()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One invocation, as a caller sees it.
struct Ran {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Ran {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|error| panic!("stdout is one JSON value ({error}): {:?}", self.stdout))
    }

    /// A successful machine command: one compact JSON line on stdout, exit 0,
    /// nothing on stderr.
    fn assert_one_json_line(&self, what: &str) {
        assert_eq!(self.code, 0, "{what}: {}", self.stderr);
        assert!(self.stderr.is_empty(), "{what}: {}", self.stderr);
        assert!(self.stdout.ends_with('\n'), "{what}: {:?}", self.stdout);
        assert_eq!(
            self.stdout.matches('\n').count(),
            1,
            "{what} is one compact line: {:?}",
            self.stdout
        );
        self.json();
    }
}

fn run(scratch: &Scratch, arguments: &[&str]) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_smut"))
        .current_dir(scratch.path())
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");

    child
        .stdin
        .as_mut()
        .expect("piped stdin")
        .write_all(b"")
        .expect("an empty stdin");

    let output = child.wait_with_output().expect("the binary exits");

    Ran {
        code: output.status.code().expect("a normal exit"),
        stdout: String::from_utf8(output.stdout).expect("UTF-8 stdout"),
        stderr: String::from_utf8(output.stderr).expect("UTF-8 stderr"),
    }
}

fn keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect()
}

/// The three-entity store every pin below reads: a plain entity, one with a
/// reference to it, and a body-only one.
fn store_of_three(scratch: &Scratch) {
    for declaration in [
        r#"{"id":"alpha","title":"Alpha","body":"First thought.","kind":"note"}"#,
        r#"{"id":"beta","title":"Beta","body":"Second.","points_at":{"id":"alpha"}}"#,
        r#"{"id":"gamma","body":"Third."}"#,
    ] {
        let declared = run(scratch, &["declare", declaration]);

        declared.assert_one_json_line("a fixture declaration");
    }
}

/// The pinned default query: no `from` (every id in ID order), no `select` (every
/// field), and no truncation at three entities.
fn default_query_pin() -> Value {
    json!({
        "dangling": [],
        "missing": [],
        "objects": [
            {"body": "First thought.", "id": "alpha", "kind": "note", "title": "Alpha"},
            {"body": "Second.", "id": "beta", "points_at": {"id": "alpha"}, "title": "Beta"},
            {"body": "Third.", "id": "gamma"},
        ],
        "truncated": false,
        "truncation_reasons": [],
    })
}

// ── VT-3: representative 0.1 invocations ────────────────────────────────────

#[test]
fn declare_pins_its_full_and_compact_envelopes() {
    let scratch = Scratch::new();

    let created = run(
        &scratch,
        &[
            "declare",
            r#"{"id":"smut","title":"Smut","body":"Persist conversational structure."}"#,
        ],
    );
    created.assert_one_json_line("a full declare");
    assert_eq!(
        created.json(),
        json!({
            "changes": [{"id": "smut", "previous": null, "status": "created"}],
            "ids": ["smut"],
            "missing": [],
        })
    );
    assert_eq!(
        keys(&created.json()),
        BTreeSet::from(["changes", "ids", "missing"])
    );
    assert_eq!(
        keys(&created.json()["changes"][0]),
        BTreeSet::from(["id", "previous", "status"])
    );

    let amended = run(
        &scratch,
        &[
            "declare",
            r#"{"id":"smut","body":"Persist structure."}"#,
            "--compact",
        ],
    );
    amended.assert_one_json_line("a compact declare");
    assert_eq!(
        amended.json(),
        json!({
            "changes": [{"id": "smut", "status": "updated"}],
            "ids": ["smut"],
            "missing": [],
        })
    );
    assert_eq!(
        keys(&amended.json()["changes"][0]),
        BTreeSet::from(["id", "status"]),
        "--compact omits `previous` everywhere"
    );
}

#[test]
fn query_pins_its_defaults() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = run(&scratch, &["query", "{}"]);

    ran.assert_one_json_line("a default query");
    assert_eq!(ran.json(), default_query_pin());
    assert_eq!(
        keys(&ran.json()),
        BTreeSet::from([
            "dangling",
            "missing",
            "objects",
            "truncated",
            "truncation_reasons",
        ])
    );
}

#[test]
fn query_pins_the_default_limit_of_one_hundred() {
    let scratch = Scratch::new();

    let declarations: Vec<Value> = (1..=101)
        .map(|n| json!({"id": format!("e{n:03}"), "body": format!("note {n}")}))
        .collect();
    let declared = run(
        &scratch,
        &["declare", &Value::Array(declarations).to_string()],
    );
    declared.assert_one_json_line("a batch of 101 declarations");

    let ran = run(&scratch, &["query", "{}"]);
    ran.assert_one_json_line("a default query over 101 entities");

    let envelope = ran.json();
    let objects = envelope["objects"].as_array().expect("an array");
    assert_eq!(
        objects.len(),
        100,
        "no `limit` still caps the result at 100"
    );
    assert_eq!(
        objects[0]["id"],
        json!("e001"),
        "roots are in ID order, so the omission is the last id"
    );
    assert_eq!(objects[99]["id"], json!("e100"));
    assert_eq!(envelope["truncated"], json!(true));
    assert_eq!(envelope["truncation_reasons"], json!(["result_limit"]));
}

#[test]
fn render_pins_its_markdown_text() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = run(&scratch, &["render", r#"{"from":["alpha","beta"]}"#]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);
    assert_eq!(
        ran.stdout, "## Alpha\n\nFirst thought.\n\n## Beta\n\nSecond.\n",
        "the Markdown layout is settled, so it is pinned as exact text"
    );
    assert!(
        serde_json::from_str::<Value>(&ran.stdout).is_err(),
        "render writes Markdown, not JSON: {:?}",
        ran.stdout
    );
}

#[test]
fn get_pins_explicit_order_and_missing() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = run(&scratch, &["get", "beta", "alpha", "ghost"]);

    ran.assert_one_json_line("an explicit get");
    assert_eq!(
        ran.json(),
        json!({
            "dangling": [],
            "missing": ["ghost"],
            "objects": [
                {"body": "Second.", "id": "beta", "points_at": {"id": "alpha"}, "title": "Beta"},
                {"body": "First thought.", "id": "alpha", "kind": "note", "title": "Alpha"},
            ],
            "truncated": false,
            "truncation_reasons": [],
        }),
        "get returns the ids in the order given and reports the missing one"
    );
}

#[test]
fn where_is_refused_by_name_as_a_0_2_key_and_the_pinned_query_still_works() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let refused = run(&scratch, &["query", r#"{"where":{"type":"question"}}"#]);

    assert_eq!(refused.code, 2, "a 0.2 key is invalid input in 0.1");
    assert_eq!(
        refused.stdout, "",
        "a refusal never prints a success envelope"
    );
    assert!(refused.stderr.contains("where"), "{}", refused.stderr);
    assert!(
        refused.stderr.contains("0.2"),
        "the refusal names the release that widens the surface: {}",
        refused.stderr
    );

    // The point of the pin: refusing the wider surface did not disturb the 0.1
    // invocation beside it.
    let after = run(&scratch, &["query", "{}"]);
    after.assert_one_json_line("the pinned 0.1 query, after the refusal");
    assert_eq!(after.json(), default_query_pin());
}

#[test]
fn an_unknown_key_is_refused_without_naming_0_2() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let refused = run(&scratch, &["query", r#"{"wibble":1}"#]);

    assert_eq!(refused.code, 2);
    assert_eq!(refused.stdout, "");
    assert!(refused.stderr.contains("wibble"), "{}", refused.stderr);
    assert!(
        !refused.stderr.contains("0.2"),
        "an unknown key is a typo, not a named-future key: {}",
        refused.stderr
    );
}

/// The brief's end-to-end acceptance example, for the steps 0.1 can run.
///
/// Two of the example's invocations use `follow` and one uses `where`, which are
/// named-future keys refused by name (pinned above), so the workflow's 0.1 part
/// is what is fixed here: the nested declaration, the patch that preserves
/// unnamed fields, `DROP` of one field, the delete envelope, and the reference
/// that survives the delete. The example's closing "the final query reports a
/// dangling reference" needs traversal, which 0.1 does not have, so the final
/// query here pins `dangling: []` for a store that never traversed.
#[test]
fn the_briefs_end_to_end_example_still_runs_where_0_1_can() {
    let scratch = Scratch::new();

    let declared = run(
        &scratch,
        &[
            "declare",
            r#"[{"id":"smut","title":"Smut","body":"Persist conversational structure.","questions":[{"id":"q-query","type":"question","status":"open","title":"Query design","body":"Keep selection small."}]}]"#,
        ],
    );
    declared.assert_one_json_line("the brief's nested declaration");
    assert_eq!(declared.json()["ids"], json!(["smut"]));
    assert_eq!(declared.json()["missing"], json!([]));

    // The nested declaration became an entity of its own and an id-only
    // reference in its container — the brief's stated outcome.
    let stored = run(&scratch, &["get", "smut", "q-query"]);
    stored.assert_one_json_line("both declared entities");
    assert_eq!(
        stored.json()["objects"],
        json!([
            {"body": "Persist conversational structure.", "id": "smut", "questions": [{"id": "q-query"}], "title": "Smut"},
            {"body": "Keep selection small.", "id": "q-query", "status": "open", "title": "Query design", "type": "question"},
        ])
    );

    // A patch preserves the fields it does not name; DROP removes only the field
    // it names.
    run(
        &scratch,
        &[
            "declare",
            r#"{"id":"q-query","status":"resolved","body":"Use bounded traversal and simple predicates."}"#,
        ],
    )
    .assert_one_json_line("the brief's patch");
    run(
        &scratch,
        &["declare", r#"{"id":"q-query","status":"DROP"}"#],
    )
    .assert_one_json_line("the brief's DROP");

    let after = run(&scratch, &["get", "q-query"]);
    after.assert_one_json_line("q-query after the patch and DROP");
    assert_eq!(
        after.json()["objects"],
        json!([{
            "body": "Use bounded traversal and simple predicates.",
            "id": "q-query",
            "title": "Query design",
            "type": "question",
        }])
    );

    // One batch with its own envelope, and the reference in `smut` survives it:
    // querying `smut` after the delete does not recreate `q-query`.
    let deleted = run(&scratch, &["delete", "q-query"]);
    deleted.assert_one_json_line("the brief's delete");
    assert_eq!(
        deleted.json(),
        json!({"deleted": ["q-query"], "missing": []})
    );

    let survivors = run(&scratch, &["query", r#"{"from":"smut"}"#]);
    survivors.assert_one_json_line("smut after the delete");
    assert_eq!(
        survivors.json()["objects"],
        json!([{
            "body": "Persist conversational structure.",
            "id": "smut",
            "questions": [{"id": "q-query"}],
            "title": "Smut",
        }])
    );
    assert_eq!(survivors.json()["missing"], json!([]));
    assert_eq!(survivors.json()["dangling"], json!([]));
}

#[test]
fn learn_writes_a_non_empty_guide_and_touches_no_store() {
    let scratch = Scratch::new();

    let ran = run(&scratch, &["learn"]);

    // The guide's content is PHASE-08's to write; only its shape is pinned here.
    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);
    assert!(!ran.stdout.trim().is_empty(), "learn prints a guide");
    assert!(!scratch.has_store(), "learn creates neither store nor lock");
}
