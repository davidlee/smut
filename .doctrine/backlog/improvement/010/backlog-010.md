# IMP-010: A provably contradictory field condition answers with the empty set and says nothing

**Decided — DEC-027.** This item is now the implementation of a settled refusal
class, not an open question. The sound subset is fixed there; this is the work order.

## What was measured

The 0.2.0 rc trial's first friction, and the reporter was careful about it:

> `{"status":{"$eq":"open","$ne":"open"}}` is a legal ANDed predicate, returns
> `[]`, says nothing. Every other malformed input gets a named error; this one is
> a well-formed question with a known-false answer. It's not wrong — it's
> *unhelpful* at exactly the point where an agent is most likely to have made a
> typo.

```
❯ smut query '{"where":{"status":{"$eq":"open","$ne":"open"}}}'
{"dangling":[],"missing":[],"objects":[],"truncated":false,"truncation_reasons":[]}
(exit 0)
```

## What the decision fixes

Refuse at parse time, before any store access, when **one field's condition list** is
provably unsatisfiable — six forms, all false for every store:

- `$exists:false` with any other condition on that field;
- `$exists:true` with `$exists:false`;
- `$eq v` with `$ne v`;
- `$eq v` with `$in [...]` not holding `v`;
- `$in []`;
- `$in A` with `$in B` whose intersection is empty.

Nothing else is checked: `$contains` is undecidable, and a contradiction spanning
`$and` clauses is out of scope by design.

## What the implementation needs

- **One `Reject` variant** naming the field and the contradiction, in the shape
  `MixedPredicateObject` already uses — it names the field and says the fix
  ("compare the whole object with `$eq`"). The new message should do the same: what
  is impossible, and what to write instead. The rationale it may cite is DEC-027's
  own — *a condition that cannot hold is a typo, not a question* — not
  `MixedPredicateObject`'s "no behaviour", which does not apply to a false
  conjunction.
- **The check lives in `parse_field`** (`src/predicate.rs`), after the conditions are
  built and before they are returned. It reads only the condition list, so the
  predicate module keeps its no-store, no-clock property.
- **Tests, red first**: one per form, a positive control that `$eq v` with
  `$in [v]` is *accepted*, another that `$contains` combinations are accepted, and
  a pin asserting the `$and`-spanning case is **not** refused — so nobody later
  "fixes" it into a silent behaviour change.
- **No envelope change and no stderr.** This closes `RFC-001`'s diagnostic-channel
  question: no member, no success-path warning, DEC-019 untouched.
- **Compatibility is free**: `where` is a 0.2 key, so no 0.1 caller can express a
  predicate, and DEC-019's "unchanged 0.1 response" property cannot move.

## Risks recorded with the decision

- **A partial detector can breed false assurance.** The guide must scope the
  guarantee (IMP-012), or "no error" becomes a new wrong belief replacing an old
  silence.
- **Runtime-composed predicates change behaviour**: a caller templating `$eq`/`$ne`
  from variables can synthesise a contradiction and now gets exit 2 instead of `[]`.
  `from: []` remains the documented spelling for an intentional empty result.

## Not in this item

- Any diagnostic for a `where` naming a field no entity carries — taught, never
  refused (IMP-012).
- A `kind`-without-`type` warning at `declare` — delivery, not diagnostics
  (IMP-011); if ever revived, an advisory inside `changes[]`, never stderr.
- `$and`-spanning contradictions.
- Any envelope member or success-path stderr.

## Neighbours

**ISS-004** is the other refusal message on the same surface — a `$` key in the wrong
position — and both are one-message changes in the same parse region; they should be
done together rather than touching refusal text twice. **IMP-012** owns the two
sentences the decision obliges. **DEC-027** is the decision.

## Provenance

0.2.0 rc trial, friction 1 (the reporter's framing, "not wrong, unhelpful", is the
premise); the sound subset and the consistency argument came from the assessment that
preceded it; the three-store exercise supplied the absent-field neighbour and the
stderr contract cost. Settled 2026-11-02 — see DEC-027.
