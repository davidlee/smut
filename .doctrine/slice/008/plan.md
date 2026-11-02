# Implementation Plan SL-008: Conventions are layered — a set is a file the project, the user or the binary supplies, and every listing says which

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

The locked design (run `dr-01a19b61`, revision 36) replaces a module rather than
extending one: `src/conventions.rs` today is a compiled-in set printed three ways,
and the slice turns it into a loader over three roots whose answers carry their
origin. Five phases, cut so that each one is a complete, independently green
behaviour with its own evidence, and so that no phase leaves a surface half-switched
on the tree.

The design is the authority for shape — §1's types, §6's signatures, §7's
properties. This plan adds sequencing, the fixtures, and the handful of decisions
the design left at the level of a phrase (§ Decisions taken below). Nothing here
widens the scope: section addressing, the set's content, README prose, and anything
inside `.smut/` stay out.

## Sequencing & Rationale

**Why roots first.** `project_root` and `user_root` are pure functions over values
they are handed and touch nothing else in the module. Landing them first buys the
slice's most novel edge cases — a `.git` directory versus a worktree's `gitdir:`
file versus a symlink, and the three shapes of a config home — an isolated red/green
cycle, with the existing surface untouched and the suite still green. It also puts
RV-029 F-17's obligation where it belongs: the terminus is a property of
`project_root`, and proving it here means the later phase only has to prove it again
through the binary.

**Why the reader before the loader.** `parse` is pure over bytes, so it can be
proved exhaustively — four derived keys, unknown keys, fence position, blank lines,
an empty body, the title and the trailing-newline rule — without a filesystem. It is
also the phase where the design's vocabulary arrives, and that vocabulary is what
makes the shipped set a case of the general rule rather than a constant: the
`include_str!` bytes go through the same reader a file does, the summary comes from
the file's own frontmatter, and the title stops being synthesised in code. Landing
`parse` and the types before `load` means the loader is written against proved
pieces, and it means the `unwrap`-shaped shortcut (parsing the embedded bytes with
an `expect`) never has to exist: a set that cannot be loaded is a refusal class from
the first phase that can produce one.

**Why the loader precedes its consumer.** `load` is the slice's risk centre — absent
versus unreadable, the candidate filter, non-UTF-8 filenames, the sort, the kind
joined once, the crate's version applied to the binary layer only. Writing it with
nothing calling it is deliberate: it gets a phase whose evidence is the loader, not
the loader plus a rewritten command surface. The cost is one phase where a public
function has no caller in the binary; the benefit is that PHASE-04 debugs the
surface against a loader already known to behave.

**Why the layer-root join is its own function.** The design names three roots and one
relative suffix, `<root>/.config/smut/<kind>/` (`slice-008.md` scope 1, DEC-026), but
§2's `project_root` returns the repository and §2's `user_root` already returns
`<config-home>/smut`. The `.config/smut` segment therefore belongs to a third thing,
and if nothing owns it `load` reads `<repo>/conventions/` — both audit lanes found
this independently, and §5's example path and §7's walk-up row both print the
segment. So `Roots::under` (PHASE-01/EX-3) is where it is joined, once, with a unit
test over the constructed value; `project_root` keeps returning the repository,
`user_root` keeps returning `<config-home>/smut`, and `load` still joins only
`<kind>`. One function, one segment, one test — not a second suffix rule spread over
three callers.

**Why the surface swaps in one phase.** `answer` gains its roots and its `Result`,
`Ask` becomes names, the listing gains two columns and two verdicts, the unknown-name
refusal gains origins, and `main` gains the environment reads — all of which must
move in one commit because they are one compile unit: the signature change is forced
by the loader, the listing's shape is forced by the layers, and `main` is forced by
the signature. `error.rs` needs a second convention refusal class, and the guide and
the store format become untrue the moment this lands, so they land with it. That is
the plan's largest phase; it is also the phase with the most independent evidence per
criterion, which is the trade the §7 table makes possible.

