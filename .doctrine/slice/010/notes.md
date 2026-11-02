# Notes SL-010 — the humane query surface

Durable per-slice scratchpad. The disposable phase sheets live under
`.doctrine/state/`; anything that must survive `rm -rf` lands here.

## Design surface triage (exploring, 2026-11-02)

### Constraining governance

| authority | what it fixes here |
| --- | --- |
| brief § Query contract (`smut-implementation-brief.org:105-165`) | the machine surface: ten keys and their defaults, the pipeline order, five operators with their exact semantics, the six-member envelope. A human surface may be **stricter only** |
| DEC-011 + DEC-019 | a 0.1/0.2 `query` call stays valid and additive; `query {}` stays byte-identical; compatibility is a **tested** property (`tests/compatibility.rs`) |
| DEC-004 | the whole request is validated before any store exists; the store is `<cwd>/.smut`, never searched upward |
| DEC-008 | clap v4 derive owns the command grammar; `--` terminates flags for opaque ids; its `alternatives` line **rejects hand-rolled argument parsing** |
| DEC-001 rules 1, 4 | a core `Reject` is exit 2 because the core can express only user error; the parsed `Limit` has no user-reachable unlimited form |
| DEC-027 | a field condition list that is provably unsatisfiable is refused at parse time by **demand + decidability**; a field no entity carries is **taught, never refused** |
| DEC-030 | the contradiction refusal is one variant, one pinned sentence `field {field} cannot be satisfied: {contradiction}`; a refusal never invents a field and never prints an operand literal; one line, no epigram |
| DEC-016 | the two-call recipe is normative in `learn` and its **JSON spelling is pinned**; the recipe must carry its direction and its completeness guard |
| DEC-007, DEC-012, DEC-013, DEC-014, DEC-015 | envelope and traversal members, all inherited unchanged through `Request::Query` |
| DEC-010 | `learn` is the delivery channel and is limited to the shipped surface |
| ADR-001 | identity is `smut_id`; it decides what "the identity field" means for value typing and for `--ids` |
| STD-001, STD-002 | nothing lands inside `.smut/`; no stored-validity rule changes, so no format bump |
| RFC-001 | the harm ranking: this slice buys Tiers 2–3 (calls and reading), and buys nothing in Tier 1 (silence) |
| IMP-014 | no `Reject` census exists; new variants land unpinned by default |
| IMP-016 | the five operators are already enumerated twice in `src/predicate.rs` |

No specs, no policies; one ADR and two standards, of which only ADR-001 bears.

**The two that actually bite.** **DEC-016** pins the recipe's spelling in prose, in
`src/learn.rs` constants, and in an executed test (`tests/fixture.rs:547-600`), so
changing the recipes to DSL lines obliges a decision amendment. **DEC-008**'s
alternatives line rejects hand-rolled argv parsing, so a failure of the clap spike is a
decision change rather than an implementation detail. Everything else constrains
without colliding.

### Shaping decisions — settled before this run, not re-opened

IDE-006 (`humane query DSL`) is the design: the grammar is defined over **argv tokens**,
not over a string; `parse(argv) -> Spec` and the DSL **adds no semantics**; `from` is
demoted to bare words and `where` to infix conditions (`=`, `!=`, `~`, comma list);
every other specification key is promoted to a flag of the same name; shell-active
characters are banned from the grammar, which is why the wildcard is `any` and `$exists`
is `--has`/`--missing`; `smut query` stays the complete machine surface forever.

The run's work list is the research artefact's **design-input deltas D1–D7**
(`research/research.md` § Design-input deltas), not the mechanism.

### Open questions

Leans are recorded before inquiring and are kept as the prior, not as the answer.

