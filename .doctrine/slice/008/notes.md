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

**fresh-as-of:** `SL-008` status `plan`, design run locked at revision 36, head `d4f593f`
plus this commit. Captured before handover to `/plan`.

### Produced

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
