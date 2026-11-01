//! The acceptance fixture: the first-use store's entity array, declared into a
//! scratch store and queried through the binary (SL-002 PHASE-04, EX-3 / VT-1).
//!
//! `tests/fixtures/interview/entities.json` is one committed, normalised array
//! of the first-use store's entities, and the only copy of that data in the
//! repository (DEC-018). [`Declared`] reads it, declares it into an empty scratch
//! directory with `smut declare -`, and hands back both the store and the parsed
//! array, so every test derives its facts from the data instead of restating
//! them. Assertions are on entities and never on the store's vocabulary prose:
//! the `vocab` entity is stale in two directions, which is exactly why the
//! evidence is read from the entities themselves (design §2).
//!
//! Every invocation is a real process run in its own scratch directory, as
//! `tests/cli_contract.rs` keeps it, so the store found is the one `$PWD/.smut`
//! names.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

/// The committed array, relative to the crate root. It is the only data source
/// these tests read: no store internals, no second copy.
const FIXTURE: &str = "tests/fixtures/interview/entities.json";

/// The five clusters §13.3 scenario 3 walks, in the array's id order.
const CLUSTERS: [&str; 5] = [
    "cluster-friction",
    "cluster-mind",
    "cluster-schema",
    "cluster-setup",
    "cluster-thesis",
];

// ── the process harness, copied from `tests/cli_contract.rs` ─────────────────

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-fixture-{}-{}",
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

// ── the fixture, declared ───────────────────────────────────────────────────

/// A scratch store with the committed array declared into it, plus the array it
/// was declared from.
struct Declared {
    scratch: Scratch,
    /// The array as `entities.json` holds it, in file order.
    entities: Vec<Value>,
}

impl Declared {
    /// Reads the committed array, declares it into a fresh scratch directory and
    /// checks that the declaration wrote one entry per declared entity.
    ///
    /// A missing or unparsable file is the one reason this suite is allowed to
    /// fail: the array is generated and committed before the properties below can
    /// hold.
    fn new() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "the committed fixture {} is unreadable: {error}",
                path.display()
            )
        });
        let entities = match serde_json::from_str::<Value>(&text).unwrap_or_else(|error| {
            panic!(
                "the committed fixture {} is one JSON value: {error}",
                path.display()
            )
        }) {
            Value::Array(entities) => entities,
            other => panic!(
                "the committed fixture {} is one JSON array, found {other}",
                path.display()
            ),
        };

        let scratch = Scratch::new();
        let declared = run(&scratch, &["declare", "-"], Some(&text));
        assert_eq!(declared.code, 0, "declaring {FIXTURE}: {}", declared.stderr);

        let reply = declared.json();
        let ids = reply["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("a declaration reply carries the ids it wrote: {reply}"));
        assert_eq!(
            ids.len(),
            entities.len(),
            "one written id per declared entity: {reply}"
        );

        Self { scratch, entities }
    }

    /// Runs `smut query` with `request` against the declared store and returns
    /// the whole envelope; a refusal is a test failure, not a branch.
    fn query(&self, request: &Value) -> Value {
        let argument = request.to_string();
        let ran = smut(&self.scratch, &["query", &argument]);
        assert_eq!(ran.code, 0, "query {argument}: {}", ran.stderr);

        ran.json()
    }

    /// The ids a `select: ["id"]` query returned, in result order.
    fn ids(&self, request: &Value) -> Vec<String> {
        ids_of(&self.query(request))
    }

    /// One stored entity, whole, as the store reads it back.
    fn entity(&self, id: &str) -> Value {
        let envelope = self.query(&json!({"from": [id]}));
        let objects = envelope["objects"]
            .as_array()
            .unwrap_or_else(|| panic!("a query envelope carries objects: {envelope}"));
        assert_eq!(objects.len(), 1, "the store holds {id}: {envelope}");

        objects[0].clone()
    }
}

/// The ids a `select: ["id"]` query returned, in result order.
fn ids_of(envelope: &Value) -> Vec<String> {
    envelope["objects"]
        .as_array()
        .unwrap_or_else(|| panic!("a query envelope carries objects: {envelope}"))
        .iter()
        .map(|object| {
            object["id"]
                .as_str()
                .unwrap_or_else(|| panic!("every returned object names its id: {object}"))
                .to_owned()
        })
        .collect()
}

/// The first call of the acceptance composition §13.3 scenario 2 names.
///
/// Containment points child → container — a question names the cluster it hangs
/// off, and a cluster names the session — so "which questions hang off this
/// session" is the set of incoming `part-of` references at depth 2: `backlinks`.
fn reached_from_session() -> Value {
    json!({"from": ["session"], "backlinks": ["part-of"], "depth": 2, "select": ["id"]})
}

/// Every reference target in one stored entity's fields: the entity's `id` is
/// its name, not a reference to itself.
fn targets_of(entity: &Value) -> BTreeSet<&str> {
    let mut found = BTreeSet::new();

    if let Some(fields) = entity.as_object() {
        for (label, child) in fields {
            if label != "id" {
                collect_references(child, &mut found);
            }
        }
    }

    found
}

