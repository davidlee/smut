# smut: dev shell and the crane-built binary. Agents and quotient forms come
# from the dev-envs consumer (`use_dev_envs` in .envrc).
{
  description = "smut: dev shell and package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # The Rust builder the package below is assembled with. It declares no inputs of
    # its own — crane is a library over whatever `pkgs` it is handed — so there is
    # nothing to `follows`, and adding one earns "override for a non-existent input
    # 'nixpkgs'".
    crane.url = "github:ipetkov/crane";
    doctrine.url = "github:davidlee/doctrine";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    nixpkgs,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devshell.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem = {system, ...}: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        inherit (pkgs) lib;

        # One statement of the channel: `beta.latest`, the same channel
        # `rust-toolchain.toml` names for rustup and for CI. It is a function so
        # crane's `overrideToolchain` can consume it directly.
        toolchainFor = p: p.rust-bin.beta.latest.default;

        # The devshell's compiler: same channel, plus the components an editor
        # wants and a build does not.
        rust = (toolchainFor pkgs).override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rust-docs"
          ];
        };

        # -- Customise these for the project --

        devToolPkgs = with pkgs; [
          just
          rust
          mold
          shellcheck
          cargo-edit
          stdenv.cc
          stdenv.cc.cc.lib
          sccache
        ];

        # -- The package --
        #
        # A devshell is not a build. This is the binary `cargo publish` also ships,
        # assembled from the checkout.
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchainFor;

        # `lib.fileset` rather than `craneLib.cleanCargoSource`, because the crate
        # is not only `.rs`: `tests/readme_example.rs` embeds `README.md` with
        # `include_str!`, and `tests/fixture.rs` reads
        # `tests/fixtures/interview/entities.json` at run time. A filter that kept
        # `.rs` and the manifests alone would fail the suite inside the build. The
        # fileset is the crate's payload, matching `Cargo.toml`'s `exclude` — the
        # governance tree, the flake and CI stay out.
        crateSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./README.md
            ./src
            ./tests
          ];
        };

        # `doCheck` stays on, so a package that builds is one whose `cargo test`
        # passed. Not `just check`: the fault-injection doubles belong to the gate,
        # and coupling them here would let a sandbox quirk read as a broken build.
        smut = craneLib.buildPackage {
          src = crateSrc;
          strictDeps = true;
        };
      in {
        _module.args.pkgs = pkgs;

        packages = {
          default = smut;
          inherit smut;

          dev-tools = pkgs.buildEnv {
            name = "smut-dev-tools";
            paths = devToolPkgs;
          };
          # What the room takes: `dev-tools` plus Doctrine, as a room has no host install
          # (flakes:DEC-130).
          capsule-tools = pkgs.buildEnv {
            name = "smut-capsule-tools";
            paths = devToolPkgs ++ [inputs.doctrine.packages.${system}.default];
            ignoreCollisions = true;
          };
        };

        # `nix flake check` builds the package on every declared system.
        checks = {inherit smut;};

        devshells.default = {
          packages = devToolPkgs;
        };
      };
    };
}
