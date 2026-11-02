# CHR-001: README rewrite for 0.2 must land before cargo publish — and the gate is blind to it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The operator is partway through a full README rewrite for the 0.2 release, and will
finish it before `cargo publish`. The rewrite is operator-owned and in flight: the
working tree carries staged and unstaged README changes that no agent may sweep,
restage or resolve.

This is the same surface SL-004 moves (`id` → `smut_id`, format version 1 → 2), in
the opposite direction: the in-flight README already claims `smut_id` while the
binary still speaks `id`, and SL-004 deliberately does not own the file. The
boundary is recorded in SL-004's plan: the README lands outside that slice.

## The sharp edge

`tests/readme_example.rs:78` matches an exact ```` ```console ```` fence. The
README's in-flight examples sit in a ` ``` console ` block (with a space), which the
harness therefore never matches — so the gate is green over README examples that are
never executed and may contradict the binary. Watch for this in the rewrite: use the
exact fence the harness matches (`console`, no space), or the examples stay
unverified.

## Progress

- **The fence defect is fixed, and the harness was not widened.** The two blocks under
  `## Getting smut` were ` ``` console ` — the exact spelling `transcripts()`
  deliberately fails on. They are not transcripts and never could be: the harness
  runs each `console` line as a `smut` invocation and compares it with the line
  below, and `cargo install smut` is not one. They are `sh` now, as the pre-0.2
  build block was. With the parser no longer aborting on them, every remaining
  transcript runs and matches its recorded output: 9 of 10 tests pass.
- **The release paragraph is still missing, so the over-claim guard is red.**
  `the_readme_names_the_published_release_beside_this_tree` fails because the page
  now names no version at all, while `cargo install smut` sits directly above
  "That'll get you one self-contained binary" — false while crates.io carries
  0.1.1. The paragraph that said this plainly is in `git show 09708c8:README.md`
  (`## Install`). Restoring it is the operator's prose, not an agent's edit.
- **Its version literal moved with the bump.** `tests/readme_example.rs` now looks
  for `smut 0.2.2`, matching `tests/cli_contract.rs` (9abe6a5). The literal is right
  and the test still fails, which is the correct state: the sentence it asks for
  does not exist yet.

## Done when

- The rewrite is complete and its examples execute under `tests/readme_example.rs`
  (exact fence shape), or the harness is deliberately widened with a reason.
- No declaration or response example contradicts the shipped binary at 0.2.
- `cargo publish` for 0.2 runs on a README that has been read against the binary.
