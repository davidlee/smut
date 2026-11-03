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

8. **`design.md` §5's third field has no disjointness rule (RV-033 F-7).** `listing` interpolates
   `set.summary` verbatim, so a file declaring `summary: shadowed` (or `-`, or `did not load: …`)
   renders a row shaped exactly like the demoted or failed state, and a summary containing two
   spaces yields a five-field row — which breaks the suite's own `split("  ")` reading. Not an
   EX-2 violation: the criterion says the field *is* the summary and no escaping is specified. The
   "no two states can claim one row" guarantee holds for the renderer's choice, not the bytes.
   Owed: declare the third field's vocabulary disjoint from author-supplied summaries, or give
   the listing a delimiter its helpers can trust. Raised by the PHASE-04 review and deliberately
   not patched in the phase.

**Audit's additions and corrections (RV-035, 2026-11-03).** The eight above stand; four
things changed when the audit checked each against the artefact it names, and the review's
`## Reconciliation Brief` is now the operative list for `/reconcile`
(`doctrine review show 35 --full`):

- **Obligation 6's quotation is wrong** (RV-035 F-3). `design.md` carries no `None` at all;
  its only sorting text is `:246`'s "sorts each layer's entries by name". The quoted
  "a `name: None` entry sorts last … sorting is by the decoded name only" is
  PHASE-03/EX-4's paraphrase. The gap is real — §6's one clause is under-determined for a
  nameless entry, which §1 permits — so the brief writes the sentence against `:246`.
- **The `.config/smut` literals cannot go to reconcile** (RV-035 F-2). PHASE-01/EX-3's "the
  one place" and PHASE-04/EX-5's "nowhere else" are false (the segment is built in
  `Roots::under` *and* in `user_root`'s home fallback), but `plan.toml` is a completed plan's
  approved text, not a post-plan write surface. The brief routes the truth to `design.md` §2
  instead; the code is not "fixed" to match the false sentence.
- **Two upheld plan items were missing from this list** — plan.md item 3 (`parse` derives the
  name from `origin.path`; §6 should say so) and item 4 (an unreadable layer *directory* has
  no `Origin`, so §6 should name `StoreError::FileSystem` with a layer-worded `step`). Both are
  per-slice design edits and both are in the brief. Item 9 (`§5's "absolute for a file
  layer"` is falsified by a relative `$XDG_CONFIG_HOME`) was ruled "no design change" and the
  audit recommends the one clause anyway.
- **A ninth obligation the audit found** (RV-035 F-1): design §2 names `$HOME` as the user
  root's source while the binary asks `std::env::home_dir()`, which resolves the passwd entry
  for an unset or empty `$HOME` — so §2's "a container with no home … has no user root" is
  false. Per-slice edit to §2; no code change.

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

**fresh-as-of:** `2026-11-03` · `SL-008` lifecycle **`done`**, closed · last source change `c6cebe2`
(everything since is `.doctrine` markdown, a knowledge amendment or a memory record).
`PHASE-01`..`PHASE-05` landed green and reviewed (`c3385c2`, `8503b93`, `1ced7c8`, `e692735`,
`c6cebe2`); PHASE-02..04 reviewed (RV-030, RV-032, RV-033), PHASE-01 and PHASE-05 verified by
their seats; audited (RV-035, five findings, none blocking) and reconciled (ten `design.md`
edits, three decision amendments, no REV). `just check` exit 0 at closure with SL-010's work in
the same tree — 451 default and 464 `--features fault-injection` lib tests, zero warnings; all
16 VT rows PASS. **The slice is closed; `IMP-013` is resolved and `SL-006` is unblocked.**

### Produced

- **The slice closed.** `reconcile → done`; the `⚠` divergence marker is gone and the rollup
  reads 5/5. `IMP-013` resolved `fixed` (it is the implementation brief this slice delivered);
  `SL-006`'s blocker cleared. `just check` re-run green at closure with SL-010's committed work
  in the same tree (451/464 lib tests, zero warnings), so the guide edits SL-008 owns still
  hold under SL-010's later changes to `src/learn.rs`.
