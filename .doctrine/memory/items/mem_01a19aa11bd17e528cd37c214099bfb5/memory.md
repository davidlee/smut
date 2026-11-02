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


## The same literal can go stale by the plan's own design

A keyword with signal today can be *false by design* tomorrow. SL-005 pinned PHASE-01/VT-2
on `[(Error, u8); 25]`, and PHASE-02/EX-2 mandated the same array move to 26 — one array
at two lengths, so the first phase's keyword became unsatisfiable the moment the second
landed, and `verify-vt` exited 1 on an otherwise fully green, fully complete slice.

A mutable count or literal is only a safe keyword if no later phase of the same plan
changes it. Otherwise pin something the change *introduces* rather than something it
*edits*: the variant name, a whole pinned line, a test function name. When the gate goes
red after the final phase with every test green, check the plan's own cross-phase
keywords before suspecting the code.
