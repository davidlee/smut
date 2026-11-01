# Identity is smut_id, and generated ids can be minted

## Context

ADR-001 (proposed, awaiting a second review) moves the identity key from `id` to
`smut_id`. This slice is the change that follows from it, and it takes the three
backlog items it governs as one unit, because each one's answer moves the other
two:

- **ISS-002** — `id` is reserved in every nested object, so a payload's own `id`
  cannot be stored. Measured today: a nested `{"id":"abc","name":"Ext"}` silently
  declares an entity and strips the payload to `{"id":"abc"}`; a nested `{"id":"zzz"}`
  is a reference reported `missing`; a nested `{"id":null}` or `{"id":7}` refuses the
  whole batch at exit 2. The rename closes this: bare `id` becomes ordinary data at
  any depth.
- **ISS-003** — generated ids are drawn only for a top-level declaration, so a nested
  entity must be *named* at the moment of writing. A connected graph therefore takes
  two batches (create, then wire), where semantic ids need one. The minting verb is
  the cure, and it is only coherent once `id` is no longer reserved.
- **IDE-002** — ids cannot be renamed, so a name chosen badly is permanent. With the
  minting verb making generated ids the easy default, ids stop being chosen by hand,
  which is what drains the motivation for a rename verb. This slice *dispositions*
  that idea rather than implementing it.

Placing the change: pre-1.0, one real store in the wild
(`interview.local/.smut`, 66 entities, outside this repo) and one committed fixture
store. There is no external consumer to keep working, which is why the ADR takes an
unmitigated break rather than a migration.

## Scope & Objectives

1. **The key moves.** `smut_id` expresses identity everywhere an entity object
   appears — entity files, `objects[]`, `changes[].previous`, the
   `conventions --format json` declaration, rendered examples — and in every
   reference at any depth. `changes[]` names its subject `smut_id`. Container-named
   id *values* keep their names (`ids`, `deleted`, `missing`, `frontier`, `depths`,
   `edges[].source`/`.target`, `dangling[].source`), per ADR-001's rule — an entity
   object carries the key, a value named by its container does not.
2. **Bare `id` becomes data.** A nested `{"id":7}`, `{"id":null}`, `{"id":""}` or
   `{"id":{"deep":"object"}}` is stored as an ordinary value. `extract` cannot
   refuse a nested id any more because it never looks at one; the two
   refusal paths tied to it disappear rather than being softened.
3. **The store format breaks loudly.** `format_version` moves to 2, and a store at
   version 1 is refused by version with a message naming what it read and what it
   reads. No migration verb, no dual-key reader, no repair path — operator
   direction: "we're not writing a migration script, let alone legacy handler. blow
   up loudly."
4. **A minting verb.** `smut new` (name, count semantics and response shape are the
   design's) mints ids without storing anything, so a caller can author a connected
   graph — cycles included — in one declaration batch using names it did not invent.
   The unreserved-id race is accepted (ADR-001): a store that gains a minted id
   before the declaring batch turns that declaration into an upsert, expected never
   to occur.
5. **The binding records are reconciled, not silently contradicted.** Six accepted
   DECs carry the old spelling in their letter, and one of them reverses:
   DEC-005 defines an entity node as "a `Value::Object` with a member `id`" and
   states the consequence "an embedded object carrying `id` cannot be inline
   metadata" — which this slice makes *false*. DEC-007 (`id` first in entity
   serialisation, `select: []` yields `id` only), DEC-010 (the documentation set's
   version of the same claim), DEC-016 (`learn`'s recipe spells `select:["id"]` and
   `where:{"id":...}`), and the examples inside DEC-002 and DEC-012 all move. They
   are tier-2 binding, so this is a `/reconcile` obligation, not a find-and-replace:
   DEC-005 wants amending in substance, not just in spelling.
