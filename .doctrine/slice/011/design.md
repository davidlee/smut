<!-- doctrine:section sec-1 -->
## The surface — two human verbs, one grammar, one escape

`smut r` is to `smut render` what `smut q` is to `smut query`: the same line of words, compiled by
the same compiler, run through the same pipeline — with render's Markdown at the end instead of the
envelope (DEC-042).

```console
$ smut r q-query --select title,body                 # was: smut render '{"from":"q-query","select":["title","body"]}'
$ smut r session --backlinks part-of --depth 2 -s    # tokens, selection flags and display flags mix freely
$ smut q -j '{"where":{"$or":[...]}}'                # the whole JSON spec, on the human verb
$ smut r -j - < spec.json                            # -j - reads the spec from stdin
```

```
            ┌─────── tokens + selection flags ───────┐      ┌── -j JSON | -j - ──┐
            │  dsl::compile  (DEC-038, unchanged)     │      │  input::parse       │
            └──────────────────┬──────────────────────┘      └─────────┬───────────┘
                               └──────────────► spec::parse ◄──────────┘
                                                     │
                  q ─► Request::Query (envelope)     │     r ─► Request::Render (Markdown, + display flags)
```

| verb | selection is spelled | output | changes in this slice |
| --- | --- | --- | --- |
| `query` | one JSON operand, or stdin | envelope | none — byte-identical |
| `render` | one JSON operand, or stdin | Markdown | none — byte-identical; its flags move into a shared struct |
| `q` | tokens + flags, **or** `-j JSON` | envelope | gains `-j`; refusal escape pointers say `-j` |
| `r` *(new)* | tokens + flags, **or** `-j JSON` | Markdown | the whole verb |

Rules, each inherited rather than invented:

- **An empty `r` line is `{}`**, as an empty `q` line is (DEC-037): every entity, capped at 100.
  It never waits on stdin; stdin is read only for `-j -`.
- **`-j` takes the whole specification.** Beside any token or selection flag it is refused
  (DEC-044, § Refusals). Display flags (`-f -t -l -i -s`) stay legal beside it on `r`.
- **`-j`'s value is read by `input::decide`/`input::parse`** exactly as `render`'s operand is:
  `-` is stdin, anything else is the text; trailing input and duplicate keys are refused as today.
- **`r` has every display flag `render` has, with the same letters** (IMP-007). One letter means
  one thing across the CLI (`src/cli.rs` `conventions_takes_the_same_format_letter_as_render`);
  `-j` is unused anywhere, so it means `--json` wherever it appears.
- **`render` and `query` do not gain `-j`.** They already take the spec positionally; a second
  spelling there adds nothing.

### Where `r` and `render` deliberately differ

- **No operand.** `render` with nothing on a terminal is `MissingInput`; `r` with nothing is `{}`.
- **A `[`-led word.** `render '[1]'` is refused as not a specification; `r '[1]'` asks for the root
  id `[1]` (ids are free-form, ADR-001). Only `{` is special to the short form.

### What `r` deliberately cannot say

Anything `q` cannot: `$or`/`$not`, `$regex` (IDE-007), unlimited limits. The escape is `-j`, on
the same verb, which is the point of DEC-042: a human never has to change verbs to reach the
complete surface.

<!-- doctrine:section sec-2 -->
## The command line — two shared argument structs

Today `Sub::Q` carries q's twelve flags inline and `Sub::Render` carries the five display flags
inline. `r` needs both, so each set becomes one `#[derive(clap::Args)]` struct, flattened into
every verb that takes it (DEC-043):

```rust
/// The selection line `q` and `r` share: tokens, q's flags, or `-j`.
#[derive(clap::Args)]
pub struct SelectionArgs {
    tokens: Vec<String>,               // NO attribute — DEC-037; the probe below proves flatten keeps that
    #[arg(long)] follow: Option<String>,
    /* … the other eleven, exactly as Sub::Q has them today … */
    /// One JSON specification in place of the line; `-` reads stdin.
    #[arg(long, short = 'j')] json: Option<String>,
}

/// The display flags `render` and `r` share (DEC-009, IMP-007).
#[derive(clap::Args)]
pub struct DisplayArgs { /* -f -t -l -i -s, exactly as Sub::Render has them today */ }

enum Sub {
    Q      { #[command(flatten)] selection: SelectionArgs },
    R      { #[command(flatten)] selection: SelectionArgs, #[command(flatten)] display: DisplayArgs },
    Render { json: Option<String>, #[command(flatten)] display: DisplayArgs },
    /* … */
}
```

Two small methods carry the behaviour; both live in `cli.rs`:

