# ISS-004: A top-level `$` key in a predicate is refused as an unknown operator, which sends the reader to the wrong list

Found in the 0.2.0 rc trial. The operator wrote

```
❯ smut query '{"where":{"status":"open","$ne":"x"}}'
smut: unknown predicate operator: $ne
(exit 2)
```

and concluded from it that smut does not know `$ne` — then proposed the cure for
the *field-mixing* refusal instead. That misreading is the defect: `$ne` is a
defined operator, and nothing is wrong with the operator list. What is wrong is
its **position** — an operator lives inside a field's condition object, not
beside the field.

## The field-level mixing case is already right

`Reject::MixedPredicateObject` exists and names the field (`src/predicate.rs`,
`parse_field`):

```
❯ smut query '{"where":{"status":{"literal":true,"$exists":true}}}'
smut: field status mixes a literal with operator keys; compare the whole object with `$eq`
```

So there is no field-level defect to fix — this is what the reporter asked for,
and it is already there. The misdirection is one arm in `parse`: a `$`-prefixed
key at the predicate's top level (`src/predicate.rs:71`), where the name is
genuinely not an operator and the message says only that.

## What a cure looks like

A sentence and a shape, no new refusal class and no behaviour change:

```
smut: unknown predicate operator: $ne, and an operator belongs inside a field's
condition object: {"status":{"$ne":"x"}}, not beside the field
```

Constraints that make it more than a string edit:

- **The name stays.** `$foo` really is unknown, and that half is pinned
  (`src/error.rs` tests assert the offending name appears). A better message
  must not trade the true statement for the helpful one.
- **No field can be named.** `{"status":"open","$ne":"x"}` has a literal under a
  field and an operator as its sibling; there is no single field whose
  condition object is malformed. The message can show the shape, not a name.
  The literal-beside-operator case *inside* a field stays
  `MixedPredicateObject`, which has a field to name.
- **The guide already states the rule** (`smut learn` §Querying: "one with only
  `$` keys is an operator object and its operators are ANDed"). The fix is a
  refusal that points at that sentence. A guide change is optional and, if made,
  one clause.

## Neighbours

IMP-010 (a provably contradictory condition answered silently) is the same
predicate surface with the opposite fault: there a well-formed predicate is
answered too quietly, here a plain typo is refused too vaguely. Neither blocks
the other.

## Provenance

0.2.0 rc trial, friction 2. The reporter's diagnosis — "the field mixes a
literal with operators" — is wrong for their input and right for a different
one; both cases are measured above, on `target/debug/smut` 0.2.0.
