# IMP-022: smut -C <dir>: run as if started in <dir>, so an agent's command starts with smut and needs no cd prefix

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised 2026-11-21 by the operator: an agent should not have to prefix every
command with `cd $dir &&`.

## Why a flag

In Claude Code, `cd` carries over between Bash calls only in the main session, and
only inside its working directories; subagents and delegated workers start fresh,
and environment variables never carry over. A `cd x && smut …` compound also
defeats a `Bash(smut:*)` permission prefix and can prompt. `smut -C <dir> …`
starts with `smut`, is stateless, and is the idiom `git -C` / `make -C` already
taught every agent. The environment-variable form is IDE-008.

## Design

```
smut -C <dir> <command> …
        │
        ▼
cli::parse ──▶ CommandLine { directory, invocation }
        │
        ▼
start = <dir> if absolute, else <cwd>/<dir>; no -C → <cwd>
        ├──▶ store root:   <start>/.smut          (never searched upward)
        └──▶ project layer: nearest .git from <start>  (conventions only)
```

- `-C` names the directory smut acts as if started in: **both** the store root and
  the start of the conventions project-root walk, so a store and its conventions
  never come from different checkouts.
- It is a top-level option, before the verb, as in git. It is not `global`: a `q`
  line's bare-token positional (DEC-037) stays untouched.
- Exact, not searched: `<dir>/.smut`, never a parent (the store-format rule holds).
- A relative `<dir>` is joined to the current directory, so a refusal names an
  absolute path and the conventions walk can climb above it.
- No up-front existence check. A missing `<dir>` falls out of the paths that
  already exist: a read exits 1 with `no store at <dir>/.smut`; `declare` exits 1
  at `create_store_directory` (`create_dir`, not `create_dir_all`), creating
  nothing anywhere.
- The environment is still read only in the binary (DEC-032). `dispatch` takes a
  function from start directory to `Roots` instead of a precomputed `Roots`, so
  in-process tests still never read `$HOME`; the roots are now computed only for
  `conventions`, the one verb that uses them.

## Resolution (2026-11-21)

Built as designed above.

- `src/cli.rs`: `Cli` gains `-C <DIR>`; `parse` answers a `CommandLine
  { directory, invocation }`, and the subcommand match moved into `invocation`.
- `src/main.rs`: `start_directory` resolves `-C` (absolute as given, relative
  joined to the cwd, else the cwd). The store root and the conventions roots both
  come from it. `dispatch`/`execute` take `&dyn Fn(&Path) -> Roots`, so roots are
  found only for `conventions`, and in-process tests still pass a no-layer root.
- Taught in the `learn` guide (DEC-010), the README command section, and
  `docs/store-format.md` § Layout (STD-001; a location rule, not a format change,
  so no version bump).
- Tests: `tests/cli_contract.rs` — the store follows `-C` (absolute and relative)
  and never the caller's directory; a missing `-C` directory exits 1 naming the
  absolute path and creates nothing; `--help` lists `-C <DIR>`.
  `tests/conventions.rs` — the project layer is found from the `-C` directory.
  `src/learn.rs` — the guide teaches the spelling.
- Mutation probe: conventions roots computed from the process cwd instead of the
  start directory → the conventions test went red. Reverted.

Environment note: this machine has an empty `/tmp/.git` (dated 2026-11-15). Every
test using `assert_no_repository_above` (8 existing, 1 new) fails under the
default `TMPDIR` for that reason alone; the gate was run green with
`TMPDIR=/var/tmp`. The operator then removed `/tmp/.git` (a leftover from their
own testing), and the suite is green under the default `TMPDIR`.
