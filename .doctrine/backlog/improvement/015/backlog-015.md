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

## Resolution (2026-11-10)

Narrowed. The corrupt-store option was dropped as redundant: with no store present, a
request that reaches the store already exits 1 with `no store`. So `code == 2` is the
discriminating ordering evidence, and a corrupt `.smut` adds nothing.

- Removed the 8 vacuous `!has_store()` assertions on `query`/`q` refusals:
  `tests/predicates.rs`, ×4, in the malformed, contradictory, misplaced and unknown
  cases; `tests/cli_contract.rs`, ×4, in bad depth, bad budget, unknown key and
  `--depth` without a direction.
- The comment in `a_malformed_predicate_rejects_the_whole_request_before_any_store_is_opened`
  now names exit 2 as the evidence, and says why a no-store assertion would prove nothing.
- Kept: the `declare` sites, where the assertion does discriminate, and the "a read /
  `--help` creates nothing" sites, which are true statements and say no more.
- Removed `predicates.rs`'s now-unused `Scratch::has_store` and `Scratch::store`.
- Mutation probe: `src/main.rs::execute` was made to return `StoreError::Absent` for
  `query`/`q` before parsing. All 6 tests that held the removed sites went red; the
  2 `predicates` tests failed at their `code == 2` assertions. Probe reverted, gate green.
