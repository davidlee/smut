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
  store, and the message claims no store (PHASE-02/EX-6). It joins both
  hand-maintained inventories, which is the only thing that notices a new refusal.
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

## Critical pass

What is under-specified, assumed or risky in this plan, and what is done about it.

- **PHASE-02 is the widest cut** — types, reader, shipped bytes, `Ask` change, the
  new refusal class, `hygiene.md` and one guide line. It cannot be split without
  either leaving the crate uncompilable or inventing a temporary shape that PHASE-04
  would delete; the phase's criteria are written as separable assertions so a review
  can reject one without rejecting the phase. If execution finds it too large, the
  seam to split on is `hygiene.md` plus the guide line, which are the phase's only
  content-only edits.
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
