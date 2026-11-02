# Identity is smut_id — the format break, and a loader that refuses what it cannot trust

## Context

ADR-001 moves the identity key
from `id` to `smut_id`. This slice is the change that follows from it, and it takes
the two items it governs — ISS-002 and IMP-005 — as one unit: the rename is a format
break, and a format break is the only honest moment to fix the loader validation and
corruption story at the same time.

- **ISS-002** — `id` is reserved in every nested object, so a payload's own `id`
  cannot be stored. Measured today: a nested `{"id":"abc","name":"Ext"}` silently
  declares an entity and strips the payload to `{"id":"abc"}`; a nested `{"id":"zzz"}`
  is a reference reported `missing`; a nested `{"id":null}` or `{"id":7}` refuses the
  whole batch at exit 2. The rename closes this for a caller's own `id` — while
  reserving `smut_id` at every depth, which the ADR states as a boundary rather than
  a disappearance.
- **IMP-005** — the loader validates the root and the root-id/index agreement only, so
a hand-edited store can make `scan_refs` panic (exit 101) on a nested
  `{"id":null}`. **Folded into this slice by operator direction ("fix alongside the
  loader/format change")**, and independently reproduced in RV-022 F-3. Under
  `smut_id` the loader must refuse malformed stored identity as corruption (exit 1)
  while accepting arbitrary ordinary `id` values in payloads.

Two neighbours are recorded and deliberately *not* in this slice:

- **ISS-003** (pre-minting ids) — operator direction: "settle its contract
  separately. It works under either identity spelling and gives two calls, one
  atomic graph write." Sequenced after, not behind.
- **IDE-002** (rename with referential stability) — **stays deferred, not resolved.**
  Generated ids reduce naming pressure; they do not retarget existing references or
  repair an entity already carrying a bad id. A future reference-preserving rename
  can be additive, so nothing here forecloses it.

Placing the change: pre-1.0, one real store in the wild
(`interview.local/.smut`, 66 entities, outside this repo) and one committed fixture
store. The operator requires no legacy-consumer compatibility: "there are essentially
no legacy consumers i care about, so fail clean and loud" (DEC-022). The break does
not depend on proving that no external consumer exists.

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
   refuse an ordinary nested id any more because identity parsing never looks at
   that key. Malformed structural `smut_id` remains invalid.
3. **The store format breaks loudly.** `format_version` moves to 2, and a store at
   version 1 is refused by version with a message naming what it read and what it
   reads. No migration verb, no dual-key reader, no repair path — operator
   direction: "we're not writing a migration script, let alone legacy handler. blow
   up loudly." Keep the existing version-diagnostic shape and exit 1, without
   migration advice. Check committed journal manifests by version before replay
   (DEC-022).
4. **The loader stops trusting a store it did not write.** `scan_refs`'
   precondition (every nested identity is a nonempty string) is established at load
   instead of assumed: a malformed stored identity is corruption, exit 1, naming the
   entry and an unambiguous field/index path — never a panic on a later traversal.
   Use `reference_id`'s rule in an eager recursive walk over each parsed entity,
   after its root checks; return `StoreError::Corrupt` and keep `scan_refs`
   non-fallible. The extra linear walk adds no file read or JSON parse (DEC-021).
   Ordinary payload `id` values are no longer identity at all.
5. **The binding records are reconciled, not silently contradicted.** Six accepted
   DECs carry the old spelling in their letter, and one of them reverses:
   DEC-005 defines an entity node as "a `Value::Object` with a member `id`" and
   states the consequence "an embedded object carrying `id` cannot be inline
   metadata" — which this slice makes *false*. DEC-007 (`id` first in entity
   serialisation, `select: []` yields `id` only), DEC-010 (the documentation set's
   version of the same claim), DEC-016 (`learn`'s recipe spells `select:["id"]` and
   `where:{"id":...}`), and the examples inside DEC-002 and DEC-012 all move. They
   are tier-2 binding, so this is a `/reconcile` obligation, not a find-and-replace:
   DEC-005 wants amending in substance, not just in spelling. Amend the affected
   passages in all six records in place during reconciliation, retaining their
   durable ids and unrelated rationale, with dated ADR-001/SL-004 amendment notes
   (DEC-025).
