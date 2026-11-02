# Notes SL-004: Identity is smut_id — the format break, and a loader that refuses what it cannot trust

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · **audit complete** — RV-028 8/8 terminal, pass concluded; slice at `reconcile` · code base `64e5f4b` + the audit repair (F-5) · gate green after the repair

### Produced
- SL-004 — six-section materialised design and code-impact selectors
- RV-023 — design self-review and verification limits
- SL-004 — implementation plan and phase boundaries
- RV-024 — pre-implementation pass (13 findings: design→plan capture, sequencing, adjacent decisions)
- RV-025 — amendment pass (19 findings over design revs 40-43; both blockers and every major verified)
- SL-004 — design re-locked at rev 47 after two adversarial passes; plan repaired append-only
- STD-002 — version discipline for stored-validity rules
- CHR-001 — operator's 0.2 README rewrite
- RSK-001 — unenforced-invariant gap (promoted into STD-002)
- SL-004 — three review reports under `research/raw/`
- SL-004 PHASE-01 — `validate_identity_tree` + `walk_identities` in `src/store/mod.rs`, the
  bracketed identity-path notation and its pending-PHASE-02 doc material; 31 new tests across
  `tests/store_corruption.rs` (11), `tests/cli_contract.rs` (6) and the inline walk suite (14)
- SL-004 PHASE-02 — the format break itself: `ENTITY_ID` = `smut_id`, both versions 2, ordinary
  `id` as data, four diagnostic spellings, the spelling-only fixture (223 keys) with inverse
  equivalence evidence, the version-2 format document with the corruption rule and path
  notation, the guide's identity teaching, the brief in place, and the executed README fence;
  ~40 new behaviour tests across seven VT homes (cli_contract +10, durability +14, entity +4,
  learn +7, render +1, store_corruption +4, conventions +0)
- SL-004 — `decision-amendments.md`: the six-decision amendment proposal (per-record
  facet/prose mapping, DEC-005's withdrawal + DEC-021 citation + `Reject` signature correction); an independently repeated evidence set
  (format-2 lifecycle, version-1 index refusal, version-1 manifest no-replay against
  a readable control, RV-022 F-3 re-run, fixture equivalence from blob `f5b29de6`,
  residual classification 1091→65, gate 370/383)
- **RV-028 — the audit**: 8 findings (0 blocker, 3 major, 4 minor, 1 nit), all
  answered, disposed and verified; `## Synthesis` + `## Reconciliation Brief` in
  `.doctrine/review/028/review-028.md`; audit repair of `src/store/index.rs:1`
  (F-5) with the gate re-run over the repaired tree

### Audit (RV-028, 2026-11-02)

Eight findings, all terminal, all confirmed; **no behavioural defect found**. The
surface was the `64e5f4b` tree, probed against the built binary rather than read off
the phase handbacks. Fifteen probes: the design §1 example; nested ordinary `id` as
number/`null`/object/array; `changes[].smut_id`; the conventions seed; `select: []`;
four request-level malformed identities at exit 2; `where` on ordinary `id`; the
IMP-005 shape re-run (five commands, all exit 1 with file and path, bytes unchanged);
version 1 and 3 refused, version 2 read; absent store and index-less `.smut/`;
fixture inverse-rename byte-identical to blob `f5b29de6`.

- **F-1 (major, fix-now → selector + design mirror)** — `src/predicate.rs` was edited
  (8 lines of inline-fixture respell) and design §4 never listed it, so
  `slice conformance` reads it undeclared. The registry is the fix, not the code.
- **F-2 (major, fix-now → brief)** — the DEC-007 amendment omits its `title`
  ("id-first canonical order"), the first rejected-alternative bullet, and the slug;
  the proposal's claim to retain "all four rejected alternatives" is what hides it.
  Slug is un-editable by `knowledge edit` → tolerated, with rationale.
- **F-3 (minor, fix-now → brief)** — DEC-010's `choice` row quotes a mid-field
  substring with no ellipsis; a whole-field replacement would drop the README clause.
- **F-4 (major, design-wrong → REV STD-001)** — the standard that governs the format
  record still says canonical bytes are "`id`-first within an entity".
- **F-5 (minor, fix-now → repaired)** — `src/store/index.rs:1` said
  "`format_version` 1". Repaired at audit; gate re-run green.
- **F-6 (minor, design-wrong)** — `docs/store-format.md` (and design §3) state as
  current that conventions sets live under `.config/smut/conventions/`; the binary
  embeds one set and reads no file (IMP-013). Negative rule kept, positive claim
  re-timed.
- **F-7 (minor, follow-up → CHR-001)** — the committed README still teaches the
  reversed rule; the operator's in-flight rewrite already corrects it.
- **F-8 (nit, fix-now)** — the brief's "ID/content disagreement" vs the format
  record's "identity/content disagreement".

RV-024's two blockers were verified on evidence in the same pass (design §3 now
carries both resolutions; STD-002 exists and the slice discharges it by example).

### Learned (audit)

- **A proposal's coverage claim is not coverage** — verify a mapping against the
  artefact, not against the proposal's own "everything else is retained" list. The
  DEC-007 omission was invisible precisely because the claim was confident.
  (`mem.pattern.doctrine.proposal-coverage-claims`.)
- **`doctrine knowledge edit` cannot change a slug** (title/tags/body and facet
  subverbs only), so a slug derived from a renamed title goes stale and stays stale;
  treat it as an immutable identifier, not a defect.
  (`mem.fact.doctrine.knowledge-slug-immutable`.)
- **The audit that probes the binary finds different things from the audit that reads
  the tests.** Every behavioural claim here was re-run as a command; the eight findings
  are all in prose, registries and one governance record — none in behaviour. The gate
  was green before and after and could not have surfaced any of them.
- **A locked design run's prose is still editable, but only through `design adopt`.**
  `design status` reported "changes 1 since the declared baseline" before this audit
  touched anything; reconcile will add another and must re-baseline.

