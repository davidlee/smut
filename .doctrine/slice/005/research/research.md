# SL-005 — research

Pre-design research round for **SL-005 — Refusals that name the right thing**
(IMP-010 + ISS-004, DEC-027).

## Header

- **Producers** — orchestrating agent, **degraded mode**: no project research
  agents are defined in `AGENTS.md` or `.doctrine/project-orientation.md`, so the
  two threads ran as one agent and their verbatim output is in
  `raw/thread-1-governance.md` and `raw/thread-2-code-map.md`.
- **Baseline** — `.doctrine/slice/005/research/baseline.toml`, stamped by
  `doctrine slice research 5` at round start.
- **Scope inputs read** — `SL-005` (scope), `DEC-027` (accepted), `IMP-010`,
  `ISS-004`, `RFC-001`.

## Verification legend

✓ — claim independently verified by reading or running the cited site during this
round (which, in degraded mode, is the same agent). Unmarked — cited from an
artefact read this round, not re-checked at its own site.

Design and plan may load-bear only ✓ rows, or rows they verify at point of use.

## Thread 1 — governance applicability

| authority | verdict | reason |
| --- | --- | --- |
| DEC-027 | **binding** ✓ | Fixes the mechanism (parse-time refusal, naming field + contradiction), the six forms, and the negatives (no envelope member, no success-path stderr, `$and` out of scope, `$contains` undecidable). |
| DEC-019 | **binding, must not move** ✓ | The envelope is the 0.1 five plus one conditional member; this slice adds none. `where` being 0.2-only is what makes the change compatibility-free. |
| DEC-006 | **binding, shapes the form list** ✓ | Duplicate keys in one JSON object are refused, so a condition object carries each operator at most once. This is what makes the form list a *pairwise* enumeration (see cross-thread finding 2). |
| ADR-001 | **checked, not applicable** | Identity is `smut_id`; no identity key is added or moved, and the new variant names a field, never an id. |
| STD-001 | **checked, not applicable** | Nothing stored changes; the check reads a parsed request and never a store. |
| STD-002 | **checked, not applicable** | Parse-time refusal precedes every store access (DEC-004), so no manifest — in-flight or committed — can be involved. |
| `mem.fact.smut.failure-classes` | **binding on the choice of class** ✓ | Exactly two classes: `Reject` (exit 2, user error) and `StoreError` (exit 1); internal invariants assert rather than report. A caller's own contradiction is user error → `Reject`. |

No `spec` entities exist in this project (`.doctrine/spec/` absent; `doctrine spec
validate` reports an empty, clean corpus). Product intent for this surface is the
`learn` guide text, which is SL-004's selector and this slice's explicit non-goal.

**Revision candidates** — none. DEC-027 needs no revision for this slice to
implement its own subset; dropping a form would be a scope correction, not a
decision edit.

## Thread 2 — code map

### Hotspots

- `src/predicate.rs:124` `parse_field` — the only site building a field's
  `Condition` list; `:152` is where the whole list exists as one value. ✓
- `src/predicate.rs:71` — ISS-004's arm: any non-clause `$` key at predicate level
  becomes `Reject::UnknownPredicateOperator`. ✓
- `src/error.rs:18` (`enum Reject`), `:69` (`MixedPredicateObject`), `:305-315`
  (the three predicate `Display` arms). ✓
- `tests/predicates.rs:549` — refusals asserted **through the binary**: exit 2,
  empty stdout, stderr containing the offending name, and no store created. ✓
- `src/error.rs:495` — `the_boundary_maps_each_failure_class_to_its_exit_code`,
  a fixed-size `[(Error, u8); 24]`; a new variant needs the length bumped and a row
  added, and nothing enforces it (the array is not exhaustive over variants). ✓

### Cited facts

- **One call site.** `predicate::parse` is reached only from `src/spec.rs:132`,
  inside the spec parse DEC-004 pins as complete before the store is opened. ✓
- **No-store, no-clock is a module invariant** stated in `src/predicate.rs:1-16`
  (VA-1). ✓ A `Vec<Condition>` check preserves it.
- **Structural equality is `serde_json`'s `==`**, pinned by
  `structural_equality_is_serde_json_equality` (`1 != 1.0`; key order irrelevant). ✓
- **Presence is per-operator**: `Eq`, `Ne`, `In`, `Contains` all require the field
  present; only `Exists(false)` holds of absence. ✓ This is why DEC-027's first
  form is sound as a *rule*.
- **`$ne` needs presence** — "absent is not 'not equal'" ✓.
- **Guide already covers the class**: `src/learn.rs:116` (a malformed predicate is
  exit 2 and writes nothing) and `src/learn.rs:122-127` (the disambiguation rule and
  the five field operators). Guide pin at `src/learn.rs:377`. ✓ No guide edit is
  required, which is what lets the slice stay out of SL-004's `src/learn.rs`.
- **stderr is one line**: `smut: {error}\n` (`src/main.rs:114-116`). There is no
  second-line channel. ✓
- **Pins the new message must not break**: the operator name must survive
  (`src/error.rs:764-767`; `tests/predicates.rs:549` asserts stderr contains
  `$foo`). ✓

### Naming precedents

Variants are noun phrases for the defect (`MixedPredicateObject`,
`InvalidPredicateValue`, `UnsortableValue`); messages name the offending noun first
and the fix second, lower-case, no trailing full stop; `smut:` is added by the
renderer. ✓

## Cross-thread findings

1. **DEC-027's six forms collapse to four reachable rules.** With DEC-006 refusing
   duplicate keys, `$exists:true` + `$exists:false` **cannot be expressed** in one
   condition object at all: the JSON parser refuses the repeated key before a
   `Value` exists. So form 2 is a subcase of form 1 (`$exists:false` with any other
   condition) and is unreachable in practice. Reachable forms: five. Design should
   say so rather than claim six reachable ones.
