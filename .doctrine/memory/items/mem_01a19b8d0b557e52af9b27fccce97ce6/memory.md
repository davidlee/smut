`.github/workflows/ci.yml` runs `just check` on `ubuntu-latest` and `macos-latest`, one job each, `fail-fast: false`. The first real run was green on both (operator report), including `cargo test --features fault-injection`, whose interruption tests abort real child processes — APFS tolerated the protocol.

Three mechanisms worth not re-deriving:

- **The toolchain is a property of the checkout.** `rust-toolchain.toml` names `channel = "beta"` (the same channel the flake's `rust-overlay` pins, `1.100.0-beta.7` locally at the time of writing) plus `clippy` and `rustfmt`; the workflow just runs `rustup show` and gets it. No `dtolnay/rust-toolchain` action, no second statement of the channel.
- **`just` comes from `taiki-e/install-action@v2` with `tool: just`** — the key resolves, verified by that green run. Caching is `Swatinem/rust-cache@v2`.
- **`.github/` and `rust-toolchain.toml` are excluded from the crate payload** (`Cargo.toml`'s `exclude`, checked with `cargo package --list`). The toolchain file is excluded on purpose: inside the tarball it would pin a downstream `cargo install` to a beta compiler.

The gate is still `justfile`'s `check` recipe and nothing else — CI and the devshell share one definition; `doctrine check gate` defaults to the same recipes.

Not covered: Intel macOS (`macos-latest` is arm64), Windows.