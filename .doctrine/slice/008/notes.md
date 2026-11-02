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
5. **`design.md:156`, the worked "unknown conventions set" example** — its inner text quotes
   the diagnostic `frontmatter line 2 is not `key: value``, which PHASE-02 corrected to a
   sentence true of every input that reaches it ("is not a flat `key: value` line: the key is
   one word with no space in it, and a `:` follows it"). The example illustrates clause shape
   rather than exact wording, so this is a staleness fix, not a rule change: update the quoted
   sentence in place, or drop the quotation and name the class.
6. **`design.md` §6 (and EX-4, which restates it) has no tie-break among unnameable entries.**
   "Sorted by name … a `name: None` entry sorts last … sorting is by the decoded name only" is
   total only while a layer holds at most one undecodable filename: with two, the decoded key is
   equal twice and the order is `read_dir`'s — which is exactly what the rule exists to prevent
   (RV-029 F-16). PHASE-03 ruled the tie-break as the raw filename, compared as an `OsStr`;
   § Autonomy log carries the reasoning and the alternatives. Owed: one sentence in §6.
7. **`design.md` §3 does not say what a *candidate* whose metadata cannot be read is.** Its
   failure table lists "cannot be opened or read" as a reasoned row, while its candidate filter
   requires "a regular file" — so a dangling `x.md` is either a row (§3's table) or nothing
   (the filter). PHASE-03 ruled it a failed row, which is also the only portable, root-proof
   fixture for that class; a directory whose name is undecodable stays silent because the
   metadata test runs first. Owed: one sentence in §3.

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
gate green, VT-1/VT-2 PASS at head `c3385c2`), `PHASE-02` green at `77444e8` with its review
(RV-030) discharged in `8503b93`, `PHASE-03` **`in_progress`** — the loader landed green at
`cfae50f`, `just check` exit 0, `PHASE-03 VT-1 PASS`, delta `9829de0..cfae50f`, deliberately
left open for its independent review. Design run `dr-01a19b61` locked at revision 36. Next
stage: the `PHASE-03` review, then `PHASE-04`; audit is not open.

### Produced

- **`PHASE-03` implemented and green at `cfae50f`** — `load(roots, kind) -> Result<Vec<Entry>,
  Error>` and `set_in_force(entries, name)` with a private `read_layer` and `sort_layer`, three
  path-limited commits (`b3c2fd9`, `84a8155`, `cfae50f`), 14 pinned rows plus the six negative
  controls above. The loader is written and pinned before its consumer, as EX-8 requires.
- **The attempt-1 gap, stated and ruled in § Autonomy log**: four outcomes collapsed into one
  (absent / unreadable layer / unreadable file / non-UTF-8 bytes / non-UTF-8 filename), and
  EX-4's missing tie-break between two undecodable names, resolved as decoded name → `None`
  last → raw filename. Two design sentences are owed to reconcile (below, items 6 and 7).
- **`src/error.rs`'s `FileSystem` doc comment widened** from store boundaries to layer reads —
  the one change EX-2 authorises outside `src/conventions.rs`, committed separately at
  `9281f64` because it is a doc-only tail that three foreign `doc(sl-010)` commits now sit
  between (see § Implementation run). Carried in from attempt 1's diff and kept as written:
  `step` now also names "the layer read that failed (`reading the conventions layer`)". No
  variant, no inventory row and no rendering changed, so the `every_store_refusal()` count pin
  is untouched.
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

- **A doc comment that decides a question is a decision nobody made.** Attempt 1's `load` doc
  said "this phase makes 'absent' and 'unreadable' the same thing, so nothing here errs yet" and
  its `read_layer` doc said "Nothing is sorted or deduplicated here" — and those two sentences
  were the whole of its response to two criteria that say the opposite. A contradiction written
  into prose reads as an implementation note, so it survived a whole run; the same contradiction
  in code (a bare `Vec<Entry>` return, a `continue` on every error) is what the tests caught.
  The lesson for the next charter: when a criterion and the code disagree, the doc comment is
  where the disagreement gets hidden, so it is the first thing to read.
- **A test that is green on arrival is a pin, not evidence, until a mutation reddens it.**
  `a_broken_set_still_wins_its_name` passed the moment it was written, because `set_in_force`
  already returned failed entries. It only became evidence when `set_in_force` was filtered to
  `Loaded` entries and the row went red — so a pinned name in VT-1 that is satisfiable before
  the phase starts is a name the phase has to *prove* bites.
- **A negative control's blast radius can be honest without being narrow.** The double-join
  mutant reddens 14 rows, because deleting the project layer is a bigger change than the
  duplicate segment: `the_kind_is_joined_once` names the defect precisely, and the other 13
  catch it as collateral. Reporting the breadth is the honest move; engineering the mutant so
  that only one row reddens would have proved less.
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

- **`PHASE-03` is `in_progress`, awaiting its independent code/correctness review** (standing
  cadence). Green and committed at `cfae50f`, `just check` exit 0, delta `9829de0..cfae50f`
  recorded, and the seat concludes after the review lands and any `fix-now` finding is corrected.
  The reviewer is handed the range, not the file: `src/conventions.rs` also carries PHASE-01 and
  PHASE-02, and reviewing the whole file would re-raise their findings as if they were new.
  Baseline for the range is the phase sheet's `code_start_oid` (`bfe98e2`).
- **Two design sentences are owed to reconcile and are *not* fixed here** — §6's missing
  tie-break among unreadable names and §3's silence on a candidate whose metadata cannot be
  read (items 6 and 7 of § Obligations owed at reconcile, with PHASE-03's rulings).
- **`PHASE-03`'s `Box::leak` on the exit-1 path is a deliberate, bounded leak** — one small
  allocation per failed layer read, in a process about to exit 1, because
  `StoreError::FileSystem.step` is `&'static str` and EX-2 authorises only a doc comment in
  `src/error.rs`. If a later phase widens that field (a `Cow`, or a `String` carrying the step),
  the leak should be retired with it. Recorded, not scheduled.
- **`PHASE-02`'s `in_progress`-awaiting-review note is discharged**: RV-030 was raised, disposed
  all-`fix-now` and corrected in `8503b93`; the phase is `completed` in the registry.
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
- **Review — RV-030 over `4015f6a..77444e8`: four findings, no blocker, all `fix-now`,
  discharged in `8503b93`.** F-1 (`minor`) the inventory pin's message claimed a guarantee it
  cannot give — 14 `StoreError` variants against 13 rows, `NoFreshBasename` absent since it was
  introduced; F-2 (`minor`) three assertions sat above an exact equality they could not
  falsify, the repository's own recorded lesson; F-3 (`nit`) the unterminated-fence class
  pinned only "reason is non-empty", so `line: None` survived an EX-3 requirement; F-4 (`nit`)
  a diagnostic told the reader that a `key: value` line was not one. The seat verified all four
  by reading the corrective diff, not the handback. F-4's fix improves on the suggestion it
  was raised with — the reviewer's own wording would have been false for the empty-key and
  no-colon paths that reach the same sentence, so the worker probed all three and worded it
  true of every one.
- **The review's own limits, recorded rather than glossed.** The `reviewer` role has no shell:
  it could not see the commit range and did not run the gate, and it said so instead of
  assuming them. The seat closed the two material gaps — EX-7 (the prose below the frontmatter
  is byte-identical to `4015f6a`'s body) and EX-9 (the one dropped test,
  `a_named_set_prints_the_embedded_prose_verbatim`, has its replacement
  `a_named_set_prints_the_body_verbatim` carrying the property). Both hold. The gate was
  re-run at the seat: exit 0, 408/421, zero warnings.
- **Role collapse, stated plainly:** the seat raised, disposed and verified RV-030's findings
  because the reviewer could only return prose. The ledger records every turn truthfully, so
  nothing is hidden — but no party was independent of every step. The independent check on
  this phase is the audit's pre-close review, which reads RV-030 rather than these notes.

### PHASE-03 attempt 1 — timed out, partial diff inherited

`44338764` hit the 30-minute wall-clock ceiling at 101k tokens / 109k peak context and exited
non-zero. It did not fail on the work: it was mid-TDD and had a **decision request in flight**
when it was killed, of which only one line survived — *"All three tests are red in the expected
ways. Before finalizing, I hit one design gap worth a decision rather than a silent deviation."*
The gap itself was never delivered. That is the operational lesson: a charter that ends only at
its boundary loses everything if it times out, so attempt 2 commits each green cycle.

**Inherited partial state, uncommitted, adopted by attempt 2 rather than reverted:**
`src/conventions.rs` +731 lines and `src/error.rs` +7/−5 on top of `31de296`. `load`,
`set_in_force` and `read_layer` exist; 42 `conventions::` tests pass; the gate is red on
exactly three — `a_non_utf8_filename_is_a_failed_row`, `an_undecodable_name_sorts_last`,
`each_layer_is_sorted_by_name`. `PHASE-03` is `in_progress`, nothing committed.

**Two contradictions in that diff, which are the likely gap.** Both are stated in the code's own
doc comments, which is how the dead run was thinking out loud:

1. `load`'s doc says *"This phase makes 'absent' and 'unreadable' the same thing, so nothing
   here errs yet"* and `read_layer` returns a bare `Vec<Entry>`, swallowing every error. EX-2
   requires the opposite: absent contributes no entries and no error, while
   existing-but-unreadable refuses at exit 1 through `StoreError::FileSystem { step, path,
   source }`. `load`'s `Result` exists for exactly this branch, so a signature that never errs
   contradicts the criterion that shaped it. This is silent-failure mode 1 of the five, written
   into the code rather than merely missed.
2. The doc says *"Nothing is sorted or deduplicated here"*, while EX-4 requires each layer
   sorted by name with `None` **last** — and here is the gap worth a ruling: EX-4 also says
   *"sorting is by the decoded name only"*, which gives **no** order between two undecodable
   names. With two of them in one layer the listing falls back to `read_dir` order, defeating
   the determinism EX-4 exists to guarantee (RV-029 F-16). The seat's reading, offered to
   attempt 2 as a hypothesis to confirm or replace rather than as a ruling it may assume: the
   decoded name orders what has one, `None` sorts last, and ties among `None`s break on the raw
   filename bytes — a tie-break in an unspecified case that preserves both the stated rule and
   its purpose, which is the envelope's "design clarification preserving scope, decisions and
   behaviour".

Attempt 2 is a fresh charter rather than a resume: the dead run was at 109k peak context and
had already exhausted the wall clock, so handing the same saturated context another 30 minutes
is the least likely route to green. The state that matters is in the tree, and the tree can be
read.

### PHASE-03 attempt 2 — observed at `cfae50f`

- **The inherited diff was adopted, not reverted**, and the two contradictions were corrected:
  attempt 1's `load`, `set_in_force`, `read_layer`, fixtures and 42 green tests survive, and only
  the two doc-comment contradictions were replaced by the behaviour the criteria name. The gap
  statement attempt 1 died carrying, and the rulings on it, are § Autonomy log's first four
  entries — written to disk before the first line of this attempt's code.
- **Three path-limited commits, each green before it landed** (`cargo test --lib conventions::`
  45 → 47 → 50 passed): `b3c2fd9` the per-layer sort with `None` last plus the undecodable
  filename as a pre-read failed row (this commit also carries attempt 1's 731 inherited lines,
  which is why it reads as +780); `84a8155` the absent/unreadable split and the mode-000 second
  row; `cfae50f` the two failure rows below the read plus `a_broken_set_still_wins_its_name`. No
  bare `git commit`, `add -A`, `--amend`, reset, stash or checkout was run; the other writer's
  `README.md` was still staged after all three, and `doc(sl-010)`'s edits to
  `.doctrine/slice/010/` were left untouched.
- **Recorded delta: `9829de0..cfae50f`** via
  `slice record-delta 008 PHASE-03 --start 9829de0 --end cfae50f`. `--start` is `b3c2fd9^`, the
  last commit before this phase's code landed (attempt 1's own notes commit); `--end` is
  `cfae50f`, the tip of `src/conventions.rs`. The `completed` transition was not used to record
  it — the phase stays `in_progress`, and that transition recomputes and overwrites a span.
  `--force` is not meaningful with `--start/--end` (ISS-317) and was not needed.
- **The phase's tail is deliberately outside the recorded range, and the reason is the PHASE-02
  lesson read backwards.** `src/error.rs`'s doc-comment widening (EX-2's one authorised change
  outside `src/conventions.rs`) landed in `9281f64`, and the notes record in `6a7abf9`; between
  `cfae50f` and those two, another writer landed three `doc(sl-010)` commits (`deef11e`,
  `75e4dd2`, `90eec66`). Extending `--end` to `9281f64` would hand the reviewer a range
  containing SL-010's plan, which is exactly what PHASE-02 narrowed away with `--force` in the
  other direction. So the range stays the code delta, and the reviewer also reads `9281f64`
  directly: four lines of doc comment in one file, no variant, no inventory row, no rendering.
  A moving HEAD interleaving this slice's commits with a neighbour's is the normal state here,
  not a fault in either run.
