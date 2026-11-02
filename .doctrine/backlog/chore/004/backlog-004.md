# CHR-004: The gate has no continuous integration: macOS is untested, and the portability claim says so

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`justfile` is the one definition of the gate — build, `clippy` over both feature
configurations, `fmt --check`, `test` twice (default, then `fault-injection`) — and
`doctrine check gate` / `prove` already default to those recipes rather than to a
second copy of the commands. Nothing runs that gate off this machine: there is no
`.github/`, so every green result the tree has ever had is a local one, on Linux.

`docs/store-format.md` § Portability states the position exactly: *"Linux is the
acceptance platform. The format is plain files and directories, and nothing in it is
Linux-specific by intent, but nothing else is tested."* Adding a macOS job narrows
that sentence — the suite would run there — so the document must say what the job
proves (the suite passes on macOS) and what it still does not (the `flock` and
`fsync` durability promise stays assumed, never measured). STD-001 makes that
document the format's only record, so it is part of this chore, not a follow-up.

Two more things are only visible from a build that is not local: the flake declares
`aarch64-darwin` among its systems, and GitHub's `macos-latest` is arm64 — so the
runner matches the flake's darwin system rather than the developer's host, and the
first thing CI does that the devshell never did is compile and test on it.

## What is already decided and must not be re-derived

- **Run the gate once.** The workflow calls `just check`; it does not restate the
  cargo lines. A second definition of the gate is the drift the justfile comment
  already argues against.
- **macOS alone would be the wrong matrix.** Linux is the acceptance platform, so a
  macOS-only job would exercise only the platform the format does not promise.
  Decided: both, one job each, same recipes.
- **The toolchain is unpinned outside the flake.** There is no `rust-toolchain.toml`;
  the flake pins `rust-overlay`'s `beta.latest` with `rust-src`, `rust-analyzer` and
  `rust-docs`. A runner defaulting to `stable` silently tests a different compiler
  than the devshell. Decided: a `rust-toolchain.toml` naming `beta` plus the two
  components the gate runs, so the compiler is a property of the checkout rather
  than of the workflow. Edition 2024 sets the floor at 1.85.
- **Do not gate the job off to get green.** The first run is expected to find
  something; its red result is information to triage, not a reason to disable it.

## Sharp edges

- `tests/durability_crash.rs` is `#![cfg(feature = "fault-injection")]` and asserts
  by aborting real child processes; unix-only code also sits at
  `src/store/journal.rs:1580`, `src/store/fault.rs:278`, `src/store/mod.rs:1265`
  and `:1285`, and `tests/store_corruption.rs:328` and `:351`. macOS is unix, so
  this compiles — but APFS is not the filesystem the protocol was written for, and a
  failure there is a finding about portability, not a flake to retry away.
- `fs4` takes an advisory `flock`; the format document already limits the promise to
  cooperating commands. A CI runner has no power-loss to simulate either way.
- **`.github/` would ship in the crate.** `Cargo.toml`'s `exclude` names the
  governance and flake files explicitly and does not name `.github`, so the workflow
  would enter the `cargo package` payload. Extend `exclude` in the same change.

## Done when

- A workflow exists that runs the project's gate on macOS, and on Linux if that call
  went that way, at the same recipes the devshell and `doctrine check gate` use.
- The job is real: a deliberately broken lint or test turns it red and the revert
  turns it green again.
- `docs/store-format.md` § Portability has one sentence stating what CI now
  exercises, without promoting macOS to an acceptance platform.
- `Cargo.toml`'s `exclude` keeps `.github/` out of the published crate, verified with
  `cargo package --list`.

## Progress

Started, and in the tree uncommitted alongside the operator's own 0.2.2 bump.

- `.github/workflows/ci.yml` — one `gate` job, a two-OS matrix (`ubuntu-latest`,
  `macos-latest`), `fail-fast: false`, 30-minute timeout, `contents: read`, a
  concurrency group that cancels superseded runs, and exactly two steps that
  matter: a toolchain assertion and `just check`. `just` arrives from
  `taiki-e/install-action@v2` (`tool: just`), builds from `Swatinem/rust-cache@v2`.
- `rust-toolchain.toml` — `channel = "beta"` with `clippy` and `rustfmt`, the
  channel the flake already names, resolved by `rustup show` in CI and by rustup
  anywhere else. The flake does not read it yet; teaching it to
  (`rust-overlay`'s `fromRustupToolchainFile`) would collapse the last two
  statements of one fact into one.
- `Cargo.toml` — `exclude` gained `.github/` and `rust-toolchain.toml`. The second
  one is deliberate: a toolchain file inside the tarball would pin a downstream
  `cargo install` to beta. `cargo package --list` confirms neither ships.
- `docs/store-format.md` § Portability — one paragraph now says a macOS runner
  builds and runs the whole gate, and that this is evidence of portability rather
  than a second acceptance platform, with no new claim about `flock` or `fsync` on
  APFS.

Verified locally: the workflow parses as YAML with the expected structure;
`cargo build`, `just fmt`, and `just lint` are green; `cargo test --features
fault-injection` is green apart from the item below.

## Result

Green on both platforms, first run: `ubuntu-latest` and `macos-latest`, the whole gate
on each — the lib suite, the 50-test CLI contract, and the fault-injection suite with
its real child-process aborts, on APFS. The portability doubt this chore existed to
answer is now evidence: the suite compiles and passes on the platform
`docs/store-format.md` declines to promise, and the promise itself is unchanged.

Both items that were open at authorship closed on that run: `taiki-e/install-action@v2`
resolves `tool: just` (the one line in the workflow that carried an assumption), and
the red gate was the `0.2.2` pin, moved with the bump before the run (`9abe6a5`).

Being exact about measured versus constructed, because the difference will matter to
whoever reads this next:

- **Measured.** The job runs the project's own recipes on both platforms and goes
  green. `cargo package --list` shows `.github/` and `rust-toolchain.toml` out of the
  crate. § Portability says what CI exercises and no more.
- **By construction.** That a red gate reddens the *job*. `just check` was red twice
  in this work (the `0.2.2` pin, the README fences) and its non-zero exit is the
  step's exit code, with no `continue-on-error` and no retry — but nobody has watched
  the job fail. If that ever needs proving, break one lint on a throwaway branch.

Not covered, deliberately: `macos-latest` is arm64, which is the darwin system the
flake declares. Intel macOS is not run, and Windows is not contemplated.

## Follow-Ups

- **A flake package build** (`packages.<system>.default`) is the other half of "the
  binary builds somewhere that is not this laptop", and it is deliberately out of
  scope here: a devshell is not a build, and one thing at a time. Promote it to its
  own card when CI is green.
- **Publish automation** (`cargo publish` on a tag) sits behind CHR-001's README
  rewrite and none of this chore.