6. **The human surface and the fixtures follow.** `docs/store-format.md` (version,
   key, the id-first serialisation, and the diagnostic message strings that say
   `holds id "b"`), `smut learn`'s Declaring/References/Querying/Traversal text and
   its recipe example, the README (operator's in-flight edit — coordinate), the
   whole test suite (974 literal `"id"` occurrences across 27 files), and
   `tests/fixtures/interview/entities.json` (223 occurrences), which is a real
   store's declaration list and therefore doubles as the format-break evidence.
   Keep `smut-implementation-brief.org` normative and update its identity mentions,
   response examples and format-version example in place; do not archive it or
   extract specs in this slice (DEC-023). `docs/store-format.md` remains the single
   on-disk format record and must move with the change under STD-001. `ENTITY_ID`
   remains the single key definition, with a wire-contract assertion and the
   per-surface pins specified by DEC-020.

## Non-Goals

- **No migration tooling, of any kind.** Not a verb, not a dual reader, not a
  rewrite-on-write. An old store is refused, and its owner re-declares or scripts
  it themselves.
- **No pre-minting verb** (ISS-003). Its contract is settled separately and it is
  independent of the key spelling; including it would make this slice two changes.
- **No rename verb** (IDE-002), which stays deferred rather than resolved: generated
  ids reduce naming pressure and repair nothing that already exists.
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
| `src/entity.rs` | `ENTITY_ID` keeps its name and takes value `smut_id`; structural identity validation uses that key; ordinary `id` is data |
| `src/declare.rs` | references, hoisting, `changes[]` subject key, `previous` |
| `src/conventions.rs` | fixed-id JSON declaration emits the structural identity key; seed id and content stay unchanged |
| `src/store/*` | `FORMAT_VERSION` 1 → 2; entity-file validation (`IdDisagreement` reads the new key); the loader's nested-identity validation (IMP-005) |
| `src/walk.rs`, `src/select.rs`, `src/order.rs` | identity key in selection, edges, sort tiebreak (internal ids, mostly unchanged) |
| `src/render_markdown.rs` | identity display unchanged; any reference formatting that names the key |
| `src/cli.rs` | nothing beyond the key: the grammar does not change |
| `src/learn.rs` | Declaring, References, Querying, Traversal, the two-call recipe, the command block |
| `docs/store-format.md` | format version, the key, id-first serialisation, diagnostic examples |
| `smut-implementation-brief.org` | normative identity rules, response examples and format-version example updated in place |
| `README.md` | every declaration and response example (operator's live edit — coordinate before sweeping) |
| `tests/**`, `tests/fixtures/interview/entities.json` | the sweep, plus the fixture store rewritten as evidence |

## Risks & assumptions

- **The fixture store is load-bearing.** `tests/fixtures/interview/entities.json` is
  a real 66-entity declaration list used by `tests/fixture.rs`. A mechanical rewrite
  must keep its *behavioural* expectations intact — the pinned envelopes in
  `tests/traversal.rs` and `tests/selection.rs` were written against these ids'
  order and referrals, so a rewrite that changes any id changes sort order and
  breaks pins that have nothing to do with this slice. The rewrite is therefore
  spelling-only: same ids, same values, only the key renamed. Verify this against
  the original parsed fixture by reversing only the identity-key rename; preserve
  array order and all values (DEC-024). Search residual `id` uses and classify them
  by meaning; ordinary payload `id` and container names must remain valid, so
  there is no blanket zero-match gate.
- **Silence is the failure mode this slice must not add.** A missing field fails
  every predicate operator except `$exists`, so a caller left on `id` gets an empty
  result at exit 0 rather than an error. That argues for teaching the new name
  loudly in `learn` and for a test that a `where` on `id` finds nothing (documenting
  the consequence rather than hoping) — the design should decide whether that is a
  test or just a documented sharp edge.
- **Compatibility policy:** external consumers may exist; maintaining their old
  store format is explicitly outside this slice under ADR-001 and DEC-022.
- **Risk:** the README is under active edit by the operator. Sweeping it while it is
  being written produces a conflicted draft; sequencing has to be agreed before the
  docs phase runs.
- **Risk:** IMP-007 (render short flags) is explicitly optional and must not ride
  along; it touches `src/cli.rs`, which this slice is otherwise leaving alone.

## Verification / closure intent

- Red first, per behaviour: a nested `{"id":7}` stores as data (today exit 2); a
  nested `{"id":"x","name":"n"}` stays embedded and creates no entity (today it
  does); an entity file and every envelope object carry `smut_id`, `changes[]`
  included; a version-1 store is refused by version with a message naming both
  versions; a hand-edited store carrying a nested `{"smut_id":null}` is refused as
  corruption at exit 1 rather than panicking on the next traversal (IMP-005).
- Green: `just check` — build, both clippy configurations at `-D warnings`,
  `cargo fmt --check`, both feature test runs.
- Agent-checked: a scripted store lifecycle at version 2 (declare → query → render →
  delete) carrying structural identity as `smut_id` while ordinary payload `id`
  survives as data; a version-1 store refused with exit 1 and the version message;
  an old committed journal manifest refused by version before replay; the RV-022
  F-3 probe re-run — hand-edit nested `smut_id` to `null`, then both `get` and a
  `follow` traversal refuse corruption with exit 1, file and path, without a panic;
  fixture equivalence plus existing fixture/example/regression suites establish
  that graph content and unrelated behavior are preserved (DEC-024).
- Human-checked: `smut learn` reads true end to end, the README examples match the
  binary's output verbatim, the normative brief states the new identity/version
  rules while retaining unfinished 1.0 requirements, and the six DEC amendments
  and `docs/store-format.md` state the agreed rules.
