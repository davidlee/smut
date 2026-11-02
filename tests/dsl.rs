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

// Reddens if two differing `=` stop being a contradiction, or start being voiced
// as an `$in` nobody asked for.
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

// Reddens if the flags become a second predicate rather than joining the `where`
// the tokens build, or if the collision is overwritten instead of refused.
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

// Reddens if the empty-field refusal widens to the empty value: `status=` is the
// row that proves the refusal above is narrow.
#[test]
fn a_condition_needs_a_field_but_a_value_may_be_empty() {
    let refusal = refused("=open");

    assert_eq!(
        refusal.to_string(),
        r#"cannot read "=open": a condition is field, operator and value"#
    );
    equals("status=", json!({"where": {"status": {"$eq": ""}}}));
}

// ---------------------------------------------------------------------------
// PHASE-04: the flag surface and the refusal catalogue.
//
// The compiler's core is proved above; these are the twelve flags, the one
// comma-list rule over the seven consumers that take a list, the two strictness
// rules, the two shorthand conflicts, and every sentence §3 fixes — carried as a
// table so a sentence cannot drift one row at a time.
// ---------------------------------------------------------------------------

// Every flag maps to the specification key it names, and to nothing else. The
// denotation is compared as an object *and* as the `Spec` it parses to, so a flag
// that spelled its key correctly but typed its value wrongly still fails.
#[test]
fn a_flag_sets_the_key_it_names() {
    // The two numeric flags take the token value rule, so `2` is the JSON number
    // and the traversal it bounds is the one the flags name.
    equals("--depth 2 --follow any", json!({"depth": 2, "follow": "*"}));
    // `--depth abc` is a denotation `spec::parse` must reject — the numeric flags
    // arrive as text so no numeric refusal is clap's — so it is compared without
    // routing it through `equals`, which requires a denotation that parses.
    assert_eq!(
        object("--depth abc --follow any"),
        json!({"depth": "abc", "follow": "*"})
    );
    equals(
        "--budget 5 --backlinks any",
        json!({"budget": 5, "backlinks": "*"}),
    );

    // `any` is the wildcard, spelled as the word the guide teaches; a comma list
    // is the named fields, in the order written.
    equals(
        "--follow part-of,mentions",
        json!({"follow": ["part-of", "mentions"]}),
    );
    equals("--backlinks any", json!({"backlinks": "*"}));

    // `--ids` is DEC-041's shorthand for the single-field projection.
    equals("--select a,b", json!({"select": ["a", "b"]}));
    equals("--ids", json!({"select": ["smut_id"]}));

    // A sort key's direction defaults to `asc`, and the emitted entry spells it
    // out rather than leaving the key ambiguous.
    equals(
        "--order title",
        json!({"order": [{"field": "title", "direction": "asc"}]}),
    );
    equals(
        "--order title:desc,status",
        json!({"order": [{"field": "title", "direction": "desc"}, {"field": "status", "direction": "asc"}]}),
    );

    equals("--limit 3", json!({"limit": 3}));
    equals("--edges", json!({"edges": true}));

    // The two existence flags are conditions of the same predicate, and each name
    // in the list is its own field.
    equals(
        "--has body,title",
        json!({"where": {"body": {"$exists": true}, "title": {"$exists": true}}}),
    );
    equals(
        "--missing body",
        json!({"where": {"body": {"$exists": false}}}),
    );

    // Roots keep their own order and the escape appends after them; an id the
    // grammar cannot carry bare needs no root at all.
    equals(
        "session --from=other",
        json!({"from": ["session", "other"]}),
    );
    equals("--from=1", json!({"from": ["1"]}));
}

// Reddens if a list flag emits a scalar (a comma list read as one name), or if an
// empty member is padded, skipped or collapsed instead of named.
#[test]
fn a_list_flag_is_a_list_and_an_empty_one_is_refused() {
    assert!(
        object("--select a,b")["select"].is_array(),
        "--select names fields, so it is a list even when the specification would take one"
    );
    assert!(
        object("--follow part-of,mentions")["follow"].is_array(),
        "--follow names edges, so it is a list"
    );

    // One sentence serves every list flag, and it quotes the list as typed. An
    // empty flag value names the flag, because there is no list text to quote.
    for (line, token) in [
        ("--select a,,b", "a,,b"),
        ("--follow part-of,", "part-of,"),
        ("--has a,,b", "a,,b"),
        ("--order title,,status", "title,,status"),
        ("--follow=", "--follow="),
        ("--select=", "--select="),
        ("--from=", "--from="),
    ] {
        assert_eq!(
            refused(line).to_string(),
            format!("cannot read \"{token}\": a comma list is a list of names, and one is empty"),
            "{line}"
        );
    }
}

