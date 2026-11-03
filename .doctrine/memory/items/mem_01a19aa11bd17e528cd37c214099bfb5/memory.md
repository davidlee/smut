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

## The co-tenant variant — a neighbour's number is never yours to pin

SL-008 PHASE-02 pinned `[(Error, u8); 27]` on `src/error.rs`. Nothing in that plan moved the
length; SL-010, a **different slice** sharing the same file, added six variants. The row went
red on a fully green, fully complete slice, and the red said nothing about the code PHASE-02
owned.

Rule: a VT keyword must be a presence check on a **fact the phase owns**, never on a **number a
neighbour owns**. Pin the fragment — `[(Error, u8);`, the variant name, the test function name, a
whole pinned line — rather than a count or literal another writer's plan can move. When two
slices interleave in one file, assume every count in it belongs to someone else.

The same defect one level up is a **criterion** whose literal is stricter than its truth:
SL-008 PHASE-01/EX-5 said the `.config/smut` segment is joined by `Roots::under` "and nowhere
else", while EX-2's own text mandates a second join in `user_root`'s home fallback. The
criterion sentence was the defect, not the code — and a reviewer following the sentence literally
would have "fixed" correct code.
