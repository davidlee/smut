//! The command contract, from outside the process: the envelopes, exit codes
//! 0/1/2, stdin and `-`, `--`, trailing input, `--compact`, the flag bounds, the
//! format refusal, the delete envelope, and stream separation (DEC-004,
//! DEC-008).
//!
//! Every invocation is a real process in its own scratch directory, so the
//! store it finds (or does not find) is the one `$PWD/.smut` names. The one
//! branch a spawned process cannot reach is stdin-is-a-terminal; that decision is
//! a pure function, unit-tested for both terminal states in `src/input.rs`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use smut::learn::{
    EXAMPLE_FILTER, EXAMPLE_FILTER_Q, EXAMPLE_FOLLOW, EXAMPLE_FOLLOW_Q, EXAMPLE_TRAVERSE,
    EXAMPLE_TRAVERSE_Q,
};

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-cli-{}-{}",
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

    fn store_entries(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.store())
            .map(|entries| {
                entries
                    .map(|entry| {
                        entry
                            .expect("a directory entry")
                            .file_name()
                            .into_string()
                            .unwrap()
                    })
                    .collect()
            })
            .unwrap_or_default();
        names.sort();

        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One invocation of the binary, as a caller sees it.
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
}

/// Runs `smut` in `scratch` with `arguments`, and `stdin` if a fixture supplies
/// one.
fn run(scratch: &Scratch, arguments: &[&str], stdin: Option<&str>) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_smut"))
        .current_dir(scratch.path())
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");

    if let Some(text) = stdin {
        child
            .stdin
            .as_mut()
            .expect("piped stdin")
            .write_all(text.as_bytes())
            .expect("stdin accepts the fixture");
    }

    let output = child.wait_with_output().expect("the binary exits");

    Ran {
        code: output.status.code().expect("a normal exit"),
        stdout: String::from_utf8(output.stdout).expect("UTF-8 stdout"),
        stderr: String::from_utf8(output.stderr).expect("UTF-8 stderr"),
    }
}

/// Runs `smut` with no stdin fixture at all.
fn smut(scratch: &Scratch, arguments: &[&str]) -> Ran {
    run(scratch, arguments, None)
}

fn store_of_three(scratch: &Scratch) {
    let declared = smut(
        scratch,
        &[
            "declare",
            r#"{"smut_id":"a","title":"Alpha","body":"First.","priority":3}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    for (id, title) in [("b", "Beta"), ("c", "Gamma")] {
        let declared = smut(
            scratch,
            &[
                "declare",
                &format!(r#"{{"smut_id":"{id}","title":"{title}"}}"#),
            ],
        );
        assert_eq!(declared.code, 0, "{}", declared.stderr);
    }
}

// ── VT-1: the command contract ───────────────────────────────────────────────

#[test]
fn declare_writes_its_envelope_to_stdout_and_exits_zero() {
    let scratch = Scratch::new();

    let ran = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"smut","title":"Smut","questions":[{"smut_id":"q-query","type":"question"}]}"#,
        ],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);

    let envelope = ran.json();
    assert_eq!(envelope["ids"], json!(["smut"]));
    assert_eq!(envelope["changes"][0]["smut_id"], json!("smut"));
    assert_eq!(envelope["changes"][0]["status"], json!("created"));
    assert_eq!(envelope["changes"][0]["previous"], Value::Null);
    assert_eq!(envelope["missing"], json!([]));
    assert_eq!(envelope["changes"].as_array().expect("an array").len(), 2);

    // The envelope is one compact line ending in exactly one newline, which the
    // shell writes verbatim.
    assert_eq!(ran.stdout.matches('\n').count(), 1, "{:?}", ran.stdout);
    assert!(ran.stdout.ends_with('\n'), "{:?}", ran.stdout);
    assert!(
        scratch.has_store(),
        "declare is the command that creates a store"
    );
}

#[test]
fn a_projected_entity_is_serialised_by_the_same_wrapper_as_a_stored_one() {
    // DEC-007: one shared wrapper, "used for stored entity files and for
    // projected entities alike", so a response's entity roots name `smut_id`
    // first
    // and their remaining keys lexically — the same bytes rule as the file.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(&scratch, &["get", "a"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(
        ran.stdout.contains(
            r#""objects":[{"smut_id":"a","body":"First.","priority":3,"title":"Alpha"}]"#
        ),
        "{}",
        ran.stdout
    );
}

#[test]
fn query_reads_its_specification_from_a_pipe_on_stdin_and_exits_zero() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    for arguments in [vec!["query"], vec!["query", "-"]] {
        let ran = run(&scratch, &arguments, Some(r#"{"from":"b"}"#));

        assert_eq!(ran.code, 0, "{arguments:?}: {}", ran.stderr);
        let envelope = ran.json();
        assert_eq!(
            envelope["objects"][0]["smut_id"],
            json!("b"),
            "{arguments:?}"
        );
        assert_eq!(envelope["missing"], json!([]), "{arguments:?}");
    }
}

#[test]
fn declare_reads_its_declaration_from_a_pipe_on_stdin_and_exits_zero() {
    let scratch = Scratch::new();

    let ran = run(
        &scratch,
        &["declare"],
        Some(r#"{"smut_id":"piped","title":"Piped"}"#),
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.json()["ids"], json!(["piped"]));
    // An operand present means stdin is never read, and `-` asks for it
    // explicitly even where a terminal is attached.
    let explicit = run(&scratch, &["declare", "-"], Some(r#"{"smut_id":"dashed"}"#));
    assert_eq!(explicit.code, 0, "{}", explicit.stderr);
    assert_eq!(explicit.json()["ids"], json!(["dashed"]));
}

#[test]
fn a_declaration_with_nothing_to_write_succeeds_without_creating_a_store() {
    let scratch = Scratch::new();

    for arguments in [vec!["declare", "[]"], vec!["declare", "{}"]] {
        let ran = smut(&scratch, &arguments);

        assert_eq!(ran.code, 0, "{arguments:?}: {}", ran.stderr);
        assert_eq!(ran.json()["changes"], json!([]), "{arguments:?}");
        assert!(
            !scratch.has_store(),
            "{arguments:?} had no write set, so no store is created"
        );
    }
}

#[test]
fn an_id_only_reference_on_an_absent_store_is_missing_and_creates_nothing() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["declare", r#"{"smut_id":"ghost"}"#]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(
        ran.json(),
        json!({"changes": [], "ids": ["ghost"], "missing": ["ghost"]})
    );
    assert!(
        !scratch.has_store(),
        "a reference does not create a placeholder"
    );
}

#[test]
fn an_empty_pipe_is_malformed_json_and_exits_two() {
    let scratch = Scratch::new();

    let ran = run(&scratch, &["declare"], Some("  \n"));

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("malformed JSON"), "{}", ran.stderr);
}

#[test]
fn query_against_an_absent_store_exits_one_and_creates_nothing() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["query", "{}"]);

    assert_eq!(ran.code, 1, "{}", ran.stderr);
    assert_eq!(ran.stdout, "", "a failure prints no success envelope");
    assert!(ran.stderr.contains("no store"), "{}", ran.stderr);
    assert!(!scratch.has_store(), "a read never creates a store");
}

#[test]
fn get_against_an_absent_store_exits_one_and_invents_no_missing_list() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["get", "nope"]);

    assert_eq!(ran.code, 1, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("no store"), "{}", ran.stderr);
    assert!(!scratch.has_store());
}

#[test]
fn render_against_an_absent_store_exits_one_and_creates_nothing() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["render", "{}"]);

    assert_eq!(ran.code, 1, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(!scratch.has_store());
}

#[test]
fn trailing_json_input_exits_two_before_any_store_exists() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["declare", r#"{"smut_id":"a"} {"smut_id":"b"}"#]);

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("trailing"), "{}", ran.stderr);
    assert!(
        !scratch.has_store(),
        "step 1 dies before any filesystem access"
    );
}

#[test]
fn a_duplicate_json_key_exits_two_before_any_store_exists() {
    let scratch = Scratch::new();

    let ran = smut(
        &scratch,
        &["declare", r#"{"smut_id":"a","title":"one","title":"two"}"#],
    );

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("duplicate"), "{}", ran.stderr);
    assert!(ran.stderr.contains("title"), "{}", ran.stderr);
    assert!(
        !scratch.has_store(),
        "the refusal happened before the store was touched"
    );
}

#[test]
fn a_double_dash_reaches_an_opaque_id_that_begins_with_a_hyphen() {
    let scratch = Scratch::new();

    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"-weird-id","title":"Weird"}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(&scratch, &["get", "--", "-weird-id"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.json()["objects"][0]["smut_id"], json!("-weird-id"));
}

#[test]
fn heading_level_outside_one_to_six_exits_two_with_our_own_diagnostic() {
    let scratch = Scratch::new();

    for found in ["0", "7", "abc"] {
        let ran = smut(&scratch, &["render", "{}", "--heading-level", found]);

        assert_eq!(ran.code, 2, "{found}: {}", ran.stderr);
        assert_eq!(ran.stdout, "");
        assert!(ran.stderr.contains("1 to 6"), "{found}: {}", ran.stderr);
    }

    let accepted = smut(&scratch, &["render", "{}", "--heading-level", "6"]);
    assert_eq!(accepted.code, 1, "a valid level still needs a store");
}

#[test]
fn mermaid_is_refused_with_exit_two_naming_the_release_that_plans_it() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["render", "{}", "--format", "mermaid"]);

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    // DEC-019 re-cut, PHASE-07: the diagnostic used to say "not supported in
    // 0.1". That sentence becomes false in the commit that moves the crate to
    // 0.2.0, so it now names the release that plans the format and no version it
    // does not ship. The refusal itself — exit 2, the named format, no store — is
    // unchanged.
    assert!(ran.stderr.contains("not supported"), "{}", ran.stderr);
    assert!(ran.stderr.contains("1.0"), "{}", ran.stderr);
    assert!(!scratch.has_store(), "the refusal needs no store");
}

