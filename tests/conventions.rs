//! The conventions command, from outside the process (§9.1–§9.3, §13.3
//! scenario 5).
//!
//! `smut conventions` is pure text: one shipped set, printed as a listing or as
//! the body below its frontmatter, plus a pipeable declaration of that same
//! body. The tests below pin the surface a caller can observe — the listing's
//! one line, the prose's eight items, the refusals' exit code and words, the
//! seed's fixed public id carried under its structural `smut_id`, its title,
//! which is the file's own heading, and the crate's version — and the two things
//! the command must *not* do: open a store, and lose an amendment silently on
//! re-install.
//!
//! The last section answers where a set comes from: the project's
//! `.config/smut`, then the config home, then the binary, with the first layer
//! holding a name the set in force. Those rows spawn the binary in a working
//! directory of the fixture's choosing under a `$HOME` and `$XDG_CONFIG_HOME`
//! the scratch makes, so none of them depends on the machine's own sets.
//!
//! The body is derived here from the bytes of the file the binary embeds, so a
//! second copy of the prose cannot drift from the one that ships (VA-1), and the
//! frontmatter the ship carries is metadata only: nothing the command prints may
//! reproduce it.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
use common::{Ran, Scratch, run, smut};

/// The shipped set's file, as this tree holds it: its frontmatter and its body.
///
/// The file is the one copy: [`body`] and [`heading`] derive what the command
/// prints and declares from these bytes, so the prose cannot drift from the
/// file that ships (VA-1).
const EMBEDDED: &str = include_str!("../src/conventions/hygiene.md");

/// The shipped file's body: everything below the closing frontmatter fence,
/// trailing newlines trimmed and nothing else transformed.
///
/// The derivation is the test's own, an independent oracle rather than a call
/// into the library: a reader that printed the whole file, or that left the
/// fence on line 1, disagrees with this and reddens on the assertions below. A
/// file with no frontmatter would have the whole of itself as its body; this
/// fixture opens one, and the two assertions here are what redden if it stops.
fn body() -> String {
    let lines: Vec<&str> = EMBEDDED.split('\n').collect();

    assert_eq!(
        lines.first(),
        Some(&"---"),
        "the shipped fixture opens with a frontmatter fence, so a body sits below one"
    );

    let closing = lines
        .iter()
        .skip(1)
        .position(|line| *line == "---")
        .map(|index| index + 1)
        .expect("the shipped fixture's frontmatter is closed by a second `---` line");

    lines[closing + 1..]
        .join("\n")
        .trim_end_matches('\n')
        .to_owned()
}

/// The first `# ` heading of the [`body`], prefix stripped: the seed's title.
fn heading() -> String {
    body()
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .expect("the shipped body carries a `# ` heading")
        .to_owned()
}

/// The `summary:` value the shipped frontmatter declares, trimmed: the one
/// thing the listing line says about the set.
fn summary() -> String {
    EMBEDDED
        .split('\n')
        .skip(1)
        .take_while(|line| *line != "---")
        .find_map(|line| line.strip_prefix("summary:"))
        .expect("the shipped frontmatter declares a summary")
        .trim()
        .to_owned()
}

/// This file's layer fixtures. The scratch itself, its `$HOME` and
/// `$XDG_CONFIG_HOME`, and the one spawn path are `tests/common`'s.
impl Scratch {
    /// A directory under the scratch root, created.
    fn dir(&self, name: &str) -> PathBuf {
        let path = self.path().join(name);
        fs::create_dir_all(&path).expect("a fixture directory");

        path
    }

    /// A directory under the scratch root holding `.git` as a **directory**:
    /// an ordinary clone.
    fn repository(&self, name: &str) -> PathBuf {
        let path = self.dir(name);
        fs::create_dir_all(path.join(".git")).expect("a repository's .git directory");

        path
    }

    /// A directory under the scratch root holding `.git` as a **regular file**
    /// carrying a `gitdir:` line: a worktree or submodule checkout.
    fn worktree(&self, name: &str) -> PathBuf {
        let path = self.dir(name);
        fs::write(path.join(".git"), "gitdir: ../.git/worktrees/fixture\n")
            .expect("a worktree's .git file");

        path
    }

    /// A directory nested inside `parent`, created.
    fn sub(&self, parent: &Path, name: &str) -> PathBuf {
        let path = parent.join(name);
        fs::create_dir_all(&path).expect("a nested fixture directory");

        path
    }

    /// The project layer of `repository`, created:
    /// `<repository>/.config/smut/conventions`.
    fn project_sets(&self, repository: &Path) -> PathBuf {
        let path = repository.join(".config").join("smut").join("conventions");
        fs::create_dir_all(&path).expect("a project layer directory");

        path
    }

    /// The layer this scratch's `$XDG_CONFIG_HOME` names, created:
    /// `<config-home>/smut/conventions`.
    fn user_sets(&self) -> PathBuf {
        let path = self.config_home().join("smut").join("conventions");
        fs::create_dir_all(&path).expect("a user layer directory");

        path
    }

    /// The layer a home read as a config home would name, created:
    /// `<home>/.config/smut/conventions`.
    fn home_sets(&self) -> PathBuf {
        let path = self.home().join(".config").join("smut").join("conventions");
        fs::create_dir_all(&path).expect("a home's config layer directory");

        path
    }

