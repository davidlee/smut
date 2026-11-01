# DEC-020: Identity key: one definition, a wire-contract assertion, and executed transcripts for every surface but the store document

Answer to SL-004's inq-1, taken one question at a time with the operator.

## The surfaces and what holds each

| Surface | Pin | Why it holds |
| --- | --- | --- |
| all code paths | `ENTITY_ID` (`src/entity.rs:17`), 27 readers | one definition; canonical byte ordering reads it too |
| `smut learn` | prose pins in `src/learn.rs` tests | the guide is a `const` string, so no constant can be interpolated into it |
| guide + fixture examples | executed by `tests/fixture.rs` | `EXAMPLE_TRAVERSE`/`FOLLOW`/`FILTER` are constants so the bytes shown are the bytes run |
| README | executed by `tests/readme_example.rs` | the one `console` block is read out of the file and run in order |
| `docs/store-format.md` | **STD-001** — no test | prose read by nothing; the standard's title carries the review trigger into every agent's boot context |

## Why the store document gets a standard rather than a test

Minting STD-001 was the operator's call, and it is the cheaper mechanism: the title
is loaded into every agent's boot context before it touches `src/store/`, so the
trigger ("review it whenever the format, its version, its identity key or its
corruption contract changes") arrives ahead of the work rather than in a gate after
it. A grep-based test was considered and rejected: it would either be vacuous or
would fire on payload data, which is allowed to carry an `id` key after this change.

## What this commits the slice to

- Code needs only the constant's value changed; no new indirection.
- `smut learn` keeps its prose pins; the renamed key appears in pinned sentences.
- `docs/store-format.md` is a `scope-relevant` surface of the change — it is
  delivered or it is an undelivered selector at audit, not a follow-up.
