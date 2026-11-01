# smut's gates: one word for the checks §11.4 names.

build:
    cargo build

# Default features, then the fault-injection feature, so a gated suite cannot pass vacuously.
test:
    @echo "── cargo test · default features ──"
    cargo test
    @echo "── cargo test · --features fault-injection ──"
    cargo test --features fault-injection

# Both feature configurations, so feature-gated code is linted too.
lint:
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --check

check: build lint fmt test
