# ISS-003: Generated ids are drawn only for top-level declarations, so a connected graph takes two batches

The operator's observation, and it is the authoring-shaped face of ISS-002.

`normalize` iterates declaration *roots* (`declaration_roots` in `src/declare.rs`):
only a root that arrives without an `id` receives a generated one. At depth there
is no generation at all — an object without an `id` is ordinary embedded data, so
it takes a generated id only when it is promoted to a root in its own declaration.
Measured:

```
❯ smut declare '{"id":"p","payload":{"title":"t"}}'     # payload stays a value
❯ smut get p
{"objects":[{"id":"p","payload":{"title":"t"}}],…}      # no entity was created
```

So nesting an entity means *naming* it — and the name is the one thing the store
will never let you change. That is the collision: generated ids are the safer
default given immutability (a nanoid cannot be a typo, cannot go stale, and IDE-002
would make renames unnecessary rather than unavailable), yet generated ids are
exactly the ones unavailable where the graph is authored, because a nested
occurrence must carry its target's name at the moment of writing.

## What that costs

An id is only known after the commit that generated it, so every edge must be
written after its endpoints exist. Authoring a connected graph therefore takes two
batches — create, then wire — measured here with a cycle to be sure the wiring
batch is not secretly limited:

```
❯ smut declare '[{"title":"a"},{"title":"b"}]'
{"ids":["5SRULQst","iLYEcEot"],…}
❯ smut declare '[{"id":"5SRULQst","related":[{"id":"iLYEcEot"}]},
                {"id":"iLYEcEot","related":[{"id":"5SRULQst"}]}]'
{"changes":[{"id":"5SRULQst","previous":{"id":"5SRULQst","title":"a"},"status":"updated"},…]}

❯ smut query '{"from":"5SRULQst","follow":["related"],"depth":2,"select":["id"]}'
{… "edges":[{"field":"related","source":"5SRULQst","target":"iLYEcEot"},
            {"field":"related","source":"iLYEcEot","target":"5SRULQst"}],…}
```

Authoring with semantic ids has no such seam: forward references resolve within one
batch, so a whole graph — cycles included — is one declaration and one commit. The
policy that is cheapest to *write* is thus the one that is expensive to *repair*,
and the guide's "two-call recipe" is presented as a reading discipline; the same
shape is silently mandatory for writing.

## What a cure would have to be

- **A minting verb, not new declaration syntax.** `smut new --count 3` returning
  three fresh ids with nothing stored would let a caller author a whole graph in one
  batch, with names it did not invent, and add no reserved key or second identity
  shape to the value space. It also keeps DEC-005's rule (one shape means identity)
  intact. Open question it must answer: a minted id is not reserved, so a store that
  gains that id between minting and declaring turns the declaration into an upsert
  — either mint reserves (a store write for a read-shaped verb), or the exposure is
  accepted as a single-user notebook's race.
- **A placeholder in the declaration.** `{"id":"$0"}` bound by the response would
  work in one batch, but it puts a second reserved spelling into the value space,
  which is ISS-002's problem one key along — the fix would be paid for in exactly
  the currency the other issue is complaining about.
- **Promote-on-request.** A marker key on a nested object ("declare me") generates
  an id at depth — which reserves yet another key and, worse, an occurrence whose
  identity depends on a sibling key rather than on `id`.

Of the three, the minting verb is the only one that adds no reservation, which
makes it the candidate worth designing first.

## Direction — 2026-11-01 (settled separately from the key rename)

Operator judgement, which this item adopts: "Support pre-minting ids, but settle its
contract separately. It works under either identity spelling and gives two calls, one
atomic graph write."

That settles what this item gets wrong above. Minting is **not** a cure the rename
makes possible — prechosen ids already author a cyclic graph in one declaration batch
today (verified: two prechosen ids, reciprocal references, `missing: []`, both edges
traversed). A minting verb substitutes *generated* strings for prechosen ones, so its
value is that the caller does not have to invent names, not that graph authoring
newly becomes possible. What it buys exactly is: mint, then declare — two invocations,
one mutation. "One-call graph authoring" was the wrong claim and is withdrawn.

The rest stands:

- Placeholders and declare-me markers remain rejected as "cleverness which only
  amounts to a partial solution" — each buys one case by reserving another spelling.
- The unreserved-id race is accepted, recorded for completeness: a minted id that the
  store gains before the declaring batch would turn the declaration into an upsert.
  Expected never to occur in smut's lifetime; noted, not designed for.

What this item owes, and all it owes: the verb's contract — its name, whether it takes
a count, and what its response looks like beside the existing envelopes. It is a
separate change from the identity rename (ADR-001 / SL-004) and is not sequenced
behind it.
