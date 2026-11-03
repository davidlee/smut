//! The short form's compiler: one argument line of words, and the specification
//! object it denotes. The human verbs `q` and `r` share it.
//!
//! This is a front-end, not a second engine. It emits the object [`crate::spec`]
//! already reads and lets that read it, so "the short form adds no semantics" is
//! structural rather than a convention: the specification is what reads this
//! module's output, and the whole surface is one table of equalities in
//! `tests/dsl.rs`. The decisions are DEC-034 (the verb), DEC-035 (the value rule
//! and its refinement), DEC-036 (bare roots and the escape), DEC-037 (the clap
//! shape above this module), DEC-038 (this seam), DEC-039 (refuse only what the
//! specification cannot spell), DEC-040 (what the guide teaches) and DEC-041 (the
//! selection shorthand).
//!
//! Nothing here reads a store, the filesystem or the environment: `compile` is a
//! pure function of the line it is handed, so a malformed line is refused before
//! a store is opened, locked or recovered (DEC-004's step 1).

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::error::Reject;

/// One `smut q` argument line, already lexed by the shell and cut into flags by
/// clap.
pub struct Line<'a> {
    /// The bare positional words, in source order.
    pub tokens: &'a [String],
    /// The flags, as clap handed them over.
    pub flags: Flags,
}

/// The flags `smut q` accepts: one per promoted specification key, plus four
/// shorthands.
///
/// The numeric and enumeration flags arrive as text, so that no numeric refusal is
/// clap's: with a direction present, `--depth abc` and `--depth=-1` reach the same
/// specification message, and `--limit all` reaches the front-end's own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flags {
    /// `--follow`: the outgoing fields to traverse.
    pub follow: Option<String>,
    /// `--backlinks`: the incoming fields to traverse.
    pub backlinks: Option<String>,
    /// `--depth`: how far the traversal goes.
    pub depth: Option<String>,
    /// `--budget`: how many entity expansions the traversal may spend.
    pub budget: Option<String>,
    /// `--select`: the fields to project.
    pub select: Option<String>,
    /// `--ids`: project `smut_id` alone, which is `select: ["smut_id"]`.
    pub ids: bool,
    /// `--order`: the sort keys, each with an optional direction.
    pub order: Option<String>,
    /// `--limit`: the result cap.
    pub limit: Option<String>,
    /// `--edges`: supply a traversal with no followed fields, for its edges.
    pub edges: bool,
    /// `--has`: the fields that must be present, which is `$exists: true`.
    pub has: Option<String>,
    /// `--missing`: the fields that must be absent, which is `$exists: false`.
    pub missing: Option<String>,
    /// `--from`: the root escape for an id the grammar cannot carry bare.
    pub from: Option<String>,
}

