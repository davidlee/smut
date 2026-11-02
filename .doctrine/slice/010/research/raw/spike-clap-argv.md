# SPIKE: does clap v4 derive carry `smut q`'s intended argv grammar?

**Verdict: `CLAP CARRIES IT` — with a bare `Vec<String>` positional and ZERO
`#[arg(...)]` attributes on it.** All 12 cases pass, including case 4
(flags interleaved *after* the positional has started), with token order
preserved and no error.

Evidence is executable: `cargo test --offline` in this crate runs 21 tests
(one per case + variant-comparison assertions) and they are green, in **both**
debug and release profiles.

---

## clap version actually used

Resolved from this crate's `Cargo.lock`, which is what the build linked:

```
clap         4.6.7
clap_builder 4.6.7
clap_derive  4.6.7
clap_lex     1.1.1
```

`cargo tree --offline -p clap` confirms `clap v4.6.7` (pinned as
`clap = { version = "=4.6.7", features = ["derive"] }`). Built fully offline
from the local registry cache; nothing was downloaded. The `explore` binary
prints the same version at runtime by reading the lockfile, so the number in
this document cannot drift from the number that ran.

Reproduce:

```
cd /tmp/smut-clap-spike
CARGO_TARGET_DIR=/tmp/smut-clap-spike/target cargo test  --offline   # 21 tests
CARGO_TARGET_DIR=/tmp/smut-clap-spike/target cargo test --release --offline  # 21 tests
CARGO_TARGET_DIR=/tmp/smut-clap-spike/target cargo run --offline --bin explore  # full matrix
CARGO_TARGET_DIR=/tmp/smut-clap-spike/target cargo run --offline --bin probe    # `--` placement
```

---

## Case table (winning variant: `plain`)

`smut` is the program name; the argv column omits it. Error text is verbatim
from `clap::Error::render()` with newlines shown as `⏎`.

| # | exact argv | expected | observed | pass/fail |
|---|---|---|---|---|
| 1 | `smut q session --backlinks part-of --depth 2 --ids` | tokens `["session"]`; backlinks/depth/ids set | tokens `["session"]`, backlinks=`part-of`, depth=`Some(2)`, ids=`true` | **pass** |
| 2 | `smut q --ids session` | positional survives a leading flag | tokens `["session"]`, ids=`true` | **pass** |
| 3 | `smut q status=open type=question --ids` | two tokens, `=` preserved | tokens `["status=open","type=question"]`, ids=`true` | **pass** |
| 4 | `smut q --ids session status=open --depth 2` | **key unknown**: flags after the positional has started | **accepts**, no reordering: tokens `["session","status=open"]`, ids=`true`, depth=`Some(2)` | **pass** |
| 5 | `smut q title~graph status!=open --follow any` | `~` and `!` verbatim | tokens `["title~graph","status!=open"]`, follow=`any` | **pass** |
| 6 | `smut q status=open,resolved --limit 10` | comma survives | tokens `["status=open,resolved"]`, limit=`"10"` | **pass** |
| 7 | `smut q -- -weird-id` | `--` reaches the hyphen id | tokens `["-weird-id"]`, ids=`false` | **pass** |
| 8 | `smut q id1 -- --ids id2` | after `--`, `--ids` is a plain token | tokens `["id1","--ids","id2"]`, ids=`false` | **pass** |
| 9 | `smut q --from=1` | equals-form flag, no token | tokens `[]`, from=`"1"` | **pass** |
| 10 | `smut q -weird-id` | error | `ErrorKind::UnknownArgument`, **exit 2**: `error: unexpected argument '-w' found⏎⏎  tip: to pass '-w' as a value, use '-- -w'⏎⏎Usage: smut q [OPTIONS] [TOKENS]...⏎⏎For more information, try '--help'.` | **pass** (see quirk §5.3) |
| 11 | `smut q` | valid parse, all-empty | tokens `[]`, ids=`false`, every `Option` `None` — **accepted**, so "no query" refusal stays in our layer | **pass** |
| 12 | `smut q session --ids` + siblings `query`/`q` | `q` unambiguous, no prefix inference | `q→Sub::Q`, `query→Sub::Query`; `quer` → `InvalidSubcommand` exit 2: `error: unrecognized subcommand 'quer'⏎⏎  tip: some similar subcommands exist: 'q', 'query'` | **pass** |

