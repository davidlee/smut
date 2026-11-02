# IDE-007: $regex as a JSON-only sixth predicate operator

## What

A field condition that matches a regular expression against a string field:

```console
$ smut query '{"where":{"title":{"$regex":"^Hand"}}}'
```

The operand is a JSON string carrying the pattern — the same shape `$contains`
already takes for the substring case, so the wire surface grows by one operator
and nothing else.

## Why

`$contains` is a bare substring test with no anchors, alternation or classes.
Callers who want "starts with", "either of these two prefixes", or a shape check
currently do one of three things: over-fetch and filter with `jq`, expand the
predicate into an unreadable `$or` of `$contains` clauses, or give up. The first
is the common one, and it costs the walk semantics the query surface exists to
provide.

## The four gates, in descending authority

1. **The brief bans it by name.** `smut-implementation-brief.org:140` — *"No
   coercion, regex, JSONPath, nested-field syntax, executable predicates or
   Lua."* `src/predicate.rs:6` repeats it as an invariant and SL-002's plan
   asserted it (`EX-2`). So this is a product-authority amendment first: a REQ
   and a decision, then code.
2. **The operator census must be one site first.** The five operators are
   spelled in `predicate::parse_condition`, its misplaced-operator table and the
   unknown-operator path, and `src/dsl.rs`'s `OPERATORS` is a third. RV-027
   recorded the exact trap: adding `$regex` to `parse_condition` alone makes
   `{"where":{"$regex":"x"}}` render *"unknown predicate operator: $regex"* — a
   false sentence about a defined operator. That is IMP-016, and this item
   `needs` it.
3. **The DSL cannot carry it.** `(`, `)`, `|`, `[`, `]`, `$` and `*` are all
   shell-active and all banned from the grammar. DEC-034's basis already
   deferred regex *and* grouping to `smut query` as the complete surface. So the
   operator is JSON-only by design, and its callers pay the quotes — no
   ergonomics win, only capability.
4. **It overlaps `$contains`.** `$contains` means substring on a string and
   member equality on an array. Regex subsumes the string half, so the design
   must say whether `$contains` stays the ordinary spelling, whether it becomes
   legacy, or whether the regex is anchored to substrings explicitly.

## Decisions a slice would have to settle

- **Compile at parse time.** An invalid pattern must be a `Reject` at exit 2,
  before a store exists (DEC-004). Compiling per-entity would make the refusal
  unreachable and the cost quadratic.
- **Decidability (DEC-027).** Two `$regex` conditions on one field are not
  provably unsatisfiable, so they must be *taught*, not refused — the opposite
  of the two-`=` case. A `$regex` beside `$eq` is decidable only in narrow
  cases; default to allowing.
- **Non-string fields.** `$contains` refuses to coerce; `$regex` should match
  only `Value::String` and answer false elsewhere, never stringify a number.
- **Anchoring.** Decide whether the pattern is a full-string match or a search.
  Rust's `Regex::is_match` is a search; `^…$` gives the other reading. One
  sentence in the guide, one test.
- **Dependency weight.** `regex` pulls regex-automata, aho-corasick and
  memchr (linear-time, no ReDoS). The crate currently depends on five small
  crates; if that is unacceptable, the honest fallback is anchored substring
  matching, not a hand-rolled partial regex.

## Non-goals

- Not on the `q` surface (gate 3), and not a `--where` passthrough — IDE-006
  cut that for reintroducing quoting.
- Not an operator on arrays or embedded objects; that is `$contains`'s member
  equality and it works today.
- No store-format implication: nothing lands inside `.smut/`, so STD-001/STD-002
  are untouched.