- `DisplayArgs::options(self) -> Result<Options, Reject>` — today's `check_format` plus
  `heading_level_of` plus defaults, lifted out of the `Render` arm unchanged, now called from
  `render` and `r`.
- `SelectionArgs::specification(self, stdin_is_terminal, stdin) -> Result<Spec, Error>`:

  ```rust
  let flags = dsl::Flags { follow: self.follow, /* … */ };      // the one conversion; dsl.rs stays clap-free
  let value = match self.json {
      Some(_) if !self.tokens.is_empty() || flags != Flags::default() => return Err(Reject::JsonBesideSelection.into()),
      Some(json) => operand(Some(&json), stdin_is_terminal, stdin)?,
      None       => dsl::compile(&Line { tokens: &self.tokens, flags })?,
  };
  Ok(spec::parse(&value)?)
  ```

  `spec::parse` appears once for both human verbs. `dsl::Flags` gains `PartialEq` — its only
  change besides the sentences in § Refusals. `Flags` is ten `Option<String>` and two `bool`, so
  any typed flag, `--select=` included, differs from the default.

**Order of validation (DEC-004, DEC-008).** In the `R` arm, `display.options()` runs first, then
`selection.specification(..)`, so a bad display flag never waits on stdin, and nothing touches a
store before the whole line has validated.

**Evidence (probe, clap 4.6.7, the locked version; scratchpad `probe/`).** With the positional
flattened and attribute-free:

| line | parsed as |
| --- | --- |
| `q a --follow x b` | tokens `[a, b]`, follow `x` — flags interleave as DEC-037 requires |
| `r a --follow x b -l 3 -i` | both structs filled |
| `r -- -weird` | tokens `[-weird]` |
| `r a -- --follow` | tokens `[a, --follow]` |
| `r` | empty tokens — the `{}` line |
| `q -j -` / `r -j {"from":"a"}` | `json` set, tokens empty |
| `r a --heading-level --ids` | level `--ids` → refused loudly as an invalid level (same as `render` today) |
| `r -l -j 3` | level `-j` → the same loud refusal; `-j` is never silently lost |
| `q --help` | `Usage: smut q [OPTIONS] [TOKENS]...` — the pinned line, unchanged |

`render`'s usage keeps `[JSON]`; `r`'s reads `[TOKENS]...`. Both help tests gain `r`:
`main.rs`'s verb list and `cli.rs`'s `help_is_answered_as_text_that_advertises_the_grammar`.
`cli.rs`'s `every_short_flag_reaches_exactly_the_state_its_long_form_reaches` gains `r`'s five pairs.

<!-- doctrine:section sec-3 -->
## Refusals

### One new refusal

| case | variant | sentence | exit |
| --- | --- | --- | --- |
| `-j` beside any token or selection flag | `Reject::JsonBesideSelection` | `-j takes the whole specification; drop the tokens and selection flags beside it` | 2 |

