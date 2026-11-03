# Review RV-035 — reconciliation of SL-008

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode.** Conformance, self-audit. I drove both roles with `--as`; the raiser
raised, verified and concluded, the responder disposed. Subject is the slice
itself (target-ladder rung 1); the aspect is `reconciliation`.

**Surface audited: the primary worktree at `9e5083e`.** No dispatch candidate
exists for this slice (`doctrine dispatch candidate status` holds nothing), so
there is no candidate interaction branch to review — the main tree *is* the
surface. Source head is `c6cebe2`; every commit since is `.doctrine` markdown or
a memory record, confirmed by `git diff --name-only c6cebe2..HEAD`.

**What I read first.** `.doctrine/slice/008/handover.md` (before this brief, as
instructed), then `design.md` §1–§7 (locked, revision 36), `slice-008.md`,
`plan.toml`, `plan.md` § Design questions adjudicated and its dispositions,
`notes.md` § Obligations owed at reconcile + § Autonomy log + § Harvest, and the
four resolved code-review ledgers (RV-030, RV-032, RV-033, plus RV-029's design
pass) at `--full` where a finding's discharge mattered.

**Lines of attack.**

1. **The gate, measured rather than inherited.** `just check` at the audited head:
   451 default-feature lib tests, 464 under `--features fault-injection`, 34 green
   result lines, zero warnings, `fmt --check` clean, exit 0. `doctrine check gate`
   green. `doctrine slice verify-vt 008` — 16 PASS, 0 FAIL, 0 WAIVED, 0
   UNCHECKABLE, exit 0. Restated here because the handover's numbers are evidence,
   not authority.
