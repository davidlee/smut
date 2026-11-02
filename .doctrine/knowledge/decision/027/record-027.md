# A predicate that cannot hold is refused

## The shape

One field's condition list is checked after parsing and before any store access.
If it is provably unsatisfiable, the request is refused by name, exit 2, like any
other malformed input:

    smut: field status cannot be satisfied: $eq and $ne are given the same value
    (a condition that can never hold is a typo, not a question)

Six forms are decidable on the condition list alone, and all six are false for
every store:

| form | why it cannot hold |
| --- | --- |
| `$exists:false` with any other condition on the field | absent fails the other; present fails `$exists:false` |
| `$exists:true` with `$exists:false` | contradictory by construction |
| `$eq v` with `$ne v` | same structural value, both ways |
| `$eq v` with `$in [...]` not holding `v` | structural equality again |
| `$in []` | no entity can be a member of nothing |
| `$in A` with `$in B`, intersection empty | no value satisfies both |

Nothing else is checked. `$contains` is undecidable (a substring test), and a
contradiction spanning `$and` clauses is out of scope.

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
  store-wide; with `from`, two roots saying "no such field" is weak evidence about
  the other thirty-two. A diagnostic that fires inconsistently is worse than silence.
  The cure is a guide clause: a field no entity carries fails every operator exactly
  as a missing field on one entity does.
- **A declaration carrying `kind` but no `type`: neither refused nor warned.** Its
  cause is delivery of advice, cured by the conventions surface. If it is ever
  revived it is an advisory inside `changes[]` — where `previous` already lives —
  and never stderr.

## What this commits us to

- **The guide must scope the guarantee.** A partial detector plus silence about its
  limits turns "no error" into false assurance. The guide says the check covers one
  field's conditions and nothing across `$and`.
- **Runtime-composed predicates move from exit 0 to exit 2.** A caller templating
  `$eq`/`$ne` from variables can synthesise a contradiction deliberately; it now
  handles a refusal. `from: []` remains the documented spelling for an intentional
  empty result.
- **`$and`-spanning contradictions stay silent**, by design rather than by oversight.

## Compatibility

Nothing here reaches a 0.1 caller: `where` is a 0.2 key, so no 0.1 query can express
a predicate at all. DEC-019's additivity property — an unchanged 0.1 response — is
untouched.
