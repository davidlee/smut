//! The `where` predicate from outside the crate: the five operators, the three
//! clauses, the disambiguation rule and its refusal, missing-versus-present-null,
//! and the two placement rules that make it a `roots-only` filter (§6.2, §6.3,
//! §7.1).
//!
//! Every case drives the public surface a command drives — `spec::parse` to
//! accept or refuse the whole specification, then `select` and `envelope` to
//! compute the result. Nothing here names the predicate AST: `where` is
//! internal, so a test that asserted on its shape would pin an implementation
//! detail that only the unit tests need to name — §6.1 fixes the AST, but a
//! binary-level test should not read it. The refusals are asserted the same way, by the
//! behaviour and the message text a user sees, because the refusal variants are
//! the implementation's to choose.
//!
//! The final two cases run the binary, because the two properties they fix —
//! "the whole request is validated before the store is opened" (EX-6) and "a
//! valid `where` returns the filtered objects" — are process-level promises.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use smut::declare::Entities;
use smut::id::Id;
use smut::select::{Selection, envelope, select};
use smut::spec;

fn store(entities: &[Value]) -> Entities {
    entities
        .iter()
        .map(|entity| {
            let id = entity["id"].as_str().expect("every fixture names its id");
            (Id::new(id), entity.clone())
        })
        .collect()
}

fn parse(input: Value) -> spec::Spec {
    spec::parse(&input).expect("the fixture specification is valid")
}

fn selected<'a>(selection: &'a Selection<'a>) -> Vec<&'a str> {
    selection.ids.iter().map(|id| id.as_str()).collect()
}

/// The ids a specification selects from `entities`, through the public spec
/// edge. Every predicate below goes this way, so an accepted `where` is one that
/// parses *and* computes.
fn selected_ids(entities: &[Value], specification: Value) -> Vec<String> {
    let store = store(entities);
    let specification = parse(specification);
    let selection = select(&store, &specification).expect("a filter is not a sort failure");

    selected(&selection)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

// ── §6.3: missing versus present `null` ─────────────────────────────────────

#[test]
fn missing_and_present_null_differ_exactly_where_the_table_says() {
    // §6.3: `$exists` is the only operator a missing field can satisfy, and it
    // is decided by *presence*, so a present `null` is present. Every other
    // operator needs the field present, which makes the two entities below a
    // two-row truth table for each operator.
    let entities = [json!({"id": "null", "f": null}), json!({"id": "absent"})];

    let cases: [(&str, Value, &[&str]); 6] = [
        // Present and structurally equal to `null`: the `null` entity holds.
        ("$eq null", json!({"$eq": null}), &["null"]),
        // Present and *not* equal: the `null` entity fails, and the absent one
        // fails too, so nothing is selected. A missing field is not "not equal".
        ("$ne null", json!({"$ne": null}), &[]),
        ("$in [null]", json!({"$in": [null]}), &["null"]),
        // A `null` field is neither a string nor an array, so containment does
        // not match — and it does not refuse either (§6.3 "other combinations do
        // not match").
        ("$contains null", json!({"$contains": null}), &[]),
        ("$exists true", json!({"$exists": true}), &["null"]),
        ("$exists false", json!({"$exists": false}), &["absent"]),
    ];

    for (what, condition, expected) in cases {
        assert_eq!(
            selected_ids(
                &entities,
                json!({"from": ["null", "absent"], "where": {"f": condition}})
            ),
            expected,
            "{what}"
        );
    }
}

#[test]
fn contains_matches_a_substring_or_an_array_member_and_nothing_else() {
    // §6.3's `$contains` row, positively: a string field takes a substring
    // operand, an array field takes a structurally equal member, and any other
    // combination simply does not match. `$contains` has no refusal path — any
    // operand parses, and the field's own shape decides the outcome — so this
    // case is its only positive pin; the negative half lives in
    // `missing_and_present_null_differ_exactly_where_the_table_says`'s non-match
    // row.
    let entities = [
        json!({"id": "text", "f": "keep selection small"}),
        json!({"id": "list", "f": ["one", {"k": 2}]}),
        json!({"id": "number", "f": 1}),
    ];

    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": ["text", "list", "number"], "where": {"f": {"$contains": "selection"}}})
        ),
        ["text"],
        "a string operand against a string field is a substring test"
    );
    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": ["text", "list", "number"], "where": {"f": {"$contains": {"k": 2}}}})
        ),
        ["list"],
        "an array field matches a structurally equal member, object members included"
    );
    assert!(
        selected_ids(
            &entities,
            json!({"from": ["text", "list", "number"], "where": {"f": {"$contains": "1"}}})
        )
        .is_empty(),
        "a string operand against a number field matches nothing: there is no coercion"
    );
}

