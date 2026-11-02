# IMP-015: `!has_store()` is non-discriminating in the query refusals and ~8 `cli_contract` sites

Found by the pre-close code review of SL-005 (RV-027 F-3), 2026-11-02.

`Scratch::has_store()` asserts that no `.smut` directory appeared during a refused
request, and its failure message reads *"the request is validated before the store is
opened, so no store exists"*. For a `query` (and `render`, `get`) that assertion cannot
fail: `Query::creates_store()` is `false` (`src/cli.rs:121-123`), and `Store::run` returns
`StoreError::Absent` without touching the filesystem when the store is absent
(`src/store/mod.rs:277-284`). So `!scratch.has_store()` passes under **every** ordering of
`spec::parse` against `store.run` — it is a true statement about `query`, not evidence
about ordering.

The discriminating assertion in those tests is `code == 2`: a request validated after the
store step would exit 1 with `no store at …`.

The same idiom appears ~8 times in `tests/cli_contract.rs` (`:285`, `:321`, `:340`, `:392`,
`:503`, `:526`, `:842`, `:897`, `:925`) and in `tests/predicates.rs`'s malformed-predicate
table. It is not wrong where the command *can* create a store (`declare`), which is what
`has_store()` was written for.

Wanted: either narrow the assertion to the commands where it discriminates (`declare`), or
keep it as a non-vacuity guard and fix the failure message to say what it actually proves
("a query never creates a store"). The stronger option, for the DEC-004 ordering claim
specifically: seed a corrupt `.smut` in the scratch directory and assert the contradictory
spec still exits 2 with the predicate line — a store opened before validation would exit 1
with a corruption message.

Related: RV-027 F-3, which fixes the design §5 attribution for SL-005's own test.
