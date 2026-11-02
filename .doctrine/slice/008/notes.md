# SL-008 notes — layered conventions

## Design surface triage (exploring, 2026-11-02)

### Constraining governance

| authority | what it fixes here |
| --- | --- |
| DEC-026 | three roots, one relative suffix (`<root>/.config/smut/conventions/`), project root is the walk-up terminus, project beats user beats binary, same-named sets never merge, origin always printed, an environment override exists in the choice |
| DEC-028 | a set is one markdown file: filename = name, first `#` = title, flat `summary`/`version` frontmatter, derived keys refused by name, unknown keys ignored, unparseable line refused, a malformed file never breaks another layer, seed keeps `conventions-<name>` and gains `source`, sections are a grammar implemented later |
| DEC-029 | the word `conventions` spans command, leaf directory, seed `type` and id prefix |
| DEC-017 | the seed is an upsert under a fixed id reporting the replaced body under `previous`; a set never validates a store |
| DEC-004 | the store is `<cwd>/.smut`, never searched upward — the process has one store root, and it is the cwd |
| DEC-008 | flags validated before operands, all in step 1 |
| STD-001 | `docs/store-format.md` is the store format's only record; nothing lands inside `.smut/` |
| ADR-001 | identity is `smut_id`, reserved at every depth; a container-named id value keeps its own name |

**The two that actually bite.** DEC-004 and DEC-026 disagree about what "the project" is,
by design: the store never walks up, the sets do. And `docs/store-format.md:53-55` says
"the binary shipped today embeds its one set", which is true only until this lands —
STD-001 makes that file the format's only record, so the clause is amended in the landing
commit.

### Shaping decisions — settled before this run, not re-opened

DEC-026 / DEC-028 / DEC-029 on DEC-017, with IMP-013's brief as the work list. The design
run's job is the seven deltas in `research/research.md` § Design-input deltas, not the
mechanism.

### Open questions

Answered during the run — all ten inquiries resolved at revision 11 with a disposition.
The durable answers are DEC-031, DEC-032, DEC-033 and QUE-002; the design detail is
carried by the sections. The leans below are what was recorded *before* inquiring and are
kept as the prior, not as the answer.

| id | question | lean |
| --- | --- | --- |
| inq-1 | What does one resolved set carry, and what shape makes three layers one mechanism rather than three loaders? | one record per file: name, summary, version, source path, layer, body, and a loaded-or-reason outcome |
| inq-2 | How is the project root found — and what happens when there is none? | walk up for a `.git` entry (directory **or** file); terminate at the filesystem root as cwd, no shelling out to git |
| inq-3 | How is the user layer found without the loader reading the environment ambiently? | roots are parameters; only `main` reads `$XDG_CONFIG_HOME` / `$HOME` |
| inq-4 | What does a set that cannot be loaded exit with, and what does it print? | exit 1 with layer, path and reason; exit 2 stays for what the caller typed |
| inq-5 | What does the listing print now that one line per set becomes one line per file? | name, summary, layer, path, and a shadow/skip marker |
| inq-6 | Which directory entries count as set files? | `.md` only; anything else is not a set and is not reported as a failure |
| inq-7 | How does the seed express origin without breaking "the body is the file's bytes"? | `source` names the file; `version` is the author's for files, the crate's when shipped; prose stays byte-identical |
| inq-8 | Does the environment override ship in this slice? | no — defer, since test isolation is solved by parameters |
| inq-9 | How does the shipped set stay one code path with a file, and what happens to `Set`, `Ask` and `SETS`? | the `include_str!` bytes go through the same reader; `Ask` owns its set; `SETS` becomes the binary layer's contents |
| inq-10 | What proves the layering, given the suite must be rewritten? | discriminating assertions only: a fake home holding a real competing set, a worktree `.git` file, a corrupt project set — no assertion that no input can falsify |

### Risks

- **R1 — two roots in one process.** Intended, but it makes `smut conventions` and `smut
  declare` disagree about "the project" in a subdirectory. Test the disagreement.
- **R2 — step-1 I/O.** `conventions::resolve` runs inside `cli::parse` (DEC-004 step 1);
  the refusal channel for a bad set is not `Reject`'s two-value taxonomy today.
- **R3 — home is an ancestor of nearly every checkout**, so the walk-up terminus is
  load-bearing rather than cosmetic.
