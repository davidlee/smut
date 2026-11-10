//! The walker's determinism contract for SL-002 PHASE-01 (VT-2).
//!
//! Design §7.4's six determinism rules, pinned through the public surface — named
//! fields in query order, wildcard lexical order, array position, outgoing before
//! incoming, incoming sources in ID order, a repeated label collapsing without
//! reordering — plus one identical envelope from a store rebuilt so the same
//! entities live under shuffled filenames (the "stable across filesystem
//! iteration" rule).
//!
//! This file is green against the walker: `follow`, `backlinks` and `depth` are
//! accepted keys and the envelope carries its `traversal` member, so the tests
//! below assert §7's ordering rules directly rather than a refusal.

use std::fs;

use serde_json::{Value, json};
use smut::declare::Entities;
use smut::id::Id;
use smut::select::{envelope, get, select};
use smut::spec;

mod common;
use common::{Scratch, smut, store};

/// The envelope's `traversal` member for `input`.
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
        json!({"smut_id": "hub", "alpha": {"smut_id": "n-a"}, "zeta": {"smut_id": "n-z"}}),
        json!({"smut_id": "n-a"}),
        json!({"smut_id": "n-z"}),
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
        json!({"smut_id": "array-root", "list": [{"smut_id": "c"}, {"smut_id": "a"}, {"smut_id": "b"}]}),
        json!({"smut_id": "a"}),
        json!({"smut_id": "b"}),
        json!({"smut_id": "c"}),
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
        json!({"smut_id": "r", "down": {"smut_id": "child"}}),
        json!({"smut_id": "child"}),
        json!({"smut_id": "x1", "up": {"smut_id": "r"}}),
        json!({"smut_id": "x2", "up": {"smut_id": "r"}}),
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
        json!({"smut_id": "src-b", "up": {"smut_id": "r"}}),
        json!({"smut_id": "src-a", "up": {"smut_id": "r"}}),
        json!({"smut_id": "r"}),
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
        json!({"smut_id": "q-root", "q": {"smut_id": "t"}}),
        json!({"smut_id": "t"}),
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
            r#"[{"smut_id":"alpha","title":"Alpha","next":{"smut_id":"gamma"}},{"smut_id":"beta","title":"Beta","next":[{"smut_id":"delta"}]}]"#,
        ],
    );
    assert_eq!(first.code, 0, "{}", first.stderr);

    let second = smut(
        scratch,
        &[
            "declare",
            r#"[{"smut_id":"gamma","title":"Gamma"},{"smut_id":"delta","title":"Delta"}]"#,
        ],
    );
    assert_eq!(second.code, 0, "{}", second.stderr);
}

