# smut

![smut - cli graph db](https://github.com/davidlee/smut/raw/main/smut.jpg)

`smut` is the flagrantly permissive graph database CLI for agents.

It's schemaless, unceremonious, and apt to make a mess. You choose 
whether to clean it up, hide it, or just pretend it was someone else's.

There's no server. `smut` is private, though it can be shared.

`smut` is so easy to use, it's hard to be sure that's a feature.

## What it is

`smut` is one binary and one directory. You declare entities as JSON objects
with an `id`, link them by embedding an object that has an `id`, and read them
back as JSON or as Markdown. Everything lives in `.smut/` beside the directory
you run it in; there is no server, no daemon, no ancestor search, and no
configuration file.

Recording a linked thought should cost about as much as writing it down:

```
smut declare '{"id":"smut","title":"Smut","questions":[{"id":"q-query","title":"Query design"}]}'
```

That is two entities and one reference, stored durably as one batch. An agent
mid-conversation can keep its working structure in a store and render it to
Markdown whenever the structure is the point rather than the content.

This is **smut 0.2.0**. The query surface is nine keys: `from`, `where`,
`follow`, `backlinks`, `depth`, `select`, `order`, `limit` and `budget`. 0.1 had
four of them — `from`, `select`, `order` and `limit` — and 0.2 adds the other
five: `where`, which filters the roots with a small predicate language, and the
traversal keys `follow`, `backlinks`, `depth` and `budget`, which walk the graph
from those roots, bounded by hops and by expansions, and report what the walk
found. The envelope stays additive: five keys when nothing was traversed and a
sixth, `traversal`, when something was, so a 0.1 query reads exactly as it did.
Mermaid arrives in 1.0 only: where it would be, `smut` refuses by name instead
of guessing. The commands below are the whole surface.

## Build

```sh
cargo build --release
./target/release/smut --version   # smut 0.2.0
```

Or install it from crates.io: `cargo install smut` — which installs **0.1.1**,
the release that is published there. This tree is 0.2.0 and is not published, so
building it is the only way to get the surface below; the 0.1.1 on crates.io has
none of the five keys 0.2 adds.

The binary is self-contained: `target/release/smut` is the whole program, and
`smut learn` prints the embedded guide with nothing else installed.

## Worked example

A scratch directory, a store, and the brief's end-to-end example. This is the
transcript of running the release build, command for command: a nested
declaration, the graph walked and rendered, a filtered read, an amend that keeps
the fields it does not name, the Mermaid step 0.2 refuses by name, a `DROP`, a
delete, and a walk that follows the reference the delete left behind. The last
step reports that reference as dangling rather than recreating it.

```console
$ smut declare '[{"id":"smut","title":"Smut","body":"Persist conversational structure.","questions":[{"id":"q-query","type":"question","status":"open","title":"Query design","body":"Keep selection small."}]}]'
{"changes":[{"id":"smut","previous":null,"status":"created"},{"id":"q-query","previous":null,"status":"created"}],"ids":["smut"],"missing":[]}
$ smut get smut q-query
{"dangling":[],"missing":[],"objects":[{"id":"smut","body":"Persist conversational structure.","questions":[{"id":"q-query"}],"title":"Smut"},{"id":"q-query","body":"Keep selection small.","status":"open","title":"Query design","type":"question"}],"truncated":false,"truncation_reasons":[]}
$ smut query '{"from":"smut","follow":["questions"],"select":["title","body"]}'
{"dangling":[],"missing":[],"objects":[{"id":"smut","body":"Persist conversational structure.","title":"Smut"},{"id":"q-query","body":"Keep selection small.","title":"Query design"}],"traversal":{"depths":{"q-query":1,"smut":0},"edges":[{"field":"questions","source":"smut","target":"q-query"}],"frontier":["q-query"]},"truncated":false,"truncation_reasons":[]}
$ smut render '{"from":"smut","follow":["questions"],"select":["title","body"]}'
## Smut

Persist conversational structure.

## Query design

Keep selection small.
$ smut declare '{"id":"q-query","status":"resolved","body":"Use bounded traversal and simple predicates."}'
{"changes":[{"id":"q-query","previous":{"id":"q-query","body":"Keep selection small.","status":"open","title":"Query design","type":"question"},"status":"updated"}],"ids":["q-query"],"missing":[]}
$ smut query '{"where":{"type":"question","status":"resolved"}}'
{"dangling":[],"missing":[],"objects":[{"id":"q-query","body":"Use bounded traversal and simple predicates.","status":"resolved","title":"Query design","type":"question"}],"truncated":false,"truncation_reasons":[]}
$ smut render '{"from":"smut","follow":["questions"]}' --format mermaid
smut: --format mermaid is not supported in this release; planned for 1.0
$ smut declare '{"id":"q-query","status":"DROP"}'
{"changes":[{"id":"q-query","previous":{"id":"q-query","body":"Use bounded traversal and simple predicates.","status":"resolved","title":"Query design","type":"question"},"status":"updated"}],"ids":["q-query"],"missing":[]}
$ smut delete q-query
{"deleted":["q-query"],"missing":[]}
$ smut query '{"from":"smut","follow":["questions"]}'
{"dangling":[{"field":"questions","source":"smut","target":"q-query"}],"missing":[],"objects":[{"id":"smut","body":"Persist conversational structure.","questions":[{"id":"q-query"}],"title":"Smut"}],"traversal":{"depths":{"smut":0},"edges":[],"frontier":[]},"truncated":false,"truncation_reasons":[]}
```

What the transcript shows, in order: one declaration that produced two entities
and made the nested declaration an id-only reference; a read of both; a
traversal query that follows `questions` to a depth-1 discovery and reports the
depth, the edge and the frontier behind it; the same selection rendered as
Markdown; an amend that preserved the fields it did not name; a filter that
selects on the amended status; the Mermaid request, refused by name because that
format is 1.0's; a `DROP` that removed exactly one field; a delete, after which
`smut`'s reference to the deleted entity is left dangling rather than cascaded;
and finally the same walk over the reference the delete left behind, reported as
dangling and left alone.

## Commands

| command | what it does |
| --- | --- |
| `smut declare [JSON] [--compact]` | record or amend entities; the only command that creates a store |
| `smut get <ID>...` | fetch entities by id, in the order given, with no limit |
| `smut delete <ID>...` | remove entities in one batch; incoming references are left untouched |
| `smut query [JSON]` | select entities with a query specification |
| `smut render [JSON]` | the same selection, printed as Markdown |
| `smut conventions [NAME] [--format json]` | list the built-in conventions sets, print one, or print it as a declaration |
| `smut learn` | print the embedded guide |

`declare`, `query` and `render` take one JSON argument or one complete JSON value
on stdin, when the argument is absent or is `-`. With no argument and a terminal
attached you get usage instead of a silent wait. Trailing input after the JSON
value is refused, and so is a duplicate key inside one object.

The specification keys are `from`, `where`, `select`, `order`, `limit`,
`follow`, `backlinks`, `depth` and `budget`; absent `from` means every entity in
id order, absent `select` means every field, and no `limit` still caps a result
at 100 with `"truncated": true` and a reason. `where` is a predicate over
one entity's top-level fields — `$eq`, `$ne`, `$in`, `$exists` and `$contains`,
ANDed by default, with `$and`, `$or` and `$not` to compose — and it filters the
named roots only, before any traversal. `follow` and `backlinks` take `"*"` or
an array of top-level field names, `depth` defaults to 1 (0 is roots only),
`budget` defaults to 10000 entity expansions and is inert without a traversal
key, and a supplied traversal key adds a `traversal` member carrying `depths`,
`edges` and `frontier`. `get` is selection with the ids you name, no limit and
no truncation.

`render` flags: `--format markdown` (the only format this release writes;
`mermaid` is refused by name, because it is 1.0's), `--heading-field`
(default `title`), `--heading-level 1-6` (default 2), `--show-fields`,
`--show-ids`.

Exit codes: `0` success, including reported missing ids and a truncated result;
`1` no store, or a store that could not be read or written; `2` invalid input or
usage. JSON goes to stdout, diagnostics go to stderr, and a failure never prints
a success envelope.

## Conventions

`smut conventions` prints advice, not a schema. It needs no store, it reads no
store, it validates nothing and it selects nothing for you: it is one shipped set
of prose about how entities and references are shaped, versioned with the build
that prints it. With no argument it lists the sets; with a name it prints that
set; with `--format json` it prints the set as one declaration, which you can
pipe straight into `smut declare -` —
`smut conventions hygiene --format json | smut declare -` creates a single entity
with a fixed id (`conventions-hygiene`) that a later install updates in place
rather than duplicating. The seeded `body` is advice and is replaced on
re-install; fields you add to that entity survive.

## Traversal: what it costs and what it reports

**What `budget` does not bound.** `budget` counts entity expansions — how many
entities the walk is allowed to open. It is not a bound on total work. Every hop
that looks backwards scans the store for incoming references, and a query with no
`from` or `where` scans the whole store before it has any roots at all. A small
`budget` beside a `backlinks` request therefore does not mean a small amount of
work, and `smut` makes no total-work claim.

**What a traversal reports, and what it does not.** A query that supplies
`follow` or `backlinks` — even as an empty array — carries a `traversal` member
beside the 0.1 envelope:

- `depths` maps each **returned** id to its minimum discovery depth, roots at 0.
- `edges` is the induced subgraph of the returned entities: `{source, field,
  target}` triples, deduplicated, in result order, oriented as stored. An edge
  appears when both endpoints are returned, even if neither was expanded, even
  if the field was never named as a traversal label, and even if `select` hid
  it. References to ids not in the store are not edges.
- `frontier` is what discovery reached and did not expand, in discovery order. It
  is walk-scoped: `limit` does not cut it, though a depth or budget boundary
  does.

The envelope also says why a result is incomplete: `truncated` with
`truncation_reasons`, which is `result_limit` when ordered rows were cut and
`traversal_budget` when the walk declined a permitted expansion. A depth boundary
is selection, not truncation, so a finite `depth` never sets either. `dangling`
reports the traversal's references to ids that are not in the store — one triple
per reference, deduplicated, in encounter order, and not cut to the returned
sources — and nothing is ever recreated to fill the gap. A non-traversing query
carries exactly 0.1's five keys, so nothing above appears where nothing was
walked.

## Sharp edges

- **An embedded object carrying `id` is a reference, not metadata.** Any nested
  object with an `id` is hoisted into an entity of its own and replaced by
  `{"id": "..."}` in the container, so there is no way to store inline metadata
  under a field named `id`.
- **`"DROP"` is a top-level field sentinel and cannot be assigned as a value.**
  `{"id":"x","status":"DROP"}` removes `status`; inside an embedded value or an
  array, the string `"DROP"` is ordinary data. `null` is an ordinary stored
  value everywhere.
- **A missing store is exit 1, not an empty store.** `get`, `delete` and `query`
  never create one, and a declaration with nothing to write
  (`smut declare '{}'`) does not either.
- **`1` and `1.0` are distinct for equality and equal for ordering.** In one
  batch they contradict each other and the batch is refused; in an `order` they
  tie, and the tie breaks by id.
- **`where` filters roots only, and `$` is reserved inside a predicate.** A
traversal may still return an entity that fails the predicate, and a root the
predicate rejects is omitted rather than reported missing. A field whose name
begins with `$` cannot be filtered, and an object carrying `$` keys is an
operator object unless you compare it whole with `$eq`. A missing field fails
every operator except `$exists`, which counts a stored `null` as present.
- **An unrecognised key or operator is reported as unknown**, without a promise:
  the 0.2 query surface is complete, so a typo is a typo rather than a key due in
  a later release. `--format` is the only surface still promising one (Mermaid,
  in 1.0).
- **`--` reaches an opaque id.** Ids are opaque, case-sensitive strings and may
  begin with a hyphen, so `smut get -- -weird-id` names that id. Without `--` it
  is a usage error.
- **Bodies may absorb each other, but never the completeness notice.** A body is
  rendered verbatim, so a stored body ending in an unclosed HTML comment or an
  unclosed fence hides whatever follows it. The notice for a truncated or
  incomplete result is the document's first block, so nothing stored can swallow
  it.

## What smut promises

The store is written through a journal. Its bytes, its states and its fsync
obligations are described in
[the store format document](https://github.com/davidlee/smut/blob/main/docs/store-format.md), and
this is the guarantee that protocol licenses, quoted in full:

> Under the documented filesystem and fsync assumptions, cooperating commands
> observe whole batches. Before application begins, the commit decision is made
> durable, together with every staged byte that decision commits to.
> Interrupted committed batches are replayed before any command serves store
> state. Failure before acknowledgement may leave the outcome uncertain;
> successfully acknowledged batches are durable.

That is **atomic visibility through locking and recovery**, on top of non-atomic
physical publication. It is deliberately not claimed that a group of renames is
atomic on any real filesystem, and no claim is made about what a power loss
leaves on the disk beyond the documented fsync assumptions.

The uncertainty the guarantee names is commit-before-acknowledgement. A command
can die after its batch was committed and before its success response reached
you, so an interrupted command's effect is *unknown* rather than undone. An
acknowledged batch — exit 0, response printed — is committed and durable.

## Portability

Linux is the acceptance platform. The store is plain files and directories and
nothing in it is Linux-specific by intent, but nothing else is tested.

The store lock is `flock`, which is advisory: it excludes cooperating `smut`
commands and nothing else. A process that does not take the lock — an editor, a
shell redirect, a stray script — is not excluded, and nothing is promised to it.
Editing `.smut/` by hand is not supported.

## Learn

`smut learn` prints the embedded guide: the conventions, the response shapes, the
workflows, and the rules in the order you hit them. It needs no store, and it is
version-matched to the build that prints it.

## License

`smut` is free software under the GNU General Public License, version 3
(GPL-3.0-only): you may redistribute it and/or modify it under those terms, and
it comes with no warranty. See
[the licence text](https://github.com/davidlee/smut/blob/main/LICENSE) for the
full text.
