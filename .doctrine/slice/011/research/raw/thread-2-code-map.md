# Thread 2 — code map (raw, verbatim round output)

Run inline by the orchestrating agent on 2026-11-03 at `cb402c3`. Read-only: `read`, `grep`,
`wc`. Nothing built, nothing written.

## Sweep

| command | what it established |
| --- | --- |
| `wc -l src/*.rs tests/*.rs` | `src/cli.rs` 1086, `src/dsl.rs` 607, `src/learn.rs` 1218, `src/error.rs` 1168, `tests/cli_contract.rs` 2109, `tests/fixture.rs` 1064, `tests/dsl.rs` 756, `tests/compatibility.rs` 578, `tests/render.rs` 402 |
| `grep -n "enum Sub\|Sub::Q {\|Sub::Render {" src/cli.rs` | clap surface at 239–344; parse arms at 384–449 |
| `read src/dsl.rs` (full) | `Line` 26, `Flags` 40, `compile` 75, `{`-refusal 85–86, `--select`/`--ids` 150, `--limit all` 178–180, `value()` 237, operator table 252 |
| `grep -n "fn operand\|fn check_format\|fn heading_level_of" src/cli.rs` | 492, 505, 518 |
| `grep -n "fn decide" src/input.rs` | 28–35, the pure tty/stdin decision |
| `grep -rn "flatten\|clap::Args" src/` | **no hits** — the tree has no `Args` struct and no `#[command(flatten)]` |
| `grep -c "render" tests/fixture.rs` | **0** — no render harness exists |
| `grep -n "render_pins_its_markdown_text" tests/compatibility.rs` | 272–289 |
| `grep -n "Usage: smut q\|value_name = \"ID\"" ...` | the usage pin at `tests/cli_contract.rs:2040`; `value_name` on get/delete only |
| `grep -rn "Usage: smut render\|smut render \[JSON\]"` | `src/learn.rs:47` and `README.md:275` (plus stale `target/package/*` copies) |

## The flag cross-product (checked by hand, every pair)

`q`'s twelve: `follow backlinks depth budget select ids order limit edges has missing from`.
`render`'s five: `format heading-field heading-level show-ids show-fields`.
**Intersection: empty.** No long name collides. The near-pairs are semantic only (`ids` vs
`show-ids`, `select` vs `show-fields`) — R3's shape.

Shorts in use: render `-f -t -l -i -s`; `declare` `-c`; `conventions` `-f`; clap owns `-h -V`.
A shared flag set therefore may not take a short letter at all.

## Reading comments that did not survive distillation

1. **`Sub::Q`'s twelve flags are inline fields on the variant** (cli.rs:274–306). There is no
   `Args` struct anywhere in `src/`. So scope objective 2 ("likely a clap `#[command(flatten)]`-ed
   args struct") is a genuinely new shape for this tree, not a refactor of one that exists —
   whichever struct is introduced, its derivation must be argued against DEC-037.

2. **`dsl::Flags` is already a plain owned data struct** (dsl.rs:40–72) and `compile` takes
   `&Line { tokens: &[String], flags }`. A clap struct in `cli.rs` that converts into `dsl::Flags`
   leaves `dsl.rs` clap-free (it has no clap import today). That is the cheapest sharing seam and
   it matches the slice's "at most a `Flags` conversion if the shared struct lives in `cli.rs`".

3. **`Reject::JsonOperand` is raised inside `compile`** (dsl.rs:85–86) and voiced in `error.rs:385`
   as *"q takes field=value tokens, not a JSON specification; use smut query for a JSON spec"*.
   The variant is documented (error.rs:105) as "A `smut q` operand is a JSON specification. Only
   `smut query` takes one". If render accepts a JSON operand (OQ1(a)), render must *not* raise it —
   so sharing `compile` forces this single refusal to the `q` call site.

4. **`render`'s absent-operand behaviour is a real second half of the ambiguity.** `operand()`
   (cli.rs:492) delegates to `input::decide` (input.rs:28): `Some("-")` → stdin, `Some(text)` →
   operand, `None` + tty → `Reject::MissingInput`, `None` + pipe → stdin. `q` with no tokens is
   `query {}` (DEC-037). A `Vec<String>` positional on render makes "no operand" mean both.

5. **`tests/cli_contract.rs:2028–2042` pins `Usage: smut q [OPTIONS] [TOKENS]...`** and its
   comment states that `value_name`, `trailing_var_arg` or `allow_hyphen_values` "would redden
   this line". `render`'s usage line is *unpinned* today, so nothing reddens when it changes —
   the risk is the reverse of `q`'s.

6. **`allow_hyphen_values` sits on `--heading-level`** (cli.rs:322), an option value, not on the
   positional. DEC-037's silent-swallowing finding is about a *positional* carrying it. The two
   coexist today; the only interaction is `--heading-level --ids` binding `--ids` as the level
   text and then refusing it as an invalid level — loud, exit 2.

7. **`tests/fixture.rs:615–646` executes the guide's printed examples and 648–690 runs the
   two-call recipe in both spellings** (`q` vs `query`), with `filter_over_words` (603) spelling
   the second call as a token line. There is no render execution anywhere in the file — and
   `tests/readme_example.rs:487` is the only place a `smut render ` *command* is executed at all,
   from the README.

8. **`tests/dsl.rs:29–63` is a second reader of a printed line** — whitespace split plus a flag
   table that `panic!("the guide prints no --{other}")`. A printed render line would need a third
   reader that also knows the five display flags.

9. **`tests/render.rs` is insulated.** It drives `select` → `render_markdown::render` across the
   lib boundary and never touches the CLI, so the verb change cannot redden it. `src/render_markdown.rs`
   takes `Selection` + `Options` (30–38, 56) and knows nothing about argv.

10. **`src/spec.rs:121 pub fn parse(input: &Value)`** is the single seam both verbs must funnel
    through (DEC-038). `packages`/`target/package/*` hold stale copies of `learn.rs` and
    `README.md` from earlier publishes — irrelevant to the slice, mentioned so a later grep does
    not treat them as live.
