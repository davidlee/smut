# Notes SL-004: Identity is smut_id — the format break, and a loader that refuses what it cannot trust

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

## Working mode (operator)

Inquiry is taken **one question at a time**: the agent drafts the answer, the
operator nods or pushes back, and only then is it recorded in the run. Do not
answer two nodes in one turn, and do not record an answer the operator has not
seen. The operator also reshapes the tree freely — "just reshape the tree as you
need to".

`graph-reviewed` was recorded at rev 8 and invalidated when the inquiry graph was
declared at rev 9, as designed. Both it and `sufficiency-accepted` need the
operator's assent again once the graph is final — lay out what each is assenting to
before asking.

## inq-2 draft — loader validation of nested identity (IMP-005)

**Status: drafted, not yet shown to the operator for a nod.** Not in the run.

- **Where.** `Store::load`'s entity-file path (`src/store/mod.rs:218-238`), after the
  existing is-an-object and root-identity checks. The file is already parsed there, so
  no new pass or re-read.
- **What.** A recursive walk over the loaded entity establishing exactly what
  `reference_id` already decides, so one rule and not two: any object carrying
  `smut_id` must carry a nonempty string there. A payload's ordinary `id` is never
  inspected — after this change it is data.
- **Failure shape.** `StoreError::Corrupt { entry, reason }`, exit 1, naming the file
  and the offending path (e.g. `payload.tags[2].smut_id`). Corrupt rather than a new
  variant: the store's contract is that a store it did not write is refused, never
  repaired, and `Corrupt` is that contract's spelling. Consistent with the
  failure-taxonomy memory: internal invariants assert, user data is refused, and a
  malformed store is user data.
- **Why the loader, not `scan_refs`.** Making `scan_refs` return `Result` threads an
  error through every traversal call site for a condition that must not exist in a
  loaded store, and converts corruption into a mid-query failure with partial results
  already built. `scan_refs` keeps its non-fallible validated-precondition signature;
  its panic stays as an assertion and stops being reachable from a loaded store.
- **Trade to put to the operator (already flagged, no answer yet).** The walk is
  eager, once per entity at load. Trivial for the 66-entity fixture; the same order as
  parsing for a large store. The alternative is lazy validation at `scan_refs` call
  sites, which is the shape this note rejects above — so the trade is "eager walk at
  load" vs "Result-threading through traversal". Default: eager.

## Operational hazards

- **The operator's README draft is staged and uncommitted.** A bare `git commit`
  sweeps it into the commit; this happened twice and was repaired twice. Always
  commit with an explicit pathspec (`git commit ... -- .doctrine`).
- Minting a standard is the chosen mechanism for a prose surface that no test can
  hold: its title rides every agent's boot context (STD-001 is the first).
