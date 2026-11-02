# Thread 1 — governance applicability (SL-005)

Producer: orchestrating agent, degraded mode (no project research agents defined in
AGENTS.md or `.doctrine/project-orientation.md`; the round ran as one agent).

## Binding constraints in view

- **DEC-027** (accepted) — refuse at parse time, before any store access, when one
  field's condition list is provably unsatisfiable, naming the field and the
  contradiction; six forms fixed; no envelope member, no success-path stderr,
  DEC-019 untouched; `$and`-spanning contradictions out of scope; `$contains`
  undecidable. The epigram the rationale is built on: *a condition that cannot hold
  is a typo, not a question* — explicitly not `MixedPredicateObject`'s "the design
  gives it no behaviour".
- **DEC-019** (accepted) — the 0.2 envelope is the 0.1 five keys plus one
  conditional member; additivity is the property, pinned in `tests/compatibility.rs`.
  SL-005 changes no envelope member, so this is a constraint not to breach, not a
  surface to extend.
- **DEC-006** (accepted) — an input object giving the same key twice is refused
  (`Reject::DuplicateKey`). Consequence for this slice: a condition object cannot
  carry the same operator twice, which bounds the pairwise form list (see
  cross-thread finding 2) and makes one of DEC-027's six forms unexpressible.
- **ADR-001** (accepted) — identity is `smut_id`; **checked, not applicable**: no
  reference to an identity key is added or moved by this slice, and the `Reject`
  variant names a *field*, never an id.
- **STD-001** (default) — `docs/store-format.md` is the store format's only record;
  **checked, not applicable**: nothing stored changes; the check reads a parsed
  request and never a store.
- **STD-002** (default) — recovery replays a committed manifest before the loader
  validates it; **checked, not applicable**: parse-time refusal precedes every
  store access (DEC-004's "the whole request is validated before the store is
  opened, locked or created"), so no manifest, in-flight or otherwise, can be
  involved.
- **`mem.fact.smut.failure-classes`** (memory, medium trust) — exactly two failure
  classes: `Reject` (exit 2, "the core cannot express anything but user error") and
  `StoreError` (exit 1); an internal invariant asserts instead of reporting. A
  contradiction in the caller's own predicate is user error, so it is a `Reject`
  variant — never `StoreError`, never a panic.

## Specs

No `spec` entities exist in this project (`.doctrine/spec/` absent; `doctrine spec
validate` reports a clean, empty corpus). Product intent for this surface lives in
the `learn` guide text (`src/learn.rs:100-135`), which is a selector of SL-004 and a
non-goal of this slice.

## Revision candidates

- **DEC-027 needs no revision**: SL-005 implements the subset it fixes. If design
  drops a form (the scope allows it, "the subset is a floor for the decision, not a
  quota"), that is a scope correction, not a decision revision — surfaced to the
  operator, not edited into DEC-027.
