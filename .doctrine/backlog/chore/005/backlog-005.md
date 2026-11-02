# CHR-005: The flake builds a devshell and nothing else: there is no package for the binary

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`flake.nix` is a devshell: `packages` holds the jails, the quotient forms and
`dev-tools`, and nothing in it is `smut` itself. The binary exists only as
`cargo build --release` on whoever's machine, or as 0.2.2 on crates.io. A devshell
is not a build — it pins a compiler and a pile of tools, and says nothing about
whether the crate assembles reproducibly.

The operator has used **crane** for this elsewhere and it works well, so the shape
is settled: `crane.mkLib` over the flake's existing `rust-overlay` toolchain, one
`buildPackage`.

## Decisions

- **Crane, with no input of its own.** `crane.url = "github:ipetkov/crane"`, and
  that is all. The first attempt also set `inputs.nixpkgs.follows = "nixpkgs"`, which
  nix refused in the only way it can — a warning that the override names a
  non-existent input — because this crane declares no inputs: it is a library over
  whatever `pkgs` it is handed. Nothing followed, nothing needed to: the lockfile
  gained one node and no second nixpkgs, which is what the `follows` was trying to
  achieve by a mechanism that did not apply here.
- **`stdenv.hostPlatform.isLinux`, not `stdenv.isLinux`.** The template's
  `inherit (stdenv) isLinux` is deprecated and `nix flake check` says so. Same
  question, asked of the attribute that still answers it.
- **One statement of the channel, two derivations by role.** `toolchainFor = p:
  p.rust-bin.beta.latest.default` is the single place the compiler is named —
  `beta.latest`, the channel `rust-toolchain.toml` names for rustup and CI. The
  devshell overrides it with `rust-src`, `rust-analyzer` and `rust-docs`; the
  package build takes it bare, so the build closure does not carry an editor's
  component set.
- **`lib.fileset`, not `craneLib.cleanCargoSource`.** The crate is not only `.rs`.
  `tests/readme_example.rs` embeds `README.md` with `include_str!`, and
  `tests/fixture.rs` reads `tests/fixtures/interview/entities.json` at run time; a
  source filter that kept `.rs` and the manifests alone would fail the suite inside
  the derivation. The fileset names `Cargo.toml`, `Cargo.lock`, `README.md`, `src/`
  and `tests/` — the crate's own payload, not the repository (`.doctrine/`,
  `.github/`, the flake and `AGENTS.md` stay out, per `Cargo.toml`'s `exclude`).
- **The package is also a check.** `checks.smut = smut`, so `nix flake check`
  builds it on every declared system, and `packages.default` makes `nix build` do
  it without naming the attribute.
- **The build runs `cargo test`, not `just check`.** `buildPackage`'s `doCheck`
  is left on, so a package that builds is one whose tests passed. The
  fault-injection doubles belong to the gate — packaging is not the place to
  learn whether the crash suite is green, and coupling them would make a sandbox
  flake look like a build failure.

## Not in scope

- **Wiring `nix build` into CI.** Not because it is blocked — every input was
  checked and is publicly fetchable (`ipetkov/crane`, `davidlee/nix-config`,
  `davidlee/quotient` all answer 200), so a runner could evaluate this flake with no
  credential. It is a second job with its own install step (ubuntu has no nix;
  macOS images do), its own cold-cache time (the rust beta toolchain arrives as a
  fixed-output fetch, then crane compiles the crate), and its own failure surface.
  Its own decision, not a rider on this one.
- **Intel macOS and Windows.** `systems` stays `x86_64-linux` and
  `aarch64-darwin`, matching what CI runs and what the store format promises.
- **Publishing.** crates.io already carries 0.2.2; this builds the same binary a
  nix user can install without a registry.

## Verification

- `nix build` → `./result/bin/smut --version` prints `smut 0.2.2`, and
  `./result/bin/smut learn` prints the embedded guide (the `include_str!` payload
  survived the fileset).
- `nix flake check` builds the same on both declared systems.
- `nix develop` still gives the devshell it gave before (the toolchain split must
  not change what an interactive session sees).

## Result

`nix build` is green (operator, on their machine). The bare `buildPackage` was
enough: crane vendored from `Cargo.lock` on its own, the `lib.fileset` source
carried the tests, and the suite compiled and passed inside the sandbox. The
`toolchainFor` split did not disturb the devshell.

`flake.lock` pins crane — `ipetkov/crane`, rev `8782adaf` — and the graph gained one
node and **no second nixpkgs**. Not because a `follows` held: crane has no inputs to
follow (see Decisions). Before the pin the build was running against an unlocked
input, which is the failure mode the pin closes.

Reported green: `nix build` (the default package) and `nix build .#smut` (the named
attribute), so both entry points into the same derivation resolve.

`nix develop` is confirmed too, and it closes the one thing the split could have
broken: `nix develop --command rustc --version --verbose` prints `1.100.0-beta.7`,
host `x86_64-unknown-linux-gnu`. That number is the point — the devshell, the crane
build and the CI runner (through `rust-toolchain.toml`'s `channel = "beta"`) all
resolve the same channel. The split made the *derivations* differ by role, never the
channel they come from: the build gets `toolchainFor` bare, the shell gets
`(toolchainFor pkgs).override { extensions = …; }`, and `rustc` in both is the same
compiler.

Not run: `nix flake check` — the same derivation, entered as a check. Nothing else in
this chore is unlooked-at.

## Unverified-by-construction

This checkout has no `nix` on `PATH`, no `/nix/var/nix/db`, and no daemon socket,
so nothing here could evaluate the flake. Everything above rests on the operator's
run, not on a measurement this session made.

## Follow-Ups

- A `nix flake update` cadence: `rust-overlay`'s `beta.latest` moves under the
  package the same way it moves under the devshell.
- Reading the channel from `rust-toolchain.toml` via rust-overlay's
  `fromRustupToolchainFile` would collapse the last two statements of one fact
  (the file, and `toolchainFor`) into one.
