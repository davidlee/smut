# Implementation Plan SL-005: Refusals that name the right thing — a contradiction refused and a misplaced operator explained

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

The locked design (run `dr-01a19a1e`, revision 43) settles two message surfaces in
one parse region: a field whose conditions cannot all hold is refused by name
(DEC-027), and a known operator written at the predicate's top level stops being
reported as unknown (DEC-030). Both land in `src/predicate.rs` and `src/error.rs`;
both are proved through the binary in `tests/predicates.rs`.

The plan is two phases, cut by *defect*, not by file. Both phases touch the same
three files, so a file-based split would buy nothing and force a half-built message
surface to sit on `master` between phases. Cut by defect, each phase is a complete,
independently green behaviour: one new refusal class with its rule, or one existing
refusal class told apart into two true sentences.

Phase order is also evidence order. The contradiction rule is the larger and
subtler piece — a walk, seven sentences, seven demand shapes and three pins — and
it is the one whose planted non-refusals would be silently "fixed" by a later
reader. The `$`-message split is small, but it is the piece a careless edit breaks
invisibly, because both new and old sentences are pinned whole.

## Sequencing & Rationale

**PHASE-01 first because it carries the design's load.** §2's demanded-position
walk is the only place where good intentions go wrong in both directions: refuse
everywhere and correct queries are rejected; refuse only at the top level and the
silent-empty defect returns under double negation. RV-026 routed five findings here
(F-1, F-3, F-4, F-13, F-14), each repaired in prose and each guarded only by a
test, so the phase is where those guards become real.

**PHASE-02 second because it is the smaller surface and it depends on nothing in
PHASE-01 except the boundary table's length.** Its whole risk is wording: two
variants, one shared arm split in two, two placeholders, two whole-line pins. Doing
it after the rule means the `$` split is reviewed as a diff against a green tree and
not mixed with a new refusal class.

**Both phases are gated on the same ordering constraint.** SL-004's PHASE-02
respells two identity messages in `src/error.rs`, and the drift note in `notes.md`
makes that live: SL-004's PHASE-01 is `completed`, so its PHASE-02 is next. Either
SL-005 lands first (the intended order) or PHASE-02 of SL-004 lands first and both
of this slice's phases rebase. Neither phase may race a concurrent writer on that
file. This is a scheduling guard, not a design tradeoff.

**The verification split follows the failure it would catch.** The rule is unit
level (a table over seven branches, a table over seven positions); the sentence is
message level (whole rendered lines, never substrings); the process promises — exit
2, empty stdout, no store created — are binary level, because a refusal that also
printed an envelope would stay green at unit level. Every VT row carries the file
and the substrings the gate reads, so `doctrine slice verify-vt` has signal rather
than an inert passing row.

## Notes

**Verified premises (re-grepped against the tree at plan time).** `parse` has one
external ingress (`src/spec.rs:132`); the predicate-level `$` arm is
`src/predicate.rs:72` and the condition-level one `:173`; `parse_field` and
`Condition::holds` are where the design says they are; the boundary table is a
fixed `[(Error, u8); 24]`; the malformed-predicate table in `tests/predicates.rs`
already asserts exit 2, empty stdout and no store created, so it is the right home
for both new rows. No existing test or fixture expects an empty-set answer for a
shape this slice now refuses, so nothing outside the selectors has to move.

**`tests/fixture.rs:457` is discharged, not deferred.** The handover flagged that
test's computed `{"id":{"$in": reached}}`. The derivation cannot be empty:
`reached_from_session()` is the same spec whose non-emptiness the sibling test
asserts. It stays a recorded premise of PHASE-01 rather than a file to change, which
keeps the diff inside the selector set.

**One design gap found at plan time.** The split falsifies the same sentence in two
places: `tests/predicates.rs:328-330` (which §6 names) and `src/predicate.rs:397-399`
(which it does not). Both say any `$` key other than the clauses is an unknown
operator at the predicate level, which stops being true once a known operator there
is refused as misplaced. PHASE-02/EX-5 repairs both. This is not a design revision —
it is §4's own rule (each sentence must be true) applied where the claim recurs —
but it is recorded here so the reader does not read the second repair as scope
creep.

**Two governance observations, neither blocking this plan.**

- RV-026 reads `STALE` in the run envelope with one change since the baseline. The
  change is the stage move to `locked`; every section still reports
  `review=current`, and the drift is a lifecycle act, not content. No re-review is
  owed.
- DEC-030 is still `proposed` on its own record while the design run adopted it via
  `inq-3`. Its wording is load-bearing for PHASE-02's pinned lines, so the status
  should be moved to `accepted` by the operator's act before implementation treats
  it as settled authority. Raised with the operator at plan review.

**Research advisory.** `doctrine slice research 5` reports the baseline has drifted
because `slice-005.md` changed and `design.md` was added. That is the expected shape
of a design landing on top of research, not a stale evidence base: the three
selectors this plan assumes come from research's Thread 2 hotspot map, and each
hotspot was re-checked against the tree above. No restamp is needed to plan; it can
be done at audit if the research artifact is to be reused.

**What is deliberately absent.** No third phase for evidence assembly — three files
and ~120 lines do not need one, and `/audit` owns integration. No guide text
(DEC-027's scoping sentence belongs to IMP-012's delivery), no envelope change, no
`$or` analysis, no `$and`-spanning analysis, and no repair of CHR-002's own
misattribution in `src/error.rs`. Each absence is a decision recorded in the design
or the backlog, not an omission.