**Why the seed is last.** The declaration is the only surface that has to stay
*identical* in shape while gaining a field: fixed id, verbatim title, body equal to
the prose less one newline, one compact line, upsert reporting `previous`. It depends
on the layers (the version rule distinguishes a file set from the shipped one) and on
the answer path, and it is small enough that a phase of its own keeps PHASE-04
reviewable. It is also where the two strongest byte-identity assertions live — prose
versus body, body versus seed — so it is the right place for the slice to end.

Order is therefore evidence order: pure branch coverage, then pure parsing, then the
filesystem, then the process surface, then the wire format. Each phase's tests are
the next phase's floor.

## Decisions taken

The design fixes shape; these are the points where it named a property and left the
mechanism to the implementation. Each is recorded with the criterion that carries it.

- **A non-UTF-8 filename is an entry with no name.** §1 says such a file "never
  becomes an `Entry` with a name" but `Entry { name, origin, outcome }` gives the
  field no room for that. `Entry.name` is `Option<String>`, `None` only for that
  case, and the listing prints the token `<not UTF-8>` in the name column
  (PHASE-02/EX-1, PHASE-04/EX-2). No lossy `String` is derived: the fourth column is
  the path, rendered by `Path::display`, which is std's display of a path rather than
  a name we built.
- **A new refusal class lives in the existing two-class taxonomy.** §6 asks for one
  variant holding `Origin` and `Cause` with exit code 1, and `Error` documents
  exactly two classes. `StoreError::ArtefactNotLoaded` joins the runtime class, whose
  documented meaning is "a storage or runtime failure"; the name says artefact, not
  store, and the message claims no store (PHASE-02/EX-6). It joins the three
  hand-maintained lists that notice a new refusal: the `Display` arm, the
  `every_store_refusal()` inventory, whose length the phase's criterion requires the
  test to assert — today nothing pins it, so a variant left out of the vec is simply
  untested — and the boundary table.
- **The unreadable-layer fixture must not depend on who runs it.** §7 names a
  directory with mode 000, which a root user can still read, so the primary fixture is
  a path that exists and cannot be read as a directory at all, with mode 000 as a
  second row where the platform allows it. The class is unchanged — "exists and
  cannot be read" — and the row stops being able to pass by accident
  (PHASE-03/EN-3, PHASE-04/EX-8).
- **The refusal's separator and field name follow the new content.** §4 renders one
  clause per resolved file joined by `; `, where today the refusal joins bare names
  with `, `. The field holds rendered clauses and says so, so the type does not lie
  about what it carries (PHASE-04/EX-3).
- **The crate's version is applied in one place.** `parse` knows no releases (§3), so
  the binary-layer helper from PHASE-02/EX-4 applies it as the layer is built, and
  `load` calls that helper rather than injecting a second time (PHASE-03/EX-5).
- **The project layer's `.config/smut` segment is joined by `Roots::under`.** §2 names
  two functions that cannot join it — one returns the repository, the other returns
  `<config-home>/smut` — while DEC-026 and §7 both require it. A third, pure function
  joins it once and a unit test pins the value, so no root carries a kind, no caller
  repeats a suffix, and `load` still joins only `<kind>` (PHASE-01/EX-3, EX-4).
- **The set's name is derived in `parse`, from `origin.path`.** §1 needs the filename
  stem and §3 turns a non-UTF-8 filename away before any read, so the derivation can
  only live where both the bytes and the origin are: `parse` (PHASE-02/EX-1). §6's
  signature does not say so — one of the questions owed to `/feedback` below.
- **An unreadable layer directory refuses through `StoreError::FileSystem`.** It has
  no file, so §6's one `Origin`-carrying variant cannot represent it, though §5 puts
  it at exit 1. The existing variant already renders `<step> failed for <path>: …`
  with no store vocabulary; the phase words `step` as the layer read and widens the
  doc comment's scope from store boundaries to layer reads (PHASE-03/EX-2).
