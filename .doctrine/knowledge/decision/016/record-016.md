# The two-call idiom's home

Amended 2026-11-02 by SL-004 under ADR-001 (identity is `smut_id`, a reserved
key in a permissive store) and DEC-025: structural identity is spelled `smut_id`;
an ordinary `id` is caller data at every depth. Only the passages listed in SL-004
PHASE-03's amendment proposal (plus RV-028 F-2/F-3) changed; the rationale, durable
ids and unrelated decisions are retained verbatim.

The normative home is `smut learn`. It is descriptive and version-bound, and a
release that introduces traversal must teach the composition:

```
1. smut q session --backlinks part-of --depth 2 --ids
   smut query '{"from":"session","backlinks":["part-of"],"depth":2,"select":["smut_id"]}'
2. smut q <ids> status=open --select title
   smut query '{"from":[<ids>],"where":{"status":"open"},"select":["title"]}'
```

Each call is printed in both spellings, the short form first and the JSON it denotes
directly beneath it, and `smut learn` executes both. They agree on the set and not on
order: roots are the short form's spelling of an `$in` filter on identity, answered in
the order written, while the JSON twin filters in `smut_id` order.

Amended 2026-11-03 by SL-010 (RV-034 F-6): the recipe gained its short-form spelling
beside each JSON line. The JSON lines, the guard and the direction sentence are
unchanged, and `tests/fixture.rs` runs both spellings so a printed line that stops
working fails the suite.

The direction is the first thing a reader gets wrong, so the example states it:
`part-of` points child → container (a cluster is part-of a session), therefore
"what hangs off this session" is a **backlink** question. `follow:["part-of"]`
from a session returns the session and nothing else, and the second call then
answers "no open questions" in a complete-looking envelope. Containment is
always asked with `backlinks`, and `follow` answers the opposite question — what
this entity belongs to, addresses or cites. (The store this release is built
from carries the inverted example in `finding-smut-18`; copying an example out
of the store is how the error travels.)

`select: ["smut_id"]` keeps the first call cheap; the second is an ordinary root
filter because `from` takes an explicit id list. `where` stays roots-only; the
composition is the answer, not a limitation to route around.

What the composition does **not** give away is completeness. The second call
cannot see what the first dropped, so the recipe carries a guard that is part
of it, not advice beside it: read `truncated` and `truncation_reasons` on the
first response before extracting ids. Either reason — `result_limit` or
`traversal_budget` — makes the second answer a statement about the ids you
received, not about the graph. Retry once with a bound the store can meet —
`limit` above the discovered count, or `budget` above the expansions the query
needs. If it truncates again, or the caller will not spend that scan, stop and
answer partially, naming the reason and the bound that was refused. Repetition
alone never establishes completeness: only a response that says it truncated
nothing does, and even then only about the ids that response returned. Two bare calls that skip the guard can
report "no open questions" while holding a truncated answer, and that is the
one failure this idiom exists to avoid.

The conventions library states the **principle** — filter roots, not what
traversal discovered; prefer two calls to a wider language — and points at
`learn` for the recipe. It does not repeat the recipe. Two live copies of one
procedure drift, and this repository has already paid for that class of defect.

The guide also covers, in the same revision: the query keys and defaults,
predicate summary with missing/null semantics, traversal labels as top-level
fields, the `traversal` provenance member, truncation reasons, a minimal label
vocabulary, and bounded rendering. `learn` stays one document.



