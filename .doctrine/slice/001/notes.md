# Notes SL-001: smut v1 — persisted mutable entity graph CLI

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring · 2026-11-01)

Feedstock for the inquiry map. Sources: the brief (authority), SL-001 scope,
`research/research.md`. Blocking marks here are proposals — the user confirms the
map before the run moves to `inquiring`.

### Constraining governance

No specs, ADRs, policies or standards exist (`spec|adr|policy|standard list` all
empty). The brief is the whole canon; `AGENTS.md` binds process only. Consequence:
every load-bearing choice needs a recorded rationale, because there is no other
authority for a later reader to consult. Nothing here may be recorded as an ADR
mid-design — these become design-run DEC records.

### Open questions

Blocking = an answer that changes the shape of everything downstream.

| id | question | blocking | why |
| --- | --- | --- | --- |
| Q1 | What exactly is the pure core, and what crosses its boundary in each direction? | yes | Decides module layout, every test seam, and whether the journal can stay content-blind. Working answer in `research.md` § Cross-thread; needs to be stated as a contract. |
| Q2 | What is the exact journal protocol — states, files, fsync points, recovery algorithm — and what is the tested durability claim? | yes | The only substantive correctness risk in v1; the brief mandates interruption testing at every boundary. Decides the README/format-doc language (recoverable, not atomic). |
| Q3 | Is there one reference predicate shared by declaration and traversal, and what exactly does it consider a reference? | yes | Two consumers, one rule. Divergence makes query see edges `declare` never created. |
| Q4 | What is the precise equality used for coalescing/conflict, and how does it treat JSON numbers (`1` vs `1.0`), key order, and `null`? | yes | Silently loses or silently rejects assignments if wrong. Brief says "structurally"; `serde_json::Number` makes that ambiguous. |
| Q5 | What are the traversal determinism rules — discovery order, budget accounting, dedupe timing, dangling collection — such that output is stable across filesystems? | yes | Determinism is a contracted property, not a nicety; budget/limit interact with truncation reporting. |
| Q6 | How are `select` (projection) and traversal separated so Mermaid keeps edges whose fields are hidden? | yes | Fusing them destroys connectivity the moment `select` is used; also fixes the query/get envelope shape. |
| Q7 | What does the lock cover, when does recovery run relative to store validation, and how is initialisation coordinated? | yes | Ordering is load-bearing: recovery must precede the "unindexed files are not adopted" refusal, or one crashed command bricks the store. Concurrency tests depend on it. |
| Q8 | How is CLI input modelled (clap derive vs hand-rolled), including `-`, missing-stdin-on-tty, and varargs-vs-JSON positional commands? | no | Bounded; but it fixes the exit-code/stderr discipline the tests pin. |
| Q9 | What are the Markdown and Mermaid layout rules in detail — render order, fallbacks, escaping, truncation disclosure? | no | Bounded, but escaping needs a hostile-label fixture. |
| Q10 | What is the module layout and naming, and where do entropy/generated IDs enter? | no | Follows from Q1; naming matters for the shared predicate. |
| Q11 | What docs ship (README, on-disk format, portability limits) and where do they live? | no | Bounded; the format doc is the durability promise's home. |
| Q12 | What is the test strategy — unit vs integration split, fixtures, crash injection, concurrency determinism? | no | Follows from Q1/Q2; determines whether the mandated tests are honest or theatre. |

### Risks

- **R1** `flock` is advisory: a non-cooperating process (editor, `cat >`) is not
  excluded. Brief already forbids supported manual editing; document it.
- **R2** fsync cannot be *proven* in tests on overlayfs/tmpfs. The honest tested
  model is deterministic crash injection at protocol boundaries, not simulated
  power loss. Say so in the format doc.
- **R3** Crash-injection tests are flaky if they depend on timing; they must abort
  at *named* boundaries via an env var, never sleep-and-kill.
- **R4** Concurrency tests can be flaky; assert invariants (serialisable outcome,
  no lost update, no corrupt state) with a bounded retry, not an interleaving.
- **R5** The duplicate-key visitor is easy to implement only at the top level; it
  must recurse, and the test must include a duplicate inside a nested array item.
- **R6** The `1` vs `1.0` equality edge will surprise users; it needs a test and a
  line in the format doc, not a footnote in a comment.
- **R7** Mermaid mis-rendering is silent; escaping must be table-driven with a
  hostile-label fixture (quotes, brackets, newlines, `#`, unicode).
- **R8** Scope creep lives in the query language. Guard: the brief's non-goals are
  binding; any extra operator is an explicit recorded decision.
- **R9** Ordering: relying on directory iteration for "all IDs in ID order" is the
  classic way to pass tests and still be unstable. Order must come from the map
  type.

### Assumptions (to be recorded as ASM in the run)

- **A1** Single-user, single-host, local filesystem with working `flock` and
  `fsync`; Linux is the acceptance platform.
- **A2** The store fits comfortably in memory: whole-store scan for `where` /
  backlinks is acceptable, per the brief's explicit no-hard-total-work-bound note.
- **A3** Public IDs are user-chosen and stable; internal UIDs never appear in any
  output. Identity is never renamed.
- **A4** Stored content is data, not instruction; no trust enforcement (brief).
- **A5** Beta toolchain is acceptable for development; no MSRV promise in v1.

### Shaping decisions already taken (proposed; to become DEC in the run)

- **D1** Crate set: `serde`, `serde_json`, `nanoid`, `fs4`, `clap`. No atomic-write
  crate; no strict-JSON crate.
- **D2** In-memory entities are an ordered map keyed by public ID; uid/path lives
  only in the shell.
- **D3** Pure core = `normalize` / `apply` / `select` / `render`; shell = parse,
  lock, recover, commit, emit.
- **D4** Hand-rolled three-state journal (stage → journal → COMMITTING → apply →
  clear), matched to prior art in `research.md`.
- **D5** Durability is documented as *recoverable batch commit* with a stated
  post-commit/no-response ambiguity — never "atomic".
- **D6** Duplicate JSON keys are rejected during deserialisation by a recursive
  visitor, at the input edge, before any `Value` exists.
- **D7** Deterministic crash injection: a documented env var aborts at named
  protocol boundaries; no timing-based interruption.
- **D8** Determinism comes from ordered collections, never from `readdir`.

### Release decomposition (user-accepted 2026-11-01)

SL-001 as inherited crammed the whole brief into one change, following the
brief's *build* order (normalise → store → commands → query → renderers) rather
than a value order. Agreed decomposition into three releases, each a complete
round trip for the primary user (an agent mid-conversation):

| Release | Capability | Carrier |
| --- | --- | --- |
| 0.1 notebook | write linked thoughts, read them back as Markdown, never lose a batch | SL-001 (re-scoped) |
| 0.2 graph | ask questions across the graph — predicates and traversal | IMP-001 |
| 1.0 presentation | present as Mermaid, document the limits | IMP-002 |

Sizing judgement: 0.1 ≈ 60% of the engine, 0.2 ≈ 25%, 1.0 ≈ 15%. The first is
irreducibly large because the journal cannot be deferred (without it a crash
mid-batch leaves a coherent-looking, silently partial store) and the Markdown
projector is the release's entire payoff.

The user's alternative was to split 0.1 further, shipping the journal and the
Markdown renderer sequentially, to cap the design volume per slice. **Call taken:
keep 0.1 whole.** Reasons: (a) the projector's design content is thin and almost
entirely brief-pinned, so splitting it buys little design headroom while costing
a full lifecycle; (b) splitting at the render boundary inverts the sequencing —
the projection seam (`select` → render) would be retrofitted into the core's
selection result afterwards, i.e. the load-bearing type of inq-1/inq-6 refactored
twice; (c) the run already bounds design volume per slice by reviewing and
attesting each design section separately. Escape hatch: if drafting produces
more than ~six substantive sections, re-open the split at the render boundary
via `/consult` rather than pressing on.

Consequence for the run: `where`/`follow`/`backlinks`/`depth`/`budget` are out of
this slice, so the predicate language is *not* designed here. inq-5 and inq-6 are
narrowed to the ordering and projection halves that 0.1 does need, and a new node
records the 0.1 query-spec surface and its forward compatibility.

## RV-002 integration (reviewing · 2026-11-01)

RV-002 was concluded over revision 37 with three findings raised on it: F-1
(blocker) commit protocol omits the durability barrier for staged recovery
material; F-2 (major) `normalize` cannot enforce existing-ID collision avoidance
through its declared API; F-3 (major) appending diagnostics after verbatim
Markdown can hide incompleteness.

All three verified against the artefact and disposed `design-wrong`, route
`review` — they are defects in the design text, repaired in prose rather than
deferred to an instrument. Repairs landed in revisions 38–42 (stage back to
drafting, eight sections re-declared, materialised at 42) and touched DEC-001,
DEC-003 and DEC-009 in their prose and facets:

