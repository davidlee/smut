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
- **A further pass would probe:** (1) the refusal sentences once *implemented* — the design's truth
  test is a reading; the phase that changes `dsl.rs`/`error.rs` should re-run it against the real
  strings (`mem.pattern.smut.refusal-pins-hold-shape-not-truth`); (2) clap behaviour of the
  flattened structs in the real crate rather than the scratch probe — first phase's tests own it.
  Neither needs a pre-lock design pass: both are execution-time checks named in § Verification.
  An independent (GPT) design pass was offered and the user chose pi first; not run.

## Reconcile obligations (from design § Reconcile obligations)

- Amend DEC-034, DEC-008, DEC-039, DEC-040 via `doctrine knowledge edit decision`, comma-free arrays.