2. **Conformance algebra.** `doctrine slice conformance 008`: undeclared 55,
   undelivered 0, conformant 8. Every one of the 55 undeclared paths was attributed
   by `git log --format=… -- <path>`: `src/dsl.rs`, `src/lib.rs`, `tests/dsl.rs`,
   `tests/cli_contract.rs`, `tests/fixture.rs`, `README.md` are SL-010's or a
   release/readme commit's; `.doctrine/knowledge/decision/034–041`,
   `.doctrine/backlog/*`, `.doctrine/slice/010/*` are SL-010's; `.quotient/*` is a
   foreign tool's; `.doctrine/review/030` and `.doctrine/slice/008/{notes.md,
   plan.toml}` are this slice's own review ledger and slice documents. **No SL-008
   commit touches a path outside its design-target set**, and undelivered is 0, so
   the registries agree with the code. The undeclared cell is a co-tenant's work
   plus doctrine bookkeeping, exactly as the handover predicted.
3. **Design §7 row by row.** All 26 properties were mapped to the test that reddens
   them (named in the Synthesis). The three rows which had no test of their own at
   review time — the mode-000 fixture, `parse`'s derived name, §5's third-field
   disjointness — are the recorded design questions, not silent gaps.
4. **The eight reconcile obligations plus the three RV-030 design questions**, each
   checked against the artefact it names rather than against its restatement: DEC-028
   (the malformed-file sentence really does overreach — a broken project set does
   break the user's copy of that *name*), DEC-026 ("origin printing plus a set header
   naming its own file" is at `record-026.md:44`), DEC-017 ("a file plus a match arm",
   twice), design §5's worked binary row and §4's "one clause per resolved name",
   `design.md:157`'s quoted diagnostic against `not_key_value`'s sentence, §6's
   missing tie-break, §3's stat-failed candidate, §5's third field.
5. **Behaviour, observed rather than asserted.** A scratch repository, a scratch
   config home and a scratch `HOME`, driving the built binary: listing with three
   layers and a broken file (exit 0, `did not load:` row, binary row `shadowed`);
   unknown name (exit 2, every file named with its layer); broken winner (exit 1,
   path and reason, no store vocabulary); a seed piped straight into `declare`
   (accepted, six keys or five when no version is declared); `summary: shadowed`
   rendering a shadow-shaped row (obligation 8, reproduced); no `.smut/` created.
6. **Where the bodies were likely buried.** PHASE-01 and PHASE-05 were never
   independently reviewed (their seats verified them), so `project_root` /
   `user_root` / `Roots::under` and `seed` got the closest reading. Also probed: every
   doc sentence that resolves a contradiction (the class this slice produced three
   times), the environment's reach into `roots()`, and the facts the handover
   asserts as pinned rather than as reasoned.

**Non-Goals held.** No `.smut/` file appears (conformant paths only); `README.md` is
untouched by any SL-008 commit (CHR-001's); no section addressing, no teeth, seed
identity unchanged. The set's *content* is SL-006's, not this audit's.

## Synthesis

**The closure story.** The implementation matches the design it was built from, and
the interesting question was never "does it work" — five phases, four review
rounds and 24 findings got that far before this stage. The question was whether
the *records* tell the truth about the code, and in three places they did not.

Design §7's table is the slice's coverage contract, and it is met row for row. Each
row, with the test that reddens it:

| §7 row | the test that reddens |
| --- | --- |
| project beats user beats binary | `the_project_set_beats_the_user_set_beats_the_binary`; `the_project_layer_beats_the_user_layer` |
| the user layer is really read | `a_set_that_exists_only_in_the_config_home_resolves` |
| the walk-up happens | `the_walk_up_finds_the_repositorys_set` |
| a worktree counts as a repository | `a_worktree_is_a_repository` |
| the walk stops at the repository | `the_walk_stops_at_the_repository` |
| no repository means cwd, no walk-up | `no_repository_means_the_cwd_and_no_walk_up` |
| a broken project set shadows a good user set | `a_broken_project_set_shadows_a_good_user_set` |
| a failed file behind a working one reports its reason | `a_failed_row_behind_a_working_one_keeps_its_reason` |
| the listing reports and succeeds | `the_listing_reports_a_broken_file_and_succeeds` |
| an absent layer is not an error | `an_absent_layer_is_not_an_error` (unit + spawned) |
| an unreadable layer is exit 1 | `an_unreadable_layer_exits_one`; `an_unreadable_layer_is_an_error`; `a_layer_that_cannot_be_opened_is_an_error` |
| a set with no summary lists with `-` | `a_bare_markdown_file_lists_with_a_dash`; `an_in_force_set_without_a_summary_lists_with_a_dash` |
| frontmatter is not prose | `a_named_set_prints_the_body_verbatim`; `frontmatter_is_not_part_of_the_body` |
| the body and the seed agree | `the_prose_and_the_seed_agree_on_the_body` |
| all four derived keys refused by name | `the_four_derived_keys_are_refused_by_name`; `the_four_derived_keys_exit_one` |
| unknown keys are ignored | `an_unknown_frontmatter_key_is_ignored` |
| an empty body is malformed | `an_empty_body_is_malformed` |
| a non-UTF-8 file and a non-UTF-8 name | `a_file_that_is_not_utf8_text_is_a_failed_row`; `a_non_utf8_filename_is_a_failed_row` |
| the seed is fixed and origin-bearing | `the_seed_names_its_source_and_the_authors_version`; `the_source_is_the_conventions_leaf_path` |
| the shipped set is not special | `the_shipped_set_is_the_same_reader_as_a_file` |
| a second install still upserts | `a_second_install_reports_the_body_it_replaced`; `the_seed_declares_once_and_a_second_install_updates_the_same_entity` |
| no store is created | `the_command_still_creates_no_store` (both halves: the absent store *and* the paired `get`) |
| the two roots disagree on purpose | `the_two_roots_disagree_on_purpose` |
| preserved: unknown name at exit 2 | `an_unknown_name_still_exits_two` |
| preserved: `--format json` with no name | `json_without_a_name_is_still_a_usage_error`; `json_without_a_name_is_a_usage_error_naming_the_flag` |
| preserved: unknown format before the operand | `an_unknown_format_is_still_refused_before_the_operand`; `any_other_format_is_refused_before_the_name_is_read` |
| preserved: the seed is one line | `the_seed_is_one_compact_line` (unit + spawned) |

Three rows §7 phrases more strongly than any test: the mode-000 fixture cannot fail
as root (IMP-020, and PHASE-03/EN-3's exists-but-not-a-directory row carries the
class), `parse`'s name derivation is pinned by outcome rather than by signature
(plan item 3), and §5's third field is not disjoint from an author's summary
(obligation 8). All three are the recorded design questions, not silent gaps.

**What this audit added that four review rounds did not.** Five findings, three of
them about records rather than behaviour:

1. **The user root's source (F-1, `major`).** §2 promises `$HOME/.config` and "a
   container with no home … has no user root". The binary asks
   `std::env::home_dir()`, which answers with the passwd entry when `$HOME` is
   unset *or empty* (measured), so that container does read a user layer. The plan
   chose this knowingly (EX-5, EN-4); the design never recorded it. This is the one
   finding I would call consequential: it is the ambient-read hazard R4 names,
   restored one layer above the test suite that guards against it.

2. **`.config/smut` is built twice (F-2, `minor`).** The property holds and the code
   comments were corrected in `e692735`; the two *plan criteria* that say "one
   place"/"nowhere else" still overstate, and they were lodged with reconcile,
   which cannot write `plan.toml`. The brief names `design.md` §2 instead, so the
   truth has an owner.

3. **Obligation 6 quotes a sentence §6 does not contain (F-3, `nit`).** The gap is
   real — §6 sorts "by name" in a vocabulary that admits a nameless entry — but the
   quoted wording is the plan's. A reconcile run would have hunted the wrong
   clause.

4. **A ruling with no implementation and no pin (F-4, `minor`).** Notes record a
   PHASE-01 ruling on an empty `home` and claim it is pinned by one row in
   PHASE-04's file. The row does not exist and the guard is not in the code:
   `user_root(None, Some(""))` returns a relative layer root. Unreachable from the
   binary, so it is not a close-gate item — but the claim that it was done is
   false, and false records are the defect this slice keeps producing.

5. **The fourth false doc sentence (F-5, `nit`).** `load`'s doc counts three
   filesystem reaches "and this is the middle one" where there are two and `load`
   is neither — in the very commit that discharged RV-032 F-3, the finding that this
   doc claimed a monopoly it did not have.

**Standing risks, deliberately left standing.**

- **Validity is not a tie-breaker.** A corrupt project set makes that *name*
  unaskable and shadows a perfectly good user copy. Accepted (DEC-031 rule 1):
  precedence is positional so the advice a caller gets never depends on a parse
  outcome they cannot see. The cost is visible in the listing and in obligation 1.
- **A symlinked `.git` is not a repository** though git resolves it (plan item 6).
  A repository laid out that way would read its project layer from the cwd. Exotic,
  named, ruled as §2's rule read as written.
- **The listing's third field is not disjoint** from an author's `summary`
  (obligation 8). "No two states can claim one row" is the renderer's guarantee, not
  the bytes': `summary: shadowed` renders a shadow-shaped row and a summary holding
  two spaces yields a five-field row. Reproduced by hand during this audit. Owed to
  reconcile as a choice — a disjoint vocabulary or a delimiter the helpers can trust
  — rather than patched in a phase.
- **An unreadable *file* and an absent *layer* are the pair a single `is_dir`
  check would collapse**, and they are two tested rows (RV-029 F-12). The reverse
  pairing — a layer path whose parent is a regular file — is exit 1 by ruling, not
  by design text.
- **`smut conventions` reads the user layer of whoever runs it**, which is what the
  slice wants and what F-1's silent fallback makes less obvious than design §2
  claims.

**Tradeoffs consciously accepted.** The body stays the file's bytes, so printing a
set is not a round trip and `learn` says so in one line (DEC-031). The seed's `id`
and `source` are layer-blind, so a stored record cannot name the layer that supplied
it and the listing is the answer (DEC-017). The plan's two "one place" literals
stand as approved text and are read as shorthand rather than rewritten. No code was
edited at audit: `src/conventions.rs`'s last change is PHASE-05's concluded delta
(`e54f7aa..c6cebe2`), and moving it now would leave the phase registry describing
code no phase concluded — so the two code residuals (F-4, F-5) are CHR-006's, not a
commit at closure.

**A route through the material, for a reader who wants one.** The behaviour at the
centre of the slice is precedence: `load` concatenates project → user → binary and
`set_in_force` takes the first entry for a name, validity never consulted
(`src/conventions.rs:431` and `:458`); `listing` renders the same order, so the
first row for a name is the advice in force (`:680`). The tests that bear on it are
`the_project_set_beats_the_user_set_beats_the_binary`,
`a_broken_project_set_shadows_a_good_user_set` and
`the_binary_row_is_shadowed_by_a_nearer_file`. The remaining risk I would watch is
F-1's: a container's passwd home quietly becoming a conventions layer. Sources, if
you want them verbatim: `doctrine show DEC-031` (precedence is by layer, never by
validity), `doctrine show DEC-033` (exit 2 for what the caller typed, 1 for what the
filesystem supplied), `doctrine show DEC-028` (what a set is — and the sentence
obligation 1 narrows). `/walkthrough` or `/pair` would go deeper on any of these;
neither is needed to close.

## Reconciliation Brief

Five findings, plus the eight inherited obligations and two of the plan's upheld
ambiguities. Grouped by the surface `/reconcile` actually writes. `plan.toml` appears
nowhere below on purpose: `EX-/EN-/VT-` rows are a completed plan's approved text,
and a brief item naming them stalls.

### Per-slice (direct edit — `design.md`)

- **[F-1] §2, *User root*.** Replace the source rule with the value the binary hands
  the pure function: `std::env::home_dir()` — `$HOME` when set and non-empty, else
  the passwd entry. Keep "the layer root is `<config-home>/smut`" and "no home at
  all is an absent layer" (now: no `$HOME` and no passwd entry). Note that the
  signature `user_root(config_home: Option<&Path>, home: Option<&Path>)` is
  unchanged, so "the binary is the only code that reads the environment" now names
  the whole read.
- **[F-2] §2, after "The declaration-kind segment is joined exactly once, inside
  `load`".** Add: the *compound* `.config/smut` segment is deliberately constructed
  twice — `Roots::under` onto the repository root, `user_root` for the home fallback
  (`<home>/.config` then `smut`), because a config home already names `.config`. One
  join *per layer root* is the rule; one join for the segment is not.
- **[F-3] §6, the sorting clause (`design.md:246`).** Append: sorted by name; an
  entry whose filename is not UTF-8 sorts last, and two such entries order by their
  raw filename, so the order is total whatever `read_dir` returns. (Cite
  `src/conventions.rs:642` `sort_layer` and
  `two_undecodable_names_break_their_tie_on_the_raw_filename`.)
- **[obligation 6] §6** — same clause as F-3; the obligation's quoted text is the
  plan's, not §6's, so write the sentence against :246 as it stands.
- **[obligation 4] §5's worked listing and §4's refusal clause.** §5's binary
  `hygiene` row prints its summary while project and user both hold `hygiene`; under
  the third-field rule (repeated as DEC-031 rule 4) it must read `shadowed`. §4's
  "one clause per resolved name" must be restated as one clause per resolved
  *file*, matching its own example (three `hygiene` clauses for three files) and
  `clauses()` at `src/conventions.rs:864`.
- **[obligation 5] `design.md:157`'s quoted diagnostic.** The example quotes
  `frontmatter line 2 is not `key: value``; `not_key_value`
  (`src/conventions.rs:361`) now says "is not a flat `key: value` line: the key is
  one word with no space in it, and a `:` follows it". Staleness, not a rule change:
  update the quotation or drop it and name the class.
- **[obligation 7] §3, the candidate filter.** §3's table lists "cannot be opened or
  read" as a reasoned row while the filter requires "a regular file", so a dangling
  `x.md` is either a row or nothing. One sentence: such a candidate is a failed row,
  and a directory whose name is undecodable stays silent because the metadata test
  runs first (`read_layer`, `src/conventions.rs:507`).
- **[obligation 8] §5's third field.** Declare the field's vocabulary disjoint from
  author-supplied summaries, or give the listing a delimiter its readers can trust.
  Reproduced: `summary: shadowed` renders a shadow-shaped row; a summary with two
  spaces yields five fields, breaking the suite's own `split("  ")` reading. Raised
  as RV-033 F-7 and deliberately not patched in a phase.
- **[plan item 3] §6, `parse`'s signature.** `parse(bytes, origin)` derives the name
  from `origin.path`, which is safe because a non-UTF-8 filename never reaches it.
  One sentence; the plan ruled the design's silence worth recording.
- **[plan item 4] §6, the unreadable layer directory.** §5 puts it at exit 1 and §6
  says `load` returns a `Result` for it, but the one new variant carries `Origin` and
  `Cause` and a directory has no `Origin`. One sentence: that class is
  `StoreError::FileSystem` with a layer-worded `step` (`reading the <kind> layer`),
  so no diagnostic about a set claims a store.
- **[plan item 9] §5's "absolute for a file layer".** Falsified by a relative
  `$XDG_CONFIG_HOME` (row: `a_relative_origin_renders_a_relative_row`). The plan
  ruled "no design change"; one clause — "absolute exactly when its layer root is" —
  would make §5 true and costs nothing.
- **[design questions A–C from RV-030]** owed a ruling rather than a repair: **A**
  duplicate frontmatter keys — the last silently wins (`frontmatter`,
  `src/conventions.rs:318`), which DEC-028 neither permits nor refuses and which is
  the silent-wrongness class DEC-028's derived-key refusal exists for; **B** three
  input shapes silently demote frontmatter to prose (`--- ` with a trailing space,
  CRLF, a leading blank line — `parse`, `:246`) and nothing warns; **C** the body
  trim is asymmetric by design (trailing newlines trimmed, a leading blank line
  after the fence preserved), so a conventionally written set prints and seeds a
  body opening with an empty line. A and B most naturally land as one sentence each
  in §3 and, for A, in DEC-028's revision; C is a §1/§3 clarification.

### Governance/spec (REV)

- **[obligation 1] DEC-028** — narrow "A malformed file never breaks the other
  layers" to listing and loading, not name resolution: a broken project set *does*
  make that name unaskable, which is DEC-031 rule 1's intent. In-place revision, the
  repo's practice (`REV` modify).
- **[obligation 2] DEC-026** — its cost paragraph (`record-026.md:44`) still promises
  "origin printing plus a set header naming its own file", which DEC-031 declined.
  Amend to point at the listing.
- **[obligation 3] DEC-017** — "Adding a later set is a file plus a match arm"
  (choice and consequences) is no longer how sets arrive; a project or a user
  supplies them, and the binary's table is one row per shipped pair.
- **[design question A, if ruled as a refusal or as a format rule] DEC-028** — inside
  the same revision as obligation 1, while the format is one phase old.

### Not in this brief, on purpose

- **`plan.toml`** — off-surface (see above). The two false literals are read as
  shorthand; F-2 gives the truth a home in `design.md`.
- **Code and tests** — F-4 and F-5 are `CHR-006`; the mode-000 skip is `IMP-020`;
  `every_store_refusal()`'s missing `NoFreshBasename` row is `IMP-019`.
- **Close-stage lifecycle** — `IMP-013` (the implementation brief this slice
  delivers) is still `open` and should be resolved at close, noting the slice that
  landed it; `SL-006` unblocks on this slice's terminal transition; `CHR-001`,
  `QUE-002` and `IMP-008` stay open by design.

## Reconciliation Outcome

Confirmed before writing: `doctrine review status RV-035` reads `done · await=none`, all five
findings `verified`, pass concluded. Findings were **not** mutated; remediation is recorded
here.

### Direct edits applied — `design.md` (per-slice, direct edit)

- **§2 *User root*** — the source rule now names `std::env::home_dir()` (`$HOME` when set and
  non-empty, else the passwd entry), and "no home at all" means neither resolves. **F-1**,
  obligation 9's finding.
- **§2, after the declaration-kind sentence** — the compound `.config/smut` segment is stated
  as deliberately built twice (`Roots::under`; `user_root`'s home fallback), so "one join per
  layer root" is the rule and "one join for the segment" is not. **F-2**; gives the truth a
  surface the plan criteria's "one place"/"nowhere else" literals can no longer contradict.
- **§3 candidate paragraph** — a candidate whose metadata cannot be read is the failed row; a
  directory with an undecodable name stays silent because the metadata test precedes the name
  decode. **Obligation 7.**
- **§3 frontmatter** — a repeated key is the second value; the exact fence test demotes
  frontmatter to prose for `--- ` with a trailing space, CRLF, or a leading blank line. Both
  named as limitations rather than repaired, because repairing either changes the on-disk
  format. **Design questions A and B** (and the same two facts in DEC-028).
- **§3 *Title and body*** — the trim's asymmetry is stated: trailing newlines go, a leading
  blank line after the fence stays. **Design question C.**
- **§4 refusal** — "one clause per resolved *file*" (the file is the unit of listing), an
  undecodable filename contributes no clause, and the quoted diagnostic now reads
  `is not a flat \`key: value\` line`. **Obligation 4 (§4 half), obligation 5.**
