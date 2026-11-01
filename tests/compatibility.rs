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
//! The centre of the file is
//! [`where_accepted_adds_a_predicate_without_disturbing_the_pinned_query`]: a
//! 0.1 query keeps returning its pinned result beside a `where` query whose
//! predicate filters and whose envelope is still the same five members.

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

/// The additivity pin that replaces the `where`-refusal pin (DEC-019).
///
/// This test used to fix the *refusal*: `where` was a 0.2 key, refused by name
/// with the release that would accept it. That release is the work this tree
/// carries, so the refusal is gone and the pin is re-cut to what survives it —
/// accepting `where` adds a filter and changes nothing else a 0.1 caller can
/// see. The predicate is a real one, and the envelope is still exactly the five
/// 0.1 members, because no traversal key was supplied.
#[test]
fn where_accepted_adds_a_predicate_without_disturbing_the_pinned_query() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let queried = run(&scratch, &["query", r#"{"where":{"type":"question"}}"#]);
    queried.assert_one_json_line("a query with a predicate");

    let document = queried.json();
    assert_eq!(
        keys(&document),
        BTreeSet::from([
            "dangling",
            "missing",
            "objects",
            "truncated",
            "truncation_reasons",
        ]),
        "the five 0.1 members and no `traversal`: `where` adds no member: {document}"
    );
    assert_eq!(
        document["objects"],
        json!([]),
        "the store holds no `type` field, so no entity is question-typed: {document}"
    );
    assert_eq!(document["truncated"], json!(false), "{document}");
    assert_eq!(document["truncation_reasons"], json!([]), "{document}");
    assert_eq!(document["missing"], json!([]), "{document}");

    // The half the empty selection above cannot show: the predicate must
    // *compute*. `kind` is a field this store does hold, so the same request
    // shape selects exactly the one note — an implementation that accepted
    // `where` and ignored it would return all three.
    let filtered = run(&scratch, &["query", r#"{"where":{"kind":"note"}}"#]);
    filtered.assert_one_json_line("a predicate over a field the store holds");
    assert_eq!(
        keys(&filtered.json()),
        keys(&document),
        "a predicate that selects something still adds no member"
    );
    assert_eq!(
        filtered.json()["objects"],
        json!([{"body": "First thought.", "id": "alpha", "kind": "note", "title": "Alpha"}]),
        "the predicate selects the one note: {}",
        filtered.json()
    );

    // The point of the pin: the wider surface did not disturb the 0.1
    // invocation beside it.
    let after = run(&scratch, &["query", "{}"]);
    after.assert_one_json_line("the pinned 0.1 query, beside the accepted predicate");
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

/// The brief's end-to-end acceptance example, re-cut for every step 0.2 can run
/// (EX-12, DEC-019: renamed and re-commented, not deleted).
///
/// 0.1 could run the workflow only up to the delete and had to stop there: the
/// brief's closing "the final query reports a dangling reference" needs
/// traversal, which 0.1 does not have. 0.2 has it, so the closing query now
/// follows `smut`'s `questions` and reports the reference `q-query` left behind
/// — the example's last step is what the brief says it is. `where` is still
/// refused by name (pinned above), so the `where` invocation stays out of the
/// run; what is fixed here is the nested declaration, the patch that preserves
/// unnamed fields, `DROP` of one field, the delete envelope, and the traversal
/// query that reports the deleted entity as dangling.
#[test]
fn the_briefs_end_to_end_example_runs_every_step_0_2_can() {
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

    let survivors = run(
        &scratch,
        &["query", r#"{"from":"smut","follow":["questions"]}"#],
    );
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
    assert_eq!(
        survivors.json()["dangling"],
        json!([{"source": "smut", "field": "questions", "target": "q-query"}]),
        "the brief's closing step: the traversal reports the deleted entity as dangling"
    );
}

#[test]
fn learn_writes_a_non_empty_guide_and_touches_no_store() {
    let scratch = Scratch::new();

    let ran = run(&scratch, &["learn"]);

    // The guide's content is PHASE-06's, which wrote it; only its shape is pinned
    // here. (This comment named a PHASE-08 that was never planned — RV-020 F-7.)
    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);
    assert!(!ran.stdout.trim().is_empty(), "learn prints a guide");
    assert!(!scratch.has_store(), "learn creates neither store nor lock");
}

// ── VT-6: a traversal query is additive beside the 0.1 pins ──────────────────

#[test]
fn a_traversal_query_adds_one_member_beside_the_pinned_0_1_envelope() {
    // DEC-019 additivity: 0.2 widens the envelope by exactly one conditional
    // member and changes nothing else. `beta` points at `alpha`, so a depth-1
    // walk (the default) discovers `alpha` at the boundary and never expands it
    // — which is why `alpha` is frontier even though it is returned.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let walked = run(
        &scratch,
        &["query", r#"{"from":"beta","follow":["points_at"]}"#],
    );
    walked.assert_one_json_line("a traversal query");
    assert_eq!(
        keys(&walked.json()),
        BTreeSet::from([
            "dangling",
            "missing",
            "objects",
            "traversal",
            "truncated",
            "truncation_reasons",
        ]),
        "the five 0.1 members plus exactly one conditional member: {}",
        walked.json()
    );

    let envelope = walked.json();
    assert_eq!(
        envelope["objects"],
        json!([
            {
                "body": "Second.",
                "id": "beta",
                "points_at": {"id": "alpha"},
                "title": "Beta",
            },
            {
                "body": "First thought.",
                "id": "alpha",
                "kind": "note",
                "title": "Alpha",
            },
        ]),
        "discovery order puts the root before what it discovered: {envelope}"
    );
    assert_eq!(
        envelope["traversal"]["depths"],
        json!({"beta": 0, "alpha": 1}),
        "{envelope}"
    );
    assert_eq!(
        envelope["traversal"]["edges"],
        json!([{"source": "beta", "field": "points_at", "target": "alpha"}]),
        "{envelope}"
    );
    assert_eq!(
        envelope["traversal"]["frontier"],
        json!(["alpha"]),
        "the default depth of 1 stops at the discovered target: {envelope}"
    );
    assert_eq!(envelope["dangling"], json!([]));
    assert_eq!(envelope["truncated"], json!(false));

    // The point of the pin: the 0.2 envelope appeared without touching the 0.1
    // one beside it.
    let pinned = run(&scratch, &["query", "{}"]);
    pinned.assert_one_json_line("the pinned 0.1 query, after a traversal query");
    assert_eq!(pinned.json(), default_query_pin());
}