- **The binary row is `shadowed` like any other.** §5's third-field rule is worded
  over nearer layers; its own worked example's binary row prints a summary in that
  state. The rule is the normative text and the example row is the slip, so the
  shipped set is not exempt — the alternative would make the one row whose origin is
  not a file the only row that reports differently (PHASE-04/EX-2).
- **A known key with no value declares nothing.** `summary:` alone is a `key:` with an
  empty value, and §5 already has the vocabulary for "in force, declares no summary" —
  the dash — so the field is absent rather than fatal: `summary` lists as `-` and
  `version` is omitted from the seed (PHASE-02/EX-2). Reading it as malformed would make
  a content-valid file unaskable, and because a broken nearer set still wins its name
  (DEC-031) a stray colon would shadow a working user copy. The mistake worth catching
  loudly survives either way: a value continued across lines is not flat, and its
  continuation line fails naming the line.
- **§7's preserved behaviours survive under new names.** §7 says they must stay green
  "without a rewrite", and none of them changes; but a keyword equal to a test name
  that already exists is satisfied before the phase starts, so each survivor is
  asserted under a name that can only exist afterwards (PHASE-04/EX-7). The
  unknown-name message necessarily changes in PHASE-04, so its test cannot keep the
  old name in any case; renaming the other three keeps the gate meaningful rather
  than cosmetic.

## Critical pass

What is under-specified, assumed or risky in this plan, and what is done about it.

- **PHASE-02 is the widest cut** — types, reader, shipped bytes, `Ask` change, the
  new refusal class, `hygiene.md`, one guide line and one `src/main.rs` assertion. It
  cannot be split without either leaving the crate uncompilable or inventing a
  temporary shape that PHASE-04 would delete; the phase's criteria are written as
  separable assertions so a review can reject one without rejecting the phase. Two
  sessions is the honest estimate. The seam, if it is needed, is **A** = types +
  `parse` + `Cause` + the `ArtefactNotLoaded` variant + the parser unit tests (purely
  additive, nothing removed, green) and **B** = `SHIPPED` and its helper + the
  `Ask`/`answer`/`listing`/`seed` adaptation + `hygiene.md` + the guide line + the
  `src/main.rs` and `tests/conventions.rs` moves. The seam is *not* "`hygiene.md` plus
  the guide line": once `title_of` is gone the title is the file's own heading, so the
  frontmatter and the heading must land with the reader.
- **`src/main.rs` is compile-forced in PHASE-02, not PHASE-04.** Its in-process test
  asserts `conventions::HYGIENE.prose`, and `HYGIENE` retires in PHASE-02. PHASE-02/EX-10
  owns that one assertion so the phase can end green; PHASE-04/EX-5 keeps only the
  move to the spawned suite and the roots threading. Without this, the phase's own
  exit criterion ("`cargo test` is green") is unreachable.
- **The `.config/smut` gap is the slice's one unowned obligation, and it is now owned
  by a name.** Two independent audits reached it; the repair is one function and one
  unit test in PHASE-01 plus the walk-up row that would have caught it. The cheap
  wrong fix — teaching a fixture that `<repo>/conventions/` is the project layer — is
  ruled out by writing the expected path into the criterion.
- **`dispatch` sits between `main` and `execute`, and the in-process tests call it.**
  PHASE-04/EX-5 names it, because roots computed one function too high would put a
  developer's `$HOME` back under every unit test — the defect F-3 recorded.
- **A keyword that already exists is not a gate.** `verify-vt` matches substrings, so
  `pub struct Set`, `pub fn seed`, `source`, `conventions::answer`,
  `every_store_refusal`, `.config/smut/conventions/` and `fn the_command_creates_no_store`
  were all satisfied before the phase that cites them. Every keyword is now a test name
  or a symbol whose spelling can only exist after its phase, and names cited in a
  criterion appear in that phase's keyword list character for character.
