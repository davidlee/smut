# Plan SL-001 — smut 0.1

Executable sequencing for the capability designed in
[`design.md`](design.md) and scoped in [`slice-001.md`](slice-001.md). The plan
adds no design; where it needed a fact the design did not carry, the fact is
stated here and flagged rather than invented in a phase.

## What the plan is built on

- **Design is locked** — run `dr-01a19258`, revision 53, twelve sections
  attested, review concluded by disposition on RV-003.
- **No routed findings to transcribe.** Every finding on RV-001, RV-002 and
  RV-003 was routed `review` and repaired in prose, so no criterion owes an
  instrument obligation on the review's behalf.
- **Research advisory** — the baseline drifted when the scope and `design.md`
  moved. Re-stamped after checking the drift: the scope edits sharpened three
  assertions and the design added the staged barrier, neither of which touches a
  crate choice or a Thread 2 fact. The rested-on facts (duplicate-key rejection
  must be a custom visitor; `serde_json::Map` is order-insensitive and sorted;
  clap exits 2; a durable rename needs a parent fsync; `CARGO_BIN_EXE` for
  end-to-end tests) all still hold against the current tree.
- **Design premises re-grepped** against the tree: `Cargo.toml` (no
  dependencies, edition 2024), `justfile` (placeholder targets), `src/main.rs`
  (placeholder), no `tests/` and no `docs/` — all as the design's current-state
  section states.

## Ordering, and why

| | phase | why here |
|---|---|---|
| 01 | foundation | The error boundary and the durable-write helpers are what every later phase is written against. Building them first means no phase open-codes a write or an fsync that a later phase has to unpick. The gates go in here too, because a repo whose `just check` is a placeholder can accumulate an unchecked phase. |
| 02 | declaration core | Pure, depends only on 01, and defines the vocabulary (`normalize`, `apply`, `WriteSet`) the store phases carry. |
| 03 | selection core | Pure sibling of 02. Kept separate so each algebra lands with its own property tests rather than one oversized "core" phase. |
| 04 | store format, index, lock | The container the journal mutates. Deliberately stops short of writing an index: publication belongs to 05, so exactly one code path can create one. |
| 05 | journal and its evidence | The hardest part, and the one the user named as such. It gets the sharpest verification in the plan: per-boundary interruption, repeated recovery, per-member injected fsync failure, and a human read of the suite against the design's three protocol sections. |
| 06 | renderer | Pure presentation over `Selection`; lands before the CLI so the pipeline phase integrates a finished part instead of inventing one. |
| 07 | pipeline and CLI | The integration phase. Everything it wires is already tested in isolation, which is the only way the seams — round trip, concurrency, cross-process recovery — are visible as seams. |
| 08 | documentation | Last because every claim it makes must be checked by running the binary, not by reading the design. |

The durability phase is not last, and the concurrency scenarios live in the first
phase that has a binary rather than in a deferred hardening phase. That is the
user's priority expressed as an ordering decision, not as an aspiration.

## The two things this plan had to decide for itself

1. **`README.md` already exists.** Design §12 says all paths are new except
   `justfile` and `Cargo.toml`, and calls `README.md` new. The tree disagrees:
   the owner's README and `smut.jpg` predate this slice, carrying brand copy.
   The plan treats it as an **edit that preserves that copy** and adds the
   documented sections, and PHASE-08's first exit criterion says so. The design
   sentence is now factually wrong; it is recorded as an erratum for the next
   design touch rather than silently repaired, because the design is locked and
   nothing downstream depends on the sentence. This is the one place the plan
   departs from a locked design statement, and it departs in the direction the
   owner's existing file requires.
2. **The notice's position is load-bearing, so it is tested as such.** PHASE-06
   tests the notice against the two hostile bodies the review raised, rather
   than testing the notice's text. A guarantee tested only on polite input is not
   evidence.

## Risk, and what the plan does about it

- **The journal is where a wrong build is silent.** Mitigation is structural:
  every write and fsync goes through 01's helpers, 05's boundary list is
  enumerated from §4.3/§4.4 rather than from memory, and the human criterion
  reads the suite against those sections. The plan asserts code-path ordering
  only; it never claims power-loss durability, and PHASE-05's last exit
  criterion says so in the test file itself.
- **The unit-test seam is real.** `normalize → write set → commit → reload` is
  covered by neither side's unit tests. PHASE-07 gives it a small dedicated
  suite (EX-6) rather than hoping the CLI tests cover it.
- **Concurrency tests are flaky by nature.** They assert invariants — created at
  most once, no partial batch visible, no unrecovered journal — with bounded
  retries, never interleavings.
- **Property tests are the only defence on two modules.** `numeric.rs` and the
  declaration fold get proptest from PHASE-01 and are never "covered later".
- **Scope creep into 0.2.** The named-future key rejection and the compatibility
  suite (PHASE-07) exist so that widening the spec surface later cannot silently
  change a 0.1 call.

## What is deliberately not a phase

- No migration, repair or recovery-of-corruption phase: 0.1 refuses, and the
  design says so.
- No benchmarking phase: the brief schedules a smoke benchmark for 1.0 (IMP-002).
- No Mermaid work: refused by design in 0.1 with a message naming 1.0.
- No release, packaging or licensing work: not a product behaviour, and the
  owner has not decided licensing.