    /// Writes `<directory>/<name>.md` holding exactly `contents`, and answers
    /// its path.
    fn set(&self, directory: &Path, name: &str, contents: &str) -> PathBuf {
        let path = directory.join(format!("{name}.md"));
        fs::write(&path, contents).expect("a set file");

        path
    }

    /// Asserts that `start` and every ancestor up to the filesystem root holds
    /// no `.git` entry.
    ///
    /// The `$TMPDIR` these scratch directories are made in must not itself be
    /// inside a checkout: if it were, the walk-up would find that repository for
    /// its own reason and the fixture would redden with a cause no assertion
    /// here is about. The message says so, because that failure is otherwise
    /// indistinguishable from a walk that is one directory too eager.
    fn assert_no_repository_above(&self, start: &Path) {
        for ancestor in start.ancestors() {
            let git = ancestor.join(".git");

            assert!(
                fs::symlink_metadata(&git).is_err(),
                "no `.git` entry above {}: found {}. A $TMPDIR inside a checkout would redden \
                 this fixture for the checkout's own reason rather than the test's",
                start.display(),
                git.display()
            );
        }
    }
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
    // The whole row, name, layer, the summary the shipped file's own frontmatter
    // declares, and the leaf path the bytes are embedded under, two spaces
    // apart: a listing that printed the frontmatter, invented a summary, dropped
    // the layer or spaced the columns differently reddens here. It is asserted
    // last on purpose — an exact row above the four weaker rows would make each
    // of them unfalsifiable.
    assert_eq!(
        listed.stdout,
        format!("hygiene  binary  {}  conventions/hygiene.md\n", summary()),
        "the row is the set's name, its layer, its declared summary and its own path"
    );
}

#[test]
fn a_named_set_prints_the_body_verbatim() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene"]);

    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    // The frontmatter is metadata, and none of it is printed: a reader that
    // reproduced the fence reddens here, before the exact equality at the end of
    // the test can stand in for these three.
    assert!(
        !printed.stdout.contains("summary:"),
        "the frontmatter's summary is not printed: {:?}",
        printed.stdout
    );
    // The version is the binary layer's, applied after the file is read; a
    // reader that folded it back into the frontmatter would print this key.
    assert!(
        !printed.stdout.contains("version:"),
        "the declared version is metadata and is not printed: {:?}",
        printed.stdout
    );
    assert!(
        !printed.stdout.starts_with("---"),
        "the printed prose does not open with the fence: {:?}",
        printed.stdout
    );

    // The set tells the reader which id a re-install will overwrite, so that id
    // must be the one the seed actually emits: a renamed set whose prose still
    // named the old id would send the reader to an entity nothing updates.
    let seed = smut(&scratch, &["conventions", "hygiene", "--format", "json"]).json();
    let id = seed["smut_id"]
        .as_str()
        .expect("the seed names its entity in smut_id");
    assert!(
        printed.stdout.contains(id),
        "the printed set names the id the seed emits ({id})"
    );

    // The body and the one newline the shell writes: a reader that printed the
    // whole file reddens here. It is asserted last on purpose — an exact
    // equality above the four weaker assertions would make each of them
    // unfalsifiable, since `body()` derives from the same bytes this equality
    // compares the output to.
    assert_eq!(
        printed.stdout,
        format!("{}\n", body()),
        "the prose is the file's body, plus the one newline the binary writes"
    );
}

#[test]
fn the_seed_title_is_the_heading_verbatim() {
    let scratch = Scratch::new();

    let declaration = smut(&scratch, &["conventions", "hygiene", "--format", "json"]).json();
    let title = declaration["title"]
        .as_str()
        .expect("the seed carries a title");

    let expected = heading();
    assert!(
        !expected.is_empty(),
        "the shipped body opens with a `# ` heading, so the title has something to be"
    );
    // Verbatim: a synthesised `Conventions: ` prefix, a lowercased first letter
    // or a title read off the filename reddens here.
    assert_eq!(
        title, expected,
        "the title is the file's own heading, character for character"
    );
}

