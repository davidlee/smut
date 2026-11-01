//! The walker's determinism contract for SL-002 PHASE-01 (VT-2).
//!
//! Design §7.4's six determinism rules, pinned through the public surface — named
//! fields in query order, wildcard lexical order, array position, outgoing before
//! incoming, incoming sources in ID order, a repeated label collapsing without
//! reordering — plus one identical envelope from a store rebuilt so the same
//! entities live under shuffled filenames (the "stable across filesystem
//! iteration" rule).
//!
//! **This file is expected to be RED until the walker lands.** 0.1's
//! `spec::parse` refuses `follow`, `backlinks` and `depth` by name, so every walk
//! test stops at that refusal — `follow is not supported in 0.1; it arrives in
//! 0.2` — and never in a compile error. Once parsing accepts the keys, the same
//! tests fail on the envelope's missing `traversal` member until the walker
//! exists.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use smut::declare::Entities;
use smut::id::Id;
use smut::select::{envelope, get, select};
use smut::spec;

// ── the in-memory harness, as `tests/selection.rs` keeps it ──────────────────

fn store(entities: &[Value]) -> Entities {
    entities
        .iter()
        .map(|entity| {
            let id = entity["id"].as_str().expect("every fixture names its id");
            (Id::new(id), entity.clone())
        })
        .collect()
}

/// The envelope's `traversal` member for `input`, or the red this file expects:
/// first the parse refusal, then a missing member.
fn walked(entities: &Entities, input: Value) -> Value {
    let specification = spec::parse(&input)
        .unwrap_or_else(|reject| panic!("the walker specification is refused: {reject}"));
    let selection = select(entities, &specification)
        .unwrap_or_else(|reject| panic!("the walker selection is refused: {reject}"));
    let document = envelope(&selection);

    document
        .get("traversal")
        .cloned()
        .unwrap_or_else(|| panic!("the envelope carries no traversal member: {document}"))
}

/// Asserts the walk's frontier in discovery order, printing the whole traversal
/// member when it differs.
fn assert_frontier(traversal: &Value, expected: Value) {
    assert_eq!(traversal["frontier"], expected, "the walk: {traversal}");
}

/// One hub whose two reference fields are lexical `alpha`, `zeta` — the reverse
/// of the query order the named-field tests ask for.
fn hub() -> Entities {
    store(&[
        json!({"id": "hub", "alpha": {"id": "n-a"}, "zeta": {"id": "n-z"}}),
        json!({"id": "n-a"}),
        json!({"id": "n-z"}),
    ])
}

// ── §7.4 determinism rules, in memory ───────────────────────────────────────

#[test]
fn named_fields_beat_the_lexical_scan() {
    // §7.4 "Named fields in query order": iterate the requested labels, filter
    // `scan_refs`; never sort the requested list.
    //
    // The trap: `scan_refs` yields `alpha` first because it walks the entity's
    // map lexically, so the walker has to group by the requested label list
    // (`zeta`, then `alpha`) rather than trust the scan's order.
    let traversal = walked(
        &hub(),
        json!({"from": "hub", "follow": ["zeta", "alpha"], "depth": 1}),
    );

    assert_frontier(&traversal, json!(["n-z", "n-a"]));
}

#[test]
fn wildcard_follows_the_lexical_scan() {
    // §7.4 "Wildcard fields lexical": `follow: "*"` is `scan_refs` order.
    let traversal = walked(&hub(), json!({"from": "hub", "follow": "*", "depth": 1}));

    assert_frontier(&traversal, json!(["n-a", "n-z"]));
}