#[test]
fn compact_omits_previous_everywhere_and_changes_nothing_else() {
    let scratch = Scratch::new();

    let created = smut(&scratch, &["declare", r#"{"smut_id":"a","body":"first"}"#]);
    assert_eq!(created.code, 0, "{}", created.stderr);

    let full = smut(&scratch, &["declare", r#"{"smut_id":"a","body":"second"}"#]);
    let full = full.json();
    assert_eq!(full["changes"][0]["status"], json!("updated"));
    assert_eq!(full["changes"][0]["previous"]["body"], json!("first"));

    let compact = smut(
        &scratch,
        &["declare", r#"{"smut_id":"a","body":"third"}"#, "--compact"],
    );
    let compact = compact.json();
    assert_eq!(compact["ids"], json!(["a"]));
    assert_eq!(compact["changes"][0]["status"], json!("updated"));
    assert_eq!(compact["changes"][0]["smut_id"], json!("a"));
    assert_eq!(compact["missing"], json!([]));
    assert!(compact["changes"][0].get("previous").is_none(), "{compact}");
}

#[test]
fn reported_missing_ids_and_truncation_are_success_with_exit_zero() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let truncated = smut(&scratch, &["query", r#"{"limit":2}"#]);
    assert_eq!(truncated.code, 0, "{}", truncated.stderr);
    let envelope = truncated.json();
    assert_eq!(envelope["objects"].as_array().expect("an array").len(), 2);
    assert_eq!(envelope["truncated"], json!(true));
    assert_eq!(envelope["truncation_reasons"], json!(["result_limit"]));

    let missing = smut(&scratch, &["get", "nope", "a"]);
    assert_eq!(missing.code, 0, "{}", missing.stderr);
    let envelope = missing.json();
    assert_eq!(envelope["missing"], json!(["nope"]));
    assert_eq!(envelope["objects"][0]["smut_id"], json!("a"));
    assert_eq!(envelope["truncated"], json!(false));
}

#[test]
fn render_emits_text_on_stdout_with_exactly_one_trailing_newline() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"a","title":"Alpha","body":"First."}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(&scratch, &["render", "{}"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.stdout, "## Alpha\n\nFirst.\n");
    assert_eq!(ran.stdout.matches('\n').count(), 3, "{:?}", ran.stdout);
    assert!(!ran.stdout.ends_with("\n\n"), "{:?}", ran.stdout);
}

#[test]
fn render_honours_the_heading_field_heading_level_show_ids_and_show_fields_flags() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"a","name":"Alpha","status":"open"}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(
        &scratch,
        &[
            "render",
            "{}",
            "--heading-field",
            "name",
            "--heading-level",
            "3",
            "--show-ids",
            "--show-fields",
        ],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stdout.contains("### Alpha"), "{:?}", ran.stdout);
    assert!(ran.stdout.contains("`a`"), "{:?}", ran.stdout);
    assert!(
        ran.stdout.contains("- **status:** open"),
        "{:?}",
        ran.stdout
    );
}

#[test]
fn learn_prints_the_guide_on_stdout_and_touches_no_store() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["learn"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);
    for named in ["smut declare", "DROP", "--compact", "0.2"] {
        assert!(ran.stdout.contains(named), "the guide omits {named}");
    }
    assert!(ran.stdout.ends_with('\n'), "{:?}", ran.stdout);
    assert!(!ran.stdout.ends_with("\n\n"), "{:?}", ran.stdout);
    assert!(
        !scratch.has_store(),
        "learn never creates, locks or recovers"
    );
}

#[test]
fn help_and_version_exit_zero_and_touch_no_store() {
    let scratch = Scratch::new();

    let help = smut(&scratch, &["--help"]);
    assert_eq!(help.code, 0, "{}", help.stderr);
    assert!(help.stdout.contains("Usage: smut"), "{}", help.stdout);
    assert!(help.stdout.contains("learn"), "{}", help.stdout);

    let version = smut(&scratch, &["--version"]);
    assert_eq!(version.code, 0, "{}", version.stderr);
    // DEC-019 re-cut, PHASE-07 EX-5: the literal was `smut 0.1.1`. The release
    // number is the releaser's to move, so the pin moves with it — and it stays a
    // literal, because this is the one place a bump that missed a file would be
    // caught (`src/main.rs` and `src/cli.rs` both read `CARGO_PKG_VERSION`).
    // Moved to 0.2.1 when `edges` joined the surface (IMP-018): additive, optional,
    // and no 0.1 caller sends it, so the bump is a patch. Moved to 0.2.2 for a
    // release that changes no `src/` at all — the README was rewritten and
    // `tests/readme_example.rs` grew the harness that executes its transcripts, so
    // the binary is 0.2.1's binary and the bump is again a patch. Moved to 0.3.0
    // with the version bump in 04cce16.
    assert_eq!(version.stdout, "smut 0.3.0\n");

    assert!(
        !scratch.has_store(),
        "neither --help nor --version builds a store"
    );
}

#[test]
fn an_unknown_subcommand_exits_two_with_nothing_on_stdout() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["frobnicate"]);

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("frobnicate"), "{}", ran.stderr);
}

#[test]
fn a_rejected_specification_exits_two_and_names_the_offending_operator() {
    // DEC-019: this test used to pin `where` being refused with the release that
    // would accept it. `where` is accepted here, so the pin is re-cut to what
    // survives: a valid predicate succeeds with one JSON line on stdout, and a
    // *malformed* one is still invalid input — exit 2, nothing on stdout, and a
    // message naming the offending operator rather than a release, because no
    // release is coming.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let accepted = smut(&scratch, &["query", r#"{"where":{"title":"Alpha"}}"#]);

    assert_eq!(accepted.code, 0, "{}", accepted.stderr);
    assert!(accepted.stderr.is_empty(), "{}", accepted.stderr);
    assert_eq!(
        accepted.stdout.matches('\n').count(),
        1,
        "one JSON line: {:?}",
        accepted.stdout
    );
    assert_eq!(
        accepted.json()["objects"],
        json!([{"body": "First.", "smut_id": "a", "priority": 3, "title": "Alpha"}]),
        "the predicate filters, it is not merely accepted: {}",
        accepted.json()
    );

    let refused = smut(&scratch, &["query", r#"{"where":{"x":{"$in":1}}}"#]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "");
    assert!(refused.stderr.contains("$in"), "{}", refused.stderr);
    assert!(
        !refused.stderr.contains("0.2"),
        "a malformed operand is not a named future key: {}",
        refused.stderr
    );
}

// ── VT-2: the delete contract ────────────────────────────────────────────────

#[test]
fn delete_reports_deleted_and_missing_deduped_in_first_occurrence_order() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(&scratch, &["delete", "c", "a", "c", "nope", "nope"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(
        ran.json(),
        json!({"deleted": ["c", "a"], "missing": ["nope"]})
    );
    assert_eq!(ran.stdout.matches('\n').count(), 1, "{:?}", ran.stdout);

    // One batch, so the store is left with no journal state to recover.
    let entries = scratch.store_entries();
    assert!(
        entries.iter().all(|entry| !entry.starts_with("journal.")),
        "{entries:?}"
    );

    let remaining = smut(&scratch, &["query", "{}"]);
    assert_eq!(
        remaining.json()["objects"]
            .as_array()
            .expect("an array")
            .len(),
        1
    );
    assert_eq!(remaining.json()["objects"][0]["smut_id"], json!("b"));
}

#[test]
fn delete_leaves_incoming_references_untouched() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"parent","title":"Parent","children":[{"smut_id":"child","title":"Child"}]}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let before = smut(&scratch, &["get", "parent"]);
    assert_eq!(
        before.json()["objects"][0]["children"],
        json!([{"smut_id": "child"}])
    );

    let deleted = smut(&scratch, &["delete", "child"]);
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    assert_eq!(deleted.json(), json!({"deleted": ["child"], "missing": []}));

    let parent = smut(&scratch, &["get", "parent"]);
    assert_eq!(parent.code, 0, "{}", parent.stderr);
    assert_eq!(
        parent.json()["objects"][0]["children"],
        json!([{"smut_id": "child"}]),
        "the delete does not cascade into the referencing entity"
    );

    let child = smut(&scratch, &["get", "child"]);
    assert_eq!(child.code, 0, "{}", child.stderr);
    assert_eq!(child.json()["missing"], json!(["child"]));
    assert_eq!(child.json()["objects"], json!([]));
}

#[test]
fn delete_of_only_missing_ids_is_success_and_removes_nothing() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(&scratch, &["delete", "nope"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.json(), json!({"deleted": [], "missing": ["nope"]}));

    let remaining = smut(&scratch, &["query", "{}"]);
    assert_eq!(
        remaining.json()["objects"]
            .as_array()
            .expect("an array")
            .len(),
        3
    );
}

#[test]
fn delete_against_an_absent_store_exits_one_and_creates_nothing() {
    let scratch = Scratch::new();

    let ran = smut(&scratch, &["delete", "nope"]);

    assert_eq!(ran.code, 1, "{}", ran.stderr);
    assert_eq!(ran.stdout, "");
    assert!(ran.stderr.contains("no store"), "{}", ran.stderr);
    assert!(!scratch.has_store());
}

// ── VT-5: the traversal grammar through the binary (§5) ─────────────────────

#[test]
fn the_traversal_keys_accept_the_wildcard_as_a_whole_value() {
    // The wildcard has one legal spelling: `"*"` as the key's whole value.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    for specification in [r#"{"from":"a","follow":"*"}"#, r#"{"backlinks":"*"}"#] {
        let ran = smut(&scratch, &["query", specification]);

        assert_eq!(ran.code, 0, "{specification}: {}", ran.stderr);
        assert!(ran.stderr.is_empty(), "{specification}: {}", ran.stderr);
        assert_eq!(
            ran.stdout.matches('\n').count(),
            1,
            "{specification} is one compact line: {:?}",
            ran.stdout
        );

        let envelope = ran.json();
        assert!(
            envelope.get("traversal").is_some(),
            "{specification} supplied a traversal key, so the envelope carries it: {envelope}"
        );
    }
}

