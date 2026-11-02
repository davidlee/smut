# A predicate refusal names the defect and the position

Two refusals on one parse region, settled together in SL-005 because they are one
message surface.

## The contradiction refusal

    smut: field status cannot be satisfied: $eq and $ne are given the same value

- **One variant.** `Reject::ContradictoryFieldConditions { field, contradiction }`,
  one `Display` arm, one row in the failure boundary table. The detecting branch
  supplies the description.
- **Seven sentences**, one per reachable branch of the rule DEC-027 states, each naming
  the two operators or the empty list it is about.
- **No epigram.** DEC-027's *a condition that can never hold is a typo, not a
  question* stays in that decision and in the guide sentence IMP-012 owes. stderr is
  one line, so a parenthetical would be repeated on every instance forever.

## The predicate-level `$` refusal is two messages, not one

The same `Display` arm was reached from the predicate level and from inside a condition
object, so a single sentence could not be true of both.

    smut: predicate operator $ne belongs inside a field's condition object: {"<field>": {"$ne": <operand>}}
    smut: unknown predicate operator: $foo

- **`MisplacedPredicateOperator { operator }`** — the predicate level, when the name is
  one of the five operators. It is not unknown; it is in the wrong place, and the
  sentence says where it belongs. A parenthetical about position would have been false
  inside a condition object, and the shape it recommends would itself be refused there.
- **`UnknownPredicateOperator { operator }`** — unchanged, at both levels, for a name
  the language does not define. `$foo` really is unknown; so is `$and` used as a
  condition, which is a clause and not an operator.
- **A shape, not a field.** A predicate-level `$` key carries no field, and the one
  thing the original proposal did wrong was print `status` — a field the caller may not
  have. The placeholder states that no field is being named.
- **It must read correctly for three slips.** The same arm catches an operator beside a
  field, an operator alone with no field at all, and an operator beside a clause. The
  chosen sentence does, because it describes where an operator belongs rather than what
  it sits next to.
- **`MixedPredicateObject` is untouched.** The field-level mixing case names its field
  and states its own cure; these messages must not converge.
