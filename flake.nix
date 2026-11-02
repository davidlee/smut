# Template: jailed LLM agents
#
# Copy into project, then:
#   1. Adjust projectPkgs for the language/toolchain
#   2. Set workspaceDeps (read-only) and workspaceDepsRw (read-write) to
#      sibling repos the agents need to see
#   3. Tune jailEnvOptions: forward host env vars into the jail
#   4. Trim jailPkgs to the agents actually wanted
#   5. Jails use the LLM gateway; see `gateway` below to opt out
#   6. quotient forms: `quotient-forms` links them, then `quotient allow`
#
# Jails reach models through the local LLM gateway (`gateway` below).
#
# Usage:
#   mv _envrc .envrc && direnv allow   # live agents: agent bumps need no lock update
#   nix build                          # the binary, assembled by crane
#   nix develop
#   jailed-claude   # or jailed-pi, jailed-pi-research, jailed-codex
#   jcl             # shorthand: jailed-claude --dangerously-skip-permissions
{
  description = "Dev shell with jailed LLM agents";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # The Rust builder the package below is assembled with. Following nixpkgs keeps
    # one universe for the shell and the build.
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    agents = {
      url = "github:davidlee/nix-config?dir=flakes/agents";
      #inputs.nixpkgs.follows = "nixpkgs";
    };
    # quotient's form-family builder, for quotientForms below.
    quotient = {
      url = "github:davidlee/quotient/v0.6.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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

        inherit (pkgs) lib stdenv;
        inherit (stdenv) isLinux;

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

        # jail.nix is Linux-only (bubblewrap). Darwin gets a plain devshell.
        jailLib =
          if isLinux
          then inputs.agents.lib.${system}.mkJailedAgents {}
          else {};

        # Unjailed agent CLIs (llm-agents builds) under short names. Linux
        # only; on Darwin fall back to nixpkgs. To get them on `pkgs`
        # directly instead, apply the overlay at the top of perSystem:
        #   _module.args.pkgs = import inputs.nixpkgs {
        #     inherit system; config.allowUnfree = true;
        #     overlays = [ (inputs.agents.lib.${system}.agentsOverlay {}) ];
        #   };
        agents = lib.optionalAttrs isLinux jailLib.agentsByName;

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

        projectPkgs =
          devToolPkgs
          ++ (with pkgs; [
            (agents.codex or codex) # mcp server slave — llm-agents build on linux
          ]);

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

        # Sibling repos to bind-mount (for editable deps / source inspection).
        # Each path appears at /workspace/<basename> inside the jail; a host
        # symlink like ./some-lib -> ../some-lib resolves correctly.
        # workspaceDeps bind read-only; list a dep the agents must write in
        # workspaceDepsRw instead. Machine-local additions: the .envrc's
        # JAIL_WORKSPACE_DEPS and JAIL_WORKSPACE_DEPS_RW.
        workspaceDeps = [
          # "/home/you/dev/some-lib"
        ];
        workspaceDepsRw = [
          # "/home/you/notes"
        ];

        # Interactive jails (`jailed-claude`, ...): the cwd at
        # /workspace/deck_of_dwarf, a persisted home, network on, SSH push
        # blocked.
        #
        # Doctrine records `current_exe()` in what it writes (.codex/hooks.json
        # here), so the jail presents the host's install at the host's path, not
        # the lib's /run/doctrine-bin; otherwise a `doctrine install` in a jail
        # rewrites the recorded binary. A directory bind, not the file: `cargo
        # install` renames a new binary over the old inode. `$HOME` is quoted so
        # the launching shell expands it. As malebolge and panopticon; belongs
        # upstream in the agents lib's hostDoctrineOptions.
        doctrineDir = "\"$HOME/.local/share/doctrine/bin\"";
        doctrineOptions = with jailLib.combinators; [
          (try-ro-bind (noescape "~/.local/share/doctrine/bin") (noescape doctrineDir))
          (add-path doctrineDir)
        ];

        # bwrap starts with a clean env. Forward / set what agents need.
        # Combinators: set-env, fwd-env (error if unset), try-fwd-env (skip if unset),
        # try-readwrite (bind a host path rw if it exists).
        jailEnvOptions =
          lib.optionals isLinux (with jailLib.combinators; [
            # pulse # opt-in host audio; uses $XDG_RUNTIME_DIR
            # (try-readonly "/path/on-this-host/notification.wav")
            # (try-fwd-env "DATABASE_URL")
            # (try-fwd-env "DOCKER_HOST")
            # (try-readwrite "/path/to/docker.sock")
            # (set-env "CGO_ENABLED" "0")
          ])
          ++ doctrineOptions;

        # Every jail reaches models through the local LLM gateway with the shared
        # `pi` key (~/.config/llm-gateway/keys/pi); no provider key enters a jail.
        # It is the only model access (agents/README.md § Model access).
        gateway = jailLib.gatewayClient "pi";

        # -- Agents --
        #
        # Profiles:
        #   specDev   — shared persistent home, network on, SSH push blocked, sandboxed identity
        #   research  — separate persistent home, network on, SSH push blocked, host identity
        #   offline   — separate persistent home, no network
        #
        # Extra per-agent knobs (all optional):
        #   exposePostgres       = true;   # bind /run/postgresql into the jail
        #   blockSshGitPush      = false;  # SSH only; HTTPS remains available
        #   allowSelfAsSubagent  = true;   # let agent recursively spawn itself
        #   maxSubagentDepth     = 2;      # cap recursion depth (default 1)
        #   gateway              = null;   # no model, unless the agent signs in itself

        jailPkgs = lib.optionalAttrs isLinux {
          jailed-pi = jailLib.makeJailedPi {
            profile = "specDev";
            allowSelfAsSubagent = true;
            maxSubagentDepth = 2;
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
            inherit gateway workspaceDeps workspaceDepsRw;
          };

          jailed-pi-research = jailLib.makeJailedPi {
            name = "pi-research";
            profile = "research";
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
            inherit gateway workspaceDeps workspaceDepsRw;
          };

          jailed-claude = jailLib.makeJailedClaude {
            profile = "specDev";
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
            inherit gateway workspaceDeps workspaceDepsRw;
          };

          jailed-shell = jailLib.makeJailedAgent {
            name = "shell";
            agent = pkgs.zsh;
            extraPkgs = projectPkgs;
            subagents = ["pi" "claude"];
          };

          inherit (pkgs) bubblewrap;
        };

        # quotient's per-call forms for this root (`quotient help form`): the
        # jails quotient's `pi_agent` runs a delegate in, on the gateway's pi
        # key, with the project's toolchain.
        quotientForms = lib.optionalAttrs isLinux (let
          family = jailLib.quotientFormFamily {
            inherit (inputs) quotient;
            packages =
              projectPkgs
              ++ (with pkgs; [
                bash
                findutils
                gnugrep
                gnused
                gawk
                ripgrep
                fd
                git
              ]);
          };
        in {
          quotient-read = family.read;
          quotient-write = family.write;
        });
      in {
        _module.args.pkgs = pkgs;

        packages = jailPkgs // quotientForms // {
          default = smut;
          inherit smut;

          dev-tools = pkgs.buildEnv {
            name = "smut-dev-tools";
            paths = devToolPkgs;
          };
        };

        # `nix flake check` builds the package on every declared system.
        checks = {inherit smut;};

        devshells.default = {
          packages =
            projectPkgs
            ++ lib.optionals isLinux (lib.attrValues jailPkgs);

          commands = [
            {
              name = "jcl";
              help = "jailed-claude --dangerously-skip-permissions";
              command = "jailed-claude --dangerously-skip-permissions $@";
            }
            {
              name = "quotient-forms";
              help = "link .quotient/forms/{read,write}; then `quotient allow`";
              command = ''
                mkdir -p .quotient/forms
                for c in read write; do
                  nix build -o ".quotient/result-$c" ".#quotient-$c"
                  ln -sfn "$(readlink -f ".quotient/result-$c")/bin/quotient-$c" ".quotient/forms/$c"
                done
              '';
            }
          ];
        };
      };
    };
}
