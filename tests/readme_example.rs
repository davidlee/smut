//! The README's worked example, executed (PHASE-08 EX-4).
//!
//! `README.md` carries exactly one `console` block: the worked example. This
//! test reads that block out of the README, runs every command in it, in order,
//! in a scratch store, and compares what the binary printed with the transcript
//! beside it. Editing the README's transcript without re-running it fails here.
//!
//! **What "matches" means for JSON.** A recorded output that parses as JSON is
//! compared as a parsed value, not as bytes: `render`'s Markdown and the JSON
//! envelopes are both compared structurally, so a reordering that changes no
//! value does not fail here — the byte form of an entity node is pinned where it
//! belongs, in `src/entity.rs` and `tests/cli_contract.rs`. A recorded output
//! beginning `smut: ` is a diagnostic: that command must fail, write nothing to
//! stdout, and print exactly that line on stderr.
//!
//! **The README also has to survive publication.** crates.io renders it against
//! its own paths, so a relative link is dead on the published page; one test here
//! requires every link target to be absolute.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

const README: &str = include_str!("../README.md");

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-readme-example-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a scratch directory");

        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One command of the transcript and the output the README records for it.
struct Step {
    command: String,
    args: Vec<String>,
    output: String,
}

/// The worked example: the README's one `console` block, as steps.
fn worked_example() -> Vec<Step> {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut inside = false;

    for line in README.lines() {
        if line.trim_start().starts_with("```") {
            if inside {
                inside = false;
            } else {
                inside = line.trim() == "```console";
                if inside {
                    blocks.push(Vec::new());
                }
            }

            continue;
        }
        if let Some(block) = blocks.last_mut()
            && inside
        {
            block.push(line);
        }
    }

    assert_eq!(
        blocks.len(),
        1,
        "the README carries exactly one console block, and it is the worked example"
    );

    let mut steps: Vec<Step> = Vec::new();

    for line in &blocks[0] {
        let Some(command) = line.strip_prefix("$ ") else {
            steps
                .last_mut()
                .expect("the block opens with a command")
                .output
                .push_str(&format!("{line}\n"));

            continue;
        };
        let mut args = arguments(command);

        assert_eq!(
            args.first().map(String::as_str),
            Some("smut"),
            "the transcript invokes the binary by name: {command}"
        );
        args.remove(0);

        steps.push(Step {
            command: command.to_owned(),
            args,
            output: String::new(),
        });
    }

    for step in &mut steps {
        step.output = step.output.trim_end_matches('\n').to_owned();
    }

    assert!(!steps.is_empty(), "the worked example has commands");

    steps
}

