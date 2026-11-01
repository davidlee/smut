# IMP-008: `smut learn` has no section selector, so re-reading one rule costs the whole guide

Trial friction 3, and the only friction the reporter said would change behaviour
rather than prose:

> Reading it once was right. Re-reading one rule costs the whole thing, which
> means in practice it doesn't get re-read — the agent guesses and moves on.

Measured on 0.2.0:

```
❯ smut learn | wc -c
13469
❯ smut learn | wc -l
217
❯ smut learn traversal
error: unexpected argument 'traversal' found
(exit 2)
```

The guide is the store's only teaching surface and it is all-or-nothing. Every
read either pays 13.5 KB or does not happen.

## The pieces are already there

The text carries thirteen headings — `Commands`, `Exit codes`, `Declaring`,
`References`, `Batch responses`, `Querying`, `Traversal`, `The two-call recipe`,
`Labels and rendering`, `Conventions`, `Rendering`, `Opaque ids`, `Workflows`.
This is not new material; it is making the existing names addressable.

`smut conventions [NAME]` is the precedent: an embedded printed resource with an
optional name, where the unnamed call prints the whole set. `smut learn
[SECTION]` mirrors it — no argument prints the guide byte-identically, a name
prints that section.

## The second half of the same complaint: which section would this be?

Friction 4 from the same trial is a sentence that is true and still cost two
readings. At depth 1 `objects` and `frontier` overlap, because a depth boundary
is not a truncation and the walk's unexpanded discoveries are the deepest
returned objects:

```
❯ smut query '{"from":["s1"],"follow":["asks"],"depth":1,"select":["id"]}'
{"objects":[{"id":"s1"},{"id":"q1"},{"id":"q2"}], …
 "traversal":{"frontier":["q1","q2"], …}}
```

The guide's sentence (`src/learn.rs:162`) reads "frontier and dangling describe
the walk, so frontier may name ids that objects does not carry". That stresses
the budget case — the rare one — and leaves the reader to derive the depth-boundary
case, which is the common one. Swap the emphasis: frontier is what the walk
discovered and did not expand; at a depth boundary that is the deepest returned
objects, and under a budget it can name ids the result does not carry.

The section that needs to say it is `Traversal` — which is why this item carries
both halves. A selector that cannot reach the one paragraph a call keeps
re-reading has not fixed the problem.

## Constraints

- **DEC-010 pins the guide.** `src/learn.rs`'s prose pins assert coverage on
  whitespace-flattened text and `the_guide_teaches_exactly_the_nine_keys` reads a
  structural key block. A section split must not move a pinned sentence; the
  pins are the ratchet that makes the rewrite safe.
- **One const, many views.** Cutting sections from `GUIDE` by heading keeps one
  source of truth, so `learn` and `learn traversal` print the same bytes. Separate
  section constants assembled at print time would also keep the unnamed print
  unchanged, but the whole-guide pins would then read an assembly rather than the
  text. Prefer the cut; an unrecognised section name should refuse by name and
  list the names, never fall back to the whole guide.
- **A name is a new surface.** Once a caller can address a section, renaming a
  heading is a break. Either say so in one sentence of the guide, or accept the
  obvious variants (`Traversal`/`traversal`, `Querying`/`query`). Decide
  deliberately rather than by accident of matching.
- **`learn` stays store-free and side-effect-free**, as `--help`/`--version` are —
  a selector touches only argument parsing, never the store.

## Neighbours

IMP-007 (`smut render` short flags) is the other CLI-surface tidy from the same
period; no dependency either way. IMP-009 (the envelope's six members) is *why*
the guide gets re-read: if the response surface is regrouped, this item's value
drops and its sections change.

## Provenance

0.2.0 rc trial, frictions 3 and 4. The complaint sentences are the reporter's;
the measurements, line references and the DEC-010 constraint are this repo's.
