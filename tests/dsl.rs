//! SL-010 PHASE-01: `smut q`'s compiler, from outside the crate.
//!
//! One spine runs through this file: **a line of words equals the specification
//! it denotes.** Each case compiles a printed line and compares the object — and
//! the `Spec` that object parses to — against the JSON the line means, so the
//! two spellings cannot drift apart without a red. Around it sit the value-typing
//! rule, the three merge rules, and the refusals `compile` itself raises. The
//! verb's pipeline and its exit codes are `tests/cli_contract.rs`'s (PHASE-02).
//!
//! Nothing here opens a store: `compile` is a pure function of the line it is
//! handed, and a case that needed a directory would be testing the verb.

use serde_json::{Value, json};
use smut::dsl::{self, Flags, Line};
use smut::error::Reject;
use smut::learn::{
    EXAMPLE_FILTER_Q, EXAMPLE_FOLLOW, EXAMPLE_FOLLOW_Q, EXAMPLE_TRAVERSE, EXAMPLE_TRAVERSE_Q,
};
use smut::spec::{self, Spec};

/// The `(tokens, flags)` a printed `smut q` line denotes.
///
/// The guide prints one argument line per recipe, so a whitespace split plus the
/// flag table is the whole conversion: `--name=value` and `--name value` are the
/// same flag, and `--ids`/`--edges` take no value. A second reader of a printed
/// line exists only because this suite calls `compile` directly instead of a
/// process — PHASE-02 hands the same words to clap, and the fixture harness types
/// them into the binary.
fn argv(printed: &str) -> (Vec<String>, Flags) {
    let mut tokens = Vec::new();
    let mut flags = Flags::default();
    let mut words = printed.split_whitespace();

    while let Some(word) = words.next() {
        let Some(flag) = word.strip_prefix("--") else {
            tokens.push(word.to_owned());
            continue;
        };

        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let mut take = || inline.clone().or_else(|| words.next().map(str::to_owned));

        match name {
            "follow" => flags.follow = take(),
            "backlinks" => flags.backlinks = take(),
            "depth" => flags.depth = take(),
            "budget" => flags.budget = take(),
            "select" => flags.select = take(),
            "order" => flags.order = take(),
            "limit" => flags.limit = take(),
            "has" => flags.has = take(),
            "missing" => flags.missing = take(),
            "from" => flags.from = take(),
            "ids" => flags.ids = true,
            "edges" => flags.edges = true,
            other => panic!("the guide prints no --{other}"),
        }
    }

    (tokens, flags)
}

/// The object a printed line compiles to.
fn compiled(printed: &str) -> Result<Value, Reject> {
    let (tokens, flags) = argv(printed);

    dsl::compile(&Line {
        tokens: &tokens,
        flags,
    })
}

/// The object a line compiles to, refusing to hide a refusal.
fn object(printed: &str) -> Value {
    compiled(printed).unwrap_or_else(|refused| panic!("{printed:?} was refused: {refused}"))
}

/// The `Spec` a line compiles to, so a case proves the pipeline's view too.
fn spec_of(printed: &str) -> Spec {
    spec::parse(&object(printed))
        .unwrap_or_else(|refused| panic!("{printed:?} did not parse: {refused}"))
}

/// The `Spec` a JSON spelling means.
fn spec_of_json(printed: &str) -> Spec {
    let value: Value = serde_json::from_str(printed).expect("the constant is JSON");

    spec::parse(&value).unwrap_or_else(|refused| panic!("{printed} did not parse: {refused}"))
}

/// The refusal a line earns.
fn refused(printed: &str) -> Reject {
    match compiled(printed) {
        Ok(object) => panic!("{printed:?} compiled to {object}, and should have been refused"),
        Err(refused) => refused,
    }
}

/// One row of the spine: the line compiles to this object, and to this `Spec`.
fn equals(printed: &str, denotes: Value) {
    assert_eq!(object(printed), denotes, "{printed:?}");

    let denied = spec::parse(&denotes).expect("every denotation here parses");

    assert_eq!(
        spec_of(printed),
        denied,
        "{printed:?} parses to a different specification than the JSON it denotes"
    );
}

