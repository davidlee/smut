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
4. **`design.md` §5 and §4** — §5's worked listing prints a summary in its binary row where
   the third-field rule (repeated as DEC-031 rule 4) requires `shadowed`; §4's "one clause per
   resolved name" contradicts that same rule 4. In place: the example row must read
   `shadowed`, and the per-name clause restated to match. (Recorded in detail under
   § Harvest → Produced; listed here so the four obligations read as one set.)

### Design questions the PHASE-02 review raised (RV-030, judgement calls)

Not defects — the design or the operator owns them — but they are owed a ruling, and none of
this phase's work should invent one.

- **A. Duplicate frontmatter keys are unspecified and the last one silently wins.**
  `src/conventions.rs:323-326`: a second `summary:` overwrites the first, no refusal. DEC-028
  covers neither duplicates nor a refusal for them — but the repo refuses a duplicate key
  loudly in JSON (`Reject::DuplicateKey`, and the guide's own "so is a duplicate key inside
  one object"), and DEC-028's stated rationale for refusing `title:` *is* the silent-wrongness
  class, which a duplicated `summary:` is. Owed: settle in the same in-place DEC-028 revision
  as obligation 1, while the format is one phase old.
- **B. Three input shapes silently demote frontmatter to prose.** `--- ` with a trailing
  space, CRLF line endings, and a leading blank line all fail the "exactly `---`" test
  (`:222`, with an honest comment at `:219-221`), so a Windows-authored set prints its
  `summary:` as advice and nothing warns. Design-consistent, so not a defect — but it is the
  reader's one remaining silent-loss path, and EX-8's guide line covers only the *opposite*
  loss (printed prose omitting frontmatter). Owed: name it deliberately at reconcile, or buy
  it a one-clause addition to `src/learn.rs` in PHASE-04.
- **C. The body trim is asymmetric by design — and a conventional file trips it.** Trailing
  newlines are trimmed (`:244`); a leading blank line after the fence is preserved (`:241`).
  The shipped file is written without one, so it is invisible today, but the first
  user-authored set written the way Markdown conventionally is (`---\n…\n---\n\n# Title`)
  prints and seeds a body that opens with an empty line. Spec-consistent ("everything after
  the frontmatter … nothing else transformed"), and §7's byte-for-byte row will not catch it.
  Owed: a design question at reconcile, not a change made in a later phase.

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

**fresh-as-of:** `SL-008` lifecycle `started`, `PHASE-01` `completed` (roots landed,
gate green, VT-1/VT-2 PASS at head `c3385c2`), `PHASE-02` **`in_progress`** (the reader
landed green at `77444e8`, VT-1..VT-5 PASS, deliberately left open for its independent
review), design run `dr-01a19b61` locked at revision 36. Next stage: the `PHASE-02` review,
then `PHASE-03`; audit is not open.

### Produced

- **`PHASE-02` implemented and green at `77444e8`** — `src/conventions.rs` rewritten around
  `Layer`/`Origin`/`Cause`/`Set`/`Entry`/`Outcome` with the pure `parse(bytes, origin)`,
  `SHIPPED` as a `(name, bytes)` table read by that same reader, `Ask` owning its set,
  `StoreError::ArtefactNotLoaded` at exit 1, `hygiene.md` carrying its summary in
  frontmatter and its own `Conventions:` heading, one guide sentence on the frontmatter
  loss, and the shipped-surface assertions moved to the body below the fence.
  Six owned files, 1033 insertions / 121 deletions, one path-limited commit.
- **The recorded delta is `4015f6a..77444e8`** — exactly the phase's own patch. See
  § Implementation run → PHASE-02 for why it was narrowed with `--force`.
- **Five negative controls, each reddening exactly its intended test**, plus a sixth on the
  spawned surface. They are the phase's real evidence that the reader's tests bite.
- **A verifier correction of the worker's own green claim** — the gate caught a
  `clippy::cmp_owned` in the new inventory test that the worker's clippy run had reported
  clean (see § Learned).

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
- **PHASE-01 is implemented** — `Roots { project, user }`, `Roots::under` (the crate's only
  `.config/smut` join), `project_root` (`symlink_metadata` on `<ancestor>/.git`, inclusive
  walk, `start` as the terminus fallback) and `user_root` (non-empty config home, else
  `<home>/.config`, else `None`), with 10 in-module tests over temp-dir fixtures. Commits
  `289d179` (`feat`) and `c3385c2` (`test`), both path-limited to `src/conventions.rs`.
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

- **`record-delta --commit <S>` records exactly `[S^, S]`, and its guard fires when the
  phase sheet's stamp predates a foreign commit.** PHASE-02's stamp was `43ca052`, a
  `doc(sl-010)` commit (`4015f6a`, two `.doctrine/slice/010/` files, proven doc-only with
  `git show --stat`) landed between the stamp and the phase's code, so the safe default
  refused to narrow until `--force`. The narrow is the honest range: widening it to the
  stamp would hand the reviewer a range containing another slice's commit and make the
  review lie. Narrow only after proving the dropped commit carries none of this phase's
  code, and say so in the handback.
- **A mutation that does not compile is not a negative control.** The first version of the
  "version applied inside `parse`" mutant inserted a duplicate struct field, so the result
  was a compile error and the "no test failed" reading proved nothing; rewritten to replace
  the existing field, it reddened `the_shipped_set_parses` and four neighbours. Check that a
  mutant reddens a *test*, not the compiler.
- **A worker's clippy claim needs the gate behind it.** The session-B handback reported
  `cargo clippy --all-targets -- -D warnings` clean while the gate failed on
  `clippy::cmp_owned` in the new inventory test — a comparison against a freshly built
  `PathBuf`. One-line fix (`Path::new`); the lesson is that the gate, not the handback, is
  the evidence.
- **An exact-value assertion placed first makes every weaker assertion after it
  unfalsifiable.** A strength-ordered listing test (exact row, then `starts_with`, then
  `contains`) reads well and hollows out three assertions; the exact row belongs last, where
  each weaker row still has an input that reddens it.
- **The compile-forced coupling is exactly what EN-2 predicted.** `HYGIENE`'s retirement
  broke `src/main.rs:179` and four unit tests; nothing else in the crate referenced the old
  shape. A type replacement phase is one compile unit, so its "two sessions" are sessions of
  reasoning, not two independently green commits.

- **Origin printing and byte-identity cannot both hold.** DEC-026 promised a header naming the
  file; DEC-028 made a set's bytes load-bearing. Keeping the bytes and moving the origin to
  the listing was the whole hinge of the design.
- **Validity is not a tie-breaker.** A broken nearer set makes that name *unaskable*. Accepted
  on the operator's word ("fails loud and clearly") after being stated as a consequence.
- **A pure resolver needs its roots passed in**, and the leak it prevents is in the *unit*
  tests, not the CLI — `main`'s in-process `dispatch_of` reads real cwd and `$HOME`.
- Attestation is not coverage: a pass opened before late edits still reports `STALE` after the
  edits are re-attested.
- **`verify-vt` attributes by recorded delta, not by the file being on disk.** Before
  PHASE-01's delta was recorded, both its rows read `UNATTRIBUTABLE — keyword present but
  src/conventions.rs not modified by this slice`; the `completed` transition recorded a span
  covering three foreign `doc(sl-010)` commits and warned, so the range was tightened to
  `--start 289d179^ --end c3385c2` via `slice record-delta`. Both rows then read `PASS`. The
  command still exits 1 at this boundary because PHASE-02..05's keywords are legitimately
  absent.
- **A negative assertion the preceding assertion already guarantees is not evidence.** A
  worker's `assert_ne!(found, scratch.path)` after `assert_eq!(found, child)` could not fail
  once the fixture existed; it was reverted and the property was carried instead by a
  fixture that makes it falsifiable (`a_home_above_a_repository_is_not_the_project_root`).

### Open

- **`PHASE-02` is `in_progress`, awaiting its independent code/correctness review** (standing
  cadence). Green and committed; the seat concludes after the review lands and any
  `fix-now` finding is corrected.
- **`every_store_refusal()` omits `NoFreshBasename`** — pre-existing, and PHASE-02 added only
  its own row. The list's length is now pinned at 13 with a comment naming the gap, so the pin
  is honest rather than complete. Flagged, not fixed: repairing another phase's inventory is
  outside this phase's "one variant plus its three inventories".
- **`77444e8`'s commit message has a hole.** A backtick in the `-m` string ran command
  substitution, so one line reads "and  is the pure reader over bytes". `--amend` is
  forbidden on this shared index, so the blemish stands; the subject and every other line
  are intact and this paragraph is the record.
- **`doctrine slice research 008` still reports drift**, naming `plan.md` and `plan.toml`
  only — the sections written after the last restamp. Chronological, not evidential: the
  code-map hotspots it lists are precisely what PHASE-02 replaced, and PHASE-01's notes
  already re-verified the map at `d224dc6`. No restamp performed (`research.md` is outside
  the phase's six files); the seat may restamp.

- **QUE-002** — the environment override (name, target, rank).
- **IMP-008** — the section grammar and the `--with-conventions` flag; deliberately deferred,
  free to add later because the loader never interprets the body.
- **Reconcile obligations** (three governance amendments, in § Obligations owed at reconcile):
  DEC-028's malformed-file clause, DEC-026's "set header naming its own file", DEC-017's
  "a file plus a match arm".
- **RV-029 coverage residue** — the two hedge paragraphs (§5, §6 at revisions 30–31) are
  attested but no pass ran over them.
- **PHASE-01 residual — an empty `home` yields a relative layer root.**
  `user_root(Some(""), Some(Path::new("")))` returns `Some(".config/smut")`, because EX-2
  rules only that the *config* home be non-empty and says nothing about `home`. Unreachable
  from the binary — `main` passes `home_dir()`, which is absolute or `None` — so nothing is red
  today. But a relative path as a layer root would resolve against the cwd and read the wrong
  layer without a symptom: the PHASE-03 failure class in miniature. **Seat ruling:** the
  non-emptiness EX-2 already applies to `config_home` reads onto `home`, since an empty `home`
  resolves no config home either — that is what "neither resolves" already means, not a new
  rule. Pinned by one row in PHASE-04's owned `src/conventions.rs` (charter 4 brief), not by
  reopening PHASE-01. If audit reads it the other way, this paragraph is the record of what
  was decided and why.
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

### Review cadence — operator direction, 2026-11-02

- **A code/correctness review runs over PHASE-03 after its gate goes green** — "in any case",
  which means regardless of how clean the charter's self-report is. Operator direction, taken
  at charter 1.
- **Mechanism:** `/code-review` at its per-phase moment, landing on the RV ledger — a review
  on `SL-008` scoped to `PHASE-03` (`doctrine review new --facet code-review --target
  SL-008@PHASE-03`), primed from the slice selectors, findings raised with honest severity,
  disposed before the phase concludes. Not prose in a handback: the finding must outlive the
  conversation and be queryable at audit.
- **Independence:** a fresh child reviews. Not the charter that wrote the code, and not the
  seat re-litigating the diff — the seat's job is to raise what the reviewer returns and
  decide dispositions.
- **Sequencing, so "before the phase concludes" stays true:** charter 3 ends at *green gate on
  a committed* `src/conventions.rs` and deliberately leaves `PHASE-03` `in_progress`. Review →
  fix-now corrections land while the tree is hot → only then `completed`, then charter 4.
- **Review target is a commit range, not a file.** `src/conventions.rs` carries PHASE-01, -02
  and -03 changes, so the seat records the pre-PHASE-03 oid at charter time and the reviewer is
  handed that range; a review of the whole file would re-raise P1/P2 as if they were new work.
- **PHASE-02 and PHASE-04 get the same cadence.** This is the code-review skill's own default
  at a below-adherence-bar model tier — which the inherit ruling made universal — and P2/P4 are
  the frontier-shaped phases. PHASE-01 and PHASE-05 stay discretionary: both are small and P1's
  trap is pinned by a named test. Logged by the seat as verification ownership (envelope:
  decide, log, continue), not as a change to the plan or design.
- **Severity honesty applies both ways** — `minors`/`nits` are dispositioned `follow-up` to the
  backlog rather than repaired inside the phase, and a clean review is reported as clean.

### PHASE-01 — observed at `c3385c2`

- **Commits, both path-limited with a pathspec (`-- src/conventions.rs`):** `289d179`
  `feat(sl-008)` — the four items plus 9 tests, 339 insertions, 0 deletions;
  `c3385c2` `test(sl-008)` — the F-17 home row, 31 insertions, 0 deletions. Index
  untouched otherwise: the other writer's staged `README.md` was still staged after both
  commits, `--amend`/`add -A`/reset never run, transient HEAD moves (three `doc(sl-010)`
  commits) absorbed without rebase.
- **Gate:** `just check` exit 0 after each commit — 389 lib tests on default features and 402
  under `--features fault-injection` (baseline 379/392, +10 from this phase), zero warnings in
  both clippy configurations, `fmt --check` clean. `pre-existing failures: none`.
- **`doctrine slice verify-vt 008`:** PHASE-01 `VT-1 PASS`, `VT-2 PASS`; the command still
  exits 1 because PHASE-02..05 keywords are legitimately absent (`pub fn parse`, `fn
  the_four_derived_keys_are_refused_by_name`, `pub fn load`, `pub fn answer(roots`, `fn
  the_seed_omits_a_version_a_file_does_not_declare`). Attribution needed a recorded delta
  first — see § Harvest → Learned.
- **VA-2 — the EN-1 greps, re-run beside the new code.** Before PHASE-01 both returned
  nothing in `src/` (observed at `976f9dc`, exit 1 each). After it, `grep -rn '\.git\b' src/`
  hits only the new predicate (`src/conventions.rs:78`, `symlink_metadata(ancestor.join(".git"))`),
  its doc prose, and the fixtures; `grep -rn 'home_dir\|XDG\|config_dir' src/` hits only one
  doc-comment line naming `XDG_CONFIG_HOME`/`HOME` as the binary's to read. No `std::env`, no
  `Command`, no `fs::read`/`read_to_string`/`read_dir`, no `unwrap`/`expect`/`panic!` in the
  new non-test code — the only filesystem call is `symlink_metadata`.
- **Deferred/out-of-scope:** none found. The five reconcile obligations (DEC-028's
  malformed-file clause, DEC-026's cost sentence, DEC-017's "a file plus a match arm",
  `design.md` §5's worked listing, §4's per-name clause) were not touched and this phase
  surfaced nothing new about them.
- **Left dirty for the seat:** `.doctrine/slice/008/slice-008.toml` (lifecycle `ready →
  started`, written by the CLI) and another writer's untracked `.doctrine/slice/010/research/`.
  The phase sheet under `.doctrine/state/` is gitignored.

### PHASE-02 — observed at `77444e8`

- **One commit, path-limited with a six-path pathspec:** `77444e8` `feat(sl-008)` over
  `src/conventions.rs`, `src/conventions/hygiene.md`, `src/error.rs`, `src/learn.rs`,
  `src/main.rs`, `tests/conventions.rs` — 1033 insertions, 121 deletions. The other writer's
  staged `README.md` and their untracked `_envrc` were untouched; no `--amend`, `add -A` or
  reset. HEAD moved once mid-phase (`4015f6a`, `doc(sl-010)`) and was absorbed without a
  rebase. The message has one blemish — see § Harvest → Open.
- **Red observed before green:** the test-author run added 17 unit tests against the
  unimplemented API and the lib test target failed with 58 errors of exactly the expected
  classes (missing `Layer`/`Origin`/`Cause`/`Set`/`Entry`/`Outcome`/`parse`/`SHIPPED`/
  `binary_layer`/`listing_of`, the old `Set` fields, `StoreError::ArtefactNotLoaded`).
- **Negative controls (six, each reddening exactly its intended test):** blank-line skip
  removed → `a_blank_line_inside_the_fence_is_skipped`; `type`'s sentence replaced by
  `name`'s → `the_four_derived_keys_are_refused_by_name`; the dash rendered empty →
  `a_set_that_declares_no_summary_lists_with_a_dash`; the fence allowed anywhere →
  `a_horizontal_rule_opens_no_frontmatter`; the crate version applied inside `parse` →
  `the_shipped_set_parses` plus four neighbours; the prose printing a fence line →
  `a_named_set_prints_the_body_verbatim` and the bins test `conventions_prints_the_shipped_set_without_a_store`.
- **Gate at `77444e8`:** `just check` exit 0 — 408 lib tests on default features and 421 under
  `--features fault-injection` (baseline 389/402, +19 each), both clippy configurations clean
  under `-D warnings`, `fmt --check` clean. `cargo test --all-targets` green (EX-10's only
  possible evidence: a retired `HYGIENE` cannot compile past an unrewritten `src/main.rs`
  test, which is exactly what the pre-phase tree did).
- **`doctrine slice verify-vt 008`:** PHASE-02 `VT-1..VT-5 PASS`; PHASE-01's two rows still
  `PASS`; PHASE-03..05 `FAIL`/`UNATTRIBUTABLE` on their legitimately absent keywords.
  Attribution needed the recorded delta first — before it, PHASE-02's rows read
  `UNATTRIBUTABLE`.
- **VA-2 — the module read for ambient reach.** No `std::env` read, no `std::fs` call except
  PHASE-01's `symlink_metadata`, no `Command`, no `unwrap`/`expect`/`panic!`/`unreachable!` in
  any new non-test code (the only environment is the `env!("CARGO_PKG_VERSION")` build-time
  constant, in `binary_layer`); `parse` touches no filesystem at all. RV-029 F-1/F-2/F-6/F-8/
  F-15/F-20 are each discharged by a named line or test — see the handback table, also
  recorded in the phase sheet's § Findings.
- **Verifier corrections after the workers' green (logged, not rebriefed):** the exact listing
  row moved to the end of its test so the four weaker rows above it stay falsifiable; one
  `clippy::cmp_owned` in the new inventory test fixed (`PathBuf::from` → `Path::new`), which
  the gate caught and the worker's own clippy run had missed; the inventory-length assertion's
  message reworded so it does not claim a completeness it does not have.
- **Deferred/out-of-scope:** the five reconcile obligations were not touched and this phase
  surfaced nothing new about them. `every_store_refusal()`'s pre-existing omission of
  `NoFreshBasename` is flagged, not fixed.
- **Left dirty for the seat:** `.doctrine/slice/008/slice-008.toml` (the `in_progress` flip)
  and another writer's untracked `_envrc` / `.doctrine/slice/010/`.

## Autonomy log

- **PHASE-02 — the phase was split by compile unit, not by the charter's editorial seam.**
  The charter put `SHIPPED`, the binary-layer helper, `ArtefactNotLoaded` and `hygiene.md` in
  session B, but B cannot compile without A (the new `Set` has no `HYGIENE`) and A cannot
  compile without the refusal variant (its `resolve` needs it) or without a parseable shipped
  file. Decided: A = `src/conventions.rs` + `src/conventions/hygiene.md` + `src/error.rs`;
  B = `src/main.rs` + `src/learn.rs` + `tests/conventions.rs`. Two sessions of reasoning kept;
  one compile unit, so one commit.
- **PHASE-02 — `listing()` keeps its no-argument signature; the renderer is a private
  `listing_of(&[Entry])`.** PHASE-04/EX-2 introduces `listing(entries)`; leaving the public
  name free keeps that VT keyword (`fn listing(entries`) honest instead of already satisfied.
  Decided.
- **PHASE-02 — the recorded delta was narrowed past a foreign commit with `--force`.**
  `record-delta --commit 77444e8` records exactly `4015f6a..77444e8`; the guard fired because
  the phase-sheet stamp was `43ca052` and `doc(sl-010)`'s `4015f6a` sat between. Proven
  doc-only (`git show --stat 4015f6a` → two `.doctrine/slice/010/` files), so sweeping it into
  the review range would have made the review lie. Decided, evidence in § Harvest → Learned.
- **PHASE-02 — three verifier edits after the workers' green** (assertion ordering, the
  clippy `cmp_owned` fix, the inventory message). Decided: minute, mechanical, inside the
  owned files, and each logged here rather than rebriefed.
- **PHASE-02 — the workers' clippy claim was wrong and the gate caught it.** Recorded as a
  process lesson rather than a finding: the gate, not a handback, is the evidence. Decided.
- **PHASE-02 — `every_store_refusal()`'s missing `NoFreshBasename` row deferred.** Adding it
  is not this phase's "one variant plus its three inventories", and the pinned count now
  names the gap. Deferred to the reviewer / backlog.
- **PHASE-02 — the research advisory's drift left alone.** It names `plan.md`/`plan.toml`
  (post-restamp plan text), not a code surface; `research.md` is outside the phase's six
  files. Deferred to the seat, which may restamp.
- **PHASE-02 — the phase was left `in_progress` on purpose**, per standing review cadence:
  green, committed, delta recorded, and the seat concludes after the independent review.
  Decided (operator cadence, not this charter's choice).

- **PHASE-01 — three mutations used as negative controls, each reddening exactly the intended
  test.** `fs::metadata` for `symlink_metadata` → the `.git`-symlink row; `Roots::under`
  joining `<repo>/conventions` → `the_layer_roots_are_the_two_smut_directories`; the walk
  fallback returning `/` → `no_repository_above_leaves_the_start_directory_as_the_root`.
  Decided: keep the fixtures, no criterion change.
- **PHASE-01 — `.config/smut` written as one literal.** `Roots::under` joins the single
  segment `.config/smut` rather than two chained joins: same path, but it is the wording EX-3
  and DEC-026 use and it keeps "exactly one join" greppable. Verifier edit after the worker's
  green, logged rather than rebriefed.
- **PHASE-01 — a second, `test(sl-008)` commit.** RV-029 F-17's directory-level half (EN-3)
  was implied but never fixtured; the home-above-a-repository row now builds a real competing
  layer and pins the consequence through `Roots::under`. Appended after the phase was flipped
  `completed`, so the recorded registry range was re-tightened to cover both commits.
- **PHASE-01 — one tautological assertion removed.** Reverted with its fixture; the property
  is re-homed in a falsifiable row (see § Harvest → Learned). Not a test weakening.

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