/// Compiles one argument line to the specification object it denotes.
///
/// Tokens are read first, in source order, and then the flags in the fixed order
/// `follow`, `backlinks`, `depth`, `budget`, `select`, `ids`, `order`, `limit`,
/// `edges`, `has`, `missing`, `from` — the design's table order, not the order
/// they were written. So a line with two defects always reports the same one,
/// however it was spelled on the command line — the discipline
/// [`crate::spec::parse`] already keeps.
pub fn compile(line: &Line<'_>) -> Result<Value, Reject> {
    let mut roots: Vec<Value> = Vec::new();
    let mut filter = Where::default();
    let mut spec = Map::new();
    let flags = &line.flags;

    for token in line.tokens {
        // The classification order of design §1's flowchart: the JSON operand is
        // decided by the first character, before any operator is looked for — a
        // spec of `{"status":"open"}` carries `=` and must not reach the split.
        if token.starts_with('{') {
            return Err(Reject::JsonOperand);
        }

        match spelled_operators(token).as_slice() {
            [] => roots.push(Value::String(token.clone())),
            [one] => {
                let (at, spelling, key) = *one;
                let field = &token[..at];

                if field.is_empty() {
                    return Err(unreadable(
                        token,
                        "a condition is field, operator and value",
                    ));
                }

                let raw = &token[at + spelling.len()..];

                filter.add_token(field, spelling, key, raw, token)?;
            }
            found => return Err(ambiguous(token, found)),
        }
    }

    // The flags are read in the order below and no other — follow, backlinks, depth,
    // budget, select, ids, order, limit, edges, has, missing, from. A line with two
    // flag defects reports the one earliest in that table, so the order the caller
    // typed them in never decides what they are told.
    if let Some(raw) = &flags.follow {
        spec.insert("follow".to_owned(), traversal("--follow", raw)?);
    }

    if let Some(raw) = &flags.backlinks {
        spec.insert("backlinks".to_owned(), traversal("--backlinks", raw)?);
    }

    // The direction test is the flag's presence and not its value, and `--edges` is not
    // a direction: it asks for an edge set, which is not something to walk.
    if let Some(raw) = &flags.depth {
        if flags.follow.is_none() && flags.backlinks.is_none() {
            return Err(Reject::MissingDirection {
                flag: "--depth".to_owned(),
            });
        }

        // The same rule a token's value gets, so no numeric refusal is clap's and
        // `--depth 2` is the number `2` rather than its text (DEC-035).
        spec.insert("depth".to_owned(), value(raw));
    }

    if let Some(raw) = &flags.budget {
        if flags.follow.is_none() && flags.backlinks.is_none() {
            return Err(Reject::MissingDirection {
                flag: "--budget".to_owned(),
            });
        }

        spec.insert("budget".to_owned(), value(raw));
    }

    if let Some(raw) = &flags.select {
        // The pair is one projection spelled two ways, so it is refused by name rather
        // than resolved by precedence (DEC-041) — before the list is read.
        if flags.ids {
            return Err(Reject::ConflictingSelection);
        }

        spec.insert("select".to_owned(), names(&flag_names("--select", raw)?));
    }

    if flags.ids {
        // `--select` was read first and refuses the pair, so reaching here means the
        // projection is `--ids`'s alone: DEC-041's two-call recipe's exact shape.
        spec.insert(
            "select".to_owned(),
            Value::Array(vec![Value::String("smut_id".to_owned())]),
        );
    }

    if let Some(raw) = &flags.order {
        let entries = flag_names("--order", raw)?
            .into_iter()
            .map(sort_key)
            .collect::<Result<Vec<Value>, Reject>>()?;

        spec.insert("order".to_owned(), Value::Array(entries));
    }

    if let Some(raw) = &flags.limit {
        // There is no unlimited limit to read: the parsed `Limit` has no
        // user-reachable unlimited form (DEC-001 rule 4), and reading everything is
        // `smut get`'s work.
        if raw == "all" {
            return Err(Reject::UnreachableLimit);
        }

        spec.insert("limit".to_owned(), value(raw));
    }

    if flags.edges {
        spec.insert("edges".to_owned(), Value::Bool(true));
    }

    // `--has` and `--missing` merge into the same `where` as the tokens, because they
    // are conditions of the same predicate: `--has body status=open` is one filter.
    if let Some(raw) = &flags.has {
        for field in flag_names("--has", raw)? {
            filter.add_exists(field, true, raw)?;
        }
    }

    if let Some(raw) = &flags.missing {
        for field in flag_names("--missing", raw)? {
            filter.add_exists(field, false, raw)?;
        }
    }

    // The escape is one id and not a list — a comma may be part of the id it carries —
    // and it appends to the roots in write order, after the bare ones.
    if let Some(raw) = &flags.from {
        if raw.is_empty() {
            return Err(unreadable(
                "--from=",
                "--from carries one id, and none was given",
            ));
        }

        roots.push(Value::String(raw.clone()));
    }

    // `from` and `where` are written last because the tokens and the flags above are
    // what fill them.
    if !roots.is_empty() {
        spec.insert("from".to_owned(), Value::Array(roots));
    }

    if let Some(predicate) = filter.into_value() {
        spec.insert("where".to_owned(), predicate);
    }

    Ok(Value::Object(spec))
}