It names only `-j`, which the caller typed, and describes the rest by kind rather than naming a
flag (DEC-030, DEC-039). It is raised in `SelectionArgs::specification`, before any operand is
read and before any store. Rejected: clap's `conflicts_with_all` — a fourth hand-written list of
the flag names (IMP-016's drift class) answered in clap's multi-line voice (DEC-044).

### The escape pointer becomes `-j` (DEC-045)

Five sentences, verified: `src/dsl.rs:319,327,505,581` and `src/error.rs:385`. Each names `smut query` as its escape, and two name `q` as the subject.
Through `r` both would be wrong. They become verb-blind, so the compiler stays verb-blind:

| site | today | after |
| --- | --- | --- |
| `error.rs` `JsonOperand` | `q takes field=value tokens, not a JSON specification; use smut query for a JSON spec` | `a word beginning with { is not read as a token: a JSON specification goes after -j, and an id after --from=` |
| `dsl.rs:319` | `it carries two {first}, so q cannot tell the field from the value; use smut query` | `it carries two {first}, so the short form cannot tell the field from the value; use -j` |
| `dsl.rs:327` | `…; use smut query for the same predicate as JSON` | `…; use -j for the same predicate as JSON` |
| `dsl.rs:505` | `… write two conditions or use smut query` | `… write two conditions or use -j` |
| `dsl.rs:581` | `q carries one {spelling} per field; use smut query` | `the short form carries one {spelling} per field; use -j` |

The plan re-reads each sentence against `mem.pattern.smut.refusal-pins-hold-shape-not-truth`:
the subject of each must be true of every input that reaches it. The subject of each is the short
form — true of both verbs — never the caller's token. `JsonOperand` is reached by any `{`-led word
(`src/dsl.rs:85` checks the first character only), which may be JSON or a free-form id such as
`{weird}`, so its sentence names both escapes (DEC-036's `--from=` for the id).

`JsonOperand` stays inside `dsl::compile`: the fix is identical for both verbs, so "which verb" is
no longer a call-site fact (research CF-2 dissolves). A `[`-led token stays a root — ids are
free-form (ADR-001), and the leak it would guard belonged to the dispatch rule DEC-042 rejected.

### Inherited unchanged

Every other `q` refusal reaches `r` verbatim, because both arms call one `compile` and one
`spec::parse`. A `-j` value is refused by `input::parse` and `spec::parse` exactly as `render`'s
operand is. Display-flag refusals (`--format mermaid` naming 1.0, an unknown format, a heading
level outside 1–6) reach `r` from the same `DisplayArgs::options`.

### Named debt, not paid here

The new variant joins an uncensused `Reject` set (IMP-014); refusal tests use the existing
`!has_store()` check, which IMP-015 notes is non-discriminating.

<!-- doctrine:section sec-4 -->
## The guide and the README

### `smut learn` (DEC-010, DEC-040)

- **Commands block** gains `smut r <TOKEN>... [display flags]` ("render as words") and both
  `q` and `r` rows show `[-j|--json JSON]`.
- **The operand paragraph** ("declare, query and render each take one JSON argument …") adds that
  `-j` takes the same JSON, `-` included, on `q` and `r`, and that `q` or `r` with no words
  answers every entity, capped at 100, rather than waiting or refusing. Its last sentence ("the others, q included,
  do", `src/learn.rs:59`, pinned) names `r` too.
- **The short-form section** — heading `The short form: smut q` (`SHORT_FORM_HEADING`, pinned twice)
  becomes `The short form: smut q and smut r`; the escapes at `:143` and `:149` name `-j` rather
  than `smut query` (the `--ids`/`--from=ID`/`smut query` escape pin at `:933` moves with them).
  The clause `smut query takes the full specification and is the complete surface` (`:149-150`,
  pinned at `:898` and `:1028`) stays verbatim — `query` is still the complete surface — and the
  sentence before it reads "… are all for -j, or smut query: …";
  `query`/`render` are named as the machine verbs that are unchanged.
- **Rendering** says `r` takes q's line, and the Workflows recipe becomes
  `smut r q-query --select title,body` — quote-free, no display flags. Two new constants:
  `EXAMPLE_RENDER` (the JSON spec `{"from":"q-query","select":["title","body"]}`, today inline prose
  at `src/learn.rs:337`) and `EXAMPLE_RENDER_R` (the line). The guide prints both; the pin
  `the_guide_prints_the_examples_the_fixture_executes` (`:702`) enumerates them.
- The two warts (value typing, `~`'s two meanings) apply to `r` unchanged; the text says "the short
  form" rather than "q" where it now covers both.

### README (user direction: in scope)

- The Commands table gains `smut r [TOKEN]...` ("the same line of words, printed as Markdown") and
  the `q` row notes `-j`.
- The operand paragraph mirrors the guide's.
- The JSON render examples (`README.md:53,67,91,145`) stay as they are: they are valid, executed by
  `tests/readme_example.rs`, and `render` is unchanged. Converting one to `r` is optional and
  only if the README harness runs it without new machinery.

<!-- doctrine:section sec-5 -->
## Code impact

| path | change |
| --- | --- |
| `src/cli.rs` | `SelectionArgs`, `DisplayArgs`, their two methods; `Sub::R`; `Sub::Q`/`Sub::Render` re-expressed by flatten; the `R` arm → `Request::Render`; unit tests: short/long pairs (`:986`) and help (`:667`) gain `r` |
| `src/dsl.rs` | `Flags` derives `PartialEq`; four escape sentences (§ Refusals); module doc mentions `r` |
| `src/error.rs` | `Reject::JsonBesideSelection` and its sentence; `JsonOperand`'s sentence and doc; the exit-code table and message test |
| `src/learn.rs` | commands block; operand paragraph incl. the pinned store-need sentence (`:59`); short-form heading (`:766`) and escapes (`:143`, `:149`, pin `:933`); Rendering; Workflows recipe; `EXAMPLE_RENDER` + `EXAMPLE_RENDER_R` and the examples pin (`:702`) |
| `src/main.rs` | the `--help` verb-list test names `r` |
| `README.md` | commands table and operand paragraph |
| `tests/cli_contract.rs` | `r` end-to-end, `-j` on `q` and `r`, the new refusal, refusal parity, help lines |
| `tests/dsl.rs` | message pins for the changed sentences |
| `tests/fixture.rs` | a render runner (`Declared::r(&str)`) and the recipe executed in both spellings |

### Unchanged, and why

- `src/spec.rs`, `src/predicate.rs`, `src/select.rs`, `src/render_markdown.rs` — both new
  spellings funnel through the existing seams (DEC-038, DEC-009).
- `src/input.rs` — `-j`'s value goes through `decide`/`parse` as `render`'s operand does.
- `tests/compatibility.rs` — the guard; edited never.
- `.smut/` and `docs/store-format.md` (STD-001).

### Selectors

Design targets: `src/cli.rs`, `src/dsl.rs`, `src/error.rs`, `src/learn.rs`, `src/main.rs`,
`README.md`, `tests/cli_contract.rs`, `tests/dsl.rs`, `tests/fixture.rs`.

<!-- doctrine:section sec-6 -->
## Verification

### Equivalence — the spine

On one fixture store, byte-identical stdout, stderr and exit code for:

| left | right |
| --- | --- |
| `r <line>` | `render <json>` — for the three recipes with JSON twins in `learn.rs` (`EXAMPLE_TRAVERSE`/`_FOLLOW`/`_FILTER` and their `_Q` lines), the Workflows recipe (`EXAMPLE_RENDER`/`_R`), and the other two hot paths (`learn.rs:154-158`) with JSON twins written in the harness |
| `r -j <json>` | `render <json>` |
| `r -j -` (stdin) | `render -` |
| `q -j <json>` | `query <json>` |
| `r <line> -s -i -l 3 -t name` | `render <json> -s -i -l 3 -t name` — display flags agree across verbs |
| `r` (empty) | `render '{}'` |

### Refusals

- `JsonBesideSelection` at exit 2 for: a token beside `-j`; a selection flag beside `-j`
  (`--ids`, `--depth 2`); both on `q` and `r`. And **not** refused: `r -j '{}' -s`.
- **Parity through `r`** (research D-8): `--ids` with `--select`, `--depth` with no direction,
  `--limit all`, `--from=` empty, `--order title:user`, a two-operator token — each the same
  message and exit through `r` as through `q`. This table catches a copy-paste between the two
  arms.
- The changed sentences pinned whole in `tests/dsl.rs` / `error.rs`, each test naming the change
  that reddens it.
- No store created by any refused line (DEC-004).
- `r -l --ids` and `r -l -j 3` refused as an invalid heading level (the loud cases from the probe).
- `JsonOperand` through `r` for both a JSON-looking word and a `{`-led id, and `r --from={weird}`
  reaching that id.

### Stated divergences, tested

- `r` with no words answers like `render '{}'`; `render` with no operand on a terminal is
  `MissingInput` — both asserted.
- `r '[1]'` asks for the id `[1]`; `render '[1]'` is refused — both asserted.

### Surfaces that must not move

- `tests/compatibility.rs` passes **unedited**.
- `smut q --help` still prints `Usage: smut q [OPTIONS] [TOKENS]...` (existing pin).
- `render`'s existing contract tests (`tests/cli_contract.rs:453-505`) pass unedited; the
  short/long pair test (`src/cli.rs:986`) keeps every existing pair and gains `r`'s.

### The guide stays executed

`tests/fixture.rs` gains `Declared::r(line)` and `Declared::render(&Value)` beside `Declared::q`,
and asserts `EXAMPLE_RENDER_R` and `EXAMPLE_RENDER` produce byte-identical Markdown
(research CF-3/D-4). The README harness
still passes.

<!-- doctrine:section sec-7 -->
## Reconcile obligations

Knowledge-record amendments — not revisions (REV-003's precedent) — landed at reconcile with
`doctrine knowledge edit decision`, keeping commas out of `--alternatives`/`--consequences`
(`mem.fact.doctrine.knowledge-facet-array-comma-splitting`):

| record | sentence that becomes false | amendment |
| --- | --- | --- |
| DEC-034 | "`q` is the only human front-end and has no alias" | two human verbs `q` and `r` over one grammar; superseded in part by DEC-042 |
| DEC-008 | "one JSON operand" | the JSON spec arrives as an operand on `query`/`render` or as `-j` on `q`/`r` |
| DEC-039 | refusals "point at `smut query`" | they point at `-j` (DEC-045) |
| DEC-040 | the guide's division of labour names `q` and `query` | adds `r` and `-j` |

Backlog: none new. IMP-014, IMP-015 and IMP-016 are touched and named, not paid.