/// Adds every reference target under `value` to `found`.
///
/// A reference is an object carrying a string `id`, at any depth; an object that
/// is itself a reference is not descended into, exactly as `entity::scan_refs`
/// treats one.
fn collect_references<'a>(value: &'a Value, found: &mut BTreeSet<&'a str>) {
    match value {
        Value::Object(fields) => match fields.get("id") {
            Some(Value::String(id)) => {
                found.insert(id);
            }
            _ => {
                for child in fields.values() {
                    collect_references(child, found);
                }
            }
        },
        Value::Array(items) => {
            for item in items {
                collect_references(item, found);
            }
        }
        _ => {}
    }
}

// ── the properties the acceptance case needs, asserted against entities ──────

#[test]
fn the_committed_array_declares_into_a_store_that_reads_back_the_same_entities() {
    let declared = Declared::new();

    assert!(
        !declared.entities.is_empty(),
        "{FIXTURE} is one non-empty array of entities"
    );

    // A limit above any plausible count, so the read-back count is the store's
    // and not the default cap's.
    let envelope = declared.query(&json!({"limit": 400, "select": ["id"]}));

    assert_eq!(
        envelope["truncated"],
        json!(false),
        "the whole store fits under the requested limit: {envelope}"
    );
    assert_eq!(
        ids_of(&envelope).len(),
        declared.entities.len(),
        "the store reads back one entity per element of the committed array"
    );
}

#[test]
fn the_array_holds_entities_nothing_points_at_including_a_focus() {
    let declared = Declared::new();

    let targets: BTreeSet<&str> = declared.entities.iter().flat_map(targets_of).collect();
    let unreferenced: BTreeSet<&str> = declared
        .entities
        .iter()
        .filter_map(|entity| entity["id"].as_str())
        .filter(|id| !targets.contains(*id))
        .collect();

    assert!(
        !unreferenced.is_empty(),
        "at least one entity has no incoming reference: {} entities, {} reference targets",
        declared.entities.len(),
        targets.len()
    );
    assert!(
        unreferenced.contains("a-focus"),
        "a-focus is named as a reference by no other entity; unreferenced: {unreferenced:?}"
    );
}

#[test]
fn the_superseded_finding_is_retired_in_place_rather_than_removed() {
    let declared = Declared::new();
    let retired = declared.entity("finding-smut-16");

    assert_eq!(
        retired["status"],
        json!("superseded"),
        "finding-smut-16 is still stored and carries its superseded status: {retired}"
    );
}

#[test]
fn a_settled_finding_is_stored_with_its_own_type() {
    let declared = Declared::new();
    let settled = declared.entity("finding-smut-20");

    assert_eq!(
        settled["status"],
        json!("settled"),
        "finding-smut-20 is stored as settled: {settled}"
    );
    assert_eq!(
        settled["type"],
        json!("finding"),
        "finding-smut-20 keeps its own type: {settled}"
    );
}

#[test]
fn finding_smut_17_and_the_conventions_proposal_reference_each_other() {
    let declared = Declared::new();
    let finding = declared.entity("finding-smut-17");
    let proposal = declared.entity("prop-conventions-library");

    assert!(
        targets_of(&finding).contains("prop-conventions-library"),
        "finding-smut-17 references the conventions proposal: {finding}"
    );
    assert!(
        targets_of(&proposal).contains("finding-smut-17"),
        "the conventions proposal references finding-smut-17: {proposal}"
    );
}

#[test]
fn the_clusters_are_exactly_the_five_this_slice_walks() {
    let declared = Declared::new();

    let mut clusters = declared.ids(&json!({"where": {"type": "cluster"}, "select": ["id"]}));
    clusters.sort();

    assert_eq!(
        clusters,
        CLUSTERS.map(String::from),
        "the store's type-cluster entities are exactly the five the acceptance walk names"
    );
}

#[test]
fn no_entity_in_the_store_is_retracted() {
    let declared = Declared::new();

    let retracted = declared.ids(&json!({"where": {"status": "retracted"}, "select": ["id"]}));

    assert!(
        retracted.is_empty(),
        "the array holds no retracted record, and a status it never uses cannot match one: {retracted:?}"
    );
}

// ── the two-call acceptance query, asserted by id ───────────────────────────

#[test]
fn backlinks_on_part_of_at_depth_two_reaches_the_clusters_from_the_session() {
    let declared = Declared::new();
    let envelope = declared.query(&reached_from_session());

    assert_eq!(
        envelope["truncated"],
        json!(false),
        "the acceptance graph fits under the default cap: {envelope}"
    );

    let ids = ids_of(&envelope);
    assert!(
        !ids.is_empty(),
        "the walk returns the session and everything contained under it: {envelope}"
    );
    assert!(
        ids.contains(&"session".to_owned()),
        "the root is a returned id: {ids:?}"
    );

    let depths = &envelope["traversal"]["depths"];
    for cluster in CLUSTERS {
        assert_eq!(
            depths[cluster],
            json!(1),
            "{cluster} is one incoming part-of hop from the session: {envelope}"
        );
    }
}

#[test]
fn filtering_the_reached_ids_for_open_status_names_the_eight_questions() {
    let declared = Declared::new();

    let reached = ids_of(&declared.query(&reached_from_session()));
    let ids = declared.ids(&json!({
        "where": {"id": {"$in": reached}, "status": "open"},
        "select": ["id"],
    }));

    assert_eq!(
        ids,
        [
            "q-audit-cost",
            "q-channel-fix",
            "q-fix-order",
            "q-impact-query",
            "q-more-friction",
            "q-promotion",
            "q-split",
            "q-variance-mechanism",
        ]
        .map(String::from),
        "the reached ids, filtered to open questions, are exactly the eight in id order"
    );
}