/// The value a bare token denotes: a JSON scalar — boolean, number or null — when
/// the token is one, its own text otherwise. A number compares as a number, so
/// `priority=1.0` equals no stored integer and no string.
///
/// A JSON *string* literal is deliberately not accepted here. The shell has
/// already removed the quotes the caller typed, so a token that still looks like a
/// quoted string is one whose quotes were meant — unquoting it would strip two
/// characters nobody asked to lose.
fn value(raw: &str) -> Value {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Bool(literal)) => Value::Bool(literal),
        Ok(Value::Number(number)) => Value::Number(number),
        Ok(Value::Null) => Value::Null,
        _ => Value::String(raw.to_owned()),
    }
}

/// The operator spellings, longest first so `!=` is matched before `=`.
///
/// The JSON names here are the third site that spells the five predicate
/// operators; the other two are in `predicate`'s condition parser and its
/// misplaced-operator check (IMP-016). A sixth operator joins all three, and this
/// table is the one place it joins in this module.
const OPERATORS: [(&str, &str); 3] = [("!=", "$ne"), ("=", "$eq"), ("~", "$contains")];

/// The two operators the flags supply, because their own spellings are
/// shell-hostile.
const EXISTS: &str = "$exists";
/// What a comma list on `=` denotes.
const IN: &str = "$in";
/// The operator that carries a comma list, spelled as the caller types it.
const EQUALS: &str = "=";

/// The operators that take one value each, so a comma list on them is a list this
/// surface has no spelling for.
const SINGLE_VALUE: [&str; 2] = ["!=", "~"];

/// The wildcard's spelling on this surface: `*` would glob, so the word the guide
/// teaches is `any`.
const ANY: &str = "any";
/// What `any` denotes: the specification's own wildcard.
const WILDCARD: &str = "*";

/// The two directions a sort key may carry, and the one it means when none is written.
const ASCENDING: &str = "asc";
const DESCENDING: &str = "desc";

/// Every operator a token carries, with its byte offset, in source order.
///
/// Scanning the table rather than the characters keeps one spelling of the
/// operators in the module, and `!=` is taken whole so its `=` is never counted
/// twice. A token with no operator is a root id: the caller's own text.
fn spelled_operators(token: &str) -> Vec<(usize, &'static str, &'static str)> {
    let mut found = Vec::new();
    let mut at = 0;

    while let Some(rest) = token.get(at..) {
        match OPERATORS
            .iter()
            .find(|(spelling, _)| rest.starts_with(*spelling))
        {
            Some(&(spelling, key)) => {
                found.push((at, spelling, key));
                at += spelling.len();
            }
            None => match rest.chars().next() {
                Some(character) => at += character.len_utf8(),
                None => break,
            },
        }
    }

    found
}

/// The refusal for a token carrying more than one operator.
///
/// The same operator twice is one token the front-end cannot split into field and
/// value, because the second operator may sit in either half; two different
/// operators are a token nobody can split without guessing. Naming either repair in
/// the token's own characters is the whole migration story (DEC-039), and the
/// same-operator sentence names neither half, so it stays true of both.
fn ambiguous(token: &str, found: &[(usize, &'static str, &'static str)]) -> Reject {
    let (_, first, _) = found[0];
    let (_, second, _) = found[1];

    if found.iter().all(|(_, spelling, _)| *spelling == first) {
        unreadable(
            token,
            format!(
                "it carries two {first}, so the short form cannot tell the field from \
                 the value; use -j"
            ),
        )
    } else {
        unreadable(
            token,
            format!("it carries both {first} and {second}; use -j for the same predicate as JSON"),
        )
    }
}

/// A refusal that quotes the caller's own words.
fn unreadable(token: impl Into<String>, reason: impl Into<String>) -> Reject {
    Reject::UnreadableToken {
        token: token.into(),
        reason: reason.into(),
    }
}