- **F-1** — §4.3 requires every staged file and the manifest written and fsynced
  individually, then the prepare directory fsynced, before the rename; barrier
  failure is pre-commit (discard, store untouched), the asymmetry with
  roll-forward-only being explicit. §4.4, §4.6, §10 and §11.1 follow.
- **F-2** — `normalize` takes the entity map it must not collide with; the
  snapshot-aware `IdSource` alternative is recorded as rejected. §3.4, §5,
  §8.1, §9 follow; §11.1 gains the two collision cases.
- **F-3** — the completeness notice leads the document, before any entity
  content, with the boundary stated: smut's own notice cannot be absorbed, and
  stored bodies absorbing each other remains accepted. §8.3, §9, §11.1 follow.

`review.scope` re-read the scope against all eleven decisions and sharpened
three assertions (generation collision set, staged durability, unabsorbable
disclosure); `review.selectors` confirmed §12 commits to the recorded set with
nothing moved. Re-entering reviewing opened **RV-003** over revision 45.

**RV-003** (self-conducted, concluded): probed the five lead seams from
`review.passes` and dissolved all five — §4.4-refuse vs §4.5-roll-forward own
different failures; step 7 delegates the barrier so the first-write path reaches
it; the pre-compute snapshot's only product is the empty-write-set decision; the
notice leads even a zero-entity document; §12 does name the barrier's owner. It
raised one minor: §11.1 disclaimed the barrier-order coverage its own durability
row asserted — repaired at 49, verified.

**The contest worth remembering.** The raiser contested RV-002 F-1: my barrier
recipe said "every staged entity file and manifest.json" and silently dropped the
staged `index.json`, which every commit stages. The general claim above the
recipe said every staged byte; the recipe — the thing an implementer follows — was
wrong. Repaired at 51 in §4.3, §11.1 and DEC-003, with the class swept: the two
numbered recipes and DEC-003's facet were the only narrow enumerations; §4.4 and
§4.6's general claims were already correct.

**What a further pass would probe** (`review.passes`): the three repairs for what
they displaced, and the moved passages since. Specifically — §4.3's barrier
against §5 step 7's commit order, and whether the initialisation path in §5.1
reaches the same barrier; whether the notice-first rule holds when a selection
yields no entities at all, and whether DEC-010's `learn` guide text absorbs the
changed presentation rule; and whether any *other* enumeration of staged files,
fsync points or renderer blocks has the same narrowing that the staged-index miss
had. The §4.4/§4.5 seam and the stale-snapshot seam listed here before RV-003
were probed and dissolved by that pass — see above.

**Planning direction from the user, given before the lock** (recorded here because
it binds `/plan`, not the design text): move to implementation planning rather
than extend design discussion — the scope is bounded, component responsibilities
are clear, and the failure cases carry concrete test obligations. The recovery
protocol is named as the hardest part to implement correctly, so interruption,
**repeated recovery**, and concurrent-initialisation tests take priority over
other test layers. The design's durability claims are accepted as *stated*, not
as *evidenced*: the plan owes implementation evidence for them, and §11.1's
distinction (code-path ordering asserted by injection, power-loss durability
assumed) is the line that evidence has to respect.

## Implementation run

Durable record of the unattended implementation run. Every charter points here
instead of restating it.

### Tiering

pi-subagents replaces the Claude tiering, with the same three layers and the
same discipline (no context above ~250k, the seat reads hand-backs and not code):

| tier | what | model | why |
|---|---|---|---|
| seat | this session, human attached | deepseek-flash | holds escalation and the envelope |
| charter | `capsule-orchestrator` — 1 chartered run of phases | deepseek-flash:high | spawns, verifies gate + commits, records lifecycle |
| phase | `capsule-worker` — exactly one phase | deepseek-flash:high | implements TDD in the main worktree |

Agent definitions live outside the repo at
`~/.pi/agent/agents/{capsule-orchestrator,capsule-worker}.md` (harness config,
not slice artefacts — nothing added to §12's module map). `deepseek-flash`'s 1M
window is why it is the default at both tiers: a Rust phase plus the design
sections it must read does not fit a 272k window comfortably.

**Escalation ladder when a phase resists:** one re-brief of the same worker on
the same model; if the phase fails twice, the charter stops and the seat decides
whether to re-charter on `gpt-6.1-sol:high` (real judgement) or park it. Nothing
below the seat picks a stronger model on its own.

### Division of labour (load-bearing)

- **The worker owns code and commits.** Scoped `git commit -- <paths>` only;
  never `git add .`, never pathless, never `stash`/`checkout`/`reset`.
- **The orchestrator owns doctrine lifecycle and the autonomy log.** It flips
  `in_progress` immediately *before* the worker spawns (stamping `code_start_oid`
  at `HEAD`) and `completed` only *after* the worker's work is committed, because
  the flip stamps `code_end_oid` at `HEAD`. A premature flip records the wrong
  boundary.
- **No worker carries two phases.** One worker, one phase.
- **Workers are spawned foreground.** The tree is strictly serial: nothing else
  in it can make progress while a phase is being built.
- **No worktrees, no confinement, no import step.** Main worktree throughout.

### Escalation

`capsule-worker → contact_supervisor → capsule-orchestrator →
contact_supervisor → seat → human`. Subagents in this harness *can* ask, unlike
the Claude path, so the seat is only reached for decisions that are really the
human's (see the interrupt tier in the orchestrator's envelope).

### Environment (probed 2026-11-01, before the first charter)

- Host gate command, as the orchestrators will run it: `just check` once
  PHASE-01 wires the justfile; until then the equivalent is
  `cargo clippy --all-targets -- -D warnings && cargo fmt --check && just test`.
- Toolchain is on the global PATH already (`cargo 1.100.0-beta.7`, `just 1.58.0`,
  `clippy 0.1.100`, `rustfmt 1.11.0-beta`); no `nix develop` wrapper needed and
  `IN_NIX_SHELL` is unset. `CARGO_TARGET_DIR` is unset.
- crates.io is reachable (`cargo search serde` succeeds), so PHASE-01's five
  runtime dependencies can be resolved.
- Baseline green: `cargo build` and `cargo test` both pass, `0` tests executed.
  Tree clean at `b0eac66` when the run started.
- `doctrine` is on PATH at `/home/david/.local/share/doctrine/bin/doctrine` for
  every tier.
- Harness probe before the first charter: a `capsule-orchestrator` spawned a
  foreground `capsule-worker`, which ran `cargo --version` and
  `doctrine slice phase --help`; nested fan-out works (depth 2 available, 62 of
  64 spawns unspent).
- **Minting ids fails in this checkout** (no `origin` credentials): backlog and
  knowledge items raised during the run are recorded in prose here and minted
  once a remote is reachable.
- **`verify-vt` is red by construction** until the files exist. Run it at each
  phase conclude; treat a red row the phase does not own as expected.
- No sandbox is involved (main worktree, unconfined), so there are no
  sandbox-only failures to work around and no safety rule to widen. The flip
  side is that the durability suite must be told `GIT_GUARD`-style guards apart
  from real failures if that ever comes up.
- No operator-supplied fixtures, credentials or private data are needed for
  0.1; nothing was captured.

### Charter plan

One heavy phase per charter, pairs only for the two pure cores:
`01` · `02+03` · `04` · `05` · `06` · `07` · `09` · `08`.

### Autonomy log

<!-- one line per envelope decision: PHASE-NN · what · why · decided|deferred|escalated -->

- run · seat is pi, tiers are pi subagents, agent defs outside the repo · the
  human asked for this path rather than `/dispatch` · decided
- PHASE-01 · boundary registry given its home as an ungated inline `pub mod
  boundary` in `src/store/mod.rs` rather than a new `src/store/boundary.rs` ·
  keeps the charter's path list exact ("nothing else in src/ yet") while giving
  the registry one home that `journal.rs` consumes in PHASE-05 · decided
- PHASE-01 · rustfmt's default 4-space *source* indent accepted, no
  `rustfmt.toml` added · §4.2's 2-space rule is about the JSON bytes (held:
  `serde_json::to_string_pretty` with the 2-space default), and repo-wide
  formatting style is not this phase's call · decided
- PHASE-01 · `serde` declared without the `derive` feature; `entity.rs`
  implements `Serialize` by hand over `Value` · DEC-007 arranges keys at the
  final serialisation boundary, so derive was not needed; adding the flag later
  is a feature on a declared crate, not a new dependency · deferred to PHASE-02
- PHASE-01 · `Cargo.lock` is modified by the Cargo.toml change but §12's
  design-target selectors (`src/**`, `tests/**`, `docs/**`, `README.md`,
  `Cargo.toml`, `justfile`) omit it, so `slice conformance` reports it
  undeclared · a §12 errata of the same kind as the README and lib-target ones
  already recorded in `plan.md`, not a phase defect · deferred to `/audit` →
  `/reconcile`
