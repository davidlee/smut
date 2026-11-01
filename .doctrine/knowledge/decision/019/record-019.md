# Additivity for 0.1 callers

Widening the grammar must not move a working 0.1 response.

- `where`, `follow`, `backlinks`, `depth` and `budget` become accepted keys; the
  future-key refusal is removed. Unknown keys stay plainly unknown, and bad
  types or malformed predicates reject the whole request before any store
  access, as 0.1 already does.
- A 0.1 spec parses to the same semantics: `query {}` still selects every
  entity subject to the default limit, `from`/`select`/`order`/`limit` are
  unchanged.
- The non-traversing envelope is exactly 0.1's five keys. `traversal` appears
  only when `follow` or `backlinks` is supplied (DEC-012), so the pinned 0.1
  response keeps its bytes and its key set.
- `traversal_budget` can appear only on a traversing request, where 0.1 had no
  behaviour to preserve.

The compatibility suite is re-cut, not deleted. The 0.1 `query {}` pin stays
byte-for-byte. The test that asserted `where` is refused by name becomes a 0.2
acceptance test, and a new pin asserts the 0.1 response is unchanged immediately
beside a successful 0.2 query — additivity is the property, not the old
refusal. The re-cut carries a comment saying what changed and why.