/// What a comma list's members are, for the one sentence that names an empty one.
#[derive(Clone, Copy)]
enum ListKind {
    /// An `=` token's operand list: `status=open,resolved`.
    Values,
    /// A flag's list of field names: `--select title,body`.
    Names,
}

impl ListKind {
    /// The word the refusal calls the members.
    fn word(self) -> &'static str {
        match self {
            ListKind::Values => "values",
            ListKind::Names => "names",
        }
    }
}

/// The refusal an empty member earns, in whichever list it was written.
fn empty_member(spelled: &str, kind: ListKind) -> Reject {
    unreadable(
        spelled,
        format!(
            "a comma list is a list of {}, and one is empty",
            kind.word()
        ),
    )
}

/// The members of a comma list, in the order written — this module's only splitter.
///
/// A comma means a list only where the surface says so: on `=` it is `$in`, and on a
/// flag list it is one name per field. An empty member is refused rather than padded or
/// dropped, because the list the caller wrote is not the list they meant, and `spelled`
/// is how that refusal quotes them.
fn members<'a>(raw: &'a str, spelled: &str, kind: ListKind) -> Result<Vec<&'a str>, Reject> {
    let listed: Vec<&str> = raw.split(',').collect();

    if listed.iter().any(|member| member.is_empty()) {
        return Err(empty_member(spelled, kind));
    }

    Ok(listed)
}

/// The names a flag's list holds, quoting the list as the caller typed it — the list's
/// own text, or the flag with `=` when it is empty and there is no text to quote.
fn flag_names<'a>(flag: &str, raw: &'a str) -> Result<Vec<&'a str>, Reject> {
    let spelled = if raw.is_empty() {
        format!("{flag}=")
    } else {
        raw.to_owned()
    };

    members(raw, &spelled, ListKind::Names)
}

/// A list of field names as the specification spells it: an array, in the order written.
fn names(fields: &[&str]) -> Value {
    Value::Array(
        fields
            .iter()
            .map(|field| Value::String((*field).to_owned()))
            .collect(),
    )
}

/// A traversal flag's value as the specification spells it: the wildcard `"*"` when the
/// whole value is the word `any`, and otherwise the list of field names written.
///
/// `any` inside a longer list is refused rather than read as a field name: the wildcard
/// stands alone, and a caller who wrote it beside a name meant both.
fn traversal(flag: &str, raw: &str) -> Result<Value, Reject> {
    if raw == ANY {
        return Ok(Value::String(WILDCARD.to_owned()));
    }

    let fields = flag_names(flag, raw)?;

    if fields.contains(&ANY) {
        return Err(unreadable(raw, "any is the wildcard, and stands alone"));
    }

    Ok(names(&fields))
}

/// One `--order` entry: `field`, or `field:direction`.
///
/// The direction defaults to `asc`, and the emitted entry always spells it out rather
/// than leaving the key ambiguous. A direction that is neither word is refused with the
/// entry as typed, because there is no third direction to fall back to.
fn sort_key(entry: &str) -> Result<Value, Reject> {
    let (field, direction) = match entry.split_once(':') {
        Some((field, direction)) => (field, direction),
        None => (entry, ASCENDING),
    };

    if direction != ASCENDING && direction != DESCENDING {
        return Err(Reject::UnknownDirection {
            entry: entry.to_owned(),
        });
    }

    let mut key = Map::new();
    key.insert("field".to_owned(), Value::String(field.to_owned()));
    key.insert("direction".to_owned(), Value::String(direction.to_owned()));

    Ok(Value::Object(key))
}

/// The `where` object under construction: one condition per operator per field.
#[derive(Default)]
struct Where {
    /// Field, then the operator's spelling, then its condition. A `BTreeMap` so the
    /// emitted object has one key order and the same line compiles to the same
    /// bytes twice in a row.
    fields: BTreeMap<String, BTreeMap<&'static str, Condition>>,
}