- PHASE-01 · `--help` advertises only `--help`/`--version` · the 0.1 grammar
  lands with `src/cli.rs` in PHASE-07 (EX-7 says that phase completes the
  grammar), and help text listing unwired commands would lie · decided
- seat · read-only intent at the seat briefly issued `doctrine slice phase SL-001
  PHASE-01 --status in_progress` while PHASE-01 was already `in_progress`, which
  appended a second, note-less `in_progress` progress row at 05:44:06 · the
  transition was idempotent (`code_start_oid` unchanged at `6e2a1f1`), so only
  the log gained a no-op line; the read-only verbs at this altitude are
  `doctrine slice status SL-001` and `doctrine slice phases` · decided
- PHASE-02 · `serde`'s `derive` feature stays off (PHASE-01 deferred the call
  here) · responses are assembled as `serde_json::Value` and every byte form comes
  from PHASE-01's hand-written wrapper, so nothing in this phase needs derive and
  `Cargo.toml` is untouched · decided
- PHASE-02 · the declaration vocabulary (`Decl`, `Op`, and the DROP-sentinel
  conversion) lives in `src/entity.rs` with `extract`, and `src/declare.rs` adds
  the fold · keeps the dependency direction `declare → entity`, which matches
  DEC-005's "normalize is built on extract" · decided
- PHASE-02 · a top-level empty object `{}` declares nothing — no id generated,
  no entity created · generating an id for an object with no fields would report
  a phantom `missing` reference, and the brief's "objects lacking IDs receive
  NanoIDs" is about declarations, of which `{}` contains none · decided
- PHASE-02 · `scan_refs` (malformed nested `id`) and `apply` (non-object snapshot
  value) panic rather than silently repair · both are preconditions of a
  shell-validated `Entities` snapshot (DEC-001), and the store loader owns turning
  either into exit 1; the alternatives (skip, or start over) lose corruption or
  invent data · decided
- charter-2 · PHASE-02's charter authorised `src/error.rs` as a fifth, additive
  path beyond the plan's four (`src/entity.rs`, `src/id.rs`, `src/declare.rs`,
  `src/lib.rs`) · EX-4 and EX-5 are stated in terms of `Reject` for conflicting
  batches and for null/numeric/empty ids, and `Reject` lives only in
  `src/error.rs`; three variants, their `Display` arms and the exit-code table
  rows were added, while `StoreError` and the two-class `Error` mapping stayed
  untouched, so no error class was introduced · decided
- charter-2 · the PHASE-02 worker wrote its own four `## Autonomy log` lines and
  committed them as `7adefa4`, which the Implementation run assigns to the
  orchestrator · the lines are substantive and correct, so they stand; recorded
  here so the division of labour stays visible rather than being re-litigated
  mid-run · decided
- charter-2 · PHASE-02's response objects serialise their keys lexically
  (`changes`, `ids`, `missing`) where the brief's example shows a different
  order · JSON key order is insignificant and §4.2/DEC-007 make the canonical
  stdout form lexical apart from the id-first entity root, so the example is
  illustrative; if the brief's display order is ever wanted it is a PHASE-07
  question, not a core change · deferred to PHASE-07
- charter-2 · PHASE-03 authorised `src/error.rs` additively again, for six new
  `Reject` variants (unsortable value, not-a-specification, unknown order member,
  future spec key, unknown spec key, invalid spec value) · DEC-011's rejections
  are stated in terms of `Reject` and there is no other home for them; the
  two-class `Error` mapping and `StoreError` stayed untouched · decided
- charter-2 · `Limit` is an opaque struct over a private enum, with the cap lifted
  only by a `pub(crate)` constructor that `get` uses · a public `Unlimited`
  variant would let any caller assemble a defaulted-and-bypassed `Spec`, which is
  exactly what DEC-001 rule 4 forbids; the crate-visible constructor keeps the
  bypass internal as the rule states · decided
- charter-2 · `from: ""` is accepted as a necessarily-missing root instead of
  being rejected · DEC-005 makes an empty string invalid only where an id is
  *read* as one; the spec surface DEC-011 fixes names no such rejection and
  inventing one would narrow it · decided
- charter-2 · the PHASE-03 worker spotted that `src/entity.rs`'s canonical
  wrapper serialises nested objects lexically, so a projected entity inside an
  envelope's `objects` array is not id-first, though DEC-007 wants an entity root
  id-first wherever it appears · the fix lands at the byte boundary with the
  owner of `src/entity.rs` once the CLI projects into an envelope, not in the
  selection core · deferred to PHASE-07
- charter-3 · PHASE-04's commit step in the seam refuses with
  `StoreError::NotYetImplemented` instead of doing nothing · a silent no-op would
  let a command that asked for a write report success without publishing, and
  would hide the fact that PHASE-04 has no index writer; refusing keeps "no code
  path publishes an index" structural and makes a `declare` against a real store
  exit 1 until PHASE-05 fills the body (which deletes the variant) · decided
- charter-3 · initialisation's parent-directory fsync is labelled
  `Boundary::ParentDirectorySynced` instead of a new registry name · the registry
  has no initialisation-parent name and the charter forbids adding one here, and
  it is the same durable step on the same directory; PHASE-05 owns initialisation
  interruption tests and may append a distinct name when it needs to distinguish
  the two · decided
- charter-3 · `StoreError` gained its refusal vocabulary (absent, locked, unknown
  format version, unexplained entry, symlinked entry, id disagreement, corrupt,
  not-yet-implemented) with `Error`'s two-class mapping untouched · all are
  exit-1 store failures, which is the additive pattern PHASE-02/03 set for
  `Reject`; no third error class and no exit-code change · decided
- charter-3 · a top-level `.smut/manifest.json` is refused as an unexplained
  entry · §4.1 reserves the manifest name for a manifest *inside* a journal state
  directory and DEC-004's explained list (lock, journal states, `index.json`,
  mapped files, the reserved temporary namespace) excludes a top-level one, so a
  stray one is evidence of the tool's own namespace being violated, not noise ·
  decided
- charter-3 · nested stored entity-node ids stay unvalidated · §5.2's list and
  EX-5 name only "each file's `id` equal to its index key", so validating nested
  reference shapes would invent an obligation outside this phase's bar; PHASE-02's
  recorded "the store loader owns it" stands as the target, and the concrete risk
  (a corrupt-but-index-consistent nested `id` reaching a `scan_refs` panic in
  PHASE-07) is handed to `/audit` → `/reconcile` as a backlog candidate ·
  deferred
- charter-3 · a symlinked `.smut/lock` would still be followed by
  `OpenOptions::create(true)` (no `O_NOFOLLOW`) · EX-5 names entity files and
  `index.json` for the symlink refusal, so hardening the lock is outside this
  phase's bar and is recorded as a backlog candidate · deferred
- charter-3 · duplicate `objects` *keys* inside `index.json` collapse at
  `serde_json` parse, so only distinct-ids-mapping-to-one-file duplicates are
  detectable · EX-5's "duplicate path mapping" is that detectable case; refusing
  duplicate JSON keys needs the custom visitor DEC-006 scopes to declaration
  input, so it is a backlog candidate rather than a silent widening · deferred
- charter-3 · `fs4` 1.1.0 has no `fs4::fs_std::FileExt` (the path the charter
  named from memory); the lock uses the crate-root `fs4::FileExt` via UFCS ·
  std's inherent `File::lock`/`try_lock` (Rust 1.89+) would otherwise win and
  leave `fs4` unused, which is the one thing the charter asked to avoid; `fs4` is
  no longer reported as an unused dependency · decided
- charter-4 · PHASE-05 is one worker, one phase, and the crash suite was not
  truncated to fit a budget · the user named this phase as the hardest and the
  plan's risk section makes the boundary enumeration structural, so a shorter
  suite would have been the failure mode the phase exists to prevent · decided
- PHASE-05 · appended only `InitialisationParentSynced` to the single registry
  (COMMIT_PATH is now 15) · with the shared `ParentDirectorySynced` label an
  abort fired inside `initialise` before anything was staged, so the first-write
  interruption test could not distinguish "the store directory survived" from
  "the decision is durable" · decided
- PHASE-05 · §4.3's two unnamed `.smut` fsyncs (post-apply and post-done) reuse
  `RemovalsApplied` / `DoneRenamed`, and recovery's two reuse
  `RecoveryDoneRenamed` · all four are post-decision steps with the same
  roll-forward reaction, so a distinct name would duplicate an identical test
  rather than add coverage · decided
- PHASE-05 · recovery fsyncs `.smut` after replay and before its `commit → done`
  rename · §4.4's recipe ("replay, rename to `done`, fsync, clear") omits that
  step, which would risk keeping the state transition while losing the replayed
  bytes; the addition mirrors §4.3's apply rule · decided
- PHASE-05 · `src/store/fs.rs` gained `remove_directory` (plus a test-only
  `Call::RemoveDirectory`) · unlinking a state directory had no existing site and
  `fs.rs` must stay the crate's only removal site · decided
