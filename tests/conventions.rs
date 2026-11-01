//! The conventions command, from outside the process (§9.1–§9.3, §13.3
//! scenario 5).
//!
//! `smut conventions` is pure text: one shipped set, printed as a listing or as
//! prose, plus a pipeable declaration of the same embedded prose. The tests
//! below pin the surface a caller can observe — the listing's one line, the
//! prose's eight items, the refusals' exit code and words, the seed's fixed id
//! and the crate's version — and the two things the command must *not* do: open
//! a store, and lose an amendment silently on re-install.
//!
//! The body is asserted against the file the binary embeds, so a second copy of
//! the prose cannot drift from the one that ships (VA-1).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

/// The shipped set, as this tree holds it. The seed's `body` is this text, and
/// the prose `smut conventions hygiene` prints is this text: one copy, printed
/// twice.
const EMBEDDED: &str = include_str!("../src/conventions/hygiene.md");

/// A scratch working directory holding at most one store, removed when dropped.
///
/// Copied from `tests/cli_contract.rs` by the plan's decision: 0.2 does not add
/// a shared `tests/common`.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-conventions-{}-{}",
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

/// The seed, as the binary prints it: the declaration a caller pipes on.
fn seed(scratch: &Scratch) -> String {
    let printed = smut(scratch, &["conventions", "hygiene", "--format", "json"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);

    printed.stdout
}

// ── VT-1: listing, prose, refusals, the seed, and no store ───────────────────

#[test]
fn the_bare_command_lists_exactly_one_set_on_one_line() {
    let scratch = Scratch::new();

    let listed = smut(&scratch, &["conventions"]);

    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(listed.stderr, "", "a listing says nothing on stderr");
    assert!(
        listed.stdout.ends_with('\n'),
        "the shell writes one newline: {:?}",
        listed.stdout
    );
    assert_eq!(
        listed.stdout.lines().count(),
        1,
        "one set is one line: {:?}",
        listed.stdout
    );
    assert!(
        listed.stdout.starts_with("hygiene"),
        "the line names the set: {:?}",
        listed.stdout
    );
    assert!(
        listed.stdout.trim_end().len() > "hygiene".len(),
        "the line says what the set is about: {:?}",
        listed.stdout
    );
    assert!(
        !listed.stdout.contains("epistemic-records"),
        "the deferred set is not listed as shipped: {:?}",
        listed.stdout
    );
}

#[test]
fn a_named_set_prints_the_embedded_prose_verbatim() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene"]);

    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert_eq!(
        printed.stdout, EMBEDDED,
        "the prose is the embedded file, byte for byte"
    );
}

#[test]
fn an_unknown_set_is_refused_with_exit_two_naming_the_sets() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["conventions", "nonsense"]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "a refusal prints no success text");
    assert!(
        refused.stderr.contains("nonsense"),
        "the refusal names what was asked for: {:?}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("hygiene"),
        "the refusal names the sets that exist: {:?}",
        refused.stderr
    );
}

#[test]
fn the_deferred_epistemic_records_set_is_refused_rather_than_half_shipped() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["conventions", "epistemic-records"]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "");
    assert!(
        refused.stderr.contains("epistemic-records"),
        "the refusal names the deferred set: {:?}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("hygiene"),
        "and names the set that does exist: {:?}",
        refused.stderr
    );
}

#[test]
fn format_text_is_identical_to_no_flag_at_all() {
    let scratch = Scratch::new();

    let cases: [(&[&str], &[&str]); 2] = [
        (&["conventions"], &["conventions", "--format", "text"]),
        (
            &["conventions", "hygiene"],
            &["conventions", "hygiene", "--format", "text"],
        ),
    ];

    for (plain, spelled) in cases {
        let without = smut(&scratch, plain);
        let with = smut(&scratch, spelled);

        assert_eq!(without.code, with.code, "{}", with.stderr);
        assert_eq!(without.stdout, with.stdout);
        assert_eq!(without.stderr, with.stderr);
        assert_eq!(with.code, 0, "{}", with.stderr);
    }
}

#[test]
fn json_without_a_name_is_refused_because_a_listing_is_not_a_declaration() {
    let scratch = Scratch::new();

    let refused = smut(&scratch, &["conventions", "--format", "json"]);

    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "no seed is printed for a listing");
    assert!(
        refused.stderr.contains("json"),
        "the refusal names the format that needs a set name: {:?}",
        refused.stderr
    );
    assert!(
        !refused.stderr.contains("unknown command"),
        "and it is not the unknown-command refusal: {:?}",
        refused.stderr
    );
}

