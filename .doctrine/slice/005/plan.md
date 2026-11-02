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

**Size, and the one seam if a budget forces one.** The phases are not evenly
weighted. PHASE-01 is roughly 250-300 lines and 13-15 tests — two seven-row tables,
seven pinned sentences, a four-step rule and three pins — and it is the slice's
reasoning load. PHASE-02 is roughly 60 lines and 4-6 tests. The ~120-line figure this
plan first carried was the handover's, taken before the seven-sentence table and the
demand table existed; the conclusion it supported still holds, but the arithmetic did
not. No third phase for evidence assembly belongs here, and `/audit` owns
integration. If a budget ever forces a cut, the only separable seam is PHASE-01's
binary rows (`EX-6`): the rule and the demand walk are never split, since a rule
without the walk re-opens RV-026 F-1.

**A chain, never two worktrees.** Both phases write the same `parse` function and the
same `malformed` array literal in `tests/predicates.rs`. PHASE-01 appends rows to that
array; PHASE-02 adds a separate test rather than restructuring it. Dispatch them in
order onto one branch: parallel worktrees would collide on both literals, and the
second would have to redo the first.

**Model requirements.** PHASE-01 needs sustained frontier reasoning. Its four failure
modes all look green at first glance: the walk installed at every recursive return,
sentences chosen by `BTreeMap` key order, step 2 read as "refuse any `$exists:false`",
and a pin "fixed" instead of honoured. Dispatch it to a model that can hold §2's
propagation rule and the seven shapes in one context and that will honour a pin
contradicting its own instinct. PHASE-02 is mechanical adherence with exact-string
discipline — two `write!` arms, two placeholder slots, two whole-line pins, two
comment repairs — and is safe for a mid-tier model provided DEC-030 is in context and
the pinned lines are copied, never paraphrased.

The same one-line floor belongs in each runtime phase sheet at `/phase-plan` time,
because the sheet, not this plan, is what a dispatched worker reads.

## Notes

**Verified premises (re-grepped against the tree at plan time).** `parse` has one
external ingress (`src/spec.rs:132`); the predicate-level `$` arm is
`src/predicate.rs:72` and the condition-level one `:173`; `parse_field` and
`Condition::holds` are where the design says they are; the boundary table is a
fixed `[(Error, u8); 24]`; the malformed-predicate table in `tests/predicates.rs`
already asserts exit 2, empty stdout and no store created, so it is the right home
for the contradiction rows — PHASE-02's whole-line pin needs a separate test beside
it, because that table's rows are 2-tuples asserted by substring. No existing test
or fixture expects an empty-set answer for a shape this slice now refuses, so
nothing outside the selectors has to move.

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

**What is deliberately absent.** No third phase for evidence assembly: PHASE-01 alone
is 250-300 lines and 13-15 tests, and `/audit` owns integration. No guide text
(DEC-027's scoping sentence belongs to IMP-012's delivery), no envelope change, no
`$or` analysis, no `$and`-spanning analysis, and no repair of CHR-002's own
misattribution in `src/error.rs`. Each absence is a decision recorded in the design
or the backlog, not an omission.

## Hardening

Two independent read-only reviews ran over this plan before dispatch: one over
obligation coverage — every claim in design §1-§7 mapped to a criterion — and one over
implementation-surface viability — call sites, assertion homes and gate sharpness
checked against the tree. The changes below are the adjudicated result.

**What the reviews changed.**

- **The call-site split is named.** "Called once at the end of `parse`" is writable as a
  walk at every recursive return, which refuses a contradiction inside `$or` or under a
  single `$not` — two of the phase's own pins. PHASE-01/EX-2 now names the private
  builder, the one-call public `parse`, and `a_single_disjunct_or_is_not_refused` as the
  pin that catches the mistake.
- **Fixed order is pinned by more than one adjacency.** PHASE-01/EX-4 enumerates the
  seven named tests, the verbatim sentences, the positive controls, the rows that prove
  §2's order across steps 3 and 4, the array-order pair and the `1`/`1.0` row, plus
  `{"$exists":false}` alone, which step 2 must not swallow.
- **Inert VT keywords are replaced.** Rows keyed on substrings that already sat in the
  named file — `INVALID_INPUT` in `src/error.rs`, `has_store`, `$contains` and
  `UnknownPredicateOperator` in the other two — now key on strings that can only exist
  once the phase lands, and PHASE-01 gains VT-4 keyed on the seven test names.
- **The whole-line assertion has a home.** The malformed-predicate table is 2-tuples
  asserted with `contains`, so it cannot carry a whole-line assertion; PHASE-02/EX-4
  puts the new pin in a separate test beside it and leaves the three rows untouched.
- **`$foo` gets a whole-line pin.** The three existing `$foo` assertions are variant
  equality or substring, so none of them could see a reworded unknown-name sentence.
- **DEC-030's status is an entrance condition.** PHASE-02/EN-1 requires the decision to
  read `accepted`, or the operator's recorded assent to be in the evidence, and names
  its `choice` facet rather than its stale body example as the byte source.

**Disposed, not carried.** Each was a defect in the locked design or in a decision it
transcribes. Two were decision records rather than design prose, so they were repaired
in place; the rest are the locked design's own sentences, parked on `CHR-003` for the
audit/reconcile seam — editing the materialised `design.md` would desync the run's
fingerprints for prose that changes no behaviour, and the plan implements the correct
behaviour in every case.

- **DEC-030 settled and its body corrected.** It read `proposed` while the design run had
  adopted it; it now reads `accepted`, and its body's misplaced-operator example prints
  `<operand>` as its own facet always said.
- **DEC-027's position rule corrected in both tiers.** "Any even number of `$not`s
  provided no `$or` interrupts the path" was too narrow: an `$or` that must *not* hold
  hands its children `false`, so a `$not` beneath it restores demand and the field is
  judged — and that refusal is sound. The choice now states demand propagation and names
  both `$or` directions; the body's "`$or` contents are not analysed at all" is narrowed
  to a must-hold `$or`, and the provenance records both directions of the shorthand's
  error.
- **Six `design.md` sentences → `CHR-003`.** §1's diagram label ("judged only when
  reached through `And`" is false for a bare root field, the same error DEC-027
  corrected); §2/§6's "called once at the end of `parse`" against §1's own recursion
  premise; §5's binary row silent on the `smut: ` prefix; §6's "four pins" against §5's
  three; §7's "`$or` contents are not analysed" over-claim; §3's second sentence naming
  one operator where DEC-030 asks for two. Terminology ("required-position" vs
  "demanded") is recorded there too.
