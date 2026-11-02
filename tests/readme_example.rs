//! The README's transcripts, executed (PHASE-01 EX-11, extended by PHASE-07
//! VT-2; an earlier header named a PHASE-08 that was never planned).
//!
//! Every `console` block in `README.md` is a transcript, and every one is run.
//! A `## ` section's blocks are one continuous transcript, executed in document
//! order in a scratch store of that section's own, so a section can put prose
//! between its steps and still declare, then read back, what it declared.
//! Editing a recorded output without re-running the example fails here.
//!
//! A fence must say exactly `console`. CHR-001 found the README wearing
//! ` ``` console ` — with a space — which no harness matched, so the examples sat
//! green and unexecuted. A fence that means to be `console` and is not is a
//! failure here rather than a block silently skipped.
//!
//! A command may also wrap: while a quote is open, the following console lines
//! are still that command, exactly as a shell reads them. The declaration that
//! writes one small graph is wrapped that way, so the alternative is a harness
//! that cannot run a valid example.
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

/// One `## ` section's console blocks, in document order, as steps.
struct Transcript {
    heading: String,
    steps: Vec<Step>,
}

/// The README's transcripts: one per `## ` section that carries a `console` block.
fn transcripts(readme: &str) -> Vec<Transcript> {
    let mut transcripts: Vec<Transcript> = Vec::new();
    let mut heading = String::new();
    // The open fence's info string, while one is open.
    let mut fence: Option<String> = None;
    // A command whose quote is still open, while it wraps across console lines.
    let mut continuation: Option<String> = None;

    for line in readme.lines() {
        let trimmed = line.trim();

        // Inside a fence: the closing line, or a line of the console block.
        if let Some(open) = fence.take() {
            if trimmed.starts_with("```") {
                assert!(
                    continuation.is_none(),
                    "a wrapped command closes its quote inside its block: {}",
                    continuation.as_deref().unwrap_or_default()
                );

                continue;
            }

            fence = Some(open.clone());

            if open == "console" {
                let steps = &mut transcripts
                    .last_mut()
                    .expect("a console block opens a transcript")
                    .steps;

                // A command may wrap while a quote is open. A shell accepts a
                // quote that spans newlines, so the harness must too: otherwise
                // `smut declare '` reads as a command with unbalanced quotes and
                // its JSON is recorded as the output it is not.
                if let Some(pending) = continuation.take() {
                    let command = format!("{pending}\n{line}");

                    if quotes_balanced(&command) {
                        steps.push(step(&command));
                    } else {
                        continuation = Some(command);
                    }
                } else {
                    match line.strip_prefix("$ ") {
                        Some(command) if quotes_balanced(command) => steps.push(step(command)),
                        Some(command) => continuation = Some(command.to_owned()),
                        None => steps
                            .last_mut()
                            .expect("a console block opens with a command")
                            .output
                            .push_str(&format!("{line}\n")),
                    }
                }
            }

            continue;
        }

        // Opening a fence. A block that means `console` and is not is invisible
        // to this file, which is exactly the defect CHR-001 records, so it fails.
        if trimmed.starts_with("```") {
            // As written after the backticks, untrimmed: ` ```console ` is the one
            // fence this file executes, and ` ``` console ` is the mistake it refuses.
            let info = trimmed.trim_start_matches('`');

            assert!(
                !info.trim_start().starts_with("console") || info == "console",
                "the harness matches the fence exactly ` ```console `, so this block never \
                 runs: {trimmed}"
            );

            if info.trim() == "console" {
                // A section's blocks are one transcript: prose may sit between
                // them, and the store carries over so a later block can read
                // what an earlier one declared.
                let continues_section = transcripts
                    .last()
                    .is_some_and(|transcript| transcript.heading == heading);

                if !continues_section {
                    transcripts.push(Transcript {
                        heading: heading.clone(),
                        steps: Vec::new(),
                    });
                }
            }

            fence = Some(info.to_owned());

            continue;
        }

        if let Some(text) = trimmed.strip_prefix("## ") {
            heading = text.trim().to_owned();
        }
    }

    for transcript in &mut transcripts {
        assert!(
            !transcript.steps.is_empty(),
            "`## {}` carries a console block with commands",
            transcript.heading
        );

        for step in &mut transcript.steps {
            step.output = step.output.trim_end_matches('\n').to_owned();
        }
    }

    transcripts
}

