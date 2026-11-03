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
outputs — JSON envelope (`q`) and Markdown (`render`) — e.g.

```
smut render --backlinks part-of --depth 2 session
```

Today a human who wants Markdown must drop back to quoted JSON:

```console
$ smut render '{"from":"q-query","select":["title","body"]}'
```

This slice is that deferred phase. The compiler exists; the work is the verb shape,
the flag merge, the refusals, the guide and the equivalence proofs.

Binding constraints inherited, none of which this slice may relax:

- **DEC-034** — `q` is a new verb *because* "a bare positional on `query` cannot be
  distinguished from a JSON operand, so a grammar bolted onto the machine verb would
  have to guess." `render` takes the same JSON positional, so the same argument
  applies to it. Whatever spelling this slice picks must answer that rationale, or
  amend DEC-034 explicitly (OQ1).
- **DEC-011 / DEC-019** — every existing `smut render JSON` and `smut render -`
  call stays valid, byte-identical; `tests/compatibility.rs`
  (`render_pins_its_markdown_text`) is the guard and is not edited.
- **DEC-038** — the DSL path emits the object and calls `spec::parse`; no second
  compiler, no render-specific grammar.
- **DEC-037** — the clap shape: a bare `Vec<String>` positional with no
  attributes. Render's `--heading-level` carries `allow_hyphen_values`; whether that
  survives beside a bare token list is a design question, not a given.
- **DEC-004** — the whole line validates before any store access.
- **DEC-009** — render's Markdown rules (fields only on request, safe delimiters,
  in-document diagnostics) are untouched.
- **DEC-008 / IMP-007** — render's existing short flags (`-f -t -l -i -s`) keep
  their meaning.
- **DEC-010** — the embedded guide (`smut learn`) is the delivery channel; a
  spelling it does not teach does not exist.
- **IDE-006's shell-safety constraint** — nothing the grammar introduces uses a
  shell-active character.

## Scope & Objectives

1. **One selection grammar, reachable for Markdown.** A human can render any
   selection `q` can express, with `q`'s exact tokens and flags, and no quotes.
2. **Reuse, not a parallel path.** The `q` flag set and its extraction into
   `dsl::Flags` exist once and are shared by both verbs (likely a clap
   `#[command(flatten)]`-ed args struct). Render's display flags stay render's.
3. **Same selection, same output.** For any line, the DSL spelling and the JSON
   spelling of render produce **byte-identical** Markdown on the same store.
4. **Refusals.** Everything `q` refuses, the render spelling refuses identically
   (same `Reject`, same exit 2). Any new ambiguity the merge creates — a JSON operand
   beside tokens, a flag-name collision, `--ids` vs `--show-ids` — is refused by name
   or designed out, never resolved by silent precedence.
5. **The guide teaches it.** `src/learn.rs`'s render recipe becomes a DSL line; the
   JSON form stays documented as complete.

## Non-Goals

- **New semantics or new operators** — the DSL's vocabulary is SL-010's.
- **Mermaid / new render formats / `--brief`** — IMP-002.
- **Changing render's Markdown output or display-flag meanings.**
- **Changing `q`'s grammar**, beyond extracting its flag set for sharing.
- **Shell completion, REPL** — as SL-010.
- **README** — operator-owned (CHR-001).
- **The store** — nothing in `.smut/`; STD-001 untouched.

## Affected surface

| path | what changes |
| --- | --- |
| `src/cli.rs` | the render verb's clap shape (or a new verb, per OQ1); the shared `q` flag struct; dispatch into `dsl::compile` then `Request::Render` |
| `src/dsl.rs` | ideally nothing; at most a `Flags` conversion if the shared struct lives in `cli.rs` |
| `src/error.rs` | only if the merge creates a refusal `q` lacks (e.g. JSON operand beside tokens on one verb) |
| `src/learn.rs` | the render recipe as a DSL line; the usage line |
| `src/main.rs` | the `--help` verb list, if a verb is added |
| `tests/cli_contract.rs` | byte-identity DSL-vs-JSON render; refusals; no store created; short flags still work |
| `tests/compatibility.rs` | untouched — the guard |
| `tests/fixture.rs` | the guide's render recipe stays executed |

## Risks, assumptions and open questions

- **OQ1 — the verb.** DEC-034's reasoning forbids guessing between a JSON operand
  and tokens on one positional. Candidates:
  - (a) `smut render` accepts tokens, with a deterministic dispatch rule (e.g. a
    single positional beginning `{`, or exactly `-`, is the JSON operand; anything
    else compiles). `q` already refuses a `{`-leading token, so the rule may be
    total rather than a guess — but it must be argued against DEC-034 and DEC-034
    amended or reinterpreted on the record.
  - (b) a new verb (e.g. `smut r`), mirroring `q`/`query`: zero back-compat risk,
    one more verb to teach.
  - (c) a render mode on `q` (e.g. `smut q … --markdown`): one verb, but render's
    five display flags then become `q` flags, which dilutes `q`'s surface.
  Settled at `/design`; IDE-006 § 5 leans (a).
- **R1 — clap.** Render's `allow_hyphen_values` on `--heading-level` and DEC-037's
  attribute-free positional must coexist; the design proves it with a test, not by
  reading clap docs.
- **R2 — `-` the stdin operand vs `--` the id escape.** Under (a), `smut render -`
  must stay stdin; `--` must still reach a hyphen-leading id.
- **R3 — flag-name proximity.** `--ids` (projection) vs `--show-ids` (display);
  `--select` (projection) vs `--show-fields` (display). Distinct, but adjacent in
  meaning — the guide must say which is which; the design decides whether any pair
  needs a refusal.
- **R4 — SL-010 is in audit.** This slice builds on `src/dsl.rs` as SL-010 leaves
  it; design should not start until SL-010's reconcile settles (`after` SL-010).

## Verification and closure intent

- **Equivalence:** for each guide recipe and a table of representative lines,
  `render <tokens>` and `render <json>` are asserted byte-identical on one store.
- **Compatibility:** `tests/compatibility.rs` passes unedited.
- **Refusal parity:** every `q` refusal reproduces through the render spelling at
  exit 2, same message; any new refusal names its fix (DEC-030 voice).
- **No store created** by any refused line.
- **Guide:** the render recipe in `smut learn` is a quote-free DSL line, and is
  executed by `tests/fixture.rs`.

## Summary

## Follow-Ups
