# Implementation Plan SL-002: smut 0.2 — graph release: predicates and traversal

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Seven phases, each ending on a green tree and a coherent request surface. The
design fixes the shape (`design.md`, locked at run revision 62); the scope fixes
the boundary and its work order; this plan decides only *where the seams between
the work are*.

The seams are chosen so that no phase leaves a promise the next phase has to
keep. Three consequences dominate the ordering:

- **A traversal key and its provenance are inseparable.** DEC-012 makes
  `traversal` present exactly when `follow` or `backlinks` is supplied. A phase
  that accepted the key while emitting nothing would answer a traversal request
  with a 0.1 envelope — silently claiming the walk knew nothing. So the walker,
  the gates that bound it and the member that reports it land together, and
  `budget` — which only *shapes* what the walk refuses to do — is the one key
  held back to its own phase, with the walk called unbounded until then.
- **Predicates are independent, and are sequenced last of the mechanisms.** The
  scope's work order (from `finding-smut-19`) puts the walker first because the
  label filter was never the cost; the predicate interpreter is a closed
  function over one entity and gains nothing from landing earlier. Doing it third
  also means the roots-only rule is positioned against a walker that already
  works, rather than against a promise.
- **A refusal may not outlive the key it refuses.** `FUTURE_KEYS` names the keys
  a later release will accept; it retires in the commit that accepts the last of
  them, not before, or `where` becomes an unknown typo naming no release while
  the shipped guide still promises it.

## Sequencing & Rationale

**PHASE-01 — the walker, its labels and the provenance it carries.** The risk
lives here: the reference definition must be consumed (not re-derived), the six
determinism rules must be behaviour, and the three provenance views have
different scopes (`depths`/`edges` result-scoped, `frontier`/`dangling`
walk-scoped) — an asymmetry that is easy to blur and expensive to unpick later.
The induced-edge rule (DEC-014) lands here too, because it is derived from the
*returned* entities after the limit: shipping the walk without it would make
`edges` a second, contradictory account of the same graph. Three smaller
decisions are taken here rather than discovered later: the walk receives an
unbounded budget until the key exists (so this phase cannot report `truncated:
false` over a walk it silently cut), `select::run` takes one input value instead
of crossing clippy's argument threshold one phase at a time, and the Markdown
notice gains its dangling lines now — `dangling` is already real in JSON, and a
renderer that dropped the release's new diagnostic for a phase would be the one
channel left lying. The README's executed example is re-recorded in this commit
for the same reason: accepting `follow` turns its final recorded step from a
refusal into a working query, and the transcript the test runs is part of the
tree.

**PHASE-02 — budget, truncation and the reason order.** Budget semantics are
small and unforgiving: counted in expansions, checked *before* expanding, and
`traversal_budget` only for a permitted expansion the walk declined. Four
distinct "complete" states (empty queue, exact budget, depth boundary only, no
traversal at all) each need their own test, which is why this is a phase rather
than a paragraph in PHASE-01. `budget` becomes real; `where` is now the only
future key, and it stays refused *by name* here — the retirement belongs to the
phase that accepts it.

**PHASE-03 — predicates over roots.** The brief's closed interpreter: five
operators, three clauses, one disambiguation rule with a refusal, and
missing-versus-null. It runs before traversal and filters roots only, so the
composition idiom — not a traversal filter — remains the answer to "filter what
the walk found". Nothing here touches storage, the clock or the store, and that
is the phase's own invariant. Because this is the commit in which the last
future key becomes real, `FUTURE_KEYS`, its rejection and its display arm go
with it, and the three integration pins that assert the where-refusal are re-cut
into the assertions that survive.

**PHASE-04 — the acceptance fixture and its privacy gate.** Data, moved early
enough that the guide (PHASE-06) can assert its example against real entities
instead of describing one. It carries the project's only human gate: the
operator's read-through before the commit (DEC-018). Deliberately before the
user-facing surfaces, because a fixture that turns out to be unpublishable
changes what the guide may claim, and it is cheaper to learn that here than at
audit. A refusal of publication is a decision for the operator, recorded before
the dependent phases open.

