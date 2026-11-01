# Review RV-004 — reconciliation of SL-001

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Mode: **conformance** audit, post-implementation, at lifecycle `audit`
(head `abf690e`, 9/9 phases complete). Self-audit: both roles driven by the
audit seat (`--as raiser` / `--as responder`).

What this pass holds the slice to:

1. **DEC-001…DEC-011 are canon.** Each accepted decision is a governance entity
   in this repo; where the build's observable behaviour and a decision's
   mechanism clause disagree, the decision wins and the code is the defect.
   §4.2/DEC-007's "one shared Entity wrapper, used for stored entity files and
   for projected entities alike" is the first place to look, because the
   envelope path and the file path are separate call sites.
2. **The plan's `EX-`/`VT-`/`VA-`/`VH-` bar**, phase by phase. Verification
   rows are re-run, not taken from the run's own account; `VH-` rows are
   *human* obligations and are not treated as passed because the seat says so.
3. **§8.1's ten invariants**, especially: one index writer; recovery before
   validation under the lock; the core cannot express a storage failure; only
   ids are rendered; entity serialisation at the byte boundary through one
   wrapper.
4. **The brief's exit-code and stream contract** (`0/1/2`; JSON on stdout,
   diagnostics on stderr; no success envelope on failure) — an exit code outside
   the table is a contract violation however rare the trigger.
5. **Path conformance**: `design-target` selectors vs what git actually touched.
   Undeclared paths are dispositioned, not tutted at; undelivered selectors are
   checked for dropped work.
6. **The audit's own tooling is in scope**: `doctrine check gate` is the
   per-phase gate cadence the lifecycle expects, so a project whose `justfile`
   cannot answer it has a gap, not a convention.
7. **Every open item the run recorded in `notes.md` `## Harvest` → Open** gets
   an explicit disposition, including the one the seat deliberately left free
   (entity-root key order inside an envelope). Nothing carries over as
   "still open" without a rationale or an owning future unit.

Evidence gathered before the first finding: `just check` exit 0 (0 warnings);
`slice verify-vt` 28 PASS / 0 FAIL; `slice conformance` 32 conformant /
2 undeclared / 0 undelivered; `doctrine validate` clean; tree clean at
`abf690e`; the built binary exercised by hand in scratch stores (declaration
semantics, patch/DROP, generated ids, envelope, exits, streams, TTY and pipe
paths, corruption shapes). Findings are measured against the built binary, not
against the run's notes.

Environment caveat recorded here because it changes how ids were minted: this
checkout cannot push reservation refs (HTTPS credentials absent, SSH disabled),
so `review new` failed until `.doctrine/doctrine.toml` gained
`[reservation] reach = "local"`. RV-004 is therefore locally reserved only —
the same posture as RV-001…003 and DEC-001…011 in this clone.

## Synthesis

**The closure story.** The implementation is the plan's, and the plan's bar held:
`just check` green (0 warnings), `slice verify-vt` 28 PASS / 0 FAIL across nine
phases, `slice conformance` 32 conformant / 0 undelivered, `doctrine validate`
clean, and every `EX-` criterion re-read against the code rather than against the
run's own account. The three things a review of *evidence* rather than *claims*
found were therefore not failures of the plan — they were the plan's own recorded
gaps, plus one place where the run's aggregate numbers hid a divergence:

- **A mechanism clause DEC-007 states and the code did not obey.** The wrapper was
  applied at the file root only, so the same entity serialised two ways depending
  on where it appeared. Fixed (`78a7eb1`): the rule is now structural — an entity
  node names `id` first wherever it appears — and the two entry points are one
  function, so the divergence has nowhere to live. This is the one change to
  documented output bytes in the audit, and it was made because the alternative
  was amending canon to match an accident.
- **A panic outside the exit table.** `print!` on a closed stdout exited 101.
  Fixed (`35b62ed`): the prepared bytes are written fallibly, a broken pipe exits
  1 without noise, any other output failure is reported.
- **The lifecycle's own gate name was unanswerable.** `doctrine check gate`
  defaulted to a `just gate` recipe that did not exist. Fixed (`dfc6e37`).