- **R4 — ambient environment in tests.** The helper inherits the parent environment
  (`tests/conventions.rs:81-90`); without isolation a developer's own `~/.config/smut/`
  silently changes the suite.
- **R5 — the suite rewrite can weaken evidence.** `tests/conventions.rs` is 582 lines
  pinned to the compiled-in shape, and the behaviour-preservation gate would normally
  call a rewritten seam suspect. This seam is deliberately changing, so the discipline
  moves to the *assertions*: each one must name the change that reddens it.
- **R6 — `.config` in a repository already means "a home directory"** (DEC-026 accepts
  this; origin printing is the cure, and it now has to land in the listing).

### Assumptions

- **A1** — no new dependency: frontmatter is two flat keys (hand-parsed) and
  `std::env::home_dir()` exists on the pinned toolchain (checked: rustc 1.100).
- **A2** — "is this a repository?" is the whole question; the `.git` entry's *contents*
  are never needed, so a `.git` file (worktree, submodule) is enough.
- **A3** — a set file is `<name>.md`; the name is the stem, case preserved as written.
- **A4** — text is the human channel and `--format json` the piped one; the json
  declaration's shape does not change beyond `source`.

## Critical pass before drafting (informal, 2026-11-02)

An independent read-only review — a second agent, given the scope, this file, the research
artefact, the three decisions, the ten resolved inquiries and the code they claim things
about — returned ten findings. All ten were upheld on the evidence; three changed a
decision, and three became obligations owed to another stage.

