//! The integration tests' one harness: a self-removing scratch directory, and the
//! one way to run the binary in it (CHR-007).
//!
//! Each test file is its own crate and compiles this module with `mod common;`,
//! using whichever part it needs, hence the `dead_code` allowance. A file adds
//! its own fixtures as an `impl Scratch` block beside its tests.
#![allow(dead_code)]

use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;
use smut::declare::Entities;
use smut::entity::{Form, serialise};
use smut::id::Id;
use smut::spec;
use smut::store::Store;
use smut::store::index::Index;

/// A scratch working directory holding at most one store, removed when dropped.
pub struct Scratch {
    path: PathBuf,
}

impl Scratch {
    /// A fresh directory named for the test file, the process and a counter.
    ///
    /// Canonicalized, because the binary reports resolved paths and macOS's temp
    /// dir sits behind a symlink (`/var` -> `/private/var`; STD-003).
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-{}-{}-{}",
            env!("CARGO_CRATE_NAME"),
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a scratch directory");
        let path = path.canonicalize().expect("a resolvable scratch directory");

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The `.smut` directory a store in this scratch lives in.
    pub fn store_dir(&self) -> PathBuf {
        self.path.join(".smut")
    }

    /// The store in this scratch, opened through the library.
    pub fn store(&self) -> Store {
        Store::in_directory(&self.path)
    }

    pub fn has_store(&self) -> bool {
        self.store_dir().is_dir()
    }

    /// Every entry under `.smut/`, sorted; none when there is no store.
    pub fn store_entries(&self) -> Vec<String> {
        listing(&self.store_dir())
    }

    /// The `$HOME` every run sees. Not created: a fixture that needs a layer
    /// under it creates that layer.
    pub fn home(&self) -> PathBuf {
        self.path.join("home")
    }

    /// The `$XDG_CONFIG_HOME` every run sees, likewise not created.
    pub fn config_home(&self) -> PathBuf {
        self.path.join("config-home")
    }

    /// The binary, ready to run in `cwd`, with its streams piped.
    ///
    /// Both layer roots are set and neither is removed: `HOME` and
    /// `XDG_CONFIG_HOME` unset would not mean "no user layer", because
    /// `std::env::home_dir()` falls back to the passwd entry and the binary would
    /// then read the developer's own sets. This is the only place a run's
    /// environment is spelled.
    pub fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_smut"));
        command
            .current_dir(cwd)
            .env("HOME", self.home())
            .env("XDG_CONFIG_HOME", self.config_home())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        command
    }

    /// Runs the binary in `cwd` to completion, with `stdin` if a fixture
    /// supplies one.
    pub fn run_in<A: AsRef<OsStr>>(&self, cwd: &Path, arguments: &[A], stdin: Option<&str>) -> Ran {
        let mut child = self
            .command(cwd)
            .args(arguments)
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

    /// Runs the binary in `cwd` with no stdin fixture at all.
    pub fn smut_in<A: AsRef<OsStr>>(&self, cwd: &Path, arguments: &[A]) -> Ran {
        self.run_in(cwd, arguments, None)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Runs the binary in `scratch`'s root, with `stdin` if a fixture supplies one.
pub fn run<A: AsRef<OsStr>>(scratch: &Scratch, arguments: &[A], stdin: Option<&str>) -> Ran {
    scratch.run_in(scratch.path(), arguments, stdin)
}

/// Runs the binary in `scratch`'s root with no stdin fixture at all.
pub fn smut<A: AsRef<OsStr>>(scratch: &Scratch, arguments: &[A]) -> Ran {
    run(scratch, arguments, None)
}

/// One invocation of the binary, as a caller sees it.
pub struct Ran {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|error| panic!("stdout is one JSON value ({error}): {:?}", self.stdout))
    }

    /// A successful machine command: one compact JSON line on stdout, exit 0,
    /// nothing on stderr.
    pub fn assert_one_json_line(&self, what: &str) {
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

/// Every entry in `directory`, sorted; none when it does not exist.
pub fn listing(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .map(|entries| {
            entries
                .map(|entry| {
                    entry
                        .expect("a directory entry")
                        .file_name()
                        .to_string_lossy()
                        .into_owned()
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();

    names
}

/// An in-memory store of `entities`, keyed by each one's `smut_id`.
pub fn store(entities: &[Value]) -> Entities {
    entities
        .iter()
        .map(|entity| {
            let id = entity["smut_id"]
                .as_str()
                .expect("every fixture names its smut_id");
            (Id::new(id), entity.clone())
        })
        .collect()
}

/// A JSON object's top-level key set, so a conditional member cannot hide.
pub fn keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect()
}

/// A specification the fixture asserts is valid.
pub fn parse(input: Value) -> spec::Spec {
    spec::parse(&input).expect("the fixture specification is valid")
}

/// An entity with an id and a body, and nothing else.
pub fn entity(id: &str, body: &str) -> Value {
    json!({ "smut_id": id, "body": body })
}

/// An entity's file bytes, exactly as the store writes them.
pub fn entity_bytes(value: &Value) -> Vec<u8> {
    serialise(value, Form::File).into_bytes()
}

/// An index's bytes mapping each id to its basename, as the store writes them.
pub fn index_bytes(objects: &[(&str, &str)]) -> Vec<u8> {
    let mut index = Index::empty();

    for (id, basename) in objects {
        index
            .insert(Id::new(*id), (*basename).to_owned())
            .expect("a safe basename");
    }

    index.to_bytes().into_bytes()
}
