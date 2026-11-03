# IMP-016: The five predicate operators are enumerated in two places; one `Operator` parser consulted by both sites would collapse the drift class

Found by the pre-close code review of SL-005 (RV-027 F-2), 2026-11-02.

The closed operator set is written twice in `src/predicate.rs`, independently:

- `:94` — inside the `operator if operator.starts_with('$')` arm of `parse_predicate`:
  `"$eq" | "$ne" | "$in" | "$exists" | "$contains"`, which decides *misplaced* rather than
  *unknown*;
- `:193-204` — `parse_condition`'s five match arms, whose catch-all at `:205` raises
  `UnknownPredicateOperator`.

The only coupling is a pair of cross-referencing comments (`:88-92`, `:189-191`) that
SL-005's PHASE-02/EX-1 credits with the property "a sixth operator cannot be added to one
and not the other". Comments cannot enforce that, and no test reads both lists.

Drift symptom: add `$regex` to `parse_condition` alone and
`{"where":{"$regex":"x"}}` renders `smut: unknown predicate operator: $regex` — a false
sentence about an operator the language defines, which is precisely the defect RV-026 F-2
and F-9 removed at this seam for the five current operators.

A const cannot fix this: `parse_condition`'s arms each carry different operand handling
(`$in` needs an array, `$exists` a boolean, the rest clone), so a string list consulted only
by the predicate-level arm closes the *removal* half of the drift and leaves the *addition*
half open. The shape that closes both:

```rust
enum Operator { Eq, Ne, In, Exists, Contains }

impl Operator {
    fn parse(name: &str) -> Option<Operator> { /* the one list */ }
}
```

`parse_predicate` then reads `Operator::parse(operator)` — `Some` is misplaced, `None` is
unknown — and `parse_condition` matches on the enum instead of the string. One list, no
comment required, and the compiler enforces that every arm handles every variant.

Deliberately not done inside SL-005: PHASE-02/EX-1 fixed the two-arm shape, the risk is
latent (no sixth operator exists), and redesigning a module the slice just settled is scope
creep. Related: RV-027 F-2.

Widened by SL-010 (RV-034 F-6, 2026-11-03): `src/dsl.rs` is the third site that spells the
predicate operators. `OPERATORS` carries `!=`/`=`/`~` mapped to `$ne`/`$eq`/`$contains`,
and `EXISTS`/`IN` name the two the flags supply (`$exists` through `--has`/`--missing`,
`$in` through a comma list on `=`). It derives from neither of the other two sites on
purpose — the DSL's spellings are its own vocabulary and the JSON names are the emit
target, so collapsing them into `Operator::parse` would put a user-facing spelling table
inside the predicate layer. That makes a sixth operator a three-site change now, not two,
and the `//!` header says so.
