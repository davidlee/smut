# Traversal provenance

A result that traversed carries what the walk knew. The envelope gains one
top-level member, `traversal`, present exactly when the request supplies
`follow` or `backlinks` — either key, even an empty array. A non-traversing
request keeps 0.1's five keys, so the compatibility pin does not move (DEC-019).

```json
{
  "objects": [{"id": "q-query", "body": "..."}],
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
  `(source, field, target)`, in the order the walk encountered them, oriented
  source → target as stored. Derived from full entities before projection, so a
  `select` that hides the reference field still yields the edge. Dangling
targets are absent here and appear in `dangling`.
- `frontier` — discovered but never expanded, in discovery order (DEC-013).

`depths` and `edges` describe the result and are cut to selected ids after the
limit; `frontier` describes the walk and is not cut. The scalar is not sugar
over a node list: `objects` is the node list, and duplicating it would create a
second account of one fact.