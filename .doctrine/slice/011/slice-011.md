# Markdown from a `q` line — render reachable through the humane grammar

## Context

SL-010 gave selection a humane spelling: `smut q` compiles a line of argv tokens
to the ten-key specification (`dsl::compile` → `spec::parse`, DEC-038) and adds no
semantics. It covered only the JSON output. Its non-goals named the other half
explicitly:

> **`smut render` sharing the grammar.** Genuinely cheap *later* precisely because
> the compiler is a pure function over tokens; but render's operand order and
> display flags are their own migration.

IDE-006 (humane query DSL) § 5 states the target: one selection grammar, two
outputs — JSON envelope and Markdown. Today a human who wants Markdown must drop
back to quoted JSON:

```console
$ smut render '{"from":"q-query","select":["title","body"]}'
```

This slice is that deferred phase. The compiler exists; the work is the verb, the
flag sharing, one refusal, the guide, the README and the equivalence proofs.

Binding constraints inherited, none of which this slice may relax:

- **DEC-034** — `q` is a new verb *because* "a bare positional on `query` cannot be
  distinguished from a JSON operand, so a grammar bolted onto the machine verb would
  have to guess." The same holds for `render`, so this slice adds a verb (`r`)
  rather than overloading `render` (DEC-042), and amends DEC-034's "only human
  front-end" sentence at reconcile.
- **DEC-011 / DEC-019** — every existing `smut render JSON`, `smut render -` and
  `smut query` call stays valid, byte-identical; `tests/compatibility.rs` is the
  guard and is not edited.
- **DEC-038** — the token path emits the object and calls `spec::parse`; no second
  compiler.
- **DEC-037** — the token positional is a bare `Vec<String>` with no attributes.
- **DEC-004** — the whole line validates before any store access.
- **DEC-009** — render's Markdown rules are untouched.
- **DEC-008 / IMP-007** — render's short flags (`-f -t -l -i -s`) keep their meaning.
- **DEC-010 / DEC-040** — the embedded guide is the delivery channel; a spelling it
  does not teach does not exist.
- **IDE-006's shell-safety constraint** — nothing the grammar introduces uses a
  shell-active character.

## Scope & Objectives

1. **One selection grammar, reachable for Markdown.** A new verb `smut r` renders
   any selection `q` can express, with `q`'s exact tokens and flags, and no quotes
   (DEC-042).
2. **The machine syntax is a flag away.** `-j/--json JSON` (`-` = stdin) on `q` and
   `r` takes one JSON specification in place of the line; beside tokens or
   selection flags it is refused (DEC-044). Short-form refusals point at `-j`
   rather than `smut query` (DEC-045).
3. **Reuse, not a parallel path.** `SelectionArgs` (tokens, q's flags, `-j`) and
   `DisplayArgs` (render's five flags) are each one flattened clap struct, shared by
   every verb that takes them (DEC-043).
4. **Same selection, same output.** `r <line>`, `r -j <json>` and `render <json>`
   produce byte-identical Markdown on one store; `q -j <json>` and `query <json>`
   the same envelope.
5. **Refusals.** Everything `q` refuses, `r` refuses identically (same `Reject`,
   exit 2). The one new ambiguity — `-j` beside a selection line — is refused by
   name, never resolved by silent precedence.
6. **The guide and README teach it.** `src/learn.rs`'s render recipe becomes an
   executed `r` line; `-j` is taught on both verbs; README's command table gains
   `r` and `-j` (user direction, 2026-11-03).

## Non-Goals

- **New semantics or new operators** — the DSL's vocabulary is SL-010's.
- **Mermaid / new render formats / `--brief`** — IMP-002.
- **Changing render's Markdown output or display-flag meanings.**
- **Changing `q`'s grammar**, beyond extracting its flag set for sharing and adding `-j`.
- **`-j` on `query` or `render`** — they already take the spec positionally.
- **Shell completion, REPL** — as SL-010.
- **The store** — nothing in `.smut/`; STD-001 untouched.

## Affected surface

| path | what changes |
| --- | --- |
| `src/cli.rs` | `SelectionArgs`, `DisplayArgs`, `Sub::R`; `Q`/`Render` re-expressed by flatten; the `R` arm |
| `src/dsl.rs` | `Flags: PartialEq`; escape sentences point at `-j` |
| `src/error.rs` | `Reject::JsonBesideSelection`; `JsonOperand`'s sentence |
| `src/learn.rs` | commands block, operand paragraph, the `r` recipe as an executed constant |
| `src/main.rs` | the `--help` verb list names `r` |
| `README.md` | commands table and operand paragraph |
| `tests/cli_contract.rs` | `r` end to end; `-j`; the new refusal; refusal parity; no store created |
| `tests/dsl.rs` | pins for the changed sentences |
| `tests/fixture.rs` | a render runner, so the guide's `r` recipe is executed |
| `tests/compatibility.rs` | untouched — the guard |

## Risks, assumptions and open questions

- **OQ1 — the verb.** *Settled by DEC-042*: a new verb `smut r` plus `-j` on `q` and
  `r` (user direction b+c). Overloading `render` was rejected: a first-character
  dispatch rule leaks on quote and bracket spellings.
- **R1 — clap.** *Answered by DEC-043's probe*: the flattened attribute-free
  positional keeps DEC-037's behaviour; `-l --ids` binds loudly, never silently.
- **R2 — `-` vs `--`.** Moot for `render` (unchanged); on `q`/`r`, `-` is only ever
  `-j`'s value, and `--` still delivers hyphen-leading ids.
- **R3 — flag-name proximity.** `--ids` (projection) vs `--show-ids` (display);
  `--select` vs `--show-fields`. Distinct and both legal together; the guide says
  which is which.
- **R4 — SL-010 is done** (research), so `src/dsl.rs` is final as SL-010 left it.

## Verification and closure intent

- **Equivalence:** `r <line>` / `r -j <json>` vs `render <json>`, and `q -j` vs
  `query`, byte-identical on one store (design § Verification).
- **Compatibility:** `tests/compatibility.rs` passes unedited.
- **Refusal parity:** the shared `q` refusals reproduce through `r` at exit 2, same
  message; `JsonBesideSelection` on both verbs.
- **No store created** by any refused line.
- **Guide:** the render recipe in `smut learn` is a quote-free `r` line, executed by
  a new render runner in `tests/fixture.rs`.

## Summary

## Follow-Ups