- PHASE-05 · deleted `StoreError::NotYetImplemented` with the `commit` stub, its
  `Display` arm and the three tests that referenced it · discharges charter 3's
  record; a surviving variant would have been a failed criterion, not tidy-up ·
  decided
- PHASE-05 · `StoreError` gained `AmbiguousJournalState` and `InvalidWriteSet`, and
  a manifest version mismatch reports as `Corrupt { entry: "manifest.json" }` ·
  the additive pattern PHASE-02/03/04 set for `Reject`; the two-class `Error`
  mapping and the exit-code table are untouched, and `UnknownFormatVersion`'s
  message is about the index · decided
- PHASE-05 · `tests/durability_crash.rs` carries one file-level
  `#![allow(non_snake_case)]` · the `interrupted_at__<boundary>` and
  `recovery_interrupted__<step>` names are exactly what EX-6 and EX-9 make
  greppable, so rustfmt's convention gives way to the criterion · decided
- PHASE-05 · `Store::load` treats a *regular file* named `journal.commit` as an
  explained entry, so a direct `load()` accepts what recovery refuses as corrupt ·
  the refusal lives on the pipeline path (`run` → `recover` before `load`), which
  is the only path a command takes; a file-shaped journal state is a backlog
  candidate rather than a widening of this phase's bar · deferred
- PHASE-05 · closed `completed` with `VH-1` recorded outstanding · VH-1 is a
  human read of evidence that now exists (22-row boundary table, seven
  `recovery_interrupted__*` tests, the one-line claim at
  `tests/durability_crash.rs:18`), and the seat carries it to the human; the
  phase is not complete-with-a-hole, it is complete-with-a-review-pending ·
  decided
- charter-5 · PHASE-06 stayed on the plan's exact paths — `src/render_markdown.rs`
  and `tests/render.rs` new, `src/lib.rs` one `pub mod` line — and left
  `src/error.rs` untouched · the renderer refuses nothing: its one precondition
  (`truncated` implies at least one reason) is `select`'s own invariant and is
  asserted the way `project_one` asserts its precondition, so no `Reject`
  variant was owed · decided
- PHASE-06 · `render_markdown::Options` (`heading_field`, `heading_level`,
  `show_fields`, `show_ids`; default `title`/2/false/false) is the renderer's
  input, and `--heading-level`'s 1–6 rejection stays PHASE-07's · EX-1 puts the
  layout at the renderer while PHASE-07's EX-1 explicitly owns the flag's parse,
  so the renderer repeats `#` the given number of times and validates no range ·
  decided
- PHASE-06 · the document is one flat block list joined by a blank line — the
  notice block, then each entity's blocks — so "one blank line between
  entities" and "the notice is the first block" are the same join rather than
  two rules that could drift apart · decided
- PHASE-06 · `--heading-field id` renders the id as heading text, attributes
  iterate the entity's own lexical key order and honour `select`, and `id` is
  never an attribute · not named by the bar; identity is always selected
  (DEC-001 rule 8) and DEC-007 fixes the lexical key order, so the two follow
  from records rather than inventing a rule · decided
- PHASE-06 · VT-1…VT-3 read `UNATTRIBUTABLE — keyword present but <file> not
  modified by this slice` while the phase was `in_progress`, and resolved to
  PASS on the `completed` flip that captured `code_end_oid` · attribution walks
  the phase's recorded `[code_start_oid, code_end_oid)` span, so an
  `UNATTRIBUTABLE` row mid-phase is the open boundary working, not a red row ·
  decided
- PHASE-06 · the worker's note that `entity::Form::Stdout` already appends a
  trailing newline while `render` ends the document in one is handed forward
  rather than fixed here · the shell is PHASE-07's and printing a third newline
  would be that phase's defect · deferred to PHASE-07
- PHASE-07 · the trailing newline is settled at the byte boundary and nowhere
  else: `prepare_output` returns bytes that already end in exactly one `\n` (or
  are empty), and `main` writes them with `print!`, never `println!` · a third
  newline was the defect PHASE-06 handed forward, and demoting the rule to “the
  shell never adds one” makes it unrepresentable rather than remembered · decided
- PHASE-07 · `clap`'s `derive` feature is enabled in `Cargo.toml` · DEC-008 names
  clap v4 derive; a feature flag on an already-declared crate is not a new
  runtime dependency, by the same reasoning PHASE-01/02 used to keep `serde`'s
  derive off · decided
- PHASE-07 · `Render` renders inside `recompute` and returns the document through
  `prepare_output` unchanged, where `query`/`get`/`delete` return a `Value` that
  `prepare_output` serialises · `Selection` borrows the snapshot, so it cannot
  outlive `recompute`, and rendering is output preparation either way — both run
  under the lock, which is what §5.3 requires · decided
- PHASE-07 · response and envelope keys stay lexical rather than following the
  brief's example display order · the brief's ordering is illustrative, JSON key
  order is insignificant, and DEC-007 fixes canonical stdout as lexical apart
  from the entity root; this discharges charter-2's deferral · decided
- PHASE-07 · `--heading-level` carries `allow_hyphen_values` so `-1` and `9`
  reach the shell's own 1–6 refusal · without it clap answers “unexpected
  argument”, and EX-1 makes the range check a `Reject` rather than a clap error ·
  decided
- PHASE-07 · six `Reject` variants added additively (duplicate key, trailing
  input, malformed JSON, missing input, invalid heading level, future/unknown
  format) with `StoreError` and the two-class `Error` mapping untouched · the
  pattern PHASE-02/03/04/05 set; every one is exit 2 · decided
- PHASE-07 · `input::read_stdin` splits its two failures: an IO error is a
  `StoreError` (exit 1, a runtime failure) while bytes that are not UTF-8 are a
  `Reject` (exit 2, no JSON value is recoverable from them) · the brief's exit
  table names the classes, not the layer that noticed · decided
- PHASE-07 · `Store::run` is untouched; the seam needed no new parameter, so the
  charter's conditional path on `src/store/mod.rs` went unused · steps 1 and 11
  really were the caller's, as PHASE-04 documented · decided
- PHASE-07 · the deferred byte-boundary defect (a projected entity inside an
  envelope's `objects` array serialises lexically rather than id-first, DEC-007)
  is NOT fixed here · the charter lists `src/entity.rs` among the settled parts
  to use rather than restructure and does not include it in this phase's paths,
  so the fix returns to `/audit` → `/reconcile` with a backlog candidate instead
  of widening the phase · deferred
- PHASE-07 · `print!` to a closed stdout (`smut query | head -1`) panics with
  exit 101 rather than dying of SIGPIPE · pre-existing in PHASE-01's skeleton and
  outside this phase's bar; §4.5 documents the post-commit ambiguity, not a
  panic, so it is a backlog candidate rather than a silent widening · deferred
- PHASE-07 · `src/learn.rs`'s honest first pass turns PHASE-08's VT-2 green
  already (`DROP` and `0.2` present) · that is keyword attribution against a file
  PHASE-08 owns and will rewrite, so the audit should not read it as PHASE-08
  work done · decided
- charter-6 · `doctrine slice phase SL-001 PHASE-07 --status completed` was
  issued twice (the second while re-reading the boundary warning) · the
  transition is idempotent: `code_end_oid` stayed `585096d` and the registry row
  is unchanged, so only a note-less `completed` row was appended; the warning's
  ten commits are all this phase's own, so no `record-delta` tightening was
  needed · decided
- charter-6 · VA-1 discharged by reading the diff rather than by a test: the
  duplicate-key check is inside `input`'s visitor at each `next_key` (before the
  second value is even read), installed at `input::parse`, called from
  `cli::operand`/`cli::parse`, which `main::execute` reaches before the only
  `Store::in_directory(..)` sits in the `Invocation::Request` arm; `Learn`, `Help`
  and `Version` are `Invocation` variants, so no store is constructible on their
  path · decided
- seat · envelope key ordering for entity roots inside `objects` is left for
  `/audit` rather than ruled here · DEC-007 says "entity serialisation at the
  final boundary through one shared Entity wrapper; id-first root", while the
  envelope path emits projected `Value`s lexically: entity *files* are id-first
  (`id, body, kind, title`) but `query`/`get` objects are not
  (`{"body":"First.","id":"smut"}`). The brief's response examples put `id`
  first, the design's mechanism clause says one shared wrapper — but reading
  either way changes a documented output byte, which is a conformance question
  for a review, not a seat improvisation. Evidence below · deferred
- PHASE-09 · `stale_empty_snapshot_precheck` is proved at the seam with a
  test-defined `Command`, not with the real `Declare` · the candidate ids the
  real command draws come from `nanoid` inside the binary and cannot be scripted
  from an integration test, so a collision cannot be forced through it; what the
  criterion names is `Store::run`'s own contract — step 2 against an empty
  snapshot, step 6 against the loaded one — and the store is filled in between by
  a real spawned child, so the precheck is provably stale rather than
  hypothetically so · decided