// ── §6.2: the clauses ───────────────────────────────────────────────────────

#[test]
fn empty_and_is_true_and_empty_or_is_false() {
    // §6.3's identities: empty `$and` holds for every root, empty `$or` for
    // none. Two entities make "every root" a real selection rather than a
    // one-element accident.
    let entities = [json!({"id": "a"}), json!({"id": "b"})];

    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": ["a", "b"], "where": {"$and": []}})
        ),
        ["a", "b"],
        "`$and` of nothing is true"
    );
    assert!(
        selected_ids(&entities, json!({"from": ["a", "b"], "where": {"$or": []}})).is_empty(),
        "`$or` of nothing is false"
    );
}

#[test]
fn sibling_keys_and_nested_not_hold_conjunctively() {
    // §6.2: a `where` object's keys are ANDed, and a field condition beside a
    // clause is one more conjunct — not an argument to the clause. `c` matches
    // the clause but not the field condition, and `b` matches the field
    // condition but not the clause, so only `a` survives either way.
    let entities = [
        json!({"id": "a", "type": "question", "status": "open"}),
        json!({"id": "b", "type": "question", "status": "blocked"}),
        json!({"id": "c", "type": "decision", "status": "open"}),
    ];
    let roots = ["a", "b", "c"];

    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": roots, "where": {"type": "question", "$or": [{"status": "open"}]}})
        ),
        ["a"],
        "a field condition beside `$or` is a conjunct, not a member of the clause"
    );
    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": roots, "where": {"type": "question", "$not": {"status": "blocked"}}})
        ),
        ["a"],
        "`$not` negates one predicate object and is ANDed with its sibling"
    );
    assert_eq!(
        selected_ids(
            &entities,
            json!({"from": roots, "where": {"$not": {"$not": {"status": "open"}}}})
        ),
        ["a", "c"],
        "`$not` nests, and two negations restore the inner result"
    );
}

// ── §6.3: the six examples that must pass ───────────────────────────────────