Everything else the run had already recorded honestly, and this pass dispositioned
it rather than inheriting it: seven hand-back candidates (all `tolerated` or
`aligned`, each with the reason the *design's own posture* gives), the two §12
errata and the two unnamed fsyncs (`design-wrong`, folded into the brief), the
design review's stale cache (a measure artefact, not drift — the reviewed text has
not moved since `c32f316`), the brief divergence (the slice's artefacts owe the
sentence, not the brief), and the two `VH-` criteria the plan always said a human
must read.

**Standing risks, consciously accepted.**

- **Power-loss durability is assumed, never measured.** The interruption suite
  proves code-path ordering only; the guarantee's evidence is a human read of
  that suite plus the file's own scoping sentence. That is PHASE-05/VH-1, and it
  is still owed.
- **`Store::load` is laxer than the pipeline** — it accepts a live journal state
  that `recover` would refuse. No command can reach it; a future test could be
  fooled by it. Recorded, not engineered around, because the invariant it would
  duplicate is held one frame up and the fixture that depends on its current
  behaviour (`tests/concurrency.rs`) is real evidence about the pipeline.
- **Corruption handling is refusal-only**, which is right for 0.1 and means a
  store damaged outside the tool has no recovery path. Nested entity-node ids
  inside stored content are the one shape the loader does not inspect; inert in
  0.1, an obligation for 0.2's traversal.
- **The lock is advisory and follows a symlink.** Both are stated in the
  portability note; neither can corrupt store bytes.
- **`reservation reach = "local"`** (`.doctrine/doctrine.toml`). This checkout
  cannot push reservation refs, so ids are reserved locally only — the same
  posture RV-001…003 and DEC-001…011 already have. Revert to `shared` when the
  remote is writable; it is a configuration posture, not a release property.

**Tradeoffs the audit made and would defend.** Two behaviours were fixed at audit
time rather than deferred because the alternative was to ship a release whose
documented guarantee (id-first through one shared wrapper) and documented exit
table (0/1/2) were both false in reachable use; each fix is small, pinned by a
test that fails without it, and separable in its own commit. Nothing else was
touched: the tolerated set is deliberate, not leftover.

**What is still owed, and by whom.** PHASE-05/VH-1 and PHASE-08/VH-1 are human
reads. Both are presented with their mechanical evidence; neither is discharged
by this audit's re-run of the machines, and the ledger keeps both non-terminal
until the human answers.

## Reconciliation Brief

Ordered by surface. Every write below is driven by a finding; the finding ids are
the ledger's.

### Per-slice (direct edit)

- **`design.md §12` — selector registry (F-4).** Load-bearing verb first:
  `doctrine slice selector add SL-001 Cargo.lock --intent design-target`
  (`slice conformance` reads the registry, not the prose). Then mirror it in §12's
  "Design-target selectors" line, and say in the `Cargo.toml` row that the
  dependency set rewrites `Cargo.lock`. After this, the only undeclared path left
  is `.doctrine/slice/001/notes.md`, which F-5 tolerates deliberately.
- **`design.md §12` — two path errata (F-6).** The intro sentence stops calling
  `README.md` new: it is *edited*, around the owner's pre-existing 383-byte brand
  block and `smut.jpg` reference. The module map gains
  `src/lib.rs — the library target; the binary stays a thin dispatcher`, with
  §11.1's `tests/**` layers as the reason it exists.
- **`design.md §4.3` and `§4.4` — three unnamed durable steps (F-7).** §4.3 names
  the post-apply `fsync(.smut)` that precedes `commit → done` and the one after
  the `done` rename; §4.4's recipe gains the fsync after replay and before its own
  `commit → done` rename, with the note that it mirrors §4.3's apply rule (without
  it the state transition could survive while the replayed bytes do not). Record
  that these post-decision steps share a roll-forward reaction, which is why the
  registry gives them a name each step can reach — or say plainly that
  `RemovalsApplied` / `DoneRenamed` / `RecoveryDoneRenamed` cover them, so a
  future reader is not left inferring a coverage gap.