/// One `$ `-prefixed command line, and the output the README records for it.
fn step(command: &str) -> Step {
    let mut args = arguments(command);

    assert_eq!(
        args.first().map(String::as_str),
        Some("smut"),
        "the transcript invokes the binary by name: {command}"
    );
    args.remove(0);

    Step {
        command: command.to_owned(),
        args,
        output: String::new(),
    }
}

/// The transcript under one `## ` heading — the exercised section's only one.
fn transcript(heading: &str) -> Vec<Step> {
    let mut found: Vec<Vec<Step>> = transcripts(README)
        .into_iter()
        .filter(|transcript| transcript.heading == heading)
        .map(|transcript| transcript.steps)
        .collect();

    assert_eq!(
        found.len(),
        1,
        "the README carries console commands under exactly one `## {heading}` heading"
    );

    found.pop().expect("the section was found")
}

/// Whether every single quote a command opens is closed again.
fn quotes_balanced(command: &str) -> bool {
    command.matches('\'').count().is_multiple_of(2)
}

/// A shell-ish command line, split into the arguments the binary receives.
///
/// The line may span newlines while a quote is open, so this walks characters,
/// not lines: whitespace outside a quote separates arguments and anything while
/// quoted is content.
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

/// A quoted operand may wrap across lines. `README.md` declares one graph that
/// way — a shell accepts a quote that spans newlines, so the harness must too:
/// the continuation lines belong to the command, never to its recorded output.
#[test]
fn a_quoted_command_may_wrap_across_lines() {
    let sample = r#"## Wrapped

```console
$ smut declare '
 {"smut_id":"a"}
'
{"ids":["a"]}
```
"#;

    let transcripts = transcripts(sample);
    assert_eq!(transcripts.len(), 1, "one section, one transcript");
    assert_eq!(
        transcripts[0].steps.len(),
        1,
        "a wrapped command is one step, not a command plus output"
    );

    let step = &transcripts[0].steps[0];
    assert_eq!(
        step.args,
        ["declare", "\n {\"smut_id\":\"a\"}\n"],
        "the whole quoted operand reaches the binary as one argument"
    );
    assert_eq!(
        step.output, "{\"ids\":[\"a\"]}",
        "only the line after the closing quote is output"
    );
}