- **The listing is a diagnostic, not a format.** Two spaces and a fourth column that
  is a path cannot be parsed reliably, because a summary contains spaces. Nothing in
  the plan tests the listing by parsing it; every assertion names a row and reads its
  fields, which is also what §7 asks for. A caller who needs a machine channel has
  `--format json`, which is the seed and is unchanged in shape.
- **`Ask` carries a resolved set in PHASE-02 and becomes a question in PHASE-04.**
  That is a shape the plan changes on purpose: with one layer, the answer cannot fail;
  with three, resolution is what can fail, so the question has to survive parsing.
  The criterion pair that proves the change is deliberate is PHASE-02/EX-5 and
  PHASE-04/EX-1, and `cli::parse`'s unit tests are the control that no filesystem
  reach leaks into step 1.
- **Two roots in one process is intended, and it is tested rather than smoothed.**
  A cwd inside a repository reads the repository's sets and writes its own store;
  PHASE-04/EX-7 pins both halves in one fixture. Teaching the store to walk up stays
  a non-goal (DEC-004).
- **The suite is rewritten on purpose, so the evidence is the assertions.** Each row
  names the change that reddens it; PHASE-04/EX-8 and PHASE-04/VA-2 require that the
  §7 table be walked row by row and that uncovered rows be named rather than assumed
  covered. The rewritten seam is the one this repository would normally treat as
  suspect, which is why the coverage walk is a criterion and not a habit.
- **A green review state is not coverage.** RV-029's recorded pass predates §5's and
  §6's hedge edits; both are attested, neither was passed over. PHASE-04/EN-2 carries
  that fact forward so the audit inherits it instead of rediscovering it.
- **Concurrent threads.** This tree holds other work: `Cargo.toml`, `README.md` and
  `tests/readme_example.rs` belong to someone else. Commits are path-limited, never
  `-a`, never `--amend`, never a reset past the phase's own commit. `README.md`'s two
  compiled-in paragraphs go stale by design and are CHR-001's, not this slice's.

## Model requirement, per phase

`plan.toml` has no field for this, so it lives here. The tiers are the skill's
vocabulary, not a staffing instruction: they say what the phase punishes.

- **PHASE-01 — careful adherence with adversarial fixtures.** Two mechanical but
  invisible traps: `fs::metadata` instead of `symlink_metadata` would make a `.git`
  symlink a repository, and joining `.config/smut` anywhere but `Roots::under` would
  duplicate the suffix. A competent mid-tier model with the criterion list succeeds.
- **PHASE-02 — sustained frontier reasoning.** The frontmatter reader carries the
  subtle invariants (fence only on the first line, `---` as a horizontal rule
  outside it, blank lines inside it, a line number that must survive into both
  `Cause` and the sentence, four refusals that must not collapse, byte-identical
  bodies) in the same session as a public-shape rewrite. Two sessions.
- **PHASE-03 — the phase that must not be handed to a weak model.** Every failure
  mode here is silent: an unreadable directory treated as absent, a lossy filename
  decode, `read_dir` order leaking into the listing, `<kind>` joined twice, the
  crate's version injected twice. A happy-path suite passes with all five present;
  the phase's criteria are the only defence.
- **PHASE-04 — careful adherence at volume, with one frontier-shaped invariant.** The
  invariant is the third field's fixed precedence (failure beats shadow beats summary
  beats dash) plus shadowing being positional rather than a parse outcome; the rest is
  roughly twenty exact requirements across six files, and a model that drops items
  from a long list fails even though nothing is hard. Two to three sessions behind one
  commit; if a split is forced, the seam is A = the surface swap + rewriting the
  existing tests onto the new shapes + `main`'s `roots()`/`dispatch`, and B = the new
  adversarial spawned rows + the guide and store-format clauses. Splitting into a sixth
  phase would land the layering with only the old behaviours pinned, which is worse
  for a phase whose whole point is the diagnostic.