- PHASE-09 · retry bounds: `ATTEMPTS = 3` around
  `two_simultaneous_first_declarations` with the identical assertion set on every
  attempt, and **no** retry at all for the other three scenarios · repetition
  varies timing, it is never "retry until green", and the lock-gated scenarios
  are deterministic because the parent holds the lock before the child exists ·
  decided
- PHASE-09 · the JSON envelopes in `tests/compatibility.rs` are pinned
  structurally (parsed `Value` equality, key sets, exit code, stream discipline)
  while `render` is pinned as exact text · entity-root id-first ordering inside
  an envelope's `objects` is the open `/audit` question two entries above, so a
  byte pin on envelope key order would pre-empt a decision that is not this
  phase's · decided
- PHASE-09 · the cross-process recovery scenario fabricates a decided-but-unapplied
  `journal.commit/` from the public helpers instead of killing a real process
  mid-commit · a real process cannot be stopped between the commit rename and
  apply without the `fault-injection` feature, which these three suites must not
  be gated behind; the fixture is built the way `tests/durability_crash.rs`
  builds one, and a direct `load()` assertion proves it really is unrecovered ·
  decided
- PHASE-09 · no `src/` change, and therefore no re-export: every seam was
  reachable from the public surface (`cli::parse` → `Request::run`,
  `store::{Command, Computed, Snapshot}`, `declare::WriteSet`,
  `id::{IdSource, Occupied}`, `journal::{COMMIT_STATE, MANIFEST_FORMAT_VERSION}`,
  `index::Index`, `entity::serialise_entity`) · the charter preferred no library
  widening and none was needed · decided
- PHASE-09 · one pin beyond the criterion's "at least" list — the brief's own
  end-to-end acceptance example, for the steps 0.1 can run · it is the strongest
  compatibility statement available, and its `follow`/`where` steps are pinned as
  the refusals, so a 0.2 widening cannot silently change either half · decided
- PHASE-09 · carried forward, not fixed: `Store::load()` treats a
  *directory*-shaped live journal state as an explained entry, so a direct
  `load()` returns the pre-decision state silently (PHASE-05 recorded the
  file-shaped variant) · `src/store/mod.rs` is not this phase's path and the
  refusal lives on the pipeline path every command takes; the concurrency
  scenario's "the fixture is unrecovered" assertion rests on the current
  behaviour and would invert if `/audit` rules that `load()` must refuse ·
  deferred to `/audit`
- PHASE-08 · one file beyond the plan's three — `tests/readme_example.rs` — added
  to keep EX-4 true after the phase: it reads the README's single `console`
  block out of `README.md`, runs each command in a scratch store, and compares
  JSON as parsed values (never bytes, since the entity-root ordering inside
  `objects` is open for `/audit`), `render` as exact text, and a `smut: ` line as
  exit-non-zero plus that stderr · the charter invited the file and asked for
  this line; it is the only `.doctrine` write this worker made · decided
- charter-8 · PHASE-08 carried exactly one worker on the plan's three paths plus
  the invited `tests/readme_example.rs`, and no re-brief · the worker's gate
  (349 default / 406 fault-injection, +2) and the doc claims were re-checked by
  the orchestrator: the worked example re-run end to end against
  `target/release/smut` in a fresh `/tmp` store, byte-identical to the README's
  `console` block; the §4.6 guarantee `diff`ed against `design.md`, `README.md`
  and `docs/store-format.md` (three-way identical); the brand copy's first 383
  bytes `cmp`ed against `bb5de77:README.md` · decided
- PHASE-08 · the ID-first envelope question is left exactly as PHASE-07 left it,
  and the phase is checked for not foreclosing it: `tests/readme_example.rs`
  compares a JSON transcript line as a parsed `Value` and only `render` as
  text, and neither `README.md` nor `docs/store-format.md` states a stdout key
  order (the format doc's canonical-bytes section scopes ordering to *files*) ·
  the review must still rule; the documentation regenerates by being re-run ·
  decided
- PHASE-08 · the README's worked example is the brief's end-to-end example
  truncated where 0.1 stops, and it ends by *showing* the `follow` refusal as
  the boundary · the brief's nine steps use `follow` twice and `where` once,
  which 0.1 refuses by name, so running it verbatim is impossible; the brief is
  untouched, `tests/compatibility.rs` already pins the 0.1-runnable prefix and
  both refusals, and no example claims behaviour this build lacks · decided
- PHASE-08 · closed `completed` with `VH-1` recorded outstanding · VH-1 is a
  human read of evidence that now exists (the README's sections, the copied
  worked-example command sequence, the design's own guarantee, and one code
  location per sharp edge), and the seat carries it to the human as it did
  PHASE-05's; the phase is complete-with-a-review-pending, not
  complete-with-a-hole · decided

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · **audit complete** (RV-004: 17 findings — 3
`fix-now` landed, 5 `design-wrong` to reconcile, 6 `tolerated`, 2 `aligned`, 2
human reads outstanding; 28/28 verification rows PASS) · **4 post-audit
packaging commits** (`2e6cfe7` GPL-3.0-only + crates.io metadata, `094005f`
README image, `105db99` notes) · head `105db99`

### Produced

- PHASE-01 landed the crate spine (9 commits, `e0ce040…397c5c8`); PHASE-02 the
  declaration core (9 commits, `614ea33…7adefa4`); PHASE-03 the selection core
  (7 commits, `e08bb7f…3fc8086`); PHASE-04 the store format, index, lock,
  validation and pipeline seam (5 commits, `8403ac1…49d81d7`); PHASE-05 the
  journal, its recovery and its interruption evidence (7 commits,
  `7d841fd…0e2a519`); PHASE-06 the Markdown renderer and its notice evidence
  (6 commits, `5d3af21…f5d0773`); PHASE-07 the clap grammar, ingress, pipeline
  wiring, envelopes, delete contract and learn guide (10 commits,
  `212e574…585096d`); PHASE-08 the documentation set (6 commits,
  `75c8471…54440dc`). Every path §12 names is now delivered.
- `just check` green at every close; after PHASE-07: 276 lib + 6 bin + 26
  `tests/cli_contract.rs` + 5 `tests/render.rs` + 5 `tests/selection.rs` + 12
  `tests/store_corruption.rs` + 0 (default-gated) durability = **330** on default
  features, and 289 + 6 + 26 + 44 + 5 + 5 + 12 = **387** with `--features
  fault-injection`. The durability target executes 44 tests under the feature and
  0 by construction under the default build.
- `slice verify-vt` green for every row PHASE-01…PHASE-07 own (23/23); PHASE-08's
  VT-2 also reads PASS because `src/learn.rs` exists with its keywords, which is
  attribution rather than PHASE-08 work done.
- The binary is the brief's: `src/cli.rs` (clap v4 derive grammar, the five
  commands as `store::Command`, `--` for opaque ids, `--heading-level` refused by
  our own 1–6 check, `--format mermaid` refused naming 1.0), `src/input.rs` (the
  pure operand/tty decision, stdin, and a `serde` `DeserializeSeed` visitor that
  refuses a duplicate key as `next_key` yields it), `src/main.rs` (dispatch;
  `learn`/`--help`/`--version` are answered as `Invocation` variants, so no store
  is constructible on their path), `src/learn.rs` (`GUIDE`, honest first pass).
  `Store::run` needed no new parameter.
- `src/render_markdown.rs` renders a `Selection` from `Options { heading_field,
  heading_level, show_fields, show_ids }`: heading only when the field is
  selected, a string and non-empty (newlines collapsed); body verbatim and
  untrimmed; attributes only under `show_fields`; `code_span` fences with
  longest-backtick-run + 1 and pads at the edges; the visibility fallback; and
  the completeness notice as the first block of a flat block list. No refusal,
  no filesystem, no clock, no entropy.
- The protocol now exists end to end: `src/store/journal.rs` drives
  `prepare → commit → done` through `store::fs` only, `Store::run` recovers before
  validating and publishes a write set, and the single boundary registry has 22
  names (15 commit path, 7 recovery path). Recovery refuses ambiguity, removes
  only reserved temporaries, validates a commit state completely and replays it
  totally.
- `StoreError::NotYetImplemented` is gone with the stub, so no code path refuses
  a write any more: the store publishes.
