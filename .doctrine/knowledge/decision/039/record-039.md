# DEC-039: The DSL refuses only what JSON cannot spell

IDE-006 carried a rule refusing duplicate fields in the DSL, with a suggested cure: *write
`status=open,resolved` for either*. The research round found that the tree already owns most of
that ground, and says it better.

`predicate::parse` refuses a field condition list that is **provably unsatisfiable**, at parse
time, before any store access — by *demand* (where in the predicate the field is required) and
*decidability* (`$eq`, `$in` and `$exists:false` bound the field to something evaluable). DEC-030
pins the wording to one sentence: `field status cannot be satisfied: $eq and $ne are given the
same value`.

So `smut q status=open status!=resolved` needs no DSL rule: compile it, and the existing mechanism
refuses it in the pinned voice. What remains is the **one shape the specification cannot spell** —
two `=` on one field, since a JSON object has one key per field. That shape is also unsatisfiable
(two different values cannot both hold), so it belongs in the same family, refused by name.

The suggested cure in IDE-006 was half-right, which is why the rule retires rather than gets
reworded: `status=open,resolved` expresses *either*, and is only the right advice when both
conditions were `=`. Compiling two `=` into `$in` would answer a question nobody asked, silently.

Every DSL-only refusal — an ambiguous token, a JSON operand, an unknown flag, `--limit all`, a
comma list a `!=` cannot carry — exits 2, names the defect, and names only flags and fields the
caller actually typed (DEC-030).