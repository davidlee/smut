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
- **The release paragraph is not missing after all.** crates.io carries 0.2.2, the
  same version this tree is, so `cargo install smut` above "That'll get you one
  self-contained binary" is simply true and the page owes no qualification.
  `the_readme_names_the_published_release_beside_this_tree` asserted the earlier
  arrangement — published 0.1.1, an unpublished tree at 0.2.1, a paragraph naming
  both — and could only be satisfied now by writing a distinction that no longer
  exists, so it is retired. Its own literal had moved with the bump first (9abe6a5,
  c0b526d), which is how the staleness became visible.
- **The guard lapses, the risk does not.** RV-017's failure mode — over-claiming in
  composition while no single sentence is false — returns the moment a bump outruns
  crates.io: the page would again promise an install of a version the registry does
  not carry, and no offline test can see the registry. Re-arm it when the next
  unpublished bump lands, as a version-consistency test over whatever the page names
  or a checklist line before publish.

## Done when

- The rewrite is complete and its examples execute under `tests/readme_example.rs`
  (exact fence shape), or the harness is deliberately widened with a reason.
- No declaration or response example contradicts the shipped binary at 0.2.
- `cargo publish` for 0.2 runs on a README that has been read against the binary.
