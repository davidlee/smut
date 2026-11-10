# CHR-007: The test harness is copied into eleven integration-test files; one tests/common module

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised 2026-11-10 while resolving IMP-015, which had to edit two copies of the same helper.

## What is duplicated

`Scratch` (a self-removing temp working directory) is defined 11 times, in two families:

- **CLI-process** (`cli_contract`, `compatibility`, `conventions`, `fixture`, `predicates`,
  `readme_example`, `traversal`): `Scratch { path }`, `Ran { code, stdout, stderr }`, a
  `run` that spawns `CARGO_BIN_EXE_smut`, and `smut`. The copies differ only in the
  temp-dir prefix, which subset of methods each file keeps, whether stdin gets an empty
  write or none (the same EOF), and `assert_one_json_line` (in 2 files).
- **Library-store** (`concurrency`, `durability_crash`, `roundtrip`, `store_corruption`):
  a `Scratch` whose `store()` returns a `smut::store::Store`, plus file-specific helpers.

Drift already present: only `conventions.rs` canonicalizes the scratch path (STD-003) and
isolates `$HOME` / `$XDG_CONFIG_HOME`. Every other CLI test spawns the binary against
the developer's real environment.

`tests/conventions.rs` records the copying as deliberate: "0.2 does not add a shared
`tests/common`". That was scoped to 0.2; the crate is at 0.3.0.

## Shape

`tests/common/mod.rs`, with `mod common;` in each test file:

- `Scratch`: `new()` (the prefix comes from `CARGO_CRATE_NAME`, the path is
  canonicalized), `path`, `store_dir` (`.smut`), `store` (`Store`), `has_store`,
  `store_entries`, `home`, `config_home`.
- `Ran`: `json`, `assert_one_json_line`.
- One spawn path, `Scratch::run_in(cwd, args, stdin)`, which always isolates both env
  vars, plus `run` and `smut` wrappers.
- File-specific helpers stay in their files as inherent `impl Scratch` blocks.

Done when per-binary test counts are unchanged under both feature sets and `just check`
is green.

## Resolution (2026-11-10)

Done as shaped, then widened to the identical free helpers found on the way.

`tests/common/mod.rs` now holds:
- `Scratch`: canonicalized path, `store_dir`, `store`, `has_store`, `store_entries`,
  `home`, `config_home`.
- `Scratch::command(cwd)`: the one place a run's environment is spelled.
- `run_in`, `smut_in`, `run`, `smut`; `Ran` with `json` and `assert_one_json_line`.
- `listing`, plus the in-memory and store-byte fixtures: `store` (×4 copies before),
  `keys` (×2), `parse` (×2), `entity`, `entity_bytes`, `index_bytes` (×2 each).

All 14 integration-test files now use it. Their own fixtures sit in `impl Scratch` blocks
(`conventions`, `concurrency`, `durability_crash`, `roundtrip`, `store_corruption`).
Net: −1140 / +177 lines.

Behaviour changes, deliberate:
- Every spawned binary now runs with an isolated `$HOME` / `$XDG_CONFIG_HOME`. Only
  `conventions.rs` did this before.
- Every scratch path is canonicalized (STD-003).
- `home/` and `config-home/` are not created up front, so a listing of the scratch root
  stays clean. The one fixture that writes inside `config-home`
  (`an_unreadable_layer_exits_one`) creates it.
- `store_corruption`'s hand-spawned `get` now goes through `smut()`.
- `concurrency`'s `spawn_in` builds on `Scratch::command` (stdin overridden to null);
  `StalePrecheck` borrows the `Scratch`.

Verified: per-binary test counts are identical to the pre-change baseline (default
features), `durability_crash` runs 48 under `fault-injection`, the per-file `#[test]`
counts equal HEAD, and `just check` is green.

Left alone: `ids_of`, `seed`, `selected_ids` and `store_of_three` each appear twice with
different bodies. They are different fixtures sharing a name, not copies.