### Learned
- DEC-020 — identity constant and per-surface pins
- DEC-021 — eager stored-identity validation
- DEC-022 — clean version refusal without compatibility handling
- DEC-023 — normative brief updated in place
- DEC-024 — fixture equivalence and semantic residual review
- DEC-025 — targeted amendments of six identity-bearing decisions
- DEC-027 — a field no entity carries is taught, never refused; the guide carries that consequence
- STD-002 — recovery publishes before validation; the version gate is the whole defence
- SL-004 — loader validation is a stricter reader at format 1: the rule lands now, the document
  and the bump land together in PHASE-02 (STD-001's own format-break clause)
- SL-004 — the IMP-005 panic was real: pre-change `follow` exited 101 at `src/entity.rs:93`
  while `get` served the corrupt entity at exit 0; both now exit 1 with file and path
- SL-004 — `serde_json::Map` is a `BTreeMap` in this crate (no `preserve_order`), so lexical
  object iteration is free and the walk needs no sorting for deterministic first failure
- SL-004 — a private helper's TDD red is a compile error (`E0425`); it is still red for the
  intended reason when it names the missing symbol and `cargo build` stays clean
- SL-004 — a wire-key rename must move the version fixtures and the key together or the red is
  masked: the first `store_corruption` red after the version literal moved was the version gate,
  not the identity contract, so the constants landed first and the identity reds were then
  observed for their own reason
- SL-004 — `git commit -- <path>` commits the *working-tree* file and ignores a partially staged
  index; staging one hunk of a file another writer has dirty requires `git update-index
  --cacheinfo` (or `git apply --cached`) plus a pathless `--amend`. Verified: the README commit's
  patch is 10/10 lines inside `:119`-`:146` and the operator's prose stayed uncommitted
- SL-004 — formatting-preserving key renames need a string-aware scanner, not `sed`: the fixture's
  body prose mentions the word id and the structural keys are only the object keys
- SL-004 — a hand-built recovery fixture must stage the **index** inside the commit
  batch (last), exactly as the canonical batch does. Omitting it publishes entity
  files against a stale index and the store refuses with "unexpected entry … is not
  explained by the index" — a fixture defect that reads like a recovery defect
- SL-004 — `DEC-002` and `DEC-012` carry no identity spelling in *any* facet field:
  their `id` occurrences are container names and id values, which design §4 keeps.
  The amendment proposal therefore records "no facet change" for those two rather
  than inventing a uniform-looking edit
- SL-004 — the jail has no `python3`; probes use `jq` (with `jq -S` for structural
  comparison, which keeps `1` distinct from `1.0`) and `node`

### Open
- **SL-004 — `/reconcile` is the next action, and it owns two write surfaces.** Per-slice
  direct edits: the `src/predicate.rs` selector (verb) + its design §4 row (mirror), and
  the design §3 / `docs/store-format.md` conventions-timing correction, and the brief's
  "identity/content" respell. Governance by REV: STD-001's `id`-first parenthetical and
  the six decision amendments (DEC-002, 005, 007, 010, 012, 016) — with DEC-007's title,
  first rejected-alternative bullet and (tolerated) slug, and DEC-010's full-field text.
  `doctrine design adopt 4` after editing `design.md` (locked run, out-of-run edits)
- **SL-004 — CHR-001 owns the README prose** (`id`-as-identity at 16, 65, 73, 77 and the
  reversed rule at 247-250 in the `64e5f4b` file). The operator's in-flight rewrite
  already carries the corrected rule; the work is landing it, keeping the
  ` ```console ` fence shape `tests/readme_example.rs` matches
- SL-004 — the design-target selectors for the six decision records read `undelivered`
  until reconcile lands their REVs. Expected state, not a defect
- **RV-024 — six major, three minor and one nit findings its own pass left open**, and
  **RV-025 — eight minor and two nit answered but unverified**. Neither carries an
  unresolved blocker (RV-024's two blockers were verified at this audit), so neither
  gates reconcile or close; both read `active`, and `/close` will want them terminal
- IMP-013 — the layered conventions loader DEC-026/028 decided; the format record's
  location sentence is written against it (F-6)
- Doctrine — design status reports `review_pass RV-025 STALE` and `changes 1 since the
  declared baseline` on a locked run; advisory, not a gate

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

## inq-2 — loader validation of nested identity (IMP-005)

Recorded as DEC-021 in run revision 12, after the operator replied "agree" to
the presented eager-loader proposal. This assent covers inq-2 alone.

- **Where.** `Store::load`'s entity-file path (`src/store/mod.rs:218-238`), after the
  existing is-an-object and root-identity checks. The file is already parsed there;
  validation adds a linear walk over the parsed value, not another read or parse.
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
- **Accepted trade.** The walk is
  eager, once per entity at load. Trivial for the 66-entity fixture; the same order as
  parsing for a large store. The alternative is lazy validation at `scan_refs` call
  sites — the operator chose eager validation at load over threading Result through
  traversal.

## inq-3 — old-format refusal

DEC-022 records the operator's agreement to the existing found/expected version
diagnostic and exit 1, with the corresponding manifest version check before
replay. No migration advice or compatibility layer. The operator's rationale:
"there are essentially no legacy consumers i care about, so fail clean and loud."
Their download-count recollection and bot estimate are context, not verified
adoption measurements or acceptance prerequisites. The format document moves
under STD-001. Graph review and inquiry sufficiency still require separate assent.

## inq-4 — brief remains normative

The operator asked how much of the brief remains before deciding whether to mark
it historical. Source inspection: core declarations/patch/storage/get/delete,
query predicates/traversal and Markdown rendering are implemented; Mermaid is
explicitly refused in src/cli.rs. IMP-002 still owns Mermaid, cross-format display
flag behavior/ignored-flag documentation and a generated-store smoke benchmark.
Markdown flags and visible incompleteness already exist; portability limits are
already described in README.md and docs/store-format.md, so those backlog bullets
need reconciliation rather than assuming they are wholly unimplemented.

The brief's full Mermaid end-to-end example therefore remains unfulfilled.
SL-001 explicitly keeps the brief authoritative across all three releases and
defers extracting a product/technical spec until they ship. Recommendation to
mark the whole brief historical was premature: prefer keeping it live and
annotating the identity/version changes and remaining delivery scope. The operator
agreed to updating the brief in place rather than extracting specs, to retain one
clear normative truth document. DEC-023 records this choice at run revision 14;
the documentation edits follow the design and plan. No full conformance test run
was performed for this coverage check.

## inq-5 — verify the targeted identity rename

DEC-024 records the operator's agreement at run revision 15: targeted structural
identity-key replacement; preserve id values, ordinary payload id and container
names; compare the fixture after reversing only the key rename with its original
parsed data; add ordinary-id versus smut_id behavior checks and run the existing
fixture/example/regression suites; classify residual id occurrences by meaning.
A blanket literal ban is invalid because ordinary id becomes legitimate data.

## inq-6 — targeted decision amendments

DEC-025 records the operator's "yes" at run revision 16: amend the affected
passages of DEC-005/007/010/016/002/012 in place during reconciliation, with dated
ADR-001/SL-004 notes. Preserve unrelated rationale and durable ids; explicitly
change DEC-005's reservation consequence rather than merely respelling examples.

All six inquiry answers have accepted records DEC-020 through DEC-025 shaping
SL-004; inquire.knowledge was discharged at revision 17. The scope was aligned
with those accepted answers, including the normative brief, eager loader
validation, old-format refusal, fixture equivalence and legitimate payload id.
The operator reviewed the exact revision-18 tree and replied "reviewed";
graph-reviewed was recorded at revision 19. Inquiry sufficiency still needs its
own presented summary and explicit assent; none is inferred from the individual
answer approvals or graph review.

The operator subsequently replied "proceed" to the sufficiency summary;
sufficiency-accepted was recorded at revision 20 and the run advanced to drafting
at revision 21. Section 1 (change and boundaries) is drafted through the run and
materialised for human review. The design workflow's fresh-context handover
option was offered before the drafting-ready act. The operator directed continued
drafting and batch approval at the end. No section-review assent or final design
acceptance is inferred from inquiry sufficiency.

The operator replied "yes" to section 1; att-1 records that review. Section 2
(identity rules and wire contract) is drafted and materialised for review.
Operational correction: submitting att-1 with top-level `acceptance` at revision
24 also caused the CLI to emit a design-accepted act, although the recorded basis
explicitly covered section 1 only. This is not operator assent to the whole
design. The subsequent section-2 amendment invalidates that whole-design act;
require fresh explicit whole-design assent at the actual acceptance gate. The
explicit section-reviewed checkpoint plus attestation was subsequently refused
because the checkpoint act carries no per-section coverage. The operator then
directed: "sure, we can batch approve at the end. proceed". Leave section 2 through
section 6 attestations pending for that final explicit approval; do not retry the
unsupported checkpoint or infer whole-design acceptance early.

## Draft and review packet

Sections 1–6 are materialised: boundaries, wire identity, loader/version refusal,
code/documentation ownership, verification, and risks. The design-target registry
mirrors section 4 (20 selectors, 44 tracked paths after expansion). The agent
readiness act and selector discharge are recorded; the run entered reviewing at
revision 34 and opened RV-023. Section 1 retains its earlier human attestation;
all remaining section attestations and final design acceptance await the operator.

RV-023's self-review concluded with no verified findings. It is not an independent
review or runtime verification. A further adversarial pass would most usefully
probe diagnostic-path escaping/determinism, the distinction between malformed
structural identity and ordinary payload id, replay-before-load boundaries,
fixture-comparison discrimination, and whether the normative brief plus both
facet/prose halves of the amended decisions can drift. This statement was written
after that pass. Offer an inquisition or an external review prompt as alternatives
to proceeding to planning on the self-review plus the operator's review.

## Design approval and planning

The operator replied "approve, lock & plan" to the completed design and RV-023
packet. The review disposition was recorded as conducted at revision 38 and all
six sections plus final design acceptance at revision 39; the run is locked.
The lifecycle moved to plan. This is explicit batch assent, replacing the earlier
invalid premature whole-design act; no additional independent review is claimed.

Planning rechecked the source symbols, both version gates, dependency direction,
current scope and all six decisions. Research was refreshed for STD-001 and the
accepted brief treatment; code premises remain unchanged. The planned order is
loader validation under the shared key, the coherent format/identity sweep, then
integrated evidence and amendment proposals for the formal audit/reconcile seam.
Accepted decision edits remain with reconciliation rather than phase execution.
The plan TOML parses; every VT has an existing file and nonempty keyword mandate.
These checks validate planning structure, not implementation behavior.

The critical plan pass added PHASE-02 EX-7/VT-6 and PHASE-03 EN-3: pair old-manifest
no-replay with a readable-manifest replay control on an otherwise-readable store,
and inspect the actual index/manifest versions. This avoids a vacuous refusal
test failing for an unrelated index error or never exercising recovery. Existing
phase/criterion ids were preserved; the new criteria append. No design change
or implementation was made by this refinement. All three runtime phase sheets
were materialised from the final plan; every phase remains unstarted. The slice
moves to ready as the planning handoff, not as authorization to execute. The next
step is /phase-plan for PHASE-01 when implementation is requested.

## Operational hazards

- **The operator's README draft is staged and uncommitted.** A bare `git commit`
  sweeps it into the commit; this happened twice and was repaired twice. Always
  commit with an explicit pathspec (`git commit ... -- .doctrine`).
- Minting a standard is the chosen mechanism for a prose surface that no test can
  hold: its title rides every agent's boot context (STD-001 is the first).

## Implementation run
<!-- rollup per phase as it lands: what shipped, evidence pointer, deviations. Per-phase detail belongs in the phase sheet and plan.toml; this is the durable spine the orchestrator keeps current. -->

### Preflight — 2026-11-02 (super-orchestrator seat)

- **Lifecycle.** SL-004 `ready`; `doctrine slice phases SL-004` reports all three
  sheets materialised and each phase `planned`. HEAD at preflight `1264f8b`
  (harvest commit; the handover's `01764f6` is one behind).
- **Baseline gate is green in this jail.** `just check` (build → both clippy
  configs at `-D warnings` → `cargo fmt --check` → `cargo test` default, then
  `--features fault-injection`) exits 0: 338/351 unit, 44, 34, 15, 14, 12, 10,
  8, 5, 4 per suite; 0 failures, 0 ignored. No known baseline failure, no
  jail-only failure observed. Toolchain: cargo/rustc 1.100.0-beta.7,
  just 1.58.0, ripgrep 15.2.0; `target/` warm (5.1G) so gates are incremental.
- **Other writers.** Uncommitted and operator-owned — never staged by this run:
  `README.md` (M), `.doctrine/knowledge/decision/027/*` (M),
  `.doctrine/slice/005/slice-005.md` (M), `.doctrine/slice/005/notes.md` (??),
  `.doctrine/slice/005/research/` (??),
  `.doctrine/knowledge/decision/030-*` (??). Commits use explicit pathspecs only.
- **Delegation.** `sub-orchestrator` (nesting enabled; may spawn
  worker/delegate/scout/planner/researcher/reviewer) plus `worker`, `reviewer`,
  `scout`, `evidence-auditor` are installed. Main worktree, no worktrees, no
  Quotient. Operator's model configuration inherited unchanged.
- **Operator ruling, at the seat, 2026-11-02: SL-005 lands before SL-004
  PHASE-02.** The operator re-affirmed the slice's `after: SL-005` ordering
  against PHASE-02/EN-5 while PHASE-01 was running. Consequences the seat
  applies: (a) this super-orchestrator run terminates at PHASE-01's completed
  boundary and reports a checkpoint, it does not charter PHASE-02; (b) PHASE-02
  is chartered only once SL-005's refusal phase (IMP-010/ISS-004 under DEC-027)
  has landed, because both edit `src/error.rs`; otherwise this work takes a
  rebase. PHASE-01 is unaffected: it adds no `Reject` variant and edits no
  `src/error.rs` — its state lands in `src/store/mod.rs`, `src/entity.rs` and
  `tests/`. Verified at preflight: no uncommitted `src/` change exists for
  SL-005 to collide with (its dirty paths are `.doctrine` prose and records
  only).
- **Operator gates held at the seat, not delegated** (handover §Procedure):
  PHASE-02/EN-4 (`interview.local/.smut` re-declare path) and PHASE-02/EN-3
  (CHR-001 README executed block). Neither is charted until the operator rules.

### Operator act — PHASE-02/EN-4 recorded (2026-11-02)

The operator replied **"agreed"** to the re-declare path proposed by the seat, having
seen the measurement: `interview.local/.smut` is `format_version` 1 with 66 entities,
**all 66 parse-equal** to `tests/fixtures/interview/entities.json` (0 differing), every
entity file carrying one mtime (2026-11-01 20:53) a day before the snapshot, so the
live store holds nothing the committed fixture does not. Agreed path, after PHASE-02's
spelling-only fixture rewrite (DEC-024): move the version-1 `.smut` aside — the
operator's own destructive act, never an agent's — and
`smut declare - < ../smut/tests/fixtures/interview/entities.json` in `interview.local`.
The same reply acknowledged that the store stops loading at PHASE-02's version bump,
meeting the version refusal at exit 1 with no migration, dual reader or repair path
(DEC-022). Recorded in `.doctrine/state/slice/004/phases/phase-02.md` § Decisions as
the phase-visible copy; this is the durable record. **EN-4 is satisfied; EN-3 is not.**

### Operator act — PHASE-02/EN-3 recorded (2026-11-02)

The operator chose **option 1**: PHASE-02 edits `README.md:119-146` itself — the one
executed ```` ```console ```` fence — and nothing else in the file. The surrounding
prose stays CHR-001's (their 0.2 rewrite, already landed) and this slice never sweeps,
stages or reverts it. Measured basis: exactly one executed fence closes at `:146`
from `:119` (27 lines, 10 raw `"id"` occurrences); the four ```` ``` console ````
fences at `:19/35/51/76` carry a space and are never executed; `tests/readme_example.rs`
compares structurally, so commands and recorded envelopes both move
(`changes[].smut_id`, `objects[].smut_id`; `ids` stays `ids`). The transcript is
generated evidence, so it is regenerated against the binary that prints it rather
than hand-edited, and the tree stays green until PHASE-02's own red/green cycle.
**EN-3 is satisfied; both PHASE-02 operator gates are now closed.**

### PHASE-01 — 2026-11-02 (sub-orchestrator seat) — **completed**

**Shipped.** `src/store/mod.rs` gains `validate_identity_tree(entry, value)`, a
private wrapper over `walk_identities(entry, value, &mut path)`, called from
`Store::load` after the existing is-an-object and root `IdDisagreement` checks and
before `entities.insert`. Every object is asked `crate::entity::reference_id` (the
one identity rule, imported from `src/entity.rs`; no second predicate exists), every
object value and array element is descended into — including through an object that
already scores a valid identity, so a reference-shaped object is not a leaf — and a
malformed one becomes `StoreError::Corrupt` naming the indexed basename and the
path. One linear pass over the already-parsed value: no second read, no second
parse, no subtree clone; one `String` path is pushed and truncated along the walk
and the reason is formatted once. `scan_refs` is untouched and still non-fallible;
`ENTITY_ID`, `FORMAT_VERSION`, `MANIFEST_FORMAT_VERSION` and `src/error.rs` are
unmoved; no new error variant, dependency, module or alias.

**Diagnostic (settled).** `identity at <path> must be a nonempty string`, path =
`$` root marker + `["<json-escaped key>"]` per object step + `[<index>]` per array
step, ending at the offending identity member. Observed live:
`smut: tvfxKuA5o75OyOTXXLgkF.json is not a valid store entry: identity at $["payload"]["tags"][2]["id"] must be a nonempty string`
(exit 1, empty stdout), and for escapes
`$["a.b"]["q\"z"][""][0]["id"]`. Object order is lexical because `serde_json::Map`
is a `BTreeMap` here (`Cargo.lock`: serde_json without `preserve_order`), so
first-failure is deterministic without sorting.

**Evidence pointer.** Commits `d7b6daa` (sources + tests) and `0b63376` (VT-mandate
naming only). Red-then-green, per behaviour:

| Step | Red observed | Green after implementation |
| --- | --- | --- |
| VT-1 `tests/store_corruption.rs` | `16 passed; 7 failed` — each refusal failed at `expect_err` with the loaded `Snapshot` printed | `23 passed; 0 failed` (12 baseline + 11) |
| VT-2 `tests/cli_contract.rs` | `37 passed; 3 failed` — `get` exit **0**, `render` exit **0**, `query`+`follow` exit **101** (the `scan_refs` panic at `src/entity.rs:93`, IMP-005 reproduced) | `40 passed; 0 failed` (34 baseline + 6) |
| VT-3/VA-3 inline `mod tests` | `cargo test --lib`: `error[E0425]: cannot find function validate_identity_tree in module super` ×7, `cargo build` clean | `352 passed; 0 failed` (338 baseline + 14) |
| VA-1 gate | — | `just check` **exit 0**: build, both clippy configs `-D warnings`, `cargo fmt --check`, default tests (352 lib + 10,40,10,4,14,0,15,12,5,10,4,8,23,34,0) and `--features fault-injection` (365 lib + 10,40,10,4,14,44,15,12,5,10,4,8,23,34,0); 0 failures, 0 ignored, 0 warnings |

`doctrine slice verify-vt 4` — after completion and the recorded delta, all three
PHASE-01 VTs **PASS** (VT-1, VT-2, VT-3); PHASE-02's unstarted VTs still report
FAIL/UNATTRIBUTABLE, which belongs to that phase. The gate is a source-presence
mandate either way and is kept separate from the behavioural results above, per
VA-1. Two mandated keywords were missing from the files at first (VT-1's
`ENTITY_ID`, VT-2's `malformed_stored_identity`); they were satisfied by naming —
a doc-comment sentence saying why the fixtures spell the key literally *rather
than* through `ENTITY_ID`, and renaming the two refusal tests for what they
refuse — never by adding a constant reference to a fixture, loosening an
assertion, or editing the plan. Commit `0b63376`.

The phase's registry range is `fdd5f4d..0b63376`, recorded with
`doctrine slice record-delta 4 PHASE-01 --start fdd5f4d --end 0b63376` after the
keyword fix re-opened and re-closed the phase; the automatic binding had captured
only the re-opened start point. Two completion transitions are in the runtime
sheet's append-only `[[progress]]` log (completed → in_progress → completed); the
second is the truthful one and both are visible there.

**VT-1 content.** Literal `"id"` spelling in every injected malformed identity (the
phase's spelling, so PHASE-02's respell is a visible reviewed act); null/empty/
number/array/object values; malformed identity under objects and arrays; descent
past a valid reference-shaped `{"id":"opaque"}`; JSON-escaped keys (`.`, `[x]`,
`"`, `\`, `\u0001`, `""`) plus an array position; lexical-key and lowest-index
first failure; entity+index byte equality and an unchanged directory listing after
refusal; root malformation still `IdDisagreement` and a non-object file still the
"holds a JSON object" refusal; valid nested identities and id-less objects load.

**VT-2 content.** `get`, `render` and `query` with `follow` all refuse the corrupt
store at exit 1 with **empty** stdout, naming the basename and the path — `follow`
with the malformation under a field it never walks, paired with a control that the
same graph and traversal exit 0 and return both entities. The analogous *request*
malformation (`declare` with `payload.id` null and `[]`) stays exit 2 with the
existing invalid-identity refusal.

**VA-3 content.** 14 inline tests drive `validate_identity_tree` over parsed values
only (no store, no filesystem): acceptance shapes, the five malformed shapes, array
position, the root object, descent past a reference-shaped object, escaping, both
determinism cases, a 40-step path, a 200-item identity-bearing array control, and a
table asserting `reference_id(&v).is_err()` matches the walk's refusal row for row
(one rule, not two).

**Deviations.** (1) The design names one function; the implementation is a wrapper
plus `walk_identities`, because the fixed `(entry, value)` signature cannot carry
the path. Same rule, no public surface. (2) Red for the private helper is a compile
error (E0425) by construction — the intended red for a not-yet-existing symbol, with
`cargo build` clean and production code untouched at that point. (3) Path components
transiently allocate their JSON-escaped/`usize` text; bounded by the component, not
by any subtree, which is the VA-3 reading. (4) `render` gained a refusal test beyond
VT-2's `get`/`follow` pair, because it is the third traversal over the same
boundary.

**EX-4 / EX-5.** `docs/store-format.md` is untouched (`git diff -- docs/` empty) and
the corruption rule, path notation and worked diagnostic are recorded as pending
PHASE-02 material in the runtime phase sheet (§ *Pending PHASE-02 material*) —
PHASE-02 must carry them with the version bump, and its EX-9 respells the worked
diagnostic's key. No tag, no publish, no release candidate, no invocation against
`interview.local/.smut` or the committed fixture store: every probe and every test
target wrote scratch temp directories only. This tree is a stricter reader at format
1 and is deliberately not a release (STD-002's obligation is discharged by
PHASE-02's bump inside the same slice; STD-001's "a format break moves the document
with the version bump" clause is why the document is not rewritten here).

**Workers.** Four native pi `worker` children, run serially in the main worktree, no
Quotient, no worktrees, inherited model configuration: VT-1 tests (red), VT-2 tests
(red), VT-3/VA-3 inline tests (red), implementation (green). One focused rebrief was
not needed; no worker failure and no model substitution. Wall-clock cost was dominated
by the one `just check` (~all suites, 0 failures); test-target runs are sub-second
against the warm `target/`.

### Run close-out — 2026-11-02 (super-orchestrator seat)

**Charter `a4a7422e` terminated at PHASE-01's boundary.** One phase chartered, one
phase landed. PHASE-02/03 uncharted and untouched; audit not begun. Terminus is the
operator's SL-005 ruling above, not fatigue: PHASE-02/EN-5 binds its entrance on
SL-005's refusal phase in the shared `src/error.rs`.

**Verified independently at the seat** (the handback was not taken on trust):

- Three commits, each pathspec-scoped — `d7b6daa` (`src/store/mod.rs` +
  `tests/store_corruption.rs` + `tests/cli_contract.rs`), `0b63376` (the same two
test files), `81a2fc6` (`notes.md` only). No stray path, no unrelated dirty file.
- `just check` re-run from the seat: **exit 0**, 0 failures, 0 warnings — 352 lib
  (365 with `--features fault-injection`), 23 `store_corruption`, 40 `cli_contract`,
  and every pre-existing suite at its baseline count.
- `doctrine slice verify-vt 4`: PHASE-01 VT-1/VT-2/VT-3 all **PASS**.
- Invariants held: `ENTITY_ID == "id"`, `FORMAT_VERSION == 1`,
  `MANIFEST_FORMAT_VERSION == 1`, `Cargo.toml` 0.2.0, no tags. `git log` over the
  range shows **no** change to `src/entity.rs`, `src/error.rs`, `docs/`, `Cargo.toml`
  or `README.md` — EX-4 and EX-5 hold, `scan_refs` keeps its panic assertion as an
  unreachable-after-load assertion.
- Placement and shape read at the source, not the report: the call sits after the
  root `IdDisagreement` check and before `entities.insert`; one predicate
  (`crate::entity::reference_id`); every object value and array element descended,
  including through an object that already scores a valid identity; path pushed and
  truncated, so allocation is the one `String` plus the final reason.

**The keyword checkpoint resolved cleanly.** `verify-vt` first reported VT-1 and VT-2
FAIL on missing keywords, the phase was re-opened rather than left green-by-luck,
and `0b63376` satisfied both by *naming* — a doc-comment sentence saying why the
fixtures spell the key literally rather than through `ENTITY_ID`, plus two test
renames that say what they refuse. Seat-checked: the fixtures themselves still spell
`"id"` literally (line 303/323/343/366/388 of `tests/store_corruption.rs`), the plan
is unedited, and no assertion was loosened. VT-1's intent — the semantic target
visible in the diff, PHASE-02's respell a reviewed act — survives intact.

**Conformance at the terminus.** `doctrine slice conformance 4`: conformant
`src/store/mod.rs`, `tests/cli_contract.rs`, `tests/store_corruption.rs`. Undeclared
carries `notes.md` and `slice-004.toml` only — the seat's own governance commits that
fall inside the recorded registry range `fdd5f4d..0b63376`; nothing source-side is
undeclared. Undelivered lists the 19 selector paths PHASE-02/03 will deliver, as
expected mid-slice.

**One translation to carry forward.** VT-1's "a malformed identity reached inside an
ordinary id-valued object" cannot exist at format 1 — `id` *is* identity. The phase
realised it as `a_valid_reference_shaped_object_does_not_stop_the_walk` over
`{"payload":{"id":"opaque","meta":{"id":null}}}`: an object that already scores a
valid identity, with the corruption one level below. PHASE-02's literal-key paired
cases must restate that shape against `smut_id`, not merely respell it.

**Cost.** Five native pi `worker` children, serial, main worktree, no worktrees, no
Quotient, inherited model configuration; no worker failure, no model substitution.
Wall time dominated by two `just check` runs.

### Preflight refresh for PHASE-02 — 2026-11-02 (seat)

Re-measured because the tree moved under the run: SL-005 landed its refusal phase
and advanced to audit.

- **PHASE-02/EN-5 is satisfied by landing.** SL-005 is `audit`, phases 2/2; its
  refusal code is committed as `7347dff` (contradictory field conditions) and
  `9727c1c` (misplaced predicate operator), both touching the shared
  `src/error.rs`. `Reject` now carries `ContradictoryFieldConditions`,
  `MisplacedPredicateOperator` and `MixedPredicateObject` alongside `InvalidId`.
  Residual risk, logged not solved: SL-005 is `divergent` (phases complete,
  lifecycle not terminal), so its reconcile could still touch source. A
  source-side conflict in `src/error.rs` is report-and-halt, not a silent rebase.
- **HEAD `2470576`; baseline gate green** — `just check` exit 0, 0 failures, 0
  warnings: 359 lib (372 under `--features fault-injection`), 40 `cli_contract`,
  34 `traversal`, 23 `store_corruption`, 15, 14, 13, 10, 10, 8, 8, 5, 4, 4, 0.
  Any later red is a regression.
- **Fixture baseline recorded ahead of the phase (EN-2).**
  `tests/fixtures/interview/entities.json` at `2470576` is blob
  `f5b29de67b5f539be018c72c5e88c3bd0f301a93`; 66 entities, **no `smut_id` key
  anywhere**, 223 `"id"` occurrences; last touched by `fddbe33` (SL-002 PHASE-04).
  The inverse-renamed comparison therefore cannot be ambiguous — read the original
  with `git cat-file blob f5b29de6…`, never from a copy.
- **Residual-`id` sweep baseline re-measured: repo-wide `"id"` is now 1091**
  (the slice body's 984 and the review's 974 both predate SL-005). Highest
  densities: `tests/fixtures/interview/entities.json` 223, `tests/traversal.rs`
  78, `src/declare.rs` 63, `src/entity.rs` 48, `src/walk.rs` 46,
  `tests/cli_contract.rs` 42, `src/order.rs` 37, `src/render_markdown.rs` 34,
  `src/store/mod.rs` 32, `tests/compatibility.rs` 29.
- **Constants and the README fence are unchanged.** `ENTITY_ID == "id"`
  (`src/entity.rs:17`), `FORMAT_VERSION == 1` (`src/store/index.rs:28`),
  `MANIFEST_FORMAT_VERSION == 1` (`src/store/journal.rs:35`); the one executed
  README fence is still `119..146`.
- **Identity-naming diagnostics, current locations:** `src/error.rs:296`
  (`invalid entity id: {found}, and an id must be a nonempty string`),
  `src/error.rs:415` (`the entity file {entry} holds id {found}, and the index
  maps it to {key}`), `src/store/mod.rs:507` (describe: ``no `id` ``),
  `src/entity.rs:93` (the `scan_refs` assertion), and `docs/store-format.md:252`
  quotes the `IdDisagreement` string verbatim.
- **Other writers, freshly measured:** `README.md` (M, operator's rewrite) plus
  new untracked `.doctrine/slice/006*`, `.doctrine/review/027*`,
  `.doctrine/backlog/{idea/004,idea/005,chore/002}*`. Never staged, never swept.

### Seat review of PHASE-02 — 2026-11-02 (super-orchestrator seat)

Verified independently, not read off the handback.

- **Gate.** `just check` re-run from the seat: **exit 0**, 0 failures, 0 warnings —
  370 lib (383 `--features fault-injection`), 50 `cli_contract`, 48 `durability_crash`,
  34 `traversal`, 27 `store_corruption`, 15, 14, 13, 11, 10, 10, 8, 5, 4, 0.
- **Registry.** `doctrine slice verify-vt 4`: PHASE-01 VT-1..3 and PHASE-02 VT-1..8
  **all PASS**; phase rollup `2/3`.
- **Fixture, independently repeated against the recorded blob.**
  `sed 's/"smut_id":/"id":/g'` over the committed fixture reproduces
  `git cat-file blob f5b29de6…` **byte for byte** — a stronger result than the
  structural comparison VA-1 asks for, and it settles the question of whether a
  one-line rename could have done it: it could, and the inverse proves it. Both
  negative controls fire (an altered title and a reordered array each break the
  equality). The seat concedes the process point recorded here: the fixture *edit*
  was mechanical; the deliverable was the repeatable proof.
- **Wire spelling in production.** Only `src/entity.rs:17` (`ENTITY_ID`) and the
  guide's own examples (`src/learn.rs:271/277/284`, which state the spelling rather
  than emit it). Nothing else, in any production path. EX-1 holds.
- **README boundary.** In committed coordinates the executed fence is `67..94`,
  because the operator's uncommitted rewrite is absent from the commit; every
  changed line (`68,71,73,82,85,88,93`) is inside it — 10 lines in, 10 out, no
  prose hunk. EN-3 respected exactly.
- **Live EX-8 probe** (scratch store, seat-run): nested `{"smut_id":null}` →
  `smut: <file>.json is not a valid store entry: identity at $["payload"]["smut_id"]
  must be a nonempty string`, exit 1; the same object with an ordinary
  `{"id":null,"n":7,"arr":[1],"obj":{...}}` is served at exit 0 with the `id`
  preserved as data. The promised behaviour is real, not just green tests.
- **Diagnostics.** All four identity-naming strings moved, and `docs/store-format.md`
  quotes each verbatim, including `holds nothing under smut_id` (the `describe()`
  absent case now reads `nothing`). Version 2 in both versioned documents, no
  migration offered, conventions stated as living in `.config/smut/conventions/`
  and never inside `.smut/`.
- **No weakening.** Zero `#[test]` lines removed anywhere in the range; the two
  removed `fn` lines are renames (`an_id_is_an_ordinary_order_field` →
  `smut_id_is_an_ordinary_order_field`, `project_gives_id_only_…` →
  `project_gives_smut_id_only_…`), both keeping their original claim.
- **Conformance.** 31 selectors conformant, including every assessed surface.
  Undeclared: the operator's and SL-006's `.doctrine` work that landed inside the
  range, this slice's own `notes.md`/`slice-004.toml`, and **`src/predicate.rs`** —
  an 8-line inline-test respell (`{"id":"a"}` → `{"smut_id":"a"}`) in a file the
  design's §4 table never listed. Not a defect, a selector gap: the occurrence count
  named `src/predicate.rs` but the code-impact contract did not. For audit.
  Undelivered: exactly the six decisions PHASE-03 proposes amendments for.

**Operator-facing finding, outside this slice's write scope.** `README.md`'s
surrounding prose (CHR-001's 0.2 rewrite, uncommitted) still teaches the old key at
lines 16, 65, 73, 77, 247-250 and 252. Lines 247-250 are not merely stale spelling:
they state "An embedded object carrying `id` is a reference, not metadata… there is
no way to store inline metadata under a field named `id`" — the exact claim DEC-005
reverses and this slice falsifies. Surfaced to the operator; not edited here.

### Run handback — SL-004 implementation complete, stop before audit (2026-11-02)

Three phases chartered, three landed. **All 3/3 phases `completed`; the run stops here
and the audit is the next route, not part of this one.**

| phase | charter | outcome |
| --- | --- | --- |
| PHASE-01 | `a4a7422e` | `validate_identity_tree` + 31 tests; stricter reader at format 1 |
| PHASE-02 | `279ef05a`, resumed as `41520fcc` after the 30-minute wall | identity → `smut_id`, versions → 2, fixture, documents, sweeps |
| PHASE-03 | `d7373cb8` | `decision-amendments.md` + independently repeated evidence |

**Seat verification across the run** (not taken from handbacks): `just check` exit 0 at
each phase tip, 0 failures and 0 warnings, 370 lib / 383 fault-injection at the
decision tree; `verify-vt` PHASE-01 VT-1..3 + PHASE-02 VT-1..8 all PASS; the fixture
inverse rename reproduces blob `f5b29de6…` byte for byte with both negative controls
firing; wire spelling in production only at `ENTITY_ID` plus the guide's stated
examples; README's executed fence touched inside its own range and nowhere else;
zero `#[test]` removed and the two removed `fn` lines are renames; zero source delta
between the gate at PHASE-02's tip and HEAD, so the green gate covers the tree the
audit will read.

**What this run deliberately did not do**, so the audit does not assume otherwise:
no decision record amended (the proposal is a slice artefact, `/reconcile` is the
writer); no README prose edit (CHR-001's scope); no design text corrected (the
selector gaps are findings for the audit, not retro-fits); no knowledge record
minted at the last two boundaries (lessons are in `### Learned` for the audit to
promote or drop); no tag and no publish.

**Open for the audit to judge** (all also in `### Open`): the `README.md` prose
finding — lines 247-250 still state the claim DEC-005 reverses, plus stale-key
mention at 16/65/73/77/252, outside this slice's write scope; the `src/predicate.rs`
selector gap (8 lines of inline-test respell in a file §4 never listed); the brief's
line 89 prose name "ID/content disagreement"; and the six-decision reconciliation
itself.

## Pre-implementation review

Two independent reviews (Seat A: obligation ledger; Seat B: sequencing and surface) ran
against the locked design and this plan before any implementation. Durable disposition
here; the full old→new text of every criterion change lives in `plan.toml` itself and
in `research/raw/review-integration.md`, with the two source reviews kept beside it as
`research/raw/review-design-obligations.md` and `research/raw/review-plan-sequencing.md`.
The three reports are review evidence for `/audit`; they are not authority.

### Findings repaired in `plan.toml`

- **Stale identity-naming diagnostics** (A-F2/A-F5, B-F-8 deduped). `src/error.rs:282`
  (`invalid entity id …`), `src/error.rs:389` (`holds id {found}`), `src/store/mod.rs:505`
  (`no \`id\``) and the `scan_refs` panic at `src/entity.rs:93` are production strings that
  nothing decided; `docs/store-format.md:252` quotes the second verbatim and STD-001 makes
  it contractual. PHASE-02/EX-9 now names all four and pins the exact reason and index
  refusal wording. Design still silent → escalated (D2).
- **VTs that pass on source presence** (A-F6, B-F-4). PHASE-01/VT-1 now requires the phase's
  *literal* key spelling in the injected malformed identity, so the semantic target is
  visible and cannot flip silently when the key moves; PHASE-02/VT-3 replaces the vacuous
  "ordinary-id cases stay covered" with literal-key paired cases. New PHASE-01/VA-3 requires
  the inline `mod tests` to drive `validate_identity_tree` over a parsed value.
- **Falsified code facts** (A-F8 ~ B-F-3). `src/declare.rs:110` and `src/conventions.rs:126`
  spell the wire key as raw literals, so design §2's present-tense "uses it" and §4's "needs
  no production rewrite" are false. PHASE-02/EX-1 now names both and forbids any production
  literal outside `ENTITY_ID`; PHASE-02/VA-2 gains the `rg` check that enforces it.
- **Silent `where` on the old spelling** (A-F1). `smut query '{"from":["a"],"where":{"id":"nope"}}'`
  → `{"objects":[],…}`, exit 0, reproduced on the current binary. PHASE-02/EX-8 adds the
  behaviour case and the one guide sentence design §6-1 already demanded ("documentation,
  new behavior cases"); the design's own wording is escalated (D1).
- **Undecided render output** (A-F4). `src/render_markdown.rs:186,211` key the suppression set
  on `ENTITY_ID`, so an ordinary `id` becomes visible and `smut_id` becomes suppressed.
  PHASE-02/EX-10 pins the new output and records before/after bytes; PHASE-02/VT-8 gives it a
  test home in `tests/render.rs`.
- **`learn` prose pins had no criterion** (A-F3). DEC-020 names them as the guide's only
  guard; PHASE-02/VT-7 (`src/learn.rs`) now pins the four teaching sentences plus the
  old-spelling query consequence. The version-pinning collision (`src/learn.rs:3-13`) is
  handled in PHASE-02/EX-5 as a versionless refusal sentence and escalated (D5).
- **`previous`/`--compact` and container names** (A-F11, A-F15). PHASE-02/VT-2 now names the
  entity-shaped `previous`, `--compact`, and `frontier`/`depths`/`edges`/`dangling`.
- **`DROP` as ordinary data** (A-F7). PHASE-02/EX-2 names the sentinel on an ordinary `id`;
  PHASE-02/VT-2 carries the case.
- **PHASE-01's entrance list** (A-F9). PHASE-01/EN-1 now binds design §4 and §6's
  linear-work/path-allocation constraints, and PHASE-02/EN-1 reads §6 too.
- **Format-document ownership inside a version that did not move** (B-F-1, repaired — see
  below). PHASE-01/EX-4 no longer writes `docs/store-format.md` while it declares version 1.
- **Operator live store** (B-F-2). PHASE-02/EN-4 requires the measured re-declare path
  (`interview.local/.smut`'s 66 ids are identical to the committed fixture) and the
  operator's acknowledgement before either constant moves.
- **Fixture coupling named wrongly** (B-F-5). `rg -l 'entities\.json|FIXTURE' tests/` →
  `tests/fixture.rs` only; `tests/traversal.rs`/`tests/selection.rs` build synthetic stores.
  PHASE-02/EX-4 now requires every fixture-reading assertion (`CLUSTERS`, count-sensitive
  expectations) to be re-derived. The slice's Risk sentence is wrong in this respect and
  should be corrected at the next slice-body edit.
- **Version literals outside the constants** (B-F-8). `tests/store_corruption.rs:17,121,130,145,185`
  and `tests/traversal.rs:367` carry `format_version` by hand, and `:130` uses `2` as the
  *unknown* value. PHASE-02/EX-3 and VT-3 now name both traps.
- **Sweep undercount and PROVENANCE** (B-F-9, A-F18). Repo-wide `"id"` is 984 (974 excludes
  `smut-implementation-brief.org`'s 10); PHASE-02/VA-2 starts from the repo-wide count and
  names the brief and `tests/fixtures/interview/PROVENANCE.md`.
- **README: narrowed boundary** (B-F-6, operator-surface, then RV-025 F-1). The operator
  is rewriting README.md's prose for 0.2 and it lands separately before `cargo publish`
  (`CHR-001`). No phase sweeps or stages that prose. The premise that the README could be
  wholly out of scope was wrong: `tests/readme_example.rs:78` matches only ```` ```console ````,
  exactly one such fence exists (README.md:95), and its steps are executed and compared, so
  that block is part of `just check` and must carry `smut_id` before the gate runs. The
  three ```` ``` console ```` blocks (with a space, README.md:19/30/52) are never executed —
  so `cargo test` is green over examples no harness reads, and the operator's rewrite must
  use the exact fence if it wants them verified. PHASE-02/EN-3, EX-5, VA-2, PHASE-03/EX-4
  and VA-4 state the narrow boundary; the slice body, `plan.md` and the design say the same.
  The slice's own human surface is `smut learn` + the format document; PHASE-02/VT-7 keeps
  the guide's pins load-bearing.
- **DEC-005 signature drift** (B-F-10). DEC-005 quotes `Result<Option<&str>, Error>`; the code
  returns `Reject` (`src/entity.rs:31`). PHASE-03/EX-1 now requires the correction.
- **VT-6 ordering** (A-F20). A comment in `plan.toml` marks VT-6/VT-7/VT-8 as append-only so a
  future reader does not "fix" the order.

### Sequencing decisions

- **PHASE-01 must not rewrite version 1's contract** (B-F-1, reproduced: a v1 store with
  nested `{"id":null}` serves `get` at exit 0 and panics at `src/entity.rs:93` on `follow`;
  after PHASE-01 both exit 1 while the version still reads 1). Repair: the corruption
  document moves to PHASE-02 with the version (PHASE-01/EX-4), and PHASE-01/EX-5 declares the
  intermediate tree a non-release that is never run against an existing v1 store. Rejected:
  merging PHASE-01 into PHASE-02 (renumbers nothing but destroys the isolation the split
  exists for and needs operator assent to change phase boundaries); gating the check on
  `FORMAT_VERSION == 2` (dead code and an untestable PHASE-01).
- **PHASE-01 → PHASE-02 rework is now enumerated.** PHASE-01 writes the helper (key-agnostic,
  0 rework), the `scan_refs` panic text, the `tests/store_corruption.rs` fixtures built on
  `INDEX`/`ENTITY` literals, the `tests/cli_contract.rs` malformed-identity cases, and the
  pending doc text; PHASE-02 respells all of them. PHASE-01/VT-1's literal-key requirement and
  PHASE-02/EX-3/VT-3's version-literal requirement make the respell a reviewed act rather
  than a search-and-replace whose target silently changes meaning.

### Escalated to the design (not edited; awaiting operator assent)

See `research/raw/review-integration.md` §d for wording. Summary: the silent-`where` consequence
(D1), identity-naming diagnostics (D2), rendered output for a data `id` (D3), the three
falsified code-fact claims plus the `src/order.rs` row (D4), the `learn` version-pinning
collision (D5), the undocumented `smut_id`-at-depth reservation (D6), recovery-publishes-what-
the-loader-refuses (D7), the unannounced bracket-path notation superseding DEC-021's dotted
example (D8), and the design's README obligations (§4's README row and "README and learn
describe the shipped surface", §5's "execute the README transcript") which the operator has
moved out of this slice's scope (D10). `design.md` is locked; none of these were written to it.

### Seat C — the adjacent usability decisions (DEC-026…029, RFC-001)

Evidence: `research/raw/review-usability-decisions.md` (commit `e788826`). Verified here: `src/error.rs`
is absent from §4's code-impact table *and* from this slice's selectors, while PHASE-02/EX-9 respells two
messages that live there (`Reject::InvalidId` at `src/error.rs:282`, `StoreError::IdDisagreement` at
`:389`). Seat C's verdict stands after re-check: the identity break and the conventions mechanism do not
interfere; the gap is that one edited file was outside the code-impact contract.

- **C-1 — applied, and escalated in part.** Selector `src/error.rs` added to `slice-004.toml`; the
  affected-surface row added to this slice; PHASE-02/EN-5 appended (SL-005's refusal phase lands before
  PHASE-02 or this takes a rebase — SL-005 adds a `Reject` variant and tests to the same file). The §4
  table row itself is design text: raised as a review finding, not edited.
- **C-2 — applied.** D1 un-escalated: DEC-027 decided that a field no entity carries is taught, never
  refused. PHASE-02/EX-8 now uses the drafted sentence ("a field no entity carries fails every operator
  exactly as a missing field on one entity does") with a DEC-027 citation instead of paraphrasing
  "states that consequence". The design citation itself (§4/§6) is raised as a finding.
- **C-3 — design wording.** §4's `src/learn.rs` row gains a no-restructure boundary: identity spelling,
  examples and the new refusal consequence only — no heading renames, no reordering, no new sections,
  one home section per new identity rule, because a later slice makes sections addressable (IMP-008).
- **C-4 — design wording.** The version-2 document must not imply a home the store layout does not have:
  DEC-026/028 keep conventions sets in `<project>/.config/smut/conventions/`, outside `.smut/`.
- **C-5 — applied.** The `/tmp` reference above now points at `research/raw/review-integration.md`.
- **C-6 — operator's call, not applied.** Appending ", and no migration is provided" to the version
  refusal. It is the first thing a 0.1 store's owner meets and DEC-022 decided there is none.
- **Not re-litigated**, per Seat C and the operator: DEC-027's teach-not-diagnose answer, DEC-026/028
  layering and the conventions name, and the no-`roots` envelope decision. Checked consistent: the
  conventions seed key renames while its value and field list do not; the envelope keeps
  `depths`/`edges`/`frontier`; DEC-027 reads no store, so it and `where` do not move each other's
  compatibility property.

## Autonomy log

<!-- seat and sub-orchestrator envelope decisions: phase · decision · reason · decided/deferred/escalated -->

- **PHASE-01 · charter only, PHASE-02 held.** Decided. The handover's two operator
  gates (PHASE-02/EN-4 live-store re-declare, PHASE-02/EN-3 CHR-001 executed block)
  are the operator's, so the seat charters PHASE-01 alone and asks before charting
  past it. Reason: both gates bind PHASE-02 entrance criteria; charting ahead would
  either breach them or have a child invent operator assent.
- **Preflight · baseline gate accepted as green.** Decided. `just check` ran to
  exit 0 in this jail with 0 failures across both feature configurations, so any
  later red is a regression, not a baseline condition.
- **PHASE-01 · helper split into `validate_identity_tree` + `walk_identities`.**
  Decided (sub-orchestrator, then verified). Reason: design §3 fixes the
  `(entry, value)` signature, which cannot carry the diagnostic path; the wrapper
  holds one `String` and the inner walk pushes/truncates one component per step.
  Same single rule, no public surface, no new module. Records as a deviation, not a
  design change.
- **PHASE-01 · compile-error red accepted for the private helper.** Decided. The
  VT-3/VA-3 inline tests were written first and the lib test target failed with
  `E0425: cannot find function validate_identity_tree` ×7 while `cargo build` stayed
  clean. Reason: a not-yet-existing private symbol has no runtime red; the observed
  failure names exactly the missing symbol and no other error, so it is red for the
  intended reason. The two integration suites still had genuine runtime reds
  (`get`/`render` exit 0, `follow` exit 101 from the `scan_refs` panic).
- **PHASE-01 · injected fixtures spell the key literally (`"id"`).** Decided.
  Required by PHASE-01/VT-1: the semantic target stays visible in the diff, and
  PHASE-02's respell becomes a reviewed act rather than an invisible search-and-
  replace.
- **PHASE-01 · `docs/store-format.md` left untouched.** Decided. Its Corruption
  section would have to state a rule version 1's released reader did not enforce
  (EX-4, review B-F-1); STD-001's own "a format break moves the document with the
  version bump" clause puts it in PHASE-02. The corruption rule, bracket notation and
  worked diagnostic are recorded as pending PHASE-02 material in the runtime sheet
  and in this file.
- **PHASE-01 · `render` refusal test added beyond VT-2's pair.** Decided. `render`
  is the third traversal over the same load boundary and was silently exiting 0 on
  the corrupt store; one test, no new obligation.
- **PHASE-01 · path-component allocations accepted as bounded.** Decided. Each
  object/array component allocates only its JSON-escaped key or index text, never an
  entity subtree, which is VA-3's "allocates only the traversal path and the final
  error". Recorded so the audit reads the criterion the same way.
- **PHASE-01 · no new knowledge record minted at harvest.** Deferred. The one durable
  fact (serde_json's map is a `BTreeMap` here, so object iteration is lexical and the
  walk needs no sorting) is already stated in design §3 and in the helper's doc
  comment; minting a memory would duplicate it without a consumer. The audit may
  promote it if it becomes load-bearing elsewhere.
- **PHASE-01 · VT keyword mandates satisfied by naming, not by weakening.** Decided
  after `verify-vt` reported VT-1's `ENTITY_ID` and VT-2's
  `malformed_stored_identity` absent. Added a doc sentence explaining why the
  fixtures spell the key literally *rather than* through `ENTITY_ID` (adding the
  constant to a fixture would have contradicted VT-1's own literal-spelling mandate)
  and renamed the two refusal tests for what they refuse. No assertion, fixture or
  plan edit; the phase was re-opened, re-gated and re-closed to make the source
  presence attributable, and the range recorded with `slice record-delta`.
- **PHASE-01 · `verify-vt` verdicts treated as non-evidence.** Decided. They are
  source-presence mandates and are recorded separately from VA-1's behavioural
  results; after the delta record all three PHASE-01 VTs pass, and PHASE-02's
  unstarted FAILs are left alone for that phase.
- **Run shape · PHASE-01 is the terminus.** Decided. Operator re-affirmed the
  SL-005 ordering at the seat, so the charter range is PHASE-01 alone and the run
  ends there with a checkpoint. Reason: PHASE-02/EN-5 binds PHASE-02's entrance on
  SL-005's refusal phase landing in the shared `src/error.rs`; charting PHASE-02
  now would either breach EN-5 or force a rebase nobody has scoped.

### PHASE-02 — 2026-11-02 (sub-orchestrator seat) — **completed**

**Shipped.** `ENTITY_ID` keeps its name and takes the wire value `smut_id`; the two
production raw literals are routed through it (`src/declare.rs`'s change-receipt
subject, `src/conventions.rs`'s seed) and no production code spells the wire key
anywhere else — after the sweep, production `"smut_id"` occurrences are exactly the
constant (`src/entity.rs:17`) plus the guide's prose/examples, which state the
spelling rather than emit it (EX-1's deliberate exception). `FORMAT_VERSION` and
`MANIFEST_FORMAT_VERSION` are both 2; every store-building version literal is 2 and
every old-version fixture names 1 explicitly, including `tests/traversal.rs`'s
rebuilt index and `tests/store_corruption.rs`'s unknown-version case (now 3).
Ordinary `id` is data at every depth; `smut_id` is the identity. No module, alias,
shim, migration, dual reader, repair path, new dependency or error variant.

**Diagnostics settled (EX-9, quoted verbatim in `docs/store-format.md`):**
- `invalid smut_id: {found}, and smut_id must be a nonempty string` (`Reject::InvalidId`, exit 2)
- `the entity file {entry} holds {found} under smut_id, and the index maps it to {key}`
  (`StoreError::IdDisagreement`; `found` is the JSON value or `nothing`, so the absent
  case reads `holds nothing under smut_id`; `describe()`'s absent text is `nothing`)
- `identity at $["payload"]["tags"][2]["smut_id"] must be a nonempty string` (nested
  corruption, exit 1; unchanged wording, path component moves with the key)
- `scan_refs requires a validated entity: a `smut_id` that is not a nonempty string is corruption`
  (internal assertion, unreachable from a loaded store)
- `the store index has format_version 1, and this build reads only 2` (index gate, exit 1)
- manifest gate: `manifest.json is not a valid store entry: format_version 1 is not the
  version this build reads (2)`, refused before any staged put/removal replays.

**Fixture (EX-4/VA-1).** `tests/fixtures/interview/entities.json` was rewritten
spelling-only by a string-aware key rename: **223** object keys `id` → `smut_id`, no
value, ordinary field, numeric form or array position touched. Evidence: the original
at `2470576` is blob `f5b29de67b5f539be018c72c5e88c3bd0f301a93` (66 entities, 0
pre-existing `smut_id` keys, so the inverse map is unambiguous); the inverse-renamed
array is **byte-identical** to that blob and serde_json-structurally equal; a
temporary (uncommitted) harness rejected an altered value and a reordered array under
the same comparison, preserved array order, and separately proved the comparator keeps
`1` distinct from `1.0` (the fixture itself carries no JSON numbers outside strings,
so there was no numeric representation to preserve). `tests/fixture.rs`'s `CLUSTERS`
re-derived against the adapted array: the same five ids, which are the array's identity
order of the five `type: cluster` entities; the eight open questions, the over-cap
graph, the budget/frontier and the two-call recipe all re-checked and unchanged.

**Red → green, per home** (each red observed before its implementation, and the
observed failure is the contract, not a compile error):

| Home | Red observed | Green |
| --- | --- | --- |
| VT-1 `src/entity.rs` | 4 fail: `ENTITY_ID` `"id"` vs `"smut_id"`; canonical bytes `{"id":...}` first; `reference_id({"id":"data"})` = `Ok(Some)` | `cargo test --lib entity::` 36 passed, 0 failed |
| VT-2 `tests/cli_contract.rs` | 10 fail: root ordinary id became the entity id, depth values refused, receipt subject `"id"`, old-spelling `where` returned rows, DROP on data, containers | 50 passed, 0 failed |
| VT-3 `tests/store_corruption.rs` | first red at the version fixture (23 fail, INDEX said 2 while the build read 1); after the constant move, 15 fail on the identity contract (`IdDisagreement` on `smut_id` roots, ordinary-id data refused) | 27 passed, 0 failed |
| VT-4/VT-6 `tests/durability_crash.rs` | baseline `staged_manifest()` carried version 1 and replayed (a v1 manifest was accepted pre-bump); after the identity move the v1 fixture was refused while the v2 control replayed — the pair discriminates | 48 passed, 0 failed (`--features fault-injection`) |
| VT-5 `tests/conventions.rs` | 5 fail: seed carried `"id"`, receipt/stored/re-install expectations old key | 14 passed, 0 failed |
| VT-7 `src/learn.rs` | 7 prose pins fail, each naming the missing claim | `cargo test --lib learn::` 24 passed, 0 failed |
| VT-8 `tests/render.rs` | 6 fail: `smut_id` rendered as an attribute, ordinary `id` suppressed | 11 passed, 0 failed |
| fixture/traversal | `fixture` 10 fail, `traversal` 29 fail on the old key | 15 and 34 passed |

**Gate.** `just check` **exit 0** after the documents landed: build, both clippy configs
`-D warnings`, `cargo fmt --check`, default tests (370 lib + 10, 50, 10, 4, 14, 0, 15,
13, 5, 11, 4, 8, 27, 34, 0) and `--features fault-injection` (383 lib + 10, 50, 10, 4,
14, 48, 15, 13, 5, 11, 4, 8, 27, 34, 0); 0 failures, 0 ignored, 0 warnings. Baseline
was 359/372 lib and 40 `cli_contract`; nothing was removed or weakened. Scratch
lifecycle spot-check: declare with ordinary `id` data at depth (null, number, array,
object), `get`/`query` exit 0, `where` on ordinary data and on `smut_id` both exit 0,
old-spelling `where` empty at exit 0, `select:["smut_id"]` identity alone, `DROP` on
ordinary `id`, index `format_version` 2, delete — all as specified.

**Residual `id` classification (EX-6/VA-2), from the 1091 baseline to 65.**
- *Ordinary payload data under test* (the deliberate new cases): `tests/cli_contract.rs`
  28, `tests/store_corruption.rs` 12, `src/entity.rs` 13, `tests/render.rs` 1 — ids
  with null/empty/number/array/object values, mixed `id`+`smut_id` objects, `where`
  and `order` and `select` on `id` as data, `DROP` on `id`.
- *Arbitrary member in a non-entity fixture*: `src/input.rs` 2 (object-ordering and
  duplicate-key fixtures where the member is not identity).
- *Operator-owned prose, outside this slice's write scope*: `README.md` 5 (lines 16,
  73, 77, 249, 252) — the 0.2 rewrite and its never-executed ```` ``` console ````
  blocks; left untouched, now stale about the key. Flagged, not fixed.
- *Internal vocabulary and values*: `Id`, `Change.id`, map keys, basenames, `ids`,
  `deleted`, `missing`, `frontier`, `depths`, `edges[].source/.target`,
  `dangling[].source/.target` — retained by design; they name values/containers,
  never the JSON identity member.
- *Historical product prose*: the fixture bodies and the brief's narrative keep the
  word id where it describes past behaviour; the 223 structural keys are gone.
Every other occurrence was a missed structural use and was converted; no occurrence
is banned, and `rg` shows no production raw `"id"` key at all.

**Decisions/deviations.** (1) `IdDisagreement`'s sentence was restructured to
`holds {found} under smut_id` with `describe()`'s absent arm reading `nothing`, because
respelling in place produced `holds smut_id no smut_id`; both still name the key and the
entry/key. (2) Two test names that asserted an old JSON-key claim were renamed
(`smut_id_is_an_ordinary_order_field`, `project_gives_smut_id_only_for_an_empty_select…`)
rather than left saying something false. (3) `tests/store_corruption.rs` gained
`ENTITY_UNDER_EITHER_KEY`, a fixture carrying both keys so the version gate is the only
defect in the version-refusal test. (4) The `render` mechanism needed no code change —
suppression was already keyed on the constant — so EX-10 is pinned by test and
before/after bytes only. (5) The README commit initially staged the whole working-tree
file (the operator's rewrite included); it was caught before any push and **amended**
to a fence-only patch built from `HEAD`'s README plus the regenerated transcript, so
the commit's patch is 10/10 lines inside `:119`-`:146` and the operator's prose is
still uncommitted and unmodified. Recorded because it is a process miss, not a
source change.

**Workers and cost.** Thirteen native pi `worker` children, serial, main worktree, no
worktrees, no Quotient, inherited model configuration: four red-contract test runs
(entity+cli, store_corruption, durability, conventions/render/learn pins), two
implementation runs (entity, declare), two surface runs (conventions+error,
store/journal/index+lib sweep), two test sweeps (public suites, fixture+traversal),
and three document runs (learn+provenance, store-format, brief) plus the README fence.
No worker failure and no model substitution; one run was cut by the supervising
harness's wall clock after its edits had landed and was resumed by adopting the
partial diff. Wall time dominated by three `just check` runs and the document
verification probes.

**Range.** Own commits: `3e9f173` (identity core + both versions), `7d6391b` (public
suites), `b50861c` (corruption/conventions/render/recovery contract homes), `658e76e`
(fixture + re-derived assertions), `04d7424` (README fence only), `14f6a7d`
(documents + guide + provenance). First own commit's parent is `00bc823` (SL-005's
close), so the registry range is `00bc823..14f6a7d`; SL-005's `dfb14cf..00bc823`
commits sit outside it and are not hidden inside the phase's row.
- **PHASE-02 · version constants land before the identity constant.** Decided. The first
  `store_corruption` red after the version fixture moved to 2 was the version gate on every
  test, which masked the identity contract. Moving both constants to 2 first made the
  identity cases fail for their own reason (`IdDisagreement` on `smut_id` roots, ordinary-id
  data refused), so each VT's red is attributable. Reason: a red for the wrong reason is not
  evidence. No test was changed to obtain it.
- **PHASE-02 · `IdDisagreement` sentence restructured; `describe()`'s absent arm is `nothing`.**
  Decided. A literal respell produced `holds smut_id no smut_id`; the message is now
  `the entity file {entry} holds {found} under smut_id, and the index maps it to {key}` with
  `found` = the JSON value or `nothing`. Reason: EX-9 mandates the key be named and the
  entry/key be named; the absent case must read as English. Recorded as a wording decision
  inside EX-9's mandate, not a design change.
- **PHASE-02 · two test names renamed where they asserted an old JSON-key claim.** Decided.
  `an_id_is_an_ordinary_order_field` and `project_gives_id_only_…` would have been false after
  the key moved; they now name `smut_id` and assert the same behaviour. Reason: a passing test
  with a false name is worse than a renamed one; no assertion changed.
- **PHASE-02 · fixture rewritten by a string-aware key scanner, formatting preserved.**
  Decided. The alternative (parse and re-serialise) would have destroyed the file's formatting
  and made the diff unreadable against DEC-024's "spelling-only". The scanner rewrites only
  object keys, leaving body prose that mentions the word id intact; inverse rename is
  byte-identical to the recorded blob.
- **PHASE-02 · the README commit was amended to a fence-only patch.** Decided, self-caught.
  The first commit with `-- README.md` committed the working-tree file, which is the operator's
  0.2 rewrite plus the new fence — a boundary breach. Before any push, `HEAD` was amended to
  `HEAD`'s README with only the fence replaced (built via `git hash-object` +
  `git update-index --cacheinfo` + pathless `--amend`), so the commit's patch is 10/10 lines
  inside `:119`-`:146` and the operator's prose is again uncommitted. Logged because the first
  attempt was wrong, not because the outcome is.
- **PHASE-02 · README's surrounding prose stays stale.** Deferred. It names `"id"` in five
  places and its never-executed fences show old declarations; it is CHR-001's write scope and
  this slice may not sweep, stage or revert it. Flagged in Harvest/Open for the operator.
- **PHASE-02 · `src/input.rs`'s two `{"id": …}` fixtures left as data.** Decided. They are
  object-ordering/duplicate-key fixtures whose member is arbitrary, not an entity node; keeping
  `id` there adds ordinary-data coverage and asserts nothing about identity.
- **PHASE-02 · document worker verified every quoted diagnostic in a scratch store.** Decided.
  EX-9/STD-001 make the worked diagnostics contractual, so each line in `docs/store-format.md`
  was triggered against this build rather than transcribed from code.
- **PHASE-02 · `src/predicate.rs` is undeclared in `slice conformance`, knowingly.** Decided.
  Its four inline-test fixtures were respelled; only `#[cfg(test)]` lines changed (verified in
  the diff), and the file is not in design §4's code-impact table. Recorded here and left in
  the conformance cell rather than hidden or reverted: a test fixture naming an entity node
  must move with the key, and the audit should see it.
- **PHASE-02 · three foreign commits inside the range, logged not hidden.** Observed.
  `ff7bb05` (SL-006 design), `920ce82` (backlog capture) and `cb5c74d` (SL-006 harvest)
  landed in this worktree between own commits `658e76e` and `04d7424`. They are another
  writer's `.doctrine`-only work, were never staged by this run, and no source path is in
  them (verified); they therefore ride inside `00bc823..14f6a7d` and appear in
  `slice conformance`'s undeclared cell exactly as this slice's rules require.

### PHASE-03 autonomy entries (sub-orchestrator seat)

- **PHASE-03 · probes run by the seat, no worker spawned.** Decided. The phase's
  deliverable *is* independent observation: `VA-2` says "run fresh CLI lifecycle and
  refusal probes … independently" and the charter says "do not transcribe the seat's or
  PHASE-02's results". A worker's handback would insert a transcription layer between
  the command and the evidence table, and there was no source change needing a
  red/green delegation. Cost: the seat's own context, not a worker run.
- **PHASE-03 · `just check` started before anything else.** Decided. The gate is the
  long pole and no probe can influence it; starting it in the phase's first minute and
  reading it at the end is what made the 30-minute charter feasible.
- **PHASE-03 · the amendment proposal committed before the notes rollup.** Decided.
  The harness's wall clock has killed one run of this slice; the proposal is the
  phase's least reproducible artefact and the notes are its most repeatable, so the
  proposal landed as its own commit first (`24682b2`).
- **PHASE-03 · no decision record edited, no audit begun.** Decided, per charter and
  design §4. The proposal is a durable slice artefact
  (`.doctrine/slice/004/decision-amendments.md`) handed to `/reconcile`; the six
  records still read exactly as they did at `963a788`.
- **PHASE-03 · DEC-002 and DEC-012 recorded as "no facet change".** Decided. Their
  facet fields contain no identity spelling — the `id` occurrences are container
  names (`ids`, `depths`, `edges`, `frontier`) and id values, which design §4 keeps.
  Inventing a facet edit for uniformity would be a false amendment.
- **PHASE-03 · the malformed-manifest control fixture repaired, not reinterpreted.**
  Observed, then decided. My first control omitted `index.json` from the manifest's
  `puts`, so the replay published two entity files against a stale index and the
  loader refused with "unexpected entry … is not explained by the index". That is a
  fixture defect, so the batch was rebuilt to stage the index last (as the durability
  suite's canonical batch does) rather than reported as a product finding. The
  corrected pair discriminates: 1 → refusal with zero replay, 2 → whole replay.
- **PHASE-03 · no new knowledge record minted.** Deferred. The durable lessons (a
  hand-built recovery fixture must stage the index inside the batch; no `python3` in
  this jail, use `jq`; DEC-002/012 carry no identity spelling in their facets) are in
  this file's `### Learned` and the phase sheet. The audit may promote any of them.
- **PHASE-03 · the README prose finding and the `src/predicate.rs` selector gap left
  open, in writing.** Decided. Both are outside this phase's authority: the README is
  CHR-001's write scope and the design is locked. They are in `### Open` as unresolved
  work for the operator and the audit, not silently dropped.

### PHASE-03 — 2026-11-02 (sub-orchestrator seat) — **completed**

**Shipped.** No source change — the phase is the evidence and handoff phase, and the
probes found no defect to fix. Two deliverables:

1. `.doctrine/slice/004/decision-amendments.md` (commit `24682b2`) — the per-record
   amendment proposal for **DEC-002, DEC-005, DEC-007, DEC-010, DEC-012, DEC-016**.
   Every "current" passage was read from the record's facet *and* prose tier with
   `doctrine show` on 2026-11-02, not from the design table or an earlier paraphrase.
   Shape: per record a facet table (field · current verbatim · proposed · changed or
   retained) and a prose table anchored by section heading, then an explicit
   *retained* list of the rationale and consequences that do not move, then the dated
   ADR-001/SL-004 amendment note. Two hard requirements are discharged or refused
   precisely:
   - **DEC-005 is proposed as a withdrawal, not a respelling.** Its *Documented
     consequence* ("An embedded object carrying `id` cannot be inline metadata in
     0.1…") and its *What counts* rule ("A `Value::Object` with a member `id`…") are
     replaced by a rule that keeps `smut_id` reserved at every depth and states the
     converse — an ordinary `id` **is** data and may be inline metadata — with a
     DEC-021 citation for the loader enforcement. Its quoted signature is corrected
     from `Result<Option<&str>, Error>` to `Result<Option<&str>, Reject>`, which is
     what `src/entity.rs:32` returns.
   - **DEC-007, DEC-010, DEC-016** move to `smut_id` / version 2 in their facet wording
     and prose; **DEC-002 and DEC-012** move only in their examples, because no facet
     field in either names the identity member — their `id` occurrences are container
     names and id values, which design §4 keeps.
2. The independently repeated evidence set below, with the seat's and PHASE-02's
   results not transcribed.

**Gate (this run, on the implementation tree).** `just check` **exit 0**, 0 failures,
0 ignored, 0 warnings: build, both clippy configs at `-D warnings`, `cargo fmt
--check`, then default tests (lib **370**; cli_contract 50, compatibility 10,
concurrency 4, conventions 14, durability 0, fixture 15, predicates 13,
readme_example 5, render 11, roundtrip 4, selection 8, store_corruption 27, traversal
34, doctests 0) and `--features fault-injection` (lib **383**, durability_crash
**48**, every other suite identical). `doctrine slice verify-vt 4`: PHASE-01 VT-1..3
and PHASE-02 VT-1..8 **all PASS** — source presence only, recorded separately from
the behavioural probes, because it cannot prove any probe ran.

**Lifecycle probes (fresh scratch stores, `target/debug/smut`, no live store).**

| Probe | Result | Exit |
| --- | --- | --- |
| fresh format 2: `declare` p1 with `id:"legacy-7"` at the root and `id` values at depth (7, `"deep-tag"`, `null`, `[1,2]`) | created; index `format_version` **2**; stored file names `smut_id` first and keeps every `id` verbatim, including `"id": null` and `"id": [1,2]` | 0 |
| `get`, `query where id=="legacy-7"`, `query where smut_id=="p1"`, `select:["smut_id"]` | ordinary `id` is data and selectable; identity selects; identity-only projection returns `[{"smut_id":"p1"}]` | 0 each |
| `query where id=="p1"` (identity under the old spelling) | `"objects":[]` — a field no entity carries fails nobody; no error | 0 |
| `render` / `render --show-fields` | ordinary `id` renders as `- **id:** legacy-7`; `smut_id` suppressed | 0 |
| `delete` then `query` | `{"deleted":["p1"],"missing":[]}` then `"missing":["p1"]` | 0 |
| version-1 index (`jq '.format_version = 1'`) → `get` / `query` | `smut: the store index has format_version 1, and this build reads only 2`, empty stdout, index bytes unchanged (`md5sum -c`) | **1** / **1** |
| version-1 committed manifest (puts incl. a fresh uid **and `index.json`**, one removal) → `get alpha` | `smut: manifest.json is not a valid store entry: format_version 1 is not the version this build reads (2)`; all three published documents byte-identical after the refusal; `journal.commit/` retains the evidence | **1** |
| the same store and staged files, manifest `format_version: 2` | replayed whole: alpha → `"changed"`, `Fresh.json` published, beta removed, index now lists alpha+gamma | **0** |
| RV-022 F-3 re-run: nested `tags[1].smut_id` hand-edited to `null` → `get root` (field not followed) | `smut: w10eP…c.json is not a valid store entry: identity at $["tags"][1]["smut_id"] must be a nonempty string`, empty stdout | **1** |
| same corrupt store → `query follow ["kids"]`, and `render` | identical refusal; entity and index bytes unchanged | **1** / **1** |
| no-panic check on the corrupt store | exit 1, not 101 or 134 | 1 |
| control: identity restored | both objects, `depths {k1:1,root:0}`, one edge, `frontier ["k1"]` | 0 |
| seed's `IdDisagreement` re-derived | `the entity file … holds "other" under smut_id, and the index maps it to mapped` | 1 |
| mapped file with **no** identity member | `the entity file … holds nothing under smut_id, and the index maps it to mapped` — `describe()`'s absent arm is `nothing` and `docs/store-format.md:267` quotes this sentence | 1 |
| invalid identity in a *request* (`declare [{"smut_id":null}]`) | `invalid smut_id: null, and smut_id must be a nonempty string`; the store is not created | **2** |

The version-1 control is a genuine discrimination, not a no-op fixture: the same
store and the same staged bytes give a version refusal at 1 and a complete replay at
2. (First attempt omitted `index.json` from the manifest's `puts` and the replay
published against a stale index; that is a fixture defect, corrected by staging the
index last as the durability suite's canonical batch does. Recorded so the first
result is not read as a recovery defect.)

**Fixture equivalence, repeated from the recorded original.** Baseline
`tests/fixtures/interview/entities.json` at revision `2470576` is blob
`f5b29de67b5f539be018c72c5e88c3bd0f301a93` (75 147 bytes; content sha1
`4637754f0be58979b3081385854ae3377c51a6e5`), **66** entities, **0** `smut_id`,
**223** `"id"`. Read with `git cat-file blob`; the committed fixture today is 66
entities / 223 `smut_id` / **0** `"id"`. `sed 's/"smut_id":/"id":/g'` over the
committed fixture is **byte-identical** to that blob (`cmp` silent, sha1 equal), and
`jq -S` canonicalisations compare equal. Both negative controls fire on the same
procedure: an altered `title` and a swapped array pair each differ under `cmp` *and*
under the structural comparison, and 66 entities survive the swap. `[{"a":1}]` vs
`[{"a":1.0}]` stay distinct under the canonicaliser, so the comparison keeps `1`
≠ `1.0`. No second fixture was committed.

**Residual `id` classification, from the repo-wide count.** `rg -o '"id"' -g
'!target/**' | wc -l` = **65** (pre-phase baseline re-measured at **1091**; the
slice body's 984 and the review's 974 predate SL-005). By meaning: ordinary payload
data under test **54** (cli_contract 28, entity 13, store_corruption 12, render 1);
arbitrary member in a non-entity fixture **2** (`src/input.rs`); operator prose
outside this slice's scope **5** (`README.md` 16, 73, 77, 249, 252); historical
product prose keeps the *word* id with **0** structural keys; missed structural use
**0**. The six decisions' pending reconciliation targets are a separate ledger
(`.doctrine/knowledge/decision/002,005,007,010,012,016`, mapped in the amendment
proposal, not counted as residual defect). Wire spelling in production: exactly one
non-test occurrence, `src/entity.rs:17` `ENTITY_ID`, plus `src/learn.rs:271/277/284`,
which state the spelling in guide examples rather than emit it.

**Decisions.** (1) The probes were run by the seat rather than delegated: evidence
that must be *independently repeated* is not evidence if it arrives through a
worker's handback, and with no source change there was no red/green cycle to
delegate. (2) `just check` was started in the phase's first minute and read at the
end, so the long pole ran underneath the probe and authoring work. (3) The
amendment proposal is a committed slice artefact, never a record edit; `/reconcile`
remains the only writer of the six records. (4) DEC-002 and DEC-012 are recorded as
having **no facet change** rather than given a uniform-looking invented one.

**Range and registry.** Own commits `24682b2` (the amendment proposal) and
`8ea2ee7` (this rollup), plus the follow-up commit that records the resolved range;
first own commit's parent is `963a788` (the seat's PHASE-02 review), so the registry
range is `963a788..<tip>` and no foreign commit sits inside it — `git log --oneline
963a788..8ea2ee7` lists exactly the two. `doctrine slice verify-vt 4` re-run after
completion and the recorded delta (all 11 VTs PASS); `doctrine slice conformance 4`
read at the terminus: 31 selectors conformant, 24 undeclared (the foreign `.doctrine`
work of other writers, this slice's own governance files, and `src/predicate.rs`'s
selector gap), 6 undelivered — exactly the six decisions this phase proposes
amendments for.

**Workers and cost.** None spawned. Wall clock was dominated by the one `just check`
run and the seven-surface authoring of the proposal; the probes are sub-second
against the warm `target/` and scratch stores. The 30-minute charter clock was the
binding constraint, which is why the amendment proposal was committed as its own
slice before the notes rollup.