| id | question | blocking | lean |
| --- | --- | --- | --- |
| inq-1 | What is the verb called? | yes | `q`, with `query` keeping JSON forever (a bare positional on `query` would be ambiguous with a JSON operand) |
| inq-2 | How is a bare value typed? | yes | JSON scalar if it parses as one, else a string — documented, and `--ids`/`--from=` named as the escape where identity matters (ADR-001) |
| inq-3 | Is a root a bare word or a sigil-marked token? | yes | bare words, with `--from=ID` as the documented escape for an id carrying `=`, `~` or a leading `-` |
| inq-4 | Can clap's derive carry this argv grammar, or must `q` read `std::env::args` itself? | yes | **answered by spike** — see § Spike evidence below: clap 4.6.7 carries all of it with a bare `Vec<String>` and no attributes, so DEC-008 is untouched |
| inq-5 | How does the compiler reach `Spec`, and what may it share with `spec`/`predicate`? | yes | emit the specification object and route it through `spec::parse`, so "adds no semantics" is structural; DSL-only refusals are voiced in the DSL layer before that call, and the operator table derives from one `Operator` rather than being a third copy (IMP-016) |
| inq-6 | Which repeated conditions on one field are refused, and in whose voice? | no | only two `=` on one field, as a contradiction in DEC-027/DEC-030's family; everything else compiles for `predicate::parse` to judge |
| inq-7 | What does the guide teach, in which spelling, and what does that oblige? | no | `q` for the hot paths, `query` kept and marked complete; DEC-016 amended to name both spellings; the two warts (value typing, `~`'s two meanings) stated rather than hidden |
| inq-8 | Does `--ids` combine with `--select`, and is `--brief` in? | no | `--ids` and `--select` refuse each other (silent last-wins is the failure class this slice removes); `--brief` stays out — output shape is IMP-002's |

### Spike evidence — inq-4 (2026-11-02)

Run in a throwaway crate at `/tmp/smut-clap-spike`, `clap = "=4.6.7"` (the exact
`Cargo.lock` version), fully offline. 21 tests green in debug and release; re-run and
confirmed by the orchestrating agent, not just reported. The verbatim findings are kept
at `research/raw/spike-clap-argv.md`; the crate itself is disposable.

**Verdict: clap carries the grammar, with `pub tokens: Vec<String>` and no `#[arg]`
attribute at all.** Case 4 — flags *after* a positional has started — is accepted with
token order preserved: `q --ids session status=open --depth 2` gives
`tokens=["session","status=open"]`, `ids=true`, `depth=Some(2)`.

Five facts the design must carry rather than rediscover:

1. **Do not add `trailing_var_arg` or `allow_hyphen_values`.** Both are *silent*: the
   first turns every flag after the first token into a token (exit 0, four flags gone);
   the second does the same **and** leaks a literal `--` into the token list once a
   positional exists (`q a -- --ids` → `["a","--","--ids"]`), **and** makes
   `q -weird-id` parse as a valid token, deleting the guard `--` exists to give.
   `last`, `value_terminator` and `num_args` are all non-minimal or worse.
2. **A token immediately after a value-taking flag is that flag's value.**
   `q --follow session status=open` means follow=`session`. Standard CLI convention,
   but it means `--follow part-of` is not a query about a root named `part-of`, so the
   guide's examples must put the value where it reads as one (`--follow any`,
   `--follow=part-of`) and the refusal vocabulary must not imply otherwise.
3. **`q` alone is a valid parse** (all fields empty), so "no query at all" is this
   layer's refusal to make, not clap's.
4. **`q -weird-id` exits 2 naming `-w`**, not the token: clap splits the short-flag
   cluster. Loud and never inventing a field (DEC-030 holds), but inaccurate; echoing
   the real token needs a pre-scan of argv, which is a design choice, not a default.
5. **A repeated boolean flag errors** (`--ids --ids` → `ArgumentConflict`) rather than
   being idempotent; opting into `ArgAction::Count` is available if wanted.

### Risks carried from the scope

R1 (clap) is now **inq-4** and a running spike rather than an open risk. R2 (two ways to
enforce "no new semantics") is **inq-5**. R5 is answered by research F1 and needs only its
voicing decided (**inq-6**). R6 stands and is a guide obligation (**inq-7**): a typo'd
field still answers empty at exit 0, which RFC-001 ranks Tier 1 and DEC-027 decided to
teach rather than refuse. R7 (two engines) is answered structurally by reusing
`Request::Query`, and by the byte-identity test. R8 is **inq-8**. R9 (`--edges` beside a
named direction) needs no decision: the spec's own rule is inherited.

### Assumptions the design will carry

- **A1 — output does not move.** `q` returns the same envelope through the same
  `select::envelope`; any output work is a second slice wearing this one's name.
- **A2 — no new specification key.** If the DSL cannot express something, the answer is
  `smut query`, never an eleventh key.
- **A3 — shell-safety is a grammar property.** It is proved by the token classifier's
  tests, not by a shell harness: `tests/readme_example.rs:242` splits on whitespace and
  performs no glob, tilde or `$` expansion, so it can only prove token survival.
- **A4 — one release, no version gate.** `q` ships in the next release beside `query`;
  nothing feature-detects it.

### Obligations owed at reconcile (pre-recorded)

