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
//! Three cases run the binary — `a_malformed_predicate_…`, `a_misplaced_operator_…`
//! and `a_where_query_…` — because the properties they fix are process-level
//! promises no in-crate call can see: "the whole request is validated before the
//! store is opened" (EX-6), the refusal's exact stderr line, and "a valid `where`
//! returns the filtered objects".

use serde_json::{Value, json};
use smut::select::{Selection, envelope, select};
use smut::spec;

mod common;
use common::{Scratch, parse, smut, store};

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
    let entities = [
        json!({"smut_id": "null", "f": null}),
        json!({"smut_id": "absent"}),
    ];

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
        json!({"smut_id": "text", "f": "keep selection small"}),
        json!({"smut_id": "list", "f": ["one", {"k": 2}]}),
        json!({"smut_id": "number", "f": 1}),
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
    let entities = [json!({"smut_id": "a"}), json!({"smut_id": "b"})];

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
        json!({"smut_id": "a", "type": "question", "status": "open"}),
        json!({"smut_id": "b", "type": "question", "status": "blocked"}),
        json!({"smut_id": "c", "type": "decision", "status": "open"}),
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
        json!({"smut_id": "a", "type": "question", "status": "open", "body": "one"}),
        json!({"smut_id": "b", "type": "question", "status": "blocked"}),
        json!({"smut_id": "c", "type": "decision", "status": "open", "body": "two"}),
        json!({"smut_id": "d", "status": "open"}),
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
        json!({"smut_id": "x", "meta": {"$ref": "x"}}),
        json!({"smut_id": "y", "meta": {"$ref": "y"}}),
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
    // §6.2: a `$` key the language does not define is an unknown operator at
    // either level — `$foo` at the predicate level, and all three clauses inside a
    // condition object, because a clause is not a field operator. The five
    // operators out of place at the predicate level are a different refusal:
    // misplaced, not unknown. Both refusals name the key that caused them.
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
        json!({"smut_id": "a", "status": "keep", "next": {"smut_id": "b"}}),
        json!({"smut_id": "b", "status": "drop"}),
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
            json!([{"smut_id": "a", "next": {"smut_id": "b"}, "status": "keep"}]),
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
        let store = store(&[
            json!({"smut_id": "a"}),
            json!({"smut_id": "b"}),
            json!({"smut_id": "c"}),
        ]);
        let specification =
            parse(json!({"from": ["a", "b", "c"], "where": {"smut_id": "c"}, "limit": 1}));
        let selection = select(&store, &specification).expect("nothing to sort");
        let document = envelope(&selection);

        assert_eq!(
            document["objects"],
            json!([{"smut_id": "c"}]),
            "`where` runs before `limit`, and `smut_id` names the identity: {document}"
        );
        assert_eq!(
            document["truncated"],
            json!(false),
            "exactly `limit` results is not truncation: {document}"
        );
    }
}

// ── EX-6 and the end-to-end success: the binary ─────────────────────────────

#[test]
fn a_malformed_predicate_rejects_the_whole_request_before_any_store_is_opened() {
    // EX-6: the whole request is validated before the store is opened, locked or
    // created (DEC-004). There is no store here, so a request that reached the
    // store would exit 1 with `no store`: exit 2 is the ordering evidence. (A
    // query never creates a store, so asserting none appeared would prove nothing.)
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
    }

    // A field whose conditions cannot all hold is one more refusal (DEC-027).
    // A predicate composed at runtime can reach one — an `$in` list that a
    // derivation left empty is the common shape — and now gets exit 2 where it
    // used to get an empty result; `from: []` remains the documented spelling
    // for an intentional empty result.
    let contradictory = [
        (
            r#"{"where":{"status":{"$in":[]}}}"#,
            "$in is given an empty list",
        ),
        (
            r#"{"where":{"status":{"$eq":"open","$ne":"open"}}}"#,
            "$eq and $ne are given the same value",
        ),
        (
            r#"{"where":{"status":{"$ne":"open","$in":["open"]}}}"#,
            "$ne contradicts every value $in lists",
        ),
        (
            r#"{"where":{"status":{"$contains":"zzz","$in":["open"]}}}"#,
            "$contains is not satisfied by any value $in lists",
        ),
    ];

    for (specification, contradiction) in contradictory {
        let refused = smut(&scratch, &["query", specification]);

        assert_eq!(
            refused.code, 2,
            "an unsatisfiable field is exit 2 for {specification}: {}",
            refused.stderr
        );
        assert_eq!(
            refused.stdout, "",
            "a refusal never prints an envelope: {specification}"
        );
        assert!(
            refused.stderr.contains("field status cannot be satisfied"),
            "the refusal names the field and the reason for {specification}: {}",
            refused.stderr
        );
        assert!(
            refused.stderr.contains(contradiction),
            "the refusal names `{contradiction}` for {specification}: {}",
            refused.stderr
        );
    }
}

