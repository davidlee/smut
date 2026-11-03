# Notes SL-011: Markdown from a q line — render reachable through the humane grammar

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-03 · audit (RV-037 concluded) · ef76f19

### Produced

- PHASE-01..04 — `-j` on `q`, `JsonBesideSelection`, the `-j` escape pointers, the verb `r`, guide and README (commits `9a54c97`..`5aae61c`; registry exact, RV-037 F-1)
- RV-037 — audit ledger: seven findings, synthesis, reconciliation brief

### Learned

- mem.fact.clap.flatten-doc-comment-fills-undocumented-variant — a flattened struct's `///` leaks only into a variant without its own doc
- mem.pattern.smut.refusal-pins-hold-shape-not-truth — re-confirmed one layer out (README, records, comments: RV-037 F-2..F-5)

### Open

- RV-037 brief — eight DEC amendments (DEC-034, 008, 039, 040, 035, 036, 037, 043), design C1–C6 with C6 extended, README and comment mirrors
- IMP-014, IMP-015, IMP-016 — touched and named, not paid

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
  (RV-037 F-3/F-4 extended this to eight: + DEC-035, DEC-036, DEC-037, DEC-043; paid — § Reconcile.)
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

### PHASE-03 (sub-orchestrator, 2026-11-03)

- Commit `1fc3c24`; `just check` exit 0 (before and after the final fmt). Delta recorded `--commit 1fc3c24`.
- `Sub::R` sits after `Render` in `Sub` so `--help` lists `query, q, render, r`. Doc: "Render a line of words as
  Markdown: `q`'s line, `render`'s display flags."
- VA-1: `git diff --stat 046b360 -- src/spec.rs src/predicate.rs src/select.rs src/render_markdown.rs src/input.rs
  docs/store-format.md tests/compatibility.rs` prints nothing. The `R` arm calls only `display.options()` and
  `selection.specification(..)` inside `cli::parse`; no `Store` exists before `parse` returns (main dispatches after).
- Negative control: swapping the arm (selection before display flags) reddens `r_refuses_without_creating_a_store`
  (the `r -l 7 --ids --select a` row); restored.
- pi pass (implement, deepseek-flash): red tests $0.058, 34 req, red-first evidence (all r rows failed on
  `unknown command: r`; `query_and_render_do_not_take_dash_j` green as a characterisation). Misses: wrote
  `r -l -j` without the criterion's trailing `3` (fixed by me). Departures it reported, accepted: traversal twin
  drops `EXAMPLE_TRAVERSE`'s `select` so headings are visible; `r -title` asserts the identity fallback, since no
  entity carries `itle`. Green (the arm, ~10 lines) written by me. No Sonnet/Opus spawn.

### PHASE-04 (sub-orchestrator, 2026-11-03)

- Commits `738fec3` (constants + fixture runners), `9c5a2dc` (guide prose + pins), `5aae61c` (README); `just check`
  exit 0 before each. Delta recorded `bb8a05b..5aae61c`. `tests/compatibility.rs` untouched since `046b360`.
- Negative control (pi, verified by reading): dropping the `q-query` declaration leaves `r`/`render` equal on the
  missing notice and fails only the `## Query design` assertion. Filter row compared as a set via `rendered_ids`
  (ids printed as backticked paragraphs) and checked equal to `OPEN_QUESTIONS`.
- VA-1 reading (guide and README as a reader): every new spelling probed on the built binary (`r` line, `r -j -`,
  empty `q`/`r`, `r -- -title`, `r -title` → `-t itle`, `-j` beside tokens refused, `query -j` refused). Nothing taught
  is refused; `query` stays "the complete surface"; no sentence calls `q` the only front end. The Opaque ids `--`
  rescue is unchanged. The README does not repeat the `r` hyphen rule (EX-6 does not ask for it; the guide teaches it).
- Passes: pi implement (deepseek-flash) fixture: run 1 timed out on provider 503s after 10 req, $0.006, no edits;
  continued session $0.017, 19 req, clean (I renamed one local and reused `sorted_ids`). Sonnet (general-purpose;
  `capsule-worker` not offered here) guide prose + pins, red-first evidence (8 pins red, then green); one over-long
  wrapped line fixed by me. pi README $0.022, 14 req, clean.

## Autonomy log
- PHASE-01 · wrote the EX-1 pin commit (two assertions) myself rather than via pi · brief would cost more than the edit · decided
- PHASE-01 · EX-6 rows added as parse-level spec comparisons in `src/cli.rs` (`q a --follow x b`, `q -- -weird`, `q a -- --follow`, `q -- -junk`); none existed there · criterion says "where absent, added at the parse level" · decided
- PHASE-03 · wrote the `R` arm myself (green), pi wrote the tests · brief would cost more than ~10 lines; tests and code still by different hands · decided
- PHASE-03 · placed `r` after `render` in the verb list (doc text "…: `q`'s line, `render`'s display flags") · mirrors `query`/`q`; criterion allows "or equivalent" · decided
- PHASE-04 · guide prose and pins delegated to Sonnet (general-purpose, model override) rather than pi · criteria-literal prose with moving and frozen pins; provider 503s on pi · decided
- PHASE-04 · Sonnet added one guide sentence beyond the brief ("-j JSON replaces the words with the whole specification, and is refused beside a token or a selection flag") · true by probe, keeps scope · decided
- PHASE-04 · README `smut r` step placed in the `~`/comma-list block of "Querying, quickly"; README omits the `r` hyphen rule · EX-6 permits one step and does not list the rule; guide carries it · decided

