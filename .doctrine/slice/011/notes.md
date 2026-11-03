# Notes SL-011: Markdown from a q line — render reachable through the humane grammar

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

## Design review passes

- **RV-036** (2026-11-03): two pi rounds (deepseek-flash, `review` profile). Round 1 raised F-1..F-12
  against code; round 2 verified all twelve and raised F-13..F-15 (minor), fixed in place and checked
  by the orchestrator only. Concluded.
- **Plan hardening** (2026-11-03): two lanes (`plan.A.local.md` obligations catalogue,
  `plan.B.local.md` implementation surface) plus an integration pass. Three defects were
  verified against the tree and fixed in the plan, not just noted: PHASE-01/EN-3's "green on
  the slice base" was false (`tests/cli_contract.rs:547` pins `smut 0.2.2` while `Cargo.toml:3`
  reads `0.3.0`, so `just check` is red now); PHASE-03's `MissingInput` divergence was
  unassertable in the contract harness (`run` always pipes stdin, `tests/cli_contract.rs:96-124`)
  and moved in-process to `src/cli.rs`; PHASE-04's render-recipe row was vacuous (`q-query`
  occurs zero times in `tests/fixtures/interview/entities.json`, so both spellings render only
  the missing notice) and now declares the entity and asserts `## Query design`. Durable help
  pins added as PHASE-01/VT-3, VT-4, since the refactor's failure modes had none. Two lane
  reports disagreed on the design's "two name `q` as the subject"; I read the strings — three
  do, counting `src/error.rs:387`.
- **Open for the operator, before PHASE-01:** the version pin decision recorded in
  PHASE-01/EN-3 — move `tests/cli_contract.rs:547` with the bump (as CHR-004 did) or record a
  known-red baseline for that one test. Either way it must be settled before PHASE-03, which
  edits the same file.
- **A further pass would probe:** (1) the refusal sentences once *implemented* — the design's truth
  test is a reading; the phase that changes `dsl.rs`/`error.rs` should re-run it against the real
  strings (`mem.pattern.smut.refusal-pins-hold-shape-not-truth`); (2) clap behaviour of the
  flattened structs in the real crate rather than the scratch probe — first phase's tests own it.
  Neither needs a pre-lock design pass: both are execution-time checks named in § Verification.
  An independent (GPT) design pass was offered and the user chose pi first; not run.

## Slice base

- `046b360` (2026-11-03): `just check` green in both feature configurations — the base for PHASE-03/VA-1's `git diff --stat`.

## Reconcile obligations (from design § Reconcile obligations)

- Amend DEC-034, DEC-008, DEC-039, DEC-040 via `doctrine knowledge edit decision`, comma-free arrays.
- Amend design.md for C1–C6 — the single list is plan.md § Where this plan supersedes design.md
  during execution; until reconcile, those rows govern.