#[test]
fn a_misplaced_operator_is_refused_naming_its_position_and_never_a_field() {
    // DEC-030 / design §4: a `$` key that names one of the five operators is not
    // unknown — it is in the wrong place, and the refusal says where it belongs
    // without inventing a field the caller may not have.
    //
    // Asserted as the whole line, not a substring: a message that printed a
    // concrete field name, or a literal operand, would still satisfy `contains`.
    // Raw strings keep the pinned sentence byte-exact.
    let scratch = Scratch::new();

    let misplaced = [
        (
            r#"{"where":{"$ne":"x"}}"#,
            r#"smut: predicate operator $ne belongs inside a field's condition object: {"<field>": {"$ne": <operand>}}"#,
        ),
        (
            r#"{"where":{"$in":["x"]}}"#,
            r#"smut: predicate operator $in belongs inside a field's condition object: {"<field>": {"$in": <operand>}}"#,
        ),
        // The same operator beside a real field: the sentence describes where an
        // operator belongs, so it must not name `status`.
        (
            r#"{"where":{"status":"open","$ne":"x"}}"#,
            r#"smut: predicate operator $ne belongs inside a field's condition object: {"<field>": {"$ne": <operand>}}"#,
        ),
    ];

    for (specification, expected) in misplaced {
        let refused = smut(&scratch, &["query", specification]);

        assert_eq!(
            refused.code, 2,
            "a misplaced operator is exit 2 for {specification}: {}",
            refused.stderr
        );
        assert_eq!(
            refused.stdout, "",
            "a refusal never prints an envelope: {specification}"
        );
        assert_eq!(
            refused.stderr.lines().collect::<Vec<_>>(),
            [expected],
            "stderr is the whole pinned line and nothing else for {specification}: {:?}",
            refused.stderr
        );
        assert!(
            !refused.stderr.contains("status") && !refused.stderr.contains(r#""x""#),
            "no concrete field name and no operand literal for {specification}: {}",
            refused.stderr
        );
    }

    // The unknown-name sentence is unchanged at this level and pinned whole: the
    // existing `$foo` assertions are variant equality or substrings. `unknown` is
    // a name the language does not define, which is a different defect.
    let unknown = smut(&scratch, &["query", r#"{"where":{"$foo":1}}"#]);

    assert_eq!(
        unknown.code, 2,
        "an unknown name is exit 2: {}",
        unknown.stderr
    );
    assert_eq!(
        unknown.stdout, "",
        "a refusal never prints an envelope: {:?}",
        unknown.stdout
    );
    assert_eq!(
        unknown.stderr.lines().collect::<Vec<_>>(),
        [r#"smut: unknown predicate operator: $foo"#],
        "the unknown-name line is unchanged: {:?}",
        unknown.stderr
    );
}

#[test]
fn a_where_query_through_the_cli_returns_the_filtered_objects() {
    // The end-to-end success case the refusals above are paired with: a real
    // store, a valid predicate, and the objects it selects. The envelope is the
    // 0.1 five exactly, because no traversal key was supplied — `where` filters
    // and adds no member.
    let scratch = Scratch::new();

    for declaration in [
        r#"{"smut_id":"q","type":"question","status":"open"}"#,
        r#"{"smut_id":"d","type":"decision","status":"open"}"#,
        r#"{"smut_id":"q2","type":"question","status":"blocked"}"#,
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
            "objects": [{"smut_id": "q", "status": "open", "type": "question"}],
            "missing": [],
            "dangling": [],
            "truncated": false,
            "truncation_reasons": []
        })
    );
}
