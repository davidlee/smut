//! The README's worked example, executed (PHASE-08 EX-4).
//!
//! `README.md` carries exactly one `console` block: the worked example. This
//! test reads that block out of the README, runs every command in it, in order,
//! in a scratch store, and compares what the binary printed with the transcript
//! beside it. Editing the README's transcript without re-running it fails here.
//!
//! **What "matches" means for JSON.** A recorded output that parses as JSON is
//! compared as a parsed value, not as bytes: whether an entity root inside an
//! envelope is id-first is an open review question, so a byte pin here would
//! pre-empt it. `render`'s Markdown is settled and is compared exactly. A
//! recorded output beginning `smut: ` is a diagnostic: that command must fail,
//! write nothing to stdout, and print exactly that line on stderr.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

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

#[test]
fn the_worked_example_ends_with_the_0_2_refusal() {
    let steps = worked_example();
    let last = steps.last().expect("the example has commands");

    assert!(
        last.command.contains("follow"),
        "the example stops at a 0.2 key rather than pretending to traverse: {}",
        last.command
    );
    assert!(
        last.output.contains("0.2"),
        "the refusal names the release: {}",
        last.output
    );
}
