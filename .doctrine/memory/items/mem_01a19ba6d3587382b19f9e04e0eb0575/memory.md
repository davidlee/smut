`flake.nix` builds the binary through crane. Green via `nix build` on the operator's machine; nothing in the jailed session can evaluate nix (no binary, no `/nix/var/nix/db`, no daemon socket).

The shape, and why each part is that way:

- `crane = { url = "github:ipetkov/crane"; inputs.nixpkgs.follows = "nixpkgs"; }` — one nixpkgs for shell and build.
- `craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchainFor;` where `toolchainFor = p: p.rust-bin.beta.latest.default` is the **single statement of the channel**. The devshell overrides it with `rust-src`/`rust-analyzer`/`rust-docs`; the build takes it bare so the build closure carries no editor components.
- `smut = craneLib.buildPackage { src = crateSrc; strictDeps = true; }` — deliberately bare. Crane vendored from `Cargo.lock` on its own; no `cargoDeps`/`cargoVendorDir` needed.
- `crateSrc = lib.fileset.toSource { root = ./.; fileset = lib.fileset.unions [ ./Cargo.toml ./Cargo.lock ./README.md ./src ./tests ]; }` — **not** `craneLib.cleanCargoSource`, because the crate is not only `.rs`: `tests/readme_example.rs` does `include_str!("../README.md")` and `tests/fixture.rs` reads `tests/fixtures/interview/entities.json` at run time. An `.rs`-only filter breaks the suite inside the build.
- `packages.default` and `packages.smut` are the same derivation, and `checks.smut` too, so `nix build` works unqualified and `nix flake check` builds it on every declared system.
- `doCheck` is left on: packaging asserts `cargo test` (default features). The fault-injection suite stays with `just check` — coupling it here would let a sandbox quirk read as a broken build.

Caveat found after the green build: `flake.lock` still had no `crane` node, so nix used the input unlocked (dirty-tree path). `nix flake lock` must be run and `flake.lock` committed, else the next clone resolves crane to whatever `main` is that day.

CI does not build the package. Every input is publicly fetchable (`ipetkov/crane`, `davidlee/nix-config`, `davidlee/quotient` all 200), so that is a decision, not a credential blocker.