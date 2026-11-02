# A predicate that cannot hold is refused

## Two rules

**Decidability.** One field's condition list is checked after parsing and before any
store access. If the conditions *bound* the field — to one value, or to a finite set —
the question is decidable by evaluation, and a false answer is refused by name:

    smut: field status cannot be satisfied: $eq and $ne are given the same value

| what the conditions bound the field to | how it is decided |
| --- | --- |
| nothing, with `$exists:false` beside anything else | absence and presence cannot both hold |
| one value — `$eq v` present | evaluate every other condition on `v` |
| a finite set — `$in S` present | some member of `S` must satisfy every other condition |
| nothing at all — neither `$eq` nor `$in` | nothing is provable |

**Position.** A proof about one field is not yet a proof about the query. The check is
made only where the predicate **demands** the field:

- the root is demanded;
- `And` passes demand to every child — a conjunct must hold if the `And` must;
- `Not` flips demand;
- `Or` determines its children only when the `Or` is itself demanded *false*;
- a `Field` is judged iff it arrives demanded.

So a contradiction is refused at the root, under `$and`, and under any even number of
`$not`s **provided no `$or` interrupts the path**. It is not refused inside an `$or` that
must hold, nor under an odd number of `$not`s, because the query still has an answer
there. Measured on 0.2.0: `{"$not":{"status":{"$eq":"open","$ne":"open"}}}` returns the
whole store, and `{"$or":[{…contradiction…},{"type":"question"}]}` returns rows. Refusing
either would reject a correct query.

Parity alone is *not* the rule, and this is worth stating because the shorthand invites
the error: `{"$or":[{"$not":{"$not":{"f":{"$eq":1,"$ne":1}}}},{"g":2}]}` wraps the
contradiction in two `$not`s and is still satisfied by any `g: 2` entity, because the
`$or` was never analysed. `$or` contents are not analysed at all — which also means a
one-element `$or` around a contradiction is accepted, even though a one-element `$or`
means what its disjunct means. Closing that one shape would imply the rest of `$or` is
analysed; it is not, and `{"$or":[]}` staying legal (false, selecting nothing) is part of
the same decision.

Because a condition object carries each operator at most once (a repeated key is refused
in any object, at any depth, as `src/input.rs` reads it; DEC-001 rule 1 makes that exit
2), a field has at most five conditions and the decidability test is four steps, not a
lattice. A `Field` always carries at least one condition — a value without `$` keys
parses to one `Eq`, and a `$`-only object yields one condition per key — so a judged
field is never vacuously true and a *not*-demanded field never needs a refusal.

## The branches

Seven, and every one is false for every store:

| branch | why it cannot hold |
| --- | --- |
| `$in []` | no value is a member of nothing |
| `$exists:false` beside any other condition | absent fails the other; present fails `$exists:false` |
| `$eq v` with `$ne v` | one value, two incompatible requirements |
| `$eq v` with `$in` not holding `v` | the pin excludes every listed value |
| `$eq v` with a `$contains` that `v` does not satisfy | the pin fixes the value the test applies to |
| `$ne v` with `$in` all of whose members equal `v` | every listed value is excluded |
| `$contains c` with `$in` where no member satisfies `c` | the list is the whole candidate set, and none survives |

Nothing else is checked, and a contradiction spanning `$and` clauses is out of scope.

## Why refuse rather than diagnose

The alternatives were a diagnostic member on the envelope and a warning on stderr.
Both are worse than the mechanism already present:

- **stderr is failure-only today** — one call site, on the error path — and the
  brief's own rule for rendering is that incompleteness must not be hidden in
  stderr alone, i.e. advisory content belongs in the payload. A success-path
  warning would also break `stderr.is_empty()`, asserted on success in nine places.
- **a top-level envelope member amends DEC-019**, whose pin is "the five 0.1
  members plus exactly one conditional member". That is a far larger decision than
  this friction justifies, and it answers a complaint about envelope width with more
  envelope.
- **a refusal reuses the path the store already has**, costs one variant and one
  message, and cannot widen anything.

## The rationale, corrected

`MixedPredicateObject` refuses a value mixing literals and operators *because the
design gives it no behaviour*. A false conjunction does have behaviour — false — so
this refusal needs its own rationale rather than borrowing that one: **a condition
that cannot hold is a typo, not a question.** Two classes, each stated once, is
honest; stretching one rationale over both is not.

## The two neighbours, decided the other way

- **A `where` naming a field that no scanned root carries: teach, never refuse.**
  Filtering by a field is how a caller discovers whether a store carries it, and
  refusing would punish that. The check is also only sound when the roots are
  store-wide; with `from`, two roots saying "no such field" is weak evidence about the
  other thirty-two. A diagnostic that fires inconsistently is worse than silence.
  The cure is a guide clause: a field no entity carries fails every operator exactly
  as a missing field on one entity does.
- **A declaration carrying `kind` but no `type`: neither refused nor warned.** Its
  cause is delivery of advice, cured by the conventions surface. If it is ever
  revived it is an advisory inside `changes[]` — where `previous` already lives —
  and never stderr.

## What this commits us to

- **The guide must scope the guarantee**, naming the silences that remain:
  `$contains` with neither `$eq` nor `$in` beside it; anything outside demanded
  position; and the fact that `$or` contents are never analysed.
- **Runtime-composed predicates move from exit 0 to exit 2.** A caller templating
  `$eq`/`$ne` from variables can synthesise a contradiction deliberately, and a
  computed `$in` list that happens to be empty now refuses. `from: []` remains the
  documented spelling for an intentional empty result.
- **`$and`-spanning contradictions stay silent**, by design rather than by oversight.

## Compatibility

Nothing here reaches a 0.1 caller: `where` is a 0.2 key, so no 0.1 query can express
a predicate at all. DEC-019's additivity property — an unchanged 0.1 response — is
untouched.

## Provenance

Reached in five corrections, kept because each names a defect a reader would otherwise
reintroduce: an enumeration of six forms, two of which could not be written; a rule that
called `$contains` undecidable whenever `$eq` was absent, although `$in` also bounds the
value; a position-blind per-field check that would have refused contradictions inside
`$or` and under a single `$not`; a "crossed `And` only" rule that missed the
double-negated shape; and a parity shorthand that is false wherever an `$or` interrupts
the negations. Demand propagation is what all five reduce to.
