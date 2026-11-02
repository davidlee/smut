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

## Two jobs, and only one of them is the original complaint

Operator direction of 2026-11-02 settled how the discriminator advice reaches an
author: through this surface. That makes the item two pieces with different
ranking — **delivery** (the cure for IMP-011, the top-ranked finding) and
**legibility** (the section selector, ranked seventh on its own). Delivery first.

### Delivery (Wave 1)

**These are sketches of reach, not a specification.** The operator offered them as
illustrations and has since set the mechanics aside: what is decided is that advice
reaches an author through this surface, not by which arguments. The pieces below
are therefore candidates, and the last one waits on IMP-013.

- **Strengthen the guide's own recommendation.** `learn` already ends with a
  `Conventions` section (`src/learn.rs:218`) describing the sets as advice. Turn it
  from a description into a recommendation: the habits that make the store
  queryable later, and the reason `type` heads the list. **Does not wait on
  anything.**
- **`smut learn <section>`** — retrieve one section by name (`labels`, `traversal`,
  `querying`). The names already exist as headings; the work is making them
  addressable, not naming anything new.
- **A flag, shape undecided.** `smut learn --with-conventions NAME` was the
  illustration: one call that delivers the guide *and* the named set, so an
  `AGENTS.md` can encode a single invocation. It cannot be specified until IMP-013
  settles whether sets resolve across `$PWD/.smut/conventions/`,
  `~/.config/smut/conventions/` and the binary — a flag that hardcodes a set name
  would be wrong the moment layering exists. Decide the loader first.
- **Nothing lands in a response envelope.** Conventions stay out of `declare` and
  `query` replies. The one edge case raised — a stderr warning when a declaration
  carries `kind` but no `type` — came with the same illustrations and is not a
  proposal; the fact worth keeping is that it would contradict a success-path
  `stderr is empty` pin, so it is IMP-011/IMP-012's contract question, not this
  item's work.

Consequences to plan for, not discover:

- **The default print changes.** A footer recommendation alters output that the
  guide's own pins hold (coverage pins over whitespace-flattened text, plus
  `the_guide_points_at_smut_conventions_once`, which deliberately pins the pointer
  *sentence* rather than the phrase). Re-cut those pins with a comment saying what
  changed and why — the DEC-019 precedent for re-cutting a compatibility pin.
- **A flag and a positional argument on one command** would need one grammar —
  does the flag stand alone, or does it also accept a section name beside it? That
  question is deferred with the flag itself (below), not answered here.

### Legibility (later)

The original complaint: 13,469 bytes, 217 lines, all-or-nothing, and an agent that
needs one rule back guesses instead of paying for the whole guide.

The material is already sectioned — thirteen headings (`Commands`, `Exit codes`,
`Declaring`, `References`, `Batch responses`, `Querying`, `Traversal`, `The
two-call recipe`, `Labels and rendering`, `Conventions`, `Rendering`, `Opaque ids`,
`Workflows`) — so this is not new writing. It is making existing names addressable,
which is what `smut conventions [NAME]` already does for its sets. The unnamed call
keeps printing the whole guide, footer included; a name prints that section.

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
drops and its sections change. IMP-013 (user and project conventions) gates the
flag: a `--with-conventions NAME` that hardcodes a set name would be wrong the
moment layered resolution exists, so the footer and section names ship first.

## Provenance

0.2.0 rc trial, frictions 3 and 4. The complaint sentences are the reporter's;
the measurements, line references and the DEC-010 constraint are this repo's.
RFC-001 ranks the *selector* seventh of ten; after the operator's 2026-11-02
decision the *delivery* half of this item carries the top-ranked finding
(IMP-011) instead, which is why the two halves are sequenced rather than ranked
together.
