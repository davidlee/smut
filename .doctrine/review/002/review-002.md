# Review RV-002 — design of SL-001

Adversarial-review ledger. Structured findings live in the sister ledger toml;
this prose companion carries the reviewer's framing.

## Brief

**Why this pass exists.** It re-establishes coverage over the *settled* design.
RV-001 was opened over the draft at revision 33 and concluded after three
findings were integrated into §2, §4 and §5 — which moved the content the pass
was covering, so the run correctly marked it stale. Returning to drafting and
re-entering reviewing opened this pass over the current materialisation
(rev 37, twelve sections, `changes since baseline = 0`).

**Inherited, already verified.** RV-001's three findings — the missing
absent-store branch in §5's pipeline, the unstated "every commit stages a fresh
index" rule, and the ungated `store/fault.rs` entry in the module map — are
fixed in the design and verified on that ledger. This pass does not re-litigate
them; it checks that the fixes are coherent with everything around them and
looks for what they displaced.

**Invariants, unchanged from RV-001.** Every brief-mandated behaviour is stated
or delegated by citation to a DEC that states it; no section contradicts another
or a decision; an implementor cannot read a section into brief-violating
behaviour; diagrams agree with prose and carry the branches that matter; code
impact and the design-target selectors agree.

**Where to probe next if this pass is prolonged.**

- §5's new "only `declare` creates a store" rule against §7.2's input rules and
  §8.3's absent-store row — three places now assert the same behaviour.
- §4.3's uniform index staging against §11's crash-suites list, which must
  assert one write-set shape everywhere.
- §2's feature gate against §12's `Cargo.toml` row and §11.1's injection rule.
- Anything the §5 diagram change displaced: the pre-compute step now reads as
  declare-only, so the recompute step's rationale (step 6) should still hold for
  every command that reaches it.

**Out of scope.** Implementation planning, and the contents of IMP-001 / IMP-002.
