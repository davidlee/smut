# IMP-009: The query envelope is six flat members whose boundaries a caller has to learn from prose

The reporter's "thing I'd actually change", and the one friction they declined to
put a number on:

> A single envelope now carries `objects`, `missing`, `dangling`, `truncated`,
> `truncation_reasons`, `traversal{depths,edges,frontier}` — six fields whose
> meanings are related but distinct, and I confused two of them *while reading
> documentation I had just read*. Documentation competes with momentum, and
> momentum wins when the envelope is verbose.

They are right about the shape. The members divide three ways and nothing in the
JSON says so:

| subject | members |
| --- | --- |
| the result | `objects`, `truncated`, `truncation_reasons` |
| the walk | `traversal.depths`, `traversal.edges`, `traversal.frontier`, `dangling` |
| the request | `missing` |

## The instance that actually cost answers

The exercise behind RFC-001 narrowed the complaint to something more concrete than
"six members are confusable": **the root of a traversal is unidentifiable inside
`objects`**. `where` filters roots only, so the root cannot be excluded from the
result; `traversal.depths` carries the information out of band, keyed by id. Every
"how many entities point at this one" answer therefore needs the root subtracted
by hand, and an off-by-one there is invisible in the answer.

Seven of the eighteen answers in that exercise did exactly that subtraction
(`.length-1`, or `select(.id != …)`), across two of the three stores:

```
❯ smut query '{"from":["zone-eastgate"],"backlinks":["in_zone"],"depth":1,"select":["id"]}'
["zone-eastgate","p-church-eastgate","p-oak-eastgate"]
```

The root is present because it is a result of the walk, and there is no marker in
the object that says so — only a side map the caller must consult. The earlier
framing (six members with related meanings) is not wrong; it is the *second*
problem, and the first one has a cheaper cure.

**Correction (2026-11-02).** The information was never missing: a root is exactly
the object whose `depths[id]` is `0`. Those seven answers subtracted the root by
hand because the convention is unstated and out of band — a teaching gap, which is
the same class as this project's top-ranked finding (advice unheard, not data
absent). That demotes the cure from "add a member" to "state the rule", and it is
the reason option 1 below is now deferred rather than argued.

## This is a contract move, not a tidy

- **DEC-019 (accepted)** is explicit: non-traversing envelopes keep exactly the
  0.1 five keys, `traversal` is conditional (DEC-012), and `tests/compatibility.rs`
  pins the whole parsed 0.1 envelope including its key set. `query {}` is
  byte-identical, deliberately.
- **SL-002 design §8** keeps the envelope flat by decision, not by default.
- So any restructuring supersedes DEC-019 and re-cuts that pin. The honest moment
  is at or before 1.0 — i.e. with or before IMP-002 — because afterwards it is a
  break with consumers rather than a firming.
- **But not everything here is that large.** A traversing response is 0.2-only, so
  an *additive* member on it is invisible to the 0.1 pin: `traversal.roots` breaks
  no 0.1 caller because no 0.1 caller made a traversing request. Only the
  restructure below needs a DEC-019 successor; the first option does not.

## Options, not a proposal

Status is `design-deferred` for the *restructure* and for the first option, which
was argued as the cheap half and is now deferred instead:

1. **State the convention before naming the roots.** One sentence in the guide's
   envelope paragraph — `depths` keys every returned object and `0` marks a root —
   is the whole measured cure (see the correction above). A `traversal.roots`
   member is the fallback if that sentence does not take, and two measured hazards
   make it less obviously good than it looked:
   - **roots can be the whole store.** `follow` with `from` absent makes every
     entity a root, so an uncut `roots` duplicates the entire result, and cutting
     it by `limit` makes provenance lie. `frontier` is the precedent for an uncut
     array, but it is one hop past the result, not the store.
   - **`order` + `limit` can cut the root out of `objects` entirely.** Measured:
     `objects` `[q1,q2]` with no depth-0 entry at all, `truncated: true,
     result_limit`. A cut `roots` would then be empty and an uncut one would
     disagree with `objects` — a second thing to reconcile exactly where the guard
     already says "stop and answer partially".

   If it is ever added it goes **inside `traversal`, never top-level**: the
   traversing response's key set is pinned to "the five 0.1 members plus exactly one
   conditional member" (`tests/compatibility.rs`), which is the property DEC-019
   exists to protect, and only the two exact envelope literals in
   `tests/fixture.rs` would need re-cutting for an inner member.
2. **Keep the shape; carry the taxonomy in the guide.** Cheapest of the rest, and
   the friction mostly stands. IMP-008's section selector lowers the cost of the
   re-read the reporter actually avoided, which may be enough on its own.
3. **Rename before restructuring.** Keep it flat and let the walk's members say
   "walk": `traversal` → `walk`, `dangling` → `walk.dangling` or `dangling`
   kept but named in the same breath. One rename, no nesting, smaller blast radius
   than a restructure — and still a break.
4. **Group by subject.** `{"result": {objects, truncated, reasons}, "walk":
   {depths, edges, frontier, dangling}, "request": {missing}}`. The most legible
   and the largest break; it moves every pin at once.
5. **A diagnostic channel.** If a caller-visible "nothing matched" or "this
   condition is contradictory" note is ever wanted (IMP-010), it needs a member
   that does not exist today. That is this conversation, not a separate one —
   adding a sixth flat key to fix a complaint about six flat keys would be an
   unhappy answer.

Whatever is chosen must keep the per-request honesty the recipe depends on: a
`truncated: false` on a second response is not evidence about the first, because
the envelope describes one request. A nested `result.reasons` must not read like
store-wide truth either — grouping changes shape, never scope.

## Related decisions, not to be reopened here

- `traversal` is conditional on a traversal being requested (DEC-012), and that
  conditionality is what lets the 0.1 pins survive. Any regrouping keeps it.
- `truncation_reasons` order (`traversal_budget`, then `result_limit`) and the
  exit-code contract are outside this item; they are pinned and settled.

## Neighbours

IMP-008 (guide sections) is the cheap half of the same complaint and may retire
the expensive half. IMP-010 (a contradictory condition answered silently) is the
friction that would *want* a diagnostic channel, if one is ever added. Neither
blocks this.

## Ready to pick up when

A design conversation on the response surface is scheduled. The natural host is
the 1.0 presentation release (IMP-002), or a dedicated surface-review slice,
which does not exist yet. Either way it must decide DEC-019's successor before
1.0 ships.

## Provenance

0.2.0 rc trial, friction 5 — the one the operator flagged as "the thing I'd
actually change" — reframed by the three-store exercise behind RFC-001, which
supplied the measured instance and the additivity nuance. The taxonomy,
the DEC-019/SL-002 §8 constraints and the options are this repo's reading.