// Reddens if any sentence in §3 is edited, reordered, merged with its neighbour,
// or replaced by a variant's summary: each row is the whole rendered line, so a
// change anywhere inside it fails here rather than in a transcript.
#[test]
fn every_refusal_names_what_was_typed_and_never_a_field_it_invented() {
    // Design §3's front-end table in its own order, then the plan's three owed
    // rows (`title~a,b`, `--follow any,part-of`'s sibling case, the empty flag
    // values), each pinned as the line the caller sees.
    for (line, sentence) in [
        (
            "status~a=b",
            r#"cannot read "status~a=b": it carries both ~ and =; use smut query for the same predicate as JSON"#,
        ),
        (
            "=open",
            r#"cannot read "=open": a condition is field, operator and value"#,
        ),
        (
            "status=a,,b",
            r#"cannot read "status=a,,b": a comma list is a list of values, and one is empty"#,
        ),
        (
            "status!=a,b",
            r#"cannot read "status!=a,b": a comma list means one of them, which != cannot express; write two conditions or use smut query"#,
        ),
        (
            "status=open status=resolved",
            "field status cannot be satisfied: $eq is given two different values",
        ),
        (
            r#"{"where":{"status":"open"}}"#,
            "q takes field=value tokens, not a JSON specification; use smut query for a JSON spec",
        ),
        (
            "--limit all",
            "the limit is a positive number and has no unlimited spelling; use smut get to read everything",
        ),
        (
            "--ids --select title",
            "--ids is select smut_id; it cannot also be given --select",
        ),
        (
            "--depth 2",
            "--depth needs a direction: add --follow, --backlinks, or drop it",
        ),
        (
            "--budget 5",
            "--budget needs a direction: add --follow, --backlinks, or drop it",
        ),
        (
            "status!=a status!=b",
            r#"cannot read "status!=a status!=b": q carries one != per field; use smut query"#,
        ),
        (
            "title~a title~b",
            r#"cannot read "title~a title~b": q carries one ~ per field; use smut query"#,
        ),
        (
            "--has status --missing status",
            "field status cannot be satisfied: $exists is given both true and false",
        ),
        (
            "--select a,,b",
            r#"cannot read "a,,b": a comma list is a list of names, and one is empty"#,
        ),
        (
            "--follow any,part-of",
            r#"cannot read "any,part-of": any is the wildcard, and stands alone"#,
        ),
        (
            "--order title:user",
            r#"cannot read "title:user": a direction is asc or desc"#,
        ),
        (
            "a=b=c",
            r#"cannot read "a=b=c": a field name holding = is outside q's grammar; use smut query"#,
        ),
        (
            "title~a,b",
            r#"cannot read "title~a,b": a comma list means one of them, which ~ cannot express; write two conditions or use smut query"#,
        ),
        (
            "--follow=",
            r#"cannot read "--follow=": a comma list is a list of names, and one is empty"#,
        ),
        (
            "--from=",
            r#"cannot read "--from=": a comma list is a list of names, and one is empty"#,
        ),
    ] {
        assert_eq!(refused(line).to_string(), sentence, "{line}");
    }
}