6. **The human surface and the fixtures follow.** `docs/store-format.md` (version,
   key, the id-first serialisation, and the diagnostic message strings that say
   `holds id "b"`), `smut learn`'s Declaring/References/Querying/Traversal text and
   its recipe example, the README (operator's in-flight edit — coordinate), the
   whole test suite (974 literal `"id"` occurrences across 27 files), and
   `tests/fixtures/interview/entities.json` (223 occurrences), which is a real
   store's declaration list and therefore doubles as the format-break evidence.

## Non-Goals

- **No migration tooling, of any kind.** Not a verb, not a dual reader, not a
  rewrite-on-write. An old store is refused, and its owner re-declares or scripts
  it themselves.
- **No rename verb** (IDE-002). This slice records that the minting verb drains the
  motivation, and leaves the idea deferred rather than implementing a graph-wide
  reference rewrite in the same change.
- **No envelope-vocabulary creativity.** `edges[].source`/`.target` are not renamed
  to anything identity-shaped, and no new envelope key is invented; one key moves
  (`changes[].id`) and one rule decides the rest.
- **The render short flags (IMP-007) are not part of this**, even though they touch
  `src/cli.rs`; a breaking format change is enough for one slice.
- **The internal uid/`basename` shape is untouched** — no length or alphabet change
  (IMP-006 settled that), and nothing about the store's file layout moves.
- **No `smut_id` alias or per-store alias**, and no compatibility shim for a caller
  who liked `id`.

## Affected surface

| Path | What changes |
| --- | --- |
| `src/entity.rs` | `ENTITY_ID` → the new key; `reference_id`, `hoist`, `scan_refs` read it; the nested-id refusal paths go |
| `src/declare.rs` | references, hoisting, `changes[]` subject key, `previous` |
| `src/store/*` | `FORMAT_VERSION` 1 → 2; entity-file validation (`IdDisagreement` reads the new key) |
| `src/walk.rs`, `src/select.rs`, `src/order.rs` | identity key in selection, edges, sort tiebreak (internal ids, mostly unchanged) |
| `src/render_markdown.rs` | identity display unchanged; any reference formatting that names the key |
| `src/cli.rs` | the minting verb's grammar; nothing else beyond the key |
| `src/learn.rs` | Declaring, References, Querying, Traversal, the two-call recipe, the command block |
| `docs/store-format.md` | format version, the key, id-first serialisation, diagnostic examples |
| `README.md` | every declaration and response example (operator's live edit — coordinate before sweeping) |
| `tests/**`, `tests/fixtures/interview/entities.json` | the sweep, plus the fixture store rewritten as evidence |

## Risks & assumptions

- **The fixture store is load-bearing.** `tests/fixtures/interview/entities.json` is
  a real 66-entity declaration list used by `tests/fixture.rs`. A mechanical rewrite
  must keep its *behavioural* expectations intact — the pinned envelopes in
  `tests/traversal.rs` and `tests/selection.rs` were written against these ids'
  order and referrals, so a rewrite that changes any id changes sort order and
  breaks pins that have nothing to do with this slice. The rewrite is therefore
  spelling-only: same ids, same values, only the key renamed.
- **Silence is the failure mode this slice must not add.** A missing field fails
  every predicate operator except `$exists`, so a caller left on `id` gets an empty
  result at exit 0 rather than an error. That argues for teaching the new name
  loudly in `learn` and for a test that a `where` on `id` finds nothing (documenting
  the consequence rather than hoping) — the design should decide whether that is a
  test or just a documented sharp edge.
- **Assumption:** nothing outside this repo consumes the format. Stated from the
  pre-1.0 position in the Context; if it is wrong, this slice is the wrong shape and
  the ADR must be revisited before implementation.
- **Risk:** the ADR is `proposed` pending a second review, and every phase below
  rests on it. Design must not start before it is accepted, or the decision is made
  by default.
- **Risk:** the README is under active edit by the operator. Sweeping it while it is
  being written produces a conflicted draft; sequencing has to be agreed before the
  docs phase runs.

## Verification / closure intent

- Red first, per behaviour: a nested `{"id":7}` stores as data (today exit 2); a
  nested `{"id":"x","name":"n"}` stays embedded and creates no entity (today it
  does); an entity file and every envelope object carry `smut_id`; a version-1 store
  is refused by version with a message naming both versions; `smut new` mints ids
  that a following batch can use for a cycle in one declaration.
- Green: `just check` — build, both clippy configurations at `-D warnings`,
  `cargo fmt --check`, both feature test runs.
- Agent-checked: a scripted store lifecycle at version 2 (declare → query → render →
  delete) with no `"id"` in any output; a version-1 store written by hand refused
  with exit 1 and the version message; the rewritten fixture renders identically
  apart from the key.
- Human-checked: `smut learn` reads true end to end, the README examples match the
  binary's output verbatim, and the DEC reconciliation in `docs/store-format.md`
  states the new rules rather than the old ones.
