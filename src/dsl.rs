//! `smut q`'s compiler: one argument line of words, and the specification object
//! it denotes.
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
/// clap's: `--depth abc` and `--depth=-1` reach the same specification message, and
/// `--limit all` reaches the front-end's own.
#[derive(Clone, Debug, Default)]
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
/// Refusals are raised in source order for tokens and in a fixed order for flags,
/// so a line with two defects always reports the same one — the discipline
/// [`crate::spec::parse`] already keeps.
pub fn compile(line: &Line<'_>) -> Result<Value, Reject> {
    let mut roots: Vec<Value> = Vec::new();
    let mut filter = Where::default();

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

    // `--has` and `--missing` merge into the same `where` as the tokens, because
    // they are conditions of the same predicate, and they are read in `Flags`'
    // order — so a line carrying both defects reports the same one every time.
    if let Some(fields) = &line.flags.has {
        for field in members(fields) {
            filter.add_exists(field, true, fields)?;
        }
    }

    if let Some(fields) = &line.flags.missing {
        for field in members(fields) {
            filter.add_exists(field, false, fields)?;
        }
    }

    let mut spec = Map::new();

    if !roots.is_empty() {
        spec.insert("from".to_owned(), Value::Array(roots));
    }

    if let Some(predicate) = filter.into_value() {
        spec.insert("where".to_owned(), predicate);
    }

    if let Some(fields) = &line.flags.follow {
        spec.insert("follow".to_owned(), names(fields));
    }

    if let Some(fields) = &line.flags.backlinks {
        spec.insert("backlinks".to_owned(), names(fields));
    }

    if let Some(raw) = &line.flags.depth {
        // The same rule a token's value gets, so no numeric refusal is clap's and
        // `--depth 2` is the number `2` rather than its text (DEC-035).
        spec.insert("depth".to_owned(), value(raw));
    }

    if line.flags.ids {
        spec.insert(
            "select".to_owned(),
            Value::Array(vec![Value::String("smut_id".to_owned())]),
        );
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
/// The same operator twice is a field name holding it, which the grammar cannot
/// address; two different operators are a token nobody can split without guessing.
/// Naming either repair in the token's own characters is the whole migration story
/// (DEC-039).
fn ambiguous(token: &str, found: &[(usize, &'static str, &'static str)]) -> Reject {
    let (_, first, _) = found[0];
    let (_, second, _) = found[1];

    if found.iter().all(|(_, spelling, _)| *spelling == first) {
        unreadable(
            token,
            format!("a field name holding {first} is outside q's grammar; use smut query"),
        )
    } else {
        unreadable(
            token,
            format!(
                "it carries both {first} and {second}; use smut query for the same predicate \
                 as JSON"
            ),
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

/// The members of a comma list, in the order written.
///
/// A comma means a list only where the surface says so: on `=` it is `$in`, and on
/// a field list it is one condition per name. Everywhere else the comma is the
/// caller's own text.
fn members(raw: &str) -> Vec<&str> {
    raw.split(',').collect()
}

/// A field list as the specification spells it: an array of names, in the order
/// written.
fn names(raw: &str) -> Value {
    Value::Array(
        members(raw)
            .into_iter()
            .map(|field| Value::String(field.to_owned()))
            .collect(),
    )
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
        // A comma list on `=` is `$in`; on every other operator the comma is the
        // caller's own text and the value is read whole.
        let (key, operand) = if spelling == EQUALS && raw.contains(',') {
            let listed = members(raw).into_iter().map(value).collect();

            (IN, Value::Array(listed))
        } else {
            (key, value(raw))
        };

        self.insert(
            field,
            spelling,
            Condition {
                spelled: token.to_owned(),
                key,
                operand,
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
                format!("q carries one {spelling} per field; use smut query"),
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
