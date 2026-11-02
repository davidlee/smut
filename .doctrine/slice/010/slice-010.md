# A humane query surface — argv is the DSL: the grammar compiles to the ten-key spec and adds no semantics

## Context

The machine surface is fine and stays exactly as it is. Ten keys, five operators,
two traversal directions (`smut-implementation-brief.org` § Query contract) is a
small, well-shaped specification. What makes it miserable at a prompt is not its
size — it is that **the shell eats almost all of JSON's punctuation, so every
character has to be paid for with a quote**:

```console
$ smut query '{"where":{"title":{"$contains":"graph"},"status":"open"}}'
              └─────────── 6 quotes, and $eq/$in/$exists stay hostile inside them ─┘
```

The guide's own five recipes cost 54–95 characters and carry 8–16 quotes
apiece; `$` is shell-active inside double quotes; and `follow` vs `backlinks` —
the thing the guide itself calls "what is easy to get wrong" — sits at column
40, mid-object, where direction is the eleventh thing you read instead of the
first.

That last point is what connects this slice to the evidence already gathered.
RFC-001 ranked 0.2.0's agent-facing findings by expected harm: three fail
*silently* (Tier 1), and the rest "cost calls or reading" (Tiers 2–3). This slice
buys nothing in Tier 1 — a typo'd field still answers empty at exit 0, and that
is IMP-011/IMP-012's work, not this. It buys Tier 2/3: fewer calls, no quoting,
and a direction token you can *see*:

```
smut q session --backlinks part-of --depth 2 --ids     # the questions under the session
smut q session --follow    part-of --depth 2 --ids     # the session, alone
```

**The design is IDE-006's** (humane query DSL) and is not this slice's work to
reinvent. Its spine: define the grammar over **argv tokens**, not over a string,
so a bare word is a token, a space is a separator, and the shell has already done
the lexing. Two spec keys are *demoted* to the hot path — `from` to bare words,
`where` to infix conditions — and the remaining eight are *promoted* to flags of
the same name. The invariant that keeps it a front-end instead of a second
engine:

> `parse(argv) -> Spec`. The DSL **compiles to the existing specification and adds
> no semantics.** Anything that cannot be expressed as a `Spec` is not a DSL
> feature.

Binding constraints inherited, none of which this slice may relax:

- **The brief is the product authority** for the query contract; the DSL may be
  *stricter* than it, never different in meaning (§ Query contract).
- **DEC-011** keeps 0.1/0.2 `smut query` calls valid, and compatibility there is a
  **tested property** (`tests/compatibility.rs`), not a consequence of refusing
  input. The machine surface does not move: no key added, none removed, no
  refusal reworded.
- **DEC-004** orders step 1 — the whole argument line is validated *before* a
  store exists. `q` validates in the same place and creates nothing.
- **DEC-001 rule 1** — a `Reject` from the core is always exit 2, because the core
  can express nothing but user error; **rule 4** — the parsed `Limit` has no
  publicly reachable unlimited form, so `--limit all` cannot be a flag.
- **DEC-008** — the CLI grammar is clap's; **DEC-010** — the embedded guide is
  the delivery channel, so a verb the guide does not teach does not exist.
- **IDE-006's shell-safety constraint**, which is a design veto, not a style note:
  `*`, `?`, `[`, `]`, `$`, leading `!`, `(`, `)`, `|`, `&`, `;`, `<`, `>`, leading
  `#`, leading `~`, backtick and backslash are all banned from bare tokens. The
  wildcard is therefore spelled `any`, and `$exists` is `--has`/`--missing`.

## Scope & Objectives

1. **One compiler, `parse(argv) -> Spec`, as a pure function.** No store, no
   argv beyond the tokens it is handed, no environment. The design settles *how*
   "adds no semantics" is enforced — either by emitting the spec object and
   routing it through the existing `spec::parse`, or by building `Spec` from the
   same parse helpers — but the property is not negotiable: **every DSL line
   equals the JSON spec it corresponds to**, asserted against `spec::parse` of
   that JSON.
2. **The token grammar.** A bare word is a root `smut_id`; `field=value`,
   `field!=value` and `field~value` are `$eq`, `$ne` and `$contains`; a comma
   list on `=` is `$in`; the word `any` is the wildcard `"*"`. The one rule to
   learn: *bare = a thing, `field=value` = a filter, `--word` = structure.*
