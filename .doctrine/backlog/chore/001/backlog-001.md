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

## Done when

- The rewrite is complete and its examples execute under `tests/readme_example.rs`
  (exact fence shape), or the harness is deliberately widened with a reason.
- No declaration or response example contradicts the shipped binary at 0.2.
- `cargo publish` for 0.2 runs on a README that has been read against the binary.
