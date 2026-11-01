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
            r#"{"id":"a","title":"Alpha","body":"First.","priority":3}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    for (id, title) in [("b", "Beta"), ("c", "Gamma")] {
        let declared = smut(
            scratch,
            &["declare", &format!(r#"{{"id":"{id}","title":"{title}"}}"#)],
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
            r#"{"id":"smut","title":"Smut","questions":[{"id":"q-query","type":"question"}]}"#,
        ],
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(ran.stderr.is_empty(), "{}", ran.stderr);

    let envelope = ran.json();
    assert_eq!(envelope["ids"], json!(["smut"]));
    assert_eq!(envelope["changes"][0]["id"], json!("smut"));
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
    // projected entities alike", so a response's entity roots name `id` first
    // and their remaining keys lexically — the same bytes rule as the file.
    let scratch = Scratch::new();
    store_of_three(&scratch);

    let ran = smut(&scratch, &["get", "a"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert!(
        ran.stdout
            .contains(r#""objects":[{"id":"a","body":"First.","priority":3,"title":"Alpha"}]"#),
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
        assert_eq!(envelope["objects"][0]["id"], json!("b"), "{arguments:?}");
        assert_eq!(envelope["missing"], json!([]), "{arguments:?}");
    }
}

#[test]
fn declare_reads_its_declaration_from_a_pipe_on_stdin_and_exits_zero() {
    let scratch = Scratch::new();

    let ran = run(
        &scratch,
        &["declare"],
        Some(r#"{"id":"piped","title":"Piped"}"#),
    );

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.json()["ids"], json!(["piped"]));
    // An operand present means stdin is never read, and `-` asks for it
    // explicitly even where a terminal is attached.
    let explicit = run(&scratch, &["declare", "-"], Some(r#"{"id":"dashed"}"#));
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

    let ran = smut(&scratch, &["declare", r#"{"id":"ghost"}"#]);

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

    let ran = smut(&scratch, &["declare", r#"{"id":"a"} {"id":"b"}"#]);

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
        &["declare", r#"{"id":"a","title":"one","title":"two"}"#],
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
        &["declare", r#"{"id":"-weird-id","title":"Weird"}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let ran = smut(&scratch, &["get", "--", "-weird-id"]);

    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.json()["objects"][0]["id"], json!("-weird-id"));
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
    assert!(
        ran.stderr.contains("not supported in 0.1"),
        "{}",
        ran.stderr
    );
    assert!(ran.stderr.contains("1.0"), "{}", ran.stderr);
    assert!(!scratch.has_store(), "the refusal needs no store");
}

#[test]
fn compact_omits_previous_everywhere_and_changes_nothing_else() {
    let scratch = Scratch::new();

    let created = smut(&scratch, &["declare", r#"{"id":"a","body":"first"}"#]);
    assert_eq!(created.code, 0, "{}", created.stderr);

    let full = smut(&scratch, &["declare", r#"{"id":"a","body":"second"}"#]);
    let full = full.json();
    assert_eq!(full["changes"][0]["status"], json!("updated"));
    assert_eq!(full["changes"][0]["previous"]["body"], json!("first"));

    let compact = smut(
        &scratch,
        &["declare", r#"{"id":"a","body":"third"}"#, "--compact"],
    );
    let compact = compact.json();
    assert_eq!(compact["ids"], json!(["a"]));
    assert_eq!(compact["changes"][0]["status"], json!("updated"));
    assert_eq!(compact["changes"][0]["id"], json!("a"));
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
    assert_eq!(envelope["objects"][0]["id"], json!("a"));
    assert_eq!(envelope["truncated"], json!(false));
}

#[test]
fn render_emits_text_on_stdout_with_exactly_one_trailing_newline() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &["declare", r#"{"id":"a","title":"Alpha","body":"First."}"#],
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
        &["declare", r#"{"id":"a","name":"Alpha","status":"open"}"#],
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
    assert_eq!(version.stdout, "smut 0.1.1\n");

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
        json!([{"body": "First.", "id": "a", "priority": 3, "title": "Alpha"}]),
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
    assert_eq!(remaining.json()["objects"][0]["id"], json!("b"));
}

#[test]
fn delete_leaves_incoming_references_untouched() {
    let scratch = Scratch::new();
    let declared = smut(
        &scratch,
        &[
            "declare",
            r#"{"id":"parent","title":"Parent","children":[{"id":"child","title":"Child"}]}"#,
        ],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);

    let before = smut(&scratch, &["get", "parent"]);
    assert_eq!(
        before.json()["objects"][0]["children"],
        json!([{"id": "child"}])
    );

    let deleted = smut(&scratch, &["delete", "child"]);
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    assert_eq!(deleted.json(), json!({"deleted": ["child"], "missing": []}));

    let parent = smut(&scratch, &["get", "parent"]);
    assert_eq!(parent.code, 0, "{}", parent.stderr);
    assert_eq!(
        parent.json()["objects"][0]["children"],
        json!([{"id": "child"}]),
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
        json!([{"body": "First.", "id": "a", "priority": 3, "title": "Alpha"}]),
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
    // a refusal that only says "not supported in 0.1" would satisfy the rest of
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
        &["declare", r#"{"id":"a","title":"A","next":{"id":"b"}}"#],
    );
    assert_eq!(declared.code, 0, "{}", declared.stderr);
    let declared = smut(&scratch, &["declare", r#"{"id":"b","title":"B"}"#]);
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