- **§5 worked listing** — the binary `hygiene` row reads `shadowed`, with a paragraph saying
  why the rule beats the example and the shipped set is not exempt. **Obligation 4 (§5 half).**
- **§5 fourth field** — absolute exactly when its layer root is; nothing rewrites a path.
  **Plan item 9**, which the plan had declined; the audit recommended the clause and the
  operator agreed.
- **§5 third field** — the disjointness *ruling*: the four states are the renderer's choice, a
  summary interpolates verbatim, so `summary: shadowed` and a two-space summary can shape a
  row; "no two states claim one row" is a guarantee about the renderer, not the bytes. Nothing
  is escaped, because the listing is a report before it is a table (DEC-031 rule 3).
  **Obligation 8**, and the choice named in the brief — the alternative (a delimiter the
  helpers can trust) needs a code change and is not this pass.
- **§6 `load` paragraph** — one rewrite carrying four items: "every filesystem read *of a
  layer*" (so `project_root`'s walk and `parse`'s bytes are not claimed), the sort tie-break
  (undecodable last, ties on the raw filename, order total), the unreadable-directory class
  reusing `StoreError::FileSystem` with a layer-worded `step`, and `parse` taking no name
  parameter because `origin` carries the stem. **F-3** (obligation 6's *real* clause is now
  the sentence it describes), **plan items 3 and 4**, and the design-side half of F-5's
  confusion.

