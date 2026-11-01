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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · charter 4 close (PHASE-01…PHASE-05 complete, VH-1
outstanding on a human) · `0e2a519`

### Produced

- PHASE-01 landed the crate spine (9 commits, `e0ce040…397c5c8`); PHASE-02 the
  declaration core (9 commits, `614ea33…7adefa4`); PHASE-03 the selection core
  (7 commits, `e08bb7f…3fc8086`); PHASE-04 the store format, index, lock,
  validation and pipeline seam (5 commits, `8403ac1…49d81d7`); PHASE-05 the
  journal, its recovery and its interruption evidence (7 commits,
  `7d841fd…0e2a519`). `docs/` and `README.md` remain undelivered by design —
  PHASE-08.
- `just check` green at every close; after PHASE-05: 197 lib + 4 bin + 5
  `tests/selection.rs` + 12 `tests/store_corruption.rs` + 0 (default-gated)
  durability = **218** on default features, and 210 + 4 + 44 + 5 + 12 = **275**
  with `--features fault-injection`. The durability target executes 44 tests
  under the feature and 0 by construction under the default build.
- `slice verify-vt` green for every row PHASE-01…PHASE-05 own (18/18); later
  phases red because their files do not exist yet.
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
  wrapper.

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

### Learned

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

### Open

- **VH-1 outstanding** (PHASE-05, by seat decision): a human reads
  `tests/durability_crash.rs` against §4.3/§4.4/§4.5 and confirms every named
  boundary has an interruption test, that repeated recovery is explicit, and that
  the file claims only code-path ordering. The 22-row boundary table is carried
  in the charter-4 hand-back.
- Two byte-boundary defects for PHASE-07 to fold in: nested entity objects are
  serialised lexically rather than id-first (DEC-007), and the response/envelope
  key order is lexical where the brief's examples show a display order (accepted
  as insignificant; DEC-007 fixes the stdout form).
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
- PHASE-06 needs nothing from PHASE-05 except the notice's placement rule: the
  journal phase added no rendering surface and did not touch `select`.
- QUE (unminted — id reservation needs an authenticated remote push, which this
  checkout cannot do): should design §12 and §11.4 be amended for the lib target
  and the pre-existing README? The plan departs from the table and records why in
  `plan.md`.
- QUE (unminted): §4.3 names neither the post-apply nor the post-done `.smut`
  fsync, so both borrow a neighbour's boundary name. A design touch should name
  them; the implementation records the borrowing rather than inventing names.