- `slice conformance`: 16 declarations conformant; two undeclared (`Cargo.lock`,
  `.doctrine/slice/001/notes.md` — neither is in §12's selector list) and the two
  deliberately undelivered targets.
- serde `derive` settled **off** for the run so far: responses are assembled as
  `serde_json::Value` and every byte form comes from PHASE-01's hand-written
  wrapper. `clap`'s `derive` **is** on: DEC-008 names it, and a feature flag on a
  declared crate is not a new dependency. Both declared-and-unused warnings are
  gone (`nanoid` from PHASE-05's uid generation, `clap` from PHASE-07).
- PHASE-09 landed the three end-to-end seam suites (4 commits,
  `e9f9cda…9e41e47`), `tests/roundtrip.rs`, `tests/concurrency.rs` and
  `tests/compatibility.rs` — **no `src/` change at all**. `just check` after it:
  276 lib + 6 bin + 26 `cli_contract` + 9 `compatibility` + 4 `concurrency` + 0
  (default-gated) durability + 5 `render` + 4 `roundtrip` + 5 `selection` + 12
  `store_corruption` = **347** default, and 289 + 6 + 26 + 9 + 4 + 44 + 5 + 4 + 5
  + 12 = **404** with `--features fault-injection`; none of the three new suites
  is feature-gated and none is `#[ignore]`d. 25 consecutive runs of the three:
  zero flakes, ~40 ms per run. `slice verify-vt` green for PHASE-01…PHASE-07 and
  PHASE-09; PHASE-08's VT-1 red only because `docs/` does not exist yet.

- PHASE-08 delivered the prose (6 commits, `75c8471…54440dc`): `README.md`
  edited around the owner's 383-byte brand block and `smut.jpg` reference (byte
  `cmp`-identical) with `## What it is`, `## Build`, `## Worked example`,
  `## Commands`, `## Sharp edges`, `## What smut promises`, `## Portability`,
  `## Learn`; `docs/store-format.md` (303 lines) carrying `format_version` 1, the
  layout, the index, entity files, the manifest, canonical bytes, the state-as-name
  protocol with the staged barrier as three numbered obligations, puts→index→
  removals, roll-forward-only apply failure, recovery's three steps and complete
  `commit` validation, reserved names, corruption responses with the build's real
  messages, what is never adopted, portability, and §4.6 quoted verbatim; and
  `src/learn.rs`'s guide completed for the 0.1 surface. Every claim in both
  documents was produced by running the binary in a scratch store, and the
  README's transcript is now pinned by `tests/readme_example.rs`.
- `just check` green at close: **349** default and **406** with
  `--features fault-injection` (the two suites of `tests/readme_example.rs` are
  the only addition). `slice verify-vt`: 9/9 phases, every row PASS.
- Design locked: run rev 53, 12 sections attested, RV-002 and RV-003 terminal
  (commits 1b1f574, c32f316, f28860e).
- Plan authored, then revised after an independent review: 9 phases,
  PHASE-01…PHASE-09 (commits 5224e4a, f16167d).
- Errata recorded in `plan.md` rather than silently repaired: design §12 calls
  `README.md` new while owner copy pre-exists; §12 lists no lib target while
  §11.1's `tests/**` layers need one.
- Research baseline re-stamped after the scope and design drift.
- Verification gate: `doctrine check gate` is N/A (no code yet); `doctrine
  validate` clean; `slice verify-vt` red by construction until phases land.
- `.doctrine` committed with the artefacts, never separately.
- **At audit entry, seat-verified on `eb26398`:** `just check` exit 0 with **0
  warnings**, **349** executed default / **406** with `--features
  fault-injection`; `slice verify-vt` **28 PASS / 0 FAIL** across 9 phases;
  `slice conformance` **32 conformant, 2 undeclared** (`Cargo.lock`,
  `.doctrine/slice/001/notes.md`), **0 undelivered**; `doctrine validate` clean;
  tree clean; 77 commits since `b0eac66`. Boundary table for every phase is in
  `.doctrine/state/slice/001/boundaries.toml` (gitignored runtime state).
- **The audit (RV-004) landed three fixes**, each its own commit and each with a
  test that fails without it: `78a7eb1` — the serialisation wrapper is now
  structural (an entity node names `id` first *wherever it appears*), so a
  projected entity in a response's `objects` array serialises exactly as its file
  does, and `serialise_entity`/`serialise_document` collapsed into one
  `serialise`; `35b62ed` — the shell writes prepared bytes fallibly, so a closed
  stdout exits 1 instead of panicking with 101; `dfc6e37` — `gate: check` and
  `prove: check` in the justfile, so `doctrine check gate`/`prove` answer at all.
  After them: **354** default / **411** with `--features fault-injection`,
  `just check` green with 0 warnings, `verify-vt` 28/28, `doctrine check gate`
  and `prove` exit 0.
- The README's `console` transcript was regenerated from the release build after
  `78a7eb1` (`tests/readme_example.rs` compares its JSON lines as parsed values,
  so only the bytes moved), and `docs/store-format.md`'s canonical-bytes section
  now says the wrapper's subject is the entity node, not the root position.
- **RV-004 is the audit's record**: `.doctrine/review/004/` carries the 17
  findings, the `## Synthesis` and the `## Reconciliation Brief`. Six findings are
  `tolerated` with their rationale on the ledger (`Store::load`'s laxness, nested
  stored ids, duplicate index keys, the symlinked lock, and both conformance cells
  the audit declines to "fix" by widening a selector), two are `aligned`
  (`spec.rs`'s named-future refusal, empty `dangling`), and five are
  `design-wrong`: §12's omission of `Cargo.lock`, the two §12 path errata, the
  three unnamed `.smut` fsyncs, the brief-example boundary, and the
  §4.2/DEC-007 clarification the serialisation fix rests on.

### Learned

- (unminted) A "stale precheck" is provable without scripting the binary's
  entropy: `Store::run` calls `Command::recompute` twice, so a test-defined
  `Command` (the trait is public) can use its first call — the empty snapshot —
  to spawn the real binary and let a second process commit the candidate id, and
  its second call to generate past the now-occupied id with a scripted
  `IdSource`/`Occupied`. The collision is forced, not hoped for.
- (unminted) "Termination before the first lock acquisition" needs no timing: the
  parent takes `Store::lock()` *before* spawning the child, so the child cannot
  have acquired it however far it got; kill it, and every invariant (no partial
  batch, no journal state, no `.tmp`, next command converges) holds for all
  interleavings.

- mem_01a1959995dc7611882307746b4495fd — six recurring review finding classes:
  obligation-versus-API, absorbable guarantee, recipe narrower than the claim
  above it, vacuous cfg-gated suite, tests needing a lib target, unmeetable
  criteria.
- (unminted — id minting fails in this checkout) `fs4` 1.1.0 exposes `FileExt` at
  the crate root, not the pre-1.0 `fs4::fs_std` path; call it by UFCS
  (`FileExt::lock(&file)`) or std's inherent `File::lock`/`try_lock` (Rust
  1.89+) shadows it and the crate is reported unused.
- (unminted) A "this function performs no writes" claim is testable without
  instrumentation: `src/store/fs.rs`'s `#[cfg(test)] call_record` records every
  write, fsync, rename and removal, so `take()` around the call and asserting the
  record is empty is real evidence rather than a comment.
- (unminted) A `cfg`-gated integration suite can be kept from passing vacuously
  without a runtime count assertion: have the suite read its own source with
  `include_str!` and require one named test per registry entry. Adding a boundary
  without a test then fails a test, which is stronger than a printed count.
- (unminted) The proven way to crash a protocol mid-flight from a test without
  killing the harness is to re-exec `std::env::current_exe()` with
  `--exact <test path>` and the abort variable set, then assert the child died by
  signal. Ten-plus boundary tests can share one driver this way.
- (unminted) A "the notice leads the document" guarantee is testable without
  parsing Markdown: `document.split("\n\n").next()` is the first block, so
  asserting it equals the notice, plus `starts_with`, plus
  `find(body) > find(notice)`, tests *position* rather than the notice's text —
  and moving the notice after the content fails it. Verify the negative control
  once by moving the code, then revert.
- (unminted) Safe code spans per CommonMark: fence = longest internal backtick
  run + 1, and pad with one space when the content begins or ends with a
  backtick (or both begins and ends with a space); a fixed single-backtick span
  cannot delimit opaque ids.
- (unminted) `verify-vt` reports a live phase's own rows `UNATTRIBUTABLE —
  keyword present but <file> not modified by this slice` until the `completed`
  flip records `code_end_oid`; the rows resolve on the flip, so an
  `UNATTRIBUTABLE` row mid-phase is the open boundary, not a failing row.
- (unminted) `serde_json`'s `Map` is a `BTreeMap` here, so a duplicate key in the
  input text is silently overwritten by any ordinary `from_str` — the only place
  to catch it is a `DeserializeSeed`/`Visitor` pair whose `visit_map` checks each
  `next_key` against the keys already inserted, before `next_value_seed` reads the
  second value. Carrying the offending key out of serde's opaque error type needs
  a side channel (a `RefCell<Option<String>>` the seed holds) — serde's
  `de::Error::custom` will not surrender a payload.
- (unminted) A terminal-only behaviour stays honest without a pty by making the
  *decision* a pure function of `(operand present, operand is -, stdin is a tty)`
  and injecting `std::io::IsTerminal` from `main`; the `Refuse` branch is then a
  `Reject` with a unit test per branch, and the pipe branch is proven end to end
  by a spawned binary whose stdin is a pipe.
- (unminted) A documentation phase is testable as a phase: parse the README's
  `` ```console `` block out of the file itself, run each `$ `-prefixed command in
  a scratch store, compare a JSON line as a parsed `Value` and a `smut: ` line as
  (exit≠0, empty stdout, exact stderr). The prose then cannot drift from the
  binary without a test failing, and comparing parsed values keeps an open
  key-ordering question un-pre-empted.
- (unminted) Two guide claims that read plausibly were false until run: 0.1 does
  not report a deleted reference target as `missing` (the container keeps
  `{"id":"x"}` and both `missing` and `dangling` stay `[]`, because nothing
  traverses), and `smut render '{"from":"smut"}'` prints a bare heading when the
  entity has no body. Running the examples is what found them.
- (unminted) The fault seam's arming is **thread-scoped** (a thread-local
  override over a process-wide, once-read environment). A process-wide seam
  behind a mutex flaked when the feature suite ran in parallel: one test's
  armed boundary was reached by another's write. The thread-local form removes
  the class rather than serialising around it, and `SMUT_FAULT_*` stays the
  cross-process door the durability suite uses.