/// Rewrites the committed store so the same entities live under a reversed
/// permutation of their basenames, and the index's bytes name the ids in reverse
/// order as well.
fn shuffle_store(scratch: &Scratch) {
    let index_path = scratch.store_dir().join("index.json");
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
        fs::rename(
            scratch.store_dir().join(old),
            scratch.store_dir().join(park),
        )
        .expect("an entity file takes a parking basename");
    }
    for (park, new) in parked.iter().zip(&shuffled) {
        fs::rename(
            scratch.store_dir().join(park),
            scratch.store_dir().join(new),
        )
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
            "{{\n  \"format_version\": 2,\n  \"objects\": {{\n{}\n  }}\n}}\n",
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

/// The envelope's `traversal` member.
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
fn edges_is_the_request_key_spelling_of_follow_empty() {
    // DEC-012, amended for `edges`: the key exists so a request that asks no
    // question about the graph still gets the induced edge set. It is not a
    // second mechanism — it parses to `follow: []`, so the two envelopes are
    // identical, member for member.
    let entities = hub();
    let asked = envelope_of(&entities, json!({"from": ["hub", "n-a"], "edges": true}));
    let spelled = envelope_of(&entities, json!({"from": ["hub", "n-a"], "follow": []}));

    assert_eq!(asked, spelled);
    assert_eq!(
        traversal_member(&asked)["edges"],
        json!([{"source": "hub", "field": "alpha", "target": "n-a"}]),
        "the induced edge among returned objects: {asked}"
    );
}

#[test]
fn edges_beside_a_supplied_traversal_changes_nothing() {
    // A request that already traverses carries the edge set regardless, so the
    // key is redundant there — and must stay redundant rather than decisive.
    let entities = hub();
    let with_key = envelope_of(
        &entities,
        json!({"from": "hub", "follow": ["alpha"], "edges": true}),
    );
    let without = envelope_of(&entities, json!({"from": "hub", "follow": ["alpha"]}));

    assert_eq!(with_key, without);
}

#[test]
fn edges_spends_no_traversal_budget() {
    // A traversal that follows nothing expands nothing, so it cannot exhaust the
    // budget. The trap: charging a budget unit per root reports a
    // `traversal_budget` truncation on a request that traversed nothing at all,
    // which is a false account of what the store could answer.
    let envelope = envelope_of(
        &hub(),
        json!({"from": ["hub", "n-a", "n-z"], "edges": true, "budget": 1}),
    );

    assert_eq!(envelope["truncated"], json!(false), "{envelope}");
    assert_eq!(envelope["truncation_reasons"], json!([]), "{envelope}");
    assert_frontier(&traversal_member(&envelope), json!([]));
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
fn follow_empty_still_honours_backlinks_and_the_depth_bound() {
    // §10.3 "`follow: []`", third clause: "the frontier follows `depth`,
    // `budget` and any supplied `backlinks`". An empty outgoing label list
    // switches off outgoing discovery and nothing else — the walk still runs,
    // still expands what it expands, and still finds what points at a root.
    //
    // The trap: treating an empty `follow` as "no walk" would drop `x` from
    // both the returned set and the frontier, which the other two `follow: []`
    // tests cannot catch because neither supplies `backlinks`.
    let entities = store(&[
        json!({"smut_id": "hub", "alpha": {"smut_id": "n-a"}}),
        json!({"smut_id": "n-a"}),
        json!({"smut_id": "n-z"}),
        json!({"smut_id": "x", "up": {"smut_id": "hub"}}),
    ]);

    let traversal = walked(
        &entities,
        json!({"from": "hub", "follow": [], "backlinks": ["up"], "depth": 1}),
    );

    assert_eq!(
        traversal["depths"],
        json!({"hub": 0, "x": 1}),
        "the incoming discovery is a real one: {traversal}"
    );
    assert_frontier(&traversal, json!(["x"]));
    assert_eq!(
        traversal["edges"],
        json!([{"source": "x", "field": "up", "target": "hub"}]),
        "the incoming edge is induced, and `hub.alpha` is not an edge because
         `n-a` was not discovered: {traversal}"
    );

    // The same request at `depth: 0` stops at the root: the empty `follow` does
    // not make the depth bound inert.
    let stopped = walked(
        &entities,
        json!({"from": "hub", "follow": [], "backlinks": ["up"], "depth": 0}),
    );
    assert_eq!(stopped["depths"], json!({"hub": 0}), "{stopped}");
    assert_frontier(&stopped, json!(["hub"]));
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
        json!({"from": "hub", "select": ["smut_id"], "order": [{"field": "smut_id"}], "limit": 2}),
        json!({"from": "hub", "depth": 2}),
        // DEC-019 re-cut: `budget` without `follow` or `backlinks` is accepted
        // and inert (§5), so the key set the compatibility pin compares is
        // still exactly 0.1's five — the reason this test exists.
        json!({"from": "hub", "budget": 1}),
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

/// The five clusters §13.3 scenario 3 walks, shaped as the committed fixture
/// stores them: each cluster is a `part-of` the session, each cluster holds two
/// questions that are a `part-of` it, and each question is answered by one
/// entity. The graph is built here rather than read from the fixture array so
/// the walk that matters — a wildcard over *every* reference field — exercises
/// outgoing and incoming in the same expansion, which is the shape the real
/// store has and the in-memory helpers elsewhere in this file do not.
fn clusters() -> Entities {
    const NAMES: [&str; 5] = ["friction", "mind", "schema", "setup", "thesis"];

    let mut entities = vec![json!({"smut_id": "session", "type": "session"})];

    for name in NAMES {
        entities.push(json!({
            "smut_id": format!("cluster-{name}"),
            "type": "cluster",
            "part-of": [{"smut_id": "session"}],
        }));

        for hop in 1..=2 {
            let question = format!("q-{name}-{hop}");

            entities.push(json!({
                "smut_id": question,
                "type": "question",
                "part-of": [{"smut_id": format!("cluster-{name}")}],
            }));
            entities.push(json!({
                "smut_id": format!("a-{name}-{hop}"),
                "type": "answer",
                "answered-by": [{"smut_id": question}],
            }));
        }
    }

    store(&entities)
}

#[test]
fn a_wildcard_walk_over_the_five_clusters_moves_the_frontier_out_by_one() {
    // §13.3 scenario 3 (PHASE-07 VT-3): the depth-1 wildcard walk returns the
    // clusters and their depth-1 neighbours with edges and depths, and depth 2
    // pushes the frontier out by exactly one ring while the edge set grows.
    //
    // "Wildcard" is the load-bearing word, and it is wildcarded in both
    // directions: the fixture's clusters carry one outgoing `part-of` to the
    // session and are named by ten incoming `part-of` questions, so a walk that
    // follows only outward would have no depth-2 ring to grow into. `follow: "*"`
    // plus `backlinks: "*"` is therefore the walk scenario 3 describes — the
    // clusters and everything one hop away, in either orientation.
    let entities = clusters();
    let roots = json!([
        "cluster-friction",
        "cluster-mind",
        "cluster-schema",
        "cluster-setup",
        "cluster-thesis",
    ]);

    let depth_one = envelope_of(
        &entities,
        json!({"from": roots, "follow": "*", "backlinks": "*", "depth": 1}),
    );
    let one = traversal_member(&depth_one);

    let mut expected_frontier = vec![json!("session")];
    for name in ["friction", "mind", "schema", "setup", "thesis"] {
        expected_frontier.push(json!(format!("q-{name}-1")));
        expected_frontier.push(json!(format!("q-{name}-2")));
    }

    assert_eq!(
        one["frontier"],
        Value::Array(expected_frontier.clone()),
        "the depth-1 discoveries sit on the frontier, roots expanded: {one}"
    );
    assert_eq!(
        one["depths"].as_object().map(serde_json::Map::len),
        Some(16),
        "five clusters at depth 0 and eleven neighbours at depth 1: {one}"
    );
    for root in roots.as_array().expect("an array of roots") {
        let id = root.as_str().expect("a cluster id");
        assert_eq!(one["depths"][id], json!(0), "a root is depth 0: {one}");
    }
    for discovered in ["session", "q-friction-1", "q-thesis-2"] {
        assert_eq!(
            one["depths"][discovered],
            json!(1),
            "a neighbour is one hop out: {one}"
        );
    }

    // The edge set is the induced subgraph of the returned entities: five
    // cluster-to-session edges and ten question-to-cluster edges. The answer
    // edges are not here yet, because the answers are not returned yet.
    assert_eq!(
        one["edges"].as_array().map(Vec::len),
        Some(15),
        "the depth-1 edge set: {one}"
    );
    assert!(
        one["edges"]
            .as_array()
            .expect("an edge array")
            .contains(&json!({
                "source": "cluster-friction",
                "field": "part-of",
                "target": "session",
            })),
        "the outgoing edge is induced from the returned session: {one}"
    );
    assert!(
        one["edges"]
            .as_array()
            .expect("an edge array")
            .contains(&json!({
                "source": "q-friction-1",
                "field": "part-of",
                "target": "cluster-friction",
            })),
        "the incoming edge is induced too: {one}"
    );

    let depth_two = envelope_of(
        &entities,
        json!({"from": roots, "follow": "*", "backlinks": "*", "depth": 2}),
    );
    let two = traversal_member(&depth_two);

    let expected_next_frontier: Vec<Value> = ["friction", "mind", "schema", "setup", "thesis"]
        .iter()
        .flat_map(|name| [json!(format!("a-{name}-1")), json!(format!("a-{name}-2"))])
        .collect();

    assert_eq!(
        two["frontier"],
        Value::Array(expected_next_frontier.clone()),
        "depth 2 expands the depth-1 ring and leaves the answers on the boundary: {two}"
    );
    assert!(
        expected_frontier
            .iter()
            .all(|previous| !expected_next_frontier.contains(previous)),
        "the frontier moved out rather than repeating: {two}"
    );
    assert_eq!(
        two["depths"].as_object().map(serde_json::Map::len),
        Some(26),
        "the ten answers join the returned set at depth 2: {two}"
    );
    assert_eq!(two["depths"]["a-friction-1"], json!(2), "{two}");

    let grown = two["edges"].as_array().expect("an edge array");
    assert_eq!(grown.len(), 25, "ten answer edges join the fifteen: {two}");
    assert!(
        grown.contains(&json!({
            "source": "a-thesis-2",
            "field": "answered-by",
            "target": "q-thesis-2",
        })),
        "the new ring's edges are induced from the new return set: {two}"
    );
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
        json!({"smut_id": "root", "out": [{"smut_id": "d1"}, {"smut_id": "d2"}, {"smut_id": "d3"}]}),
        json!({"smut_id": "d1"}),
        json!({"smut_id": "d2"}),
        json!({"smut_id": "d3"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["out"], "depth": 1, "limit": 1}),
    );

    assert_eq!(
        document["objects"],
        json!([{"smut_id": "root", "out": [{"smut_id": "d1"}, {"smut_id": "d2"}, {"smut_id": "d3"}]}]),
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
        "smut_id": "root",
        "refs": [{"smut_id": "gone-a"}, {"smut_id": "gone-b"}, {"smut_id": "gone-a"}]
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
        json!({"smut_id": "root", "down": {"smut_id": "kept"}, "other": {"smut_id": "gone"}}),
        json!({"smut_id": "kept"}),
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
        json!({"smut_id": "r", "down": {"smut_id": "mid"}}),
        json!({"smut_id": "mid", "down": {"smut_id": "gone"}}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "r", "follow": ["down"], "depth": 2, "limit": 1}),
    );

    assert_eq!(
        document["objects"],
        json!([{"smut_id": "r", "down": {"smut_id": "mid"}}]),
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
    let entities = store(&[json!({"smut_id": "hub", "alpha": {"smut_id": "gone"}})]);

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
        json!({"smut_id": "a", "down": {"smut_id": "c"}, "side": {"smut_id": "b"}}),
        json!({"smut_id": "b"}),
        json!({"smut_id": "c"}),
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
        json!({"smut_id": "a", "points_at": {"smut_id": "b"}}),
        json!({"smut_id": "b"}),
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
    let entities = store(&[
        json!({"smut_id": "a", "next": {"smut_id": "b"}}),
        json!({"smut_id": "b"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "a", "follow": ["next"], "depth": 1, "select": ["smut_id"]}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"],
        json!([{"smut_id": "a"}, {"smut_id": "b"}]),
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
        json!({"smut_id": "root", "list": [{"smut_id": "c"}, {"smut_id": "c"}]}),
        json!({"smut_id": "c"}),
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
fn a_cycle_and_a_self_reference_record_their_edge_once() {
    // §10.3 "cycle / self-reference": "edge recorded once; entity expanded
    // once; no requeue". The walk's own tests pin the expansion half (`order`,
    // empty frontier, depths); this pins the half only the envelope can show,
    // because `Walk` carries no edges — `edges` is induced afterwards and must
    // still dedupe by `(source, field, target)` when a triple points at itself
    // or back where it came from.
    let cycle = store(&[
        json!({"smut_id": "a", "next": {"smut_id": "b"}}),
        json!({"smut_id": "b", "next": {"smut_id": "a"}}),
    ]);

    let document = envelope_of(&cycle, json!({"from": "a", "follow": ["next"], "depth": 2}));
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"].as_array().map(Vec::len),
        Some(2),
        "each entity in the cycle is expanded once and returned once: {document}"
    );
    assert_eq!(
        traversal["depths"],
        json!({"a": 0, "b": 1}),
        "the return edge discovers nothing new: {traversal}"
    );
    assert_eq!(
        traversal["edges"],
        json!([
            {"source": "a", "field": "next", "target": "b"},
            {"source": "b", "field": "next", "target": "a"},
        ]),
        "two directed triples, neither repeated: {traversal}"
    );
    assert_frontier(&traversal, json!([]));

    let itself = store(&[json!({"smut_id": "s", "self": {"smut_id": "s"}})]);
    let document = envelope_of(
        &itself,
        json!({"from": "s", "follow": ["self"], "depth": 1}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"].as_array().map(Vec::len),
        Some(1),
        "a self-reference does not duplicate its entity: {document}"
    );
    assert_eq!(
        traversal["edges"],
        json!([{"source": "s", "field": "self", "target": "s"}]),
        "one triple, recorded once: {traversal}"
    );
    assert_eq!(document["truncated"], json!(false), "{document}");
}

#[test]
fn edges_follow_result_order_not_discovery_order() {
    // DEC-014 trap: the walk discovers `a` then `b`, but `order` returns `b`
    // first, and `edges` is the induced subgraph of the result in result order —
    // never a second account of the walk. A re-ordered or returned-but-unexpanded
    // node must not reorder it.
    let entities = store(&[
        json!({"smut_id": "root", "kids": [{"smut_id": "a"}, {"smut_id": "b"}], "title": "Root"}),
        json!({"smut_id": "a", "link": {"smut_id": "shared"}, "title": "Zulu"}),
        json!({"smut_id": "b", "link": {"smut_id": "shared"}, "title": "Able"}),
        json!({"smut_id": "shared", "title": "Middle"}),
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
        json!({"smut_id": "root", "out": {"smut_id": "d1"}}),
        json!({"smut_id": "d1", "next": {"smut_id": "d2"}}),
        json!({"smut_id": "d2"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["out", "next"], "depth": 1, "limit": 1}),
    );
    let traversal = traversal_member(&document);

    assert_eq!(
        document["objects"],
        json!([{"smut_id": "root", "out": {"smut_id": "d1"}}]),
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

// ── §7.3 budget, §8.3 reason order (PHASE-02 VT-1, VT-2) ────────────────────

/// Two children, each with one child: at `depth: 2` three expansions are
/// permitted — `root`, `left`, `right` — and the two leaves are discovered only
/// at the boundary.
fn budget_fan() -> Entities {
    store(&[
        json!({"smut_id": "root", "kids": [{"smut_id": "left"}, {"smut_id": "right"}]}),
        json!({"smut_id": "left", "kids": [{"smut_id": "left-leaf"}]}),
        json!({"smut_id": "right", "kids": [{"smut_id": "right-leaf"}]}),
        json!({"smut_id": "left-leaf"}),
        json!({"smut_id": "right-leaf"}),
    ])
}

#[test]
fn a_budget_equal_to_the_permitted_expansions_completes_the_walk() {
    // PHASE-02 VT-1 / §7.3 "the check is before the expansion, so a walk that
    // empties its queue with exactly `budget` expansions is complete and is not
    // truncated".
    //
    // The trap: an off-by-one that treats the last permitted expansion as a
    // decline reports `traversal_budget` here, and a budget counted against
    // discoveries rather than expansions cuts the walk long before its queue
    // empties.
    let document = envelope_of(
        &budget_fan(),
        json!({"from": "root", "follow": ["kids"], "depth": 2, "budget": 3}),
    );

    assert_eq!(
        document["truncated"],
        json!(false),
        "an exact budget omits nothing: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!([]),
        "a depth boundary is not a reason: {document}"
    );
    assert!(
        !document["truncation_reasons"]
            .as_array()
            .expect("the reasons are an array")
            .iter()
            .any(|reason| reason == "traversal_budget"),
        "the budget declined no permitted expansion: {document}"
    );

    let traversal = traversal_member(&document);
    assert_eq!(
        traversal["depths"],
        json!({"root": 0, "left": 1, "right": 1, "left-leaf": 2, "right-leaf": 2}),
        "every entity within the depth bound was expanded and returned: {traversal}"
    );
    assert_frontier(&traversal, json!(["left-leaf", "right-leaf"]));

    let fan = budget_fan();
    let specification = spec::parse(&json!({
        "from": "root", "follow": ["kids"], "depth": 2, "budget": 3
    }))
    .expect("the boundary specification is valid");
    let selection = select(&fan, &specification).expect("every discovery is stored");
    let walk = selection.walk.expect("a supplied `follow` starts a walk");
    assert!(
        !walk.exhausted,
        "an exact budget declines no permitted expansion, so the walk is not exhausted"
    );
}

#[test]
fn one_fewer_than_the_permitted_expansions_declines_the_next_permitted_one() {
    // PHASE-02 VT-1 / §10.3 "budget exhausts before a permitted expansion":
    // `traversal_budget`, and the declined entity is followed by the rest of the
    // drained queue in discovery order (DEC-013, DEC-015).
    //
    // `root` and `left` expand; `right` is the permitted expansion declined, and
    // `left-leaf` was still queued behind it.
    let document = envelope_of(
        &budget_fan(),
        json!({"from": "root", "follow": ["kids"], "depth": 2, "budget": 2}),
    );

    assert_eq!(
        document["truncated"],
        json!(true),
        "a permitted expansion was declined: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!(["traversal_budget"]),
        "the only omission is the declined discovery: {document}"
    );

    let traversal = traversal_member(&document);
    assert_frontier(&traversal, json!(["right", "left-leaf"]));

    let fan = budget_fan();
    let specification = spec::parse(&json!({
        "from": "root", "follow": ["kids"], "depth": 2, "budget": 2
    }))
    .expect("the boundary specification is valid");
    let selection = select(&fan, &specification).expect("every discovery is stored");
    let walk = selection.walk.expect("a supplied `follow` starts a walk");
    assert!(
        walk.exhausted,
        "a permitted expansion was declined, so the walk reports itself exhausted"
    );
}

#[test]
fn a_budget_of_one_expands_the_first_root_and_leaves_the_second_in_the_frontier() {
    // §13.3 scenario 4 (PHASE-07 EX-2): a budget of one on a two-root store
    // returns the first root's expansion, reports the second root in the
    // frontier, and sets `traversal_budget`.
    //
    // The second root is a *permitted* expansion — it sits at depth 0, inside
    // the bound — so declining it is truncation, not selection, and the walk
    // says so. Both roots are returned: a root is a selection, not a discovery,
    // so the budget cannot remove one from the result.
    let entities = store(&[
        json!({"smut_id": "r1", "out": {"smut_id": "c1"}}),
        json!({"smut_id": "c1"}),
        json!({"smut_id": "r2", "out": {"smut_id": "c2"}}),
        json!({"smut_id": "c2"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": ["r1", "r2"], "follow": ["out"], "depth": 1, "budget": 1}),
    );

    assert_eq!(
        document["truncated"],
        json!(true),
        "the second root's expansion was declined: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!(["traversal_budget"]),
        "the budget, and only the budget, cut this walk: {document}"
    );

    let traversal = traversal_member(&document);
    assert_eq!(
        traversal["depths"],
        json!({"r1": 0, "r2": 0, "c1": 1}),
        "the first root expanded and the second did not: {traversal}"
    );
    assert!(
        traversal["depths"].get("c2").is_none(),
        "the declined root's child was never discovered: {traversal}"
    );
    assert_frontier(&traversal, json!(["r2", "c1"]));
}

#[test]
fn a_budget_that_leaves_only_depth_boundary_nodes_is_complete() {
    // PHASE-02 VT-1 / §10.3 "budget exhausts with only depth-boundary nodes
    // left": complete, because the budget bounds expansions and never
    // discoveries — three discoveries at `depth: 1` cost one expansion.
    let entities = store(&[
        json!({"smut_id": "root", "kids": [{"smut_id": "a"}, {"smut_id": "b"}, {"smut_id": "c"}]}),
        json!({"smut_id": "a"}),
        json!({"smut_id": "b"}),
        json!({"smut_id": "c"}),
    ]);

    let document = envelope_of(
        &entities,
        json!({"from": "root", "follow": ["kids"], "depth": 1, "budget": 1}),
    );

    assert_eq!(
        document["truncated"],
        json!(false),
        "nothing a permitted expansion would have reached is left: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!([]),
        "the queue holds only the depth boundary: {document}"
    );

    let traversal = traversal_member(&document);
    assert_eq!(
        traversal["depths"],
        json!({"root": 0, "a": 1, "b": 1, "c": 1}),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["a", "b", "c"]));
}

#[test]
fn depth_zero_is_never_budget_truncation() {
    // PHASE-02 VT-1 / §7.3 "`depth: 0` performs no expansions and is never
    // budget truncation": two roots and the smallest accepted budget are still
    // roots only, even though both roots sit in the frontier.
    let document = envelope_of(
        &budget_fan(),
        json!({"from": ["root", "left"], "follow": ["kids"], "depth": 0, "budget": 1}),
    );

    assert_eq!(
        document["truncated"],
        json!(false),
        "roots only expands nothing: {document}"
    );
    assert_eq!(
        document["truncation_reasons"],
        json!([]),
        "a boundary root is not a declined expansion: {document}"
    );

    let traversal = traversal_member(&document);
    assert_eq!(
        traversal["depths"],
        json!({"root": 0, "left": 0}),
        "the walk: {traversal}"
    );
    assert_frontier(&traversal, json!(["root", "left"]));
}

#[test]
fn both_reasons_appear_once_in_the_order_they_can_occur() {
    // PHASE-02 VT-2 (DEF-1) / §8.3 "Reasons are unique and in the order they can
    // occur: `traversal_budget` first (discovery stopped), then `result_limit`
    // (ordered rows omitted)".
    //
    // Asserted as the ordered vector, never as a set: a reversed or duplicated
    // pair fails here. Both cuts really happen — the budget of two stops the
    // walk before `right`, and the limit of one returns the root alone.
    let document = envelope_of(
        &budget_fan(),
        json!({"from": "root", "follow": ["kids"], "depth": 2, "budget": 2, "limit": 1}),
    );

    assert_eq!(
        document["truncated"],
        json!(true),
        "both reasons are real omissions: {document}"
    );

    let reasons = document["truncation_reasons"]
        .as_array()
        .unwrap_or_else(|| panic!("the reasons are an array: {document}"));
    assert_eq!(
        document["truncation_reasons"],
        json!(["traversal_budget", "result_limit"]),
        "the ordered reasons: {document}"
    );

    let mut distinct: Vec<&str> = reasons
        .iter()
        .map(|reason| reason.as_str().expect("a reason is a string"))
        .collect();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        reasons.len(),
        "one entry per reason, no repeats: {document}"
    );

    assert_eq!(
        document["objects"].as_array().map(Vec::len),
        Some(1),
        "the limit really omitted a row: {document}"
    );
    let traversal = traversal_member(&document);
    assert_frontier(&traversal, json!(["right", "left-leaf"]));
}
