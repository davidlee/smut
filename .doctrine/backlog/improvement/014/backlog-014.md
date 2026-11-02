# IMP-014: `test` coverage of the `Reject` taxonomy is hand-maintained: no `every_reject_refusal` analogue, one grab-bag message test

Found by the pre-close code review of SL-005 (RV-027), 2026-11-02.

`src/error.rs` has two invariants that the compiler does not enforce and that tests only
partly cover.

1. **The exit-code boundary table is hand-maintained.** `the_boundary_maps_each_failure_class_to_its_exit_code`
   is a fixed-size array (`[(Error, u8); N]`) whose length a human bumps by one per new
   variant. The compiler enforces exhaustiveness for the `Display` impl only, never a row,
   so adding a `Reject` variant without a row compiles and ships with no exit code
   assigned. PHASE-01/EN-1 of SL-005 states this explicitly and relies on the phase's own
   EX-3 row as the only guard — i.e. the guard is a human remembering. The `StoreError`
   side has the opposite arrangement: `every_store_refusal()` (`error.rs:469`) enumerates
   the variants so the table can be built from it.
2. **`a_rejection_names_what_was_wrong` is a ~150-line grab-bag** over every `Reject`
   variant's message. SL-005 added ~25 more lines to it (the two misplaced-operator pins)
   because PHASE-02's task list permitted "or a sibling"; a sibling test was chosen here for
   the misplaced message, but the existing body keeps growing.

Wanted: a `every_reject_refusal()`-style enumeration that makes a missing row a compile or
test failure rather than a human obligation, and a split of the message test along the same
seams the enum already has (predicate refusals, input/spec refusals, identity refusals), so
each message pin sits beside the variant it describes.

Out of scope for SL-005 (its EX criteria fix three selectors and no test restructuring) and
for SL-004 (identity respell).