- **`just check` exit 0** at `cfae50f`: build, clippy over both feature configurations with
  `-D warnings` (zero warnings in either), `fmt --check` clean, then the suite twice — **424 lib
  tests on default features and 437 under `--features fault-injection`** (baseline 408/421,
  +16 each from this phase). `pre-existing failures: none`; no jail-only failures.
- **`doctrine slice verify-vt 008` reports `PHASE-03 VT-1 PASS`**; PHASE-01's two rows and
  PHASE-02's five still `PASS`; PHASE-04's VT-1..VT-4 `FAIL` and its VT-5/VT-6 `UNATTRIBUTABLE`,
  PHASE-05's two `FAIL` — all on symbols those phases have not written yet. The command exits 1
  at this boundary for exactly that reason (PHASE-01's and PHASE-02's notes record the same)
  and not because of anything PHASE-03 owns.
- **EX-8 — the binary references neither function.**
  `grep -rn "conventions::load\|conventions::set_in_force\|set_in_force" src/main.rs src/cli.rs
  src/learn.rs tests/` → no matches (exit 1). The `load()` hits elsewhere in `tests/*.rs` are the
  store's own loader. `resolve`, `answer`, `listing` and `seed` are untouched by these commits.
- **VA-2 — the module read for the two properties source cannot show.** `grep -n "fs::"
  src/conventions.rs` returns four non-test sites: `:83`, `project_root`'s `symlink_metadata`
  (PHASE-01's `.git` predicate — a stat, not a read of a set), and `:469`/`:512`/`:544` inside
  `read_layer` — `read_dir`, `metadata`, `read`. `read_layer` is private and its only callers are
  `load`'s two lines (`:402`, `:406`), so every set read in the module is inside `load`;
  `binary_layer` reads bytes the build embedded, and `parse`, `listing_of`, `set_in_force` and
  `seed` touch neither filesystem nor environment. The absent-vs-unreadable branch is `:470-471`
  (`Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new())`)
  against `:472-486` (the `StoreError::FileSystem` refusal) — different paths, and
  `an_absent_layer_is_not_an_error` / `an_unreadable_layer_is_an_error` are what keep them apart.
- **Six negative controls for the five silent failure modes**, each mutation applied to
  `src/conventions.rs`, the whole `conventions::` suite run, and the file restored
  byte-identically (`git status` clean for the file after each):

  | # | mutation | reddened |
  | --- | --- | --- |
  | 1 | the `NotFound` guard widened to every `Err` (unreadable as absent) | `an_unreadable_layer_is_an_error` only; `an_absent_layer_is_not_an_error` stayed green |
  | 2 | the pre-read name decision removed and `stem_of` made lossy | `a_non_utf8_filename_is_a_failed_row` (listed as `na\u{fffd}me`) and `an_undecodable_name_sorts_last` — two rows, one property seen from two sides |
  | 3a | `sort_layer` a no-op (`read_dir` order leaking) | `each_layer_is_sorted_by_name` only |
  | 3b | `sort_layer` sorting `Option<String>` directly (`Option`'s own `Ord`) | `an_undecodable_name_sorts_last` only (`[None, Some("a"), …]`) |
  | 4 | `roots.project.join(kind).join(kind)` | `the_kind_is_joined_once` **and 13 other rows** — reported honestly: a double join deletes the whole project layer, so every project-layer row reddens, not only the genericity row that names the defect |
  | 5 | the crate's version written onto *every* loaded entry in `load` | `the_binary_layer_carries_the_crates_version` only (`authored` read `0.2.2` where it declares `9.9.9`) |
  | 6 | `set_in_force` filtered to `Loaded` entries (validity as a tie-breaker) | `a_broken_set_still_wins_its_name` only |

  Six mutations for five modes because mode 3 has two halves — no sort, and a sort in the wrong
  order — and each half has its own row. Control 6 exists because
  `a_broken_set_still_wins_its_name` was green on arrival (`set_in_force` already returned
  failed entries): a pin with no red is not evidence until a mutation reddens it, so one was
  run.
- **Left dirty for the seat, deliberately:** `.doctrine/slice/008/slice-008.toml` (the phase's
  `in_progress` flip and the registry write, both the CLI's), another writer's staged
  `README.md` and their untracked `_envrc`/`.quotient/`, and `doc(sl-010)`'s uncommitted
  `.doctrine/slice/010/plan.{toml,md}`. None was staged or committed.
- **PHASE-03 stays `in_progress`.** Green, committed and delta-recorded, but the independent
  code/correctness review runs over this diff before the phase concludes — on this phase the
  review is the point, per the standing cadence.

## Autonomy log

- **PHASE-03 attempt 2 — the gap attempt 1 died carrying, stated in full and ruled.** Two
  under-determinations sit inside EX-2/EX-3 and EX-4. The inherited diff did not ask about
  either; it wrote the wrong answer into its own doc comment, and the doc comment is what a
  reader would have believed.

  *Gap A — five outcomes collapsed into one.* Inherited `read_layer` returns a bare
  `Vec<Entry>` and swallows every failure: a `read_dir` error and an absent directory both
  yield no entries, and a stat failure, an unopenable file and a non-UTF-8 *filename* all
  yield silence. EX-2 and EX-3 name five outcomes and the code has one — absent (no entries,
  no error), a layer that exists and cannot be read (exit 1 through
  `StoreError::FileSystem`), an unopenable/unreadable file (a `Failed` row carrying the
  operating system's own words and the path), bytes that are not UTF-8 (a `Failed` row with
  §3's fixed sentence *the file is not UTF-8 text*), and the filename that is not UTF-8
  (`name: None`, decided before any read). `load`'s `Result` exists for the layer branch
  alone, so a signature that never errs contradicts the criterion that shaped it. **Ruling:
  the criteria win and each outcome gets its own branch** — not a supervisor item, because
  EX-2 states the rule and the code contradicts it, which is a defect and not a choice. The
  one judgement call inside Gap A is the discriminator for "absent": `io::ErrorKind::NotFound`
  from `read_dir` is the absent case and *every other* `Err` is exit 1, `NotADirectory`
  included — a layer path whose parent is a regular file is a malformed root and fails loud
  rather than reading as "this project has not opted in". Chosen because silence is this
  phase's failure class, and because no fixture in the crate can produce a non-`NotFound`
  error for a genuinely absent layer.

  *Gap B — EX-4 supplies no order between two undecodable names, and this is the item that
  needed a ruling rather than an implementation.* EX-4 says each layer is sorted by name with
  `None` last, and that "sorting is by the decoded name only". Two entries whose names are
  both `None` have equal keys, so a stable sort leaves `read_dir`'s order — and EX-4's own
  next clause says the `None`-last rule "makes that order total". A key-only reading cannot
  make the order total; it can only make it *mostly* total, and where it does not, the
  listing is `read_dir`'s and RV-029 F-16 is back. **Ruling (the seat's hypothesis, confirmed
  rather than merely assumed):** the decoded name orders what has one, `None` sorts last, and
  ties among `None`s break on the raw filename, compared as an `OsStr` (bytewise on unix).
  Reasons: (i) it preserves EX-4's stated rule exactly — the decoded name is what orders
  entries that have one, and nothing decoded is consulted for entries that have none; (ii) it
  preserves EX-4's purpose, determinism of the listing, which is what rules out "leave the
  tie to `read_dir`", since an unspecified tie is an unspecified listing; (iii) the tie is
  unreachable unless one layer literally holds two undecodable filenames, so the clarification
  changes no behaviour any criterion states; (iv) the envelope's "design clarification
  preserving scope, decisions and behaviour" is exactly this — a tie-break in an unspecified
  case. Alternatives considered and rejected: sorting on `Option`'s derived `Ord` (`None`
  first, contradicts EX-4, and `an_undecodable_name_sorts_last` exists to catch it); dropping
  undecodable names out of the ordering question (the same non-determinism, spelled shorter);
  keying on `Origin`'s full path instead of the filename (identical order within one layer, so
  the same rule with more words).

  Two sentences are owed to reconcile rather than fixed here, because both are design text:
  §6 does not say how a tie among unnameable entries breaks, and §3 does not say what a
  *candidate* whose metadata cannot be read is.

- **PHASE-03 attempt 2 — `StoreError::FileSystem.step` is `&'static str` and the kind is a
  parameter.** EX-2 requires `step` to name the layer read (`reading the conventions layer`),
  which cannot be a literal for the second kind the genericity pin loads. Ruled: `Box::leak`
  one `format!`, on the exit-1 path only, never on a successful load. Rejected: a literal
  `conventions` (false for kind `shapes`, and `a_second_kind_uses_the_same_loader` exists to
  make that falsehood reachable); widening the field to a `Cow` (touches every `FileSystem`
  construction site in `store.rs`, outside this phase's two files, and EX-2 authorises only the
  doc comment in `error.rs`). The leak is one small allocation in a process that is about to
  exit 1.

- **PHASE-03 attempt 2 — `x.md` whose metadata cannot be read is a failed row, not silence.**
  EX-2's filter is "a regular file", so a dangling symlink is arguably not a candidate at all.
  Ruled the other way: §3's failure table names "cannot be opened or read" as a reasoned row
  with the operating system's words and the path, and this phase's whole subject is failure
  classes that go silent. Consequences recorded honestly: the metadata test runs *before* the
  name decode, so a directory whose name is undecodable is still not an entry (the filter's
  "directories are not entries" half is untouched); and this row is what makes
  `a_file_that_cannot_be_read_is_a_failed_row` provable at all, since a mode-000 file cannot
  fail as root and no other portable fixture for the class exists.

- **PHASE-03 attempt 2 — the binary layer is sorted like the other two.** The attempt-1 phase
  sheet's decision 4 kept `SHIPPED`'s table order; EX-4 says "each layer's entries are
  sorted". Immaterial with one shipped pair, so the two readings cannot differ in behaviour
  today; the uniform rule is chosen so that a second shipped set cannot create an unsorted
  layer. Supersedes decision 4 of the attempt-1 sheet.

- **PHASE-03 attempt 2 — the inherited diff was adopted, not reverted.** `load`,
  `set_in_force`, `read_layer` and 42 green `conventions::` tests are a predecessor's real
  work: what was right (the layer order, the candidate filter, the version passthrough, the
  fixtures) was kept, and only the two contradictions above were corrected. No `checkout`,
  `reset` or `stash` was run against it.

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

- **PHASE-03 attempt 2 — six mutations for five failure modes, and the sixth admitted as one.**
  Failure mode 3 has two halves (no per-layer sort, and a sort in the wrong order), and each
  half has its own pinned row, so both were run. A seventh was added for
  `a_broken_set_still_wins_its_name`, which was green on arrival: a pin nobody can redden is not
  evidence. Decided (verification ownership) and recorded in § Implementation run as a table.
- **PHASE-03 attempt 2 — the double-join control's breadth reported rather than engineered
  away.** It reddens 14 rows. Narrowing it so only `the_kind_is_joined_once` failed would have
  meant a contrived mutant that joins twice only when the first join is empty; the honest
  reading — a duplicated segment deletes the layer, and one row names the cause — is worth more
  than a tidy table. Decided.
- **PHASE-03 attempt 2 — `84a8155`'s commit message has a typo** ("Nowread_dir's", a lost
  space). `--amend` is forbidden on this shared index, so the blemish stands and this line is
  its record; the subject and every other line are intact.
- **PHASE-03 attempt 2 — two design sentences flagged, not fixed.** §6's missing tie-break and
  §3's stat-failed candidate are design text; the phase records its ruling and its reasoning in
  this log and adds them to § Obligations owed at reconcile (items 6 and 7) rather than editing
  `design.md` from an implementation charter. Flagged.
- **PHASE-03 attempt 2 — the phase is left `in_progress` on purpose**, per the standing review
  cadence: green, committed, delta recorded, and the seat concludes after the independent review.
  Decided (operator cadence, not this charter's choice).

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