/// Split a shell-ish command line on whitespace, honouring 'single quotes'.
fn arguments(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;

    for character in command.chars() {
        match character {
            '\'' => {
                quoted = !quoted;
                started = true;
            }
            ' ' | '\t' if !quoted => {
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            other => {
                current.push(other);
                started = true;
            }
        }
    }

    assert!(!quoted, "the command line has balanced quotes: {command}");

    if started {
        args.push(current);
    }

    args
}

struct Ran {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(scratch: &Scratch, args: &[String]) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_smut"))
        .current_dir(scratch.path())
        .args(args)
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

#[test]
fn the_readme_worked_example_runs_and_prints_what_it_records() {
    let scratch = Scratch::new();

    for step in worked_example() {
        let ran = run(&scratch, &step.args);
        let shown = &step.output;

        if shown.starts_with("smut: ") {
            assert_ne!(ran.code, 0, "`{}` is shown failing", step.command);
            assert_eq!(ran.stdout, "", "`{}` shows no envelope", step.command);
            assert_eq!(
                ran.stderr.trim_end(),
                shown,
                "`{}` prints the diagnostic the README records",
                step.command
            );

            continue;
        }

        assert_eq!(ran.code, 0, "`{}`: {}", step.command, ran.stderr);
        assert_eq!(ran.stderr, "", "`{}` is quiet on stderr", step.command);

        match serde_json::from_str::<Value>(shown) {
            Ok(recorded) => {
                let actual: Value = serde_json::from_str(&ran.stdout).unwrap_or_else(|error| {
                    panic!(
                        "`{}` prints one JSON value ({error}): {:?}",
                        step.command, ran.stdout
                    )
                });

                assert_eq!(
                    actual, recorded,
                    "`{}` prints the envelope the README records",
                    step.command
                );
            }
            Err(_) => assert_eq!(
                ran.stdout,
                format!("{shown}\n"),
                "`{}` prints exactly the text the README records",
                step.command
            ),
        }
    }
}

/// The example ends with the traversal answer, not a refusal. This is a DEC-019
/// re-cut, renamed and re-asserted, never deleted: 0.1's example ended at the
/// boundary it could not cross, and 0.2's example ends with the walk, so the old
/// assertion would have pinned the README to a refusal the binary no longer
/// makes.
#[test]
fn the_worked_example_ends_with_a_0_2_traversal() {
    let steps = worked_example();
    let last = steps.last().expect("the example has commands");

    assert!(
        last.command.contains("follow"),
        "the example ends with the traversal query: {}",
        last.command
    );
    assert!(
        !last.output.starts_with("smut: "),
        "the last step shows an envelope, not a diagnostic: {}",
        last.output
    );

    let envelope: Value =
        serde_json::from_str(&last.output).expect("the last step prints one JSON envelope");

    assert!(
        envelope.get("traversal").is_some(),
        "the envelope reports the walk it performed: {envelope}"
    );
    assert_eq!(
        envelope["dangling"],
        json!([{"field": "questions", "source": "smut", "target": "q-query"}]),
        "the walk reports the reference to the deleted entity as dangling"
    );

    let ids: Vec<&str> = envelope["objects"]
        .as_array()
        .expect("the envelope carries an objects array")
        .iter()
        .map(|object| object["id"].as_str().expect("every object has an id"))
        .collect();

    assert_eq!(ids, ["smut"], "the walk returns the source entity alone");
}

/// The console block is the brief's end-to-end example, not an excerpt: traverse,
/// filter, render Markdown, delete a reference and see it reported as dangling,
/// with the Mermaid step still named as 1.0's (§13.3 scenario 1, §14.1). The
/// runner above executes every step; this test pins that the steps the criterion
/// names are the ones the block carries, so a shorter block cannot pass by being
/// executed faithfully.
#[test]
fn the_console_block_carries_the_whole_0_2_example() {
    let steps = worked_example();
    let matching = |predicate: &dyn Fn(&Step) -> bool| -> usize {
        steps.iter().filter(|step| predicate(step)).count()
    };

    // Traverse: an outgoing walk that also projects fields.
    assert!(
        matching(&|step| step.command.starts_with("smut query ")
            && step.command.contains("\"follow\"")
            && step.command.contains("\"select\""))
            >= 1,
        "the block walks the graph with a projection: {:?}",
        steps.iter().map(|step| &step.command).collect::<Vec<_>>()
    );

    // Filter: a `where` step, and the envelope it prints carries the filtered
    // entity rather than the whole store.
    let filtered = steps
        .iter()
        .find(|step| step.command.contains("\"where\""))
        .expect("the block filters with a predicate");
    let filtered_envelope: Value =
        serde_json::from_str(&filtered.output).expect("the filter step prints an envelope");
    assert_eq!(
        filtered_envelope["objects"],
        json!([{
            "id": "q-query",
            "body": "Use bounded traversal and simple predicates.",
            "status": "resolved",
            "title": "Query design",
            "type": "question",
        }]),
        "the filter selects the amended question and nothing else: {filtered_envelope}"
    );

    // Render: the same walk, printed as Markdown rather than JSON.
    assert!(
        steps
            .iter()
            .any(|step| step.command.starts_with("smut render ")
                && step.command.contains("\"follow\"")
                && step.output.starts_with("## ")),
        "the block renders a traversal as Markdown"
    );

    // The Mermaid step is still 1.0's, and says so by name rather than quietly
    // falling back to Markdown.
    let mermaid = steps
        .iter()
        .find(|step| step.command.contains("--format mermaid"))
        .expect("the block keeps the Mermaid step the brief carries");
    assert!(
        mermaid.output.starts_with("smut: "),
        "the Mermaid step is refused, not rendered: {}",
        mermaid.output
    );
    assert!(
        mermaid.output.contains("1.0"),
        "the refusal names the release that plans it: {}",
        mermaid.output
    );
    assert!(
        !mermaid.output.contains("in 0.1"),
        "the refusal claims no release this tree is not: {}",
        mermaid.output
    );

    // Delete a reference, then see it: the delete step and the dangling report
    // are both present, and the report is the last word.
    assert!(
        matching(&|step| step.command.starts_with("smut delete ")) >= 1,
        "the block deletes an entity a reference points at"
    );
    assert!(
        steps
            .iter()
            .take(steps.len() - 1)
            .all(|step| !step.output.contains("\"dangling\":[{")),
        "no step before the last reports the reference as dangling"
    );
}

/// crates.io renders the README against its own paths, so a relative link is dead
/// on the published page — which is where a crate's reader arrives first.
#[test]
fn every_readme_link_is_absolute_or_an_anchor() {
    let mut checked = 0;

    for (at, _) in README.match_indices("](") {
        let target = README[at + 2..]
            .split(')')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();

        assert!(
            target.starts_with("https://")
                || target.starts_with("http://")
                || target.starts_with('#'),
            "a relative link does not resolve where this file is published \
             (offset {at}): {target}"
        );

        checked += 1;
    }

    assert!(
        checked >= 3,
        "the README still carries its links: {checked}"
    );
}