3. **A flag per remaining key, spelled the same.** `--follow`, `--backlinks`,
   `--depth`, `--budget`, `--select`, `--order`, `--limit`, `--edges`; plus
   `--ids` (`select:["smut_id"]`, the two-call recipe's exact shape), `--has` /
   `--missing` (`$exists`), and `--from=ID` as the root escape for an id carrying
   `=`, `~` or a leading `-`.
4. **Refuse ambiguity; never guess.** A token carrying two operators
   (`status~a=b`), a JSON-looking positional (`q '{"where":…}'`), an unknown flag,
   a value whose comma list means something the spec cannot express
   (`status!=a,b`), and `--limit all`. Each refusal names the fix *and* the
   sibling — the JSON-positional refusal ("`q` takes `field=value`; use
   `smut query` for a JSON spec") is the whole migration story — and each is
   written under **DEC-030**: one line, no epigram, and it names no flag or field
   the caller did not type.
   **Repeated conditions on one field are not a DSL refusal** (research F1). The
   one shape JSON cannot spell — two `=` on the same field — is provably
   unsatisfiable and is refused as a contradiction in DEC-027/DEC-030's existing
   voice. Every other repeat (`status=open status!=resolved`) compiles, and
   `predicate::parse` judges it by demand and decidability, as it already does for
   JSON callers.
5. **The human surface may be stricter, in strictness only.** `--depth` or
   `--budget` with no direction is refused, though `spec::parse` accepts and
   ignores it: a human typed it and meant it. The same JSON stays valid — that
   asymmetry is a property to test, not a wart to hide.
6. **Shell-safety is a grammar constraint, tested.** No token the grammar admits
   needs a quote; the banned characters are absent by construction rather than
   escaped at the edge.
7. **Same engine, same output.** `q` produces the existing `Request::Query` and
   runs the existing pipeline, so the envelope, exit codes, store rules and
   traversal semantics are inherited without a second implementation. The same
   selection expressed both ways yields **byte-identical** output.
8. **The guide teaches it.** `src/learn.rs`'s hot-path recipes become DSL lines;
   `smut query` is documented as the complete surface for the tenth of cases that
   needs JSON; and the two warts are stated rather than hidden — value typing
   (§ OQ3) and `~`'s inheritance of `$contains`'s two meanings.

## Non-Goals

- **`--where JSON` passthrough.** It reintroduces quoting into `q` and creates a
  half-language. The escape is one command away.
- **Any new semantics.** Grouping, `$or`/`$not`, regex, nested paths, computed
  fields, pipelines, aggregation, an unlimited limit, or any operator the brief
  does not already have. `status=open,resolved` is the disjunction; a cross-field
  `$or` is `smut query`'s.
- **`--brief` and one-line output.** The human reader's other half of the pain,
  and IMP-002's presentation surface — not a parse concern. The envelope is the
  contract this slice preserves.
- **`smut render` sharing the grammar.** Genuinely cheap *later* precisely
  because the compiler is a pure function over tokens; but render's operand order
  and display flags are their own migration, and Q5 of IDE-006 named it as a
  sentence where it is a phase.
- **Shell completion and value completion.** The grammar must not *preclude*
  them (IDE-003's census wearing an ergonomics hat); shipping them is not this.
- **A REPL.**
- **Field-name silence.** A typo'd field answers empty at exit 0 today and will
  still answer empty through `q`. Refusing unknown fields is unsound in a
  schemaless store and impossible without a census; it is IMP-011/IMP-012's. This
  slice must not imply the DSL fixes it.
- **The machine surface.** `smut query`, `Render`, `Get`, `Delete` and every
  `Spec` key keep their exact bytes. No acceptance criterion here may require a
  change to `spec::parse`'s meaning.
- **README.** Operator-owned; the working tree already carries an in-flight README
  edit (a nix flake section) that CHR-001 forbids an agent to restage or resolve.
  `learn` is the guide's channel (DEC-010) and is in scope.
- **The store.** No file inside `.smut/`, so `docs/store-format.md` under STD-001
  is untouched.

## Affected surface

| path | what changes |
| --- | --- |
| `src/dsl.rs` *(new)* | the compiler: token classification, the three operators, the comma list, the flags, and the refusals DSL-only vocabulary needs |
| `src/lib.rs` | one `pub mod dsl;` |
| `src/cli.rs` | `Sub::Q` and its clap shape; the `Request::Query` reuse; the identity of the argv slice it hands the compiler |
| `src/main.rs` | the dispatch arm, if `q` reaches the pipeline by any path other than `Invocation::Request(Request::Query)` |
| `src/error.rs` | the DSL's `Reject` variants: two operators in one token, a duplicate field, a JSON operand, an unknown flag naming its nearest, `--limit all`, a comma list the operator cannot carry |
| `src/learn.rs` | the hot-path recipes as DSL lines; `query` as the complete surface; the value-typing and `$contains` warts |
| `src/spec.rs` | nothing semantic. Only if the design lets the compiler build the spec object directly and a helper (`Limit::capped`) needs a visibility it does not have |
| `tests/dsl.rs` *(new)* | the compiler's own suite: the worked-examples table as DSL→`Spec` equalities, token hazards, every refusal, and the strictness asymmetry |
| `tests/cli_contract.rs` | the verb end-to-end: byte-identical envelope against `query`, exit codes, `--` reaching a hyphen-leading id, flags interleaved with conditions, no store created |
| `tests/compatibility.rs` | untouched — it is the guard that proves the machine surface did not move |
| `tests/fixture.rs` | the shipped-example harness: `IDS` substitution and a token-list runner beside `Declared::query(&Value)`, so the guide's DSL recipes stay executed (research F2) |

One obligation has no path in this table: **DEC-016** pins the JSON spelling of the
two-call recipe that `learn` prints and `tests/fixture.rs` executes, so teaching the
DSL form in the guide obliges an amendment to that decision. SL-008's precedent is to
record such obligations in `notes.md` and pay them at reconcile; the design should
name this one rather than discover it at closure.

## Risks, assumptions and open questions

- **R1 — clap may not be able to express this grammar.** Arbitrary positional
  tokens interleaved with flags, tokens containing `=`, `!` and `~`, and a
  trailing `--` that must reach an id starting with `-`. If `allow_hyphen_values` /
  `trailing_var_arg` cannot carry it, `q` would have to read `std::env::args`
  itself — which contradicts DEC-008's "clap owns the grammar" and must be ruled
  on, not worked around. **This is the largest unproven implementation risk and
  the first thing design must spike.**
- **R2 — two ways to enforce "no new semantics", and they differ in refusal
  voice.** Emitting the spec object and routing it through `spec::parse` makes the
  property structural, but every refusal then arrives in JSON vocabulary ("`from`
  must be a string or an array of strings") when a human typed a token. Name, per
  refusal, which layer voices it — or the migration message will be wrong exactly
  where a new user meets it.
- **R3 — value typing is a semantic decision smuggled into a front-end.** IDE-006
  recommends parsing a value as a JSON scalar when it is one, so `done=true` and
  `priority=3` work; the cost is that `smut_id=1` compares the number 1. Whatever
  is chosen is documented in `learn` and pinned by a test, because "bare is always
  a string" turns every numeric query into a silent empty result.
- **R4 — `~` inherits `$contains`'s two meanings** (substring on a string, exact
  member equality on an array; measured on a probe store at 0.2.0). One word with
  two semantics and no signal in the answer is a silent-wrongness class the DSL
  must not deepen by implying "contains" means one thing. `learn` says it, and a
  test pins both behaviours.
- **R5 — answered by the research round (F1), not by taste.** DEC-027 already
  refuses a provably-unsatisfiable field condition list, and DEC-030 has fixed its
  wording. So the DSL refuses exactly the one shape JSON cannot spell (two `=` on
  one field, itself unsatisfiable) and compiles every other repeat for the existing
  mechanism to judge. IDE-006's rule 3 said something stronger, and its suggested
  cure ("write `status=open,resolved`") is only right when both conditions are `=`
  — which is why the stronger rule retired rather than being reworded.
- **R10 — a DSL operator table would be the third copy of the five operators.**
  `parse_condition` and `parse_predicate` already share the list by hand (IMP-016).
  Derive the DSL's table from one `Operator` enum or record the third copy
  deliberately; do not let it arrive by omission.
- **R6 — silence survives the slice.** Empty results on a garbage field name stay
  exit 0. IDE-006 calls that "loud enough"; RFC-001 calls it Tier 1. Record the
  disagreement and do not let the guide, the design or a phase sheet claim the
  DSL cures it.
- **R7 — `q` and `query` could drift into two engines.** One pipeline, and the
  byte-identity test is what keeps it one. Any temptation to special-case the
  envelope for `q` is the bug.
- **R8 — `--ids` collides with `--select`.** An open question, not an
  implementation detail: refuse the pair, or let one win? A silent last-wins rule
  is the class of failure this slice exists to remove.
- **R9 — `--edges` beside a named direction.** The spec's own rule is that `edges`
  never overrides a `follow`/`backlinks` the request named; `q` inherits it rather
  than inventing a conflict refusal.

### Open questions for the design run

- **OQ1 — the verb's name.** `q` (short, and IDE-006's recommendation) versus
  `find` (discoverable, if a one-letter verb is too terse). A one-way door once
  documented.
- **OQ2 — bare roots with an escape, or a condition sigil.** Bare words plus
  `--from=ID` against `@field=value`, which makes bare words *always* ids but
  taxes the hot path. IDE-006's explicit rejection stands either way: `a=b` is
  never resolved as an id because it happens to match one.
- **OQ3 — value typing.** JSON-scalar-if-parsable (IDE-006's recommendation)
  versus always-string (R3). Research adds a constraint on the answer: under scalar
  typing `smut_id=1` compares the *number* 1, and `smut_id` is the one field ADR-001
  makes structural, so whichever way it goes `--ids` and `--from=1` must be named in
  the same breath as the wart.
- **OQ4 — `--brief`:** cut from this slice (proposed) or carried?
- **OQ5 — `--ids` with `--select`:** refuse, or precedence (R8)?
- **OQ6 — does `q` appear in the help text's command list and the guide's
  numbered recipes, and does `query` stay in the guide at all?** Proposed: `q` for
  the hot path, `query` kept and marked complete — DEC-010 makes the guide the
  delivery channel, so a verb the guide omits is a verb nobody finds.

## Verification and closure intent

*The pre-design round is `.doctrine/slice/010/research/research.md`; its cross-thread
findings F1–F7 are what revised objective 4, R5 and the affected surface above, and
design assertions should cite it rather than re-deriving them.*

- **Equivalence is the spine.** Every worked example in IDE-006's table is
  asserted as `dsl(tokens) == spec::parse(json)`, and the end-to-end pair
  (`q <line>` vs `query <json>`) is asserted **byte-identical** on the same store.
  A DSL feature with no JSON spelling is a design bug, not a missing test.
- **Zero quotes.** The guide's hot-path recipes are DSL lines, and the byte count
  claim in IDE-006's table is re-measured rather than trusted.
- **Refusals at exit 2, each naming its fix and its sibling**, including the four
  hazards that motivate the grammar: a JSON-looking positional, `--limit all`,
  `status!=a,b`, and `status~a=b`. Every refusal test names the change that
  reddens it (`mem.fact.smut.vacuous-negative-assertions`), and no refusal invents a
  field or a flag the caller did not type (DEC-030).
- **The contradiction is inherited, not reimplemented:** `smut q status=open
  status=resolved` is refused as two `=` on one field (unsatisfiable, and
  inexpressible in JSON), while `smut q status=open status!=resolved` compiles and
  is refused by `predicate::parse` in DEC-030's pinned sentence — the same line a
  JSON caller gets.
- **Strictness asymmetry, proved both ways:** `smut q --depth 2` is refused while
  `smut query '{"depth":2}'` still succeeds — so `q` is stricter *and* the machine
  surface did not narrow.
- **Shell-safety by construction:** the grammar's token classifier refuses every
  banned character if one arrives; the guide carries one demonstration that
  `foo~bar status!=baz any` reaches the binary intact.
- **DEC-004 holds:** `q` validates before a store exists and creates nothing,
  asserted by pairing it with a store-creating command in the same directory —
  never by a bare absence check no input could falsify.
- **DEC-011 holds:** `tests/compatibility.rs` is untouched and green; the machine
  surface's bytes and refusals are unchanged, and the diff proves it.
- **The guide does not lie** (DEC-010): the recipes work as printed, `query` is
  described as complete rather than deprecated, and both warts are stated.
- **Closure is the gate:** IDE-006 can be resolved-done once a human's five
  recurring queries are typed without a quote, and `smut query` is still the
  complete surface for everything the DSL cannot say.

## Summary

Add `smut q`, a human query surface whose grammar is the argv token list rather
than a string: bare words are roots, `field=value` / `field!=value` / `field~value`
are the operators, a comma list is `$in`, `any` is the wildcard, and each of the
remaining eight specification keys is a flag of the same name. The compiler is a
pure function producing the existing `Spec`, so the envelope, exit codes, store
rules and traversal semantics are inherited rather than reimplemented, and
`smut query` stays the complete machine surface with its 0.2 compatibility tests
untouched. Shell-active characters are banned from the grammar by construction —
which is what makes `any` the wildcard and `--has`/`--missing` the existence test
— and every refusal names its fix, the JSON positional's refusal naming its
sibling command and so telling the whole migration story in one line.

## Follow-Ups

- **IMP-009** — the envelope's six flat members: `q` inherits them verbatim, and
  the tax the DSL takes off typing does not pay it.
- **IMP-011 / IMP-012** — the silent-wrongness tier: field-name silence and
  `$contains`'s two meanings survive this slice, and `learn` must say so.
- **IMP-002** — `--brief`, one line per hit, and `smut render` accepting the same
  selection tokens: the output half of the pain, on the display surface.
- **IDE-003** — the census: value completion (`status=<TAB>` → the distinct
  values in the store) is the highest-value follow-on the grammar is designed not
  to preclude.
- **IDE-004** — edge attributes, which would give `--follow` something to say.
- **`smut query`'s docs** — once `q` owns the recipes, the guide's
  specification section becomes the reference for the tenth of cases, not the
  tutorial.
