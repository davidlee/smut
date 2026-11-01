# The two-call idiom's home

The normative home is `smut learn`. It is descriptive and version-bound, and a
release that introduces traversal must teach the composition:

```
1. smut query '{"from":"session","follow":["part-of"],"depth":2,"select":["id"]}'
2. smut query '{"from":[<ids>],"where":{"status":"open"},"select":["title"]}'
```

`select: ["id"]` keeps the first call cheap; the second is an ordinary root
filter because `from` takes an explicit id list. `where` stays roots-only; the
composition is the answer, not a limitation to route around.

The conventions library states the **principle** — filter roots, not what
traversal discovered; prefer two calls to a wider language — and points at
`learn` for the recipe. It does not repeat the recipe. Two live copies of one
procedure drift, and this repository has already paid for that class of defect.

The guide also covers, in the same revision: the query keys and defaults,
predicate summary with missing/null semantics, traversal labels as top-level
fields, the `traversal` provenance member, truncation reasons, a minimal label
vocabulary, and bounded rendering. `learn` stays one document.