// Reddens if the refusal drops the name asked for or the set that exists, if
// its clause loses the layer it was read from, or if a usage refusal exits with
// anything but 2.
#[test]
fn an_unknown_name_still_exits_two() {
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
    // The clause, not just the name: a caller who retries from this message has
    // to know which layer answered, and only the shipped layer answers here.
    assert!(
        refused.stderr.contains("hygiene (binary)"),
        "the set that exists is one clause, named with its layer: {:?}",
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
fn json_without_a_name_is_still_a_usage_error() {
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

// Reddens if the flag is validated after the operand: `nonsense` is a name no
// layer holds, so an operand-first run refuses for the name first and the
// refusal then names the wrong thing.
#[test]
fn an_unknown_format_is_still_refused_before_the_operand() {
    let scratch = Scratch::new();

    for format in ["dot", "yaml", "JSON", "markdown"] {
        let refused = smut(&scratch, &["conventions", "nonsense", "--format", format]);

        assert_eq!(refused.code, 2, "{format}: {}", refused.stderr);
        assert_eq!(refused.stdout, "", "{format} prints nothing");
        assert!(
            refused
                .stderr
                .contains(&format!("unknown --format {format}")),
            "the refusal names the format it refused: {:?}",
            refused.stderr
        );
        assert!(
            !refused.stderr.contains("unknown conventions set"),
            "the name does not resolve, so the refusal is about the flag and not the name: {:?}",
            refused.stderr
        );
    }
}

#[test]
fn the_seed_key_set_is_six_names() {
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

    assert_eq!(seed["smut_id"], json!("conventions-hygiene"));
    assert_eq!(seed["type"], json!("convention"));
    assert_eq!(
        seed["version"],
        json!(env!("CARGO_PKG_VERSION")),
        "the seed's version is the crate's, never a pinned release"
    );
    assert_eq!(
        seed["title"],
        json!(heading()),
        "the title is the file's heading, not a sentence about it"
    );
    assert_eq!(
        seed["body"],
        json!(body()),
        "the body is the file below its frontmatter, not an inline copy of it"
    );
    // Structural identity is the `smut_id` member — so an ordinary `id` field
    // here would be data the seed does not carry, not a second spelling of the
    // same identity. The key list pins both: six names, and `id` is not one.
    let fields: Vec<&str> = seed
        .as_object()
        .expect("one object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(
        fields,
        ["body", "smut_id", "source", "title", "type", "version"],
        "the seed supplies exactly smut_id, type, version, source, title and body — six names, \
         and no second spelling of identity: {seed}"
    );
}

// Reddens if the declaration is printed across lines, if the binary writes no
// final newline or two of them, or if the output is more than one JSON value.
#[test]
fn the_seed_is_one_compact_line() {
    let scratch = Scratch::new();

    let printed = smut(&scratch, &["conventions", "hygiene", "--format", "json"]);

    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert!(
        printed.stdout.ends_with('\n'),
        "the output ends in a newline, because the shell never adds one: {:?}",
        printed.stdout
    );
    // One JSON value and nothing after it: `from_str` refuses trailing input, so
    // a second declaration or a log line after it reddens here.
    let declaration = printed.json();
    assert!(
        declaration.is_object(),
        "the seed is one JSON object: {}",
        printed.stdout
    );

    // Exactly one newline in the whole output: `ends_with` above tolerates a
    // second one before it, and a pretty printer re-introduced anywhere adds one
    // per field.
    assert_eq!(
        printed.stdout.matches('\n').count(),
        1,
        "the whole declaration is one line, ending in one newline: {:?}",
        printed.stdout
    );

    // The whole line, field order included: `smut_id` is hoisted and the rest
    // print lexically, so the printer's documented order for a seed is
    // `smut_id, body, source, title, type, version`. One equality pins the
    // order, the compactness and the single newline together, so a reordering
    // that leaves the JSON valid reddens here.
    let quoted = |text: &str| Value::String(text.to_owned()).to_string();

    assert_eq!(
        printed.stdout,
        format!(
            "{{\"smut_id\":\"conventions-hygiene\",\"body\":{},\"source\":\"conventions/hygiene.md\",\"title\":{},\"type\":\"convention\",\"version\":{}}}\n",
            quoted(&body()),
            quoted(&heading()),
            quoted(env!("CARGO_PKG_VERSION")),
        ),
        "the declaration prints its six fields in the documented order: {:?}",
        printed.stdout
    );
}

// Reddens if any conventions invocation opens a store: the paired `get` would
// then answer from it rather than refusing for its absence, and that message is
// what tells a created-and-valid store from a created-and-corrupt one (RV-029
// F-18).
#[test]
fn the_command_still_creates_no_store() {
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
            !scratch.store_dir().exists(),
            "{arguments:?} created a store: {}",
            scratch.store_dir().display()
        );
    }

    // Both halves design §7 names, in the same working directory: nothing on
    // disk, and the command that would read a store refusing for its absence
    // rather than for corruption. The message is what discriminates a store that
    // was never written from one this suite cannot see.
    let refused = smut(&scratch, &["get", "conventions-hygiene"]);
    assert_eq!(refused.code, 1, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "a refusal prints no entity");
    assert_eq!(
        refused.stderr,
        format!("smut: no store at {}\n", scratch.store_dir().display()),
        "the store no conventions run created is absent, not corrupt"
    );

    // A path where a store would be, holding something no snapshot can read.
    // Every command that opened it would exit 1; conventions answers anyway.
    fs::write(scratch.store_dir(), b"not a store\n").expect("a decoy path at .smut");

    let printed = smut(&scratch, &["conventions", "hygiene"]);
    assert_eq!(
        printed.code, 0,
        "the command never opens a store: {}",
        printed.stderr
    );
    assert_eq!(
        printed.stdout,
        format!("{}\n", body()),
        "the command prints the same body with or without a decoy store present"
    );
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
    assert_eq!(
        first.json()["changes"][0]["smut_id"],
        json!("conventions-hygiene"),
        "the receipt's subject is the seed's structural identity: {}",
        first.stdout
    );
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
    assert_eq!(
        fetched.json()["objects"][0]["smut_id"],
        json!("conventions-hygiene"),
        "the one entity is stored under the seed's structural identity: {}",
        fetched.stdout
    );
}

#[test]
fn a_second_install_reports_the_body_it_replaced() {
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
            r#"{"smut_id":"conventions-hygiene","body":"Our house rule: one label per relation.","owner":"ops"}"#,
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
        change["previous"]["smut_id"],
        json!("conventions-hygiene"),
        "`previous` is the entity as it stood before the batch, identity included: {}",
        reinstalled.stdout
    );
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
        json!(body()),
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
        &[
            "declare",
            r#"{"smut_id":"conventions-hygiene","body":"Draft."}"#,
        ],
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

// ── C4: where a set comes from — three layers, and the walk that finds them ──

/// A bare set file (no frontmatter), so its body is the whole of it.
const MINE: &str = "\
# Mine

mine advice: only the config home holds this one.
";

/// The project's copy of the shipped name, with a summary of its own: the
/// listing's third field for the row in force is this and not the user's.
const PROJECT_HYGIENE: &str = "\
---
summary: the checkout's own structure rules
---
# Project hygiene

project advice: the checkout's copy is the one in force.
";

/// What [`PROJECT_HYGIENE`] prints: its body, below the frontmatter, plus the
/// one newline the binary writes.
const PROJECT_HYGIENE_BODY: &str = "\
# Project hygiene

project advice: the checkout's copy is the one in force.
";

/// The user's copy of the same name, shadowed by the project's.
const USER_HYGIENE: &str = "\
# User hygiene

user advice: this copy sits behind the checkout's.
";

/// The checkout's copy of the shipped name, declaring a version of its own: the
/// seed's `version` is the author's, never the crate's release.
const PROJECT_VERSIONED_HYGIENE: &str = "\
---
summary: the checkout's own structure rules
version: 2.5.0
---
# Project hygiene

project advice: this checkout says which release of the rules it follows.
";

/// What [`PROJECT_VERSIONED_HYGIENE`] declares as its body: the text below its
/// frontmatter, trailing newlines trimmed. That is the value the seed carries;
/// the prose prints it with the one newline the binary writes added back.
const PROJECT_VERSIONED_HYGIENE_BODY: &str = "\
# Project hygiene

project advice: this checkout says which release of the rules it follows.";

/// The config home's copy of the same name, with its own version and its own
/// body, so two declarations of one name differ in everything but their
/// identity.
const USER_VERSIONED_HYGIENE: &str = "\
---
version: 3.1.0
---
# User hygiene

user advice: this copy is a later release of the same advice.
";

/// What [`USER_VERSIONED_HYGIENE`] declares as its body, trailing newlines
/// trimmed, like every body a set file declares.
const USER_VERSIONED_HYGIENE_BODY: &str = "\
# User hygiene

user advice: this copy is a later release of the same advice.";

/// A set in a home's `.config/smut`, which is no layer while
/// `$XDG_CONFIG_HOME` points elsewhere.
const HOME_HYGIENE: &str = "\
---
summary: a home's copy that is never a layer
---
# Home hygiene

home advice: the home is not a layer of this listing.
";

/// The leaf's own project set, which no walk may climb above.
const OWN: &str = "\
# Own

own advice: the leaf's own copy.
";

/// The leaf's parent's project set, reachable only by a walk that climbs too
/// far.
const PARENT: &str = "\
# Parent

parent advice: an ancestor's copy, which the walk must not find.
";

// ── EX-1..EX-3: the declaration names its file and its author's version ─────

// Reddens if the declaration's `version` falls back to the crate's release, or
// if `source` is the absolute path the file was found at: the author's version
// and the layer-relative leaf are both what the file itself says.
#[test]
fn the_seed_names_its_source_and_the_authors_version() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    let project = scratch.set(
        &scratch.project_sets(&repository),
        "hygiene",
        PROJECT_VERSIONED_HYGIENE,
    );

    let printed = scratch.smut_in(&repository, &["conventions", "hygiene", "--format", "json"]);

    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "", "a seed is no diagnostic");

    let declaration = printed.json();

    assert_eq!(
        declaration["smut_id"],
        json!("conventions-hygiene"),
        "the id is fixed and layer-blind: {}",
        printed.stdout
    );
    assert_eq!(
        declaration["type"],
        json!("convention"),
        "the declared type does not depend on the layer: {}",
        printed.stdout
    );
    // The author's version, and only the author's: a `seed` that fell back to
    // the crate's release would print `env!("CARGO_PKG_VERSION")` here.
    assert_eq!(
        declaration["version"],
        json!("2.5.0"),
        "the version is the file's own frontmatter value (2.5.0), not the crate's {}: {}",
        env!("CARGO_PKG_VERSION"),
        printed.stdout
    );
    // The leaf, and only the leaf: an `origin.path` would name the absolute file
    // this scratch made, which is the value the message prints.
    assert_eq!(
        declaration["source"],
        json!("conventions/hygiene.md"),
        "the source is the file's name under its layer root, not {} — the absolute path it \
         was found at: {}",
        project.display(),
        printed.stdout
    );
    assert_eq!(
        declaration["title"],
        json!("Project hygiene"),
        "the title is the file's own heading, verbatim: {}",
        printed.stdout
    );
    assert_eq!(
        declaration["body"],
        json!(PROJECT_VERSIONED_HYGIENE_BODY),
        "the body is the file's text below its frontmatter, unchanged: {}",
        printed.stdout
    );
}