| # | finding | disposition |
| --- | --- | --- |
| 1 | DEC-028's "a malformed file never breaks the other layers" contradicts DEC-031's precedence-by-layer, and is wider than either promise | **upheld** → DEC-031 records the collision; DEC-028's clause owes an **in-place revision at reconcile** |
| 2 | the design never said *where* resolution happens: parse-time (making `cli::parse` impure) or after it | **upheld → DEC-032 amended**: `parse` stays pure and returns the question; the binary resolves it where it computes the store root. Mirrors `Request` |
| 3 | a shadowed set is visible but unaddressable — no command can print or seed the loser | **upheld → DEC-031 amended**: stated as deliberate, with the listing's path as the workaround; layer-qualified names would change the seed's fixed id, so that is its own decision |
| 4 | the listing's path form was unspecified, and it decides whether tests can assert anything stable | **partly upheld**: the row prints the file's own path (absolute for a file layer — the row's job is to let a reader open it); the seed's `source` is layer-relative `conventions/<name>.md`, identical for all layers. Rejected the proposal to print root-relative paths in rows: a row saying `conventions/hygiene.md` cannot tell you *which* project |
| 5 | unreadable and non-UTF-8 files must be rows, not panics — `include_str!` only guarantees UTF-8 for the shipped layer | **upheld → DEC-033 amended** and DEC-031 rule 5 extended: read and decode failure are the same class as parse failure. A symlinked `.md` resolves through the link and is not a failure |
| 6 | `title_of` mangles an author's heading (`Conventions: ` + lowercased first letter) | **upheld → DEC-031**: the seed's `title` is the heading verbatim; the shipped file's own heading carries the `Conventions:` prefix. Deletes the transformation instead of special-casing it |
| 7 | two seed fields were undefined for a file set: `summary`, and `version` when the author declares none | **upheld → DEC-031**: `summary` is not a seed field; `version` is omitted rather than claiming the crate's |
| 8 | `smut conventions NAME > file.md` no longer round-trips once prose is the body | **upheld → DEC-031**: the command is a printer, not a round-trip; the suite asserts "the body", never a regeneration |
| 9 | DEC-026's cost paragraph and DEC-017's "a file plus a match arm" go stale, and neither is superseded | **upheld** → reconcile obligations below |
| 10 | "a set's identity is its origin" overclaims, since the stored id stays layer-blind | **upheld → DEC-031**: narrowed to display and resolution identity; the stored id stays `conventions-<name>` |

What the reviewer verified independently and found correct: every load-bearing research
claim — the byte-identity assertions, the absence of git-root and config-home reading, the
`Reject→2 / Store→1` map, the cfg-gated `fault-injection` subprocess as the only one, `Cli`
having no global options, and the learn exit table being one line.

### Obligations owed at reconcile

1. **DEC-028** — narrow "a malformed file never breaks the other layers" to listing and
   loading, not name resolution. In-place revision (the repo's practice, as in DEC-025 and
   REV-002).
2. **DEC-026** — its cost paragraph still promises "a set header naming its own file",
   which DEC-031 declined. Amend the clause to point at the listing.
3. **DEC-017** — the sentence about adding a later set being "a file plus a match arm" is no
   longer how sets arrive; check and amend if it still reads as mechanism.

## Lock residue (2026-11-02)

The design locked at revision 36 with all seven sections attested in both lanes and
RV-029 disposed `conducted` on the operator's word ("otherwise lgtm, let's lock"), after
the three highest-regret consequences were laid out and two hedges were written in:

- **`smut learn` gains a line** saying a printed set does not reproduce its frontmatter —
  a silent loss converted into a stated one (§5 of `design.md`).
- **The seed's layer-blind provenance is stated in the design** rather than discovered:
  `conventions-<name>` and `source: conventions/<name>.md` are identical at every layer, so
  a stored record cannot name its layer; the listing is the answer.

Two things the operator explicitly declined to pull in, and why they cost nothing later:

- **Section addressing** (DEC-028's pinned grammar, IMP-008's flag). The loader stores a
  `body` and never interprets it, so the grammar can arrive without touching the loader —
  and it should arrive *with* the flag rather than before it.
- **Validity as a tie-breaker.** A broken nearer set makes that name unaskable. Accepted as
  "fails loud and clearly": the listing still names the file and its reason.

**Residue to carry into audit:** RV-029's recorded coverage predates the two hedge edits
(revisions 30–31). Those sections were re-attested in the adversarial lane (att-5, att-6 at
revision 32), but no review *pass* has run over the edited text, so the run reports
`RV-029 STALE — it no longer covers current content`. It is an advisory, not a gate; the
audit should treat the two hedge paragraphs as reviewed-by-attestation only.

## Harvest

**fresh-as-of:** `SL-008` status `ready`, design run `dr-01a19b61` locked at revision 36,
head `de8cf3d`. Plan authored, hardened by two audit lanes, and the audit's design
questions taken through `/feedback`. Next stage: implementation (PHASE-01..05), not
audit.

### Produced

- **`plan.toml` + `plan.md`** — five phases (roots → reader → loader → surface → seed), 5
  phase sheets materialised, every VT row carrying a test name or symbol that can only
  exist after its phase. `plan.md` also carries the per-phase model requirement, the
  critical pass, and eleven adjudicated design questions.
- **A fourth reconcile obligation** — `design.md` §5's worked listing contradicts its own
  third-field rule (its binary row prints a summary where the rule, repeated as DEC-031
  rule 4, makes it `shadowed`), and §4's "one clause per resolved name" contradicts its
  own per-file example. Recorded above, with the original three.
- **Operator ruling** — a known frontmatter key with no value (`summary:`) declares
  nothing: the field is absent, the listing shows `-`, the seed omits the version
  (PHASE-02/EX-2). A value continued across lines still fails naming its line.
- **Memory** `mem_01a19c0604b57ce397b575176682fc97` — harden-plan's audit lanes are
  read-only agents: brief them to return, never to write files.
- Research baseline re-verified at `d224dc6` (line counts, both negative greps, the error
  inventories) and restamped; the advisory's drift was chronological, not evidential.

- **DEC-031** — the set in force is chosen by layer, never by validity; origin printed in the
  listing; the body stays the file's bytes. **DEC-032** — the loader takes its roots as
  parameters; nearest `.git` for the project root; the config home for the user layer; only
  the binary reads the environment; resolution happens after parsing. **DEC-033** — exit 2 for
  what the caller typed, 1 for what the filesystem supplied, a listing reporting a broken set
  at 0. All three `shapes → SL-008`.
- **QUE-002** — the environment override's name, what it names, and its rank: open by design.
- **RV-029** — twenty findings (nine major, eleven minor), all disposed `fix-now`, verified,
  ten severe ones routed (`review` ×6, `demonstrate` ×4), pass concluded.
- `design.md` (314 lines, seven sections), eight design-target selectors, one scope-relevant
  glob, `docs/store-format.md` added to the fence.
- Memories: `mem_01a19bcdb9177f53bd0d7597098b312f` (review-stage mechanics beyond the base
  set), `mem_01a19bce0c187080a465340cfaf6f8bd` (shared worktree git discipline).

### Learned

- **Origin printing and byte-identity cannot both hold.** DEC-026 promised a header naming the
  file; DEC-028 made a set's bytes load-bearing. Keeping the bytes and moving the origin to
  the listing was the whole hinge of the design.
- **Validity is not a tie-breaker.** A broken nearer set makes that name *unaskable*. Accepted
  on the operator's word ("fails loud and clearly") after being stated as a consequence.
- **A pure resolver needs its roots passed in**, and the leak it prevents is in the *unit*
  tests, not the CLI — `main`'s in-process `dispatch_of` reads real cwd and `$HOME`.
- Attestation is not coverage: a pass opened before late edits still reports `STALE` after the
  edits are re-attested.

### Open

- **QUE-002** — the environment override (name, target, rank).
- **IMP-008** — the section grammar and the `--with-conventions` flag; deliberately deferred,
  free to add later because the loader never interprets the body.
- **Reconcile obligations** (three governance amendments, in § Obligations owed at reconcile):
  DEC-028's malformed-file clause, DEC-026's "set header naming its own file", DEC-017's
  "a file plus a match arm".
- **RV-029 coverage residue** — the two hedge paragraphs (§5, §6 at revisions 30–31) are
  attested but no pass ran over them.
- **SL-006** is unblocked by this slice landing: `epistemic-records` returns to `plan`.

## Plan stage (2026-11-02)

Five phases, cut by evidence rather than by file. `plan.toml` is the authored source;
`plan.md` carries the sequencing rationale, the decisions the design left to the plan, and
the critical pass. Phase sheets materialised by `doctrine slice phases 008`.

- **PHASE-01 roots** — `Roots`, `project_root`, `user_root`. Pure, additive, green.
  Carries RV-029 F-17's directory-level half (the walk stops at the repository).
- **PHASE-02 the reader** — `Layer`/`Origin`/`Cause`/`Set`/`Entry`/`Outcome`, `parse`,
  `SHIPPED` through the same reader, `Ask` owning its set, one new refusal class at exit 1,
  `hygiene.md` frontmatter + `Conventions:` heading, one guide line on the frontmatter loss.
- **PHASE-03 the loader** — `load` + `set_in_force`, written before their consumer on
  purpose; absent vs unreadable, candidate filter, non-UTF-8 names, sort, kind joined once.
- **PHASE-04 the surface** — `answer(roots, kind, ask)`, the four-field listing with its four
  third-field states, the origin-bearing refusals, `main`'s environment read, the guide's
  layer and exit-clause lines, the store-format clause, and the rewritten spawned suite.
- **PHASE-05 the seed** — `source`, the three version shapes, body/prose agreement, upsert.

Decisions the plan had to take (recorded in `plan.md` § Decisions taken): `Entry.name` is
`Option<String>` (`None` only for a non-UTF-8 filename, listed as `<not UTF-8>`); the new
refusal is `StoreError::ArtefactNotLoaded` and stays inside the two-class taxonomy; the
unreadable-layer fixture is a path that cannot be read as a directory, with mode 000 as a
second row, so a root user cannot make it pass; the unknown-name field holds rendered
clauses and says so; the crate's version is injected in one place.

Research advisory: `slice-008.md` changed and `design.md` appeared after the baseline —
chronological drift, not evidential. The code map was re-verified at `d224dc6` (line counts,
both negative greps, the error inventories) and the artefact restamped; a note in
`research/research.md` records the check.

Carried, untouched: QUE-002, IMP-008, the three reconcile obligations above, RV-029's
post-pass coverage residue, and README/CHR-001.

## Plan hardening (2026-11-02)

Two independent audit lanes — A obligation coverage (145 rows), B implementation surface —
then one integrator. Ledger and both lane reports are gitignored local files in this folder
(`plan.harden-ledger.local.md`, `plan.coverage-audit.local.md`,
`plan.surface-review.local.md`); this paragraph is the durable summary.

**Upheld and folded in (40 of 49 findings, 2 blockers, 12 majors):**

- **The project layer's `.config/smut` segment was assigned to nobody.** §2's two functions
  cannot join it (`project_root` returns the repository, `user_root` returns
  `<config-home>/smut`), §6's `load` joins only `<kind>`, and DEC-026/§5/§7 all require it.
  As written the loader would have read `<repo>/conventions/`. Both lanes found it
  independently. Fixed by `Roots::under`, one function and one pinned unit test in
  PHASE-01/EX-3, with the walk-up row that would have caught it now gated.
- **`src/main.rs` is compile-forced in PHASE-02, not PHASE-04** — its in-process test
  asserts `conventions::HYGIENE.prose` and `HYGIENE` retires there. Owned by PHASE-02/EX-10
  so the phase can end green; the move to the spawned suite stays PHASE-04's.
- **`dispatch` was missing** from the roots-threading list; it is the function every
  in-process test calls, so roots computed there would read a developer's `$HOME`.
- **A keyword that is already present is not a gate.** Seven were satisfied before their
  phase (including `every_store_refusal`, which nothing length-pins — the criterion now
  requires that assertion). All replaced with test names or symbols that can only exist
  after the phase; §7's "preserved behaviour" list and four §7 process rows are now gated.
- Seed `source` transcribed as `<kind>/<name>.md` where `seed(set)` has no kind; the set's
  name derivation homeless; the not-UTF-8 *content* reason mis-worded; the fourth column's
  absoluteness; the guide's hand-wrapped `const` making sentence keywords wrap-fragile;
  `just check` versus bare `cargo test`; `$TMPDIR` fixtures; `Layer`'s printing; the three
  refusal inventories; the `tests/fixture.rs` disposition. All now in criteria.

**Eleven design questions owed to /feedback**, each answered in the plan by the reading the
design's own rules make least wrong and named in the criterion that carries it —
`plan.md` § Design questions owed to /feedback. The ones that matter: §2 never assigns the
`.config/smut` segment; §5's worked listing contradicts its own third-field rule (its binary
row prints a summary where the rule says `shadowed`); §6's `parse` never says where the row's
name comes from though §3 needs a non-UTF-8 filename reported before any read; an unreadable
layer *directory* has no `Origin` for §6's one new variant; `seed(set)` has no `kind`.

