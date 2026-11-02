# The situation

One worktree, several live slices. SL-004's PHASE-02 was landing while SL-005 reached
audit→reconcile→close. At close, `doctrine check gate` (which mirrors `just check`: build,
clippy ×2 with `-D warnings`, `fmt --check`, `cargo test`, `cargo test --features
fault-injection`) was red — but only in the co-tenant's files: `tests/cli_contract.rs` was
unformatted and five `entity::`/`store::index::` unit tests were failing, all of them
mid-phase SL-004 work.

# The wrong moves

- `git stash` / `git checkout --` / reformatting their files: destroys or mutates another
  writer's in-flight state, and the super-orchestrator envelope forbids all three.
- Disabling `fmt` or `clippy` to obtain green: never.
- Claiming green from an earlier run: the claim must attach to the commit being closed.
- Refusing to close at all: SL-005's work was complete and its own suites green; a
  co-tenant's mid-phase red is not a defect in the slice being closed.

# The move

The gate is a property of a **commit**, not of a working directory. Run it where nothing
else is writing:

    git worktree add --detach /tmp/close-gate <closing-commit>
    cd /tmp/close-gate && CARGO_NET_OFFLINE=true \
      (cargo build && cargo clippy --all-targets -- -D warnings \
       && cargo clippy --all-targets --all-features -- -D warnings \
       && cargo fmt --check && cargo test && cargo test --features fault-injection)
    git worktree remove --force /tmp/close-gate

Mirror the project's gate recipe exactly (`justfile`, or whatever `doctrine check gate`
resolves to) so the isolated run is the same sequence, not a weaker one. Commit the
workflow artefacts **first**, so the commit under test is the commit being closed.

# The record

Write the gate evidence into the RV's `## Reconciliation Outcome` (or the close note) with
the distinction stated plainly: *the closing commit's tree is green, verified in a detached
worktree; the shared working tree is red because slice N is mid-phase — that is slice N's
gate*. Never let "green" travel without the tree it was measured in.
