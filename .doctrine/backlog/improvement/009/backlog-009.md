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

The confusable pair is exactly the one measured in IMP-008: `objects` and
`frontier` overlap at a depth boundary, and only the guide's paragraph
distinguishes them.

## This is a contract move, not a tidy

- **DEC-019 (accepted)** is explicit: non-traversing envelopes keep exactly the
  0.1 five keys, `traversal` is conditional (DEC-012), and `tests/compatibility.rs`
  pins the whole parsed 0.1 envelope including its key set. `query {}` is
  byte-identical, deliberately.
- **SL-002 design §8** keeps the envelope flat by decision, not by default.
- So any restructuring supersedes DEC-019 and re-cuts that pin. The honest moment
  is at or before 1.0 — i.e. with or before IMP-002 — because afterwards it is a
  break with consumers rather than a firming.

## Options, not a proposal

Status is `design-deferred`: this needs a conversation about the response surface,
not an implementation task.

1. **Keep the shape; carry the taxonomy in the guide.** Cheapest, and the friction
   stands. IMP-008's section selector lowers the cost of the re-read the reporter
   actually avoided, which may be enough of a cure on its own.
2. **Rename before restructuring.** Keep it flat and let the walk's members say
   "walk": `traversal` → `walk`, `dangling` → `walk.dangling` or `dangling`
   kept but named in the same breath. One rename, no nesting, smaller blast radius
   than a restructure — and still a break.
3. **Group by subject.** `{"result": {objects, truncated, reasons}, "walk":
   {depths, edges, frontier, dangling}, "request": {missing}}`. The most legible
   and the largest break; it moves every pin at once.
4. **A diagnostic channel.** If a caller-visible "nothing matched" or "this
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
actually change". The taxonomy above, the DEC-019/SL-002 §8 constraints and the
options are this repo's reading, not the reporter's.
