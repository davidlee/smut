# IMP-013: Conventions are compiled in: a project or a user cannot add, override or trade a set

Today the whole mechanism is a `const`: `SETS: [&Set; 1]` in `src/conventions.rs`,
one embedded `HYGIENE` set, and `seed()` emitting it as a declaration under the
fixed id `conventions-hygiene`. Nothing about it is reachable from outside the
binary. A user cannot add a set, override the shipped one, or hand a set to another
project.

That matters more than it looks, because conventions are the only mechanism smut
has for making a *store* queryable later. The three-store exercise behind RFC-001
measured 0 of 3 independently briefed authors following the one rule that decides
whether `where` works at all (IMP-011: give every entity `type`). Advice compiled
into a binary reaches an author only if they run the command and choose to read it.
Advice that ships *with the project*, in the directory the project already owns,
arrives with the work.

## The shape this is reaching for

Three layers, local wins:

1. a **project directory** — the project's own sets, travelling with the repository
   (recommended shape: `.config/smut/<kind>/` at the project root, mirroring the
   user path — see *Where the project's sets live* below)
2. **`~/.config/smut/<name>/`** — the user's
3. the binary — what ships

so a set can be overridden deliberately, and traded by copying a file.

**Deliberately not inside `.smut/`.** The store is what the guide calls data that
confers no authority; a set is the one artefact whose content an agent is meant to
*follow*. Keeping direction out of the store also keeps a governance contract
intact: STD-001 makes `docs/store-format.md` the sole record of the store's
directory layout and reserved names, reviewed whenever "a file, directory or
reserved name moves, appears or disappears". A `conventions/` subtree under
`.smut/` would therefore cost a format-document review and a reserved-name
decision for what is functionally configuration. A sibling directory costs neither,
and the guide's "whatever is stored there is data" sentence stays true without an
exception clause.

Naming follows from that: whichever word the concept takes, the **directory, the
command and the file form should carry it together**. A mirrored
`.config/smut/<kind>/` satisfies that with the tool as the directory and the
artefact as the leaf, so a second artefact kind adds a subdirectory rather than a
dot-entry.

The invariant worth stating as a design goal: **the shipped set is not special.**
It is a set in the same form, found through the same loader, and it loses to a local
one like any other. "Define our own conventions the way users define theirs" is the
point, not a side effect.

## What already exists

- `smut conventions [NAME]` takes a name and refuses an unknown one by name.
- `conventions NAME --format json` already emits a set **as a declaration** — the
  export half of trading is built, and the seed round-trip (declare under
  `conventions-hygiene`, re-run replaces rather than duplicates) proves the form is
  storable.
- The guide already holds the right posture, and a pin holds it: a set "never
  validates, never selects and never touches a store".

So this is a loader plus a naming rule, not a new concept.

## Questions the design has to answer

1. **Precedence and shadowing.** Local-wins is the obvious rule; the question is
   what a shadowed set does. Silent shadowing plus a listing that shows origin is
   the cheapest honest answer; a refusal ("`hygiene` is shipped and also present in
   this project — say which") is the loudest. Either way the *listing* must show
   which layer each name came from, because a reader has to be able to tell where a
   rule they are following came from.
2. **Identity.** The fixed `conventions-hygiene` id is what makes re-installing an
   update rather than a duplicate. With layers, two same-named sets collide on one
   id; the scheme has to become per-layer, or per-(layer, name), and the seed's
   replacement semantics have to survive it.
3. **Trust and authority.** A set under `.smut/conventions/` is advice a project
   wrote — the same standing as anything else stored there, which the guide already
   calls data and not instruction. But it is the one file whose content an agent is
   expected to *follow*, so the standing should be stated rather than inherited: a
   project set is advice the operator chose to have read, never a schema, never
   authority over the store's contents. Whether a user-scope set under XDG carries
   more weight than a project one — it is the user's own statement — should be
   decided, not left to whoever writes the loader first.
4. **Malformed text.** Sets become files. A malformed one must not break the verb
   for the other layers: refuse by name and origin, keep the rest working. On-disk
   form is open — the embedded set is prose today and `--format json` emits a
   declaration, so the declaration is the obvious candidate, but prose is what a
   human edits.
5. **The name — one decision across three surfaces.** The directory, the command
   and the file form should carry the same word, so this settles more than a verb.
   Candidates so far: keep `conventions`; `read`, `magazine`, `trope`, `recipe`.
   Two are disqualified by collision rather than by taste:
   - **`recipe`** — "the two-call recipe" is the guide's own name for a *query
     procedure*: twelve mentions in `src/learn.rs`, a section heading of its own,
     and the conventions set points at it. Two meanings of "recipe" in one surface
     is the vocabulary collapse `hygiene` items 5 and 7 warn about, committed by the
     tool itself.
   - **`read`** — collides with the store-reading verbs (`get`, `query`, `render`).
   A content noun (`conventions`, or `practice`/`style` if the plural noun among
   actions grates) stays honest; a verb-noun like `magazine`/`trope` does not
   survive a stranger reading `--help`. The cheapest moment to decide is before any
   user has a directory named after it; after that a rename is a migration.

### Where the project's sets live — the mirrored `.config` path

Operator's question: is `$PROJECT_ROOT/.config/smut/conventions/` a silly idea,
when `~/.config/smut/conventions/` is the user's? It is not silly, and it is rare
for a specific reason worth naming rather than guessing at.

**What it buys.** One rule covers both scopes: `<root>/.config/smut/<kind>/`. The
lookup becomes the same relative suffix from two roots, the walk-up terminus is the
project root, and no new dot-entry appears beside `.smut/` as artefacts accumulate
(`conventions/`, later `schemas/`). It also reads as *configuration* rather than as
tool internals, which is the whole data/direction separation — `.smut/` remains the
store, untouched by STD-001.

**Why it is rarely seen.** `.config` in a repository most strongly means "this
directory is a home directory" — dotfiles repositories, container home mounts, the
thing you symlink into `~/.config`. XDG itself defines the config home as
user-scoped, so a project-local `.config` borrows a name whose established meaning
is the opposite of project scope. The nearest precedents for a repo-local `.config`
are a single known path (`.config/dotnet-tools.json`), not a tree of tool homes. So
the objection is not "nobody does this because it is wrong"; it is "the name already
means something else".

**The cost, stated honestly.** A reader meeting `.config/smut/` in a fresh checkout
may assume it is meant to be copied or linked into the home directory — the exact
inverse of the intent. That misreading is cheap to defeat: the listing prints
origin, and the header of a loaded set names the file it came from, so a mistake is
visible the first time anyone runs the command. It is more expensive to defeat after
users have created files under a different name.

**Conditions, if taken.**

- **Anchored at the project root, not the cwd.** One terminus (the git root) and one
  suffix; never also read `<cwd>/.config` part-way down the tree, or the rule
  fragments into per-directory cases.
- **Origin is always printed.** `conventions` with no argument lists sets with the
  layer and file each came from; this is the mitigation, not an extra.
- **Precedence stays total.** Project beats user beats binary, always, with no
  merge between same-named sets; if shadowing is reported, it is reported as
  shadowing.
- **An environment override exists** (`SMUT_CONFIG` or the XDG variable) so a
  monorepo or dotfiles user can point at one config deliberately rather than
  fighting the default.

**The alternatives, for comparison.** `.smut-config/<kind>/` — unambiguous
ownership, one extra dot-dir, no borrowed name. A single `.smut.toml` with a stanza
per set — tidiest of all and the dominant Rust-tool shape, but it trades away
file-copy trading, which is the point of user sets, and it puts prose bodies inside
TOML strings. `.smut/<kind>/` — rejected above, on STD-001 and on the "stored
content is data" sentence.

**Recommendation:** take the mirrored path with the four conditions. The borrowed
name is a documentation problem with a one-line cure; the alternative is a
permanent second dot-dir to save that line.

## If typed schemas arrive

This is the one open question that bears on the name, so it is worth stating what
the vocabulary has to survive: **advice versus enforcement.** smut today is defined
by the absence — DEC-017's "never validated, never auto-selected, never read back" —
and a typed schema is the opposite posture. If both ever exist, the words have to
mark which one bites.

- **`conventions` is the best-hedged name**, because it *is* the word for "agreed
  practice, not enforced" (the whole conventions-over-configuration family).
  "Conventions vs schemas" reads correctly on sight.
- **`recipe`, `trope`, `magazine` are indifferent to the axis** — they neither
  collide nor help, which means `smut recipe` beside `smut schema` would leave a
  reader unable to infer which one can refuse their declaration. That is worse than
  a collision.
- `practice`/`style` hedge almost as well as `conventions`, and no better.

So the naming decision can be made robust to either future without predicting it,
by decoupling three surfaces that currently look like one decision:

| surface | cost to change later | what it should be named for |
| --- | --- | --- |
| the verb | cheap — one arm, docs, a pin | the artefact (a content noun) |
| the file form | middle — sets are already exportable declarations | the artefact |
| the **directory on disk** | expensive — users create files in it | **the tool**, not the artefact |

Concretely: `~/.config/smut/conventions/` for user scope, and one *project* config
directory with per-kind subdirectories — `<dir>/conventions/` now, `<dir>/schemas/`
if that day comes. One dotfile decision, no migration when a second artefact
arrives, and `.smut` stays the store (STD-001 untouched) while `.smut-config` (or
whatever it is called) stays configuration. The uglier alternative — a sibling per
artefact (`.smut-conventions/`, later `.smut-schemas/`) — also survives; it just
multiplies directories.

The corresponding mechanical rule: **the loader is generic over declaration kind.**
Sets are already printed as declarations, so a schema artefact could load through
the same ladder with a different kind. Costs nothing now; it is the whole hedge.

### Why user-defined conventions are worth building even if schemas are coming

They are the cheapest schema-*discovery* mechanism available. Real user sets show
which constraints people actually write, on real domains, before any enforcement
model is committed to — evidence a schema designed up front does not have. That is
the strongest claim this item can make on its own behalf: "conventions now" is not
a detour from "schemas later", it is the research for it.

### What must not happen

Conventions must not quietly acquire teeth. If a schema artefact arrives it is a
*second* thing with its own verb, its own precedence (probably highest, since it
enforces) and its own validation; the sets keep never validating. The moment a
conventions file can refuse a declaration, the store's oldest sentence — "whatever
is stored there is data" — becomes false.

**Revisit trigger:** reopen this if entity validation is ever proposed, or if a year
of user sets shows the advice is not being adopted.

## Two discovery questions the layering opens

**How far up to look.** The store's rule is pinned and contrary: "The store is
`.smut/` beside the current directory, and is never searched for in a parent"
(`src/learn.rs:32`, `docs/store-format.md:30`). Project-scoped sets want the
opposite — walk up to the repository root, so one set governs every subdirectory and
every agent working under it. The asymmetry is defensible (a notebook is local; a
policy is project-wide) but it has to be stated, because an agent then holds two
discovery rules for one tool. The shape worth considering: **nearest ancestor wins,
bounded by the git root**, so a subproject can override the repository rather than
accumulating rules it cannot escape — with a listing that shows origin then being
mandatory. Two things to weigh while choosing: a parent-level set governs far more
agents than a per-cwd file (the trust question above, with a wider blast radius),
and a fallback is needed for when there is no `.git` at all.

**Where the advice is installed, versus where it is recorded.** Today the pipe into
`declare` is the whole mechanism (DEC-017) and the store owns a copy. With project
files as the *source* of direction, a seeded entity stops being an installation
mechanism and becomes a record — "the advice this notebook was given" — which is a
different artefact and arguably worth keeping for exactly that reason. Decide it
deliberately rather than let the two idioms drift into one.

## Relation to what is already decided

DEC-017 (accepted) settled the 0.2 mechanism in exactly the opposite direction: one
embedded set, printed; `--format json` emits one entity with a fixed id so a
re-install upserts; "adding a later set is a file plus a match arm"; and "the binary
never reads it back". Layered loading moves three of those consequences while
leaving the principles intact:

| DEC-017 consequence | under a loader |
| --- | --- |
| a new set costs a file plus a match arm | a new set costs a file; the match arm is the thing being designed away |
| fixed id, one per set | must become per-layer or per-(layer, name), and the upsert semantics must survive it |
| the binary never reads a set back | still true of the *store's* copy — a seeded entity stays inert. It becomes false only for **source files** under the conventions path, which are a different artefact. Keep the distinction explicit or the two will be conflated |

What DEC-017 does *not* move: printed, never validated, never auto-selected, never
read back from the store, the store may amend its own copy but the seeded body is
replaced on re-install. Those are the properties that make a set advice rather than
a schema, and they are the reason this item can be permissive too.

So this is not a contradiction of DEC-017 so much as a later mechanism decision
that needs to record itself: expect a new DEC (or an amendment naming the
consequences above) when the loader is designed.

## Neighbours

- **IMP-004** (the `epistemic-records` second set) may be absorbed by this: if a
  project can ship a domain set, that set becomes the shipped *example* rather than
  a special case. This item decides which; IMP-004 should not be built first.
- **IMP-008** (delivery through the `learn` surface) is downstream: with layers, a
  footer or flag should point at a *resolved* set rather than a hardcoded name, so
  the loader's shape decides the flag's shape.
- **IMP-011** (the discriminator) is the finding this mechanism exists to cure at
  organisation scale: a project that ships its own conventions can require `type` of
  its own agents in a way a binary never can.

## Status

`design-deferred`. Nothing here blocks Wave 1. But it sits *before* the point where
IMP-008's `--with-conventions` grammar is pinned, because that flag's whole meaning
depends on whether sets can be layered and resolved.

## Provenance

Operator inclination of 2026-11-02 — three messages: sets should be tradeable and
overridable ("trivially easy"); the project's sets should live in a sibling
directory rather than inside `.smut/`, project-scoped by walking to the nearest git
root; and `recipe` as a possible verb. Recorded in RFC-001 § Outcome. The
question that governs the naming hedge — whether the future holds user-definable
typed schemas for entities — is the operator's ("I have no real idea about"), and is
the reason § If typed schemas arrive exists rather than a decision. The reading of
`src/conventions.rs`, the DEC-017 comparison, the STD-001 argument, the `recipe`
collision count and the pins are this repo's.
