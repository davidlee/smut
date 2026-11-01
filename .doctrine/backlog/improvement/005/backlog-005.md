# IMP-005: Store loader refuses a malformed nested id instead of letting a traversal read panic

Found at the SL-002 audit (`doctrine show RV-021`, finding F-7), from the
external GPT code review the operator ordered before reconcile; reproduced
independently at the audit seat. Disposed `follow-up` because the fix sits behind
the slice's §14.2 fence and the refusal it needs is not in the design.

## What happens

A store whose entity file carries an object with an `id` that is not a nonempty
string — `{"bad": {"id": null}}`, `{"bad": {"id": 5}}`, `{"bad": {"id": {}}}` —
is loaded without complaint by the store loader, and then **panics** the first
time anything scans that entity for references:

```
thread 'main' panicked at src/entity.rs:93:28:
scan_refs requires a validated entity: an `id` that is not a nonempty string is corruption
```

exit 101. Reproduction: declare the interview fixture into a scratch store, add
the offending key to one entity's file (the file the index maps to that id), then

```
smut query '{"from":"a-focus","follow":["about"]}'
```

The same store answers `get a-focus` (exit 0) and `query '{"from":"a-focus"}'
(exit 0) — only the traversal read path reaches `scan_refs` on a stored entity.
`src/select.rs`'s induced-edge derivation reaches it too, since edges are derived
by scanning returned entities.

## Why it is a 0.2 defect rather than a 0.1 one

At `b47dd79` the only `scan_refs` caller was `src/declare.rs:220`, which scans a
*declaration* — and a declaration carrying an invalid id is refused at exit 2
(`invalid entity id: null, and an id must be a nonempty string`). So 0.1 never
scanned a stored entity and this store shape was inert. The walker made it
reachable: the malformation was always there, and 0.2 is the first release that
looks at it.

## Why it was not fixed in 0.2

- **Not reachable through the CLI.** No declaration produces it; `declare`
  refuses every variant. It needs a hand-edited store, which README calls
  unsupported.
- **The panic is deliberate** — its message names corruption — and panic on an
  invariant breach is an accepted style here (`walk::stored_entity`, and the
  recorded decision that `depths()` panics rather than omit a returned object).
- **Every plausible home is fenced or unspecified.** The honest fix is the loader
  refusing the shape (`src/store/mod.rs`, or `src/entity.rs`), both inside
  SL-002's §14.2 "unchanged" fence; a refusal in the callers instead
  (`src/walk.rs`, `src/select.rs`) needs a refusal shape the design does not
  specify, since §10.3's 32 resolved edge cases contain no rule for a malformed
  nested id.
- It is not a data-loss or durability issue: the failing path is read-only, and
  the store is untouched.

## What the fix owes

1. A malformed nested `id` is refused at load, as exit 1 with the store-format
   document's corruption posture — "every inconsistency listed under Corruption
   is refused, never reconstructed" — rather than reaching a reader as a panic.
2. An assertion in `tests/store_corruption.rs`, which today covers malformed
   JSON, a non-object entity and top-level id disagreement and nothing below the
   top level.
3. A line in `docs/store-format.md`'s Corruption section naming the rule, so the
   document and the loader agree — that document also still speaks in 0.1's voice
   (see RV-021 F-5), so both belong to the same pass.

Likely owner: whichever release next touches the store loader or the store
format document (`IMP-002` names documentation and hardening). Not urgent: no
CLI-only store can hold this shape.