/// One condition, and the token it was read from.
struct Condition {
    /// The token as typed, so a repeat can quote both tokens the caller wrote.
    spelled: String,
    /// The key inside the field's condition object.
    key: &'static str,
    /// The operand the condition carries.
    operand: Value,
}

impl Where {
    /// Adds the condition a bare token carries.
    fn add_token(
        &mut self,
        field: &str,
        spelling: &'static str,
        key: &'static str,
        raw: &str,
        token: &str,
    ) -> Result<(), Reject> {
        // A comma list is a list only where the surface says so: on `=` it is `$in`. On
        // the single-value operators it has no spelling at all, which is said before the
        // value is read — the operator is what cannot hold it, so the value is not the
        // thing to complain about.
        if raw.contains(',') {
            if spelling == EQUALS {
                let listed = members(raw, token, ListKind::Values)?;

                return self.insert(
                    field,
                    spelling,
                    Condition {
                        spelled: token.to_owned(),
                        key: IN,
                        operand: Value::Array(listed.into_iter().map(value).collect()),
                    },
                );
            }

            if SINGLE_VALUE.contains(&spelling) {
                return Err(unreadable(
                    token,
                    format!(
                        "a comma list means one of them, which {spelling} cannot express; \
                         write two conditions or use -j"
                    ),
                ));
            }
        }

        self.insert(
            field,
            spelling,
            Condition {
                spelled: token.to_owned(),
                key,
                operand: value(raw),
            },
        )
    }

    /// Adds one `$exists` condition, from `--has` or `--missing`.
    fn add_exists(&mut self, field: &str, present: bool, spelled: &str) -> Result<(), Reject> {
        self.insert(
            field,
            EXISTS,
            Condition {
                spelled: spelled.to_owned(),
                key: EXISTS,
                operand: Value::Bool(present),
            },
        )
    }

    /// One condition per operator per field.
    ///
    /// An exact repeat collapses into the condition already there. A repeat the
    /// emitted object cannot carry is refused in the caller's own tokens, because
    /// the condition object has one key per operator — and only the case that is
    /// truly unsatisfiable is refused in the contradiction voice, which would be
    /// false of the others (DEC-027, DEC-039).
    fn insert(
        &mut self,
        field: &str,
        spelling: &'static str,
        condition: Condition,
    ) -> Result<(), Reject> {
        let held = self.fields.entry(field.to_owned()).or_default();

        let Some(previous) = held.get(spelling) else {
            held.insert(spelling, condition);

            return Ok(());
        };

        if previous.key == condition.key && previous.operand == condition.operand {
            return Ok(());
        }

        let contradiction = match spelling {
            // Two different single values cannot both hold, and the condition object
            // carries one `$eq`, so this joins DEC-027/DEC-030's family rather than
            // starting a second one.
            EQUALS if previous.key == condition.key && condition.key != IN => {
                Some("$eq is given two different values")
            }
            // `$exists: true` beside `$exists: false`: unsatisfiable for a store that
            // has the field and for one that does not.
            EXISTS => Some("$exists is given both true and false"),
            _ => None,
        };

        match contradiction {
            Some(contradiction) => Err(Reject::ContradictoryFieldConditions {
                field: field.to_owned(),
                contradiction: contradiction.to_owned(),
            }),
            None => Err(unreadable(
                format!("{} {}", previous.spelled, condition.spelled),
                format!("the short form carries one {spelling} per field; use -j"),
            )),
        }
    }

    /// The `where` object, or nothing when no condition was read — `{}` is a
    /// request with no filter, and an empty `where` is not the same thing.
    fn into_value(self) -> Option<Value> {
        if self.fields.is_empty() {
            return None;
        }

        let mut predicate = Map::new();

        for (field, conditions) in self.fields {
            let mut condition = Map::new();

            for (_, held) in conditions {
                condition.insert(held.key.to_owned(), held.operand);
            }

            predicate.insert(field, Value::Object(condition));
        }

        Some(Value::Object(predicate))
    }
}