### Extra probes that matter for the grammar (plain variant)

| probe | argv | observed |
|---|---|---|
| positional resumes after a flag | `q a --ids b` | tokens `["a","b"]`, ids=`true` |
| mid-run flag, then more tokens | `q a --depth 2 b c` | tokens `["a","b","c"]`, depth=`Some(2)` |
| 4 tokens around a flag | `q a b --ids c d` | tokens `["a","b","c","d"]`, ids=`true` — **source order preserved** |
| flag first, many tokens | `q --depth 2 session status=open --ids` | tokens `["session","status=open"]`, ids=`true` |
| repeated boolean flag | `q session --ids --ids` | `ArgumentConflict` exit 2: `the argument '--ids' cannot be used multiple times` |
| `--` mid-run | `q a -- --ids` | tokens `["a","--ids"]`, ids=`false` |
| `--` first | `q -- a --ids` | tokens `["a","--ids"]`, ids=`false` |
| hyphen token after a positional | `q session -weird-id` | `UnknownArgument` exit 2 (loud, good) |
| single unknown short | `q -w` | `UnknownArgument` exit 2 |
| equals-form mid-run | `q a --from=1 b` | tokens `["a","b"]`, from=`"1"` |
| flag value binds the next word | `q --follow session status=open` | tokens `["status=open"]`, follow=`Some("session")` — standard CLI convention, but see §5.4 |
| equals-form numeric flag | `q --depth=2 --ids` | tokens `[]`, depth=`Some(2)`, ids=`true` |

---

## Minimal derive block that carries the grammar

Compilable verbatim (this is `src/lib.rs`'s `plain` module; `cargo test` uses
it). The essential fact: **`tokens` carries no `#[arg]` attribute at all.**

```rust
use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct QArgs {
    // No attribute. Bare Vec<String> positional: zero-or-more values,
    // accepts values before, between and after flags, honors `--`,
    // rejects unknown hyphen tokens.
    pub tokens: Vec<String>,

    #[arg(long)] pub follow: Option<String>,
    #[arg(long)] pub backlinks: Option<String>,
    #[arg(long)] pub depth: Option<u64>,
    #[arg(long)] pub budget: Option<u64>,
    #[arg(long)] pub select: Option<String>,
    #[arg(long)] pub ids: bool,
    #[arg(long)] pub order: Option<String>,
    #[arg(long)] pub limit: Option<String>,
    #[arg(long)] pub edges: bool,
    #[arg(long)] pub has: Option<String>,
    #[arg(long)] pub missing: Option<String>,
    #[arg(long)] pub from: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Sub {
    Q(QArgs),
    Query(QArgs), // sibling; coexists unambiguously (case 12)
}

#[derive(Parser, Debug)]
#[command(name = "smut")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Sub>,
}
```

Generated usage line: `Usage: smut q [OPTIONS] [TOKENS]...`.

**Minimal attribute set on `tokens`: the empty set.** Nothing needs
`allow_hyphen_values`, `trailing_var_arg`, `last`, `value_terminator`, or
`num_args`.

---

## 5. Silent wrongness — flagged loudly

Ranked worst first. Items 1 and 2 are **silent**: exit code 0, plausible
struct, wrong data.

### 5.1 `trailing_var_arg = true` — SILENTLY eats every flag after the first token

`q session --backlinks part-of --depth 2 --ids` parses **successfully** into
`tokens=["session","--backlinks","part-of","--depth","2","--ids"]`,
`ids=false`, `backlinks=None`, `depth=None`. No error, exit 0, all four flags
gone into the token list. 7 of 12 cases silently misparse. Do not use this.

