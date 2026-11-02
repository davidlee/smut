# Traversal provenance

Amended 2026-11-02 by SL-004 under ADR-001 (identity is `smut_id`, a reserved
key in a permissive store) and DEC-025: structural identity is spelled `smut_id`;
an ordinary `id` is caller data at every depth. Only the passages listed in SL-004
PHASE-03's amendment proposal (plus RV-028 F-2/F-3) changed; the rationale, durable
ids and unrelated decisions are retained verbatim.

A result that traversed carries what the walk knew. The envelope gains one
top-level member, `traversal`, present exactly when the request supplies
`follow` or `backlinks` — either key, even an empty array. A non-traversing
request keeps 0.1's five keys, so the compatibility pin does not move (DEC-019).

```json
{
  "objects": [{"smut_id": "q-query", "body": "..."}],
  "missing": [],
  "dangling": [{"source": "smut", "field": "questions", "target": "q-gone"}],
  "truncated": true,
  "truncation_reasons": ["traversal_budget"],
  "traversal": {
    "depths": {"smut": 0, "q-query": 1},
    "edges": [{"source": "smut", "field": "questions", "target": "q-query"}],
    "frontier": ["q-answer"]
  }
}
```

- `depths` — one scalar per **returned** object: minimum discovery depth in the
  mixed graph (both directions), fixed by breadth-first first discovery.
- `edges` — the induced edge set among **returned** objects, deduplicated by
  `(source, field, target)`, in result order: for each returned id in the
  result's own order, first occurrence wins. Oriented source → target as stored.
  Derived from the returned entities after the limit and before projection, so a
  `select` that hides the reference field still yields the edge. Dangling
  targets are absent here and appear in `dangling`.
- `frontier` — discovered but never expanded, in discovery order (DEC-013).

`depths` and `edges` describe the result and are cut to selected ids after the
limit; `frontier` describes the walk and is not cut. The scalar is not sugar
over a node list: `objects` is the node list, and duplicating it would create a
second account of one fact.


## Amendment — 2026-11-02: `edges` is the third trigger

SL-002 shipped `traversal` conditional on `follow` or `backlinks`, and its own
rationale said the edge set was "the single derivation the 1.0 renderer and
clients share". The conditionality defeated half of that: a `where`-only query
returned no edge set at all, so the only way to reach a renderer outside `smut`
was to spell a projection request with a traversal knob — `"follow": []`. The
capability existed; the spelling was a trick.

Amended because IMP-018 measured the gap and the operator chose the request-key
spelling over documenting the trick (2026-11-02, this session):

- `edges: true` is accepted as a tenth specification key. It parses to
  `follow: Some(Fields::Named([]))` — the same representation as `follow: []`,
  applied after the key loop so it never overrides a direction the request
  named. There is no second representation for anything downstream to disagree
  with.
- `false` and absent are the absent request. DEC-019's 0.1 key-set pin does not
  move: no 0.1 caller sends `edges`, and a request that does now traverses.
- The same amendment fixed a defect the spelling would otherwise have blessed:
  the walk charged one budget unit per root expansion even when no field was
  requested, so `follow: []` (and therefore `edges`) on a request with more
  roots than `budget` reported a phantom `traversal_budget` truncation. A
  traversal that requests no field in either direction now expands nothing, so
  it spends no budget and cannot truncate. `depth: 0` still frontiers its roots,
  because the depth boundary is decided before any expansion.
