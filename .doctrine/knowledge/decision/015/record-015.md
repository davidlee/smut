# Budget accounting

`budget` counts **entity expansions**. One expansion is one scan of an entity's
requested outgoing and incoming fields, however many edges that finds. It is not
a count of discovered entities and not a count of edge visits.

Before each expansion the walker checks `expanded < budget`. When the check
fails, the entity and everything else still due for expansion become the
frontier (DEC-013), and the walk stops.

`truncated: true` with reason `traversal_budget` appears **only when expansion
was actually declined** — there was at least one entity that the depth bound
would have allowed expanding. A walk that empties its queue exactly at the
budget is complete and is not truncated, mirroring `result_limit`'s rule that
only a real omission truncates.

`depth: 0` performs no expansions and is never budget truncation. `result_limit`
is applied after ordering, so both reasons can fire; they are listed in the
order they can occur, `traversal_budget` then `result_limit`, unique.

The budget does **not** bound the initial whole-store scan for an absent `from`
or for `where`, and it does not bound the store scan each `backlinks` expansion
performs. Those are costs inside an expansion, and the README and guide say so
rather than implying a hard total-work bound.
