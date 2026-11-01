# smut

![smut - cli graph db](./smut.jpg)

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

This release is **0.1**: writing, reading back, and rendering. Predicates and
traversal arrive in 0.2, and Mermaid in 1.0; where those would be, `smut` refuses
by name instead of guessing. The commands below are the whole surface.

## Build

```sh
cargo build --release
./target/release/smut --version   # smut 0.1.0
```

The binary is self-contained: `target/release/smut` is the whole program, and
`smut learn` prints the embedded guide with nothing else installed.

## Worked example

A scratch directory, a store, and the round trip. This is the transcript of
running the release build; the failure at the end is the boundary of 0.1, shown
on purpose.

```console
$ smut declare '[{"id":"smut","title":"Smut","body":"Persist conversational structure.","questions":[{"id":"q-query","type":"question","status":"open","title":"Query design","body":"Keep selection small."}]}]'
{"changes":[{"id":"smut","previous":null,"status":"created"},{"id":"q-query","previous":null,"status":"created"}],"ids":["smut"],"missing":[]}
$ smut get smut q-query
{"dangling":[],"missing":[],"objects":[{"id":"smut","body":"Persist conversational structure.","questions":[{"id":"q-query"}],"title":"Smut"},{"id":"q-query","body":"Keep selection small.","status":"open","title":"Query design","type":"question"}],"truncated":false,"truncation_reasons":[]}
$ smut render '{"from":"smut","select":["title","body"]}'
## Smut

Persist conversational structure.
$ smut declare '{"id":"q-query","status":"resolved","body":"Use bounded traversal and simple predicates."}'
{"changes":[{"id":"q-query","previous":{"id":"q-query","body":"Keep selection small.","status":"open","title":"Query design","type":"question"},"status":"updated"}],"ids":["q-query"],"missing":[]}
$ smut declare '{"id":"q-query","status":"DROP"}'
{"changes":[{"id":"q-query","previous":{"id":"q-query","body":"Use bounded traversal and simple predicates.","status":"resolved","title":"Query design","type":"question"},"status":"updated"}],"ids":["q-query"],"missing":[]}
$ smut get q-query
{"dangling":[],"missing":[],"objects":[{"id":"q-query","body":"Use bounded traversal and simple predicates.","title":"Query design","type":"question"}],"truncated":false,"truncation_reasons":[]}
$ smut delete q-query
{"deleted":["q-query"],"missing":[]}
$ smut query '{"from":"smut","follow":["questions"]}'
smut: follow is not supported in 0.1; it arrives in 0.2
```

What the transcript shows, in order: one declaration that produced two entities
and made the nested declaration an id-only reference; a read of both; a Markdown
render of the selected fields; an amend that preserved the fields it did not
name; a `DROP` that removed exactly one field; a delete, after which `smut`'s
reference to the deleted entity is left dangling rather than cascaded; and
finally the traversal query this release cannot answer.

## Commands

| command | what it does |
| --- | --- |
| `smut declare [JSON] [--compact]` | record or amend entities; the only command that creates a store |
| `smut get <ID>...` | fetch entities by id, in the order given, with no limit |
| `smut delete <ID>...` | remove entities in one batch; incoming references are left untouched |
| `smut query [JSON]` | select entities with a query specification |
| `smut render [JSON]` | the same selection, printed as Markdown |
| `smut learn` | print the embedded guide |

`declare`, `query` and `render` take one JSON argument or one complete JSON value
on stdin, when the argument is absent or is `-`. With no argument and a terminal
attached you get usage instead of a silent wait. Trailing input after the JSON
value is refused, and so is a duplicate key inside one object.

The specification keys are `from`, `select`, `order` and `limit`; absent `from`
means every entity in id order, absent `select` means every field, and no
`limit` still caps a result at 100 with `"truncated": true` and a reason.
`get` is selection with the ids you name, no limit and no truncation.

`render` flags: `--format markdown` (the only format in 0.1), `--heading-field`
(default `title`), `--heading-level 1-6` (default 2), `--show-fields`,
`--show-ids`.

Exit codes: `0` success, including reported missing ids and a truncated result;
`1` no store, or a store that could not be read or written; `2` invalid input or
usage. JSON goes to stdout, diagnostics go to stderr, and a failure never prints
a success envelope.

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
- **`where` and traversal arrive in 0.2.** `where`, `follow`, `backlinks`,
  `depth` and `budget` are refused by name, with the release that will accept
  them. An unrecognised key is reported as unknown, without a promise.
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
obligations are described in [docs/store-format.md](docs/store-format.md), and
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