### Knowledge amendments (`doctrine knowledge edit`) — no REV

The brief filed DEC-028/026/017 under "Governance/spec (REV)". **Corrected during this pass:**
the REV target set is `SPEC | PRD | REQ | ADR | POL | STD`, and the engine refuses a DEC target
(the precedent is REV-001, abandoned for exactly that reason; REV-002's narrative documents the
same six-decision landing through `doctrine knowledge edit` in SL-004). Nothing in this slice
targets a SPEC/PRD/REQ/ADR/POL/STD, so **no revision was created or needed** — a REV here would
have failed at `revision change add`.

- **DEC-028** (`choice` + dated body note) — the malformed-file sentence narrowed to *never
  takes the process down, never stops another layer being listed, and does shadow a working
  user copy of the same name*; plus the duplicate-key and exact-fence rules. **Obligation 1,
  design questions A and B.**
- **DEC-026** (body prose, `record-026.md` § *The cost we accepted*) — "a set header naming its
  own file" replaced with the listing row that names the file in force, which is what DEC-031
  kept when it declined the header. **Obligation 2.**
- **DEC-017** (`choice` + `consequences` + dated body note) — "a file plus a match arm" replaced
  with the three-layer arrival and, for the binary, a file plus one shipped-table entry.
  **Obligation 3.**

Each amendment carries a dated section naming the audit finding or reconcile obligation that
drove it, and each leaves an id, a slug and unrelated rationale untouched.

### Withdrawn / tolerated

- **Nothing withdrawn.** Every finding held under evidence.
- **F-4 and F-5 are `follow-up`** — `CHR-006`, minted by the audit, carries both code
  residuals with their replacement text. Not landed here: a source edit at reconcile would
  leave `src/conventions.rs` changed after PHASE-05's concluded delta with no phase that
  concluded it.
- **The design run stays at revision 36.** `doctrine design adopt` refuses on a locked run and
  says so explicitly: "at reconcile, edit `design.md` directly and do not adopt". `design show`
  therefore reports `design.md is behind its run (revision 36)` — expected, and the reason the
  reconciled prose is the file rather than the run's baseline.
- **Carried to `/close`, not reconcile's**: `IMP-013` is the implementation brief this slice
  delivers and should be resolved with the slice named; `SL-006` unblocks on the terminal
  transition. `CHR-001`, `QUE-002`, `IMP-008`, `IMP-019`, `IMP-020` stay open by design.
