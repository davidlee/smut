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
  `docs/store-format.md` under STD-001, which already disclaims conventions; that
  document needs no change.
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
| `src/conventions/` | `hygiene.md` unchanged as text; its header is now metadata the reader parses |
| `src/cli.rs` | `Invocation::Conventions(Ask)` — `resolve` runs in DEC-004's step 1 and now reads the filesystem |
| `src/main.rs` | the dispatch arm: which root the loader is handed, and where a load failure is reported |
| `src/error.rs` | the refusal vocabulary: a set that did not load, a derived frontmatter key, an origin-bearing unknown name |
| `src/learn.rs` | the grammar lines and the sentence that says a set needs no store — still true, but it must not also imply the sets are only the binary's |
| `tests/conventions.rs` | the argument order and the scratch directories the loader needs |
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
- **OQ-1 — the environment override.** DEC-026's choice includes one ("an environment
  override points at one config deliberately") and names no surface for it. Naming a
  variable is a compatibility commitment, so the default is to defer it to a recorded
  follow-up unless it falls out of the root plumbing for free.
- **OQ-2 — how the origin reaches the prose reader.** DEC-026's cure for
  `.config`-in-a-repository is "origin printing plus a set header naming its own
  file". Whether the printed prose carries an injected header line (bytes differ from
  the file) or the origin appears in the listing and on shadowing only (bytes stay the
  file) is undecided and must be — printing a set cannot silently differ from the
  file an author edits.
- **OQ-3 — where shadowing is announced.** "Reported as shadowing" is decided;
  the channel is not. Listing row, prose footer, or stderr. Whatever is chosen must
  not make prose output non-deterministic for a caller piping it into `declare`.

## Verification and closure intent

- Precedence: a project set beats a user set beats the shipped one, same-named sets
  never merge, and the shadowed one is visible rather than absent.
- One reader: the embedded bytes and a file with identical bytes resolve to the same
  `Set`; the shipped set is not a special case in the code.
- Isolation: an unparseable frontmatter, a missing file, an unreadable directory and
  a derived key each list their layer, path and one-line reason, and none of them
  prevents another layer from answering.
- Refusal: an unknown name lists every resolved name with its origin, and the
  malformed file among them.
- Seed: fixed `smut_id`, `source` relative to the layer root, author-declared
  `version` for a file set, crate version for the shipped one, and re-declaring still
  reports the body it replaced under `previous`.
- No store: `smut conventions` creates, locks, recovers and validates nothing
  (VA-1), which now needs proving with a file-backed loader rather than by
  construction.
- Closure is the gate, not the feature: SL-006 can move to `plan` once a project can
  ship a set and see which layer it read.

## Summary

Turn the compiled-in conventions set into three structurally identical layers — a file
in the project's `.config/smut/conventions/`, one in the user's, and the binary's
embedded set — resolved by one loader, with the listing and the refusals printing
which layer each set came from so a mis-placed or unreadable file is diagnosed rather
than silently lost. The design is DEC-026 and DEC-028; this slice is their
implementation and the last gate on the deferred `epistemic-records` set.

## Follow-Ups

- **IMP-008** — the section selector and the `--with-conventions` flag: downstream of
  this slice, and it must not be pinned before the loader has layers.
- **CHR-001** — the README paragraphs describing the compiled-in set.
- **OQ-1** — the environment override, if deferred here.
- **SL-006** — unblocked by this slice; `epistemic-records` moves back into reach.