- **DEC-016** must be amended to name the DSL spelling of the two-call recipe, because
  `learn` prints and `tests/fixture.rs` executes it. (Contingent on inq-7.)
- **DEC-008** may need an amendment if inq-4's spike fails. (Contingent on inq-4.)
- **IMP-014**'s census gap widens with each new `Reject` variant; recording that here so
  closure does not read the missing census as coverage.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-03 · started · PHASE-01 completed · 1fb66de

### Produced

- `research/research.md` — two threads, cross-thread findings F1–F7, design-input deltas
  D1–D7, plus `raw/spike-clap-argv.md` (the twelve-case clap evidence). F1 and D1 were
  **corrected, not rewritten**, after the design review falsified them; the baseline is
  restamped.
- **DEC-034 … DEC-041** — one settled decision per inquiry: the verb, value typing, bare
  roots and the escape, the grammar inside clap's derive, the emit-for-`spec::parse` seam,
  refusing only what JSON cannot spell, what the guide teaches, `--ids`/`--select`.
- `.doctrine/slice/010/design.md` — seven sections, locked at revision 35 and adopted at 27.
- **RV-031** — the review pass: eleven findings raised, disposed `fix-now`/`control`, verified,
  pass concluded. `.doctrine/review/031/` is the authored ledger.
- `.doctrine/slice/010/plan.toml` + `plan.md` — three phases, every routed finding transcribed
  into the criterion that settles it; phase sheets materialised under `.doctrine/state/`.
- Eleven `design-target` and `scope-relevant` selectors, matching design §6.
- **PHASE-01** (`1fb66de`) — `src/dsl.rs` (the compiler: `Line`, `Flags`, `compile`,
  `value`, `OPERATORS`, `Where`), `tests/dsl.rs` (13 tests), `Reject::{JsonOperand,
  UnreadableToken}` with their whole-line tests, and the three `EXAMPLE_*_Q` constants in
  `src/learn.rs` (declared here, printed by PHASE-03). Evidence and the run's environment
  facts: `notes.md` § Implementation run. Runtime sheet:
  `state/slice/010/phases/phase-01.md` § Outcome.

### Learned

- DEC-027/DEC-030 already own the same-field contradiction this slice's IDE-006 rule 3
  wanted to invent; the rule shrinks rather than gets reworded (F1).
- The guide's recipes are pinned three ways — prose decision, `EXAMPLE_*` constants, and
  an executed test — so teaching a new spelling is a decision amendment (F2).
- **clap 4.6.7 carries the whole grammar with a bare `Vec<String>` and no attributes.**
  The danger is not failure but the *silent* alternatives: `trailing_var_arg` and
  `allow_hyphen_values` both misparse with exit 0, and the second leaks `--` into the
  tokens. DEC-008 survives untouched.
- clap already refuses a repeated scalar flag by name at exit 2, so the front-end needs no
  repeated-flag rule — the residual disclosed at sufficiency acceptance closed by evidence.
- `smut q` with nothing at all is `query {}`, which the specification already defines as
  legal, so the front-end has no no-query refusal to write.
- **A recipe's two spellings need not be `Spec`-equal.** Roots compile to `from` while the
  JSON filter twin is `where.smut_id.$in`: only the result set agrees, so the equivalence
  test asserts each line against the JSON *it* denotes and set agreement is PHASE-02's
  (PHASE-01 A5).
- **The plan's flag seam was narrower than its prose.** The guide's recipes force
  `--backlinks`, `--follow`, `--depth` and `--ids` into PHASE-01, and `--depth` already runs
  through `value()` — so PHASE-04's numeric-flag criterion is pinned, not built. `Flags` is
  declared in full; six fields are PHASE-04's to wire, and `dsl::members()` is the single
  comma splitter the list rule must attach to.

### What the hostile pass taught (RV-031)

- **A design claim can be unreachable rather than wrong.** The JSON-operand refusal was
  written in the tables and asserted in the verification section while no branch of the
  classifier could ever reach it: the rule and its detector were only reconciled on paper.
- **A reflective reuse can carry a false sentence.** `ContradictoryFieldConditions` fits the
  two-`=` case exactly; extended to a repeated `!=`/`~` it renders `cannot be satisfied`
  about a predicate some entity satisfies. What forces a refusal is the *emit shape* — one
  key per operator — not the specification, which can spell it with `$and`.
- **`predicate::parse` answers what the design said it would refuse.** `$eq open` beside
  `$ne resolved` passes the decidability walk; the design's verification row asserted exit 2.
