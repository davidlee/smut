# IMP-013: Conventions are compiled in: a project or a user cannot add, override or trade a set

**Design settled — this is now an implementation brief.** The mechanism is DEC-026
(where sets live, how they resolve) and DEC-028 (what a set file is and what the
loader does with it). One call remains before code: the name.

## The problem, unchanged

`src/conventions.rs` holds one embedded `Set`, `include_str!`ed from
`src/conventions/hygiene.md`, and `seed()` emits it under a fixed id. Nothing is
reachable from outside the binary: a user cannot add a set, override the shipped one,
or hand a set to another project.

That matters because conventions are the only mechanism smut has for making a store
queryable later. Three independently briefed authors wrote three 34-entity stores and
**0 of 3** followed the one rule that decides whether `where` works (IMP-011). Advice
compiled into a binary reaches an author only if they run the command and read it;
advice that ships with the project arrives with the work.

## What is decided

**Where sets live (DEC-026).** Three structurally identical layers, one relative
suffix, local wins:

    <project-root>/.config/smut/conventions/<name>.md
    ~/.config/smut/conventions/<name>.md
    the binary

Project root is the walk-up terminus — the git root, else the cwd with no walk-up.
Precedence is total, same-named sets do not merge, and the listing prints every set's
layer, path and (per DEC-028) whether it loaded.

**What a set is (DEC-028).** One markdown file, filename = name, first `#` heading =
title, flat two-key frontmatter (`summary`, `version`), derived keys refused, unknown
keys ignored, unparseable frontmatter refused, malformed files isolated to
themselves, seeds keyed `conventions-<name>` with a `source` field, section
addressing grammar pinned and implemented later.

## The work

1. **A loader over three roots**, resolving a name to (file, layer, body, metadata)
   with DEC-026's precedence. Nothing lands inside `.smut/`: the store's layout stays
   owned by `docs/store-format.md` under STD-001.
2. **The same code path for the shipped set.** It is already markdown; the layer-3
   reader should look like the layer-1 reader, not like a special case.
3. **The listing becomes the diagnostic**: name, summary, layer, path, and a
   one-line reason when a file did not load. This replaces a static list and is where
   a mis-placed file is discovered.
4. **The unknown-name refusal lists resolved names and origins** — the same
   information, refusals included.
5. **Seeds carry `source`** and an author-owned `version` for file sets; the fixed id
   and DEC-017's upsert-and-report-`previous` semantics stay.
6. **`Ask` becomes owned.** It holds `&'static Set` today; `resolve` gains file I/O
   and `answer` stays pure. The type split survives, the lifetime does not.
7. **Loader generic over declaration kind**, so a future artefact kind reuses the
   ladder rather than re-deriving it (see § If typed schemas arrive).

## What is still open

- **The name.** Recommendation: **keep `conventions`** — it is the word that marks
  advice-not-enforcement, which is the axis a schema future tests, and it costs zero
  churn (`learn`'s section and its pin, this file's siblings, the
  `conventions-<name>` id prefix, `type: "convention"`, the leaf directory). `recipe`
  is disqualified by collision with the guide's "two-call recipe" (twelve mentions and
  a section heading); `practice` reads as a verb phrase; `advice` hedges equally well
  and is a rename for no gain. **Not yet recorded as a decision — one word from the
  operator closes it.** Reopen only if schemas arrive and the vocabulary needs a
  finer distinction.
- **The `--with-conventions` flag** (IMP-008) is downstream and must not be pinned
  before this lands: a flag naming a set means something different once resolution
  has layers.

## Neighbours

**IMP-011** is the finding this mechanism exists to cure at organisation scale: a
project that ships its own conventions can require `type` of its own agents in a way
a binary never can. **IMP-008** consumes the section grammar this decision pins, and
owns the flag. **IMP-004** (the deferred `epistemic-records` set) may become the
shipped *example* of a domain set rather than a special case — this item decides
which, and IMP-004 must not be built first.

## If typed schemas arrive

Today's posture is the opposite (DEC-017: never validated, never read back), so a
schema would be a second enforcement-bearing artefact, not an extension. The naming
recommendation above assumes the vocabulary has to keep marking that difference; the
loader's genericity over declaration kind is the hedge, and it costs nothing now.
Conventions must never acquire teeth — a set that can refuse a declaration makes the
store's oldest sentence, "whatever is stored there is data", false.

## Provenance

Operator inclinations of 2026-11-02 (tradeable sets; a project directory outside
`.smut/`; the mirrored config home; `recipe` floated then withdrawn), settled as
DEC-026 and DEC-028. The measurements, the STD-001 argument, the `recipe` collision
and the code reading are this repo's. Usability evidence: RFC-001.