#[test]
fn array_references_keep_their_position() {
    // §7.4 "Array items preserve order": `scan_refs` walks arrays positionally,
    // so the discoveries follow the array, not the ids' lexical order.
    let entities = store(&[
        json!({"id": "array-root", "list": [{"id": "c"}, {"id": "a"}, {"id": "b"}]}),
        json!({"id": "a"}),
        json!({"id": "b"}),
        json!({"id": "c"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "array-root", "follow": ["list"], "depth": 1}),
    );

    assert_frontier(&traversal, json!(["c", "a", "b"]));
}

#[test]
fn outgoing_discoveries_precede_incoming_ones() {
    // §7.4 "Outgoing before incoming": `expand_outgoing` runs before
    // `expand_incoming`, and the queue records discovery, not interleaving.
    let entities = store(&[
        json!({"id": "r", "down": {"id": "child"}}),
        json!({"id": "child"}),
        json!({"id": "x1", "up": {"id": "r"}}),
        json!({"id": "x2", "up": {"id": "r"}}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "r", "follow": ["down"], "backlinks": ["up"], "depth": 1}),
    );

    assert_frontier(&traversal, json!(["child", "x1", "x2"]));
}

#[test]
fn incoming_sources_come_in_id_order() {
    // §7.4 "Incoming sources in ID order": the store is scanned in `Entities`
    // order, which is ID order — not the order the sources were declared in.
    let entities = store(&[
        json!({"id": "src-b", "up": {"id": "r"}}),
        json!({"id": "src-a", "up": {"id": "r"}}),
        json!({"id": "r"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "r", "backlinks": ["up"], "depth": 1}),
    );

    assert_frontier(&traversal, json!(["src-a", "src-b"]));
}

#[test]
fn a_repeated_label_collapses_without_reordering() {
    // §5 "duplicates behave as a set for edge selection but do not reorder the
    // first occurrence": one discovery, one edge, no second visit.
    let entities = store(&[
        json!({"id": "q-root", "q": {"id": "t"}}),
        json!({"id": "t"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "q-root", "follow": ["q", "q"], "depth": 1}),
    );

    assert_frontier(&traversal, json!(["t"]));
    assert_eq!(
        traversal["edges"],
        json!([{"source": "q-root", "field": "q", "target": "t"}]),
        "one label asked twice is still one edge: {traversal}"
    );
    assert_eq!(
        traversal["edges"].as_array().map(Vec::len),
        Some(1),
        "the edge list holds exactly one entry: {traversal}"
    );
}

// ── the CLI harness, as `tests/cli_contract.rs` keeps it ────────────────────

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-traversal-{}-{}",
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

/// A JSON string literal, so hand-written index bytes escape as the format does.
fn quoted(text: &str) -> String {
    serde_json::to_string(&Value::String(text.to_owned())).expect("a string serialises")
}

/// Declares four entities whose references make the induced edge list non-empty,
/// in two batches so the declaration order is fixed.
fn declare_four(scratch: &Scratch) {
    let first = smut(
        scratch,
        &[
            "declare",
            r#"[{"id":"alpha","title":"Alpha","next":{"id":"gamma"}},{"id":"beta","title":"Beta","next":[{"id":"delta"}]}]"#,
        ],
    );
    assert_eq!(first.code, 0, "{}", first.stderr);

    let second = smut(
        scratch,
        &[
            "declare",
            r#"[{"id":"gamma","title":"Gamma"},{"id":"delta","title":"Delta"}]"#,
        ],
    );
    assert_eq!(second.code, 0, "{}", second.stderr);
}

/// Rewrites the committed store so the same entities live under a reversed
/// permutation of their basenames, and the index's bytes name the ids in reverse
/// order as well.
fn shuffle_store(scratch: &Scratch) {
    let index_path = scratch.store().join("index.json");
    let index: Value =
        serde_json::from_str(&fs::read_to_string(&index_path).expect("the committed index"))
            .expect("the index is one JSON value");
    let objects = index["objects"]
        .as_object()
        .expect("the index maps ids to basenames");

    let ids: Vec<String> = objects.keys().cloned().collect();
    let held: Vec<String> = objects
        .values()
        .map(|basename| {
            basename
                .as_str()
                .expect("a basename is a string")
                .to_owned()
        })
        .collect();
    let shuffled: Vec<String> = held.iter().rev().cloned().collect();
    assert_eq!(ids.len(), 4, "the fixture declares four entities");
    assert_ne!(
        shuffled, held,
        "the fixture's filenames are really shuffled"
    );

    // Two phases: parking each file under a fresh name lets the permutation point
    // one file at another file's old basename without clobbering it.
    let parked: Vec<String> = (0..held.len())
        .map(|position| format!("parked-{position}.json"))
        .collect();
    for (old, park) in held.iter().zip(&parked) {
        fs::rename(scratch.store().join(old), scratch.store().join(park))
            .expect("an entity file takes a parking basename");
    }
    for (park, new) in parked.iter().zip(&shuffled) {
        fs::rename(scratch.store().join(park), scratch.store().join(new))
            .expect("an entity file takes its new basename");
    }

    // The entity that held `held[i]` (`ids[i]`) now lives in `shuffled[i]`, which
    // is the reversed permutation. The index's keys are written in reverse id
    // order, so the reader meets an order the writer would never emit.
    let mut rebuilt: Vec<(&String, &String)> = ids.iter().zip(&shuffled).collect();
    rebuilt.reverse();
    let entries: Vec<String> = rebuilt
        .iter()
        .map(|(id, basename)| format!("    {}: {}", quoted(id), quoted(basename)))
        .collect();
    fs::write(
        &index_path,
        format!(
            "{{\n  \"format_version\": 1,\n  \"objects\": {{\n{}\n  }}\n}}\n",
            entries.join(",\n")
        ),
    )
    .expect("the rebuilt index");

    let mut expected = vec!["index.json".to_owned(), "lock".to_owned()];
    expected.extend(shuffled.iter().cloned());
    expected.sort();
    assert_eq!(
        scratch.store_entries(),
        expected,
        "the rebuild renamed every entity file and left nothing behind"
    );
}

#[test]
fn a_store_rebuilt_with_shuffled_filenames_yields_an_identical_envelope() {
    // §7.4 "Stable across filesystem iteration": `Snapshot::load` iterates the
    // index's `BTreeMap`, so no filesystem iteration order reaches the result.
    let scratch = Scratch::new();
    declare_four(&scratch);
    assert!(scratch.has_store(), "the declarations created a store");

    let query = r#"{"follow":"*","depth":1}"#;
    let first = smut(&scratch, &["query", query]);
    assert_eq!(
        first.code, 0,
        "the walker accepts the traversal query: {}",
        first.stderr
    );
    let before = first.json();

    shuffle_store(&scratch);

    let second = smut(&scratch, &["query", query]);
    assert_eq!(
        second.code, 0,
        "the walker reads the rebuilt store: {}",
        second.stderr
    );
    assert_eq!(
        second.json(),
        before,
        "the envelope is the same whatever the filenames are"
    );
}

// ── §8 envelope and provenance (VT-3, VT-4) ────────────────────────────────

/// The five members every 0.1 envelope carries, in the lexical order a
/// `serde_json::Map` keeps.
const ENVELOPE_0_1_KEYS: [&str; 5] = [
    "dangling",
    "missing",
    "objects",
    "truncated",
    "truncation_reasons",
];

/// The whole envelope for `input`, through the same public pipeline `walked`
/// drives.
fn envelope_of(entities: &Entities, input: Value) -> Value {
    let specification = spec::parse(&input)
        .unwrap_or_else(|reject| panic!("the walker specification is refused: {reject}"));
    let selection = select(entities, &specification)
        .unwrap_or_else(|reject| panic!("the walker selection is refused: {reject}"));

    envelope(&selection)
}

/// The envelope's `traversal` member, or the red this file expects.
fn traversal_member(document: &Value) -> Value {
    document
        .get("traversal")
        .cloned()
        .unwrap_or_else(|| panic!("the envelope carries no traversal member: {document}"))
}

/// The envelope's member names, lexical because `serde_json::Map` is a
/// `BTreeMap`.
fn envelope_keys(document: &Value) -> Vec<&str> {
    document
        .as_object()
        .unwrap_or_else(|| panic!("an envelope is an object: {document}"))
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn follow_empty_still_supplies_the_traversal_member() {
    // §8.1 "including an empty array": `follow: []` is a supplied traversal that
    // follows nothing, so the member is present with an empty walk.
    let traversal = walked(&hub(), json!({"from": "hub", "follow": [], "depth": 1}));

    assert_eq!(
        traversal["depths"],
        json!({"hub": 0}),
        "the walk: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([]),
        "no returned pair is connected by a reference: {traversal}"
    );
    assert_frontier(&traversal, json!([]));
}

#[test]
fn follow_empty_keeps_the_induced_edges_among_returned_objects() {
    // §10.3 "`follow: []` … induced edges among the returned objects still
    // apply": `hub`'s `alpha` reference is an edge because `n-a` is a root, even
    // though no field was followed.
    let traversal = walked(
        &hub(),
        json!({"from": ["hub", "n-a"], "follow": [], "depth": 1}),
    );

    assert_eq!(
        traversal["edges"],
        json!([{"source": "hub", "field": "alpha", "target": "n-a"}]),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!([]));
}

#[test]
fn an_empty_from_with_a_traversal_request_has_an_empty_member() {
    // §10.3 "`from: []`": nothing is selected, but the supplied `follow` still
    // makes the member present with empty views.
    let document = envelope_of(&hub(), json!({"from": [], "follow": ["alpha"]}));
    let traversal = traversal_member(&document);

    assert_eq!(document["objects"], json!([]), "the envelope: {document}");
    assert_eq!(traversal["depths"], json!({}), "the walk: {traversal}");
    assert_eq!(traversal["edges"], json!([]), "the walk: {traversal}");
    assert_frontier(&traversal, json!([]));
}

#[test]
fn get_and_non_traversing_specs_keep_the_five_0_1_keys() {
    // §8.1 / §5 "Presence is recorded, not inferred": `traversal` is absent for
    // `get` and for every query whose keys are from/select/order/limit or an
    // inert `depth`, so the envelope keeps exactly 0.1's five keys.
    let entities = hub();

    let roots = [Id::new("hub")];
    let document = envelope(&get(&entities, &roots));
    assert!(
        document.get("traversal").is_none(),
        "get never traverses: {document}"
    );
    assert_eq!(
        envelope_keys(&document),
        ENVELOPE_0_1_KEYS.to_vec(),
        "get: {document}"
    );

    for input in [
        json!({}),
        json!({"from": "hub"}),
        json!({"from": "hub", "select": ["id"], "order": [{"field": "id"}], "limit": 2}),
        json!({"from": "hub", "depth": 2}),
    ] {
        let document = envelope_of(&entities, input.clone());

        assert!(
            document.get("traversal").is_none(),
            "{input} supplied no traversal key: {document}"
        );
        assert_eq!(
            envelope_keys(&document),
            ENVELOPE_0_1_KEYS.to_vec(),
            "the keys of {input}: {document}"
        );
    }
}

#[test]
fn depth_zero_makes_every_root_frontier_and_expands_nothing() {
    // §10.3 "`depth: 0`": roots only; the root is frontier and nothing is
    // discovered, so no reference of its own reaches the returned set.
    let traversal = walked(
        &hub(),
        json!({"from": "hub", "follow": ["alpha", "zeta"], "depth": 0}),
    );

    assert_eq!(
        traversal["depths"],
        json!({"hub": 0}),
        "the walk: {traversal}"
    );
    assert_eq!(traversal["edges"], json!([]), "the walk: {traversal}");
    assert_frontier(&traversal, json!(["hub"]));
}

#[test]
fn depth_one_puts_the_depth_one_discoveries_in_the_frontier() {
    // §10.3 "`depth: 1`": the root expands and its discoveries sit on the depth
    // boundary, unexpanded and in discovery order.
    let traversal = walked(
        &hub(),
        json!({"from": "hub", "follow": ["alpha", "zeta"], "depth": 1}),
    );

    assert_eq!(
        traversal["depths"],
        json!({"hub": 0, "n-a": 1, "n-z": 1}),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["n-a", "n-z"]));
}

#[test]
fn a_named_field_no_entity_carries_discovers_nothing_without_error() {
    // §10.3 "a named traversal field no entity carries": not an error, no
    // outgoing discoveries, and the induced edges among returned objects are
    // unaffected.
    let traversal = walked(
        &hub(),
        json!({"from": ["hub", "n-a"], "follow": ["nonexistent"], "depth": 1}),
    );

    assert_eq!(
        traversal["depths"],
        json!({"hub": 0, "n-a": 0}),
        "the walk: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "hub", "field": "alpha", "target": "n-a"}]),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!([]));
}

#[test]
fn depths_are_result_scoped_and_the_frontier_is_not_cut_by_the_limit() {
    // §8.2 / DEC-013: `depths` names returned ids only, while the frontier is
    // walk-scoped and therefore keeps every discovery the limit dropped.
    let entities = store(&[
        json!({"id": "root", "out": [{"id": "d1"}, {"id": "d2"}, {"id": "d3"}]}),
        json!({"id": "d1"}),
        json!({"id": "d2"}),
        json!({"id": "d3"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["out"], "depth": 1, "limit": 1}),
    );

    assert_eq!(
        document["objects"],
        json!([{"id": "root", "out": [{"id": "d1"}, {"id": "d2"}, {"id": "d3"}]}]),
        "the limit returned the root alone: {document}"
    );
    assert_eq!(
        document["truncated"],
        json!(true),
        "the envelope: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!(["result_limit"]),
        "a depth boundary is not a reason: {document}"
    );

    let traversal = traversal_member(&document);
    assert_eq!(
        traversal["depths"],
        json!({"root": 0}),
        "an omitted id takes no depth: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([]),
        "an omitted id takes no edge: {traversal}"
    );
    assert_frontier(&traversal, json!(["d1", "d2", "d3"]));
}

#[test]
fn dangling_dedupes_by_triple_in_encounter_order() {
    // §7.2 "Deduped by `(source, field, target)`" and §8.2 "in encounter order":
    // the third occurrence of `gone-a` adds nothing and reorders nothing.
    let entities = store(&[json!({
        "id": "root",
        "refs": [{"id": "gone-a"}, {"id": "gone-b"}, {"id": "gone-a"}]
    })]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["refs"], "depth": 1}),
    );

    assert_eq!(
        document["dangling"],
        json!([
            {"source": "root", "field": "refs", "target": "gone-a"},
            {"source": "root", "field": "refs", "target": "gone-b"},
        ]),
        "the envelope: {document}"
    );
}

#[test]
fn a_reference_in_an_unrequested_field_is_not_dangling() {
    // §7.2 "a reference in an unrequested field is not reported": only `down` is
    // expanded, so `other`'s absent target is silent.
    let entities = store(&[
        json!({"id": "root", "down": {"id": "kept"}, "other": {"id": "gone"}}),
        json!({"id": "kept"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["down"], "depth": 1}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(document["dangling"], json!([]), "the envelope: {document}");
    // The walk really did expand the requested field, so `[]` is silence about
    // `other` rather than a walk that never ran.
    assert_frontier(&traversal, json!(["kept"]));
}

#[test]
fn dangling_is_walk_scoped_and_not_cut_to_returned_sources() {
    // §8.2 "It is not cut to returned sources": `mid` is discovered and expanded
    // but omitted by `limit`, and its dangling reference is still reported.
    let entities = store(&[
        json!({"id": "r", "down": {"id": "mid"}}),
        json!({"id": "mid", "down": {"id": "gone"}}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "r", "follow": ["down"], "depth": 2, "limit": 1}),
    );

    assert_eq!(
        document["objects"],
        json!([{"id": "r", "down": {"id": "mid"}}]),
        "the limit returned the root alone: {document}"
    );
    assert_eq!(
        document["dangling"],
        json!([{"source": "mid", "field": "down", "target": "gone"}]),
        "the walk's finding outlives the limit: {document}"
    );
}

#[test]
fn a_dangling_target_that_is_also_a_missing_root_appears_in_both_lists() {
    // §10.3 "dangling target also an explicit missing root": `missing` records
    // the request, `dangling` records the walk, and neither suppresses the other.
    let entities = store(&[json!({"id": "hub", "alpha": {"id": "gone"}})]);

    let document = envelope_of(
        &entities,
        json!({"from": ["hub", "gone"], "follow": ["alpha"]}),
    );

    assert_eq!(
        document["missing"],
        json!(["gone"]),
        "the envelope: {document}"
    );
    assert_eq!(
        document["dangling"],
        json!([{"source": "hub", "field": "alpha", "target": "gone"}]),
        "the envelope: {document}"
    );
}

#[test]
fn an_edge_in_a_field_the_traversal_did_not_name_is_induced() {
    // §10.3 "an edge in a field not named in `follow`/`backlinks`": edges come
    // from full entities, so `side` counts although only `down` was followed.
    let entities = store(&[
        json!({"id": "a", "down": {"id": "c"}, "side": {"id": "b"}}),
        json!({"id": "b"}),
        json!({"id": "c"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": ["a", "b"], "follow": ["down"], "depth": 1}),
    );

    assert_eq!(
        traversal["edges"],
        json!([
            {"source": "a", "field": "down", "target": "c"},
            {"source": "a", "field": "side", "target": "b"},
        ]),
        "the walk: {traversal}"
    );
}

#[test]
fn two_returned_nodes_neither_expanded_still_contribute_their_edge() {
    // §10.3 "two returned nodes neither of which was expanded": `depth: 0`
    // expands nothing, yet the edge is induced because both endpoints are
    // returned.
    let entities = store(&[
        json!({"id": "a", "points_at": {"id": "b"}}),
        json!({"id": "b"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": ["a", "b"], "follow": ["points_at"], "depth": 0}),
    );

    assert_eq!(
        traversal["depths"],
        json!({"a": 0, "b": 0}),
        "the walk: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "a", "field": "points_at", "target": "b"}]),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["a", "b"]));
}

#[test]
fn a_select_hiding_the_reference_field_keeps_edges_and_depths() {
    // §10.3 "`select` hides the reference field": edges and depths are read from
    // full entities, so the projection cannot erase connectivity.
    let entities = store(&[json!({"id": "a", "next": {"id": "b"}}), json!({"id": "b"})]);

    let document = envelope_of(
        &entities,
        json!({"from": "a", "follow": ["next"], "depth": 1, "select": ["id"]}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"],
        json!([{"id": "a"}, {"id": "b"}]),
        "the projection hides `next`: {document}"
    );
    assert_eq!(
        traversal["depths"],
        json!({"a": 0, "b": 1}),
        "the walk: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "a", "field": "next", "target": "b"}]),
        "the walk: {traversal}"
    );
}

#[test]
fn an_entity_reachable_twice_through_one_field_contributes_one_edge() {
    // §8.2 "dedupe by `(source, field, target)`": a repeated array entry is one
    // discovery and one edge.
    let entities = store(&[
        json!({"id": "root", "list": [{"id": "c"}, {"id": "c"}]}),
        json!({"id": "c"}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "root", "follow": ["list"], "depth": 1}),
    );

    assert_eq!(
        traversal["edges"],
        json!([{"source": "root", "field": "list", "target": "c"}]),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["c"]));
}

#[test]
fn edges_follow_result_order_not_discovery_order() {
    // DEC-014 trap: the walk discovers `a` then `b`, but `order` returns `b`
    // first, and `edges` is the induced subgraph of the result in result order —
    // never a second account of the walk. A re-ordered or returned-but-unexpanded
    // node must not reorder it.
    let entities = store(&[
        json!({"id": "root", "kids": [{"id": "a"}, {"id": "b"}], "title": "Root"}),
        json!({"id": "a", "link": {"id": "shared"}, "title": "Zulu"}),
        json!({"id": "b", "link": {"id": "shared"}, "title": "Able"}),
        json!({"id": "shared", "title": "Middle"}),
    ]);

    let traversal = walked(
        &entities,
        json!({
            "from": "root",
            "follow": ["kids", "link"],
            "depth": 2,
            "order": [{"field": "title"}]
        }),
    );

    assert_eq!(
        traversal["depths"],
        json!({"a": 1, "b": 1, "root": 0, "shared": 2}),
        "the walk: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([
            {"source": "b", "field": "link", "target": "shared"},
            {"source": "root", "field": "kids", "target": "a"},
            {"source": "root", "field": "kids", "target": "b"},
            {"source": "a", "field": "link", "target": "shared"},
        ]),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["shared"]));
}

#[test]
fn the_limit_cuts_depths_and_edges_but_not_the_frontier() {
    // §10.3 "`limit` omits a discovered node": its depth and its edges are cut
    // because only returned objects carry them, while the frontier still names it.
    let entities = store(&[
        json!({"id": "root", "out": {"id": "d1"}}),
        json!({"id": "d1", "next": {"id": "d2"}}),
        json!({"id": "d2"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["out", "next"], "depth": 1, "limit": 1}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"],
        json!([{"id": "root", "out": {"id": "d1"}}]),
        "the limit returned the root alone: {document}"
    );
    assert_eq!(
        traversal["depths"],
        json!({"root": 0}),
        "the omitted id takes no depth: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([]),
        "the omitted id takes no edge: {traversal}"
    );
    assert_frontier(&traversal, json!(["d1"]));
}