Rejected with evidence (7): `From<Reject> for Error` already makes the `?` compile;
`&[&Set; 1]` coerces; shadowed-unaddressability and `Entry.name: Option<String>` are settled;
no other suite runs `conventions`; one cosmetic test rename.

## /feedback on the audit's design questions (2026-11-02)

Eleven items came out of the hardening audits; each was re-read against the design text
rather than the auditors' framing. Full dispositions are in `plan.md` § Design questions,
adjudicated. Net: **7 upheld, 3 rejected, 1 was this plan's own defect.**

- **A fourth reconcile obligation.** `design.md` §5's worked listing prints the binary
  `hygiene` row's *summary* while project and user both hold `hygiene`, which its own
  third-field rule — repeated as DEC-031 rule 4 — makes `shadowed`. §4's "one clause per
  resolved name" is likewise contradicted by its own per-file example (`hygiene (project);
  hygiene (user)`). Both are rendering statements in the locked design, so they are corrected
  at reconcile, in place, with the other three: **the example row must read `shadowed`, and
  §4's sentence should say per file.**
- **Rejected, with evidence.** §2's walk-up fallback cites DEC-028 *correctly* — DEC-028
  carries a section *The walk-up terminus* saying exactly that (the finding was
  confabulated). `seed(set)` having no `kind` is not a design gap: §5 fixes `source` as the
  literal `conventions/<name>.md`, identical at every layer, and the `<kind>/<name>.md` in
  this plan was the plan's error, corrected before this pass.
- **This plan's defect, not the design's.** The `.config/smut` segment: §2 line 64 says the
  two roots *are* the `…/smut` directories, §5's example path shows
  `<repo>/.config/smut/conventions/`, and DEC-026 fixes the suffix. The design assigns it;
  no phase of this plan pinned it. Two independent lanes called it a design blocker, which
  is the shape of error that a green suite would never have caught.
- **Recorded, not repaired.** `parse` deriving the name from `origin.path` (§6's signature
  admits no other reading); an unreadable layer directory riding `StoreError::FileSystem`
  (§6's one new variant cannot carry a file-less failure); a symlinked `.git` not being a
  repository to §2 although git resolves it (§2's rule is explicit; exposure exotic); the
  fourth field's absoluteness under a relative `$XDG_CONFIG_HOME`.
- **Settled by the operator: a known key with no value declares nothing.** `summary:`
  alone is absent, not malformed — the listing shows `-` and the seed omits the version.
  One branch of PHASE-02/EX-2, two pinned tests; a value continued across lines still
  fails naming its line, which is the mistake worth catching.

## Implementation run

Seeded before handover to `/super-orchestrator`; the seat's own preflight refreshes and
extends it. Facts here were observed this session at head `de8cf3d`.

- **Gate:** `just check` — build, clippy over both feature configurations with `-D warnings`,
  `fmt --check`, then the suite twice (default and `--features fault-injection`). A bare
  `cargo test` is not the gate; a VA row's narrower `cargo` command is evidence of one
  property, never a substitute.
- **Toolchain:** `rustc 1.100.0-beta.7`, edition 2024. The suite spawns the real binary via
  `CARGO_BIN_EXE_smut`; scratch trees go under `std::env::temp_dir()`, never
  `interview.local/.smut`.
- **Shared index, right now:** another writer has `README.md`, `.doctrine/backlog/idea/006*`
  **staged** in the index. Commits must be path-limited *with a pathspec*
  (`git commit -m … -- <paths>`) or run from paths just added by this run. A bare
  `git commit` would sweep their work; `git add -A`, amend and reset are out.
- **Doctrine state:** lifecycle `ready`; `doctrine slice verify-vt 008` is the gate at each
  phase conclude (0 UNCHECKABLE at handover); phase sheets under
  `.doctrine/state/slice/008/phases/` are disposable and gitignored.
- **Design run:** `dr-01a19b61` locked, `review_pass RV-029 STALE` and `changes 1 since the
  declared baseline` — the known post-pass residue. Advisory, not a gate; do not read it as
  coverage.

### Seat preflight — observed at `c9b979e`

- **Baseline gate is green.** `just check` at `c9b979e` exits 0 with no warnings: build, two
  clippy configurations, `fmt --check`, then the suite twice — 379 lib tests on default
  features and 392 under `--features fault-injection`. No known failures, no jail-only
  failures. A red gate from here is a regression.
- **Toolchain confirmed present:** `rustc`/`cargo` 1.100.0-beta.7, `just` 1.58.0. `just
  --dry-run check` confirms the six gate steps; `gate`/`prove` alias `check`, so a phase's
  cadence-name gate is the same non-mutating sequence.
- **Roster:** custom `sub-orchestrator` (nested spawn allowed), `capsule-orchestrator`,
  `capsule-worker`. Builtin `worker`, `delegate`, `scout`, `researcher`, `reviewer`, `oracle`,
  `evidence-auditor` are available to children as their `allowedAgents`.
- **Model posture:** `subagents` carries no `defaultModel` and every `agentOverrides` entry is
  empty, so children inherit the seat's model — `llm-gateway/deepseek-flash`, thinking high.
  PHASE-01 and PHASE-05 are chartered on that inherited setting. PHASE-02/03/04 are the
  frontier-shaped phases and are **held pending an operator pin**; the skill forbids the seat
  from switching a model on its own initiative.
- **Lifecycle ownership:** the sub-orchestrator owns `doctrine slice phase … --status` inside
  its charter (skill), so the seat does not pre-flip `PHASE-01`; each charter is briefed to
  flip its own phase to `in_progress` on entry and `completed` only once criteria, tests and
  the commit exist.

## Autonomy log

- **PHASE-01..05 — model envelope: inherit, all five. Operator ruling, not the seat's.**
  Asked once, at charter 1 (`6daf8f6e`), because `super-orchestrator` forbids the seat from
  switching a model on its own initiative and `subagents` carries no pins. Answered
  `llm-gateway/deepseek-flash (inherit)`. Every charter therefore runs on the configured
  default at thinking high — PHASE-02/03/04 included, though `plan.md` § Model requirement
  rates those frontier-shaped and calls PHASE-03 "the phase that must not go to a weak
  model". Decided: inherit. Standing, not per-charter: do not reopen it at charter 3.
  The `plan.md` escalation triggers survive the ruling unchanged — a phase that cannot hold
  its criterion list splits at its stated seam (P2's or P4's) or escalates, and appending a
  phase is always available; dropping rows, weakening a bound or handing back a green gate
  with a silent failure mode present is not. A frontier-shaped phase that reddens twice on
  the inherited model is a `contact_supervisor` item, not a re-charter on another model.
