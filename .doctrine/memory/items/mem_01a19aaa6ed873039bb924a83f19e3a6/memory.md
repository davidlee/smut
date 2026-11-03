`doctrine slice verify-vt <ID>` resolves each VT's `test_file` against the phase's source delta recorded in the slice's arm-neutral registry, not against the working tree or a raw `git diff`. That row is normally upserted by the `doctrine slice phase ... --status completed` transition (which captures `code_end_oid`), so running the gate while the phase is still `in_progress` reports every VT as `UNATTRIBUTABLE — keyword present but <file> not modified by this slice`, however green the work is.

Observed 2026-11-02 on SL-005 PHASE-01: all four VTs unattributable after the code commit, all four `PASS` immediately after `doctrine slice record-delta 5 PHASE-01 --commit 7347dff` (the safe mode, which records exactly that commit's own patch). The later completion transition upserts the same range, so the manual step is harmless.

Remedy order for a solo in-tree phase: implement → commit the code → `record-delta --commit <S>` → `verify-vt` → flip the phase to `completed`. Sibling fact `mem.fact.doctrine.vt-inert-keywords` covers the other way a VT can pass or fail vacuously (keywords already in the tree).

## Correction — the completion transition recomputes, so correct the row *after* it

Observed on SL-008 (2026-11-02). The last line above ("the later completion transition upserts
the same range, so the manual step is harmless") holds only while the recorded row already is
the range the transition would compute. It is not a superset guarantee:

- `doctrine slice phase … --status completed` **recomputes** the phase's delta and **overwrites**
  the registry row. A range narrowed beforehand with `record-delta --commit <S> --force` survives
  only if the recomputation lands on the same span; SL-008 PHASE-02 and PHASE-03 were both bitten
  by an auto-span that swept a co-tenant slice's interleaved commits into the row.
- `--force` is only meaningful with `--commit` (ISS-317). `--start/--end` is never guarded, so
  naming both ends deliberately is the way to correct a span without a fight.

Revised order for a solo in-tree phase:

    commit the code  →  flip `completed`  →  record-delta --start <base> --end <last code commit>
                     →  verify-vt  →  conformance

Recording **after** the flip means the corrected row is the one that survives; recording before
it means the transition may overwrite it. `doctrine slice conformance <ID>` is where a swept-in
foreign commit shows up (undeclared cell, unexpected paths) — check it after recording, not
before.

Sibling: `mem.pattern.doctrine.cross-slice-patch-review` for the review-side counterpart.