// The guide's recipes are read out of the constants the guide prints, so a recipe
// edited in one place and not the other fails here. The traversing and follow
// twins are compared to the JSON constants themselves: they are the same request
// in two spellings.
#[test]
fn a_line_of_words_equals_the_specification_it_denotes() {
    assert_eq!(
        spec_of(EXAMPLE_TRAVERSE_Q),
        spec_of_json(EXAMPLE_TRAVERSE),
        "the traversing recipe's two spellings"
    );
    assert_eq!(
        spec_of(EXAMPLE_FOLLOW_Q),
        spec_of_json(EXAMPLE_FOLLOW),
        "the follow recipe is the same request in the other direction"
    );

    // The filter twin is *not* the JSON twin: roots are this surface's spelling of
    // the `$in` filter, so the two agree on the selected set rather than on a
    // `Spec` (design §5.2), and PHASE-02 runs both to show it.
    let filter = EXAMPLE_FILTER_Q.replace("IDS", "q-query q-other");

    equals(
        &filter,
        json!({
            "from": ["q-query", "q-other"],
            "where": {"status": {"$eq": "open"}},
            "select": ["smut_id"],
        }),
    );

    // The five token shapes of design §1, each in the spelling the guide teaches.
    equals("session", json!({"from": ["session"]}));
    equals("status=open", json!({"where": {"status": {"$eq": "open"}}}));
    equals(
        "status!=open",
        json!({"where": {"status": {"$ne": "open"}}}),
    );
    equals(
        "title~graph",
        json!({"where": {"title": {"$contains": "graph"}}}),
    );
    equals(
        "status=open,resolved",
        json!({"where": {"status": {"$in": ["open", "resolved"]}}}),
    );

    // Roots and conditions interleave, and the roots keep their own order.
    equals(
        "q-query q-other status=open",
        json!({
            "from": ["q-query", "q-other"],
            "where": {"status": {"$eq": "open"}},
        }),
    );
}

// Reddens if `value()` starts accepting JSON *string* literals: `"open"` would
// lose the two quote characters the caller meant to keep.
#[test]
fn a_value_is_a_json_literal_only_when_it_is_one() {
    equals("done=true", json!({"where": {"done": {"$eq": true}}}));
    equals("done=false", json!({"where": {"done": {"$eq": false}}}));
    equals("priority=3", json!({"where": {"priority": {"$eq": 3}}}));
    equals("offset=-2", json!({"where": {"offset": {"$eq": -2}}}));
    equals("status=null", json!({"where": {"status": {"$eq": null}}}));
    equals("ratio=1.0", json!({"where": {"ratio": {"$eq": 1.0}}}));
    equals("big=1e3", json!({"where": {"big": {"$eq": 1000.0}}}));

    // Everything else stays its own text, including the numbers JSON does not
    // have: a leading zero, a sign, a base prefix, a separator, a version. And a
    // comma list is a list, not a value.
    for text in ["007", "+1", "0x10", "1_000", "v1.2"] {
        equals(
            &format!("id={text}"),
            json!({"where": {"id": {"$eq": text}}}),
        );
    }

    equals("status=", json!({"where": {"status": {"$eq": ""}}}));
}

#[test]
fn a_quoted_looking_token_keeps_its_quotes() {
    // The shell has already removed the quotes the caller typed, so a token that
    // still looks quoted is one whose quotes were meant. Unquoting it here would
    // be the second unquoting design §2 forbids.
    equals(
        r#"title~"graph""#,
        json!({"where": {"title": {"$contains": "\"graph\""}}}),
    );
    equals(
        r#"status="open""#,
        json!({"where": {"status": {"$eq": "\"open\""}}}),
    );
}

// Reddens if the empty line emits `from: []` — which the specification reads as
// "nothing at all" — or refuses a line with no defect in it.
#[test]
fn an_empty_line_compiles_to_the_empty_specification() {
    assert_eq!(object(""), json!({}));
    assert_eq!(
        spec_of(""),
        spec::parse(&json!({})).expect("{} is a legal request")
    );
    assert_eq!(
        spec_of("").from,
        None,
        "no root was typed, so nothing is filtered"
    );
}

#[test]
fn two_equals_on_one_field_is_a_contradiction_not_an_in_list() {
    // Two different values cannot both hold, and the emitted condition object has
    // one `$eq` per field, so the front-end joins DEC-027/DEC-030's family instead
    // of inventing an `$in` nobody asked for.
    let refusal = refused("status=open status=resolved");

    assert_eq!(
        refusal,
        Reject::ContradictoryFieldConditions {
            field: "status".to_owned(),
            contradiction: "$eq is given two different values".to_owned(),
        }
    );
    assert_eq!(
        refusal.to_string(),
        "field status cannot be satisfied: $eq is given two different values"
    );
}