- (unminted) `std::env::set_var` is `unsafe` in edition 2024, so a fault seam
  cannot be configured by mutating the environment from a test: make the
  configuration a value type with a pluggable lookup, and no test needs
  `unsafe`.
- (unminted) cargo's own `unused_dependencies` lint is a *manifest* warning, not
  a rustc lint — `clippy -D warnings` does not fail on it, so "0 warnings" can be
  true while `Cargo.toml` still declares a dependency nothing calls yet. Check
  the two separately.
- (unminted) Mutation probes are worth running and reverting inside a review, not
  only a phase: PHASE-06 reverted an appended notice and a fixed single-backtick
  fence to confirm the tests fail, which is the cheap negative control for "the
  notice leads" and "the fence is safe".
- (unminted) A conformance cell can be a **measure artefact**. `design-target`
  selectors model the design's predicted *implementation* surface; a slice's own
  `notes.md` legitimately reads `undeclared` in every doctrine slice, so widening
  the selector set to silence it would misdescribe the design.
- (unminted) `review new` — and every minting verb — needs a writable remote.
  With no push credentials, `[reservation] reach = "local"` in
  `.doctrine/doctrine.toml` is the sanctioned fallback (values `local | shared |
  auto`; `auto` does *not* fall back on an auth failure — it fails like
  `shared`).
- (unminted) "The shell adds no trailing newline" is one rule instead of three:
  the byte boundary (`Form::Stdout`, `render`) owns the single `\n`, and `main`
  writes with `print!`. Pinning it with a count assertion
  (`stdout.matches('\n').count()`) catches both a missing and a doubled newline.

### Open at audit entry (dispositioned in RV-004 — see "Audit outcome" below)

- PHASE-08 leaves nothing open against its own bar: `EX-1…EX-4` hold,
  `VT-1`/`VT-2` green, `VH-1` carried to the human. Its two forward notes are the
  first two below.
- **VH-1 outstanding** (PHASE-08, by seat decision): a human reads `README.md` as
  a newcomer — that the worked example runs, the sharp edges match what they hit,
  and the quoted guarantee is the design's own wording. The packet (section list,
  copy-pasteable command sequence, the guarantee in full, one code location per
  sharp edge) is in the charter-8 hand-back.
- (unminted) `src/spec.rs` reports a *known* 0.2 key with a wrong type
  (`{"where":1}`) as a named-future refusal rather than a type error; harmless in
  0.1 and worth a look when 0.2 lands.
- (unminted) Nothing in 0.1 populates `dangling`, so `docs/store-format.md` and
  the guide say so in prose; a 0.2 reader will want the envelope documentation to
  say when it starts being populated.
- PHASE-09 leaves nothing open against its own bar: `EX-1…EX-3` hold,
  `VT-1…VT-3` green, no `src/` change. Two forward notes below.
- For `/audit`, from PHASE-09: `Store::load()` treats a **directory**-shaped live
  journal state as an explained entry, so a direct `load()` silently returns the
  pre-decision state (PHASE-05 recorded the file-shaped variant of the same
  hole). The refusal lives only on the pipeline path every command takes, so
  this is not a behaviour any command can reach — but
  `tests/concurrency.rs`'s "the fixture is unrecovered" assertion rests on it and
  would invert if the review rules that `load()` must refuse a live journal.
- ~~For PHASE-08: the brief's end-to-end acceptance example cannot be run
  verbatim in 0.1.~~ **Landed:** the README's worked example is that flow
  truncated at the 0.1 boundary and ends by showing the `follow` refusal, pinned
  by `tests/readme_example.rs`; `tests/compatibility.rs` pins the runnable prefix
  and both refusals. **But the divergence itself is unrecorded against the
  brief** — the brief is product authority and still carries a nine-step
  acceptance example three of whose steps 0.1 refuses by design. For `/audit`:
  is the brief owed an erratum, or is refusal-by-name the intended 0.1 boundary?
- PHASE-07 leaves nothing open of its own: `EX-1…EX-7` hold, `VT-1`/`VT-2` green,
  `VA-1` discharged by reading the ingress path rather than by a test. Its three
  forward notes are the first three below.
