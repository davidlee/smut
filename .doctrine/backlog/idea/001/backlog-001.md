# IDE-001: smut and cordage: how they stack up next to each other

An idea, not a plan. Captures one session's comparison so the question is not
re-derived from scratch the next time it comes up.

**cordage** — `~/dev/doctrine/crates/cordage`, 0.2.0, published, zero
dependencies. A *build-then-query* evaluation engine over one closed node set and
N typed directed overlays. Each overlay declares a `CyclePolicy`
(`Reject`/`Evict`) and an `Arity` (`AtMostOne`/`Unbounded`). Queries: reachability
in both directions (optionally depth-bounded, with a `truncated` flag), composed
topological ordering across overlays, channel propagation (`Any`/`All`/`Max`/
`CountDistinct`) with per-node contributor sets, and structured provenance for
cycles, evictions and order conflicts. It carries no domain vocabulary and never
mutates.

**smut** — this repo. A mutable, schemaless, one-binary graph store. Edges are
implicit: any embedded object carrying an `id`, labelled by its top-level field
name. Traversal is a BFS over full entities with cycle termination by
expand-once, dangling references reported, record ordering by declarative sort
keys. No schema, no configuration file, no derived state.

## The shapes, on one axis

```
  untyped, mutable, permissive            typed, resolved, explainable
  smut ──────────────────────────────────────────────── cordage
        labels from field names            labels declared as overlays
        cycles terminate, unreported       cycles diagnosed or evicted
        ordering = record sort keys        ordering = topological composition
        no derived state                   channel propagation + provenance
```

## Why this is interesting at all

1. **cordage is a candidate implementation of one of smut's explicit
   non-goals.** The v1 brief (`smut-implementation-brief.org` § *Deliberate
   follow-ups, not v1*) defers "first-class edge types, schema enforcement and
   integrations". Typed overlays are cordage's entire reason to exist. So this is
   not "two graph things, maybe they fit" — it is a named deferral meeting a
   built answer to it.

2. **The ethos matches more than the domain does.** Both refuse to guess. smut
   "refuses by name instead of guessing" and reports dangling references "not
   expanded or repaired". cordage's `build()` errors only on malformed input;
   "cycles, evictions, and degradation are data in provenance, never errors", and
   invalid channel seeds are surfaced, never coerced. Both are obsessive about
   determinism (smut's total field comparator with missing-last; cordage's
   `OrderKey`). Same posture, different altitude.

Expected and unsurprising: both come from the same author and the same
house style.

## Capability overlap

| capability | smut today | cordage | worth it? |
|---|---|---|---|
| edge typing | field name is the de facto label | overlay = explicit type + policy | **deferred by the brief** — a decision, not a gap |
| cycles | terminate by expand-once | full diagnostic (nodes + back edges) | marginal: ~60-line DFS does this, no engine needed |
| backlinks | whole-store scan per incoming hop (`src/walk.rs`) | prebuilt in-adjacency, built once | perf only; a wash at one hop, a win at many |
| depth budget | `depths` + frontier | `reachable_bounded` + `truncated` | already equivalent |
| ordering | record sort keys | topological compose across overlays | **orthogonal, not competing** |
| derived state | *absent — cannot be expressed* | `evaluate` + contributors | **the real gap** |

The last row is the only capability smut genuinely cannot express. `where` is
per-record; "any open question anywhere under this project" or "max severity in
this subtree" is a fold over a reachable set, and smut has nowhere to put one.
cordage offers it as a read-time projection with a contributor set — a much better
story than persisting derived fields into a schemaless store.

## Three collisions, and how bad each is

1. **Build-then-query vs. an open mutable store.** Resolves itself: build a
   `Graph` per query from a snapshot. smut already reads a whole snapshot per
   invocation, so the O(V+E) build is not new cost — and for incoming traversal
   it may beat the current per-hop store scan. Least scary of the three.

2. **Opaque `NodeId` vs. user-authored string ids.** An adapter map both ways.
   Trivial, but exactly where cordage's product-neutrality claim gets tested.
   cordage does no serde, so smut keeps `serde_json` and cordage stays
   dependency-free. Clean seam.

3. **Policy lives per overlay; smut has no configuration file.** The real one.
   cordage's dialect is *configured* overlays; smut's is *zero* configuration.
   Any adoption must therefore infer uniform defaults — every field label becomes
   `Unbounded` + `Reject`, provenance surfaced in the envelope. Legitimate, but it
   means adopting cordage buys *one* policy decision made for you, and deferring
   "first-class edge types" was precisely the decision to not make those policy
   decisions. Integration is therefore in tension with the reason the feature was
   deferred in the first place.

Also note: cordage's own README says not to use it when you need graph mutation
after construction. smut's thesis is mutation. That is fine at the per-query
snapshot boundary and wrong at the storage boundary. **Never put cordage under
the store.**

## The reverse direction is cheaper and less speculative

cordage claims product-neutrality, but doctrine is currently its only consumer, so
the claim is untested. smut is an honest second consumer for a small example
harness: a JSON entity store that can be grown to any messiness with one
`declare`, containing nested references, dangling targets, duplicate labels and
cycles. cordage's scale gates (`tests/scale_cliffs.rs`) use synthetic shapes
(`deep_chain`, dense cycles); a messy real store is a better adversarial corpus
for build validation and determinism than another generator.

Caveat on how much this proves: a second consumer written by the same author
still makes the API match a differently shaped model, so it tests the *API*. It
does not independently test whether the abstraction is genuinely general, because
one author holds both mental models at once and will unconsciously accommodate.

## Boundary rule, if anything ever comes of this

The adapter lives in `smut/src/`. Nothing smut-shaped (field labels, `dangling`,
the envelope) grows into `cordage/src/`; nothing doctrine-shaped either — cordage
is a declared product-neutral leaf (`ADR-001`, `REQ-079`), and with a single
copyright holder there is no legal friction to hold that line. Only discipline
does.

## Possible next steps

Unscheduled. Believed order of value:

1. **Cheap adapter experiment, when the question is live.** Load a real smut
   store, build a `Graph`, add one field name as an overlay, evaluate one
   `Combinator::Any` channel, and try to express the result in the existing
   envelope without lying about where the value came from. Read: does it look
   natural in under ~150 lines (cordage pre-vetted for the day edge types stop
   being deferred), or does the mapping and the policy defaults need more than
   ~300 lines of apology (the answer is no)? Both crates are published, so it is
   a `cargo add` (or a scratch path dep) and an hour, not a day.
2. **Write the answer down either way.** A decided-no with reasons
   (per-query snapshot, uniform policy defaults, loses the zero-config posture) is
   worth more than the experiment, because this question will be asked again.
3. **Second-consumer determinism fixture for cordage.** Feed a generated messy
   smut store to cordage's own harness. The direction where the combination is
   unambiguously positive, and it costs nothing in smut's tree.

## Current lean

Do not integrate now. The two highest-value intersections (derived state, graph
ordering) are blocked behind a deferral that was deliberate, and the cheapest one
(cycle diagnosis) is too small to justify a dependency. Note that graph ordering
specifically *requires* edge typing: without knowing which field name means
"sequence", there is no axis to topologically order along — so it inherits the
same deferral rather than standing on its own.