// Reddens if `source` or the id is read off the layer a file was found at: the
// two declarations differ in their bodies and in their versions, so the two
// identical strings asserted last are the rule and not a coincidence.
#[test]
fn the_source_is_the_same_at_every_layer() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    scratch.set(
        &scratch.project_sets(&repository),
        "hygiene",
        PROJECT_VERSIONED_HYGIENE,
    );
    scratch.set(&scratch.user_sets(), "hygiene", USER_VERSIONED_HYGIENE);
    let inside = scratch.sub(&repository, "sub");
    let work = scratch.dir("work");
    // Asserted first: no ancestor of this directory is a repository, so the
    // user layer is the only file layer this declaration can come from.
    scratch.assert_no_repository_above(&work);

    let from_project = scratch.smut_in(&inside, &["conventions", "hygiene", "--format", "json"]);
    assert_eq!(from_project.code, 0, "{}", from_project.stderr);
    let from_user = scratch.smut_in(&work, &["conventions", "hygiene", "--format", "json"]);
    assert_eq!(from_user.code, 0, "{}", from_user.stderr);

    let project_seed = from_project.json();
    let user_seed = from_user.json();

    // The two files really are two different files, so nothing below can pass by
    // the two replies being one reply.
    assert_eq!(
        project_seed["body"],
        json!(PROJECT_VERSIONED_HYGIENE_BODY),
        "the project layer's body is the checkout's text: {}",
        from_project.stdout
    );
    assert_eq!(
        user_seed["body"],
        json!(USER_VERSIONED_HYGIENE_BODY),
        "the user layer's body is the config home's text: {}",
        from_user.stdout
    );
    assert_eq!(
        project_seed["version"],
        json!("2.5.0"),
        "the checkout declares its own version: {}",
        from_project.stdout
    );
    assert_eq!(
        user_seed["version"],
        json!("3.1.0"),
        "the config home declares its own version: {}",
        from_user.stdout
    );
    // The two exact bodies and the two exact versions above are what make the
    // identical strings below a rule rather than a coincidence: the two replies
    // provably read two different files, so a shared `source` and `smut_id`
    // cannot come from a shared file.

    // … and one id and one source for both, because neither names a layer.
    assert_eq!(
        project_seed["smut_id"],
        json!("conventions-hygiene"),
        "the fixed id does not name the project layer: {}",
        from_project.stdout
    );
    assert_eq!(
        user_seed["smut_id"],
        json!("conventions-hygiene"),
        "the user layer declares under the same fixed id: {}",
        from_user.stdout
    );
    for (layer, seed) in [("project", &project_seed), ("user", &user_seed)] {
        assert_eq!(
            seed["source"],
            json!("conventions/hygiene.md"),
            "the {layer} layer's source is the literal leaf, the same at every layer — a path \
             taken from that layer would name its own file: {seed}"
        );
    }
}