### 5.2 `allow_hyphen_values = true` — SILENTLY eats flags, leaks `--`, kills case 10

- `q session --ids` → `tokens=["session","--ids"]`, `ids=false`. Silent.
- `q -weird-id` → `tokens=["-weird-id"]`, **no error at all**. Case 10's guard is
  silently gone; the user can no longer be told they forgot `--`.
- `q title~graph status!=open --follow any` → `--follow`/`any` swallowed into tokens.
- **`--` leaks into the token list once a positional value exists.** `--` is only
  consumed while the positional is still empty; after that it is captured as a
  literal token:

  | argv | plain (correct) | `allow_hyphen_values` |
  |---|---|---|
  | `q -- -weird-id` | `["-weird-id"]` | `["-weird-id"]` |
  | `q --ids -- -weird-id` | `["-weird-id"]` | `["-weird-id"]` |
  | `q a -- -weird-id` | `["a","-weird-id"]` | **`["a","--","-weird-id"]`** |
  | `q a -- --ids` | `["a","--ids"]` | **`["a","--","--ids"]`** |
  | `q a b -- c` | `["a","b","c"]` | **`["a","b","--","c"]`** |
  | `q -- a b` | `["a","b"]` | `["a","b"]` |

  This is the worst of the silent failures: a stray `--` token reaches the query
  parser as if the user had typed it deliberately. `trailing_var_arg` +
  `allow_hyphen_values` behaves identically here.

### 5.3 Case 10's error names `-w`, not the token `-weird-id`

Not silent (exit 2, loud), but inaccurate: clap splits `-weird-id` into the
short-flag cluster `-w -e -i -r...` and reports the first unknown member.
Observed: `error: unexpected argument '-w' found` + `tip: to pass '-w' as a
value, use '-- -w'`. If `smut` wants to echo the user's actual token in that
message, it must build the message itself (e.g. pre-scan `argv` for
hyphen-leading tokens not preceded by `--`). The 12-case grammar is unaffected.

### 5.4 A token immediately after a value-taking flag is consumed by the flag

`q --follow session status=open` → `follow=Some("session")`,
`tokens=["status=open"]`. This is standard `--flag value` convention, and
case 1 relies on it, but it means `smut q --follow part-of` is *not* a query
about a root named `part-of`. Not a clap defect — a grammar consequence worth
writing down. No ambiguity if callers write `--follow=part-of` or place the
flag last.

### 5.5 Repeated boolean flag errors instead of being idempotent

`q --ids --ids` → `ArgumentConflict` exit 2. Harmless, but if `smut` wants
idempotent flags it must opt in (`action = clap::ArgAction::Count`, or
`overrides_with_all`). Not part of the 12 cases.

### 5.6 `last = true` + `trailing_var_arg = true` — debug abort, release silent fallback

clap's debug assertion at `clap_builder-4.6.7/src/builder/debug_asserts.rs:540`
rejects the pair: `q:tokens: 'Arg::trailing_var_arg' and 'Arg::last' cannot be
used together`.

- **debug build** (`cfg!(debug_assertions) == true`): abort on first parse.
- **release build** (`debug_assertions` compiled out): **no panic** — it silently
  degrades to `last` semantics, i.e. `[-- <TOKENS>...]`, and rejects bare
  positionals with `error: unexpected argument 'a' found`.

So the same source produces a runtime abort in dev and wrong-but-plausible
behaviour in the shipped binary. Another reason to use neither attribute. Asserted
in both profiles by `last_plus_trailing_var_arg_is_a_forbidden_pair`.

---

## 6. Alternatives tried, in the requested order — what each costs

Each row = same `QArgs` with that attribute added to `tokens`. Measured over
the 12 cases (matrix: `cargo run --bin explore`).

