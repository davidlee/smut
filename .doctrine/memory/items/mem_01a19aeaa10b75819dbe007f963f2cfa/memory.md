Measured 2026-11-02 on a probe store, binary 0.2.0, while designing a conventions set
(SL-006). Four records, one field holding a string in one entity and arrays in others.

- `{"field": "some text"}` (literal = `$eq`) matched only the string row. An array row
  is not equal to a string, so the same question needs a different operator per shape.
- `{"field": {"$eq": {"id": "x"}}}` matched nothing when the field is an array —
  the field is the array, not the object. `{"$contains": {"id": "x"}}` matched it.
- **`$contains` changes meaning with the value's shape**: substring test on a string,
  exact-element test on an array. `$contains: "trev"` matched a string row containing
  "trev" and an array row whose element is exactly "trev", but NOT an array row whose
  element is "trev from the disco". One operator, two semantics, no signal in the
  response about which ran. This is a silent-wrongness class, not an error class.
- `order` refuses such a field outright: "an array has no ordering, and a sort value
  must be null, a boolean, a number or a string". So a field that is sometimes scalar
  can never be sorted.

Related trap: a reference object carrying extra fields does not annotate the edge — it
*declares* the entity. `{"id": "pty-cfo", "role": "chair"}` inside a reference array
silently added `role: chair` to `pty-cfo` and left a plain reference behind. There is no
spelling of an annotated edge (backlogged as IDE-004).

Consequence for any conventions set: prescribe one shape per field per store, and keep
prose out of reference-shaped fields (a prose name in an actor field is the same trap).
The measured detail is also appended to IMP-012 (what `learn` should teach about the
surface's limits) and is what makes IDE-004 worth thinking about.
