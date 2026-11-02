# Key a VT row on the shape that only exists after the change

`doctrine slice verify-vt` checks VT `keywords` as plain substring presence over the raw
source of the row's `test_file` — no comment stripping, no parsing, no execution. A row
therefore passes vacuously when its keyword is a token the file already contains.

Real examples from SL-005's first plan draft, all inert:

- `INVALID_INPUT` — already at `src/error.rs:14`, so it could not see a missing
  boundary-table row;
- `has_store` — already in `tests/predicates.rs`, so it could not see a missing new test;
- `$contains` and `UnknownPredicateOperator` — already in `src/predicate.rs`, so they
  could not see the predicate-level arm being split.

What has signal: a name the change introduces (`MisplacedPredicateOperator`), a literal
the change edits (`[(Error, u8); 25]`), a whole pinned line, or a test function name.
Prefer the narrowest string that is false today and true afterwards, and verify that by
grepping the tree before the phase runs, not after.

Corollary for reviewers: a VT mandate is a floor, not evidence of behaviour. A green
`verify-vt` says the strings are present; only the test run says the behaviour holds.
