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
//! names. The guide's own example constants are executed here as well, so the
//! shipped recipe and this suite cannot drift (PHASE-06, EX-5 / VT-2).

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

use smut::learn::{EXAMPLE_FILTER, EXAMPLE_FOLLOW, EXAMPLE_TRAVERSE};

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

    /// The ids a `select: ["smut_id"]` query returned, in result order.
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

/// The ids a `select: ["smut_id"]` query returned, in result order.
fn ids_of(envelope: &Value) -> Vec<String> {
    envelope["objects"]
        .as_array()
        .unwrap_or_else(|| panic!("a query envelope carries objects: {envelope}"))
        .iter()
        .map(|object| {
            object["smut_id"]
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
    json!({"from": ["session"], "backlinks": ["part-of"], "depth": 2, "select": ["smut_id"]})
}

/// Every reference target in one stored entity's fields: the entity's `smut_id`
/// is its name, not a reference to itself.
fn targets_of(entity: &Value) -> BTreeSet<&str> {
    let mut found = BTreeSet::new();

    if let Some(fields) = entity.as_object() {
        for (label, child) in fields {
            if label != "smut_id" {
                collect_references(child, &mut found);
            }
        }
    }

    found
}

/// Adds every reference target under `value` to `found`.
///
/// A reference is an object carrying a string `smut_id`, at any depth; an object
/// that is itself a reference is not descended into, exactly as
/// `entity::scan_refs` treats one.
fn collect_references<'a>(value: &'a Value, found: &mut BTreeSet<&'a str>) {
    match value {
        Value::Object(fields) => match fields.get("smut_id") {
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
    let envelope = declared.query(&json!({"limit": 400}));

    assert_eq!(
        envelope["truncated"],
        json!(false),
        "the whole store fits under the requested limit: {envelope}"
    );
    // The round trip is structural, not arithmetic: a normalisation that dropped
    // a field, coerced a value or reordered an array inside one entity would
    // leave a count assertion green while falsifying PROVENANCE's claim. The
    // query's default order is ascending id and the committed array is sorted by
    // id, so the two sides align without further sorting.
    assert_eq!(
        &envelope["objects"],
        &Value::Array(declared.entities.clone()),
        "the store reads back the committed array entity for entity"
    );

    // And the fixture's own reference walk is held to the crate's. This is a
    // regression guard, not independent evidence about this data: both walks
    // encode the same rule and collapse to unordered target sets, and the
    // committed array's references are flat `[{"smut_id": ...}, ...]` lists, a shape on
    // which the two definitions cannot differ. It would catch either algorithm
    // changing, which is what it is for (RV-020 F-6, correcting RV-019 F-2's
    // overstated disposition).
    for entity in &declared.entities {
        let mine = targets_of(entity);
        let theirs: BTreeSet<&str> = smut::entity::scan_refs(entity)
            .map(|(_, target)| target)
            .collect();
        assert_eq!(
            mine, theirs,
            "the fixture's reference walk agrees with `entity::scan_refs` on {}:
             two definitions of a reference would make these properties vacuous",
            entity["smut_id"]
        );
    }
}

#[test]
fn the_array_holds_entities_nothing_points_at_including_a_focus() {
    let declared = Declared::new();

    let targets: BTreeSet<&str> = declared.entities.iter().flat_map(targets_of).collect();
    let unreferenced: BTreeSet<&str> = declared
        .entities
        .iter()
        .filter_map(|entity| entity["smut_id"].as_str())
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

    let mut clusters = declared.ids(&json!({"where": {"type": "cluster"}, "select": ["smut_id"]}));
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

    let retracted = declared.ids(&json!({"where": {"status": "retracted"}, "select": ["smut_id"]}));

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
        "where": {"smut_id": {"$in": reached}, "status": "open"},
        "select": ["smut_id"],
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
// ── the shipped recipe, executed (PHASE-06, EX-5 / VT-2) ────────────────────

/// The eight open questions §13.1 names, in id order: what "which open
/// questions hang off this session" has to answer.
const OPEN_QUESTIONS: [&str; 8] = [
    "q-audit-cost",
    "q-channel-fix",
    "q-fix-order",
    "q-impact-query",
    "q-more-friction",
    "q-promotion",
    "q-split",
    "q-variance-mechanism",
];

/// What a guarded composition concludes from one envelope.
enum Guarded {
    /// Both fields agree the envelope is whole, so its ids are the answer.
    Complete(Vec<String>),
    /// The answer is a statement about the ids received, and it carries every
    /// reason the envelope gave for not being whole.
    Partial {
        ids: Vec<String>,
        reasons: Vec<String>,
    },
}

/// The completeness guard the guide states as part of the recipe, clause by
/// clause: `truncated` and `truncation_reasons` are read **before** any id is
/// extracted, so a truncated envelope can never be reported as a whole answer.
///
/// It is client-side: nothing here is passed back to smut, and smut is never
/// told what the first call said (§9.4, invariant 13).
fn guarded(envelope: &Value) -> Guarded {
    let truncated = envelope["truncated"]
        .as_bool()
        .unwrap_or_else(|| panic!("the envelope says whether it was cut: {envelope}"));
    let reasons: Vec<String> = envelope["truncation_reasons"]
        .as_array()
        .unwrap_or_else(|| panic!("the envelope names its truncation reasons: {envelope}"))
        .iter()
        .map(|reason| {
            reason
                .as_str()
                .unwrap_or_else(|| panic!("a truncation reason is a string: {envelope}"))
                .to_owned()
        })
        .collect();

    let ids = ids_of(envelope);

    if truncated {
        assert!(
            !reasons.is_empty(),
            "a truncated envelope names a reason, so the guard never has to guess: {envelope}"
        );
        Guarded::Partial { ids, reasons }
    } else {
        Guarded::Complete(ids)
    }
}

/// Parses one of the guide's example constants into the request value a query
/// takes, so the request tested is the request printed.
fn example(text: &str) -> Value {
    serde_json::from_str(text)
        .unwrap_or_else(|error| panic!("the guide's example is JSON ({error}): {text}"))
}

/// The recipe's second call, with the ids from the first reply substituted for
/// the guide's `IDS` placeholder — the one thing the guide leaves to the caller.
fn filter_over(ids: &[String]) -> Value {
    let ids = serde_json::to_string(ids).expect("ids serialise as a JSON array");
    example(&EXAMPLE_FILTER.replace("IDS", &ids))
}

#[test]
fn the_guide_prints_the_example_this_suite_executes() {
    let declared = Declared::new();
    let printed = smut(&declared.scratch, &["learn"]).stdout;

    for printed_example in [EXAMPLE_TRAVERSE, EXAMPLE_FOLLOW, EXAMPLE_FILTER] {
        assert!(
            printed.contains(printed_example),
            "smut learn prints the example verbatim, so the shipped recipe is the tested one: {printed_example}"
        );
    }
}

#[test]
fn the_shipped_example_reaches_the_eight_open_questions_in_two_calls() {
    let declared = Declared::new();

    let first = declared.query(&example(EXAMPLE_TRAVERSE));
    let Guarded::Complete(reached) = guarded(&first) else {
        panic!("the acceptance graph fits under the default cap: {first}");
    };
    assert!(
        reached.contains(&"session".to_owned()),
        "the root is a returned id: {reached:?}"
    );

    let second = declared.query(&filter_over(&reached));
    let Guarded::Complete(open) = guarded(&second) else {
        panic!("the filtered call is whole: {second}");
    };

    assert_eq!(
        open,
        OPEN_QUESTIONS.map(String::from),
        "the shipped example's two calls answer with the eight open questions, in id order"
    );
}

#[test]
fn the_follow_variant_of_the_shipped_example_reaches_none_of_the_questions() {
    let declared = Declared::new();
    let envelope = declared.query(&example(EXAMPLE_FOLLOW));

    let Guarded::Complete(ids) = guarded(&envelope) else {
        panic!("the wrong direction is a complete walk of nothing: {envelope}");
    };
    assert_eq!(
        ids,
        ["session".to_owned()],
        "follow on part-of from the session returns the session alone: {envelope}"
    );
    assert!(
        OPEN_QUESTIONS
            .iter()
            .all(|question| !ids.contains(&question.to_string())),
        "the same request in the outgoing direction answers a different question: {ids:?}"
    );
}

#[test]
fn the_completeness_guard_reads_both_reasons_before_it_extracts_any_id() {
    // Both reasons the envelope can carry, on the first response where they
    // matter: the guard names each one rather than reporting a whole answer.
    for reason in ["result_limit", "traversal_budget"] {
        let first = json!({
            "objects": [{"smut_id": "q-split"}],
            "missing": [],
            "dangling": [],
            "truncated": true,
            "truncation_reasons": [reason],
            "traversal": {"depths": {"q-split": 0}, "edges": [], "frontier": []},
        });

        match guarded(&first) {
            Guarded::Partial { ids, reasons } => {
                assert_eq!(
                    ids,
                    ["q-split".to_owned()],
                    "the ids stay readable beside the reason: {first}"
                );
                assert_eq!(
                    reasons,
                    [reason.to_owned()],
                    "the guard carries the reason it read: {first}"
                );
            }
            Guarded::Complete(_) => {
                panic!("a truncated first response is not a whole answer: {first}")
            }
        }
    }

    // An envelope is whole only when both fields say so, and no reason is
    // manufactured when nothing was cut.
    let whole = json!({
        "objects": [{"smut_id": "q-split"}],
        "missing": [],
        "dangling": [],
        "truncated": false,
        "truncation_reasons": [],
        "traversal": {"depths": {"q-split": 0}, "edges": [], "frontier": []},
    });
    match guarded(&whole) {
        Guarded::Complete(ids) => assert_eq!(ids, ["q-split".to_owned()]),
        Guarded::Partial { reasons, .. } => {
            panic!("nothing was cut, so nothing is named: {reasons:?}")
        }
    }
}

#[test]
fn a_complete_looking_second_envelope_is_no_evidence_about_the_first() {
    let declared = Declared::new();

    // A cap far below what the walk reaches: the first response is cut, and the
    // questions it did not return are invisible to the second call.
    let capped = declared.query(&json!({
        "from": ["session"],
        "backlinks": ["part-of"],
        "depth": 2,
        "select": ["smut_id"],
        "limit": 3,
    }));
    let Guarded::Partial { ids, reasons } = guarded(&capped) else {
        panic!("three ids cannot hold the whole walk: {capped}");
    };
    assert_eq!(reasons, ["result_limit".to_owned()]);

    let second = declared.query(&filter_over(&ids));
    assert_eq!(
        second["truncated"],
        json!(false),
        "the second call sees only what it was given, so it is whole: {second}"
    );
    assert!(
        OPEN_QUESTIONS
            .iter()
            .all(|question| !ids_of(&second).contains(&question.to_string())),
        "a caller trusting the second envelope alone answers 'no open questions': {second}"
    );
}

// ── §13.3 scenario 6: the over-cap composition (PHASE-07 EX-2, VT-1) ────────
//
// The committed fixture is 66 entities, comfortably under the default result
// cap, so it cannot produce the state scenario 6 names. This graph is declared
// into its own scratch store instead: one session holding 120 entities by
// `part-of`, every one of them `resolved`, and a single open question whose id
// sorts after all of them. The cap therefore cuts exactly the entity the
// question is about, and the second call — the guide's own filter — has nothing
// left to match.

/// The graph above the default cap: `session`, 120 resolved answers, and one
/// open question that sorts past them (`answer-000` … `answer-119` < `z-`).
fn over_cap_graph() -> Vec<Value> {
    let mut entities = vec![json!({"smut_id": "session", "type": "session"})];

    for number in 0..120 {
        entities.push(json!({
            "smut_id": format!("answer-{number:03}"),
            "type": "answer",
            "status": "resolved",
            "part-of": [{"smut_id": "session"}],
        }));
    }

    entities.push(json!({
        "smut_id": "z-question",
        "type": "question",
        "status": "open",
        "title": "The one open question the cap cuts",
        "part-of": [{"smut_id": "session"}],
    }));

    entities
}

/// Runs `smut query` with `request` against `scratch` and returns the envelope.
fn query(scratch: &Scratch, request: &Value) -> Value {
    let argument = request.to_string();
    let ran = smut(scratch, &["query", &argument]);
    assert_eq!(ran.code, 0, "query {argument}: {}", ran.stderr);

    ran.json()
}

/// **The wrong answer, kept in the suite on purpose.** A composition that reads
/// the ids out of the first envelope, ignores `truncated` and
/// `truncation_reasons`, and treats whatever the second call returns as the whole
/// truth. It looks like the recipe — it is the recipe with its guard removed —
/// and on an over-cap graph it answers "no open questions" about a store that
/// holds one. §9.4's guard exists because this helper is plausible.
fn naive_open_questions(scratch: &Scratch, first: &Value) -> Vec<String> {
    ids_of(&query(scratch, &filter_over(&ids_of(first))))
}

#[test]
fn an_over_cap_graph_makes_the_naive_composition_answer_no_open_questions() {
    let scratch = Scratch::new();
    let graph = over_cap_graph();
    let text = serde_json::to_string(&graph).expect("the graph serialises as one array");
    let declared = run(&scratch, &["declare", "-"], Some(&text));
    assert_eq!(declared.code, 0, "declaring the graph: {}", declared.stderr);

    // The target exists and is open: the trap is a real omission, not an empty
    // store answering honestly.
    let stored = smut(&scratch, &["get", "z-question"]);
    assert_eq!(stored.code, 0, "{}", stored.stderr);
    assert_eq!(stored.json()["objects"][0]["status"], json!("open"));

    // First call: the shipped first call, unchanged, over a graph above the cap.
    let first = query(
        &scratch,
        &json!({"from": ["session"], "backlinks": ["part-of"], "select": ["smut_id"]}),
    );
    let received = ids_of(&first);

    assert_eq!(
        first["truncated"],
        json!(true),
        "121 discoveries cannot fit the default cap: {first}"
    );
    assert_eq!(
        first["truncation_reasons"],
        json!(["result_limit"]),
        "the only cut is the result limit: {first}"
    );
    assert_eq!(
        received.len(),
        100,
        "the cap is a real cut, not an incidental one: {first}"
    );
    assert!(
        !received.contains(&"z-question".to_owned()),
        "the one open question sorts past the cap, so it is not returned: {first}"
    );
    assert!(
        received.len() < graph.len(),
        "the first call returned less than the store holds: {} of {}",
        received.len(),
        graph.len()
    );

    // The trap the guard exists for: the second envelope, over exactly the ids
    // the first returned, is complete-looking and empty.
    let second = query(&scratch, &filter_over(&received));
    assert_eq!(
        second["truncated"],
        json!(false),
        "the second call sees only what it was given, so it is whole: {second}"
    );
    assert_eq!(
        second["truncation_reasons"],
        json!([]),
        "and it has nothing to report: {second}"
    );
    assert_eq!(
        second["objects"],
        json!([]),
        "every id it was given is resolved: {second}"
    );

    // The naive composition therefore concludes "no open questions" …
    let naive = naive_open_questions(&scratch, &first);
    assert_eq!(
        naive,
        Vec::<String>::new(),
        "the naive helper answers 'no open questions': {naive:?}"
    );

    // … while the store does hold one, and the guarded conclusion says so by
    // naming the bound that hid it. Both halves of the comparison are asserted:
    // a future repair that makes the naive helper right, or the guard silent,
    // fails here.
    let truth = ids_of(&query(
        &scratch,
        &json!({"where": {"type": "question", "status": "open"}, "select": ["smut_id"]}),
    ));
    assert_eq!(
        truth,
        ["z-question".to_owned()],
        "the store's answer, asked directly"
    );

    match guarded(&first) {
        Guarded::Partial { ids, reasons } => {
            assert_eq!(ids, received, "the guard keeps the ids it read: {first}");
            assert_eq!(
                reasons,
                ["result_limit".to_owned()],
                "the guard names what cut the answer: {first}"
            );
        }
        Guarded::Complete(_) => panic!("a truncated first response is not whole: {first}"),
    }

    assert_ne!(
        naive, truth,
        "the naive helper and the store disagree, which is the whole point: {naive:?} vs {truth:?}"
    );

    // The same graph, cut by a budget instead of the cap. `depth: 2` is what
    // makes the budget bite at all: at the default depth the 121 discoveries sit
    // on the boundary and no expansion of theirs is permitted, so declining one
    // would be a depth selection rather than truncation. Discovery stops here,
    // so both reasons the envelope can carry are present, in the order they can
    // occur — and the naive helper is no less wrong for it.
    let bounded = query(
        &scratch,
        &json!({
            "from": ["session"],
            "backlinks": ["part-of"],
            "select": ["smut_id"],
            "depth": 2,
            "budget": 3,
        }),
    );

    assert_eq!(bounded["truncated"], json!(true), "{bounded}");
    assert_eq!(
        bounded["truncation_reasons"],
        json!(["traversal_budget", "result_limit"]),
        "the walk stopped, and the ordered rows were still cut: {bounded}"
    );

    let frontier = bounded["traversal"]["frontier"]
        .as_array()
        .unwrap_or_else(|| panic!("the walk reports its frontier: {bounded}"));
    assert_eq!(
        frontier.len(),
        119,
        "one root expanded plus two answers, so 119 of the 121 discoveries were left unexpanded: {bounded}"
    );
    assert!(
        frontier.contains(&json!("z-question")),
        "the question the walk never reached is squarely on the frontier: {bounded}"
    );

    let naive_bounded = naive_open_questions(&scratch, &bounded);
    assert_eq!(
        naive_bounded,
        Vec::<String>::new(),
        "the naive helper answers 'no open questions' here too: {naive_bounded:?}"
    );
    match guarded(&bounded) {
        Guarded::Partial { reasons, .. } => assert_eq!(
            reasons,
            ["traversal_budget".to_owned(), "result_limit".to_owned()],
            "the guarded conclusion names both bounds rather than reporting a whole answer: {bounded}"
        ),
        Guarded::Complete(_) => {
            panic!("a budget-truncated walk is not apparent completeness: {bounded}")
        }
    }
}
