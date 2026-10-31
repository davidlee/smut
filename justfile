build:
  cargo build

run:
  cargo run

check: lint
  echo write some tests dummy

lint:
  echo set up clippy