#[test]
fn every_readme_transcript_runs_and_prints_what_it_records() {
    for transcript in transcripts(README) {
        // A section is one continuous transcript in a store of its own: it may
        // declare in one block and read back in the next, and no section sees
        // another's entities.
        let scratch = Scratch::new();

        for step in &transcript.steps {
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
}

/// The example ends with the traversal answer, not a refusal. This is a DEC-019
/// re-cut, renamed and re-asserted, never deleted: 0.1's example ended at the
/// boundary it could not cross, and 0.2's example ends with the walk, so the old
/// assertion would have pinned the README to a refusal the binary no longer
/// makes.
#[test]
fn the_worked_example_ends_with_a_0_2_traversal() {
    let steps = transcript("Worked example");
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
        .map(|object| object["smut_id"].as_str().expect("every object has an id"))
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
    let steps = transcript("Worked example");
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
            "smut_id": "q-query",
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

/// The README states this tree's version and the published release's in the same
/// breath, because `cargo install smut` — the line directly above the claim —
/// installs 0.1.1, not this tree. Removing the distinction is how the page
/// over-claims in composition while no single sentence is false (RV-017), so the
/// pin keeps the honest limit where a reader meets it.
#[test]
fn the_readme_names_the_published_release_beside_this_tree() {
    assert!(
        README.contains("smut 0.2.2"),
        "the release paragraph names the version this tree is"
    );
    assert!(
        README.contains("0.1.1") && README.contains("crates.io"),
        "the install line is qualified by the release crates.io actually carries"
    );
    assert!(
        README.contains("is not published"),
        "and this tree is not claimed to be published"
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

/// CHR-001's defect, pinned. The README once wore ` ``` console ` — with a space
/// — and the harness of the day matched nothing, so its examples sat green and
/// unexecuted. A fence that means `console` and is not exactly that is refused
/// here rather than skipped, because a block nobody runs is worse than no block:
/// it occupies the evidence slot.
#[test]
#[should_panic(expected = "the harness matches the fence exactly")]
fn a_console_fence_that_is_not_exactly_console_is_refused() {
    transcripts("## Take a peek\n\n``` console\n$ smut learn\n```\n");
}

/// The look the `Take a peek` section needs: prose between the blocks, one store
/// across them, and an output line that begins `## ` — a rendered heading — is
/// not read as the next section's heading.
#[test]
fn a_sections_console_blocks_are_one_transcript() {
    let readme = "\
## Take a peek

```console
$ smut declare '{\"smut_id\":\"principles\"}'
{\"ids\":[\"principles\"]}
$ smut render '{\"from\":\"principles\"}'
## principles

- one
```

Prose that names no heading.

```console
$ smut get principles
{\"objects\":[]}
```

## Next section

```console
$ smut learn
the guide
```
";

    let transcripts = transcripts(readme);

    assert_eq!(transcripts.len(), 2, "one transcript per section");
    assert_eq!(transcripts[0].heading, "Take a peek");
    assert_eq!(
        transcripts[0].steps.len(),
        3,
        "both blocks of the section are one transcript, in document order"
    );
    assert_eq!(
        transcripts[0].steps[1].output, "## principles\n\n- one",
        "a rendered heading stays output, not a section"
    );
    assert_eq!(transcripts[1].heading, "Next section");
}

// ── the README's diagram ────────────────────────────────────────────────────

/// The bodies of the fences whose info string is exactly `info`, with the `## `
/// heading each sits under. The same heading rule as [`transcripts`], so a
/// diagram and its section's commands can be matched by name.
fn fenced_with_heading(readme: &str, info: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut heading = String::new();
    let mut open: Option<String> = None;
    let mut body = String::new();

    for line in readme.lines() {
        let trimmed = line.trim();

        if let Some(kind) = open.take() {
            if trimmed.starts_with("```") {
                if kind == info {
                    found.push((heading.clone(), body.clone()));
                }

                continue;
            }

            body.push_str(line);
            body.push('\n');
            open = Some(kind);

            continue;
        }

        if trimmed.starts_with("```") {
            open = Some(trimmed.trim_start_matches('`').to_owned());
            body.clear();

            continue;
        }

        if let Some(text) = trimmed.strip_prefix("## ") {
            heading = text.trim().to_owned();
        }
    }

    found
}

/// Mermaid reads `#` as the start of an entity code, so it is escaped first: the
/// codes this emits would otherwise be escaped again.
fn escaped(label: &str) -> String {
    label.replace('#', "#35;").replace('"', "#quot;")
}

/// An edge label is delimited by `|`, which its quoted sibling tolerates — the
/// reason the exporter needs two escape functions and not one.
fn escaped_pipe(label: &str) -> String {
    escaped(label).replace('|', "#124;")
}

/// The projection the README's `jq` program performs, in Rust: generated symbols
/// in result order, a title for each label, and mermaid's two escape contexts.
///
/// This is the pin; the `jq` block is the illustration. Where `jq` is installed
/// the test runs that program too and requires the two to agree, so a program
/// that stopped drawing what the README says it draws fails here as well.
fn project(envelope: &Value) -> String {
    use std::collections::BTreeMap;

    let objects = envelope["objects"]
        .as_array()
        .unwrap_or_else(|| panic!("an envelope carries objects: {envelope}"));
    let symbol: BTreeMap<&str, String> = objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            let id = object["smut_id"]
                .as_str()
                .unwrap_or_else(|| panic!("a returned object carries its id: {object}"));

            (id, format!("n{index}"))
        })
        .collect();

    let mut lines = vec!["flowchart LR".to_owned()];

    for object in objects {
        let id = object["smut_id"].as_str().expect("checked above");
        let label = object["title"].as_str().unwrap_or(id);

        lines.push(format!("  {}[\"{}\"]", symbol[id], escaped(label)));
    }

    let edges = envelope["traversal"]["edges"]
        .as_array()
        .unwrap_or_else(|| panic!("a graph envelope carries edges: {envelope}"));

    for edge in edges {
        let field = edge["field"].as_str().expect("an edge names its field");

        lines.push(format!(
            "  {} -->|{}| {}",
            symbol[edge["source"].as_str().expect("an edge names its source")],
            escaped_pipe(field),
            symbol[edge["target"].as_str().expect("an edge names its target")]
        ));
    }

    lines.join("\n")
}

/// The README's own exporter run over `envelope`; `None` when `jq` is not
/// installed, which leaves the diagram pinned but the illustration unrun.
fn readme_exporter(envelope: &Value) -> Option<String> {
    let program = fenced_with_heading(README, "jq").first()?.1.clone();

    let mut child = Command::new("jq")
        .arg("-r")
        .arg(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdin = child.stdin.take()?;
    stdin.write_all(envelope.to_string().as_bytes()).ok()?;
    drop(stdin);

    let output = child.wait_with_output().ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// A diagram is the easiest example in this file to drift, because a wrong one
/// still renders. CHR-001's lesson was that an example nothing executes rots, so
/// the drawing is pinned to the envelope recorded above it rather than to the pen
/// that drew it — and, where `jq` is installed, to the README's program as well.
#[test]
fn the_mermaid_diagram_is_the_projection_of_the_recorded_envelope() {
    let diagrams = fenced_with_heading(README, "mermaid");
    assert_eq!(diagrams.len(), 1, "the README draws one graph");

    let (heading, diagram) = &diagrams[0];
    let section = transcripts(README)
        .into_iter()
        .find(|transcript| &transcript.heading == heading)
        .unwrap_or_else(|| panic!("the diagram's section `{heading}` carries commands"));
    let recorded = section
        .steps
        .last()
        .expect("a transcript has steps")
        .output
        .clone();
    let envelope: Value = serde_json::from_str(&recorded)
        .unwrap_or_else(|_| panic!("the last reply is a graph envelope: {recorded}"));

    let projected = project(&envelope);

    assert_eq!(
        diagram.trim_end(),
        projected,
        "the diagram is the projection of the last reply in its own section"
    );

    if let Some(from_jq) = readme_exporter(&envelope) {
        assert_eq!(
            from_jq.trim_end(),
            projected,
            "the README's jq program and this test's projection have drifted"
        );
    }
}

/// The README's exporter claims *two* escape contexts and its worked example now
/// reaches one of them (`#`). A claim about escaping that no example reaches is a
/// claim nothing checks, so a synthetic envelope reaches all three characters and
/// the README's own program has to agree with the projection about each.
#[test]
fn the_readme_exporter_escapes_both_label_contexts() {
    let envelope = json!({
        "objects": [
            {"smut_id": "a", "title": "A \"quoted\" #1"},
            {"smut_id": "b"}
        ],
        "traversal": {"edges": [{"field": "x|y", "source": "a", "target": "b"}]}
    });

    let projected = project(&envelope);

    assert!(
        projected.contains(r#"n0["A #quot;quoted#quot; #35;1"]"#),
        "a node label escapes `\"` and `#`, `#` first: {projected}"
    );
    assert!(
        projected.contains(r#"n1["b"]"#),
        "an object with no title falls back to its id: {projected}"
    );
    assert!(
        projected.contains("-->|x#124;y|"),
        "an edge label escapes the delimiter its sibling tolerates: {projected}"
    );

    if let Some(from_jq) = readme_exporter(&envelope) {
        assert_eq!(
            from_jq.trim_end(),
            projected,
            "the README's jq program escapes differently from the projection"
        );
    }
}
