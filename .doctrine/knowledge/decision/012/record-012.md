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