**PHASE-05 — the conventions command.** Independent of everything above it and
of the fixture: pure text, no store, one set with a fixed-id seed. The seed
carries the crate's own version rather than a literal, so this phase asserts the
version's *source* and PHASE-07 owns the value.

**PHASE-06 — the guide.** Last of the behaviour-adjacent work, because the guide
is where drift is caught: its recipe must state the direction containment
actually has, its guard must state that the obligation is the client's, and its
example must be the example the fixture test runs. A behaviour without its
sentence in the guide is an unfinished fix (`finding-smut-17`), so this phase is
a deliverable, not documentation duty.

**PHASE-07 — README, gates and the closure sweep.** The rest of the release
surface and the proof: the full worked example (which extends the step PHASE-01
recorded), the six acceptance scenarios each in the suite that owns it, the four
gates (fmt, clippy, test, non-vacuous fault-injection), the selector conformance
diff, and the version. Nothing new is designed here; anything that needs
designing is a phase defect found late, and is reported as one.

## Verification discipline

Two rules decide whether a criterion is worth anything, and both were applied to
every row in `plan.toml`:

- **A mandate must be able to fail.** A VT whose keywords already exist before
  its phase certifies nothing, and a VT whose subject is prose is not a test at
  all. Keywords name something only the intended assertion produces — a test
  function, an asserted literal, `#[test]` where the mandate is "unit tests
  cover". Where the subject is the README, the mandate names the test that
  executes it.
- **A test is re-cut, never deleted** (DEC-019). Every phase that breaks an
  existing assertion names it in a criterion and says what it becomes, with the
  reason recorded beside it.

## Routed design-review findings

None. The design-review ledgers for this slice resolved every finding through
`review`, `dedupe` or `refresh` routes; no finding carries `demonstrate`,
`probe` or `control`, so this plan inherits no untranscribed instrument
obligation. (RV-012's F-4 is `refresh`, answered and awaiting the raiser's
verification; its repair — the seed-body clause and the `previous` reply — is in
the locked design and is asserted by PHASE-05's second VT mandate.)

## Where the plan is deliberately thin

- **Line-level file lists are the design's**, not the plan's: `design.md` §14.1
  names the touch-set and §14.2 the files that stay untouched. The plan adds no
  second inventory to drift; §14.2's unchanged set is proven by PHASE-07's diff
  because two of those paths are not slice selectors.
- **Phase sheets carry per-phase detail.** Anything a phase needs that is not a
  criterion (measured counts, the fixture's provenance outcome, the conformance
  diff) belongs in that phase's runtime sheet, which is disposable by design.
- **`specs`/`requirements` are empty** — v1 has no registry, and inventing ids
  here would create a second account of the same intent.

## Notes

- Every VT mandate names a file that exists once its phase lands: `src/walk.rs`,
  `src/predicate.rs` and `src/conventions.rs` are created by their phases;
  `tests/traversal.rs`, `tests/predicates.rs`, `tests/conventions.rs` and
  `tests/fixture.rs` likewise; `src/spec.rs`, `src/learn.rs`,
  `tests/cli_contract.rs`, `tests/compatibility.rs`, `tests/selection.rs`,
  `tests/render.rs` and `tests/readme_example.rs` already exist — for those, the
  keywords were chosen so the mandate fails on the entering tree.
- The two human/agent modes are used where a test cannot judge: the fixture's
  privacy verdict (`VH`), gate and conformance outcomes (`VA`).
- PHASE-05's fixture-independent tests stay in `tests/conventions.rs` so a
  change to the fixture cannot break the conventions gate.
- New suites copy the scratch-store harness the repository already has rather
  than inventing a second one: `tests/cli_contract.rs` holds the fullest copy
  (a per-file `Scratch`, a `run` that accepts stdin, and a one-JSON-line
  assertion), which is the shape every new suite follows. There is still no
  `tests/common`; consolidating is not this release's work.
- The fixture's array and the one helper that declares it live in
  `tests/fixture.rs`; PHASE-06 and PHASE-07 extend that file, and a suite outside
  it reads the committed array rather than committing a second copy. Integration
  test files are separate binaries, so this repository shares no helper module:
  each suite keeps its own `Scratch`.