- Two byte-boundary items the earlier harvest routed to PHASE-07, and where each
  landed: the **response/envelope key order** is settled as lexical (DEC-007; the
  brief's display order is illustrative) — closed here. The **id-first ordering of
  a projected entity inside an envelope's `objects`** is **still open**: the fix
  lives in `src/entity.rs`, which PHASE-07's charter lists among the settled parts
  and does not include in its paths, so it returns to `/audit` → `/reconcile` as a
  backlog candidate rather than widening the phase.
- PHASE-07's hand-back candidate (unminted): `print!` to a closed stdout
  (`smut query | head -1`) panics with exit 101 instead of dying quietly. The
  post-commit ambiguity is documented; the panic is not, and a pipe closing early
  is ordinary use. Not this phase's bar — it predates the shell.
- `--` reaches an opaque id, but a caller who omits `--` and names an id that
  begins with `-` still gets a clap usage error; that is the documented
  convention (DEC-008), not a defect, recorded so `/audit` does not re-raise it.
- **VH-1 outstanding** (PHASE-05, by seat decision): a human reads
  `tests/durability_crash.rs` against §4.3/§4.4/§4.5 and confirms every named
  boundary has an interruption test, that repeated recovery is explicit, and that
  the file claims only code-path ordering. The 22-row boundary table is carried
  in the charter-4 hand-back.
- PHASE-04's hand-back candidates (unminted): nested stored entity-node ids are
  unvalidated, so a corrupt-but-index-consistent nested `id` can reach a
  `scan_refs` panic instead of exit 1; a symlinked `.smut/lock` is followed
  (`OpenOptions` without `O_NOFOLLOW`); duplicate `objects` keys in `index.json`
  collapse at parse.
- PHASE-05's hand-back candidates (unminted): `Store::load` treats a regular file
  named `journal.commit` as explained, so a direct `load()` accepts what recovery
  refuses — the refusal lives on the pipeline path only; a non-UTF-8 entry inside
  a journal state directory is handled by lossy name; §4.3's two post-decision
  `.smut` fsyncs are not independently injectable without two more registry names.
- PHASE-06 needed nothing from PHASE-05 beyond the notice's placement rule, as
  predicted: the journal phase added no rendering surface and did not touch
  `select`, so the renderer consumed the PHASE-03 `Selection` unchanged.
- QUE (unminted — id reservation needs an authenticated remote push, which this
  checkout cannot do): should design §12 and §11.4 be amended for the lib target
  and the pre-existing README? The plan departs from the table and records why in
  `plan.md`.
- QUE (unminted): §4.3 names neither the post-apply nor the post-done `.smut`
  fsync, so both borrow a neighbour's boundary name. A design touch should name
  them; the implementation records the borrowing rather than inventing names.
- For `/audit`: **is an envelope's entity root id-first under DEC-007?** Measured
  on the built binary at PHASE-07 close, in a scratch store with
  `declare {"declarations":[{"id":"smut","body":"First.","kind":"note","title":"Alpha"}]}`:
  the entity *file* is `id, body, kind, title` (id-first), while
  `query {"from":["smut"],"select":["body"]}` emits
  `{"dangling":[],"missing":[],"objects":[{"body":"First.","id":"smut"}],…}` —
  lexical inside `objects`, so the "one shared Entity wrapper" is applied to
  files only, not to the stdout envelope. Top-level response keys are lexical
  too, which is a separate, already-logged decision. If the review finds the
  wrapper must be shared, the fix is local (`select`'s envelope path through
  `entity`'s stdout form) but it changes documented output bytes, so it must be
  ordered before PHASE-08's README examples are finalised or those examples
  regenerate by being re-run.

### Audit outcome (RV-004, 2026-11-01)

`doctrine review show RV-004` is the ledger; this is the index.

- **Fixed in the audit (3).** F-1 the shared serialisation wrapper
  (`78a7eb1`); F-2 a closed stdout exiting 1 instead of panicking (`35b62ed`);
  F-3 the lifecycle's `gate`/`prove` cadence recipes (`dfc6e37`).
- **Owed to reconcile (5, `design-wrong`).** §12's selector omission of
  `Cargo.lock` (F-4); §12's two path errata — README is edited, not new, and the
  lib target is missing (F-6); §4.3/§4.4's three unnamed `.smut` fsyncs (F-7);
  the brief's acceptance-example boundary (F-9); §4.2's mirror of the wrapper
  clarification, plus a one-sentence REV modify of DEC-007 (F-1). The
  `## Reconciliation Brief` in RV-004 is the structured handoff.
- **Tolerated with rationale (6).** `Store::load` accepting a live journal state
  (F-12); nested stored entity-node ids (F-13); duplicate `objects` keys (F-14);
  the symlinked `.smut/lock` (F-15); `notes.md` reading undeclared (F-5); the
  design review's stale cache (F-8).
- **Aligned (2).** `spec.rs`'s named-future refusal for a mistyped 0.2 key
  (F-16); empty `dangling` (F-17).
- **Outstanding human obligations (2), not discharged by the audit.**
  PHASE-05/VH-1 — a human reads `tests/durability_crash.rs` against
  §4.3/§4.4/§4.5; PHASE-08/VH-1 — a human reads `README.md` as a newcomer. Both
  ledger findings are `fix-now` and non-terminal until the human answers; the
  lifecycle stays at `audit` until they do, which is the honest position rather
  than a false close.
- **Environment posture recorded**: `[reservation] reach = "local"` in
  `.doctrine/doctrine.toml` because this checkout cannot push reservation refs;
  revert to `shared` when the remote is writable.

### Owner decision after the audit: packaging (2026-11-01)

The repository owner fixed the identity and licence the brief left open ("licensing
is a repository-owner decision"): **GPL-3.0-only**, repository
`https://github.com/davidlee/smut`, homepage `https://doctrinal.systems`. Landed
as `chore(pkg)` after RV-004 was concluded, so it is not audit-covered; it is
recorded here and in RV-004's reconciliation brief addendum.

- `LICENSE` is the FSF's verbatim GPL-3.0 text (gnu.org `gpl-3.0.txt`, 674
  lines). `Cargo.toml` gains `description`, `license`, `repository`, `homepage`,
  `readme`, `keywords`, `categories`, and an `exclude` list for what is project
  governance rather than crate content (`.doctrine/`, `.pi/`, `AGENTS.md`, the
  implementation brief, the Nix devshell).
- `cargo package --list` → 38 files, 1.6 MiB (1.2 MiB compressed);
  `cargo publish --dry-run` packages and verifies the crate with no warning.
- The crate name `smut` is **free** on crates.io (index lookup → 404, API →
  "crate `smut` does not exist"), so no rename is needed.
- Not yet published: this checkout holds no crates.io token (no
  `~/.cargo/credentials.toml`, no `CARGO_REGISTRY_TOKEN`), and `0.1.0` is a
  version a publish cannot re-use. The upload is the owner's act.
- Cosmetic caveat for crates.io's renderer: the README's brand image is a
  relative link (`./smut.jpg`), which crates.io does not resolve for published
  READMEs. Left alone deliberately — the owner's brand block is byte-preserved by
  this slice.
- The owner then replaced the README's relative brand image with
  `https://github.com/davidlee/smut/raw/main/smut.jpg` (crates.io does not resolve
  a relative one), so the "brand block byte-preserved" claim now covers the copy
  text only, and `smut.jpg` is excluded from the crate: **37 files, 556.8 KiB
  (132.9 KiB compressed)**, `cargo publish --dry-run` clean.

### Published, and the two `VH-` obligations closed (2026-11-01)

The owner published outside this jail. Verified from here rather than taken as
reported: the crates.io API answers for `smut` — 0.1.0, created
`2026-11-01T08:30:46Z`, one version, not yanked, with the packaged description,
keywords and categories. Consequences: `0.1.0` can never be re-published, and the
licence question the brief left open is settled *by the act* as GPL-3.0-only.

**F-10 / PHASE-05/VH-1 — the human read was waived by the owner, not performed.**
The owner's direction: the durability read "really seems more like an agent job
than a me job". So the criterion's *mode* was waived and its content delegated;
the deviation is recorded here rather than laundered as a human read. The agent
read performed, against §4.3/§4.4/§4.5:

- 22 registry names ↔ 22 tests, no orphans in either direction: 15 commit
  (`interrupted_at__*`), 7 recovery (`recovery_interrupted__*`). The audit
  finding's "8 recovery" counted the coverage guard's `format!` literal at L1259;
the corrected pair is 15 + 7 = 22.
- Every test arms the boundary its name claims (checked per site).
- Repeated recovery is explicit per step: each recovery test aborts inside
  recovery and requires the next command to converge on an uninterrupted twin's
  tree, so convergence — not survival — is what each name asserts.
- §4.3's asymmetry and §4.5's roll-forward are covered by 7 pre-commit-failure
  tests, 6 `*_rolls_forward` tests over exactly the post-decision boundaries, and
  4 failed-recovery tests.
- The file claims code-path ordering only (module doc L14–19).

Green at head: `cargo test --features fault-injection --test durability_crash` →
44 passed; 0 by default by construction.

**F-11 / PHASE-08/VH-1 — the owner read it: "it reads well enough".** The
runnable half was delegated and verified: `cargo test --test readme_example` → 2
passed, and the `console` block re-run by hand against the release build at head
matches the README's recorded bytes in order, exiting 2 on the closing `follow`
refusal.

**Residual the publish exposes.** The now-public README carries two *relative*
links — `docs/store-format.md` and `LICENSE` — which a crates.io README cannot
resolve; the brand image was already absolutised for exactly this reason. Minted
as a backlog issue rather than left in this note.

### External use evidence: the operator's interview store (read 2026-11-01)

The owner used the shipped binary in two live sessions and kept the transcript as
a **smut store**: `interview.local/.smut` (untracked — `.gitignore` carries
`.smut`), 66 entities, entry points `start-here` then `audit-handoff`. Read here
through the binary (`get start-here`, one wide `query`, `render`), not the raw
JSON.

**Triage verdict: it yields no 0.1 conformance defect.** All twenty
`finding-smut-*` were sentenced against the design and the brief:

- `DROP` eating a literal (08) and array-replace losing a reference list (20):
  §3.2's fold, §8.3's edge-case table and `learn`'s *Declaring* section specify
  both. Silent loss is real; it is a design cost, not an implementation
  divergence.
- Opaque filenames (10), no file input (03), no timestamps (09): absent from the
  brief and the design by scope, and contravened by neither.
- `--show-fields` printing compact JSON (11), and heading competition and weak
  boundaries (13): §7.1 requires exactly this — non-strings as compact JSON,
  bodies verbatim, and "bodies may still absorb *each other* — accepted and
  stated, not denied".
- The argv/JSON two-dialect complaint (02): clap exit 2 by design (§7.2,
  DEC-008), and the store's own `a-error-verdict` retracts it.
- The guide's silence on locking and disk layout (12): DEC-010 sends layout to
  `docs/store-format.md` and limits `learn` to the 0.1 surface; the README's
  Portability note carries `flock` and the acceptance platform. The store
  inspected `learn` only.
- Tool-output truncation (14): the harness, not smut.

**What it does change here.** It corroborates two audit findings independently:
F-9 (three separate statements that the brief's end-to-end flow is not
0.1-runnable, and that the two-call composition is the 0.1 idiom) and F-7 — with
the mapping measured in `src/store/journal.rs` rather than inferred:
`RemovalsApplied` (L374) covers apply's closing `fsync(.smut)`, including a batch
that removed nothing; `DoneRenamed` (L396) covers the fsync after the `commit →
done` rename; `RecoveryDoneRenamed` covers *both* the pre-rename and post-rename
fsync (L577, L583).

**What it changes for 0.2.** IMP-001's own pick-up trigger — "used in anger …
`where`'s absence is a felt gap" — is now met. The one genuinely new design
question is result provenance (nodes, induced edges, unexpanded frontier).

**One claim from it that does not transfer.** `obs-build-environment` describes a
different checkout (`/home/david/dev/smut`, read-only `target/`) and concludes the
full suite cannot run offline; here `--all-targets` runs offline against the Nix
toolchain. Perishable, as that record says of itself.

**Disposition.** Product evidence, not audit evidence. Captured in IMP-001's body
and two new backlog items; the store itself is *not* promoted into the repo — as
a 0.2 acceptance fixture it is IMP-001's decision, not SL-001's.
