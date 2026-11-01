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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · plan (design locked, run rev 53) · cf57949

### Produced

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

### Open

- QUE (unminted — id reservation needs an authenticated remote push, which this
  checkout cannot do): should design §12 and §11.4 be amended for the lib target
  and the pre-existing README? The plan departs from the table and records why in
  `plan.md`.