// EX-3, RV-029 F-19: both halves in one test, so the prose and the declaration
// cannot drift apart while each is checked somewhere else.
#[test]
fn the_prose_and_the_seed_agree_on_the_body() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    scratch.set(
        &scratch.project_sets(&repository),
        "hygiene",
        PROJECT_VERSIONED_HYGIENE,
    );

    let prose = scratch.smut_in(&repository, &["conventions", "hygiene"]);
    assert_eq!(prose.code, 0, "{}", prose.stderr);
    let seed = scratch.smut_in(&repository, &["conventions", "hygiene", "--format", "json"]);
    assert_eq!(seed.code, 0, "{}", seed.stderr);

    // A re-introduced heading or an appended footer would make these two bodies
    // disagree with the file, and each other.
    assert_eq!(
        seed.json()["body"],
        json!(PROJECT_VERSIONED_HYGIENE_BODY),
        "the seed's body is the file's text below its frontmatter: {}",
        seed.stdout
    );
    assert_eq!(
        prose.stdout,
        format!("{}\n", PROJECT_VERSIONED_HYGIENE_BODY),
        "the prose is that same body plus exactly one newline, so a lost trailing newline \
         reddens here: {:?}",
        prose.stdout
    );
}

// Reddens if a farther layer wins the name — the prose would be the user's or
// the shipped body, not the checkout's — if the listing prints a summary where a
// nearer file shadows the name, or if a layer contributes no row.
#[test]
fn the_project_set_beats_the_user_set_beats_the_binary() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    let project = scratch.set(
        &scratch.project_sets(&repository),
        "hygiene",
        PROJECT_HYGIENE,
    );
    let user = scratch.set(&scratch.user_sets(), "hygiene", USER_HYGIENE);
    let inside = scratch.sub(&repository, "sub");

    let printed = scratch.smut_in(&inside, &["conventions", "hygiene"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert_eq!(
        printed.stdout, PROJECT_HYGIENE_BODY,
        "the nearest layer's body, below its frontmatter, plus exactly one newline"
    );

    let listed = scratch.smut_in(&inside, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);

    let rows: Vec<Vec<&str>> = listed
        .stdout
        .lines()
        .map(|row| row.split("  ").collect())
        .collect();
    assert_eq!(
        rows.len(),
        3,
        "one row per file, and three files hold `hygiene`: {}",
        listed.stdout
    );
    for row in &rows {
        assert_eq!(
            row.len(),
            4,
            "a row is four fields, two spaces apart: {row:?}"
        );
    }
    assert_eq!(
        rows.iter().map(|row| (row[0], row[1])).collect::<Vec<_>>(),
        [
            ("hygiene", "project"),
            ("hygiene", "user"),
            ("hygiene", "binary"),
        ],
        "layer-major, in precedence order: {}",
        listed.stdout
    );
    assert_eq!(
        rows[0][2], "the checkout's own structure rules",
        "the row in force prints its own summary: {}",
        listed.stdout
    );
    assert_eq!(
        rows[1][2], "shadowed",
        "a user row behind a nearer file says only that it is shadowed: {}",
        listed.stdout
    );
    assert_eq!(
        rows[2][2], "shadowed",
        "the shipped row is shadowed too, and prints no summary of its own: {}",
        listed.stdout
    );
    assert_eq!(
        rows[0][3],
        project.to_str().expect("a UTF-8 fixture path"),
        "the project row names the project layer's own file: {}",
        listed.stdout
    );
    assert_eq!(
        rows[1][3],
        user.to_str().expect("a UTF-8 fixture path"),
        "the user row names the config home's file: {}",
        listed.stdout
    );
    assert_eq!(
        rows[2][3], "conventions/hygiene.md",
        "the shipped row names the path its bytes are embedded under: {}",
        listed.stdout
    );
}

// Reddens if the user layer is not read: `mine` is in no other layer, so the
// prose is an unknown-set refusal and the listing is the shipped row alone.
#[test]
fn a_set_that_exists_only_in_the_config_home_resolves() {
    let scratch = Scratch::new();
    let mine = scratch.set(&scratch.user_sets(), "mine", MINE);
    let work = scratch.dir("work");
    scratch.assert_no_repository_above(&work);

    let printed = scratch.smut_in(&work, &["conventions", "mine"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    // The file has no frontmatter, so its body is the whole of it: a reader that
    // printed nothing, or that found another layer's copy instead, reddens here.
    assert_eq!(
        printed.stdout, MINE,
        "the config home's set, printed as prose"
    );

    let listed = scratch.smut_in(&work, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    // The third field is `-` because the fixture declares no summary: a set that
    // declares none is still in force and still has a row.
    assert_eq!(
        listed.stdout,
        format!(
            "mine  user  -  {}\nhygiene  binary  {}  conventions/hygiene.md\n",
            mine.display(),
            summary()
        ),
        "the config home's file is a `user` row, ahead of the shipped one"
    );
}

// Reddens if the project root is the current directory rather than the nearest
// repository: from `<repo>/sub/dir` no layer holds `mine` and the prose is a
// refusal at exit 2.
#[test]
fn the_walk_up_finds_the_repositorys_set() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    scratch.set(&scratch.project_sets(&repository), "mine", MINE);
    let inside = scratch.sub(&scratch.sub(&repository, "sub"), "dir");

    let printed = scratch.smut_in(&inside, &["conventions", "mine"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert_eq!(
        printed.stdout, MINE,
        "the set three directories up, found by the walk from sub/dir"
    );
}

// Reddens if only a `.git` directory counts as a repository: the worktree's
// `.git` file is passed over, the walk leaves the tree, and `mine` is unknown.
#[test]
fn a_worktree_is_a_repository() {
    let scratch = Scratch::new();
    let tree = scratch.worktree("tree");
    let mine = scratch.set(&scratch.project_sets(&tree), "mine", MINE);
    let inside = scratch.sub(&tree, "src");

    let printed = scratch.smut_in(&inside, &["conventions", "mine"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(printed.stderr, "");
    assert_eq!(
        printed.stdout, MINE,
        "the worktree's own set, found through a `.git` regular file"
    );

    let listed = scratch.smut_in(&inside, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(
        listed.stdout,
        format!(
            "mine  project  -  {}\nhygiene  binary  {}  conventions/hygiene.md\n",
            mine.display(),
            summary()
        ),
        "a `.git` file is a repository, so the row's layer is `project`"
    );
}

// Reddens if the walk climbs past the repository, or if the home is read as a
// layer through a fallback from `$XDG_CONFIG_HOME`: the home's `hygiene` would
// then be adopted as a nearer layer and shadow the shipped row.
#[test]
fn the_walk_stops_at_the_repository() {
    let scratch = Scratch::new();
    scratch.set(&scratch.home_sets(), "hygiene", HOME_HYGIENE);
    // A checkout inside the home, deliberately: the walk starts inside one, so
    // nothing above it is searched. `assert_no_repository_above` is not called
    // here — there is one, and the test is about stopping at it.
    let repository = scratch.sub(&scratch.home(), "work");
    fs::create_dir_all(repository.join(".git")).expect("a repository's .git directory");
    let inside = scratch.sub(&repository, "sub");

    let listed = scratch.smut_in(&inside, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);

    let rows: Vec<Vec<&str>> = listed
        .stdout
        .lines()
        .map(|row| row.split("  ").collect())
        .collect();
    let home_config = scratch.home().join(".config");
    for row in &rows {
        assert_eq!(
            row.len(),
            4,
            "a row is four fields, two spaces apart: {row:?}"
        );
        assert!(
            !Path::new(row[3]).starts_with(&home_config),
            "no row comes from {}: the home's `.config` is not a project layer and \
             `$XDG_CONFIG_HOME` points elsewhere, so it is not the user layer either: {}",
            home_config.display(),
            listed.stdout
        );
    }

    let hygiene = rows
        .iter()
        .find(|row| row[0] == "hygiene")
        .unwrap_or_else(|| panic!("the shipped set is listed: {}", listed.stdout));
    assert_eq!(
        hygiene[1], "binary",
        "the shipped set is still in force, from the binary layer: {}",
        listed.stdout
    );
    assert_eq!(
        hygiene[2],
        summary(),
        "its third field is its own summary and not `shadowed`, so no nearer layer \
         was adopted: {}",
        listed.stdout
    );
}

// Reddens if the walk continues above the start directory or climbs to a
// `.config` it finds there: the parent's `parent.md` would then be listed and
// addressable instead of refused.
#[test]
fn no_repository_means_the_cwd_and_no_walk_up() {
    let scratch = Scratch::new();
    let leaf = scratch.dir("leaf");
    // Asserted first: a `$TMPDIR` inside a checkout would make the outcome here
    // depend on the machine rather than on the walk.
    scratch.assert_no_repository_above(&leaf);
    let own = scratch.set(&scratch.project_sets(&leaf), "own", OWN);
    scratch.set(&scratch.project_sets(scratch.path()), "parent", PARENT);

    let listed = scratch.smut_in(&leaf, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(
        listed.stdout,
        format!(
            "own  project  -  {}\nhygiene  binary  {}  conventions/hygiene.md\n",
            own.display(),
            summary()
        ),
        "the leaf's own file is the whole project layer, and the parent's is not listed"
    );

    let refused = scratch.smut_in(&leaf, &["conventions", "parent"]);
    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "a refusal prints no prose");
    assert!(
        refused.stderr.contains("parent"),
        "the refusal names what was asked for: {:?}",
        refused.stderr
    );
}

// ── C4b: the failure classes, and the rows a broken file keeps ──────────────

/// Frontmatter and nothing below it: a valid fence over an empty body, which is
/// §3's empty-body failure class (DEC-028). The frontmatter itself parses, so the
/// only thing wrong with this file is that it says nothing.
const SILENT: &str = "---\nsummary: nothing yet\n---\n";

/// The reason [`SILENT`] fails with: the reader's own sentence from §3's failure
/// table, asserted rather than restated by each fixture around it.
const SILENT_REASON: &str = "a set that says nothing is a mistake, not advice";

// Reddens if a broken nearest set falls through to a working farther copy, or if
// the loser is promoted.
#[test]
fn a_broken_project_set_shadows_a_good_user_set() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    let project = scratch.set(&scratch.project_sets(&repository), "hygiene", SILENT);
    let user = scratch.set(&scratch.user_sets(), "hygiene", USER_HYGIENE);
    let inside = scratch.sub(&repository, "sub");

    let refused = scratch.smut_in(&inside, &["conventions", "hygiene"]);
    assert_eq!(refused.code, 1, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "a refusal prints no prose");
    assert!(
        refused
            .stderr
            .contains(project.to_str().expect("a UTF-8 fixture path")),
        "the refusal names the project file, which is the set in force: {:?}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains(SILENT_REASON),
        "and the reason it did not load: {:?}",
        refused.stderr
    );
    assert!(
        !refused
            .stderr
            .contains(user.to_str().expect("a UTF-8 fixture path")),
        "the working user copy is not the diagnosis, because it is not in force: {:?}",
        refused.stderr
    );
    assert!(
        !refused.stderr.contains("store"),
        "a set that did not load claims no store, because none was opened: {:?}",
        refused.stderr
    );

    // The same fixture, listed: the user's copy is still behind the broken
    // nearer file and says so, rather than being promoted into force by a
    // failure it had no part in.
    let listed = scratch.smut_in(&inside, &["conventions"]);
    assert_eq!(
        listed.code, 0,
        "a listing reports and succeeds: {}",
        listed.stderr
    );

    let user_row = row_naming(&listed, user.to_str().expect("a UTF-8 fixture path"));
    assert_eq!(
        user_row[2], "shadowed",
        "the user's copy is shadowed by the broken project file: {}",
        listed.stdout
    );
}

// Reddens if failure loses to shadowing: the user row would say `shadowed` and
// the reason it did not load would be unreadable from the listing alone.
#[test]
fn a_failed_row_behind_a_working_one_keeps_its_reason() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    let project = scratch.set(
        &scratch.project_sets(&repository),
        "hygiene",
        PROJECT_HYGIENE,
    );
    let user = scratch.set(&scratch.user_sets(), "hygiene", SILENT);
    let inside = scratch.sub(&repository, "sub");

    let listed = scratch.smut_in(&inside, &["conventions"]);
    assert_eq!(
        listed.code, 0,
        "a listing reports and succeeds: {}",
        listed.stderr
    );

    let project_row = row_naming(&listed, project.to_str().expect("a UTF-8 fixture path"));
    assert_eq!(
        project_row[2], "the checkout's own structure rules",
        "the working project row prints its own summary: {}",
        listed.stdout
    );

    let user_row = row_naming(&listed, user.to_str().expect("a UTF-8 fixture path"));
    // Weakest first, so each fact below can redden on its own: the row is not
    // only its demotion, the demotion is rendered as the load failure it is, and
    // the failure carries the reader's real reason rather than a generic word.
    assert_ne!(
        user_row[2], "shadowed",
        "failure wins over the shadow, so the row is not only the demotion: {}",
        listed.stdout
    );
    assert!(
        user_row[2].starts_with("did not load:"),
        "a broken file states why, even behind a working one: {:?}",
        user_row[2]
    );
    assert!(
        user_row[2].contains(SILENT_REASON),
        "and carries the real reason: {:?}",
        user_row[2]
    );
}

// Reddens if an absent directory is treated as a failure, or invents a row.
#[test]
fn an_absent_layer_is_not_an_error() {
    let scratch = Scratch::new();
    let work = scratch.dir("work");
    // Asserted first: a `$TMPDIR` inside a checkout would give the walk-up a
    // repository of its own and redden this fixture for a reason it is not about.
    scratch.assert_no_repository_above(&work);

    let listed = scratch.smut_in(&work, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(listed.stderr, "", "an absent layer is not a diagnostic");
    assert_eq!(
        listed.stdout,
        format!("hygiene  binary  {}  conventions/hygiene.md\n", summary()),
        "no project and no user layer contribute no row and no failure"
    );

    let printed = scratch.smut_in(&work, &["conventions", "hygiene"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(
        printed.stdout,
        format!("{}\n", body()),
        "the shipped set still answers while two layers are simply not there"
    );
}

// Reddens if a layer that exists and cannot be read is read as "no sets here":
// the listing would print the shipped row alone and exit 0.
#[test]
fn an_unreadable_layer_exits_one() {
    let scratch = Scratch::new();
    let work = scratch.dir("work");
    scratch.assert_no_repository_above(&work);

    // A regular file where the user layer's *root* belongs, so the `conventions`
    // directory below it cannot be read as one. Portable and root-proof: a
    // mode-000 directory still opens for a root process, and that row is IMP-020's.
    fs::create_dir_all(scratch.config_home()).expect("a config home");
    let root = scratch.config_home().join("smut");
    fs::write(&root, b"not a directory\n").expect("a file where the user layer root belongs");
    let unreachable = root.join("conventions");

    let refused = scratch.smut_in(&work, &["conventions"]);
    assert_eq!(refused.code, 1, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "a refusal prints no listing at all");
    assert!(
        refused
            .stderr
            .contains(unreachable.to_str().expect("a UTF-8 fixture path")),
        "the diagnostic names the layer it could not read: {:?}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("reading the conventions layer"),
        "and the step that failed: {:?}",
        refused.stderr
    );
}

// Reddens if a set that declares no summary has no row at all, or a blank third
// field, or if the field is filled with a summary the file never declared.
#[test]
fn a_bare_markdown_file_lists_with_a_dash() {
    let scratch = Scratch::new();
    let work = scratch.dir("work");
    scratch.assert_no_repository_above(&work);
    let mine = scratch.set(&scratch.project_sets(&work), "mine", MINE);

    let listed = scratch.smut_in(&work, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(
        listed.stdout,
        format!(
            "mine  project  -  {}\nhygiene  binary  {}  conventions/hygiene.md\n",
            mine.display(),
            summary()
        ),
        "a bare markdown file is in force and its third field is the dash"
    );
}

// Reddens if a listing that reports a broken file exits non-zero, or if it hides
// the reason behind a word that would not tell a reader what to fix.
#[test]
fn the_listing_reports_a_broken_file_and_succeeds() {
    let scratch = Scratch::new();
    let work = scratch.dir("work");
    scratch.assert_no_repository_above(&work);
    let broken = scratch.set(&scratch.user_sets(), "hygiene", SILENT);

    let listed = scratch.smut_in(&work, &["conventions"]);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    assert_eq!(
        listed.stderr, "",
        "the report is the listing, not a diagnostic"
    );

    let broken_row = row_naming(&listed, broken.to_str().expect("a UTF-8 fixture path"));
    assert_eq!(
        broken_row[2],
        format!("did not load: {SILENT_REASON}"),
        "the broken row is the prefix and the real reason: {}",
        listed.stdout
    );
    // The shipped row is still listed in the same successful report, behind the
    // broken user file that carries its name — so it prints `shadowed`, and the
    // report carries both states at once.
    let shipped_row = row_naming(&listed, "conventions/hygiene.md");
    assert_eq!(
        (&shipped_row[0], &shipped_row[1]),
        (&"hygiene", &"binary"),
        "the shipped set is listed in the same report as the broken file: {}",
        listed.stdout
    );
}

// Reddens if the store root walks up with the sets, or if the sets stop at the
// cwd: either root would answer from the other's place.
#[test]
fn the_two_roots_disagree_on_purpose() {
    let scratch = Scratch::new();
    let repository = scratch.repository("repo");
    scratch.set(&scratch.project_sets(&repository), "mine", MINE);
    let inside = scratch.sub(&scratch.sub(&repository, "sub"), "dir");

    let printed = scratch.smut_in(&inside, &["conventions", "mine"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert_eq!(
        printed.stdout, MINE,
        "the advice walks up to the repository from sub/dir"
    );

    // The same cwd, the other root: a store is data placed at `<cwd>/.smut` and
    // nothing above the cwd is read (DEC-004).
    let refused = scratch.smut_in(&inside, &["get", "conventions-hygiene"]);
    assert_eq!(refused.code, 1, "{}", refused.stderr);
    assert_eq!(refused.stdout, "", "an absent store prints no entity");
    assert_eq!(
        refused.stderr,
        format!("smut: no store at {}\n", inside.join(".smut").display()),
        "the store root is the cwd alone, so the repository's sets are no store"
    );
    assert!(
        !inside.join(".smut").exists(),
        "and the failed read created nothing beside the cwd: {}",
        inside.join(".smut").display()
    );
}

/// The listing row naming `path` in its fourth field, so a fixture asserts on the
/// row for its own file rather than on a position another layer could take.
///
/// Panics with the whole listing when no row names the path: a row that vanished
/// is the thing these fixtures are about.
fn row_naming<'a>(listed: &'a Ran, path: &str) -> Vec<&'a str> {
    listed
        .stdout
        .lines()
        .map(|row| row.split("  ").collect::<Vec<&str>>())
        .find(|row| row.get(3) == Some(&path))
        .unwrap_or_else(|| panic!("a row names {path}: {}", listed.stdout))
}
