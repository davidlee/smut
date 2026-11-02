# IMP-018: `follow: []` already turns any query into a graph query, and the guide never joins that to the edge set

Raised 2026-11-02 while asking what a Mermaid/D2 renderer would cost (IMP-002).
The question was going to be "make the envelope carry edges for non-traversing
requests". Measured, that is already true — under a spelling nobody would guess.

## The fact, measured

A `where`-only query returns no edge set, because `traversal` is conditional on a
traversal being *requested* (DEC-012):

```
$ smut query '{"where":{"title":{"$in":["Hub","A"]}}}'
{"dangling":[],"missing":[],
 "objects":[{"smut_id":"a","tags":["x"],"title":"A"},
            {"smut_id":"hub","children":[{"smut_id":"a"},{"smut_id":"b"}],"title":"Hub"}],
 "truncated":false,"truncation_reasons":[]}
```

The same request with `"follow": []` supplied carries the **complete induced edge
set**:

```
$ smut query '{"where":{"title":{"$in":["Hub","A"]}},"follow":[]}'
{"…": "…",
 "traversal":{"depths":{"a":0,"hub":0},
              "edges":[{"field":"children","source":"hub","target":"a"}],
              "frontier":[]}}
```

That is not a quirk of this example. `edge` derivation in `select::envelope` is
**result-scoped and walk-independent**: it scans the full stored entity of every
returned id and keeps a pair when the target is returned too
(`src/select.rs`, `fn edges`). The walk's only job here is to make
`traversal` exist. And a walk that follows no field expands the roots and
discovers nothing, so it costs **zero store scans, zero budget, no frontier** —
`depths` is uniformly `0`, which is the tell that nothing was traversed.

So the capability is already delivered by DEC-012's own rationale, which says
"edges are the single derivation the 1.0 renderer and clients share". It is
delivered on the traversal path only, and the off-label spelling for the other
path is a traversal knob being used as a projection switch.

## Why it is worth an item anyway

Because both halves of that sentence are documented and **the join is not**:

- `learn` states the grammar (`src/learn.rs:131`): *"follow: [] is a supplied
  traversal that follows nothing, which is not the same request as an absent
  follow."*
- `learn` and `README.md:224` state that a request supplying `follow`/`backlinks`
  — even as an empty array — carries a `traversal` member.
- Nothing anywhere says the obvious consequence: **supply `follow: []` and any
  query hands you the induced subgraph of its own result, for free.**

The two facts sit in different paragraphs, and a reader who has internalised
"`follow` walks outgoing references" reads `follow: []` as "walk nothing" and
never as "give me edges". The `depth`/`budget` sentence next to it reinforces the
misreading, since it explicitly teaches that traversal keys can be present and
inert.

This matters exactly as much as having a renderer outside the binary matters:
`smut query … | jq … | <mermaid|d2|dot>` needs one thing from the tool — the edge
set — and today the obvious way to ask (`where`) does not return it. A fifteen-line
shell converter is a viable alternative to a Rust renderer (see IMP-002), but only
over a request shape that returns edges.

## What it would take

Tiered, cheapest first. The first two tiers move no pin.

1. **Join the two facts in `learn`, with the measured example.** One paragraph:
   `follow: []` is the way to ask for the edge set without traversing; every
   `depths` entry is `0` and no expansion happened; `edges` is the induced
   subgraph of the *returned* objects, so it is cut by `limit` exactly as
   `objects` is and never reaches outside the result. Same example in
   `README.md`'s query section. Cost: prose plus the guide's existing
   content tests; no code.
2. **Name the intent in the CLI.** If the join has to be repeated, a
   `smut query --edges` / `smut render --edges` that supplies `follow: []`
   under the hood — or the same folded into IMP-007's short-flag work — turns a
   trick into a surface. Requires a flag/format-ignored table entry and the
   request-shape tests.
3. **Make the envelope do it without being asked** — the original framing, and
   the one that is *not* cheap. DEC-012 makes `traversal` present iff
   `follow`/`backlinks` was supplied; DEC-019 pins the non-traversing envelope to
   0.1's exact five keys, and `tests/compatibility.rs` asserts that key set. Any
   unconditional edge member moves both, so it is DEC-019's successor, not a
   patch. It belongs to IMP-009's response-surface conversation and should not be
   decided here — especially since tier 1 delivers the same capability for free.

A fourth option is not a fifth format: emit nothing new and let callers use
`get` per entity. It is strictly worse than tier 1 because it re-derives the
edge rule outside the tool, which is the one derivation DEC-012 exists to make
shared.

## Non-goals

- Not a proposal to add D2, DOT or any second `--format`. The point is that the
  projection, not the formatter, is the contract worth publishing; if it is
  reachable, format breadth is a shell exercise.
- Not a reopening of the induced-subgraph rule (dedupe by
  `(source, field, target)`, both endpoints returned, dangling excluded, result
  order). That rule is what makes the external converter agree with the Rust
  renderer, and it is correct as written.
- Nested-path labels (`scan_refs` carries the *top-level* field name through
  embedded wrappers) are a separate expressiveness limit; not addressed here.

## Neighbours

- **IMP-002** (1.0 presentation release) is where a consumer for the edge set
  first exists, and its "README … covering the whole surface" deliverable could
  absorb tier 1 at triage time. Either way the join should land with or before
  the first format, not after it.
- **IMP-009** (envelope boundaries) owns the only expensive tier here, and has
  already established that a *traversing*-only additive member is invisible to
  the 0.1 pin. That is the escape hatch if tier 1 ever proves insufficient:
  `follow: []` responses are traversing responses, so anything added to
  `traversal` is 0.2-only by construction.
- **DEC-012 / DEC-019** are the binding constraints; this item proposes moving
  neither.
- **IDE-004** (no edge attributes) is the adjacent data-model limit; it makes a
  graph *less* expressive but not less available, so it does not block this.

## Provenance

Raised 2026-11-02 from the operator's question about building a Mermaid graph out
of a query. The measured transcripts above are from
`target/release/smut` at 0.2 with a three-entity scratch store; the two
contradictory-looking envelopes differ only by `"follow": []`.
