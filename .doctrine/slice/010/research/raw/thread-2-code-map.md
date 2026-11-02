# Thread 2 raw — code map (SL-010)

Producer: orchestrating agent, degraded mode (no project research agents defined).
Read-only: greps and reads only.

Commands run:

    grep -n "enum Sub|^pub fn parse|InvalidSubcommand =>|InvalidUsage {" src/cli.rs
    sed -n 220,300p src/cli.rs
    grep -n "smut: |fn main" src/main.rs
    grep -n "^fn parse_|pub fn capped" src/spec.rs
    sed -n 60,110p src/predicate.rs
    grep -n "mod tests" -A 30 src/error.rs
    grep -n "fn |const |^pub" src/learn.rs
    sed -n 235,310p tests/readme_example.rs
    grep -n "fn |struct " tests/fixture.rs
    sed -n 1,50p tests/cli_contract.rs tests/compatibility.rs
    grep -n "allow_hyphen_values|trailing_var_arg" src/

## Raw observations

- `src/cli.rs:237` `enum Sub` has ten variants; `Cli` is a `#[derive(Parser)]` with
  `command: Option<Sub>`. `pub fn parse` (`:296`) is the whole of DEC-004 step 1.
- clap error mapping is at `src/cli.rs:383-388`: `InvalidSubcommand` →
  `Reject::UnknownCommand(name)`, everything else → `Reject::InvalidUsage { message }`
  carrying clap's own diagnostic. Any new clap-level refusal therefore renders
  `smut: <clap text>` by default; DEC-030's one-line discipline already holds.
- `allow_hyphen_values` appears exactly once (`heading_level`, `src/cli.rs`); no
  `trailing_var_arg` anywhere. Nothing in the tree yet mixes unbounded positionals
  with flags, so there is no precedent to copy — the pattern would be new.
- `get`/`delete` prove the `--` path end-to-end (`src/cli.rs` test
  `a_double_dash_reaches_an_opaque_id`), including the `OsString` ingress.
- `src/spec.rs:121` `pub fn parse(&Value)`; the ten `parse_*` helpers are private
  (`:167-296`). `Limit::capped` is `pub const fn` (`:67`); the unlimited form is
  crate-internal, so a `--limit all` cannot exist without widening a deliberate
  privacy boundary (DEC-001 rule 4).
- `src/predicate.rs:60` `pub fn parse` = build + one `refusal` walk (the § DEC-027
  shape); the five-operator list is written twice inside this file (`:94-102` and
  `parse_condition`) with a comment admitting a sixth must be added to both.
- `src/error.rs:457` tests: `every_store_refusal()` is a real census for
  `StoreError`; `Reject` has no counterpart (IMP-014). Message tests are per-variant.
- `src/learn.rs:31` `pub const GUIDE` (a single 250-line string literal); examples at
  `:287-310`; ~35 unit tests pin guide sentences, several through a `teaching()`
  substring helper (`:623`), so a reworded recipe fails unit tests as well as the
  executed ones.
- `tests/fixture.rs:27` imports the three example constants; `:547` replaces `IDS`
  inside `EXAMPLE_FILTER` and `Declared::query(&Value)` runs it.
- `tests/readme_example.rs:242` `fn arguments(&str)` splits a README console line on
  whitespace with single-quote handling. It performs **no** glob, tilde or `$`
  expansion — it is not a shell.
- `tests/cli_contract.rs` spawns `env!("CARGO_BIN_EXE_smut")` per invocation in a
  scratch dir; `tests/compatibility.rs` pins parsed-`Value` equality plus key sets.
- `Cargo.toml:28` `clap = { version = "4", features = ["derive"] }` (v4, no `string`
  or `unstable-*` features).

## Nothing found

- No `dsl`, `q` or `human` module; no prior art in-tree for a second front-end.
- No `every_reject_refusal()`; no census over `Reject`.
- No fixture helper that runs a *token list* rather than a JSON spec — `Declared`
  only speaks `&Value`.