// Reddens if the pipeline order changes: a flag defect reported before a token
// defect, the flag table read in argv order, or `where` emitted from a `HashMap`
// — which would make the same request compile to different bytes twice.
#[test]
fn the_same_two_defects_refuse_the_same_one_every_time() {
    // Tokens are read before flags, so the token's own defect wins.
    assert_eq!(
        refused("=open --limit all").to_string(),
        r#"cannot read "=open": a condition is field, operator and value"#,
        "a token defect is read before any flag is"
    );

    // Flags are read in the design's table order (depth before limit), not in the
    // order they appear on the line.
    assert_eq!(
        refused("--limit all --depth 2").to_string(),
        "--depth needs a direction: add --follow, --backlinks, or drop it",
        "the flag table's order decides, and depth precedes limit in it"
    );

    // Asking three times is asking the same question.
    let once = refused("--depth 2");

    for attempt in 1..=3 {
        assert_eq!(refused("--depth 2"), once, "attempt {attempt}");
    }

    // The emitted `where` is a `BTreeMap`, so a condition's key order is the
    // field's name and not the order its token was written in.
    assert_eq!(
        serde_json::to_string(&object("title~graph status=open")).expect("the object serialises"),
        r#"{"where":{"status":{"$eq":"open"},"title":{"$contains":"graph"}}}"#,
        "the source order was title first, so this pins the map's order and the bytes"
    );
}

// Reddens if a refusal test loses the sentence saying what change would redden it,
// or if the note drifts somewhere other than above the `#[test]` it is keyed to —
// the note is the only record of which behaviour each assertion is holding. The
// floor keeps a scanner that matches nothing from passing vacuously.
#[test]
fn each_refusal_test_names_the_change_that_reddens_it() {
    let source = include_str!("dsl.rs");
    let lines: Vec<&str> = source.lines().collect();
    let mut checked = 0;

    for (index, line) in lines.iter().enumerate() {
        if !line.starts_with("fn ") {
            continue;
        }

        let is_test = lines[..index]
            .iter()
            .rev()
            .take(2)
            .any(|above| above.trim() == "#[test]");

        if !is_test {
            continue;
        }

        let end = lines[index + 1..]
            .iter()
            .position(|line| line.trim() == "#[test]")
            .map_or(lines.len(), |offset| index + 1 + offset);

        if !lines[index..end].join("\n").contains("refused(") {
            continue;
        }

        checked += 1;

        assert!(
            lines[index.saturating_sub(14)..index]
                .iter()
                .any(|above| above.contains("Reddens")),
            "{line} asserts a refusal and says nothing about what would redden it"
        );
    }

    assert!(
        checked >= 10,
        "only {checked} refusal tests were checked, so this guard is not guarding the suite"
    );
}

// Reddens if the numeric flags stop going through `value()`: `--depth 2` would
// reach the specification as the text `"2"`, and `--depth abc` and `--depth=-1`
// would each need a sentence of their own instead of the specification's one.
#[test]
fn numeric_flags_are_read_by_the_same_value_rule_as_a_token() {
    assert_eq!(object("--depth 2 --follow any")["depth"], json!(2));
    assert_eq!(object("--depth abc --follow any")["depth"], json!("abc"));
    assert_eq!(object("--depth=-1 --follow any")["depth"], json!(-1));

    // The asymmetry, from both sides: the front-end refuses a depth with no
    // direction, and the specification accepts the same depth as JSON. The
    // front-end is the stricter layer and never the different one.
    assert_eq!(
        refused("--depth 2"),
        Reject::MissingDirection {
            flag: "--depth".to_owned(),
        }
    );
    assert!(
        spec::parse(&json!({"depth": 2})).is_ok(),
        "the specification accepts a depth with no direction; refusing it there would be a second engine"
    );

    // What does reach `spec::parse` arrives as a value, so its sentence is the
    // specification's, and it names the key rather than the flag.
    for (line, sentence) in [
        (
            "--depth abc --follow any",
            "depth must be a nonnegative integer",
        ),
        (
            "--depth=-1 --follow any",
            "depth must be a nonnegative integer",
        ),
        ("--limit 0", "limit must be a positive integer"),
        (
            "--budget 0 --follow any",
            "budget must be a positive integer",
        ),
    ] {
        let refusal = match spec::parse(&object(line)) {
            Ok(spec) => panic!("{line:?} parsed to {spec:?}, and should have been refused"),
            Err(refused) => refused,
        };

        assert_eq!(refusal.to_string(), sentence, "{line}");
    }

    // `--limit all` never gets that far: there is no unlimited spelling to parse,
    // and reading everything is `get`'s work.
    assert_eq!(
        refused("--limit all").to_string(),
        "the limit is a positive number and has no unlimited spelling; use smut get to read everything"
    );
}