#[test]
fn a_wildcard_array_element_is_refused_naming_the_string_form() {
    // An array must contain field names; an element `"*"` is refused, and the
    // refusal points at the string form, because a mixed list would need a
    // precedence rule between wildcard lexical order and named-field query
    // order. It is a malformed value, not a key a later release will accept, so
    // it names no release.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    for (key, specification) in [
        ("follow", r#"{"from":"a","follow":["*"]}"#),
        ("backlinks", r#"{"backlinks":["*"]}"#),
    ] {
        let ran = smut(&scratch, &["query", specification]);

        assert_eq!(ran.code, 2, "{specification}: {}", ran.stderr);
        assert_eq!(
            ran.stdout, "",
            "{specification}: a refusal prints no envelope"
        );
        for needle in [key, "*", "string"] {
            assert!(
                ran.stderr.contains(needle),
                "{specification}: the refusal names {needle:?}: {}",
                ran.stderr
            );
        }
        assert!(
            !ran.stderr.contains("0.2"),
            "{specification}: the wildcard is spelled as the string on its own, so this is not \
             a key deferred to a release: {}",
            ran.stderr
        );
    }
}

#[test]
fn a_requested_field_no_entity_carries_is_accepted() {
    // A named field nobody uses is not a defect: it simply follows nothing.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(
        &scratch,
        &["query", r#"{"from":"a","follow":["nonexistent"]}"#],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    let envelope = ran.json();
    assert_eq!(
        envelope["traversal"]["frontier"],
        json!([]),
        "no target was discovered, so nothing is left unexpanded: {envelope}"
    );
    assert_eq!(
        envelope["traversal"]["depths"],
        json!({"a": 0}),
        "{envelope}"
    );
}

#[test]
fn depth_zero_is_valid_and_selects_the_roots_alone() {
    // `depth` is nonnegative and defaults to 1; zero is a legal, roots-only
    // walk — every root is discovered, none is expanded, so every root is also
    // frontier.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(
        &scratch,
        &["query", r#"{"from":"a","depth":0,"follow":"*"}"#],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    let envelope = ran.json();
    assert_eq!(
        envelope["traversal"]["frontier"],
        json!(["a"]),
        "the root was discovered and declined: {envelope}"
    );
    assert_eq!(envelope["traversal"]["depths"], json!({"a": 0}));
    assert_eq!(envelope["traversal"]["edges"], json!([]));
    assert_eq!(
        envelope["objects"],
        json!([{"body": "First.", "smut_id": "a", "priority": 3, "title": "Alpha"}]),
        "a roots-only walk returns the root alone: {envelope}"
    );
}

#[test]
fn a_bad_depth_is_refused_before_any_store_access() {
    // A malformed value rejects the whole request at step 1 of the pipeline,
    // before the store is opened, locked or created. There is no store in this
    // scratch directory, so a refusal that reached the store would be the
    // exit-1 "no store" answer instead. The diagnostic must describe the shape
    // of an accepted value rather than defer the key to a later release, since
    // a refusal that only says "not supported" would satisfy the rest of
    // this test without the key ever being accepted.
    let scratch = Scratch::new();

    for specification in [
        r#"{"depth":-1}"#,
        r#"{"depth":1.5}"#,
        r#"{"depth":18446744073709551616}"#,
    ] {
        let ran = smut(&scratch, &["query", specification]);

        assert_eq!(ran.code, 2, "{specification}: {}", ran.stderr);
        assert_eq!(
            ran.stdout, "",
            "{specification}: a refusal prints no envelope"
        );
        assert!(
            ran.stderr.contains("depth"),
            "{specification}: the refusal names the key: {}",
            ran.stderr
        );
        assert!(
            ran.stderr.contains("nonnegative"),
            "{specification}: the refusal describes the accepted value shape: {}",
            ran.stderr
        );
        assert!(
            !ran.stderr.contains("0.2"),
            "{specification}: a malformed value is not a key deferred to a later release: {}",
            ran.stderr
        );
        assert!(
            !scratch.has_store(),
            "{specification} was refused before the store was touched"
        );
    }
}

// ── PHASE-02 VT-3: the budget's grammar and its refusals ─────────────────────

#[test]
fn a_malformed_budget_is_refused_before_any_store_access() {
    // §5 "a negative or zero limit/budget … rejects with exit 2 before the store
    // is opened, locked or created" and "Depth and budget are unsigned": a
    // zero, negative, fractional, stringly or beyond-`u64` budget is a malformed
    // value, not a key deferred to a later release, so no refusal may name 0.2.
    //
    // A fresh scratch per case: there is no store to reach, so a refusal that
    // opened one would leave it behind and the exit code would be the "no store"
    // answer instead of step 1's refusal.
    let beyond = serde_json::from_str::<Value>("18446744073709551616")
        .expect("the literal is one JSON number");

    let specifications = [
        r#"{"budget":0}"#.to_owned(),
        r#"{"budget":-1}"#.to_owned(),
        r#"{"budget":1.5}"#.to_owned(),
        r#"{"budget":1.0}"#.to_owned(),
        r#"{"budget":"5"}"#.to_owned(),
        format!(r#"{{"budget":{beyond}}}"#),
    ];

    for specification in specifications {
        let scratch = Scratch::new();
        let ran = smut(&scratch, &["query", &specification]);

        assert_eq!(ran.code, 2, "{specification}: {}", ran.stderr);
        assert_eq!(
            ran.stdout, "",
            "{specification}: a refusal prints no envelope"
        );
        assert!(
            ran.stderr.contains("budget"),
            "{specification}: the refusal names the key: {}",
            ran.stderr
        );
        assert!(
            ran.stderr.contains("a positive integer"),
            "{specification}: the refusal describes the accepted value shape: {}",
            ran.stderr
        );
        assert!(
            !ran.stderr.contains("0.2"),
            "{specification}: a malformed value is not a key deferred to a later release: {}",
            ran.stderr
        );
        assert!(
            !scratch.has_store(),
            "{specification} was refused before the store was touched"
        );
    }
}

#[test]
fn an_unknown_key_stays_unknown_and_names_no_release() {
    // §5 "Unknown keys stay unknown": `FUTURE_KEYS` retired with `where` in
    // PHASE-03, so `sort` is reported plainly rather than promised to a release
    // — and `budget`'s arrival must not turn every unrecognised key into a
    // future key.
    let scratch = Scratch::new();
    let ran = smut(&scratch, &["query", r#"{"sort":[{"field":"title"}]}"#]);

    assert_eq!(ran.code, 2, "{}", ran.stderr);
    assert_eq!(ran.stdout, "", "a refusal prints no envelope");
    assert!(
        ran.stderr.contains("sort"),
        "the refusal names the key: {}",
        ran.stderr
    );
    assert!(
        !ran.stderr.contains("0.2"),
        "an unknown key is not deferred to a release: {}",
        ran.stderr
    );
    assert!(
        !scratch.has_store(),
        "the refusal happened before the store was touched"
    );
}

#[test]
fn an_accepted_budget_bounds_the_walk_and_an_exact_fit_is_not_truncated() {
    // §7.3: `budget` counts entity expansions and is checked before expanding,
    // so a roots-only walk of one permitted expansion fits a budget of one
    // exactly — and a budget that is honoured is not a truncation.
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"a","title":"A","next":{"smut_id":"b"}}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"b","title":"B"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(
        &scratch,
        &["query", r#"{"from":"a","follow":["next"],"budget":1}"#],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    let envelope = ran.json();
    assert_eq!(
        envelope["truncation_reasons"],
        json!([]),
        "the walk expanded exactly its budget: {envelope}"
    );
    assert_eq!(
        envelope["traversal"]["frontier"],
        json!(["b"]),
        "the depth boundary holds the discovery: {envelope}"
    );
}

// ── PHASE-01/VT-2: a stored identity must be a nonempty string ───────────────
// The malformed_stored_identity refusals below cover both traversals — `get`
// and `follow` — while the same shape arriving in a request stays a user error.

/// Overwrite the entity file the store's index maps `id` to with `raw`, and
/// answer with that file's basename — the name every load refusal carries.
///
/// The caller declares the store through the CLI first, so what the corruption
/// breaks is a store the binary itself wrote; only the one file changes, and
/// only inside the caller's scratch directory.
fn corrupt_nested_identity(scratch: &Scratch, id: &str, raw: &str) -> String {
    let index_path = scratch.store().join("index.json");
    let index: Value = serde_json::from_slice(&fs::read(&index_path).expect("the store's index"))
        .expect("the index is JSON");
    let basename = index["objects"][id]
        .as_str()
        .unwrap_or_else(|| panic!("the index maps {id} to a basename: {index}"))
        .to_owned();

    fs::write(scratch.store().join(&basename), raw).expect("the corrupted entity file");

    basename
}

#[test]
fn get_refuses_malformed_stored_identity_with_its_path() {
    // The entity-node rule holds for stored entities as well as requested ones:
    // a `smut_id` present below the top level and not a nonempty string is
    // corruption, never data, so the store refuses to load at all. A corrupt
    // store is not a user error, hence 1 rather than 2.
    let scratch = Scratch::new();
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"Alpha"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let basename = corrupt_nested_identity(
        &scratch,
        "a",
        r#"{"smut_id":"a","title":"Alpha","notes":{"payload":{"smut_id":null}}}"#,
    );

    let ran = smut(&scratch, &["get", "a"]);

    assert_eq!(
        ran.code, 1,
        "a corrupt store is a store failure, not a rejected request: {}",
        ran.stderr
    );
    assert!(
        ran.stdout.is_empty(),
        "a refusal prints no envelope at all: {:?}",
        ran.stdout
    );
    assert!(
        ran.stderr.starts_with("smut: "),
        "the diagnostic carries the binary's prefix: {:?}",
        ran.stderr
    );
    assert!(
        ran.stderr.contains(&basename),
        "the refusal names the file that holds the corruption ({basename}): {:?}",
        ran.stderr
    );
    assert!(
        ran.stderr
            .contains(r#"identity at $["notes"]["payload"]["smut_id"] must be a nonempty string"#),
        "the refusal names the path of the offending id: {:?}",
        ran.stderr
    );
}

#[test]
fn follow_refuses_malformed_stored_identity_outside_the_followed_field() {
    // The whole store is validated as it is loaded, so a malformation nobody's
    // traversal reaches still refuses the traversal: `links` is what `follow`
    // walks, and `notes` is what breaks the load.
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"a","links":[{"smut_id":"b"}]}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"b","title":"Beta"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let basename = corrupt_nested_identity(
        &scratch,
        "a",
        r#"{"smut_id":"a","links":[{"smut_id":"b"}],"notes":{"payload":{"smut_id":null}}}"#,
    );

    let ran = smut(&scratch, &["query", r#"{"from":["a"],"follow":["links"]}"#]);

    assert_eq!(
        ran.code, 1,
        "a corrupt store refuses the traversal: {}",
        ran.stderr
    );
    assert!(
        ran.stdout.is_empty(),
        "a refusal prints no envelope at all: {:?}",
        ran.stdout
    );
    assert!(
        ran.stderr.contains(&basename),
        "the refusal names the file that holds the corruption ({basename}): {:?}",
        ran.stderr
    );
    assert!(
        ran.stderr
            .contains(r#"identity at $["notes"]["payload"]["smut_id"] must be a nonempty string"#),
        "the refusal names the path of the offending id: {:?}",
        ran.stderr
    );
}

#[test]
fn follow_over_the_same_graph_uncorrupted_returns_both_entities() {
    // The control for the refusal above: this graph and this traversal are
    // healthy, so the refusal is the corruption and not the shape of the walk.
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"a","links":[{"smut_id":"b"}]}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"b","title":"Beta"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(&scratch, &["query", r#"{"from":["a"],"follow":["links"]}"#]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);

    let envelope = ran.json();
    let ids: Vec<&str> = envelope["objects"]
        .as_array()
        .expect("an array of entities")
        .iter()
        .map(|entity| {
            entity["smut_id"]
                .as_str()
                .expect("every object carries an id")
        })
        .collect();
    assert_eq!(
        ids,
        vec!["a", "b"],
        "the walk reaches the root and the entity its `links` names: {envelope}"
    );
}

#[test]
fn render_refuses_the_same_corrupt_store() {
    // `render` is a traversal over the same load boundary, so it refuses too,
    // and prints no text rather than a partial document.
    let scratch = Scratch::new();
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"Alpha"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    corrupt_nested_identity(
        &scratch,
        "a",
        r#"{"smut_id":"a","title":"Alpha","notes":{"payload":{"smut_id":null}}}"#,
    );

    let ran = smut(&scratch, &["render", "{}"]);

    assert_eq!(
        ran.code, 1,
        "a corrupt store refuses the rendering: {}",
        ran.stderr
    );
    assert!(
        ran.stdout.is_empty(),
        "a refusal prints no document at all: {:?}",
        ran.stdout
    );
}

#[test]
fn a_request_side_malformed_identity_stays_a_user_error() {
    // The same malformation arriving in a request is the caller's mistake, not
    // the store's: it is refused before any store access, at exit 2.
    let scratch = Scratch::new();

    for (declaration, found) in [
        (
            r#"{"smut_id":"req-bad","payload":{"smut_id":null}}"#,
            "null",
        ),
        (
            r#"{"smut_id":"req-bad","payload":{"smut_id":[]}}"#,
            "an array",
        ),
    ] {
        let ran = smut(&scratch, &["declare", declaration]);

        assert_eq!(ran.code, 2, "{declaration}: {}", ran.stderr);
        assert_eq!(
            ran.stdout, "",
            "{declaration}: a rejection prints no envelope"
        );
        assert!(
            ran.stderr.contains("invalid smut_id"),
            "{declaration}: the refusal names the identity rule: {}",
            ran.stderr
        );
        assert!(
            ran.stderr.contains(found),
            "{declaration}: the refusal names what the id held ({found}): {}",
            ran.stderr
        );
        assert!(
            ran.stderr.contains("must be a nonempty string"),
            "{declaration}: the refusal states the rule: {}",
            ran.stderr
        );
    }
}

// ── PHASE-02/VT-2: `smut_id` is structural and ordinary `id` is data ──────────
// The rename frees `id` for caller data at every depth while the structural key
// keeps controlling graph shape. These cases are the public round trips: root
// generation, nesting, predicates, projections, receipts, the patch sentinel and
// the unchanged response containers.

#[test]
fn a_root_ordinary_id_becomes_data_under_a_generated_identity() {
    // §2: a nonempty declaration root without `smut_id` takes a generated
    // identity, so an imported payload's own `id` is stored rather than read as
    // the entity's name.
    let scratch = Scratch::new();

    let declared = smut(&scratch, &["declare", r#"{"id":"external","title":"T"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let envelope = declared.json();
    let ids = envelope["ids"].as_array().expect("ids is an array");
    assert_eq!(ids.len(), 1, "one declaration root: {envelope}");
    let generated = ids[0].as_str().expect("identity is an opaque string");
    assert_ne!(
        generated, "external",
        "an ordinary id does not name the entity: {envelope}"
    );

    let fetched = smut(&scratch, &["get", generated]);
    assert_eq!(fetched.code, 0, "{}", fetched.stderr);
    let envelope = fetched.json();
    assert_eq!(
        envelope["objects"],
        json!([{"smut_id": generated, "id": "external", "title": "T"}]),
        "identity addresses the entity and `id` is served as data: {envelope}"
    );
}

#[test]
fn ordinary_id_values_at_depth_round_trip_as_data() {
    // §2 and design §5: a nested ordinary `id` of null, empty string, number,
    // array or object names nothing, so it hoists no entity and leaves no
    // `missing` entry.
    let scratch = Scratch::new();
    let payload = json!({
        "nullable": {"id": null},
        "empty": {"id": ""},
        "number": {"id": 7},
        "list": [{"id": "a"}, {"id": ["b", {"id": "c"}]}],
        "object": {"id": {"kind": "object"}},
    });
    let declaration =
        json!({"smut_id": "deep", "title": "Nested", "payload": payload.clone()}).to_string();

    let declared = smut(&scratch, &["declare", declaration.as_str()]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let envelope = declared.json();
    assert_eq!(envelope["ids"], json!(["deep"]), "{envelope}");
    assert_eq!(
        envelope["missing"],
        json!([]),
        "no ordinary `id` value was read as identity: {envelope}"
    );
    assert_eq!(
        envelope["changes"].as_array().expect("an array").len(),
        1,
        "the nested `id` values hoisted nothing: {envelope}"
    );

    let fetched = smut(&scratch, &["get", "deep"]);
    assert_eq!(fetched.code, 0, "{}", fetched.stderr);
    let envelope = fetched.json();
    assert_eq!(
        envelope["objects"],
        json!([{"smut_id": "deep", "title": "Nested", "payload": payload}]),
        "the payload round-trips unchanged: {envelope}"
    );

    let everything = smut(&scratch, &["query", "{}"]);
    assert_eq!(everything.code, 0, "{}", everything.stderr);
    assert_eq!(
        everything.json()["objects"]
            .as_array()
            .expect("an array")
            .len(),
        1,
        "exactly one entity was created"
    );
}

#[test]
fn an_object_mixing_id_and_smut_id_hoists_identity_and_keeps_id_as_data() {
    // §2's table: `smut_id` plus other fields is a declaration, hoisted and
    // replaced by a key-only reference, and the ordinary `id` it also carries is
    // one of the declared entity's fields.
    let scratch = Scratch::new();

    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"parent","title":"Parent","external":{"smut_id":"imported","id":"caller-data"}}"#,
        ],
    );

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let envelope = declared.json();
    assert_eq!(envelope["ids"], json!(["parent"]), "{envelope}");
    assert_eq!(envelope["missing"], json!([]), "{envelope}");

    let parent = smut(&scratch, &["get", "parent"]);
    assert_eq!(parent.code, 0, "{}", parent.stderr);
    assert_eq!(
        parent.json()["objects"],
        json!([{
            "smut_id": "parent",
            "external": {"smut_id": "imported"},
            "title": "Parent",
        }]),
        "the occurrence is the key-only reference, not the embedded entity"
    );

    let imported = smut(&scratch, &["get", "imported"]);
    assert_eq!(imported.code, 0, "{}", imported.stderr);
    assert_eq!(
        imported.json()["objects"],
        json!([{"smut_id": "imported", "id": "caller-data"}]),
        "the ordinary id is data on the entity the structural key names"
    );
}

#[test]
fn a_where_on_smut_id_selects_by_identity_and_on_id_selects_data() {
    // §2: field names still address the actual top-level field, so `smut_id`
    // addresses identity and ordinary `id` addresses caller data.
    let scratch = Scratch::new();

    for declaration in [
        r#"{"smut_id":"a","title":"Alpha","id":"external-a"}"#,
        r#"{"smut_id":"b","title":"Beta"}"#,
        r#"{"smut_id":"c","title":"Gamma","id":"external-c"}"#,
    ] {
        let declared = smut(&scratch, &["declare", declaration]);

        assert_eq!(declared.code, 0, "{declaration}: {}", declared.stderr);
    }

    let identity = smut(&scratch, &["query", r#"{"where":{"smut_id":"b"}}"#]);
    assert_eq!(identity.code, 0, "{}", identity.stderr);
    assert_eq!(
        identity.json()["objects"],
        json!([{"smut_id": "b", "title": "Beta"}]),
        "identity selects its own entity: {}",
        identity.json()
    );

    let data = smut(&scratch, &["query", r#"{"where":{"id":"external-a"}}"#]);
    assert_eq!(data.code, 0, "{}", data.stderr);
    assert_eq!(
        data.json()["objects"],
        json!([{"smut_id": "a", "id": "external-a", "title": "Alpha"}]),
        "only the entity carrying that value as data is selected: {}",
        data.json()
    );

    let cross = smut(&scratch, &["query", r#"{"where":{"id":"b"}}"#]);
    assert_eq!(cross.code, 0, "{}", cross.stderr);
    assert_eq!(
        cross.json()["objects"],
        json!([]),
        "an identity value is not an ordinary `id` value: {}",
        cross.json()
    );
}

#[test]
fn select_returns_identity_alone_or_identity_beside_the_ordinary_id() {
    // §2: every projected entity retains `smut_id`, `select: []`-style identity
    // is what a selection of the identity field yields, and ordinary `id` is
    // projected like any other data field.
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"a","title":"Alpha","id":"external-a"}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let identity = smut(&scratch, &["query", r#"{"from":"a","select":["smut_id"]}"#]);
    assert_eq!(identity.code, 0, "{}", identity.stderr);
    assert_eq!(
        identity.json()["objects"],
        json!([{"smut_id": "a"}]),
        "identity alone: {}",
        identity.json()
    );

    let data = smut(&scratch, &["query", r#"{"from":"a","select":["id"]}"#]);
    assert_eq!(data.code, 0, "{}", data.stderr);
    assert_eq!(
        data.json()["objects"],
        json!([{"smut_id": "a", "id": "external-a"}]),
        "the data field plus the retained identity: {}",
        data.json()
    );
}

#[test]
fn order_by_an_ordinary_id_keeps_ordinary_field_semantics() {
    // DEC-006: ordering reads the named top-level field, so an ordinary `id`
    // sorts by its data value and an entity without one sorts last.
    let scratch = Scratch::new();

    for declaration in [
        r#"{"smut_id":"alpha-entity","id":"zzz"}"#,
        r#"{"smut_id":"beta-entity","title":"Beta"}"#,
        r#"{"smut_id":"gamma-entity","id":"aaa"}"#,
    ] {
        let declared = smut(&scratch, &["declare", declaration]);

        assert_eq!(declared.code, 0, "{declaration}: {}", declared.stderr);
    }

    let ran = smut(&scratch, &["query", r#"{"order":[{"field":"id"}]}"#]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    let envelope = ran.json();
    let order: Vec<&str> = envelope["objects"]
        .as_array()
        .expect("an array of entities")
        .iter()
        .map(|entity| {
            entity["smut_id"]
                .as_str()
                .expect("every projected entity keeps its identity")
        })
        .collect();
    assert_eq!(
        order,
        vec!["gamma-entity", "alpha-entity", "beta-entity"],
        "data values ascend and absence sorts last: {envelope}"
    );
}

#[test]
fn a_where_on_the_old_spelling_is_an_empty_result_not_a_refusal() {
    // DEC-027: a field no entity carries fails every operator exactly as a
    // missing field on one entity does, so an old-spelling `id` query is an
    // empty result at exit 0 — not an error, and not a refusal.
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"holder","title":"Holder","payload":{"id":"a","title":"Embedded"}}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let control = smut(&scratch, &["query", r#"{"where":{"smut_id":"holder"}}"#]);
    assert_eq!(control.code, 0, "{}", control.stderr);
    assert_eq!(
        control.json()["objects"]
            .as_array()
            .expect("an array")
            .len(),
        1,
        "the store holds the entity, so the empty result is about the field: {}",
        control.json()
    );

    let ran = smut(&scratch, &["query", r#"{"where":{"id":"a"}}"#]);

    assert_eq!(
        ran.code, 0,
        "an unheld field is not a refused request: {}",
        ran.stderr
    );
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);
    let envelope = ran.json();
    assert_eq!(
        envelope["objects"],
        json!([]),
        "the nested ordinary id is data, not a top-level field: {envelope}"
    );
}

#[test]
fn a_change_receipt_names_its_subject_smut_id_with_a_full_previous_entity() {
    // §2: only the subject field changes in the envelope. `previous` stays the
    // whole pre-batch entity under the new key, `--compact` still omits it, and
    // the container names are untouched.
    let scratch = Scratch::new();

    let created = smut(
        &scratch,
        &["declare", r#"{"smut_id":"r","title":"one","id":"data"}"#],
    );
    assert_eq!(created.code, 0, "{}", created.stderr);
    let envelope = created.json();
    let change = &envelope["changes"][0];
    assert_eq!(change["smut_id"], json!("r"), "{envelope}");
    assert_eq!(change["status"], json!("created"), "{envelope}");
    assert_eq!(change["previous"], Value::Null, "{envelope}");
    assert!(
        change.get("id").is_none(),
        "the subject field is `smut_id`, not `id`: {change}"
    );
    assert_eq!(envelope["ids"], json!(["r"]), "{envelope}");
    assert_eq!(envelope["missing"], json!([]), "{envelope}");

    let updated = smut(&scratch, &["declare", r#"{"smut_id":"r","title":"two"}"#]);
    assert_eq!(updated.code, 0, "{}", updated.stderr);
    let envelope = updated.json();
    let change = &envelope["changes"][0];
    assert_eq!(change["status"], json!("updated"), "{envelope}");
    assert_eq!(
        change["previous"],
        json!({"smut_id": "r", "title": "one", "id": "data"}),
        "`previous` is the whole pre-batch entity under `smut_id`: {envelope}"
    );

    let compact = smut(
        &scratch,
        &["declare", r#"{"smut_id":"r","title":"three"}"#, "--compact"],
    );
    assert_eq!(compact.code, 0, "{}", compact.stderr);
    let envelope = compact.json();
    assert_eq!(envelope["ids"], json!(["r"]), "{envelope}");
    assert_eq!(envelope["missing"], json!([]), "{envelope}");
    assert!(
        envelope["changes"][0].get("previous").is_none(),
        "`--compact` omits `previous`: {envelope}"
    );

    let deleted = smut(&scratch, &["delete", "r", "absent"]);
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    assert_eq!(
        deleted.json(),
        json!({"deleted": ["r"], "missing": ["absent"]}),
        "the delete containers keep their names"
    );
}

#[test]
fn drop_applies_to_an_ordinary_id_field() {
    // DEC-002: the top-level `"DROP"` sentinel is patch algebra over ordinary
    // fields, so it removes an ordinary `id` exactly as it removes any field.
    let scratch = Scratch::new();
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"x","id":"v"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let before = smut(&scratch, &["get", "x"]);
    assert_eq!(before.code, 0, "{}", before.stderr);
    assert_eq!(
        before.json()["objects"],
        json!([{"smut_id": "x", "id": "v"}]),
        "the ordinary id is data before the drop: {}",
        before.json()
    );

    let dropped = smut(&scratch, &["declare", r#"{"smut_id":"x","id":"DROP"}"#]);
    assert_eq!(dropped.code, 0, "{}", dropped.stderr);
    assert_eq!(dropped.json()["changes"][0]["status"], json!("updated"));

    let after = smut(&scratch, &["get", "x"]);
    assert_eq!(after.code, 0, "{}", after.stderr);
    assert_eq!(
        after.json()["objects"],
        json!([{"smut_id": "x"}]),
        "the sentinel removed the data field and left the identity: {}",
        after.json()
    );
}

#[test]
fn a_traversal_response_keeps_its_container_names() {
    // §1 and §2: response containers keep their names, so a traversal still
    // reports `frontier`, `depths`, `edges` with `source`/`target` and
    // `dangling` with `source`/`target`.
    let scratch = Scratch::new();
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"kept","title":"K"}"#]);
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"smut_id":"root","links":[{"smut_id":"kept"},{"smut_id":"gone"}]}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert_eq!(
        declared.json()["missing"],
        json!(["gone"]),
        "the references are structural, so the absent one is unresolved: {}",
        declared.json()
    );

    let ran = smut(
        &scratch,
        &["query", r#"{"from":"root","follow":["links"]}"#],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    let envelope = ran.json();
    let traversal = &envelope["traversal"];
    assert_eq!(
        traversal["frontier"],
        json!(["kept"]),
        "the discovered but unexpanded id: {envelope}"
    );
    assert_eq!(
        traversal["depths"],
        json!({"kept": 1, "root": 0}),
        "{envelope}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "root", "field": "links", "target": "kept"}]),
        "edges keep `source` and `target`: {envelope}"
    );
    assert_eq!(
        envelope["dangling"],
        json!([{"source": "root", "field": "links", "target": "gone"}]),
        "dangling entries keep `source` and `target`: {envelope}"
    );

    let mut returned: Vec<&str> = envelope["objects"]
        .as_array()
        .expect("an array of entities")
        .iter()
        .map(|entity| entity["smut_id"].as_str().expect("identity"))
        .collect();
    returned.sort_unstable();
    assert_eq!(returned, vec!["kept", "root"], "{envelope}");
}

// ── smut q: the verb end to end (PHASE-02, EX-1…EX-8) ──────────────────────
//
// The compiler is `tests/dsl.rs`'s. What is here is the verb: that one line of
// words and its JSON twin come out of the same pipeline with the same bytes, that
// every refusal arrives whole and at the right code, and that `q` creates nothing.

/// The recipe's store: a session, and two questions that reach it by `part-of`,
/// one open and one resolved. `q-other`'s title carries "graph" so the compound
/// condition in `operators_and_commas_reach_the_compiler_verbatim` selects exactly
/// one entity.
fn store_of_a_session_and_two_questions(scratch: &Scratch) {
    for declaration in [
        r#"{"smut_id":"session","title":"Session"}"#,
        r#"{"smut_id":"q-query","type":"question","status":"open","title":"Query shape","part-of":[{"smut_id":"session"}]}"#,
        r#"{"smut_id":"q-other","type":"question","status":"resolved","title":"A graph view","part-of":[{"smut_id":"session"}]}"#,
    ] {
        let declared = smut(scratch, &["declare", declaration]);
        assert_eq!(declared.code, 0, "{declaration}: {}", declared.stderr);
    }
}

/// The arguments a printed `smut q` line denotes: the verb, then the words the
/// guide prints, cut where a shell would cut them. The words come from the
/// constant, so the line asserted here is the line the guide types.
fn q_words(printed: &str) -> Vec<&str> {
    std::iter::once("q")
        .chain(printed.split_whitespace())
        .collect()
}

/// Runs one `smut q` line and the JSON specification it denotes on the same store,
/// and asserts both succeeded with the *same bytes* on stdout — the evidence that
/// `q` is a front-end and not a second engine (design §7).
fn same_answer(scratch: &Scratch, words: &[&str], specification: &str) -> Ran {
    let from_words = smut(scratch, words);
    let from_json = smut(scratch, &["query", specification]);

    assert_eq!(from_words.code, 0, "{words:?}: {}", from_words.stderr);
    assert_eq!(from_json.code, 0, "{specification}: {}", from_json.stderr);
    assert_eq!(
        from_words.stdout, from_json.stdout,
        "{words:?} and {specification} are one pipeline"
    );

    from_words
}

/// The `smut_id`s an envelope selected, in the order it returned them.
fn selected_ids(envelope: &Value) -> Vec<String> {
    envelope["objects"]
        .as_array()
        .unwrap_or_else(|| panic!("an envelope of entities: {envelope}"))
        .iter()
        .map(|entity| {
            entity["smut_id"]
                .as_str()
                .expect("a stored identity")
                .to_owned()
        })
        .collect()
}

#[test]
fn the_two_call_recipe_reads_the_same_both_ways() {
    // EX-1…EX-4: the guide's two calls, each typed as words and as JSON, are one
    // answer byte for byte. The traverse and follow lines answer *differently*, so
    // neither can be an accidentally empty envelope.
    let scratch = Scratch::new();
    store_of_a_session_and_two_questions(&scratch);

    let traversed = same_answer(&scratch, &q_words(EXAMPLE_TRAVERSE_Q), EXAMPLE_TRAVERSE);
    let mut reached = selected_ids(&traversed.json());
    reached.sort_unstable();
    assert_eq!(
        reached,
        ["q-other", "q-query", "session"].map(String::from),
        "both hops back through part-of from the session: {}",
        traversed.json()
    );

    let followed = same_answer(&scratch, &q_words(EXAMPLE_FOLLOW_Q), EXAMPLE_FOLLOW);
    let mut walked = selected_ids(&followed.json());
    walked.sort_unstable();
    assert_eq!(
        walked,
        ["session"].map(String::from),
        "the session stores no part-of, so the other direction reaches it alone: {}",
        followed.json()
    );
    assert_ne!(
        walked, reached,
        "the two lines are a pair because they answer different questions"
    );

    // The filter twin is *not* the JSON twin: bare roots are this surface's spelling
    // of a `$in` on identity, so the two agree on the selected set rather than on an
    // envelope or on an order (design §5.2).
    let line = EXAMPLE_FILTER_Q.replace("IDS", "q-query q-other");
    let from_words = smut(&scratch, &q_words(&line));
    let from_json = smut(
        &scratch,
        &[
            "query",
            &EXAMPLE_FILTER.replace("IDS", r#"["q-query","q-other"]"#),
        ],
    );

    assert_eq!(from_words.code, 0, "{line}: {}", from_words.stderr);
    assert_eq!(from_json.code, 0, "{}", from_json.stderr);

    let mut selected = selected_ids(&from_words.json());
    let mut twin = selected_ids(&from_json.json());
    selected.sort_unstable();
    twin.sort_unstable();

    assert_eq!(
        selected,
        ["q-query"].map(String::from),
        "the open half of the reached pair: {}",
        from_words.json()
    );
    assert_eq!(
        selected,
        twin,
        "the two spellings select the same entities: {}",
        from_words.json()
    );
}

#[test]
fn q_creates_no_store_and_a_declare_beside_it_does() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["q", "session"]);

    assert_eq!(refused.code, 1, "a read with no store: {}", refused.stderr);
    assert_eq!(refused.stdout, "", "a failure prints no success envelope");
    assert!(!scratch.has_store(), "q creates nothing");

    // Paired in the same directory, so the assertion above discriminates: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"session","title":"S"}"#],
    );

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(scratch.has_store(), "declare creates the store q did not");
}

#[test]
fn tokens_keep_their_order_around_flags() {
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let between = same_answer(
        &scratch,
        &["q", "c", "--ids", "a", "b"],
        r#"{"from":["c","a","b"],"select":["smut_id"]}"#,
    );
    assert_eq!(
        selected_ids(&between.json()),
        ["c", "a", "b"].map(String::from),
        "a flag between the roots does not reorder them: {}",
        between.json()
    );

    let leading = same_answer(
        &scratch,
        &["q", "--ids", "a", "b"],
        r#"{"from":["a","b"],"select":["smut_id"]}"#,
    );
    assert_eq!(
        selected_ids(&leading.json()),
        ["a", "b"].map(String::from),
        "a leading flag does not swallow the tokens: {}",
        leading.json()
    );

    let trailing = same_answer(
        &scratch,
        &["q", "a", "b", "--ids"],
        r#"{"from":["a","b"],"select":["smut_id"]}"#,
    );
    assert_eq!(
        selected_ids(&trailing.json()),
        ["a", "b"].map(String::from),
        "a trailing flag does not swallow the tokens: {}",
        trailing.json()
    );

    // `--from` is the escape for an id the grammar cannot carry bare, and it appends
    // after the bare roots in write order.
    let escaped = same_answer(
        &scratch,
        &["q", "a", "b", "--from=c", "--ids"],
        r#"{"from":["a","b","c"],"select":["smut_id"]}"#,
    );
    assert_eq!(
        selected_ids(&escaped.json()),
        ["a", "b", "c"].map(String::from),
        "--from appends after the bare roots: {}",
        escaped.json()
    );
}

#[test]
fn a_token_after_a_value_flag_is_that_flags_value() {
    // Standard `--flag value` convention, and the grammar's consequence rather than
    // clap's quirk: `--follow session` is a direction named `session`, never a root.
    let scratch = Scratch::new();
    store_of_a_session_and_two_questions(&scratch);

    let ran = same_answer(
        &scratch,
        &["q", "--follow", "session", "status=open"],
        r#"{"follow":["session"],"where":{"status":{"$eq":"open"}}}"#,
    );

    assert_eq!(
        selected_ids(&ran.json()),
        ["q-query"].map(String::from),
        "`session` is --follow's value and never a root: {}",
        ran.json()
    );
}

#[test]
fn operators_and_commas_reach_the_compiler_verbatim() {
    // `~`, `!` and `,` are all shell-active; the shell already quoted them, and the
    // compiler must receive the words it was given.
    let scratch = Scratch::new();
    store_of_a_session_and_two_questions(&scratch);

    let ran = same_answer(
        &scratch,
        &["q", "title~graph", "status!=open", "status=open,resolved"],
        r#"{"where":{"status":{"$ne":"open","$in":["open","resolved"]},"title":{"$contains":"graph"}}}"#,
    );

    assert_eq!(
        selected_ids(&ran.json()),
        ["q-other"].map(String::from),
        "resolved, and the only title carrying graph: {}",
        ran.json()
    );
}

#[test]
fn ids_and_select_cannot_both_be_asked_for() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["q", "session", "--ids", "--select", "title"]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "");
    assert_eq!(
        refused.stderr, "smut: --ids is select smut_id; it cannot also be given --select\n",
        "one line, whole, naming both flags"
    );
}

#[test]
fn depth_without_a_direction_is_refused_while_the_specification_accepts_it() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["q", "--depth", "2"]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "");
    assert_eq!(
        refused.stderr,
        "smut: --depth needs a direction: add --follow, --backlinks, or drop it\n"
    );
    assert!(
        !scratch.has_store(),
        "the refusal happens before any store access"
    );

    // The asymmetry from the other side: the specification still defines `depth`
    // with no direction, and `query` still answers it.
    store_of_three(&scratch);

    let accepted = smut(&scratch, &["query", r#"{"depth":2}"#]);

    assert_eq!(accepted.code, 0, "{}", accepted.stderr);
    assert_eq!(
        selected_ids(&accepted.json()).len(),
        3,
        "a depth with no direction is accepted and walks nothing: {}",
        accepted.json()
    );
}

#[test]
fn a_same_value_contradiction_speaks_the_specifications_sentence() {
    let scratch = Scratch::new();
    store_of_a_session_and_two_questions(&scratch);

    let refused = smut(&scratch, &["q", "status=open", "status!=open"]);
    let twin = smut(
        &scratch,
        &[
            "query",
            r#"{"where":{"status":{"$eq":"open","$ne":"open"}}}"#,
        ],
    );

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "");
    assert!(
        !refused.stderr.is_empty(),
        "an equality of two empty lines would prove nothing"
    );
    assert_eq!(
        refused.stderr, twin.stderr,
        "the same line a JSON caller gets, because the specification is what refuses it"
    );

    // The near miss: a different value is satisfiable, and is answered.
    let answered = same_answer(
        &scratch,
        &["q", "status=open", "status!=resolved"],
        r#"{"where":{"status":{"$eq":"open","$ne":"resolved"}}}"#,
    );

    assert_eq!(
        selected_ids(&answered.json()),
        ["q-query"].map(String::from),
        "{}",
        answered.json()
    );
}

#[test]
fn the_specifications_own_messages_reach_the_human() {
    // Every numeric flag arrives as text, so no numeric refusal is clap's. `--follow
    // any` is present wherever it is required: without a direction the front-end
    // refuses the flag first, and that sentence is the one below.
    let scratch = Scratch::new();

    for (words, sentence) in [
        (
            &["--depth", "abc", "--follow", "any"][..],
            "smut: depth must be a nonnegative integer\n",
        ),
        (
            &["--depth=-1", "--follow", "any"][..],
            "smut: depth must be a nonnegative integer\n",
        ),
        (
            &["--budget", "0", "--follow", "any"][..],
            "smut: budget must be a positive integer\n",
        ),
        (
            &["--limit", "0"][..],
            "smut: limit must be a positive integer\n",
        ),
    ] {
        let mut arguments = vec!["q"];
        arguments.extend_from_slice(words);

        let refused = smut(&scratch, &arguments);

        assert_eq!(refused.code, 2, "{words:?}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{words:?}");
        assert_eq!(refused.stderr, sentence, "{words:?}");
    }

    // The same numeric text with no direction is refused for the missing direction,
    // so the two sentences are told apart.
    let undirected = smut(&scratch, &["q", "--depth", "abc"]);

    assert_eq!(undirected.code, 2, "{}", undirected.stderr);
    assert_eq!(
        undirected.stderr,
        "smut: --depth needs a direction: add --follow, --backlinks, or drop it\n"
    );
}

#[test]
fn q_takes_its_bare_positional_so_the_usage_line_says_tokens() {
    // The load-bearing fact of DEC-037: the positional carries no attribute, so its
    // name is its own field's. `value_name`, `trailing_var_arg` or
    // `allow_hyphen_values` would redden this line.
    let scratch = Scratch::new();

    let help = smut(&scratch, &["q", "--help"]);

    assert_eq!(help.code, 0, "{}", help.stderr);
    assert!(
        help.stdout
            .lines()
            .any(|line| line == "Usage: smut q [OPTIONS] [TOKENS]..."),
        "the usage line names the bare positional: {}",
        help.stdout
    );
    assert!(!scratch.has_store(), "--help touches no store");

    // Beside it, render's positional keeps its own display name: a flattened
    // display-flag struct must leave `[JSON]` alone.
    let render = smut(&scratch, &["render", "--help"]);

    assert_eq!(render.code, 0, "{}", render.stderr);
    assert!(
        render
            .stdout
            .lines()
            .any(|line| line == "Usage: smut render [OPTIONS] [JSON]"),
        "the usage line names render's JSON operand: {}",
        render.stdout
    );

    // EX-6, VT-6: `r` takes the same line `q` takes, so its usage line names the
    // same bare positional with the same word for it.
    let rendered = smut(&scratch, &["r", "--help"]);

    assert_eq!(rendered.code, 0, "{}", rendered.stderr);
    assert!(
        rendered
            .stdout
            .lines()
            .any(|line| line == "Usage: smut r [OPTIONS] [TOKENS]..."),
        "r's usage line names the bare positional: {}",
        rendered.stdout
    );
    assert!(!scratch.has_store(), "--help touches no store");
}

#[test]
fn claps_own_refusals_are_unchanged_and_named() {
    // clap's render is multi-line, so whole-*line* membership is this test's rule;
    // the binary's own one-line refusals are asserted whole elsewhere.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    for (arguments, line) in [
        (
            &["q", "--follo"][..],
            "error: unexpected argument '--follo' found",
        ),
        (
            &["q", "--follow", "a", "--follow", "b"][..],
            "error: the argument '--follow <FOLLOW>' cannot be used multiple times",
        ),
        (
            &["q", "-weird-id"][..],
            "error: unexpected argument '-w' found",
        ),
    ] {
        let refused = smut(&scratch, arguments);

        assert_eq!(refused.code, 2, "{arguments:?}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{arguments:?}");
        assert!(
            refused.stderr.lines().any(|rendered| rendered == line),
            "{arguments:?} must name what was typed, line for line: {:?}",
            refused.stderr
        );
    }

    // An unknown subcommand is this binary's own sentence, and it suggests no near
    // names.
    let unknown = smut(&scratch, &["quer"]);

    assert_eq!(unknown.code, 2, "{}", unknown.stderr);
    assert_eq!(unknown.stdout, "");
    assert_eq!(unknown.stderr, "smut: unknown command: quer\n");
}

#[test]
fn a_double_dash_reaches_a_hyphen_leading_root() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"-weird-id","title":"Weird"}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = same_answer(
        &scratch,
        &["q", "--ids", "--", "-weird-id"],
        r#"{"from":["-weird-id"],"select":["smut_id"]}"#,
    );

    assert_eq!(
        selected_ids(&ran.json()),
        ["-weird-id"].map(String::from),
        "{}",
        ran.json()
    );
}

/// Two invocations are indistinguishable from outside: same exit code, same
/// bytes on stdout, and the same stderr line for line.
fn same_streams(one: Ran, other: Ran, description: &str) -> Ran {
    assert_eq!(one.code, other.code, "{description}: {}", one.stderr);
    assert_eq!(one.stdout, other.stdout, "{description}");
    assert_eq!(one.stderr, other.stderr, "{description}");

    one
}

#[test]
fn q_dash_j_answers_what_query_answers() {
    // `-j` carries the whole specification, so `q -j` is `query` in every
    // respect — the same code, the same envelope, the same refusal, byte for byte.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let specification = r#"{"from":"a","select":["title"]}"#;
    let answered = same_streams(
        smut(&scratch, &["q", "-j", specification]),
        smut(&scratch, &["query", specification]),
        "-j and query answer one specification",
    );
    assert_eq!(answered.code, 0, "{}", answered.stderr);
    assert_eq!(
        selected_ids(&answered.json()),
        ["a"].map(String::from),
        "a non-empty answer, so the equality above is not two empty envelopes: {}",
        answered.json()
    );

    // The same specification on stdin, reached by `-` on both spellings.
    let from_stdin = same_streams(
        run(&scratch, &["q", "-j", "-"], Some(specification)),
        run(&scratch, &["query", "-"], Some(specification)),
        "a specification on stdin reaches the same answer either way",
    );
    assert_eq!(from_stdin.code, 0, "{}", from_stdin.stderr);
    assert_eq!(
        selected_ids(&from_stdin.json()),
        ["a"].map(String::from),
        "{}",
        from_stdin.json()
    );

    // The specification's own refusals, not clap's: `-j` hands the operand to the
    // same parser `query` does, so the sentence is the same one, whole.
    for (operand, description) in [
        ("[1]", "not a JSON object"),
        ("nope", "not JSON at all"),
        ("{} {}", "a second value after the first"),
        (r#"{"limit":1,"limit":2}"#, "a duplicated key"),
    ] {
        let refused = same_streams(
            smut(&scratch, &["q", "-j", operand]),
            smut(&scratch, &["query", operand]),
            description,
        );

        assert_eq!(refused.code, 2, "{operand}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{operand} prints no envelope");
        assert!(!refused.stderr.is_empty(), "{operand} says why");
    }
}

#[test]
fn json_beside_a_selection_is_refused_by_name() {
    // `-j` is the whole specification, so any token or selection flag beside
    // it is a conflict — refused by the one sentence that says how to resolve it.
    let scratch = Scratch::new();

    for arguments in [
        &["q", "a", "-j", "{}"][..],
        &["q", "--ids", "-j", "{}"][..],
        &["q", "--depth", "2", "-j", "{}"][..],
        // `--select=` carries an empty list, and the conflict is checked before the
        // empty-list check, so this is the conflict's sentence and not the
        // empty-select refusal.
        &["q", "--select=", "-j", "{}"][..],
    ] {
        let refused = smut(&scratch, arguments);

        assert_eq!(refused.code, 2, "{arguments:?}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{arguments:?}");
        assert_eq!(
            refused.stderr,
            "smut: -j takes the whole specification; drop the tokens and selection flags beside it\n",
            "{arguments:?}"
        );
        assert!(!scratch.has_store(), "{arguments:?} creates nothing");
    }

    // Paired in the same directory, so the assertions above discriminate: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"A"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(scratch.has_store(), "declare creates the store q did not");
}

// ── VT-2…VT-6: `r`, the render verb on the line `q` takes ───────────────────

#[test]
fn r_renders_what_render_renders() {
    // EX-2: `r` is `render` with the words `q` takes, so each row is one pipeline
    // with two front doors — compared byte for byte — and each document is then
    // asserted to carry a heading from the fixture, so two refusals or two empty
    // documents cannot pass the equality above.
    let scratch = Scratch::new();
    store_of_a_session_and_two_questions(&scratch);

    let projected = same_streams(
        smut(&scratch, &["r", "session", "--select", "title,body"]),
        smut(
            &scratch,
            &[
                "render",
                r#"{"from":["session"],"select":["title","body"]}"#,
            ],
        ),
        "a single root with a projection",
    );
    assert_eq!(projected.code, 0, "{}", projected.stderr);
    assert!(
        projected.stdout.contains("## Session"),
        "the projection still renders the title as its heading: {:?}",
        projected.stdout
    );

    // Bare roots are this surface's spelling of a root list, so the twin is
    // `from` as an array; a filter token is a `where` condition.
    let filtered = same_streams(
        smut(&scratch, &["r", "status=resolved"]),
        smut(
            &scratch,
            &["render", r#"{"where":{"status":{"$eq":"resolved"}}}"#],
        ),
        "a filter line",
    );
    assert_eq!(filtered.code, 0, "{}", filtered.stderr);
    assert!(
        filtered.stdout.contains("## A graph view"),
        "the filter selects the resolved question and nothing else: {:?}",
        filtered.stdout
    );

    // `EXAMPLE_TRAVERSE` carries `"select":["smut_id"]`, so its line needs
    // `--ids`; this row asks for the whole entities instead, so the headings are
    // visible and the equality is checkable by eye as well as by bytes.
    let traversed = same_streams(
        smut(
            &scratch,
            &["r", "session", "--backlinks", "part-of", "--depth", "2"],
        ),
        smut(
            &scratch,
            &[
                "render",
                r#"{"from":["session"],"backlinks":["part-of"],"depth":2}"#,
            ],
        ),
        "a traversal line",
    );
    assert_eq!(traversed.code, 0, "{}", traversed.stderr);
    for heading in ["## Session", "## Query shape", "## A graph view"] {
        assert!(
            traversed.stdout.contains(heading),
            "the walk reaches {heading}: {:?}",
            traversed.stdout
        );
    }

    for (id, title) in [("-weird", "Weird"), ("--follow", "Follow")] {
        let declared = smut(
            &scratch,
            &[
                "declare",
                &format!(r#"{{"smut_id":"{id}","title":"{title}"}}"#),
            ],
        );
        assert_eq!(declared.code, 0, "{id}: {}", declared.stderr);
    }

    let hyphen_led = same_streams(
        smut(&scratch, &["r", "--", "-weird"]),
        smut(&scratch, &["render", r#"{"from":["-weird"]}"#]),
        "a hyphen-led root behind `--`",
    );
    assert_eq!(hyphen_led.code, 0, "{}", hyphen_led.stderr);
    assert!(
        hyphen_led.stdout.contains("## Weird"),
        "`--` reaches the id the flag grammar cannot: {:?}",
        hyphen_led.stdout
    );

    let two_roots = same_streams(
        smut(&scratch, &["r", "session", "--", "--follow"]),
        smut(&scratch, &["render", r#"{"from":["session","--follow"]}"#]),
        "a hyphen-led root after a bare one",
    );
    assert_eq!(two_roots.code, 0, "{}", two_roots.stderr);
    assert!(
        two_roots.stdout.contains("## Follow"),
        "the escaped root is selected, not swallowed as a flag: {:?}",
        two_roots.stdout
    );
    assert!(
        two_roots.stdout.contains("## Session"),
        "{:?}",
        two_roots.stdout
    );
}

#[test]
fn r_dash_j_renders_what_render_renders() {
    // EX-2: `-j` carries the whole specification, so `r -j` is `render` in every
    // respect — the same document, byte for byte, from the operand and from stdin.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let specification = r#"{"from":["a"]}"#;

    let operand = same_streams(
        smut(&scratch, &["r", "-j", specification]),
        smut(&scratch, &["render", specification]),
        "-j carries render's operand",
    );
    assert_eq!(operand.code, 0, "{}", operand.stderr);
    assert!(
        operand.stdout.contains("## Alpha"),
        "a non-empty document, so the equality above is not two empty ones: {:?}",
        operand.stdout
    );

    let piped = same_streams(
        run(&scratch, &["r", "-j", "-"], Some(specification)),
        run(&scratch, &["render", "-"], Some(specification)),
        "one specification on stdin reaches render either way",
    );
    assert_eq!(piped.code, 0, "{}", piped.stderr);
    assert_eq!(
        piped.stdout, operand.stdout,
        "the operand and stdin carry the same specification"
    );
}

#[test]
fn display_flags_agree_across_render_and_r() {
    // EX-2: the display flags say how to render and not what, so they mean the
    // same thing on both verbs and on either spelling of the specification.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let flags = ["-s", "-i", "-l", "3", "-t", "title"];

    let mut from_line = vec!["r", "a"];
    let mut from_json = vec!["render", r#"{"from":["a"]}"#];
    let mut from_dash_j = vec!["r", "-j", r#"{"from":["a"]}"#];
    for flag in flags {
        from_line.push(flag);
        from_json.push(flag);
        from_dash_j.push(flag);
    }

    let beside_the_line = same_streams(
        smut(&scratch, &from_line),
        smut(&scratch, &from_json),
        "the flags beside the line",
    );
    assert_eq!(beside_the_line.code, 0, "{}", beside_the_line.stderr);
    assert!(
        beside_the_line.stdout.contains("### Alpha"),
        "the level reached the renderer: {:?}",
        beside_the_line.stdout
    );
    assert!(
        beside_the_line.stdout.contains("`a`"),
        "--show-ids reached the renderer: {:?}",
        beside_the_line.stdout
    );

    let beside_dash_j = same_streams(
        smut(&scratch, &from_dash_j),
        smut(&scratch, &from_json),
        "the flags beside -j",
    );
    assert_eq!(beside_dash_j.code, 0, "{}", beside_dash_j.stderr);
    assert!(
        beside_dash_j.stdout.contains("### Alpha"),
        "the level reached the renderer through -j too: {:?}",
        beside_dash_j.stdout
    );
    assert_eq!(
        beside_dash_j.stdout, beside_the_line.stdout,
        "one specification, one document, whichever door it came through"
    );
}

#[test]
fn an_empty_r_line_renders_the_empty_specification() {
    // EX-2: `r` with no words and no `-j` is `{}` and never reads stdin. The
    // fixture specification on stdin selects one entity, so a stdin-reading `r`
    // would render one title and not all three.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = same_streams(
        run(&scratch, &["r"], Some(r#"{"from":["a"]}"#)),
        smut(&scratch, &["render", "{}"]),
        "an empty line is the empty specification",
    );
    assert_eq!(ran.code, 0, "{}", ran.stderr);
    for title in ["Alpha", "Beta", "Gamma"] {
        assert!(
            ran.stdout.contains(&format!("## {title}")),
            "an empty line renders the whole store, so stdin went unread: {:?}",
            ran.stdout
        );
    }
}

#[test]
fn json_beside_a_selection_is_refused_on_r() {
    // EX-3: `-j` is the whole specification on `r` exactly as it is on `q`, so
    // any token or selection flag beside it is the same one-sentence conflict.
    let scratch = Scratch::new();

    for arguments in [
        &["r", "a", "-j", "{}"][..],
        &["r", "--ids", "-j", "{}"][..],
        &["r", "--depth", "2", "-j", "{}"][..],
    ] {
        let refused = smut(&scratch, arguments);

        assert_eq!(refused.code, 2, "{arguments:?}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{arguments:?} prints no document");
        assert_eq!(
            refused.stderr,
            "smut: -j takes the whole specification; drop the tokens and selection flags beside it\n",
            "{arguments:?}"
        );
        assert!(!scratch.has_store(), "{arguments:?} creates nothing");
    }

    // Paired in the same directory, so the assertions above discriminate: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"A"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(scratch.has_store(), "declare creates the store r did not");

    // Display flags are legal beside `-j`: they say how to render, not what.
    let shown = smut(&scratch, &["r", "-j", "{}", "-s"]);

    assert_eq!(shown.code, 0, "{}", shown.stderr);
    assert!(!shown.stdout.is_empty(), "a declaration renders a document");
}

#[test]
fn every_q_refusal_reaches_r_unchanged() {
    // EX-3: `r` compiles the same line with the same compiler, so every refusal
    // the line earns arrives whole and at exit 2, byte for byte with `q`'s.
    let scratch = Scratch::new();

    for line in [
        &["--ids", "--select", "a"][..],
        &["--depth", "2"][..],
        &["--limit", "all"][..],
        &["--from="][..],
        &["--order", "title:user"][..],
        &["status~a=b"][..],
    ] {
        let mut from_q = vec!["q"];
        from_q.extend_from_slice(line);
        let mut from_r = vec!["r"];
        from_r.extend_from_slice(line);

        let refused = same_streams(
            smut(&scratch, &from_q),
            smut(&scratch, &from_r),
            line.join(" ").as_str(),
        );

        assert_eq!(refused.code, 2, "{line:?}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{line:?} prints no document");
        assert!(
            !refused.stderr.is_empty(),
            "{line:?} says why, so two silent refusals prove nothing"
        );
        assert!(!scratch.has_store(), "{line:?} creates nothing");
    }

    // Paired in the same directory, so the assertions above discriminate: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"A"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(scratch.has_store(), "declare creates the store q did not");
}

#[test]
fn r_refuses_without_creating_a_store() {
    // EX-3, EX-4: the display flags refuse before the selection and before any
    // store, with render's own sentences, because they are render's own flags.
    let scratch = Scratch::new();

    for (from_r, from_render, description) in [
        (
            &["r", "--format", "mermaid"][..],
            &["render", "{}", "--format", "mermaid"][..],
            "a format this release does not write",
        ),
        (
            &["r", "--format", "bogus"][..],
            &["render", "{}", "--format", "bogus"][..],
            "a format the release has never heard of",
        ),
        (
            &["r", "-l", "7"][..],
            &["render", "{}", "-l", "7"][..],
            "a heading level outside 1 to 6",
        ),
    ] {
        let refused = same_streams(
            smut(&scratch, from_r),
            smut(&scratch, from_render),
            description,
        );

        assert_eq!(refused.code, 2, "{description}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{description} prints no document");
        assert!(!scratch.has_store(), "{description} creates nothing");
    }

    // `-l` takes a hyphen-leading value, so `--ids` and `-j` are heading levels
    // here and are refused as ones — the same sentence `render` prints for them.
    // `r -l -j 3` leaves `3` as a token, which the refusal never reaches.
    for (found, from_r) in [
        ("--ids", &["r", "-l", "--ids"][..]),
        ("-j", &["r", "-l", "-j", "3"][..]),
    ] {
        let refused = same_streams(
            smut(&scratch, from_r),
            smut(&scratch, &["render", "{}", "-l", found]),
            found,
        );

        assert_eq!(refused.code, 2, "{found}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{found} prints no document");
        assert_eq!(
            refused.stderr,
            format!(
                "smut: --heading-level takes a whole number from 1 to 6, and {found} is not one\n"
            ),
            "{found} is read as the level's value"
        );
        assert!(!scratch.has_store(), "{found} creates nothing");
    }

    // Display flags are validated before the selection, so the level is refused
    // and not the `--ids`/`--select` conflict beside it.
    let before_the_selection = same_streams(
        smut(&scratch, &["r", "-l", "7", "--ids", "--select", "a"]),
        smut(&scratch, &["render", "{}", "-l", "7"]),
        "the display flags answer first",
    );

    assert_eq!(
        before_the_selection.code, 2,
        "{}",
        before_the_selection.stderr
    );
    assert!(
        !before_the_selection.stderr.contains("--select"),
        "the selection was never reached: {}",
        before_the_selection.stderr
    );
    assert!(!scratch.has_store(), "-l 7 creates nothing");

    // A word beginning with `{` is `q`'s `JsonOperand` refusal, and `r` compiles
    // the line with the same compiler, so it earns the same sentence.
    for word in [r#"{"from":"a"}"#, "{weird}"] {
        let refused = same_streams(
            smut(&scratch, &["r", word]),
            smut(&scratch, &["q", word]),
            word,
        );

        assert_eq!(refused.code, 2, "{word}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{word} prints no document");
        assert!(
            refused
                .stderr
                .contains("a JSON specification goes after -j, and an id after --from="),
            "{word}: {}",
            refused.stderr
        );
        assert!(!scratch.has_store(), "{word} creates nothing");
    }

    // Paired in the same directory, so the assertions above discriminate: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"A"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(scratch.has_store(), "declare creates the store r did not");
}

#[test]
fn r_reaches_ids_the_short_form_cannot_carry_bare() {
    // EX-3, EX-4: `--from=` is the root escape, and `--` is the rescue for an id
    // that begins with a hyphen. Between them, `r` reaches every id.
    let scratch = Scratch::new();

    for (id, title) in [("{weird}", "Curly"), ("-title", "Hyphenated")] {
        let declared = smut(
            &scratch,
            &[
                "declare",
                &format!(r#"{{"smut_id":"{id}","title":"{title}"}}"#),
            ],
        );
        assert_eq!(declared.code, 0, "{id}: {}", declared.stderr);
    }

    let escaped = smut(&scratch, &["r", "--from={weird}"]);

    assert_eq!(escaped.code, 0, "{}", escaped.stderr);
    assert!(
        escaped.stdout.contains("## Curly"),
        "--from= carries the brace-led id: {:?}",
        escaped.stdout
    );

    // Characterisation (plan C6): `r -title` with no `--` is `-t itle` — the
    // heading field `itle`, and no tokens at all — because DEC-008 and IMP-007
    // keep the letters as they were typed. The rescue for a hyphen-led id is
    // `--`, below. No entity carries `itle`, so the document falls back to the
    // identity block and proves the field, not the title, was used.
    let letters_win = same_streams(
        smut(&scratch, &["r", "-title"]),
        smut(&scratch, &["render", "-title", "{}"]),
        "-title is -t itle",
    );

    assert_eq!(letters_win.code, 0, "{}", letters_win.stderr);
    assert!(
        letters_win.stdout.contains("`-title`"),
        "a heading field no entity carries renders identity instead: {:?}",
        letters_win.stdout
    );
    assert!(
        !letters_win.stdout.contains("## Hyphenated"),
        "the title is not the heading here: {:?}",
        letters_win.stdout
    );

    let rescued = smut(&scratch, &["r", "--", "-title"]);

    assert_eq!(rescued.code, 0, "{}", rescued.stderr);
    assert!(
        rescued.stdout.contains("## Hyphenated"),
        "`--` reaches the hyphen-led id: {:?}",
        rescued.stdout
    );
}

#[test]
fn r_and_render_differ_only_where_the_design_says() {
    // EX-5, VT-3: the two verbs differ on the empty line and on `[1]`, and on
    // nothing else that a spawned process can see.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    // A specification on stdin is ignored by an empty `r` line and read by a bare
    // `render`: `r` renders `{}`, `render` renders the piped selection.
    let piped = same_streams(
        run(&scratch, &["r"], Some(r#"{"from":["a"]}"#)),
        smut(&scratch, &["render", "{}"]),
        "r never reads stdin",
    );

    assert_eq!(piped.code, 0, "{}", piped.stderr);
    for title in ["Alpha", "Beta", "Gamma"] {
        assert!(
            piped.stdout.contains(&format!("## {title}")),
            "`r` rendered the empty specification and not the piped one: {:?}",
            piped.stdout
        );
    }

    // `[1]` is a root id to the short form and not a specification to `render`,
    // whose operand is JSON and must parse.
    let declared = smut(
        &scratch,
        &["declare", r#"{"smut_id":"[1]","title":"Bracketed"}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let short = smut(&scratch, &["r", "[1]"]);

    assert_eq!(short.code, 0, "{}", short.stderr);
    assert!(
        short.stdout.contains("## Bracketed"),
        "[1] is one root id in the short form: {:?}",
        short.stdout
    );

    let json = smut(&scratch, &["render", "[1]"]);

    assert_eq!(json.code, 2, "{}", json.stderr);
    assert_eq!(json.stdout, "", "the refusal prints no document");

    // The terminal half of the same divergence — `r` on a terminal is `{}` where a
    // bare `render` there is MissingInput — cannot be reached from here, because
    // `run` always pipes stdin. It lives in `src/cli.rs`'s
    // `r_with_no_words_on_a_terminal_is_the_empty_specification`.
}

#[test]
fn query_and_render_do_not_take_dash_j() {
    // EX-7's non-goal, asserted: `-j` belongs to the verb that compiles a line,
    // and the two verbs that take one specification take it as the operand.
    let scratch = Scratch::new();

    for verb in ["query", "render"] {
        let refused = smut(&scratch, &[verb, "-j", "{}"]);

        assert_eq!(refused.code, 2, "{verb}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{verb} prints nothing");
        assert!(
            refused
                .stderr
                .lines()
                .any(|line| line == "error: unexpected argument '-j' found"),
            "{verb} never took -j: {:?}",
            refused.stderr
        );
        assert!(!scratch.has_store(), "{verb} creates nothing");
    }

    // Paired in the same directory, so the assertions above discriminate: a bare
    // "no `.smut` here" check would pass for a command that never ran.
    let declared = smut(&scratch, &["declare", r#"{"smut_id":"a","title":"A"}"#]);

    assert_eq!(declared.code, 0, "{}", declared.stderr);
    assert!(
        scratch.has_store(),
        "declare creates the store neither verb did"
    );
}
