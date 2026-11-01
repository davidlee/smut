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
| 01 | foundation | The error boundary, the lib/bin seam and the durable-write helpers are what every later phase is written against. Building them first means no phase open-codes a write or an fsync that a later phase has to unpick, and integration tests have a crate to drive. The fault seam lands here too, because the durability suite cannot be retrofitted onto a protocol that did not plan for injection. The gates go in here, because a repo whose `just check` is a placeholder can accumulate an unchecked phase. |
| 02 | declaration core | Pure, depends only on 01, and defines the vocabulary (`Entities`, `normalize`, `apply`, `plan_delete`, `WriteSet`) the store phases carry. |
| 03 | selection core | Pure sibling of 02. Kept separate so each algebra lands with its own property tests rather than one oversized "core" phase. |
| 04 | store format, index, lock | The container the journal mutates, plus the pipeline seam the CLI will call. Deliberately stops short of writing an index: publication belongs to 05, so exactly one code path can create one. |
| 05 | journal and its evidence | The hardest part, and the one the user named as such. It gets the sharpest verification in the plan: per-boundary interruption, repeated recovery, per-member injected fsync failure, and a human read of the suite against the design's three protocol sections. |
| 06 | renderer | Pure presentation over `Selection`; lands before the CLI so the pipeline phase integrates a finished part instead of inventing one. |
| 07 | pipeline and CLI | Integration of already-tested parts, which is the only way the pipeline order and the envelopes are visible as contracts. |
| 09 | end-to-end seams | Round trip, concurrency, compatibility — the three suites that need a spawnable binary. Split out of 07 after review so 07 can land green in one pass. |
| 08 | documentation | Last because every claim it makes must be checked by running the binary, not by reading the design. |

The durability phase is not last, and the concurrency scenarios live in the first phase that can spawn a binary rather than in a deferred hardening phase. That is the user's
priority expressed as an ordering decision, not as an aspiration.

**Why the ids are not in array order.** PHASE-09 was split out of PHASE-07 after a
review found that phase too large to land green. Phase ids are immutable and never
renumbered, so the new phase took the next id while its three criteria — the round
trip, concurrency and compatibility obligations, formerly PHASE-07/EX-6, EX-7 and
EX-8 — moved with it. The array position, not the id, is the execution order.
PHASE-08's entrance criterion now gates on PHASE-09 for the same reason.

## The four things this plan had to decide for itself

1. **`README.md` already exists.** Design §12 says all paths are new except
   `justfile` and `Cargo.toml`, and calls `README.md` new. The tree disagrees:
   the owner's README and `smut.jpg` predate this slice, carrying brand copy.
   The plan treats it as an **edit that preserves that copy** and adds the
   documented sections, and PHASE-08's first exit criterion says so.
2. **A lib target is needed, and §12 does not list one.** §11.1 places the
   durability, corruption and selection layers under `tests/**`, and Rust
   integration tests cannot reach a binary-only crate except by spawning it —
   which three of those layers cannot do before the CLI exists, and the
   durability layer would rather not do at all. PHASE-01 therefore creates
   `src/lib.rs` beside `src/main.rs`: the library exposes the modules, the binary
   stays a thin dispatcher. This is the one structural addition to §12's module
   map, and it is recorded here rather than slipped in.
3. **`just test` runs the feature matrix.** With the fault seam `cfg`-gated out of
   default builds — which is what keeps injection out of production — a
   feature-gated suite compiles to zero tests under `cargo test` and would pass
   vacuously. `just test` runs default and `--features fault-injection`, and
   PHASE-05 requires a non-zero executed count so the silence cannot return.
4. **The notice's position is load-bearing, so it is tested as such.** PHASE-06
   tests the notice against the two hostile bodies the review raised, rather
   than testing the notice's text. A guarantee tested only on polite input is not
   evidence. Its zero-entity clause is narrowed to the `missing` notice, because
   `truncated` can only be set when at least one row was returned.

Items 1 and 2 are errata against the locked design's §12 table — factual, not
architectural. They are recorded rather than repaired because the design is
locked and nothing downstream depends on either sentence; the next design touch
should fold them in.

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
