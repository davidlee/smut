# Conventions are layered — a set is a file the project, the user or the binary supplies, and every listing says which

## Context

`smut conventions` ships exactly one set, compiled in: `src/conventions.rs` holds a
`Set` of three `&'static str` fields, `include_str!`s `src/conventions/hygiene.md`, and
`resolve` — called from `cli::parse`, before any store exists — returns an `Ask` that
carries the resolved set by reference, so `answer` is pure text. Nothing is reachable
from outside the binary: a project cannot ship its own set, a user cannot override the
shipped one, and a set cannot be traded between projects.

That matters because a conventions set is the only mechanism smut has for making a
store queryable later. Three independently briefed authors wrote three 34-entity
stores and **0 of 3** followed the one rule that decides whether `where` works
(IMP-011). Advice compiled into a binary reaches an author only if they run the
command and read it; advice that ships with the project arrives with the work.

The design is settled and is *not* this slice's work: **DEC-026** (where sets live:
one relative suffix, three roots, local wins, origin always printed), **DEC-028**
(what a set is: one markdown file, filename names it, frontmatter carries `summary`
and `version` only, derived keys refused, unknown keys ignored, malformed files
isolated to themselves), **DEC-029** (the name stays `conventions`), on top of
**DEC-017** (the seed, its fixed id, upsert-and-report-`previous`). The implementation
brief is IMP-013's. This slice is the code.

Why now: it is the last open gate on the `epistemic-records` set — SL-006, deferred
from IMP-004 — because a domain set is worth drafting only when a project can ship
it. The identity gate cleared with SL-004 (`86460d4`), and 0.2.1 is released
(`09708c8`).

## Scope & Objectives

1. **A loader over three roots.** One relative suffix, `<root>/.config/smut/<kind>/`,
   read from the project root, then the user's config home, then the binary; project
   beats user beats binary, and same-named sets do **not** merge — the loser is
   reported as shadowed. The project root is the walk-up terminus: the git root when
   there is one, otherwise the current directory with no walk-up.
2. **The shipped set takes the same code path.** Its bytes are parsed by the same
   reader a file goes through, so "the shipped set is not special" is a property of
   the code rather than a claim in a document. The `include_str!` stays — it is what
   makes the bytes that ship reviewable in the tree.
3. **The listing becomes the diagnostic.** Name, summary, layer, path, and a one-line
   reason for any file that did not load. This is where a mis-placed set is
   discovered, so it must be complete: every layer, every file, including the ones
   that failed.
4. **The unknown-name refusal lists resolved names and their origins**, refusals
   included — the same information as the listing, in the same shape.
