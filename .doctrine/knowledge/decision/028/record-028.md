# A conventions set is one markdown file

## The file

    <root>/.config/smut/conventions/hygiene.md

The **filename is the name**. The first level-1 heading is the **title**, falling
back to the filename when there is none. Everything below is the **body**. Optional
frontmatter carries metadata and nothing else.

This is the form `src/conventions/hygiene.md` already has, embedded with
`include_str!`. The loader for layers 1 and 2 is therefore the same path the binary
already takes for layer 3, and DEC-026's "the shipped set is not special" becomes a
property of the code rather than an aspiration.

## Frontmatter

Delimited by `---`, flat `key: value`, no nesting, no lists, no multi-line values.

| class | keys | behaviour |
| --- | --- | --- |
| known | `summary`, `version` | used |
| derived | `title`, `name`, `id`, `type` | **refused by name**, with a message saying where each fact comes from |
| unknown | anything else | **ignored**, deliberately |
| unparseable | a line that is not `key: value` | refused |

Frontmatter is never printed as prose, never addressable, and never seeded into a
body.

**Why unknown keys are ignored.** A set authored for a later smut must still load in
an earlier one, or trading breaks — which is the whole point of layers. Refusing
unknown keys would be a schema in all but name. Recorded consequence: a key can
never quietly *become* load-bearing later, because today's files already carry them.

**Why derived keys are refused rather than ignored.** Silently ignoring `title:`
would print a document whose title contradicts what its author declared. That is the
silent-wrongness class the usability work ranked first, and the list is four items,
so prevention is cheap and the message can teach the rule:
*the title is the file's first `#` heading; the name is the filename.*

## Sections

A pinned grammar, implemented later:

- **normalised match** — case-insensitive, spaces, dashes and underscores
  interchangeable;
- **ambiguity refuses**, naming both headings — never first-match;
- **`--format json` with a section is a usage error**: a section is not a
  declaration, the same reasoning that makes `json` without a name a usage error
  today;
- a **level-1 heading is the whole set**, so `conventions hygiene "General hygiene"`
  prints what `conventions hygiene` prints;
- output runs to the next heading of the **same or higher level**;
- an **unknown section refuses and lists the sections that exist**, as the
  unknown-set message lists sets.

`learn` shares the grammar but not the function: its sections are its own layout
(unindented capitalised lines, with unindented prose in the intro), a set's are
markdown headings. Share what a user types; extract per document.

## Seed identity

The seed keeps **`id: conventions-<name>`** and gains **`source`**, the file path
relative to its layer root. `version` is the author's when the set is a file and the
crate's when it is shipped — which is what makes the field meaningful for both and
answers "is this record stale?".

No layer-qualified id. DEC-017 already makes replacement visible through `previous`,
so the store record is *the advice in force* and the file is the durable source; a
qualified id would accumulate one record per layer and quietly break the upsert
promise.

## Malformed

Never breaks the other layers. A file that cannot be loaded lists with its path and a
one-line reason, and refuses by name when asked for. Four reasons, exhaustively:

1. unreadable;
2. not UTF-8;
3. an empty body;
4. malformed frontmatter.

Shadowing is not malformed — DEC-026 made that total precedence, reported as
shadowing. An empty file is malformed rather than an empty set: a set that says
nothing is a mistake, not advice.

## The walk-up terminus

The git root when there is one; otherwise the current directory, with no walk-up.
The bound is not cosmetic: `$HOME` is an ancestor of nearly every checkout, so an
unbounded walk would adopt `$HOME/.config/smut` as a *project* layer and double-count
the user layer as project policy. A repository supplies a boundary; a bare directory
does not, so project scope exists only where you are standing.

## What this costs the code

`Ask` can no longer carry `&'static Set`; the resolved set becomes owned, `resolve`
gains file I/O, and `answer` stays pure — the split survives, the lifetime does not.
The unknown-name refusal stops being a static list and becomes the main diagnostic
for a mis-placed file, so it must print origins (DEC-026 already requires that in the
listing).
