# Refusals that name the right thing — a contradiction refused and a misplaced operator explained

## Context

DEC-027 settled two things and closed a third. A field condition that cannot hold is
now refused by name; a diagnostic channel is **not** added (no envelope member, no
success-path stderr); and a `where` naming a field no entity carries is taught rather
than refused. Two backlog items fall out of that decision, and they are one slice
because they are one message surface in one parse region:

- **IMP-010** — the contradiction refusal: six decidable forms, all false for every
  store, refused at parse time before any store access.
- **ISS-004** — a `$` key at the predicate's top level is refused as "unknown
  operator", which sent a real caller to check the operator list. The field-mixing
  refusal it was misread as (`MixedPredicateObject`) already exists and is correct;
  the defect is that the top-level message names the position not at all.

Evidence, in order of weight: DEC-027 (accepted, with the subset and the rationale);
the 0.2.0 rc trial's frictions 1 and 2, including the operator's own misreading;
the three-store usability exercise, which ranked *silent wrong-or-empty answers*
first among eight findings and produced two more instances of the class (RFC-001).

**Why this slice first.** It is the only ranked work that touches nothing inside
SL-004's declared surface. SL-004 selects `src/learn.rs`, `src/conventions.rs`,
`src/cli.rs`, `src/store/**`, `src/entity.rs`, `src/declare.rs` and `tests/**` for the
`id` → `smut_id` sweep; the delivery work (IMP-008/011/012) and the conventions loader
(IMP-013) both live inside that set, so they queue behind it. `src/predicate.rs` and
`src/error.rs` are outside it. Cutting this first takes decided, cheap, disjoint work
while the identity break is in flight.

## Scope & Objectives

1. **A new refusal variant** for a provably unsatisfiable condition list, naming the
   field and the contradiction, exit 2, no success envelope. Its rationale is
   DEC-027's own — *a condition that cannot hold is a typo, not a question* — and not
   `MixedPredicateObject`'s "the design gives it no behaviour", which does not apply
   to a conjunction that is merely false.
2. **The six forms**, and only these: `$exists:false` with any other condition on the
   field; `$exists:true` with `$exists:false`; `$eq v` with `$ne v`; `$eq v` with
   `$in` not holding `v`; `$in []`; `$in A` with `$in B` whose intersection is empty.
   The check reads one field's condition list and nothing else, so `src/predicate.rs`
   keeps its no-store, no-clock property.
3. **The top-level `$` message gains position and shape** — it still names the
   operator (that is true and pinned), and adds where an operator belongs, with the
   shape to write instead. The field-level `MixedPredicateObject` message is not
   touched.
4. **Tests, red first**: one per form; positive controls that `$eq v` with `$in [v]`
   and `$contains` combinations are *accepted*; and a pin asserting that a
   contradiction spanning `$and` clauses is **not** refused, so a later reader cannot
   widen the check by accident.
5. **Compatibility is free and must stay documented as such**: `where` is a 0.2 key,
   so no 0.1 query can express a predicate and DEC-019's unchanged-0.1-response
   property cannot move.

## Non-Goals

- **No diagnostic channel.** No envelope member, no success-path stderr. That is
  DEC-027's answer, not an omission.
- **No `where`-names-an-absent-field diagnostic.** Taught instead (IMP-012); refusing
  it would punish filtering as a way to probe a store.
- **No `kind`-without-`type` advisory.** Delivery work (IMP-011); if ever revived it
  is a `changes[]` entry, never stderr.
- **No `$and`-spanning analysis.** Out of scope by decision; the pin above says so.
- **No guide text in this slice.** `learn`'s predicate paragraph is untouched, which
  also means no pin churn inside SL-004's surface. The scoping sentence DEC-027
  obliges ("smut refuses some contradictions, within one field's conditions") ships
  with the delivery slice — recorded as a bounded, known gap, not an oversight.

## Affected surface

- `src/predicate.rs` — the check, in `parse_field`, plus the top-level `$` arm.
- `src/error.rs` — one variant, its `Display`, and its message tests.
- `tests/predicates.rs` and/or `tests/cli_contract.rs` — the red-first cases, the
  positive controls and the non-refusal pin. These live under SL-004's `tests/**`
  selector but in files it does not rewrite; if that assumption breaks during design,
  this slice waits rather than racing.

## Risks & assumptions

- **A partial detector can read as assurance.** Bounded by DEC-027's scoping sentence
  landing in the next slice; this slice adds no claim to the guide, so it neither
  over-promises nor widens the window much.
- **Runtime-composed predicates change behaviour**: a caller templating `$eq`/`$ne`
  now gets exit 2 where it got `[]`. Accepted with the decision; worth a sentence in
  the refusal's own test, not a compatibility path.
- **A form may not survive design.** If a form turns out unsound (structural equality
  on some value shape), drop it rather than widen the check — the subset is a floor
  for the decision, not a quota.
- **Message text is a surface.** Choose the wording in design, pin it, and keep
  `MixedPredicateObject`'s message byte-identical.

## Verification / closure intent

- Red-first per form, green after; the positive controls stay green throughout.
- The `$and` case pinned as *accepted* (no refusal).
- Exit 2, message on stderr, no success envelope on stdout.
- `tests/predicates.rs` and the refusal-message tests in `src/error.rs` pass.
- No file inside SL-004's `src/` selectors is modified by this slice.
