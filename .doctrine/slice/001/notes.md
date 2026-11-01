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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · reviewing (design run rev 37, pass RV-002) · bcd1389

### Produced

- SL-001 re-scoped from the whole brief to **0.1 notebook**; slug
  `smut-0-1-notebook`. The rest is carried by IMP-001 (0.2 graph) and IMP-002
  (1.0 presentation), the latter `after` the former.
- Research round: `research/research.md` (thread 1 governance applicability,
  thread 2 substrate/prior art) plus `raw/`. Crate versions verified against
  crates.io; baseline re-stamped after the re-scope.
- Inquiry map: 11 nodes, **all resolved**, each against an accepted decision —
  DEC-001 boundary, DEC-002 equality, DEC-003 journal, DEC-004 pipeline/lock,
  DEC-005 references, DEC-006 ordering, DEC-007 projection/serialisation,
  DEC-008 CLI, DEC-009 Markdown, DEC-010 docs, DEC-011 spec surface. All linked
  `shapes` → SL-001.
- `design.md`: 12 sections, materialised at rev 37; design-target selectors
  recorded (src/**, tests/**, docs/**, README.md, Cargo.toml, justfile).
- Review: RV-001 (hostile self-pass) raised 3 findings — 1 major (§5 pipeline had
  no absent-store branch for reads/deletes), 2 minor (index staging unstated;
  `store/fault.rs` ungated) — all fixed, verified, concluded. The pass went stale
  over the fixes, so the run was taken back to drafting and re-entered,
  opening RV-002 over the settled content, concluded with no new findings.
- Commits: `acf53fc` (design run, decisions, review ledgers). No source code yet.
- `.doctrine` committed with the artefacts, not separately.
- Verification gate: **N/A** — no source code and `justfile` has no `gate`
  recipe yet; §11.4 of the design commits to wiring it.
- Left untouched: pre-existing `flake.nix` / `flake.lock` modifications and an
  untracked `flake.lock~`, owner's business.

### Learned

- Design-run mechanics that refuse or silently stale when improvised —
  `mem_01a1957016217433b68d54b18d8fb094`: inquiry disposition needs a `cp-` node
  with `dispose`; editing sections during reviewing stales the pass (back-edge to
  drafting is the remedy); the research baseline hashes the scope doc, so
  restamp after editing it; `selector add` upserts by path.
- The brief's own "implementation sequence" is a build order, not a delivery
  order — carrying it as a plan makes nothing usable until nearly everything
  exists.
- A comparator that switches numeric domains per pair can be **cyclic**, not
  merely imprecise; only exact comparison is a total order.

### Open

- **Awaiting the user's review choice** — this is *not* in the run: `/inquisition`
  hostile pass, an external adversarial reviewer (prompt to be written), or
  straight to `/plan`. All three leave the same gate work: 12 section
  attestations, a `review-disposed` naming **RV-002**, then `design-accepted`.
- `proptest` dev-dependency proposed in §12, beyond the five settled crates —
  unvetoed, not yet objected to.
- Design-run residual: `review.scope`, `review.selectors`, `review.passes`
  runbook steps outstanding, plus the three lock conditions above.