| attribute | cases broken | what it costs |
|---|---|---|
| *(none) — **winner*** | **0 / 12** | nothing; `--` works, post-`--` flags stay tokens, no reordering |
| `num_args = 1..` | 0 / 12 | **buys nothing**, behaviour bit-identical to plain (asserted in `tests/variants.rs`). Rejected as non-minimal. |
| `num_args = 0..` | 0 / 12 | **buys nothing**, identical to plain. Rejected as non-minimal. |
| `trailing_var_arg = true` | 1, 3, 4, 5, 6, 8, 12 (7) | **silent** flag swallowing (§5.1); also turns a literal `--` into a token in case 8. `--` "works" only because everything is already raw. |
| `allow_hyphen_values = true` | 1, 3, 4, 5, 6, 8, 10, 12 (8) | **silent** flag swallowing + case 10 stops erroring + `--` leaks in as a literal token (§5.2). |
| `value_terminator = "--"` | 7, 8 (2) | `--` stops terminating flags: case 7 (`q -- -weird-id`) fails outright with `unexpected argument '-weird-id'`. Loud failure, but kills the one escape hatch. |
| `last = true` | 1, 2, 3, 4, 5, 6, 8, 12 (8) | makes tokens reachable **only** behind a literal `--`; usage becomes `smut q [OPTIONS] [-- <TOKENS>...]`, which contradicts the interleaved grammar. |
| `last = true, allow_hyphen_values = true` | 1, 2, 3, 4, 5, 6, 8, 12 (8) | same as `last`; hyphen allowance changes nothing here. |
| `trailing_var_arg + allow_hyphen_values` | 1, 3, 4, 5, 6, 8, 10, 12 (8) | silent swallowing **plus** `--` leaks into tokens as a literal value. Worst of both. |
| `last + trailing_var_arg` (and `+ allow_hyphen_values`) | all 12 | debug build: **PANIC** (clap debug assertion, incompatible pair); release build: silently behaves as `last` (§5.6). |

**Cost of the winner: none.** No `allow_hyphen_values`, no `trailing_var_arg`, so:

- `--` still terminates flag parsing (case 7 pass);
- everything after `--` becomes a token verbatim, including `--ids` (case 8 pass);
- flags may appear before, between, or after positionals without reordering (cases 1–4, probes E1–E3);
- hyphen-leading tokens *without* `--` remain a hard error (case 10 pass).

---

## 7. Subcommand behaviour (case 12 detail)

- `infer_subcommands` is **off by default**: `smut quer` does **not** resolve to
  `query`; it is `InvalidSubcommand`, exit 2.
- `q` and `query` are distinct and unambiguous; no alias collision.
- clap **does invent suggestions** on an unknown subcommand (strsim-based), and
  they are advisory only — they never change the exit code or accept the input:
  - `smut qq` → `tip: a similar subcommand exists: 'q'`
  - `smut quer` → `tip: some similar subcommands exist: 'q', 'query'`
  - `smut qery` → `tip: some similar subcommands exist: 'q', 'query'`
  - `smut qx` → `tip: a similar subcommand exists: 'q'`
- `smut` with no subcommand parses to `Cli { command: None }` (accepted), so the
  "a command is required" decision is ours, not clap's.

If `smut` ever wants `q` to be a hidden alias of `query` (or vice versa) that is
`#[command(alias = "q")]`, not `infer_subcommands` — but per case 12 no change
is needed: both spellings already coexist.

---

## 8. Files

- `src/lib.rs` — the model: 11 variants of `QArgs` (`plain` is the winner) +
  `Parsed` observables + the 12-case table.
- `src/bin/explore.rs` — prints the full variant × case matrix and the
  subcommand-resolution table, including the panic cases.
- `src/bin/probe.rs` — where `--` lands per variant, and the debug-vs-release
  behaviour of the forbidden `last + trailing_var_arg` pair.
- `tests/cases.rs` — 13 tests: one per case (+ `case_04b` for the interleaving
  resume).
- `tests/variants.rs` — 8 tests asserting *why* each alternative loses
  (including the silent-swallowing and `--`-leak failures).
