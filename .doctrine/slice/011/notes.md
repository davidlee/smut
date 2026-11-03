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

## Implementation run

### Preflight (2026-11-03, seat: super-orchestrator, PHASE-01 → PHASE-04)

- **Plan.** Design locked; `plan.md` § Where this plan supersedes design.md (C1–C6) governs
  execution where a row applies. PHASE-01/EN-3 (version pin) settled at `046b360`.
- **Host gate.** `just check` from `/home/david/dev/smut` (PATH already has the dev tools; no
  `nix develop` wrapper needed). Exit 0 at `c3d244a`, both feature configurations, tree clean.
  The host gate is authoritative.
- **quotient `write` form.** cargo/rustc/just/git present, root writable; `just build`, `just lint`,
  `just fmt` ran; `cargo test` (default) passed. The probe timed out during the remaining test runs,
  likely waiting on the cargo `target/` lock while the host gate ran concurrently. No sandbox-only
  failures known. Workers: run targeted tests (`cargo test --test <name>`), and do not run cargo
  concurrently with another cargo process on the same `target/` if avoidable.
- **Dirty paths at start.** None (`git status --porcelain` empty).
- **Harness.** Background tasks enabled (spawns run in background; parent woken by notification).
  Spawn depth unset (≥ 2): Sonnet/Opus children available.
- **Operator rulings.** None beyond the plan. No operator-supplied inputs needed.
- **Charters.** PHASE-01+02, PHASE-03, PHASE-04.

### PHASE-01 (sub-orchestrator, 2026-11-03)

- Commits: `9a54c97` (EX-1 pins: about line, render usage), `7c255e2` (refactor, no test edited),
  `0c37d52` (`-j` + `JsonBesideSelection`, tests). `just check` exit 0 after the refactor and at `0c37d52`.
  Delta recorded `1ddf284..0c37d52` (all three commits are this phase's).
- VA-1: refactor diff read alone (`git diff -w 7c255e2^ 7c255e2`): fields moved with attributes and doc
  comments byte-identical; `check_format`/`heading_level_of` unchanged; `options()` still runs before
  `operand`. Struct comments are plain `//` (a `///` on a flattened `Args` struct would reach the parent's
  help). `--help`, `q --help`, `render --help` byte-identical to pre-refactor captures (also after `-j`
  for `--help`).
- pi passes (implement profile, deepseek-flash): refactor $0.012, 14 req, clean; red tests $0.030,
  27 req (mis-cited EX-4/EX-5 and DEC-008 in two test comments; corrected before commit); green $0.018,
  19 req, clean. No Sonnet/Opus spawn.

### PHASE-02 (sub-orchestrator, 2026-11-03)

- Commit `f0240d3`; `just check` exit 0. `src/learn.rs` and `tests/compatibility.rs` untouched (EX-4, EX-5);
  `grep -n 'use smut query' src/dsl.rs src/error.rs tests/dsl.rs` prints nothing.
- pi pass (implement, deepseek-flash): $0.018, 20 req, red-first evidence given; clean. It hoisted the
  JsonOperand sentence into a `JSON_OPERAND_SENTENCE` const in `tests/dsl.rs` (still a whole-line pin).
- VA-1 truth reading (inputs that reach each sentence; `r` will reach the same `compile`, so "both verbs"
  holds once PHASE-03 lands, and `-j` exists on both):
  - `JsonOperand` — any token whose first character is `{`, on either verb, including after `--`
    (`q -- {weird}`). A JSON word: "goes after -j" is true. A `{`-led id (`{weird}`, `{a,b}`): `--from=`
    takes one id with its commas as text, so "an id after --from=" is true. The subject ("a word beginning
    with {") is the observable fact, not a guessed cause. `--from={x}` never reaches it (flag, not token).
  - two of one operator (`a=b=c`, `url=a=b`) — subject is the short form, true of both verbs; `-j` can
    spell either split as a `where` key/value. True.
  - two operators (`status~a=b`) — `-j` spells the predicate as JSON. True.
  - comma list on `!=`/`~` — "write two conditions or use -j": unchanged clause plus `-j`, which has the
    same power as `smut query` (same `spec::parse`). True.
  - one spelling per field — subject "the short form", true of both verbs; escape `-j` ≡ the former
    `smut query` escape in power, so truth is unchanged from SL-010's accepted sentence.

## Autonomy log
- PHASE-01 · wrote the EX-1 pin commit (two assertions) myself rather than via pi · brief would cost more than the edit · decided
- PHASE-01 · EX-6 rows added as parse-level spec comparisons in `src/cli.rs` (`q a --follow x b`, `q -- -weird`, `q a -- --follow`, `q -- -junk`); none existed there · criterion says "where absent, added at the parse level" · decided
