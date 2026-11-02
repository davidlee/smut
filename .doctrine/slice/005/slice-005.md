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

**Why this slice first.** It is the only ranked work that does not touch the *bulk* of
SL-004's surface. SL-004 selects `src/learn.rs`, `src/conventions.rs`, `src/cli.rs`,
`src/store/**`, `src/entity.rs`, `src/declare.rs` and `tests/**` for the `id` →
`smut_id` sweep; the delivery work (IMP-008/011/012) and the conventions loader
(IMP-013) both live inside that set, so they queue behind it. `src/predicate.rs` is
outside it entirely.

**One shared file, found after the slice was cut.** SL-004's *selectors* understate its
surface: PHASE-02/EX-9 respells two identity messages in `src/error.rs`
(`Reject::InvalidId`, `StoreError::IdDisagreement`), and that file is neither a
selector nor a row in SL-004's design-target table. This slice adds a `Reject`
variant, its `Display` arm and tests to exactly that file. Neither change is large and
neither is a conflict at scope time, so the rule is ordering rather than avoidance:
land the refusal phase before SL-004's PHASE-02, or take a small rebase. Design work
here can proceed regardless.

## Scope & Objectives

1. **A new refusal variant** for a provably unsatisfiable condition list, naming the
   field and the contradiction, exit 2, no success envelope. Its rationale is
   DEC-027's own — *a condition that cannot hold is a typo, not a question* — and not
   `MixedPredicateObject`'s "the design gives it no behaviour", which does not apply
   to a conjunction that is merely false.
2. **The decidable set, stated as a criterion** — not the six-form list this slice was
   cut with, because design found that list wrong in three places (DEC-027, amended
   twice). Two of its forms need the same key twice in one object and so cannot be
   written at all (DEC-006 refuses duplicate keys before a `Value` exists); one
   decidable neighbour was missing (`$ne v` against an `$in` list whose every member
   equals `v`); and `$contains` was called undecidable except beside `$eq`, when `$in`
   also bounds the value. The criterion: the check decides when the conditions bound
   the field to one value (`$eq v`, so the rest is evaluated on `v`) or to a finite set
   (`$in S`, so some member must satisfy the rest), plus the two presence facts
   (`$exists:false` beside anything, and `$in` of nothing). Outside those bounds —
   `$contains` against a value the request does not bound — nothing is provable. The
   check reads one field's condition list and nothing else, so `src/predicate.rs` keeps
   its no-store, no-clock property. It also fires only where the predicate **demands**
   the field: the root and `And` children, however many `$not`s wrap them, and never
   inside an `$or` or under an odd number of `$not`s, where the query still has an
   answer.
3. **The predicate-level `$` refusal becomes two messages** — a known operator out of
   place gets its own variant naming the position and a shape with placeholder field
   *and* operand; an undefined name, at either level, keeps `unknown predicate
   operator: {operator}` (DEC-030). No field is ever invented, and the field-level
   `MixedPredicateObject` message is not touched.
4. **Tests, red first**: one per branch of the rule; positive controls that `$eq v`
   with `$in [v]` and `$contains` combinations are *accepted*; and a pin asserting
   that a contradiction spanning `$and` clauses is **not** refused, so a later reader
   cannot widen the check by accident. Assertions are split by level: the rule at unit
   level, the sentence in the message test, and exit code, empty stdout and
   "no store opened" at the binary level.
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
- **No `$or` analysis.** `$or` contents are never examined, so a one-element `$or`
  around a contradiction is accepted and empties the query. Closing that one shape
  alone would imply the rest of `$or` is analysed; the pin makes the choice explicit.
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
- **`src/error.rs` is shared with SL-004.** Its PHASE-02/EX-9 respells two identity
  messages in that file; this slice adds a variant, a `Display` arm and tests to the
  same module. Ordering, not avoidance: refusal phase before SL-004's PHASE-02, or a
  rebase. The overlap was invisible from the selectors, which is the lesson recorded in
  RFC-001.
- **The six-form list was wrong in three ways, which is why it became a criterion.**
  Design found two unexpressible forms, a missing decidable neighbour, and then — on
  review — an over-claimed undecidability (`$contains` beside `$in`). The operator's
  answer was to state the rule rather than patch the table, and review corrected the
  rule (DEC-027, amended twice). A later reader must not "restore" the six: two of
  them describe shapes a caller cannot write.
- **Message text is a surface.** Choose the wording in design, pin it, and keep
  `MixedPredicateObject`'s message byte-identical.

## Verification / closure intent

- Red-first per branch of the rule, green after; the positive controls stay green
  throughout.
- The `$and` case pinned as *accepted* (no refusal).
- Exit 2, message on stderr, no success envelope on stdout.
- `tests/predicates.rs` and the refusal-message tests in `src/error.rs` pass.
- No file inside SL-004's `src/` selectors is modified by this slice.