- **A file-level "unchanged" table is a claim with a census.** `smut render` had no row
  although the scope non-goals it, and clap's errors are five lines while the design
  called every refusal one.

### Open

- **The plan is hardened.** Two audits (obligations; surface, sequencing, risk) ran against
  `plan.toml`; an integrator settled every blocking and material finding. PHASE-01 split at the
  words/flags seam into `PHASE-01` and an appended `PHASE-04` (execution order is array order, so
  the id sequence is deliberately non-monotonic). Nothing about the design is open.
- **Corrections owed to the locked design** (three found by the hardening audits, one in
  PHASE-01; all pinned in `plan.md`'s "Corrections owed to the design", unpaid here):
  - Design §4's struct block prints `#[arg(value_name = "TOKEN")]` on the positional while its
    prose, its observation table and DEC-037 all say the positional carries no attribute. The
    prose is the decision; the block owes the deletion.
  - Design §3 has no row for a comma list on `~` (`title~a,b`). The plan refuses it in the shape
    `!=` already uses; §3 owes the row.
  - Design §3's rows 1 and 15 both fire on §1's "more than one operator" branch with different
    sentences and no discriminator. The plan discriminates by kind (two `=` reports the
    field-name row; differing kinds report the both-operators row); §3 owes the sentence.
  - *(added in PHASE-01, sheet decision D4)* Design §3 has no row for two comma lists on
    `=`. `status=a,b status=c,d` is satisfiable — the lists may intersect — so the
    contradiction voice would be false; the refusal is in the caller's tokens
    (`cannot read "status=a,b status=c,d": q carries one = per field; use smut query`) under
    §2's third bullet, and §3 owes the row.
- **Obligations owed at reconcile** (none may be paid mid-slice):
  - DEC-016 — amend to name the DSL spelling of the two-call recipe; the guide keeps both
    spellings executed until then.
  - DEC-035 — its "JSON scalar" wording is narrowed by design §2, which excludes JSON
    *string* literals so a value is never unquoted twice.
  - DEC-037 — its "no-query refusal belongs to this layer" consequence is superseded: the
    layer would own it, and §3 decides not to have one.
  - IMP-014 — six new `Reject` variants land with no census to catch a missing one.
  - IMP-016 — the DSL's operator table is a third site that names the five operators.
  - **DEC-039** *(added by RV-031 F-3)* — its illustration that `status=open
    status!=resolved` reaches the contradiction refusal is false: it is satisfiable and
    answered. The decision's *rule* stands; only that sentence needs correcting.
- Slice is `started` and **PHASE-01 is `completed`** (2026-11-03). Next: `/phase-plan` for
  PHASE-04, then PHASE-02, then PHASE-03 — the array's order, not the ids'.

## Implementation run

### Run preflight — continuation (2026-11-03, PHASE-04 → PHASE-02 → PHASE-03)

Seat-level facts for this run, reusing PHASE-01's and refreshing what could have
moved since:

- **Lifecycle.** SL-010 `started`, `phases: 1/4`; PHASE-01 `completed` at `1fb66de`,
  PHASE-04/02/03 `planned`. Design locked at revision 35; plan approved; the three
  remaining runtime sheets are materialised stubs and are `/phase-plan`'s first job.
- **Gate.** `just check` green at `2e3eeb0` before any charter — `build`, `lint`
  (`clippy --all-targets -D warnings`, default *and* `--all-features`), `fmt
  --check`, and `cargo test` in both feature configurations. Exit 0, no warnings,
  ~2 s warm, no network.
- **Dirty paths.** `git status --porcelain` shows only `?? .quotient/` and `??
  _envrc` — untracked, not ours, never restaged. `README.md`, `src/error.rs` and
  `.doctrine/slice/008/` are clean (SL-008 landed at `d1f0700`).
- **Toolchain in this jail.** `cargo 1.100.0-beta.7`, `rustc 1.100.0-beta.7`; `/tmp`
  and `$HOME` writable; native `subagent` delegation available with a generic
  `sub-orchestrator` role (nested children allowed).
- **`doctrine slice verify-vt 10`** is red exactly where PHASE-04/02/03 are owed, and
  its own words mislead in three places: PHASE-04 VT-3/VT-4 FAIL on absent keywords
  in `tests/dsl.rs` (real work, not a missing file); PHASE-03 VT-2 FAILs only because
  `src/learn.rs` is already in this slice's delta — its keyword lands with PHASE-03;
  PHASE-02's three read UNATTRIBUTABLE until its delta is recorded. Attribution
  follows a **recorded delta** (`doctrine slice record-delta 10 <PHASE> --commit
  <sha>`), not a commit, so each phase records its own after it lands.
- **PHASE-04's starting point.** `src/dsl.rs` (403 lines) has `Line`, the full
  `Flags`, `compile`, `value`, `OPERATORS`, `Where`, `members()` and `names()`. Six
  flag fields are unwired (`budget`, `select`, `order`, `limit`, `edges`, `from`),
  `members()` is the single comma splitter and the list rule attaches there, and
  `Reject` already holds two of the six `q` variants (`JsonOperand`,
  `UnreadableToken`) — PHASE-04 owes the other four, the twelve flag keys, the list
  and strictness rules, and every message test.
- **Charter order** is the plan array: PHASE-04 → PHASE-02 → PHASE-03. Ids are
  append-only; the split seam is not the sequence.

### PHASE-01 — the compiler's core (completed 2026-11-03)

Environment facts, so the next phase does not rediscover them:

- **Gate.** `just check` = `cargo build`, `cargo clippy --all-targets -D warnings` (default
  *and* `--all-features`), `cargo fmt --check`, and `cargo test` in both feature
  configurations. Warm it is ~2 s and needs no network.
- **Baseline before this phase** was green — 433 unit tests plus the per-suite counts, clippy
  and fmt clean — so a red later is this slice's. Green after it: 435.
- **Other writers' paths.** `README.md`, `src/error.rs` and `.doctrine/slice/008/notes.md`
  were dirty at the handover and are clean now (SL-008 landed); `.quotient/` and `_envrc` are
  still untracked and still not ours. Nothing here restaged them (CHR-001).
- **Research baseline** was restamped, because the drift was `plan.md`/`plan.toml` — the plan
  artefacts this slice itself added. No thread section was affected.
- **Files this phase touched:** `src/dsl.rs` (new), `src/lib.rs`, `src/error.rs`,
  `src/learn.rs`, `tests/dsl.rs` (new). Nothing else was staged.

Decisions taken inside the phase sheet's envelope (reasons in
`.doctrine/state/slice/010/phases/phase-01.md` § Decisions):

