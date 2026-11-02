# Notes SL-004: Identity is smut_id — the format break, and a loader that refuses what it cannot trust

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · PHASE-01 completed (commit d7b6daa) · PT-02 next, gated on SL-005

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

### Open
- SL-004 — PHASE-02 next, but chartered only after SL-005's refusal phase lands (shared
  `src/error.rs`, `after: SL-005`); both PHASE-02 operator gates (EN-3, EN-4) are now recorded
- SL-004 — PHASE-02 must carry PHASE-01's pending doc material: the nested-identity corruption
  rule, the `$["key"][0]` notation and the worked diagnostic respelled to `smut_id`
- SL-004 — PHASE-02 respells PHASE-01's literal `"id"` fixtures (`tests/store_corruption.rs`,
  `tests/cli_contract.rs`, the inline walk suite) — a reviewed act, not a search-and-replace
- SL-004 — formal audit and the six-decision reconciliation handoff (PHASE-03)
- RV-025 — 8 minor + 2 nit answered, none contested; not verified, no route required
- Doctrine — design status reports `review_pass RV-025 STALE` on a locked run whose content matches its watermark and whose cache is current (projection quirk, not a gate)

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