- **`design.md §1` — the brief's acceptance example boundary (F-9).** One
  sentence: the brief's v1 is delivered across 0.1 (this slice), 0.2 (IMP-001) and
  1.0 (IMP-002); its end-to-end example uses `follow` and `where`, so 0.1 can run
  its prefix only, and the README pins that prefix plus the two refusals. Carry
  the same sentence into `slice-001.md`'s scope if the scope does not already
  bound the brief's example.
- **`design.md §4.2` — mirror the wrapper clarification (F-1).** The rule is
  structural: the wrapper's subject is the entity node, so an entity root names
  `id` first *wherever it appears* — including a projected entity inside a
  response's `objects` array — and "embedded ordinary objects are all-lexical"
  means objects that are not entity nodes. This is what `78a7eb1` implements;
  the design should state it rather than leave it to be re-derived.

### Governance/spec (REV)

- **DEC-007 — REV modify (F-1).** One sentence in the decision's identity, not a
  redesign: *the wrapper's subject is the entity node (an object whose `id` is a
  nonempty string), not the value's position in a document* — so stored files and
  projected entities are serialised by one rule, and a value that is not an entity
  node stays all-lexical. Evidence: RV-004 F-1, commit `78a7eb1`,
  `tests/cli_contract.rs::a_projected_entity_is_serialised_by_the_same_wrapper_as_a_stored_one`.
  Everything else in DEC-007 stands as written.

### No write owed

- **F-10 and F-11** are human reads (PHASE-05/VH-1, PHASE-08/VH-1). No artefact
  changes; do not treat this brief as their discharge.
- **F-5, F-8, F-12, F-13, F-14, F-15** are tolerated with their rationale on the
  ledger; **F-16, F-17** are aligned. No edits follow from them.
- **F-1, F-2, F-3** were fixed inside this audit (`78a7eb1`, `35b62ed`,
  `dfc6e37`) and need no reconcile write beyond the design/DEC clarifications
  listed above.

### Addendum, after conclusion: the owner's packaging decision (2026-11-01)

Not audit-covered — it landed after the pass concluded — but it touches two brief
surfaces, so record it here rather than let reconcile miss it:

- **`design.md §12`** gains a `LICENSE` row (GPL-3.0-only, the FSF's verbatim
  text) and says the `Cargo.toml` row also carries the crates.io identity
  (description, repository, homepage, keywords, categories, and an `exclude` list
  for governance-only paths). The owner fixed the licence the brief had left
  open; the design should say so rather than let a package manifest be the only
  record.