- **D1** — the three `EXAMPLE_*_Q` constants are *declared* in `src/learn.rs` here and
  *printed* by PHASE-03. PHASE-01's VT-1 cannot compile otherwise, and PHASE-03 EX-4 has
  `tests/dsl.rs` import the same constant, so the constant is the shared fixture either way.
- **D2** — `Flags` is declared in full as design §2 prints it; six of its fields are wired by
  PHASE-04. The struct is the seam, not this phase's output.
- **D3** — one `Reject::UnreadableToken { token, reason }` covers the malformed *and* the
  ambiguous token, so every `cannot read "{token}": …` row of §3 is one shape and §6's
  census stays six variants. The `a=b=c` / `status~a=b` discriminator is the reason string,
  which is why the plan's third design correction needs no second variant.
- **D4** — two comma lists on `=` are refused in the caller's tokens, not as a contradiction:
  an intersection can be non-empty, so the contradiction voice would be false (§2's third
  bullet). Recorded as the fourth design correction above.

Findings worth keeping:

- **`EXAMPLE_FILTER_Q` is not `Spec`-equal to `EXAMPLE_FILTER`.** Roots land in `from`, while
  the JSON twin filters `where.smut_id.$in`; only the result set agrees. The equivalence test
  therefore asserts each line against the JSON *it* denotes, and set agreement stays PHASE-02
  EX-7's. Comparing the recipe's two spellings Spec-equal would have been a false claim — the
  same class as `mem.fact.smut.vacuous-negative-assertions`, one level up.
- **The plan's flag seam is narrower than its prose.** The three recipes need `--backlinks`,
  `--follow`, `--depth` and `--ids`, so four "PHASE-04" flags are PHASE-01's, and PHASE-04
  gains behaviour on a declared struct rather than the struct itself. `--depth` already goes
  through `value()`, so EX-9 is satisfied by construction and only *pinned* by PHASE-04.
- **The suite bites.** Four mutations were applied, red observed, and reverted: `value()`
  accepting JSON string literals; the `{` test moved below the operator scan; `$in` repeats
  voiced as a contradiction; `from` emitted even when empty. The `$exists` collision asserts
  the variant and the whole sentence, so an overwrite cannot pass it.
- **`where` key order** is the `BTreeMap`'s, so a compiled line is byte-stable; the equivalence
  tests compare `Value`s, which is order-insensitive, and determinism is PHASE-04 EX-10's pin.