#[test]
fn the_six_must_pass_examples_select_what_the_design_says() {
    // The design's six examples, verbatim (§6.3). `d` carries neither `type` nor
    // `body`, `b` has no `body` and a blocked status, and `c` is a decision, so
    // every example is separated from every other by at least one entity.
    let entities = [
        json!({"id": "a", "type": "question", "status": "open", "body": "one"}),
        json!({"id": "b", "type": "question", "status": "blocked"}),
        json!({"id": "c", "type": "decision", "status": "open", "body": "two"}),
        json!({"id": "d", "status": "open"}),
    ];
    let all = ["a", "b", "c", "d"];

    let examples: [(&str, Value, &[&str]); 6] = [
        (
            r#"{"type":"question","status":"open"}"#,
            json!({"type": "question", "status": "open"}),
            &["a"],
        ),
        (
            r#"{"type":{"$in":["question","decision"]},"body":{"$exists":true}}"#,
            json!({"type": {"$in": ["question", "decision"]}, "body": {"$exists": true}}),
            &["a", "c"],
        ),
        (
            r#"{"type":"question","$or":[{"status":"open"},{"status":"blocked"}]}"#,
            json!({"type": "question", "$or": [{"status": "open"}, {"status": "blocked"}]}),
            &["a", "b"],
        ),
        (r#"{}"#, json!({}), &all),
        (r#"{"$and":[]}"#, json!({"$and": []}), &all),
        (r#"{"$or":[]}"#, json!({"$or": []}), &[]),
    ];

    for (spelling, predicate, expected) in examples {
        assert_eq!(
            selected_ids(&entities, json!({"from": all, "where": predicate})),
            expected,
            "the design's example {spelling}"
        );
    }
}

// ── §6.2: the disambiguation rule and its escape ────────────────────────────

#[test]
fn a_mixed_object_is_refused_naming_the_field() {
    // §6.2: a value with both a literal key and an operator key names no
    // behaviour the design gives, so it is refused, naming the field. The
    // refusal variant is the implementation's to choose, so this asserts what a
    // user sees: a refusal at the spec edge whose message names `meta` and says
    // that the object mixes — or, in place of the word, shows the `$` keys.
    //
    // Discharged by `the_six_must_pass_examples_select_what_the_design_says` and
    // `a_where_query_through_the_cli_returns_the_filtered_objects`: two
    // accepted, computed `where` results through the same `spec::parse` edge, so
    // "accepted" is not this file's only untested claim.
    let rejection = spec::parse(&json!({"where": {"meta": {"$eq": 1, "x": 2}}}))
        .expect_err("a mixed literal/operator object is refused");

    let message = rejection.to_string();
    assert!(
        message.contains("meta"),
        "the refusal names the field the mix is in: {message}"
    );
    assert!(
        message.contains("mix") || message.contains('$'),
        "the refusal says what is wrong: {message}"
    );
}

#[test]
fn the_eq_escape_compares_a_literal_object_that_carries_dollar_keys() {
    // §6.2: `$` is reserved at the condition level, so a literal object holding
    // operator-shaped keys is reachable only as an explicit `$eq` operand. Both
    // entities are present, so the selected one is a real comparison and the
    // other a real refusal-to-match.
    let entities = [
        json!({"id": "x", "meta": {"$ref": "x"}}),
        json!({"id": "y", "meta": {"$ref": "y"}}),
    ];

    let store = store(&entities);
    let specification = parse(json!({
        "from": ["x", "y"],
        "where": {"meta": {"$eq": {"$ref": "x"}}}
    }));
    let selection =
        select(&store, &specification).expect("an `$eq` of a literal object is a filter");

    assert_eq!(
        selected(&selection),
        ["x"],
        "`$eq`'s operand is the literal, `$ref` key and all"
    );
}

#[test]
fn an_in_or_exists_operand_of_the_wrong_shape_is_refused() {
    // §6.2: `$in` takes the operand *array*, and `$exists` takes a boolean; a
    // number for either is a refusal, not a coercion and not a silent
    // "never matches". The messages name the operator.
    //
    // Discharged by `missing_and_present_null_differ_exactly_where_the_table_says`,
    // which accepts `$in` with an array operand and `$exists` with both booleans
    // and asserts what they select.
    let wrong_in = spec::parse(&json!({"where": {"x": {"$in": 1}}}))
        .expect_err("`$in` needs an array operand");
    let wrong_exists = spec::parse(&json!({"where": {"x": {"$exists": 1}}}))
        .expect_err("`$exists` needs a boolean operand");

    assert!(
        wrong_in.to_string().contains("$in"),
        "the refusal names the operator: {wrong_in}"
    );
    assert!(
        wrong_exists.to_string().contains("$exists"),
        "the refusal names the operator: {wrong_exists}"
    );
}

#[test]
fn an_unknown_operator_and_a_clause_inside_a_field_condition_are_refused() {
    // §6.2: at the predicate level any `$` key other than `$and`/`$or`/`$not` is
    // an unknown operator; at the condition level all three clauses are unknown
    // operators, because a clause is not a field operator. Both refusals name
    // the key that caused them.
    //
    // Discharged by `sibling_keys_and_nested_not_hold_conjunctively`, which
    // accepts and computes all three clauses at the predicate level, and by
    // `the_eq_escape_compares_a_literal_object_that_carries_dollar_keys`, which
    // accepts an operator object that does hold a legal operator.
    let unknown_operator = spec::parse(&json!({"where": {"$foo": 1}}))
        .expect_err("`$foo` is not an operator this language defines");
    let clause_in_a_condition = spec::parse(&json!({"where": {"x": {"$and": []}}}))
        .expect_err("`$and` is a predicate clause, not a field operator");

    assert!(
        unknown_operator.to_string().contains("$foo"),
        "the refusal names the unknown operator: {unknown_operator}"
    );
    assert!(
        clause_in_a_condition.to_string().contains("$and"),
        "the refusal names the clause used as an operator: {clause_in_a_condition}"
    );
}

// ── VT-3: `where` is a `roots-only` filter, before the walk and the limit ───

#[test]
fn where_filters_roots_before_the_walk_and_the_limit() {
    // Four claims, one store where they can each be separated.
    let entities = [
        json!({"id": "a", "status": "keep", "next": {"id": "b"}}),
        json!({"id": "b", "status": "drop"}),
    ];

    // (1) §7.1: `where` never filters what traversal discovers. `b` fails the
    // predicate and is still returned, because only roots are filtered.
    {
        let store = store(&entities);
        let specification =
            parse(json!({"from": "a", "follow": ["next"], "where": {"status": "keep"}}));
        let selection = select(&store, &specification).expect("every discovered id is stored");

        assert_eq!(
            selected(&selection),
            ["a", "b"],
            "a discovered entity failing `where` is returned: roots only"
        );
    }

    // (2) §7.1: a supplied root that is present but fails the predicate is
    // omitted, and is *not* missing — it was found, then filtered.
    {
        let store = store(&entities);
        let specification = parse(json!({"from": ["a", "b"], "where": {"status": "keep"}}));
        let selection = select(&store, &specification).expect("nothing to sort");
        let document = envelope(&selection);

        assert_eq!(
            document["objects"],
            json!([{"id": "a", "next": {"id": "b"}, "status": "keep"}]),
            "the filtered root is omitted: {document}"
        );
        assert_eq!(
            document["missing"],
            json!([]),
            "a filtered root is not missing: {document}"
        );
    }

    // (3) §7.1: filtering happens before the walk. The root is filtered out, so
    // the walk starts empty and `b` is never discovered. If `where` ran after
    // the walk, `b` would be returned and these three lists would not be empty.
    {
        let store = store(&entities);
        let specification =
            parse(json!({"from": "a", "follow": ["next"], "where": {"status": "none"}}));
        let selection = select(&store, &specification).expect("nothing to sort");
        let document = envelope(&selection);
        let traversal = &document["traversal"];

        assert_eq!(
            document["objects"],
            json!([]),
            "the filtered root is gone before the walk, so nothing is discovered: {document}"
        );
        assert!(
            traversal.is_object(),
            "a supplied `follow` still carries the member: {document}"
        );
        assert_eq!(traversal["depths"], json!({}), "{traversal}");
        assert_eq!(traversal["edges"], json!([]), "{traversal}");
        assert_eq!(traversal["frontier"], json!([]), "{traversal}");
    }

    // (4) §4's pipeline: roots, then `where`, then the cap. Three roots and a
    // `limit` of one would keep `a` if the cap ran first; the surviving root is
    // `c`, so the filter ran first — and exactly `limit` results is not an
    // omission.
    {
        let store = store(&[json!({"id": "a"}), json!({"id": "b"}), json!({"id": "c"})]);
        let specification =
            parse(json!({"from": ["a", "b", "c"], "where": {"id": "c"}, "limit": 1}));
        let selection = select(&store, &specification).expect("nothing to sort");
        let document = envelope(&selection);

        assert_eq!(
            document["objects"],
            json!([{"id": "c"}]),
            "`where` runs before `limit`, and `id` is an ordinary field: {document}"
        );
        assert_eq!(
            document["truncated"],
            json!(false),
            "exactly `limit` results is not truncation: {document}"
        );
    }
}

// ── EX-6 and the end-to-end success: the binary ─────────────────────────────

/// A scratch working directory holding at most one store, removed when dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        let path = std::env::temp_dir().join(format!(
            "smut-predicates-{}-{}",
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

    /// A successful machine command: one compact JSON line on stdout, exit 0,
    /// nothing on stderr.
    fn assert_one_json_line(&self, what: &str) {
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

#[test]
fn a_malformed_predicate_rejects_the_whole_request_before_any_store_is_opened() {
    // EX-6: the whole request is validated before the store is opened, locked or
    // created (DEC-004). The scratch directory is fresh, so a store appearing
    // here would be the refusal's own doing.
    //
    // Discharged by `a_where_query_through_the_cli_returns_the_filtered_objects`,
    // which runs a valid `where` through the same binary and asserts the
    // objects it computed: a build that refused every predicate would fail it.
    let scratch = Scratch::new();

    let malformed = [
        (r#"{"where":{"x":{"$in":1}}}"#, "$in"),
        (r#"{"where":{"$foo":1}}"#, "$foo"),
        (r#"{"where":[]}"#, "where"),
    ];

    for (specification, offending) in malformed {
        let refused = smut(&scratch, &["query", specification]);

        assert_eq!(
            refused.code, 2,
            "invalid input is exit 2 for {specification}: {}",
            refused.stderr
        );
        assert_eq!(
            refused.stdout, "",
            "a refusal never prints an envelope: {specification}"
        );
        assert!(
            refused.stderr.contains(offending),
            "the refusal names `{offending}` for {specification}: {}",
            refused.stderr
        );
        assert!(
            !scratch.has_store(),
            "the request is validated before the store is opened, so no store exists: {specification}"
        );
    }
}

#[test]
fn a_where_query_through_the_cli_returns_the_filtered_objects() {
    // The end-to-end success case the refusals above are paired with: a real
    // store, a valid predicate, and the objects it selects. The envelope is the
    // 0.1 five exactly, because no traversal key was supplied — `where` filters
    // and adds no member.
    let scratch = Scratch::new();

    for declaration in [
        r#"{"id":"q","type":"question","status":"open"}"#,
        r#"{"id":"d","type":"decision","status":"open"}"#,
        r#"{"id":"q2","type":"question","status":"blocked"}"#,
    ] {
        smut(&scratch, &["declare", declaration]).assert_one_json_line("a fixture declaration");
    }

    let queried = smut(
        &scratch,
        &["query", r#"{"where":{"type":"question","status":"open"}}"#],
    );

    queried.assert_one_json_line("a query with a valid `where`");
    assert_eq!(
        queried.json(),
        json!({
            "objects": [{"id": "q", "status": "open", "type": "question"}],
            "missing": [],
            "dangling": [],
            "truncated": false,
            "truncation_reasons": []
        })
    );
}