## Audit — RV-037 (2026-11-03)

Ledger `.doctrine/review/037/`: seven findings, all verified; `## Synthesis` and
`## Reconciliation Brief` in `review-037.md`. Surface: main worktree at `ef76f19`.

**Verdict.** Behaviour matches design.md as corrected by plan C1–C6; no finding is against
behaviour. `just check` exit 0, `doctrine check gate` exit 0, `verify-vt 11` 14/14 PASS,
conformance 9/0/0 over an exact registry, `tests/compatibility.rs` no diff since `046b360`.

| id | sev | disposition | one line |
| --- | --- | --- | --- |
| F-1 | nit | aligned | registry spans exact; conformance 9/0/0 |
| F-2 | major | design-wrong | README:476-478 "without `--` it is a usage error" — false on `r` (`-title` silent) and for `-j…` words |
| F-3 | minor | design-wrong | C6 understates: `-site` also silent on `r`; `-j…` words get the JSON reader's voice; DEC-037/043 own nothing of it |
| F-4 | major | design-wrong | obligations miss DEC-035, DEC-036 and DEC-040's escape sentence |
| F-5 | minor | fix-now | six comments/docs still describe one verb; `src/cli.rs:244-245` now false |
| F-6 | nit | design-wrong | README table shows `-j` on `q`'s row, not `r`'s |
| F-7 | nit | aligned | autonomy log's decided lines (bar the README one) and C1–C5 hold |

**Autonomy log, read against the tree.** The Sonnet guide sentence is true by probe; the
traversal twin is a consistent pair; the "README omits the `r` hyphen rule" decision is
overturned by F-2 — the README states the opposite.

**For reconcile.** Design prose first, then mirrors copied from it, then records (brief order).
F-5 and the README mirrors are code/README edits: the operator rules whether reconcile pays
them, as RV-034's were. `plan.toml` is off-surface.


## Reconcile — RV-037 paid (2026-11-03)

**Operator rulings (2026-11-03).**
1. Reconcile pays the off-surface edits, as RV-034 set precedent: README.md opaque-id bullet (F-2),
   README.md `r` commands row `-j` (F-6), and the F-5 code comments (src/cli.rs, src/error.rs,
   src/dsl.rs). Comment and doc text only — no behaviour change.
2. The `.doctrine/observations/` record committed in `c30398d` stays committed. No action.

**Order paid.** Design prose → mirrors copied from it → records, so each mirror copies a sentence
that already existed. The design run was **not** resumed; design.md was amended in place (locked).
`plan.toml` untouched (off-surface). plan.md's supersession table is left as the execution record;
its rows C1–C6 are now paid in design.md.

**Design (`3b1cb0b`).** C1 (three `q`-subject sentences, located), C2 (filter recipe set-equal),
C3 (three hot paths named), C4 (terminal half in-process), C5 (`render`/`query` `-j` asserted, in
§ The surface and § Verification), C6 extended per F-3 as a "Hyphen-led words" paragraph in
§ The command line — the canonical sentence the code comment and DEC-037 copy. § The guide and the
README: the `r` row notes `-j` (F-6) and the narrowed opaque-id bullet (F-2). § Reconcile
obligations lists eight records (F-4).

**Mirrors (`3b1cb0b`).** README.md opaque-id bullet and `r` row; src/cli.rs (four comment sites),
src/error.rs (two variant docs), src/dsl.rs (module doc). Evidence: built-binary probes of
`get -weird-id`, `delete -weird-id`, `q -weird` (usage error + `--` tip), `q -jane`/`r -jane`
(`malformed JSON`, exit 2), `r -site` (exit 0), `r -- -title` (reaches the id) — each matches the
sentence. `--help` for root and all nine verbs byte-identical before/after (no doc comment touched
is clap-visible: the edited `///` sit on a method and on `Reject` variants). `just check` exit 0.

**Records.** DEC-034, DEC-008, DEC-039, DEC-040, DEC-035 (choice facet), DEC-036, DEC-037, DEC-043
(consequences facet, every item rewritten comma-free and checked in the TOML diff), each with a
dated body amendment. No REV: DEC is not a `revision change add` target (SL-010/REV-003 precedent).

**Gate.** `just check` exit 0; `doctrine check gate` exit 0; `tests/compatibility.rs` unchanged.
Outcome recorded in `review-037.md § Reconciliation Outcome`. Hand-off: /close.
