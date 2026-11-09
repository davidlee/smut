# CHR-002: src/error.rs attributes the duplicate-key refusal to DEC-006, which is the ordering decision

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Documentation defect only.** No behaviour is wrong; nothing about the refusal,
its exit code or its message needs to change. Re-verified 2026-11-02 during SL-005's
audit (RV-027 F-7), which raised it as a `nit` and routed it here.

## What is wrong

`src/error.rs:101-103`, the doc comment on the `DuplicateKey` variant:

> One JSON object in the input gives the same key twice, which would lose the earlier
> assignment. Refused as the key is read, before a `Value` exists (§3.1, DEC-006).

Both citations are wrong, and for the same reason — neither document governs this.

- **DEC-006** is *0.1 ordering: per-key missing-last, exact numeric comparator, total
  order via ID tiebreak*. It has nothing to do with duplicate keys.
- **§3.1** is SL-001's design section *What an object means* — entity node, reference,
  declaration hoisting, `id` validity (governed by DEC-005). Also not this. The `§3.1`
  shorthand is used correctly elsewhere in `src/` for that section's entity-node rule
  (`src/entity.rs:48`, `src/store/journal.rs:256`), so the reference reads plausibly and
  is easy to copy onward.

The authority is **DEC-001 rule 1**: *"User-input rejection is exit 2, whichever layer
detects it. The shell rejects malformed JSON, duplicate JSON keys and invalid CLI
arguments — all exit 2."* That sentence is the source of both the refusal and its exit
code.

## What is right, and must stay

- **The substantive claims in the comment are true.** `src/input.rs`'s `Strict` visitor
  records the repeated key while deserializing (`:78`, with the collection sites at
  `:247`, `:257`, `:267`), so the refusal really does happen as the key is read, before
  any `Value` exists.
- **Exit 2 is correct and covered**: the boundary table row is `INVALID_INPUT`, and
  `a_duplicate_key_is_refused_while_the_operand_is_parsed` pins the process behaviour.

## The fix

One comment edit: replace `(§3.1, DEC-006)` with the governing citation, e.g.
`(DEC-001 rule 1)`, and keep the rest of the sentence. No code, no test, no message
change.

## Why it is not fixed yet

`src/error.rs` is SL-004's selector, and SL-004's PHASE-02 (the identity-message respell)
is editing that file in the main worktree as this is written. Landing a comment change
there from another slice would race a live writer for a nit. Take it with SL-004's
PHASE-02, or immediately after it lands.

## Resolution (2026-11-10)

Fixed as specified: `src/error.rs` `DuplicateKey` doc comment now cites `(DEC-001 rule 1)`.
Comment-only; no other `src/` site misattributes DEC-006.
