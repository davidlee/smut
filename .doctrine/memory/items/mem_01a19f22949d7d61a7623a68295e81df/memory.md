# The situation

SL-008 and SL-010 landed in the same files (`src/error.rs`, `src/learn.rs`,
`tests/cli_contract.rs`, `src/conventions.rs`) inside the same window. A commit **range** for
one slice's phase therefore contains the other slice's commits. Handing a reviewer
`git diff <start>..<end>` presents foreign code as review material: the reviewer re-raises it,
the phase's finding list inflates, and the reviewer may try to "fix" another slice's in-flight
work.

Narrowing the registry range does not solve this. The range that makes `verify-vt` and
`conformance` honest can still interleave, because the interleave is in the *files*.

# The technique

1. List the phase's own commits: `git log --oneline --no-merges <start>..<end>`.
2. Build the review context as **one patch per commit, path-filtered to the phase's owned
   files** — concatenated, in order:

       git show <oid> -- <owned paths>

3. Declare the working tree's other-slice code **out of scope** in the review brief and name
   the slices.
4. In the ledger, record the interleave as *foreign-but-attributable*, never as noise: both
   slices' audits must be able to see it, and neither may silently drop it.

Used on SL-008 PHASE-04's review (RV-033), whose recorded range spanned 26 commits for two
slices. The reviewer saw only the phase's patches and could still report a finding against
unreviewed inherited code.

# Honest limit

This reviews what the slice *wrote*, not what the tree *does* once both slices have landed.
Integration between the two is the audit's problem and belongs in the audit brief, not in a
per-phase review. Sibling: `mem.pattern.doctrine.co-tenant-gate-verification` covers the gate
side of the same co-tenant problem.