- **PHASE-05 — relatively mechanical.** One field, one version rule with three shapes,
  one six-name key list, one upsert already covered; the only care point is EX-3
  asserting the prose-newline and seed-body facts in one test.

**Escalation triggers.** Stop and re-plan rather than widening a phase when: the
design is revised on where the project layer root comes from (PHASE-01/EX-3 and
PHASE-04/EX-5 move together, and the join must stay in exactly one function); an
operator turns out to rely on a symlinked `.git` (that is §2's rule to change, not
PHASE-01/EX-1's); a new error variant is preferred over reusing
`StoreError::FileSystem` for an unreadable layer (PHASE-03 would then owe an
`src/error.rs` inventory edit its criteria do not budget for); or a `$TMPDIR` inside a
checkout reddens a no-repository fixture (a local fixture fix, not a criterion
change). A model that cannot hold PHASE-04's list splits at the stated seam — it does
not drop rows.

## Design questions, adjudicated

Found by the two hardening audits and taken through `/feedback` against the design text on
2026-11-02. Each is answered in the plan by the reading the design's own rules make least
wrong, and that reading is named in the criterion that carries it, so it is visible rather
than smuggled in. The dispositions are at the end of this section: an item upheld as a
defect is owed to reconcile, an item resolved by the design's own text is not a defect.

1. **§2 never assigns the `.config/smut` segment.** `project_root` returns the
   repository, `user_root` returns `<config-home>/smut`, §6's `load` joins only
   `<kind>` — yet §2's own summary, DEC-026, §5's example path and §7's walk-up row all
   require `<repo>/.config/smut/`. Reading: `Roots::under` joins it once
   (PHASE-01/EX-3).
2. **§5's worked listing contradicts its own third-field rule.** The rule shadows a
   row when a nearer layer holds the name; the example's binary `hygiene` row prints a
   summary while project and user both hold `hygiene`. Reading: the rule wins, the
   binary row says `shadowed` (PHASE-04/EX-2).
3. **§6's `parse(bytes, origin)` never says where the name comes from**, while §1
   makes the name the filename stem and §3 requires a non-UTF-8 filename to be reported
   before any read — i.e. outside the pinned signature. Reading: `parse` derives the
   stem from `origin.path`, which is safe because a non-UTF-8 filename never reaches it
   (PHASE-02/EX-1).
4. **An unreadable layer *directory* has no `Origin`.** §6 adds exactly one variant,
   carrying `Origin` and `Cause`; §5 nevertheless puts an unreadable directory at exit
   1. Reading: `StoreError::FileSystem` with a layer-worded `step` (PHASE-03/EX-2).
5. **`seed(set: &Set)` has no `kind`** although `source` is `<kind>/<name>.md` and
   `load` is kind-generic. Reading: `source` is the literal `conventions/<name>.md`,
   the same at every layer; the seed stays `seed(set)` (PHASE-05/EX-1).
6. **A symlinked `.git` is a repository to git and not to §2**, whose rule is written
   over the entry's own kind — so a repository using that layout would silently read
   the project layer from its cwd. Reading: §2's rule as written, and the walk
   continues (PHASE-01/EX-1), because the alternative invents a rule the design does
   not state.
7. **§7's mode-000 fixture cannot fail as root**, so the row can pass by not running.
   Reading: the exists-but-not-a-directory row carries the class, mode 000 is a second
   row only where the platform can fail it, and a skip says so in its own message
   (PHASE-03/EN-3).
8. **§3 admits `key: value` with an empty value.** Decided by the operator: a known key
   with no value declares nothing, so the field is absent (`summary` lists as `-`,
   `version` is omitted from the seed) — PHASE-02/EX-2 — while a value continued across
   lines still fails naming its line.
9. **§5 says the fourth field is absolute for a file layer**, which cannot hold when
   `$XDG_CONFIG_HOME` is relative. Reading: the field is the layer root's join, so it
   is absolute exactly when the root is (PHASE-04/EX-2).
10. **§4's prose says "one clause per resolved name"; its example renders one per
    file.** Reading: the example (PHASE-04/EX-3, as written).
11. **§2's walk-up fallback cites DEC-028, and that citation is correct** — DEC-028 carries
    the section *The walk-up terminus* verbatim. Nothing to change.

### Dispositions

Against the design text, not the auditors' framing. `U` = upheld as a design defect or a
real ambiguity; `R` = rejected, with the evidence; `P` = the design was right and **this
plan** was the defect.

- **1 — `.config/smut` unassigned: P.** §2 line 64 says the two roots *are* the `…/smut`
  directories, §5's example path shows `<repo>/.config/smut/conventions/`, and DEC-026 fixes
  the suffix — the design assigns it; no phase of this plan pinned it. The plan gap is fixed
  (PHASE-01/EX-3); no design change is owed.
- **2 — §5's binary row against its own rule: U.** DEC-031 rule 4 repeats the rule, so the
  example row is the slip. Owed to reconcile: the row must read `shadowed`.
- **3 — `parse` and the name: U (silence only).** §1 defines the name, §3 requires a
  non-UTF-8 filename to be turned away before any read, and §6 pins `parse(bytes, origin)`
  with no name parameter — the signature admits exactly one reading, which the plan takes.
  No design change is required; the silence is worth one sentence in a later revision.
- **4 — an unreadable layer directory has no `Origin`: U (under-determined, not false).** §5
  puts it at exit 1 and §6 says `load` returns a `Result` for it, but the only new variant
  carries `Origin` and `Cause`. The plan reuses `StoreError::FileSystem` with a layer-worded
  step, which keeps §6's promise that no diagnostic about a set claims a store.
- **5 — `seed(set)` has no `kind`: R.** §5 fixes `source` as the literal
  `conventions/<name>.md`, identical at every layer, and §6's signature with no kind confirms
  it. The `<kind>/<name>.md` this plan carried was the plan's error, already corrected.
- **6 — a symlinked `.git`: U (tolerated).** §2's rule for *any other kind* of `.git` entry
  is explicit, and the plan implements it as written. It diverges from git, which resolves a
  symlinked `.git`; the exposure is exotic and the alternative invents a rule the design does
  not state. Recorded, not repaired.
- **7 — the mode-000 fixture cannot fail as root: U.** A test-design defect, not a behaviour
  one; the plan's portable row carries the class instead (PHASE-03/EN-3).
- **8 — a known key with no value: settled, absent.** The operator chose the absent
  reading over the malformed one: `summary:` alone declares nothing, so the field is
  absent and the listing shows the dash (PHASE-02/EX-2). The empty column the malformed
  reading guarded against does not exist under this one, and the multi-line mistake is
  still caught by the continuation line.
- **9 — the fourth field's absoluteness: U (nit).** With a relative `$XDG_CONFIG_HOME` the
  field is as absolute as its root. The plan reads it as the root's join; no design change.
- **10 — §4's "one clause per resolved name" against its own per-file example: U (nit).**
  The example is the operative rendering (§1 makes the file the unit of listing). Folded
  into item 2's reconcile obligation.
- **11 — §2's DEC-028 citation: R.** DEC-028 owns the walk-up terminus in its own section;
  the citation is exact, and the finding was confabulated.

## Carried, not re-litigated

- **QUE-002** — the environment override's name, target and rank: deferred, and the
  loader taking roots as parameters is what removed the last reason to ship it now.
- **IMP-008** — the section grammar and its flag: deliberately later, and free to
  arrive because the loader stores a body and never interprets it.
- **Three reconcile obligations** — DEC-028's malformed-file clause, DEC-026's "set
  header naming its own file" cost sentence, DEC-017's "a file plus a match arm" —
  recorded in `notes.md` § Obligations owed at reconcile. No phase edits them; they
  are reconcile's, and a phase that "fixes" them mid-implementation has left its
  scope.
- **SL-006** moves back into reach when this slice lands, and not before.