- **Selector registry (F-4's item)** stays as written — `Cargo.lock` plus its §12
  mirror. `LICENSE` is a new path but not a *recorded phase delta*, so
  `slice conformance` does not see it; adding it to `design-target` is harmless
  and truthful if the reconcile pass prefers the table to stay complete.
- No new finding, no changed disposition: `cargo publish --dry-run` is clean, the
  name is free, and the slice's code and gates are unchanged.

## Reconciliation Outcome

Owner agreed the six per-slice edits and the governance item; this records what
landed, what changed shape while landing, and the state the slice is in for
`/close`.

### Direct edits applied

- **`design.md §12` — selector omission and path errata (F-4, F-6, addendum).**
  The load-bearing verb first: `doctrine slice selector add SL-001 Cargo.lock
  --intent design-target` (note: "the dependency set rewrites the committed lock
  file"). Mirrored in §12's selector line and in the `Cargo.toml` row, which also
  now carries the crate's published identity and the `exclude` list. The intro
  sentence stops calling `README.md` new — it is *edited* around the owner's
  pre-existing brand block — and the module map gains `src/lib.rs` as the library
  target with §11.1's `tests/**` layers as its reason.
- **`design.md §12` — `LICENSE` row (addendum).** The owner's post-audit licence
  decision is recorded in the table as a delivered path.
- **`design.md §4.2` — the wrapper's subject (F-1).** The rule is now structural:
  the wrapper's subject is the entity node (an object with a nonempty string `id`)
  wherever it appears, including inside `objects`; every other object is lexical.
  This is what `78a7eb1` implements.
- **`design.md §4.3` — the two unnamed `.smut` fsyncs (F-7).** Named: the one at the
  end of apply (after the removals, and performed even when nothing is removed) and
  the one after the `commit → done` rename. The section now says a boundary names a
  moment, not a syscall, so the shared names (`RemovalsApplied`, `DoneRenamed`) are
  deliberate rather than a coverage gap.
- **`design.md §4.4` — the recovery recipe's fsyncs (F-7).** The recipe gains the
  fsync after replay and before the rename, and the fsync after it, with why the
  first mirrors §4.3's apply rule and why both share `RecoveryDoneRenamed`.
- **`design.md §1` — the brief's acceptance-example boundary (F-9).** One sentence:
  the brief's end-to-end example uses `follow` and `where`, so 0.1 runs its prefix
  (pinned by the README's worked example) and the traversal steps are 0.2's.
  `slice-001.md` already carried the 0.1/0.2/1.0 release split, so no second copy
  was written.

### Governance

- **DEC-007 — amended, not revised.** The clarification landed through
  `doctrine knowledge edit`: the `choice` facet now names the entity node as the
  wrapper's subject, and the body gains a *Serialisation clarification* section
  citing F-1, `78a7eb1` and the pinning test.
- **REV-001 (`reconcile-sl-001`) — abandoned without a change row.** The revision
  engine refuses the target: *"`revises` target must be one of [SPEC, PRD, REQ,
  ADR, POL, STD], got a DEC"*. A decision is a knowledge record, not a governed
  artefact in the REV target set, and the refusal is not overridable — so the
  sanctioned surface is `knowledge edit`. The REV stands as the record of why the
  ordinary route did not apply; its narrative carries the finding link.

### Registry correction made during the pass

`LICENSE` was first added to `design-target` alongside `Cargo.lock`, and
`slice conformance` immediately read it **undelivered** — true to the engine and
false to the facts, because `LICENSE` was delivered in `2e6cfe7`, after the last
recorded phase delta. The selector was removed and the reason recorded in §12:
`slice conformance` measures *recorded phase deltas*, so a path delivered outside
one can only ever read as undelivered as a selector. This is the mirror image of
F-5's tolerated `notes.md`. `Cargo.lock` stayed: it *was* rewritten inside the
phases that changed the dependency set, and it now reads conformant.

### Human acts recorded

- **F-10 / PHASE-05/VH-1** — the owner waived the criterion's *human mode* and
  directed agent discharge; the read was performed against §4.3–§4.5 and found the
  suite complete against the registry, with the shared-name gap belonging to the
  design (F-7) rather than the tests. Deviation recorded, not laundered.
- **F-11 / PHASE-08/VH-1** — the owner read the README and reported it reads well
  enough; the runnable half was verified here (`readme_example` 2/2, and the
  transcript re-run by hand against the release build).

### Post-audit state recorded here rather than reconciled

- `smut` 0.1.0 is **published** (crates.io API: created 2026-11-01T08:30:46Z, one
  version, not yanked), which settles the licence question by the act.
- The operator's interview store (`interview.local/.smut`, 66 entities) was read
  and triaged: **no 0.1 conformance defect**, two findings corroborated
  independently (F-9, F-7), and 0.2 design input captured. Its residue became
  `IMP-003` (the cheap 0.1.x fixes), `ISS-001` (the published page's dead relative
  links) and a *Trigger met* section on `IMP-001`.

### Verification after the writes

`just check` → exit 0, 0 warnings, 354 default / 411 under
`--features fault-injection`; `doctrine check gate` → exit 0; `doctrine validate`
→ corpus clean; `slice verify-vt SL-001` → 28 PASS / 0 FAIL; `slice conformance
SL-001` → 33 conformant, **0 undelivered**, 1 undeclared
(`.doctrine/slice/001/notes.md`, tolerated by F-5).

### Not reconciled, deliberately

No finding was re-disposed and no brief item was skipped: the tolerated set
(F-5, F-8, F-12…F-15) and the aligned pair (F-16, F-17) are untouched, per the
brief's "no write owed". Nothing is outstanding for `/close` except the terminal
transition itself.
