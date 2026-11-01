# Frontier

The frontier is every entity the walk **discovered but never expanded**, in
discovery order and deduplicated. It answers the question `truncated: true`
cannot: what was left undone, and where.

An entity is **expanded** when the walker has scanned its requested outgoing
and incoming fields. It is in the frontier when it was discovered but that scan
did not happen, for one of two reasons:

- **depth** — first discovered at depth equal to `depth`, so expanding it would
  exceed the hop bound. With `depth: 0` every root is in the frontier.
- **budget** — still due for expansion when the expansion cap was reached.

Only the budget cause sets `truncated: true` and `traversal_budget`; a depth
boundary is intentional selection. A dangling reference is not a discovered
entity and never appears in the frontier. `limit` does not affect membership:
the frontier is fixed by the walk, before ordering and the result cap.

An entity with no edges is still expanded (the scan found nothing) and is
therefore not in the frontier. If an entity is discovered twice, breadth-first
order fixes its first, minimum depth, and it is expanded once.

The frontier can name ids that are absent from `objects`, because the result cap
may have omitted them. That is deliberate: `depths` and `edges` describe the
result, `frontier` describes the walk.