#[test]
fn null_and_missing_are_different_requests() {
    // `null` is a stored value; `$exists: false` is the field's absence. This is
    // the pair a reader is most likely to conflate (design §7).
    equals("status=null", json!({"where": {"status": {"$eq": null}}}));
    equals(
        "--missing status",
        json!({"where": {"status": {"$exists": false}}}),
    );
    equals(
        "--has status",
        json!({"where": {"status": {"$exists": true}}}),
    );
}

#[test]
fn an_exact_repeat_collapses_into_the_condition_already_there() {
    equals(
        "status=open status=open",
        json!({"where": {"status": {"$eq": "open"}}}),
    );
    equals(
        "status!=open status!=open",
        json!({"where": {"status": {"$ne": "open"}}}),
    );
    equals(
        "title~graph title~graph",
        json!({"where": {"title": {"$contains": "graph"}}}),
    );
    equals(
        "status=open,resolved status=open,resolved",
        json!({"where": {"status": {"$in": ["open", "resolved"]}}}),
    );
}

// Reddens if a satisfiable repeat is voiced as a contradiction — the false-
// sentence class RV-031 F-2 found — or if the two tokens stop being quoted.
#[test]
fn a_repeat_the_object_cannot_hold_is_refused_in_the_callers_tokens() {
    for (line, reason) in [
        (
            "status!=a status!=b",
            "q carries one != per field; use smut query",
        ),
        (
            "title~a title~b",
            "q carries one ~ per field; use smut query",
        ),
        (
            "status=a,b status=c,d",
            "q carries one = per field; use smut query",
        ),
    ] {
        let refusal = refused(line);

        assert_eq!(
            refusal,
            Reject::UnreadableToken {
                token: line.to_owned(),
                reason: reason.to_owned(),
            },
            "{line}"
        );
        assert_eq!(
            refusal.to_string(),
            format!("cannot read \"{line}\": {reason}")
        );
    }
}

// Reddens if the pair is refused instead of compiled: `$eq open` beside `$ne
// resolved` passes `predicate::parse`'s decidability walk, so the front-end must
// hand it over. Its exit code is PHASE-02 VT-3's.
#[test]
fn the_satisfiable_pair_compiles_for_the_predicate_layer() {
    equals(
        "status=open status!=resolved",
        json!({"where": {"status": {"$eq": "open", "$ne": "resolved"}}}),
    );
    assert!(
        spec::parse(&object("status=open status!=resolved")).is_ok(),
        "the specification accepts the pair; refusing it here would be a second engine"
    );
}

#[test]
fn the_existence_flags_join_the_same_predicate() {
    equals(
        "--has body status=open",
        json!({"where": {"body": {"$exists": true}, "status": {"$eq": "open"}}}),
    );

    // Overwritten rather than refused would answer a different question quietly
    // (RV-031 F-4).
    let refusal = refused("--has status --missing status");

    assert_eq!(
        refusal,
        Reject::ContradictoryFieldConditions {
            field: "status".to_owned(),
            contradiction: "$exists is given both true and false".to_owned(),
        }
    );
    assert_eq!(
        refusal.to_string(),
        "field status cannot be satisfied: $exists is given both true and false"
    );
}

// Reddens if the `{` test moves below the operator scan: `"status":"open"` would
// split at `=` and the refusal would become unreachable (RV-031 F-1's class).
#[test]
fn a_json_operand_is_refused_by_name() {
    let refusal = refused(r#"{"where":{"status":"open"}}"#);

    assert_eq!(refusal, Reject::JsonOperand);
    assert_eq!(
        refusal.to_string(),
        "q takes field=value tokens, not a JSON specification; use smut query for a JSON spec"
    );
}

// Reddens if the two rows collapse into one sentence: they name different repairs,
// and §3's "more than one operator" branch has to discriminate by kind.
#[test]
fn a_token_with_two_operators_is_refused_with_both_named() {
    let both = refused("status~a=b");

    assert_eq!(
        both.to_string(),
        r#"cannot read "status~a=b": it carries both ~ and =; use smut query for the same predicate as JSON"#
    );

    // The same operator twice is a field name holding it, which the grammar cannot
    // address: the field-name row.
    let field = refused("a=b=c");

    assert_eq!(
        field.to_string(),
        r#"cannot read "a=b=c": a field name holding = is outside q's grammar; use smut query"#
    );
}

// `status=` is the row that proves the refusal above is narrow: an empty field has
// nothing to address, an empty value is the empty string.
#[test]
fn a_condition_needs_a_field_but_a_value_may_be_empty() {
    let refusal = refused("=open");

    assert_eq!(
        refusal.to_string(),
        r#"cannot read "=open": a condition is field, operator and value"#
    );
    equals("status=", json!({"where": {"status": {"$eq": ""}}}));
}