#[test]
fn any_other_format_is_a_usage_error() {
    let scratch = Scratch::new();

    for format in ["dot", "yaml", "JSON", "markdown"] {
        let refused = smut(&scratch, &["conventions", "hygiene", "--format", format]);

        assert_eq!(refused.code, 2, "{format}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{format} prints nothing");
        assert!(
            refused.stderr.contains(format),
            "the refusal names the format it refused: {:?}",
            refused.stderr
        );
    }
}

#[test]
fn the_seed_is_one_declaration_with_a_fixed_id_and_the_crates_version() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene", "--format", "json"]);

    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert!(
        printed.stdout.ends_with('\n'),
        "one line, ending in one newline"
    );
    assert_eq!(
        printed.stdout.lines().count(),
        1,
        "the seed is one declaration on one line: {:?}",
        printed.stdout
    );

    let seed = printed.json();

    assert_eq!(seed["id"], json!("conventions-hygiene"));
    assert_eq!(seed["type"], json!("convention"));
    assert_eq!(
        seed["version"],
        json!(env!("CARGO_PKG_VERSION")),
        "the seed's version is the crate's, never a pinned release"
    );
    assert_eq!(seed["title"], json!("Conventions: general hygiene"));
    assert_eq!(
        seed["body"],
        json!(EMBEDDED.trim_end_matches('\n')),
        "the body is the embedded prose, not an inline copy of it"
    );
    assert_eq!(
        seed.as_object().expect("one object").len(),
        5,
        "the seed supplies exactly id, type, version, title and body: {seed}"
    );
}

#[test]
fn the_command_creates_no_store_and_does_not_read_one() {
    let scratch = Scratch::new();

    for arguments in [
        vec!["conventions"],
        vec!["conventions", "hygiene"],
        vec!["conventions", "hygiene", "--format", "json"],
        vec!["conventions", "hygiene", "--format", "text"],
    ] {
        let ran = smut(&scratch, &arguments);

        assert_eq!(ran.code, 0, "{arguments:?}: {}", ran.stderr);
        assert!(
            !scratch.store().exists(),
            "{arguments:?} created a store: {}",
            scratch.store().display()
        );
    }

    // A path where a store would be, holding something no snapshot can read.
    // Every command that opened it would exit 1; conventions answers anyway.
    fs::write(scratch.store(), b"not a store\n").expect("a decoy path at .smut");

    let printed = smut(&scratch, &["conventions", "hygiene"]);
    assert_eq!(
        printed.code, 0,
        "the command never opens a store: {}",
        printed.stderr
    );
    assert_eq!(printed.stdout, EMBEDDED);
}

// ── EX-5: the prose ships the eight items and the two caveats ────────────────

#[test]
fn the_prose_carries_the_eight_items_including_retracted_versus_superseded() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);

    let numbered: Vec<&str> = printed
        .stdout
        .lines()
        .filter(|line| {
            let mut words = line.splitn(2, ". ");
            matches!(words.next(), Some(number) if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
                && words.next().is_some()
        })
        .collect();

    assert_eq!(
        numbered.len(),
        8,
        "§9.2 ships eight items, one dropped or one added is a different set: {numbered:?}"
    );
    for (index, item) in numbered.iter().enumerate() {
        assert!(
            item.starts_with(&format!("{}. ", index + 1)),
            "the items are numbered in order: {item:?}"
        );
    }

    // Item 7 is the one the release is built on: a withdrawn record is not a
    // replaced one, the edge is what makes it replaced, and the store's own
    // `vocab` is the evidence that the distinction was needed.
    for token in ["retracted", "superseded", "supersedes", "vocab"] {
        assert!(
            printed.stdout.contains(token),
            "item 7 does not name {token}: {:?}",
            printed.stdout
        );
    }
    assert!(
        printed.stdout.contains("withdrawn") && printed.stdout.contains("replaced"),
        "item 7 distinguishes a withdrawn record from a replaced one"
    );
}

#[test]
fn the_prose_says_the_seeded_body_is_advice_and_points_at_the_guide() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);

    let prose = &printed.stdout;
    let lowercased = prose.to_lowercase();

    assert!(
        lowercased.contains("re-install"),
        "the printed set says a re-install replaces the seeded body: {prose:?}"
    );
    assert!(
        prose.contains("body"),
        "and names the field it replaces: {prose:?}"
    );
    assert!(
        prose.contains("smut learn"),
        "the printed set points at the guide for the recipe: {prose:?}"
    );
    assert!(
        !prose.contains('{'),
        "the set states principles; it does not repeat the recipe's query JSON: {prose:?}"
    );
}

