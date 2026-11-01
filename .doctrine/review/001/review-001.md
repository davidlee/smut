# Review RV-001 — design of SL-001

Adversarial-review ledger. Structured findings live in the sister ledger toml;
this prose companion carries the reviewer's framing.

## Brief

**What this pass probes.** That `design.md` (twelve sections, revision 29) can be
implemented by someone who was not in the conversation, without inventing
semantics. The design's own claim is that eleven decision records carry the
detail and the prose carries the structure; the pass tests exactly that claim.

**Invariants it holds the design to.**

1. Every observable behaviour in the brief is either stated in a section or
   delegated by citation to a `DEC` that states it.
2. No section contradicts another, or contradicts the accepted decisions.
3. An implementor cannot read a section and produce behaviour that violates the
   brief's exit codes, envelopes or durability obligation.
4. Diagrams agree with the prose, and carry the branches that matter.
5. The code-impact section names every path the design requires, and the
   design-target selectors match it.

**Where the bodies are likely buried.**

- The **command pipeline** (§5) is the one place where a missing branch silently
  changes an exit code, so the diagram was checked against DEC-004 line by line.
- The **commit protocol** (§4.3–4.4) is described as a state machine; the probe
  was whether the *write set* is fully determined, since the crash tests assert
  against it.
- The **module map** (§2) versus the **code impact** table (§12) is where a
  gated module or a missing file hides.
- §6.1's numeric comparator was re-checked for the transitivity defect that
  killed its first version.

**Out of scope for this pass.** Implementation planning (phase decomposition,
file-by-file sequencing) and the contents of the later releases, which have
their own design runs.
