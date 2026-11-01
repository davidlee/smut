# Thread 2 — substrate / prior-art map (verbatim notes)

Runner: orchestrating agent (degraded mode). Read-only. Date 2026-11-01.
Because the repository is greenfield there is no code to map; the equivalent
thread is the mechanism map for the decisions the brief delegates to the
implementer (crates, protocol, escaping, test harness).

## Local verification (`cargo info`, 2026-11-01)

    serde        1.0.229  MIT OR Apache-2.0   rust-version 1.56
    serde_json   1.0.154  MIT OR Apache-2.0   rust-version 1.71
    nanoid       0.5.0    MIT                 rust-version unknown
    fs4          1.1.0    MIT OR Apache-2.0   rust-version 1.75.0
    clap         4.6.7    MIT OR Apache-2.0   rust-version 1.85

Toolchain in jail: rustc/cargo 1.100.0-beta.7, clippy 0.1.100, rustfmt
1.11.0-beta, mold 3.0.0, just 1.58.0, sccache 0.19.0. Edition 2024 set in
Cargo.toml already.

## Web-sourced facts

1. `serde_json::Value` does not detect duplicate object keys. Its `Map` keeps
   the last occurrence, so the duplicate is unrecoverable after parsing.
   Source: serde-rs/json issue #1112 "Checking for duplicate keys".
   Consequence: rejection must happen during deserialisation, via a visitor
   whose `visit_map` checks each key before inserting.
2. `serde_json::Map` is a `BTreeMap` unless the `preserve_order` feature is
   enabled, so `Value` equality is key-order-insensitive by default and
   serialisation is already deterministic (sorted keys).
3. `clap` exits with status 2 on argument errors (`USAGE_CODE: i32 = 2` in
   `clap_builder/src/util/mod.rs`), matching the brief's usage exit code.
4. Directory fsync is a separate durability step from file fsync: `rename`
   and `create` mutate the directory, so a durable rename is
   temp-write → fsync file → rename → fsync parent directory.
   Sources: "Why the Directory Needs fsync Too" (mehdi.cz); "How to Durably
   Write a File on POSIX Systems" (calvin.loncaric.us); `durability::storage::
   sync_parent_dir` docs.
5. Recoverable multi-file transaction prior art has a stable three-state
   shape: stage bytes and fsync each; write a journal and fsync; write a
   COMMITTING marker and fsync the directory; apply renames/removals; remove
   COMMITTING. A crash at any instant leaves pre-commit (discard) or
   post-commit (roll forward) — never a mix.
   Sources: `reldir::transaction` module docs; `fs-transaction` journal
   (`journal.rs`); `atomicwrites` crate (rustix-based atomic replace).
6. Mermaid flowcharts: quoted labels carry arbitrary text; an embedded double
   quote is written with the entity code `#quot;` (`A["She said: #quot;hi#quot;"]`).
   Source: mermaid.js.org "Flowcharts Syntax".
7. `CARGO_BIN_EXE_<name>` is provided to integration tests so they can execute
   the built binary. Source: The Cargo Book, "Cargo Targets"; rust-lang/cargo
   PR #7697.
8. Rejected candidate: `assay_evidence::json_strict` (docs.rs) offers strict
   duplicate-key `Value` parsing, but is an unrelated project's evidence crate
   — not a dependency to take for one behaviour we can implement in ~40 lines.

## Notes carried forward

- `serde_json::Number` distinguishes `1` (u64) from `1.0` (f64): structural
  equality therefore has a documented sharp edge that must be pinned in tests.
- No project-defined research agents exist; the round ran degraded (see
  header).