// ── VT-2: the seed is an upsert, and an amendment is reported, not lost ─────

#[test]
fn the_seed_declares_once_and_a_second_install_updates_the_same_entity() {
    let scratch = Scratch::new();
    let seed = seed(&scratch);

    let first = run(&scratch, &["declare"], Some(&seed));
    assert_eq!(first.code, 0, "{}", first.stderr);
    assert_eq!(first.json()["ids"], json!(["conventions-hygiene"]));
    assert_eq!(first.json()["changes"][0]["status"], json!("created"));

    let second = run(&scratch, &["declare"], Some(&seed));
    assert_eq!(second.code, 0, "{}", second.stderr);
    assert_eq!(
        second.json()["changes"][0]["status"],
        json!("unchanged"),
        "re-installing the same seed changes nothing and adds nothing: {}",
        second.stdout
    );

    let fetched = smut(&scratch, &["query", r#"{"from":"conventions-hygiene"}"#]);
    assert_eq!(fetched.code, 0, "{}", fetched.stderr);
    assert_eq!(
        fetched.json()["objects"].as_array().expect("objects").len(),
        1,
        "the fixed id means one entity, however many times the pipe runs"
    );
}

#[test]
fn a_re_install_replaces_an_amended_body_keeps_a_user_field_and_reports_the_previous() {
    let scratch = Scratch::new();
    let seed = seed(&scratch);

    let installed = run(&scratch, &["declare"], Some(&seed));
    assert_eq!(installed.code, 0, "{}", installed.stderr);

    // The operator amends the stored copy in place and adds a field the seed
    // has never heard of.
    let amended = smut(
        &scratch,
        &[
            "declare",
            r#"{"id":"conventions-hygiene","body":"Our house rule: one label per relation.","owner":"ops"}"#,
        ],
    );
    assert_eq!(amended.code, 0, "{}", amended.stderr);
    assert_eq!(amended.json()["changes"][0]["status"], json!("updated"));

    // The pipe runs again: every seeded field is replaced — `body` included —
    // and the user's own field survives (0.1's patch semantics, §9.3).
    let reinstalled = run(&scratch, &["declare"], Some(&seed));
    assert_eq!(reinstalled.code, 0, "{}", reinstalled.stderr);

    let reply = reinstalled.json();
    let change = &reply["changes"][0];
    assert_eq!(change["status"], json!("updated"));
    assert_eq!(
        change["previous"]["body"],
        json!("Our house rule: one label per relation."),
        "the reply carries the body it replaced, so the overwrite is reported: {}",
        reinstalled.stdout
    );
    assert_eq!(
        change["previous"]["owner"],
        json!("ops"),
        "previous is the whole entity as it stood before the batch"
    );

    let fetched = smut(&scratch, &["get", "conventions-hygiene"]);
    assert_eq!(fetched.code, 0, "{}", fetched.stderr);
    let stored = &fetched.json()["objects"][0];

    assert_eq!(
        stored["body"],
        json!(EMBEDDED.trim_end_matches('\n')),
        "the seeded body replaced the amended one"
    );
    assert_eq!(
        stored["owner"],
        json!("ops"),
        "a field the seed does not supply is left alone"
    );
}

#[test]
fn compact_omits_the_previous_body_the_re_install_replaced() {
    let scratch = Scratch::new();
    let seed = seed(&scratch);

    let installed = run(&scratch, &["declare"], Some(&seed));
    assert_eq!(installed.code, 0, "{}", installed.stderr);
    let amended = smut(
        &scratch,
        &["declare", r#"{"id":"conventions-hygiene","body":"Draft."}"#],
    );
    assert_eq!(amended.code, 0, "{}", amended.stderr);

    let reinstalled = run(&scratch, &["declare", "--compact"], Some(&seed));
    assert_eq!(reinstalled.code, 0, "{}", reinstalled.stderr);

    let reply = reinstalled.json();
    let change = &reply["changes"][0];

    assert_eq!(change["status"], json!("updated"));
    assert!(
        change.get("previous").is_none(),
        "--compact omits `previous` everywhere, so the replaced body is not reported: {}",
        reinstalled.stdout
    );
    assert!(
        !reinstalled.stdout.contains("Draft."),
        "the replaced body does not appear anywhere under --compact: {}",
        reinstalled.stdout
    );
}