5. **Seeds keep their identity and gain their origin.** `smut_id` stays
   `conventions-<name>` (DEC-017's upsert semantics, unchanged), `source` names the
   file relative to its layer root, and `version` is the author's for a file set and
   the crate's for the shipped one.
6. **`Ask` becomes owned.** `resolve` gains file I/O, `answer` stays pure, and the
   `&'static` lifetime disappears with the fixity that justified it.
7. **The loader is generic over declaration kind.** `<kind>` is a parameter, so a
   future artefact kind reuses the ladder instead of re-deriving it. Nothing is built
   for a second kind: this is a shape decision, not a feature.

## Non-Goals

- **Section addressing.** DEC-028 pins the grammar (normalised match, ambiguity
  refuses, level-1 is the whole set, unknown section lists what exists) and says it
  is implemented later; IMP-008 owns the flag that would consume it. The grammar is
  recorded, not built.
- **Teeth.** A set never refuses a declaration, never validates a store, never
  becomes a schema. A malformed *set file* is isolated and reported; a stored entity
  is still whatever was stored.
- **Any file inside `.smut/`.** The store's layout stays owned by
  `docs/store-format.md` under STD-001 and does not change. One clause *in* that file
  ("the binary shipped today embeds its one set") is amended at landing, because
  STD-001 makes it the store format's only record and it would otherwise be false.
- **The set's content.** `epistemic-records` is SL-006; this slice makes it
  shippable, not correct.
- **README prose.** CHR-001 owns the README rewrite before publish, and two of its
  paragraphs describe the compiled-in behaviour that is about to change. The guide
  (`src/learn.rs`) is in scope only so far as it must not lie.
- **Seed identity across two scopes.** DEC-028 settles it — one id, upsert,
  `previous` reports the replaced body — so there is nothing to invent here.

## Affected surface

| path | what changes |
| --- | --- |
| `src/conventions.rs` | rewritten: root discovery, the file reader, layer resolution, the listing, the seed |
| `src/conventions/` | `hygiene.md` gains flat frontmatter carrying its summary (DEC-028); its version stays injected from the crate; the body below the frontmatter is the prose that prints |
| `src/cli.rs` | `Invocation::Conventions(Ask)` — `resolve` runs in DEC-004's step 1 and now reads the filesystem |
| `src/main.rs` | the dispatch arm: which root the loader is handed, and where a load failure is reported |
| `src/error.rs` | the refusal vocabulary: a set that did not load, a derived frontmatter key, an origin-bearing unknown name |
| `src/learn.rs` | the grammar lines, the exit table's clause for a set that could not be loaded (DEC-033), and the sentence that says a set needs no store — still true, but it must not also imply the sets are only the binary's |
| `tests/conventions.rs` | rewritten against the layered surface: a fake config home, a nested cwd under a repository, layer precedence, shadowing, a broken set at both exits, and no unfalsifiable no-store assertion |
| `docs/store-format.md` | one clause (49-56): the binary no longer "embeds its one set" alone |
| `tests/fixture.rs` | reused temp-dir helpers, if any exist worth sharing |

## Risks, assumptions and open questions

- **R1 — two roots in one process.** The store is `<cwd>/.smut`, never searched
  upwards (DEC-004); conventions are found from the walk-up terminus (DEC-026). So a
  cwd three levels inside a repository sees the project's sets while its store is its
  own, and a cwd outside any repository sees user-and-binary only. That is DEC-026's
  intent, but it means `smut conventions` and `smut declare` can disagree about what
  "the project" is. Record the disagreement and test it; do not smooth it by teaching
  the store to walk up.
- **R2 — step-1 purity.** DEC-004 orders flags, then input, then store. Putting a
  loader in step 1 puts filesystem reads before the store exists, and the exit-code
  taxonomy cannot simply be inherited: a malformed project set is not a usage error
  (the caller typed nothing wrong). It should refuse with the layer and path named,
  and there must be no path where unreadable advice silently prints as no advice.
- **R3 — home is an ancestor of nearly every checkout.** Bounding the walk at the git
  root is load-bearing, not cosmetic. A test must nest a repository under a fake home
  that holds a same-named set.
- **R4 — test hygiene.** If the loader reads `<config-home>` ambiently, every test
  either writes to the developer's own `~/.config` or silently inherits it — a flake
  that a laptop catches and a clean container does not. The roots are parameters of
  the loader; the binary is the only place that reads the environment.
- **A1 — no new dependency.** The frontmatter is two flat keys; it is hand-parsed, as
  the store's own documents are, not given a YAML or TOML parser.
- **A2 — the origin reaches the reader through the listing.** DEC-026's "a set header
  naming its own file" and DEC-028's byte-identity cannot both hold, and DEC-031 keeps
  the bytes: printed prose is the body, never a banner, so a seeded entity and its
  source file diff clean. Shadowing is announced in the listing too.
- **A3 — the environment override is deferred** (QUE-002). DEC-026 names a mechanism and
  no surface, and the loader taking its roots as parameters (DEC-032) removes the last
  case where a test needed one.

### Settled by the design run

- **DEC-031** — the set in force is chosen by layer, never by validity; a set's identity
  is its origin; the body stays the file's bytes; a shadowed set stays visible; a broken
  set is a row. Exit codes are DEC-033's.
- **DEC-032** — the loader is a pure function over its three roots. The project root is
  the nearest ancestor holding a `.git` entry of either kind, else the cwd with no
  walk-up; the user root is `$XDG_CONFIG_HOME` else `$HOME/.config`; an unresolvable user
  root is an absent layer, not an error; only the binary reads the environment.
- **DEC-033** — exit 2 for what the caller typed (an unknown name, an unknown format),
  exit 1 for what the filesystem supplied (unreadable directory, unparseable
  frontmatter, a derived key), and a listing that reports a broken set exits 0.
- **QUE-002** — the environment override's name, what it names, and its rank, left open
  deliberately.

Two consequences the scope did not anticipate: the listing now has **one row per file**
(name, layer, summary-or-verdict, path) rather than one per set, and `smut conventions
<name>` stops printing a file's frontmatter because frontmatter is metadata.

## Verification and closure intent

- Precedence: a project set beats a user set beats the shipped one, same-named sets
  never merge, and the shadowed one is visible rather than absent.
- One reader: the embedded bytes and a file with identical bytes resolve to the same
  entry; the shipped set is not a special case in the code.
- Isolation: an unparseable frontmatter, an unreadable directory, a file that is not
  UTF-8 and a derived frontmatter key each list their layer, path and one-line reason, and
  none of them prevents another layer from answering. A broken *project* set still wins its
  name and shadows the user's (DEC-031), which is the one case where isolation and
  precedence pull in opposite directions and the tested answer is precedence.
- Refusal: an unknown name at exit 2 lists every resolved name with its origin; the
  broken file among them is refused by name at exit 1 with its path and line; a listing
  that reports it exits 0 (DEC-033).
- Seed: fixed `smut_id`, `source` naming the file relative to its layer root, the
  author's `version` for a file set and the crate's for the shipped one, and
  re-declaring still reports the body it replaced under `previous`. The seed's `body` is
  the file's own bytes, which is what the listed prose prints (DEC-031).
- Two roots: a cwd nested inside a repository reads the repository's sets while
  `smut declare` writes a store beside the cwd (DEC-004); the disagreement is tested, not
  smoothed.
- No store: `smut conventions` creates, locks, recovers and validates nothing (VA-1) —
  asserted by pairing it with a store command that fails in the same directory, never by
  a bare absence check that no input could falsify.
- Every assertion names the change that reddens it. This suite is deliberately rewritten,
  so the discrimination discipline is the evidence, not the diff.
- Closure is the gate, not the feature: SL-006 can move to `plan` once a project can
  ship a set and see which layer it read.

## Summary

Turn the compiled-in conventions set into three structurally identical layers — a file
in the project's `.config/smut/conventions/`, one in the user's, and the binary's
embedded set — resolved by one loader, with the listing and the refusals printing
which layer each set came from so a mis-placed or unreadable file is diagnosed rather
than silently lost. DEC-026 and DEC-028 are the mechanism; DEC-031, DEC-032 and DEC-033
are what the design settled, and DEC-033's exit line — what the caller typed is 2, what
the filesystem supplied is 1 — is the part a caller sees. This slice is also the last
gate on the deferred `epistemic-records` set.

## Follow-Ups

- **IMP-008** — the section selector and the `--with-conventions` flag: downstream of
  this slice, and it must not be pinned before the loader has layers.
- **CHR-001** — the README paragraphs describing the compiled-in set.
- **QUE-002** — the environment override's name, what it names and its rank.
- **Reconcile** — three governance amendments the design created and deliberately did not
  make: DEC-028's malformed-file clause (isolation is about listing and loading, not name
  resolution), DEC-026's "set header naming its own file" cost sentence, and DEC-017's
  "a file plus a match arm". Recorded in `notes.md` § Obligations owed at reconcile.
- **SL-006** — unblocked by this slice; `epistemic-records` moves back into reach.