2. **The form list is pairwise because of DEC-006, not by choice.** Each operator
   appears at most once per condition object, so "every contradictory pair" is a
   finite, complete enumeration over {`$eq`, `$ne`, `$in`, `$exists`, `$contains`},
   and the only decidable pairs are the ones DEC-027 lists. `$contains` pairs are
   undecidable, which is exactly why it is excluded.
3. **A sound form is missing from the list, and it is the dual of one that is
   present.** `$ne v` with `$in [v]` (more generally `$ne v` with an `$in` whose
   every member equals `v`) cannot hold either — nothing is both equal to and not
   equal to `v` — and it is as decidable as DEC-027's `$eq v` with an `$in` not
   holding `v`. This is the false-assurance risk DEC-027 itself records: the
   refusal class is advertised as "a condition that cannot hold is refused", but
   the subset leaves a neighbour silent. **Not a reason to widen silently** — the
   six forms are fixed — but it is a bounded gap that should be recorded with
   IMP-012's scoping sentence rather than left to be rediscovered.
4. **`1` vs `1.0` matters and the answer is consistent.** Under `serde_json`
   equality `$eq 1` and `$in [1.0]` is a contradiction (both cannot hold of one
   value). The check must use the same `==` the evaluator uses, or refusal and
   evaluation will disagree — the failure mode `structural_equality` was written
   to prevent.
5. **`$exists:false` + a literal is a contradiction too.** A field condition with
   no `$` key parses to `Eq(value)`, which requires presence, so
   `{"f": {"$exists": false}}` beside `{"f": null}` — i.e. `Eq(null)` — is
   unsatisfiable. Form 1's "any other condition" must include the literal-`$eq`
   case, not only operator-shaped siblings. (In practice a condition object is
   either all-`$` or a literal, so the mixed shape is refused as
   `MixedPredicateObject` first — but the *rule* should be stated over conditions,
   not over keys.)
6. **Issuing the check after the list is built keeps the parse errors ordered.**
   `parse` reports the first defect in member order; validation that runs after all
   conditions of the field are built cannot pre-empt a sibling field's earlier
   parse refusal, but it also cannot be reordered by key order. Any contradiction
   message must therefore be deterministic across key order (compute over the whole
   list, not on first pair encountered).
7. **ISS-004 shares the file but not the variant.** The top-level `$` message is
   `UnknownPredicateOperator`, whose name must stay and whose name must be printed;
   the contradiction gets a **new** variant. Two messages, one parse region, no
   shared code path beyond `parse`.

## Design-input deltas

- **Count, honestly.** Objective 2 says "the six forms"; five are reachable. Design
  states the five reachable forms plus the reasoning that the sixth is
  unexpressible, so audit is not chasing a form that cannot occur.
- **Scope tension to resolve with the operator.** Objective 2 says "and only
  these"; the risk section says "the subset is a floor for the decision, not a
  quota". They disagree about whether adding cross-thread finding 3's sound form is
  in scope. Recommendation: keep the six (no widening without a decision) and
  record the gap beside IMP-012's scoping sentence — the slice's own Compatibility
  and Risk sections both assume the guarantee is scoped in the guide.
- **One variant or five?** Thread 2's precedent favours **one** variant carrying
  `field` + a contradiction description (as DEC-027's example message shows a
  single sentence naming the contradiction). Five variants would multiply the
  `Display` arms and the boundary table for no caller-visible gain.
- **The epigram is a cost, not a freebie.** stderr is one line (`src/main.rs:116`),
  so DEC-027's parenthetical — *a condition that can never hold is a typo, not a
  question* — would ride on **every** refusal instance. Recommend it stay in the
  decision and the guide, not in the message; the message names the field and the
  contradiction and stops.
- **ISS-004's message must invent no field.** "No field can be named" is ISS-004's
  own constraint, so the shape shown has to be generic — and it must not collide
  with `MixedPredicateObject`'s field-level diagnosis.
- **Tests are cheap and already patterned.** Unit refusal tests in
  `src/predicate.rs`, message tests in `src/error.rs`, and one binary-level table
  row in `tests/predicates.rs:549` for exit 2 + empty stdout + no store. Positive
  controls (`$eq` with `$in` holding it; `$contains` combinations) and the
  `$and`-spanning non-refusal pin belong in the same two files.

## Baseline refresh at plan time (2026-11-02)

The advisory reported drift on two files: `slice-005.md` changed and `design.md` was
added. Both are the expected shape of a design landing on top of research, not a stale
evidence base, so nothing in Threads 1-2 was rewritten.

Re-verified rather than assumed, before planning against this artefact:

- **Thread 2's hotspots still hold.** `parse_field` is still the only site building a
  field's `Condition` list, the predicate-level `$` arm is still `src/predicate.rs:72`
  (condition-level `:173`), the boundary table is still a hand-maintained
  `[(Error, u8); 24]` at `src/error.rs:496`, and the binary refusal table is still
  `tests/predicates.rs:549`.
- **Thread 2's cited facts still hold**, including the one call site
  (`src/spec.rs:132`), the no-store/no-clock module invariant, `serde_json` equality,
  per-operator presence, and the one-line renderer.
- **One fact the plan had to add to Thread 2:** `parse` re-enters itself through
  `parse_all` and `parse_not`, so an AST walk at the end of `parse` runs once per clause
  operand. That is why the plan puts the walk in a wrapper (PHASE-01/EX-2), and it is
  recorded as `mem.fact.smut.recursive-parse-post-check`.

The three selectors this slice uses (`src/predicate.rs`, `src/error.rs`,
`tests/predicates.rs`) come from Thread 2's hotspot map, not from recall.