- **Reconciled — RV-035's brief landed on both surfaces.** Ten `design.md` edits (§2 ×2, §3 ×3,
  §4, §5 ×3, §6): the user-root source; the compound `.config/smut` segment built twice; the
  stat-failed candidate; duplicate keys and the exact fence; the asymmetric body trim; one
  clause per resolved *file* plus the corrected diagnostic quote; the binary row `shadowed`;
  the fourth field's absoluteness; the third-field disjointness ruling; and the §6 `load`
  paragraph carrying the sort tie-break, the unreadable-directory class and `parse`'s missing
  name parameter. Three knowledge amendments — **DEC-028** (`choice` narrowed, plus the
  duplicate-key and exact-fence rules), **DEC-026** (the cost paragraph's cure), **DEC-017**
  (`choice` + `consequences`: three-layer arrival). **No REV**: the target set refuses a DEC,
  so the brief's REV routing was corrected to `doctrine knowledge edit` (precedent REV-002).
- **The slice audited — RV-035 `done · await=none`**, a `reconciliation`-facet ledger on
  `SL-008` with five findings, every one terminal: three `design-wrong` routed to `design.md`
  §2/§6 for reconcile's direct edit (§2's user-root source vs `home_dir()`; the `.config/smut`
  segment built twice, where the two plan literals are lodged with a write surface that cannot
  hold them; obligation 6's quote of a §6 sentence that does not exist), two `follow-up`
  captured as **CHR-006** (`user_root`'s empty-`home` guard and its missing pin;
  `load`'s "three places" doc arithmetic). Conformance algebra: undeclared 55 all attributed
  to SL-010 or doctrine bookkeeping, undelivered 0, conformant 8 — no SL-008 commit touches a
  path outside its design-target set. Design §7's 26 rows each mapped to the test that
  reddens them, in RV-035's `## Synthesis`. The brief also carries the eight inherited
  obligations, the two upheld plan items the list omitted (3 and 4), item 9's §5 clause, and
  the three RV-030 design questions.
- **minted: CHR-006** — `load`'s filesystem-count doc sentence, and `user_root`'s empty-`home`
  guard plus the absent `fn an_empty_home_is_an_absent_layer` row; both residuals recorded with
  their replacement text and the reversal path.
- **`PHASE-05` implemented, verified and concluded at `c6cebe2`** — `seed` gains `source` (the
  literal `conventions/<name>.md`, identical at every layer), the version rule is pinned in all
  three shapes without re-deriving the crate's, and the declaration's key list moves from five
  names to six. One path-limited commit over the two owned files (390/14); gate 451/464 lib
  tests, exit 0; four new unit rows and three spawned rows; six mutation probes, the sixth
  isolating the two-layer comparison as the only row that catches a per-layer `source`; VA-2's
  hand transcripts (field order, one line, `previous`, `--compact`) in § Implementation run.
  No new design question — `source`'s literal is plan.md item 5's recorded reading.
- **The slice's implementation is complete** — five phases, one commit apiece at the end, all
  owned paths, another writer's backlog file and `_envrc` untouched throughout.
- **The implementation review ledger is resolved.** RV-030, RV-032 and RV-033 are `done` with
  `await=none` and no open findings on `SL-008`; the two nits that still awaited the raiser were
  verified: RV-032 F-6 (a skipped mode-000 row is invisible at the gate) is IMP-020, RV-033 F-7
  (the third field's vocabulary is not disjoint) is obligation 8 below. RV-029 remains a design
  review `done`, its `review_pass` stale against revision 36 — advisory, recorded in
  § Implementation run.
- **The slice's learned leg is in the memory corpus** — five new records
  (`mem.pattern.doctrine.mutation-probe-per-property`,
  `mem.pattern.doctrine.cross-slice-patch-review`,
  `mem.pattern.doctrine.doc-comment-hides-contradiction`,
  `mem.fact.doctrine.grep-va-row-is-a-classification`,
  `mem.fact.smut.store-follows-cwd`) and three amendments to existing ones
  (`vt-inert-keywords`, `verify-vt-needs-delta-row`, `vacuous-negative-assertions`), each linked
  to its neighbours. The `## Harvest` section now carries ids rather than the prose that seeded
  them; the review-era `in_progress` lines are pruned and git holds them.

- **`PHASE-04` closed on its criteria at `cd5c0dc`, with evidence for work it did not write.**
  The tail is one commit (two guide sentences, two whole-sentence tests, one store-format
  clause); the evidence is the half the concurrent run left out — VA-1's six real refusal runs
  with exit codes and verbatim messages, VA-2's §7 row-by-row walk (two rows named as PHASE-05's)
  and the `std::env::` grep, an EX-1..EX-8 check of the six inherited commits, and two negative
  controls on the third-field precedence, each reddening exactly its named test and restored
  byte-identically. All six PHASE-04 VT rows PASS; gate 705/766 tests, exit 0.
- **Three findings for the review, none repaired here** (F1 the order test that cannot
  discriminate its order, F2 the relative-root clause with no test, F3 EX-5's "nowhere else"
  wording): all in files this charter does not own, all recorded with the smallest correction.
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

- `mem.pattern.doctrine.mutation-probe-per-property` — one mutation per claimed property, reverted
  byte-identically; a row with no probe has no evidence, and a "same at every layer" rule needs
  layers that differ elsewhere.
- `mem.fact.smut.vacuous-negative-assertions` — amended: strength ordering hollows out the weaker
  assertions, so the strongest assertion goes last.
- `mem.fact.doctrine.vt-inert-keywords` — amended: a VT keyword is a presence check on a fact the
  phase owns, never a number a co-tenant slice moves; a criterion stricter than its truth is the
  same defect one level up.
- `mem.fact.doctrine.verify-vt-needs-delta-row` — amended: the `completed` transition recomputes
  and overwrites the delta, so correct the row after the flip, not before.
- `mem.pattern.doctrine.cross-slice-patch-review` — hand a reviewer `git show <oid> -- <owned
  paths>` per commit, and declare the other slice tree out of scope.
- `mem.pattern.doctrine.doc-comment-hides-contradiction` — when a criterion and the code disagree,
  the doc comment is where the disagreement hides; read it first.
- `mem.fact.doctrine.grep-va-row-is-a-classification` — a "X appears only in Y" row is false the
  moment a doc comment uses X; report the classification and the load-bearing absence.
- `mem.fact.smut.store-follows-cwd` — the store follows the cwd while the layers follow the
  environment, so a hand transcript must `cd`, not just export.
- `mem.fact.rust.home-dir-falls-back-to-passwd` — `std::env::home_dir()` answers for an unset
  *or empty* `$HOME` from the passwd entry, so a "no home" fixture must set `HOME` and a design
  that names `$HOME` is not the same rule as the binary's (RV-035 F-1's evidence).
- `mem.pattern.doctrine.doc-comment-hides-contradiction` — amended: a doc sentence *introduced
  to discharge a finding* is the highest-risk prose in a review, and a count in it ("in three
  places", "the only place") is a fact with a falsifier. Four false doc sentences in this
  slice, two of them written by the fixes for earlier ones.
- `mem.fact.doctrine.rev-target-set-refuses-a-dec` — a reconcile brief that files a decision
  under "Governance/spec (REV)" is unroutable: REV targets are SPEC/PRD/REQ/ADR/POL/STD, and
  DECs land through `doctrine knowledge edit` (REV-001 abandoned; REV-002's narrative).

### Open

- **Nothing in the implementation is open, and both reconcile surfaces are written.** The next
  action is `/close`, which resolves **IMP-013** — the implementation brief this slice delivers
  — and unblocks **SL-006**. **CHR-006** carries the two code residuals (RV-035 F-4, F-5);
  **IMP-020** the invisible mode-000 skip; **IMP-019** the missing `NoFreshBasename` row.
  The design run stays locked at revision 36 while `design.md` carries the reconciled prose:
  `design adopt` refuses at reconcile by design ("edit design.md directly and do not adopt").

- **Two design sentences are owed to reconcile and are *not* fixed here** — §6's missing
  tie-break among unreadable names and §3's silence on a candidate whose metadata cannot be
  read (items 6 and 7 of § Obligations owed at reconcile, with PHASE-03's rulings).
- **`PHASE-03`'s `Box::leak` on the exit-1 path is a deliberate, bounded leak** — one small
  allocation per failed layer read, in a process about to exit 1, because
  `StoreError::FileSystem.step` is `&'static str` and EX-2 authorises only a doc comment in
  `src/error.rs`. If a later phase widens that field (a `Cow`, or a `String` carrying the step),
  the leak should be retired with it. Recorded, not scheduled.

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
- **PHASE-01 residual — an empty `home` yields a relative layer root.** *Corrected by the
  audit (RV-035 F-4): the ruling below was never implemented and the pin it promises was never
  written.* `user_root(None, Some(Path::new("")))` still returns `Some(".config/smut")` — the
  non-emptiness test is applied to `config_home` only (`src/conventions.rs:110-115`) — and no
  row exercises an empty `home`: `an_empty_config_home_falls_through_to_home` passes the empty
  path as *`config_home`*. Unreachable from the binary (`home_dir()` is absolute or `None`),
  reachable from the crate's `pub` API, and the "relative root reads the wrong layer without a
  symptom" class the function's own doc argues against. The guard plus the absent row are
  **CHR-006** now, not a pin in a closed phase. The original ruling, kept as the record of
  what was decided: the non-emptiness EX-2 applies to `config_home` reads onto `home` too,
  since an empty `home` resolves no config home either.
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
- **PHASE-03 concluded `completed` at `3/5`** — green, committed, delta-recorded, then reviewed
  before it concluded, per the standing cadence. On this phase the review was the point, and it
  found an exit criterion the gate could not see.

### PHASE-03 review — RV-032, eight findings, one criterion failed

`9829de0..1ced7c8` plus the out-of-range `9281f64`. Eight findings (1 major, 3 minor, 4 nit),
none a blocker; seven `fix-now` discharged in `1ced7c8`, F-6 `follow-up` to IMP-020.

- **F-1 (major) — EX-1's third clause was not met.** `binary_layer()` took no `kind`, so the
  shipped conventions set was appended to *every* kind's load: `load(&roots, "shapes")`
  yielded a `Layer::Binary` entry named `hygiene`, and `set_in_force` could resolve a
  conventions set inside a shapes lookup. Latent only because EX-8 forbids a caller this phase.
  The genericity test was structurally blind to it — it computed `names` over all entries then
  filtered to `Layer::Project`, so it only ever asserted what the *file* layer contributed.
  Fixed by `SHIPPED_KIND` + `shipped(kind)` returning `&[]` for a kind nothing ships, with the
  added assertion reddening first to `[Some("one"), Some("hygiene")]` — the leak named in the
  failure message rather than argued about.
- **F-2 — the phase's own new rule was unfalsifiable.** The tie-break `PHASE-03` ruled and
  logged (raw filename bytes among `None`s) was unreachable: every fixture held exactly one
  unnameable file, so an `Ordering::Equal` mutant stayed green. Fixed with a two-undecodable
  fixture whose raw byte order and `read_dir` order disagree by construction. The charter child
  had applied exactly this standard to someone else's row and added a sixth control for it;
  the review applied it to the child's own rule.
- **F-3, F-5 — two docs that claimed what was not true** ("the only place in this module that
  reads a filesystem"; "in a process about to exit", false for the in-process tests this crate
  runs). **F-7** the module doc still described the single-set file. **F-8** one test named
  `fs::read`'s branch while its fixture died at `metadata`, leaving that branch unreached *and*
  an undetectable duplication of the sentence above it.
- **F-4 — `reading.flatten()` confirmed as the one unbought silence**, and correctly not
  turned into a row: no `DirEntry` means no path means no `Cause` to build. The fix is a
  comment making the skip chosen. The reviewer's reading beat the seat's here.
- **The reviewer's discipline worth keeping:** it listed five candidates that did *not* survive
  verification so the seat would not re-derive them, and it corrected the seat — `load`'s doc
  *had* been rewritten; the false claim was a different sentence.

**Delta, and why it is wider than PHASE-02's.** `9829de0..1ced7c8` necessarily contains three
foreign `doc(sl-010)` commits (`deef11e`, `75e4dd2`, `90eec66`) because they sit *between* the
phase's code tip and its corrective — unlike PHASE-02, where narrowing could exclude the
foreign commit, here no contiguous range covers the code and its review fix without them. The
engine's own answer applies: they ride along into `slice conformance`'s undeclared cell, where
they are visible. The narrower `9829de0..cfae50f` was rejected because it would have left
`1ced7c8` — the commit discharging this phase's review — attributed to nobody.

### PHASE-04 — attempt 1 died on provider balance; a concurrent run did the work

**Charter 4 never ran.** Launched at 01:31, it failed on its first model call with
`litellm.BadRequestError … Insufficient Balance` for the inherited `llm-gateway/deepseek-flash`.
Zero turns of work, zero cost. Per the skill this is a concrete blocker and **not** permission to
change model configuration, so it was reported rather than worked around.

**PHASE-04 was then implemented by a concurrent jailed agent**, commits `b6cf47c` → `d1f0700`
between 02:08 and 02:26: the unknown-set refusal's rendered clauses, the listing's four fields
and four states, `answer` + `roots()` + the environment read in one place, the spawned layered
rows, the failure classes and both roots, and the CLI help sentence. `PHASE-04` was flipped
`in_progress` by that run and left there.

**State inherited, verified at `8386aeb`:** gate green (444 default), `verify-vt` PHASE-04
VT-1/VT-2/VT-3 PASS. Remaining against the criteria:

- **VT-4 FAIL — EX-6's guide sentences are absent.** `src/learn.rs` has neither
  `fn the_guide_states_where_a_set_is_found` nor `fn the_guide_exit_table_names_an_unloadable_set`,
  so both the layer sentence and DEC-033's exit-table clause are still owed, as whole-sentence
  assertions against the reassembled paragraph.
- **VT-5 FAIL — `docs/store-format.md` still lacks the clause.** The keyword `may supply more` is
  genuinely absent, so the file still claims the binary is the only source.
- **VT-6 UNATTRIBUTABLE, not absent** — `src/cli.rs:686` carries
  `fn the_conventions_help_does_not_claim_the_set_is_embedded`. It reads `UNATTRIBUTABLE` only
  because PHASE-04 has no recorded delta; attribution is by registry, not by the file on disk.
  Same for VT-5's file once the clause lands.

So PHASE-04 is a small tail, not a phase: two guide sentences and their tests, one store-format
clause, a recorded delta, then the review.

**Closed at `cd5c0dc`.** The tail landed there, and the whole evidence packet for the six
inherited commits — VA-1's real runs, VA-2's §7 walk, and the EX-1..EX-8 criteria check — is in
§ PHASE-04 — the tail, and the evidence for six commits nobody accounted for, below.

### The SL-010 collision — and a brittleness that was ours

SL-010 landed ten commits in this window and touched `src/cli.rs`, `src/conventions.rs`,
`src/error.rs`, `src/learn.rs`, `src/main.rs`, `tests/conventions.rs` — every file PHASE-04 owns.
Its own code lives in `src/dsl.rs`, `src/lib.rs`, `tests/dsl.rs`, `tests/cli_contract.rs`,
`tests/fixture.rs`; the shared files carry both slices' work and the gate is green over both.

One real casualty: **PHASE-02's VT-3 went FAIL.** SL-010 added six refusal variants
(`MissingDirection`, `UnknownDirection`, `UnreadableToken`, `JsonOperand`, `ConflictingSelection`,
`UnreachableLimit`), moving the boundary table from `[(Error, u8); 27]` to `[…; 33]` — and VT-3
pinned the literal `27`.

That is our brittleness, not their error. A criterion that pins a count another slice owns will
break every time that slice adds a refusal, and the break says nothing about this slice's code.
**Fixed by dropping the literal from the keyword** — `[(Error, u8);` — so the criterion asserts
the boundary table's shape while the *test* keeps pinning whatever count it must. Re-pinning to
33 was rejected: it would buy one green run and rearm the same trap for the next slice.
PHASE-02 is back to five-for-five. Lesson worth carrying out of this slice: **a VT keyword is a
presence check on a fact, never on a number a neighbour owns.**

### PHASE-04 — the tail, and the evidence for six commits nobody accounted for

**Observed at `cd5c0dc`.** PHASE-04 is left `in_progress` on purpose (standing cadence), green,
committed, delta recorded. Two commits of the seat's: `cd5c0dc` (the tail: `src/learn.rs` +
`docs/store-format.md`) and the notes commit that follows it. Nothing else was edited except the
`src/conventions.rs` mutation probes below, each restored byte-identically (`git diff --stat --
src/conventions.rs` empty afterwards).

**The delta is `6007a0d..cd5c0dc`, 26 commits, registry provenance `manual`** —
`code_start_oid = 6007a0d95971d62ff6b820d169d6d1537a0f93f8`,
`code_end_oid = cd5c0dc7972a5c9608a1e4e95a4664c292909ec7`. Six of those are this phase's own
(`b6cf47c`, `ec7c933`, `e50a75d`, `35eab67`, `af9fbbd`, `d1f0700`), one is the tail, one is the
seat's notes commit `56488fa`, and eighteen belong to SL-010, the backlog and a `readme` commit —
no contiguous range covers this phase without them, because the phase's own work arrived in two
runs 100 minutes apart with another slice landing in between. Not narrowed with `--force`: the
phase genuinely spans seven commits, and the foreign ones land in `slice conformance`'s
undeclared cell where they are visible. The seat re-tightens after concluding.

**What the tail changed.** Not one line of the inherited code. `src/learn.rs` gained two guide
sentences — `A set is found in the project, then the user config home, then the binary.` in the
`Conventions` section, and `, or a conventions set could not be loaded` appended to the exit-1
row — plus the two tests EX-6 names. `docs/store-format.md`'s conventions clause now says one set
ships with the binary and "a project or a user may supply more", with the leaf path, the three
roots and the nothing-inside-`.smut/` promise byte-identical.

The red was observed before the prose existed: `cargo test --lib learn::` →
`test result: FAILED. 30 passed; 2 failed; 0 ignored` with both new tests reporting the missing
sentence (not a missing helper): the exit-1 row reassembled to `0 success, … 2 invalid input or
usage`, and the Conventions paragraph had no sentence about where a set is found. Green after the
prose: `test result: ok. 32 passed`.

The two pins, for the review:

| criterion | test | what it reads |
| --- | --- | --- |
| VT-4 half 1 | `fn the_guide_states_where_a_set_is_found` | the `Conventions` paragraph, reassembled by `flattened_section`, must contain the whole sentence `A set is found in the project, then the user config home, then the binary.` |
| VT-4 half 2 | `fn the_guide_exit_table_names_an_unloadable_set` | the `Exit codes` block, reassembled, must contain the whole row `1 no store, or the store could not be read or written, or a conventions set could not be loaded` |

Both read the reassembled text, never `GUIDE.contains(…)`, so a hand wrap cannot pass for a
missing sentence; the exit-table slice ends at `JSON goes to stdout` so the row cannot be found
in the paragraph below the table instead. VT-6 needed no code: `src/cli.rs`'s
`fn the_conventions_help_does_not_claim_the_set_is_embedded` was already on disk and read
`UNATTRIBUTABLE` only for want of a registry row.

**`doctrine slice verify-vt 008` at `cd5c0dc`** — the whole PHASE-04 column:

```text
  PHASE-04:
    ✓ PASS        VT-1
    ✓ PASS        VT-2
    ✓ PASS        VT-3
    ✓ PASS        VT-4
    ✓ PASS        VT-5
    ✓ PASS        VT-6
```

#### VA-1 — every new refusal path, run rather than asserted

Four scratch trees under `/tmp` (`sl008-va1`, `sl008-va1b`, `sl008-rel`), a real `.git`
directory, a fake `HOME` and `XDG_CONFIG_HOME` set explicitly, and the built `target/debug/smut`.
No test harness stands between the run and the message.

| path | command | exit | message (verbatim) |
| --- | --- | --- | --- |
| unknown name, every resolved file listed with its layer and verdict | `smut conventions nonsense` (cwd `repo/sub/dir`, three layers, two broken files) | **2** | see below |
| broken winner | `smut conventions hygiene` (project file malformed, user file good) | **1** | `smut: /tmp/sl008-va1b/repo/.config/smut/conventions/hygiene.md did not load: frontmatter line 3 is not a flat \`key: value\` line: the key is one word with no space in it, and a \`:\` follows it` |
| `--format json` with no name | `smut conventions --format json` | **2** | `smut: conventions --format json needs a set name, because the listing is a list and not a declaration; use \`smut conventions <name> --format json\`` |
| unknown format, before the operand | `smut conventions nonsense --format yaml` | **2** | `smut: unknown --format yaml for conventions, which writes text or json` |
| existing-but-unreadable layer (mode 000) | `smut conventions` with `<config-home>/smut/conventions` at mode 000 | **1** | `smut: reading the conventions layer failed for /tmp/sl008-va1b/config-home/smut/conventions: Permission denied (os error 13)` |
| the same class, root that is a regular file | `smut conventions` with `config-home/smut` a file | **1** | `smut: reading the conventions layer failed for /tmp/sl008-va1b/config-home/smut/conventions: Not a directory (os error 20)` |

The unknown-name message, whole — the one output that carries the listing's information in the
listing's order, one clause per resolved file, `; `-joined and nothing else:

```text
smut: unknown conventions set nonsense; the sets are: broken (project, did not load: frontmatter line 3 is not a flat `key: value` line: the key is one word with no space in it, and a `:` follows it); hygiene (project); bad (user, did not load: frontmatter line 3 is not a flat `key: value` line: the key is one word with no space in it, and a `:` follows it); hygiene (user); mine (user); hygiene (binary)
```

The same tree's listing at the same moment, exit **0** while reporting two broken files — EX-2's
"the listing exits 0 even when it reports a broken file" observed, not inferred:

```text
broken  project  did not load: frontmatter line 3 is not a flat `key: value` line: the key is one word with no space in it, and a `:` follows it  /tmp/sl008-va1/repo/.config/smut/conventions/broken.md
hygiene  project  project structure advice  /tmp/sl008-va1/repo/.config/smut/conventions/hygiene.md
bad  user  did not load: …  /tmp/sl008-va1/config-home/smut/conventions/bad.md
hygiene  user  shadowed  /tmp/sl008-va1/config-home/smut/conventions/hygiene.md
mine  user  -  /tmp/sl008-va1/config-home/smut/conventions/mine.md
hygiene  binary  shadowed  conventions/hygiene.md
exit=0
```

Four states in one tree, in the fixed precedence: `did not load:` on a broken *project* file and
on a broken *user* file, `shadowed` on a loaded user file behind a working project file and on
the **binary** row behind a project file, `-` on a bare markdown set, and a summary on the one
row in force. Two rules that the design text argues about are visible in it: the binary row is
shadowed (EX-2's rule, not §5's worked example — reconcile obligation 4) and the broken winner
is not an unknown name (exit 1 above, not 2).

One more real run, for EX-2's last clause: with `XDG_CONFIG_HOME=config-home` **relative**, the
user row is `mine  user  -  config-home/smut/conventions/mine.md` — a relative row, nothing
rewritten to look absolute.

#### VA-2 — design §7 walked row by row

Every row mapped to the test that would go red, read from the artefact rather than inferred from
a green suite. "spawned" = `tests/conventions.rs`, "unit" = `src/conventions.rs`'s in-module
tests.

| §7 property | test that reddens | level |
| --- | --- | --- |
| project beats user beats binary | `the_project_set_beats_the_user_set_beats_the_binary`; `the_project_layer_beats_the_user_layer` | spawned + unit |
| the user layer is really read | `a_set_that_exists_only_in_the_config_home_resolves`; `a_set_that_only_the_user_layer_holds` | spawned + unit |
| the walk-up happens | `the_walk_up_finds_the_repositorys_set` | spawned |
| a worktree counts as a repository | `a_worktree_is_a_repository`; `a_git_file_marks_a_repository` | spawned + unit |
| the walk stops at the repository | `the_walk_stops_at_the_repository`; `a_home_above_a_repository_is_not_the_project_root` | spawned + unit |
| no repository means cwd, no walk-up | `no_repository_means_the_cwd_and_no_walk_up`; `no_repository_above_leaves_the_start_directory_as_the_root` | spawned + unit |
| a broken project set shadows a good user set | `a_broken_project_set_shadows_a_good_user_set`; `a_broken_set_still_wins_its_name` | spawned + unit |
| a failed file behind a working one still reports its reason | `a_failed_row_behind_a_working_one_keeps_its_reason`; `a_failed_row_beats_shadowed` | spawned + unit |
| the listing reports and succeeds | `the_listing_reports_a_broken_file_and_succeeds` | spawned |
| an absent layer is not an error | `an_absent_layer_is_not_an_error` (both files) | spawned + unit |
| an unreadable layer is exit 1 | `an_unreadable_layer_exits_one`; `an_unreadable_layer_is_an_error`; `a_layer_that_cannot_be_opened_is_an_error` | spawned + unit |
| a set with no summary lists with `-` | `a_bare_markdown_file_lists_with_a_dash`; `an_in_force_set_without_a_summary_lists_with_a_dash`; `a_set_that_declares_no_summary_lists_with_a_dash` | spawned + unit |
| frontmatter is not prose | `a_named_set_prints_the_body_verbatim`; `frontmatter_is_not_part_of_the_body` | spawned + unit |
| **the body and the seed agree** | — | **NOT COVERED: PHASE-05** (`the_prose_and_the_seed_agree_on_the_body`) |
| all four derived keys are refused by name | `the_four_derived_keys_are_refused_by_name`; `the_four_derived_keys_exit_one` | unit (PHASE-02's; no spawned row) |
| unknown keys are ignored | `an_unknown_frontmatter_key_is_ignored` | unit (PHASE-02's) |
| an empty body is malformed | `an_empty_body_is_malformed`; the `SILENT` fixture through `a_broken_project_set_shadows_a_good_user_set` | unit + spawned fixture |
| a non-UTF-8 file and a non-UTF-8 name | `a_file_that_is_not_utf8_text_is_a_failed_row`; `a_non_utf8_filename_is_a_failed_row`; `an_undecodable_name_sorts_last` | unit |
| **the seed is fixed and origin-bearing** | `the_seed_is_one_declaration_with_a_fixed_id_and_the_crates_version`; `the_seed_title_is_the_heading_verbatim` cover id, title, version and body — **`source` is absent** | **NOT COVERED: PHASE-05** (`the_seed_names_its_source_and_the_authors_version`) |
| the shipped set is not special | `the_shipped_set_is_the_same_reader_as_a_file` | unit |
| a second install still upserts | `the_seed_declares_once_and_a_second_install_updates_the_same_entity`; `a_re_install_replaces_an_amended_body_keeps_a_user_field_and_reports_the_previous`; `compact_omits_the_previous_body_the_re_install_replaced` | spawned |
| no store is created | `the_command_still_creates_no_store` — `!store.exists()` **and** the paired `get` failing with its absent-store message, plus a decoy file at `.smut` | spawned |
| the two roots disagree on purpose | `the_two_roots_disagree_on_purpose` | spawned |
| unknown name at exit 2 (preserved) | `an_unknown_name_still_exits_two`; `an_unknown_name_still_exits_two`'s unit sibling `the_unknown_conventions_refusal_joins_rendered_clauses` (in `src/error.rs`) | spawned + unit |
| `--format json` with no name (preserved) | `json_without_a_name_is_still_a_usage_error`; `json_without_a_name_is_a_usage_error_naming_the_flag` | spawned + unit |
| unknown format before the operand (preserved) | `an_unknown_format_is_still_refused_before_the_operand`; `any_other_format_is_refused_before_the_name_is_read` | spawned + unit — **the spawned row does not discriminate the order; see finding F1** |
| the seed's single-line output (preserved) | `the_seed_is_one_compact_line`; `the_seed_is_one_compact_line_ending_in_one_newline` | spawned + unit |

**Rows this phase does not cover, named:** *the body and the seed agree* and the `source` half of
*the seed is fixed and origin-bearing*. Both are PHASE-05's, whose VT-1/VT-2 already fail for
exactly those names. PHASE-05 also extends `the_seed_is_one_compact_line` from one line to a
six-name key set.

**The `std::env::` grep over `src/main.rs`**, because it is what proves the environment read did
not land in `dispatch`:

```text
$ grep -n "env::" src/main.rs
17:    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
114:/// `std::env::home_dir()` reports. Everything below takes the result as a
131:            std::env::var_os("XDG_CONFIG_HOME")
134:            std::env::home_dir().as_deref(),
142:    std::env::current_dir().map_err(|source| StoreError::FileSystem {
179:        Roots::under(std::env::temp_dir().join("smut-in-process-no-layer"), None)
```

Classified, because the criterion's wording ("only inside `roots()`") is not literally true and
the honest reading is stronger: **the two layer reads** are 131 and 134, both inside `roots()`
(125-137); 17 is `args_os()` in `main()`, the command line; 142 is `current_dir()` in
`current_directory()`, which `roots()` and `execute`'s store root both use (DEC-004's cwd); 179 is
`temp_dir()` in a test helper; 114 is a doc comment. `dispatch` (70-93) and `execute` (95-107)
contain **no** `env::` call at all. The property F-3 protects is that no in-process test and no
code below `main` reads `$HOME` or `$XDG_CONFIG_HOME` — and none does.

#### The inherited commits checked against EX-1..EX-8

Verified against the code and the tests at `cd5c0dc`, not against the committing run's account,
which does not exist.

| criterion | verdict | evidence |
| --- | --- | --- |
| EX-1 — `answer(roots, kind, ask)` the only entry `main` calls; `Ask` is the question; `resolve` pure; the two CLI doc comments fixed; the help test; no cli behaviour change | **met** | `execute` → `conventions::answer(roots, "conventions", ask)` is the only conventions call; `Ask::{List, Prose{name}, Seed{name}}`; `resolve` matches strings only (no `fs::`, no `env::`) and returns `Result<Ask, Reject>`; `git diff 6007a0d d1f0700 -- src/cli.rs` is two doc comments and one test, nothing executable; `the_conventions_help_does_not_claim_the_set_is_embedded` asserts the rendered help, and `smut --help` really prints `A set comes from the project, then the user config home, then the binary` |
| EX-2 — one row per file, four fields, four states with the precedence failure > shadow > summary > dash, `<not UTF-8>`, exit 0 with a broken row, the binary row shadowable, `Path::display`, absolute exactly when its root is | **met** | `listing`'s doc states the four states in order and the code matches (failure matched first, then the positional shadow, then summary, then `-`); VA-1's real listing shows all four plus a shadowed binary row; the second mutation below reddens exactly `the_binary_row_is_shadowed_by_a_nearer_file`; `the_path_column_is_absolute_for_a_file_and_a_leaf_for_the_binary` plus the relative-`XDG_CONFIG_HOME` run above for the last clause. **The relative-root clause has no test of its own — finding F2** |
| EX-3 — exit 2, one clause per resolved file in listing order joined by `; `, the field holds rendered clauses and says so, `None` contributes no clause | **met** | `clauses()` builds `name (layer)` / `name (layer, did not load: <reason>)`; `Reject::UnknownConvention { sets: Vec<String> }` doc says "each already rendered as …"; `Display` joins `; `; VA-1's message has six clauses in listing order and no `(not UTF-8)`; `the_unknown_conventions_refusal_joins_rendered_clauses` asserts the whole string |
| EX-4 — a failed winner refuses at 1 with path and reason and no store vocabulary; unknown name at 2; `json` with no name a usage error; a shadowed set unaddressable | **met** | VA-1's broken-winner run: exit 1, path, reason, and the word `store` appears nowhere; `set_named` returns `StoreError::ArtefactNotLoaded` on `Outcome::Failed` and `Reject::UnknownConvention` only when `set_in_force` finds nothing; `set_in_force` takes the *first* match, so a shadowed copy is unreachable (and `the_project_set_beats_the_user_set_beats_the_binary` prints the project body for a name all three layers hold) |
| EX-5 — roots computed once in `roots()`, `home_dir()` on the pinned toolchain, `.config/smut` joined only there, `load` joins only `<kind>`, threaded through `dispatch`/`execute`, the in-process shipped-set test moved out | **met, with a wording nit (F3)** | `roots()` is the only function reading `XDG_CONFIG_HOME`/`home_dir`; grep shows `.config/smut` as one literal in `Roots::under` and the home fallback in `user_root` (`home.join(".config").join("smut")`) — which PHASE-01/EX-2's own text requires, so EX-5's "nowhere else" is loose, not violated; `load` joins `<kind>` exactly twice (project, user); `dispatch`/`execute` take `&roots` and the per-test `no_layer_roots()` passes its own; `main.rs`'s shipped-set assertions are gone and the spawned suite owns them |
| EX-6 — the layer sentence and DEC-033's exit clause, each pinned by a whole-sentence test; the store-format clause | **met in this tail** | the two tests above, red before the prose and green after; `grep -n "may supply more" docs/store-format.md` → line 55, one hit, one line |
| EX-7 — the layered surface proved through the spawned binary, every name the plan pins, the four preserved behaviours | **met** | VT-2 passes with all 18 keywords; `tests/conventions.rs` is 1522 lines with 29 spawned rows; the rows map one-to-one onto §7's table above. One row is weaker than its name — finding F1 |
| EX-8 — every assertion names the change that reddens it; nothing carried over that no input could falsify; the spawn helper sets the environment it depends on; absent and unreadable are separate rows | **met** | `Scratch::run_in` sets `HOME` and `XDG_CONFIG_HOME` on every spawn and its doc says why neither is unset (`home_dir()` falls back to passwd); `an_absent_layer_is_not_an_error` and `an_unreadable_layer_exits_one` are two rows over the same shape (RV-029 F-12); spot-checked for vacuous pins and found none; the two negative controls below are what turns that from a reading into evidence |

#### Two negative controls on the inherited invariant

PHASE-04's one frontier-shaped rule is the third field's precedence. Both probes were made in
`src/conventions.rs`, run, and reverted from a byte-identical copy — no `checkout`, `reset` or
`stash`.

| mutation | reddened | restored |
| --- | --- | --- |
| the binary row exempted from the shadow (`if shadowed` → `if shadowed && layer != Layer::Binary`) | `the_binary_row_is_shadowed_by_a_nearer_file` — **exactly one** test, 56 passed 1 failed | `git diff --stat -- src/conventions.rs` empty |
| a failure loses to the shadow (the `Failed` arm asks the shadow first) | `a_failed_row_beats_shadowed` (unit, exactly one) **and** `a_failed_row_behind_a_working_one_keeps_its_reason` (spawned, 0 passed 1 failed) | `git diff --stat -- src/conventions.rs` empty; `cargo test --lib conventions::` → 57 passed |

The second is the phase's whole point in one probe: the same mutation breaks a unit row and a
process row, in both directions of the rule.

#### Findings — reported, not repaired

- **F1 (minor): `an_unknown_format_is_still_refused_before_the_operand` does not discriminate the
  order it is named for.** It runs `conventions hygiene --format yaml`, and `hygiene` is a name
  the binary layer holds — so if the implementation resolved the operand *first*, the name would
  succeed and the format would still be refused with the same message. Nothing goes red. The
  property itself holds (VA-1: `conventions nonsense --format yaml` → the format is refused, the
  unknown name is never mentioned), so this is a pin weaker than its name, not a defect in the
  code. **Smallest correction:** add one case with an unknown operand and assert the format is
  named and `unknown conventions set` is absent — three lines in a file this charter does not
  own. Left for the review.
- **F2 (nit): EX-2's relative-root clause has no test.** `the_path_column_is_absolute_for_a_file_and_a_leaf_for_the_binary` uses absolute roots only. The behaviour is correct and was
  demonstrated by a real run (VA-1), but nothing would go red if `listing` normalised a relative
  path to an absolute one. **Smallest correction:** one spawned row whose `XDG_CONFIG_HOME` is
  relative, asserting the row's fourth field is the relative join.
- **F3 (nit): EX-5's "the `.config/smut` segment is joined by PHASE-01/EX-3's function and
  nowhere else"** is loose. `user_root`'s home fallback joins `.config` then `smut` — which
  PHASE-01/EX-2's own text mandates, since a home is not a config home. No code change; the
  criterion sentence is the imprecise part, and audit should read it as "no caller repeats the
  suffix and `load` joins only `<kind>`".

None of the three is a contradiction of accepted authority, a weakened bound or a new reach; all
three are review material. Nothing in the inherited six commits was materially wrong.

#### Gate

`just check` at `cd5c0dc`, exit **0**: `cargo build` finished; both clippy configurations
(`--all-targets` and `--all-targets --all-features`) with `-D warnings` clean; `cargo fmt
--check` clean; the suite twice. Default features: 17 test binaries, **705 tests, 0 failed**
(lib `446` — the 444 baseline at `8386aeb` plus the two new guide tests — plus main `9`,
cli_contract `62`, compatibility `10`, concurrency `4`, conventions `29`, dsl `19`, durability
`0`, fixture `17`, predicates `13`, readme `9`, render `11`, roundtrip `4`, selection `8`,
store_corruption `27`, traversal `37`, doc `0`). `--features fault-injection`: **766 tests, 0
failed**. No warnings, no ignores, no skips.

#### For reconcile (already on the list, not touched)

All eight obligations stand as the earlier sections write them: DEC-028's malformed-file clause,
DEC-026's cost sentence, DEC-017's "a file plus a match arm", §5's worked listing (its binary row
must read `shadowed` — this phase's rule and this phase's real run both say so), §4's "one clause
per resolved name", `design.md:156`'s stale quoted diagnostic, and items 6-8 (§6's tie-break, §3's
unsettled candidate, §5's third-field vocabulary). None was edited here.

### PHASE-04 review — RV-033, and how the interleave was contained

Eight findings (4 minor, 4 nit), no blocker, no major; seven discharged in `e692735`, F-7 to
reconcile. All verified by diff inspection, not by the handback.

- **The review technique is the durable lesson.** PHASE-04's delta spans 26 commits, 18 of them
  SL-010's, so a range review would have produced findings against another slice's code — the
  most expensive kind of false finding, because they read as actionable. Instead the reviewer
  was handed `/tmp/sl008-phase04.patch`: the seven PHASE-04 commits restricted to the seven
  owned files, verified to contain zero foreign files, with the working tree declared as
  carrying SL-010's code out of scope. A shell-less reviewer can only see post-images, so the
  patch is how the interleave stops contaminating judgement rather than just attribution.
  Rebuild it with `git show <oid> -- <owned paths>` per commit in `git log` order.
- **EX-8 was the criterion not literally met:** an `assert_ne!` sitting under an `assert_eq!` on
  the same expression, a refusal-order row whose fixture fed a name that resolves (so both
  orders printed identical stderr), a relative-path renderer with no falsifier, and a
  no-repository fixture skipping `assert_no_repository_above` while its five siblings call it.
- **Third phase running, a run declared its suite clean and the review found an assertion no
  input could falsify** — PHASE-02's three tautologies, PHASE-03's unreachable tie-break,
  PHASE-04's `assert_ne!`. Not three accidents: that is what a green gate does to a model's
  judgement about its own assertions. The countermeasure that has actually worked is mutation
  probes demanded per property, plus a reviewer reading for *falsifiability* rather than for
  correctness.
- **F-3's fix beat its own brief.** I asked for `ends_with` on the relative path; a cwd-prefixed
  path still *ends* in the relative one, so `ends_with` would not have falsified the failure
  EX-2 names. The worker compared the whole row instead. Worth recording as the standard.
- **Two criteria now have literals stricter than their truth** — PHASE-02's boundary count
  (owned by SL-010) and EX-5's "`.config/smut` is joined nowhere else" (`user_root`'s home
  fallback must join it). Both went to reconcile rather than being repaired in code.

### Seat preflight — observed at `1b409c9` (PHASE-05 charter, 2026-11-02)

- **Baseline gate is green.** `just check` exits 0 at `1b409c9` with zero warnings and 34
  green result lines: 447 default lib tests and 460 under `--features fault-injection`, every
  other target green. A red gate from here is a regression.
- **The index is ours alone.** `git status --porcelain` carries two paths that are not this
  slice's: `M .doctrine/backlog/improvement/021/backlog-021.md` and an untracked `_envrc`. The
  earlier writer's staged `README.md` and IDE-006 backlog files are no longer staged. Commits
  stay path-limited with a pathspec; no `add -A`, no amend, no reset.
- **`verify-vt` at handover:** PHASE-01..04 PASS; PHASE-05's VT-1 and VT-2 FAIL only because
  their pinned test names do not exist yet. Nothing unattributable.
- **Lifecycle and sheets:** `started`, 4/5. `.doctrine/state/slice/008/phases/phase-05.toml`
  reads `planned` and `phase-05.md` is the empty skeleton, so the charter's first act is
  `/phase-plan` over it. The seat does not pre-flip the phase — the sub-orchestrator owns the
  `in_progress` and `completed` transitions.
- **Review cadence:** PHASE-05 is the discretionary one and it is skipped, per § Review cadence
  and the handover: small, mechanical, its trap pinned by a named test, and the audit's
  pre-close review covers the accumulated delta anyway.
- **Model posture unchanged:** children inherit the seat's `llm-gateway/deepseek-flash`,
  thinking high. No operator pin was requested for this phase.

### PHASE-05 — observed at `c6cebe2` (concluded, 5/5)

- **One commit, path-limited with a two-path pathspec:** `c6cebe2` `feat(sl-008)` over
  `src/conventions.rs` and `tests/conventions.rs` — 390 insertions, 14 deletions. The other
  writer's `M .doctrine/backlog/improvement/021/backlog-021.md` and untracked `_envrc` were
  untouched; no `--amend`, `add -A`, reset, stash or checkout. HEAD did not move mid-phase.
  Sheet decision 1 asked for a `test` commit then a `feat` commit; the split was not taken —
  see § Autonomy log.
- **What landed.** `seed` gains one field: `source` = `format!("conventions/{}.md", set.name)`
  — the literal leaf, at every layer, never `origin.path` and never derived from a kind
  (`seed(set: &Set)` has no `kind`: plan.md design question 5, disposition **R**). `version`
  is unchanged (`set.version`, which the loader leaves as the author's for a file and the
  crate's for the binary layer) — no `env!` call was added anywhere in `seed`. `smut_id`,
  `type`, `title` and `body` are untouched. `seed`'s doc comment now names all six fields, the
  layer-blind literal and the printed order.
- **Red observed before green** (the worker's run, re-run by the seat):
  `cargo test --lib -- conventions::` → 59 passed, **3 failed** —
  `the_seed_is_the_body_under_a_fixed_id_and_the_crates_version` (`source` left `Null`),
  `the_source_is_the_conventions_leaf_path` (`Null` vs `conventions/Mine.md`),
  `the_seed_omits_a_version_a_file_does_not_declare` (map had four names, wanted five).
  `cargo test --test conventions` → 28 passed, **4 failed** — `the_seed_key_set_is_six_names`
  (five names vs six), `the_seed_is_one_compact_line` (line without `source`),
  `the_seed_names_its_source_and_the_authors_version` (absolute path where the leaf belongs),
  `the_source_is_the_same_at_every_layer` (`Null`). Nothing else went red; the library
  compiled throughout, so this is an assertion red and not a build red.
- **Green:** `cargo test --lib -- conventions::` 62 passed; `cargo test --test conventions`
  32 passed.
- **Gate — `just check`, exit 0,** 34 green `test result` lines, zero warnings in either
  clippy configuration, `fmt --check` clean. Default features: **451** lib tests (baseline
  447, +4 — the four new unit rows); `--features fault-injection`: **464** (baseline 460, +4).
  Integration `conventions` **32** tests (29 + 3 spawned rows). Every other target identical to
  the baseline. No pre-existing failure, no jail-only failure: the baseline was green and this
  is green.
- **Six mutation probes, each reverted byte-identically** (`sha256` of the file after the last
  equals the pre-probe hash):

  | # | mutation to `seed` | rows that reddened |
  | --- | --- | --- |
  | 1 | `source` ← `set.name` (leaf and `.md` dropped) | `the_source_is_the_conventions_leaf_path`, `the_seed_is_the_body_under_a_fixed_id…`, `the_seed_names_its_source_and_the_authors_version`, `the_source_is_the_same_at_every_layer`, `the_seed_is_one_compact_line` |
  | 2 | `source` ← `conventions/<version-or-crate>/<name>.md` (a layer-visible field leaks in) | the same five |
  | 3 | `version` ← `env!("CARGO_PKG_VERSION")` unconditionally | `a_file_sets_version_is_the_authors`, `the_seed_omits_a_version_a_file_does_not_declare`, `the_seed_names_its_source_and_the_authors_version`, `the_source_is_the_same_at_every_layer` |
  | 4 | `body` dropped from the fields | `the_seed_is_the_body_under_a_fixed_id…`, `the_seed_omits_a_version…`, `the_seed_key_set_is_six_names`, `the_seed_is_one_compact_line`, `the_prose_and_the_seed_agree_on_the_body`, `a_second_install_reports_the_body_it_replaced`, `compact_omits_the_previous_body_the_re_install_replaced`, `the_seed_names_its_source…`, `the_source_is_the_same_at_every_layer` |
  | 5 | `title` ← `format!("Conventions: {}", set.title)` | `a_file_with_no_heading_falls_back_to_its_filename`, `the_seed_is_the_body_under_a_fixed_id…`, `the_title_is_the_first_heading_verbatim`, `the_seed_title_is_the_heading_verbatim`, `the_seed_key_set_is_six_names`, `the_seed_is_one_compact_line`, `the_seed_names_its_source…` |
  | 6 | `source` correct for the project layer (version 2.5.0) and `…-user.md` for the user layer (3.1.0) | **only** `the_source_is_the_same_at_every_layer` |

  Probe 6 is the one that matters for the phase's hardest rule: a `source` that is right at
  one layer and wrong at another reddens exactly one row, and it is the row that compares two
  layers. Probes 1-5 each redden their intended rows plus honest collateral (probe 4's nine
  rows are all rows that carry the body). `a_file_sets_version_is_the_authors` and
  `the_prose_and_the_seed_agree_on_the_body` are green on arrival — preserved behaviour — and
  are made evidence by probes 3 and 4 only.
- **VA-2 — the hand transcripts, from a scratch cwd** (`/tmp/sl008-va2b.3LjRSK/work`, with
  `HOME`/`XDG_CONFIG_HOME` under it, so no layer and no store of the developer's is touched):

  ```
  $ smut conventions hygiene --format json > seed.json
  seed bytes=2907  newlines=1
  seed key order: "smut_id": "body": "source": "title": "type": "version":

  $ smut declare < seed.json            # first install
  {"changes":[{"smut_id":"conventions-hygiene","previous":null,"status":"created"}],"ids":["conventions-hygiene"],"missing":[]}

  $ smut declare < seed.json            # identical re-install: nothing replaced
  {"changes":[{"smut_id":"conventions-hygiene","status":"unchanged"}],"ids":["conventions-hygiene"],"missing":[]}

  $ echo '{"smut_id":"conventions-hygiene","body":"Our house rule: one label per relation."}' | smut declare
  {"changes":[{"smut_id":"conventions-hygiene","previous":{"smut_id":"conventions-hygiene","body":"<the shipped body>",…},"status":"updated"}],…}

  $ smut declare < seed.json            # the second install that replaces an amendment
  {"status":"updated","previous":{"smut_id":"conventions-hygiene","body":"Our house rule: one label per relation.","source":"conventions/hygiene.md","title":"Conventions: general hygiene","type":"convention","version":"0.2.2"}}

  $ echo '{"smut_id":"conventions-hygiene","body":"Draft, to be replaced."}' | smut declare
  $ smut declare --compact < seed.json   # no `previous` under --compact
  {"changes":[{"smut_id":"conventions-hygiene","status":"updated"}],"ids":["conventions-hygiene"],"missing":[]}
  ```

  Read off the run: the declaration is one compact line ending in exactly one newline; its
  field order is `smut_id` first and `body`, `source`, `title`, `type`, `version` lexically;
  `version` is the crate's `0.2.2` for the shipped set; `source` is `conventions/hygiene.md`;
  the re-install that replaces an amendment reports the replaced entity under `previous`,
  `source` and `version` included; `--compact` omits `previous` entirely. A separate check in
  the same scratch: `prose == seed["body"] + "\n"` exactly.
- **`doctrine slice verify-vt 008`:** all **16 rows PASS**, zero FAIL and zero UNATTRIBUTABLE,
  exit **0** — including PHASE-05 `VT-1` and `VT-2`.
- **Recorded delta:** `e54f7aa..c6cebe2` (`--start e54f7aa --end c6cebe2`, recorded *after* the
  flip). `git log e54f7aa..c6cebe2` is the one commit; `git diff --name-only` over it is
  exactly `src/conventions.rs` and `tests/conventions.rs`. Boundaries row provenance `solo`.
- **Verifier edits after the workers' green (two, both logged in § Autonomy log):** two
  `assert_ne!` rows in `the_source_is_the_same_at_every_layer` removed as already guaranteed by
  the two exact per-layer body/version assertions above them.
- **One integrity note.** The first VA-2 attempt ran the binary from the repository root and so
  created a `.smut` store there (gitignored; index holding only the one entity that run
  declared, every file timestamped at that run). It was removed, the tree re-checked against
  `git status --porcelain`, and the transcript above is from a re-run whose cwd is the scratch.
  No tracked path was affected.
- **Out of scope, untouched:** the eight reconcile obligations, `plan.md`'s design questions
  and the design text. This phase raised no new design question: `source`'s literal is
  plan.md item 5's recorded reading, and nothing in the diff needs a different one.

## Autonomy log

- **PHASE-05 — sheet decision 1 superseded: one commit, not a `test` then a `feat`.** The
  phase is one compile unit in one file: the new assertions name the six-name key set and the
  `source` field, so the test-only state is a knowingly red tree. Splitting it into two commits
  would need hunk staging on a shared index (the other writer's backlog file is dirty) or a red
  commit in history. Decided: one `feat(sl-008)` commit carrying implementation and tests, as
  PHASE-02 did for its own compile-forced unit, with the red recorded in § Implementation run
  instead of in the log. Reversible and in-scope; logged rather than rebriefed.
- **PHASE-05 — two `assert_ne!` rows removed as verifier edits after the workers' green.**
  `the_source_is_the_same_at_every_layer` asserted that the two replies' bodies differ and that
  their versions differ, *after* two exact assertions pinning each body and each version to a
  different const — so neither `assert_ne!` could fail once the exact rows passed, which is the
  "negative assertion occupying an evidence slot" class this slice's own notes name. Removed,
  with a comment saying the exact rows are what make the shared `source` a rule rather than a
  coincidence. Decided (mechanical, inside the owned file, no property lost).
- **PHASE-05 — probe 6 added beyond the sheet's five.** The sheet's probe 2 (a version-keyed
  leaf) reddened five rows, so it proved the literal is load-bearing but not that the
  two-layer comparison has force of its own. Probe 6 — a `source` that is *correct* for the
  project layer and wrong for the user layer — reddens exactly one row,
  `the_source_is_the_same_at_every_layer`, which is the honest demonstration that this row is
  the only place a per-layer `source` can be caught when each single-layer value looks right.
  Decided: extra negative control, no criterion change.
- **PHASE-05 — the first VA-2 run wrote a store into the repository root.** The transcript was
  taken with the binary's cwd still at `/workspace/smut`, so `declare` created a gitignored
  `.smut/` there. It held only the one entity that run declared and every file was timestamped
  by that run, so it was this run's artefact: removed, `git status --porcelain` re-checked, and
  the recorded transcript re-taken from a scratch cwd. Decided (clean up my own side effect,
  disclose it in the phase record); no tracked path was touched and nothing of the developer's
  was read or removed.

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

- **PHASE-04 — the tail was delegated as two serial workers, tests first.** Charter 4 declared
  that it would not re-do the inherited work, and the two remaining criteria are prose whose
  contract is a test, so the split was: a test worker writes the two pins (GUIDE untouched),
  observe red for the right reason, then an implementation worker adds the sentences and the
  store-format clause, observe green. Reasons: the red is the only evidence that the pins pin
  anything, and a single worker would have had no reason to stop between them. Both workers ran
  foreground, model inherited (`llm-gateway/deepseek-flash`), no other provider touched. Decided.

- **PHASE-04 — two verifier edits after the workers' green, logged rather than rebriefed.**
  (i) The exit-table test's slice originally ended at the next blank line, which in this guide is
  a paragraph *below* the table; tightened to `"\nJSON goes to stdout"` so the pinned row cannot
  be found outside the table it belongs to. (ii) The store-format paragraph was left half-reflowed
  by the implementation worker's edit; re-wrapped the one sentence onto the paragraph's own
  margin, no word changed. Both are inside the two files this charter owns. Decided.

- **PHASE-04 — the recorded delta spans 26 commits on purpose, and is not narrowed.** The
  phase's own work arrived in two runs 100 minutes apart with SL-010's ten commits, a backlog
  commit, a `readme` commit and this seat's notes commit in between, so no contiguous range
  covers it and `--commit` mode cannot express a seven-commit phase. PHASE-03 recorded
  `--start/--end` for the same reason; unlike PHASE-02 nothing was narrowed with `--force`,
  because `--force` narrows and the need here is the opposite. The foreign commits land in
  `slice conformance`'s undeclared cell, where they are visible, and the seat re-tightens after
  concluding. Decided.

- **PHASE-04 — the three findings are reported, not repaired.** F1 and F2 live in
  `tests/conventions.rs`, which SL-010's wrap-up and PHASE-05 both still touch; F3 is a criterion
  sentence, not code. The charter's rule is to report before editing a shared file, and none of
  the three is a materially wrong behaviour — the code satisfies every property, the pins are
  weaker than their names. Deferred to the review with the smallest correction written down.

- **PHASE-04 — the negative controls ran against inherited code, with a file copy, not git.**
  Two mutations in `src/conventions.rs` (`if shadowed` → exempt the binary row; the `Failed` arm
  asking the shadow first), each run, each restored from a `/tmp` copy and confirmed by an empty
  `git diff --stat -- src/conventions.rs`. No `checkout`, `reset` or `stash` was run, and the
  restored file's own suite was re-run green (57 unit rows). Decided: PHASE-03's discipline,
  because a pin nobody can redden is not evidence and the inheriting run supplied none.

- **PHASE-04 — the phase is left `in_progress`.** Green, committed (`cd5c0dc` + the notes
  commit), delta recorded, all six VT rows PASS — and the seat concludes after the independent
  review, which this phase needs more than most because six of its seven commits were never
  reviewed by anyone. Decided (operator cadence, not this charter's choice).

- **PHASE-04 — what was deliberately not touched.** Audit (not open), PHASE-05 (not this
  charter), the seven reconcile obligations (flag only), `README.md` and
  `.doctrine/backlog/improvement/021/backlog-021.md` (another writer's dirty work, left exactly as
  found), `Cargo.toml`, and the five phase files the tail did not need. The `.config/smut` join,
  the binaries' kind-awareness and the four settled readings were taken as settled, not re-opened.
