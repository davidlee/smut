# Review-stage mechanics: what `mem.doctrine.design-run.review-stage` adds

The base mechanics live in `mem_01a19a7a41217a40bc841c6b9ff2489c` (Design-run reviewing
mechanics — routes, reopen-to-route, `att-` declarations, conclude-again, stale being
advisory). Read that first. SL-008 added four things it does not carry.

1. **The run's findings and the RV ledger are two records.** A finding is raised twice if
   the review must survive in the authored corpus: on the run as a `fnd-N` node
   (`{concerns: "sec-N", summary, blocking}`) and on the ledger with `doctrine review
   raise`. The run's nodes are runtime state (gitignored); the RV is what the audit reads.
2. **`blocking` is inert after creation** on a run finding: an apply that tries to clear it
   is refused ("the run already holds it"). A finding stops being open by gaining a
   `resolution`, not by clearing the flag — so raise it with the right `blocking` value the
   first time.
3. **A section body is `design.md`.** `doctrine design materialise` rewrites the whole
   document from the section nodes, so prose edited *in design.md* is lost and prose edited
   in the node is the truth. Editing one section invalidates only that section's `att-`
   nodes; re-declare just those, not all seven.
4. **Attestation is not coverage.** A pass opened before late section edits keeps reporting
   `STALE — it no longer covers current content` even after the edited sections are
   re-attested and the pass is re-concluded: re-attesting restores the section's review
   state, not the pass's coverage of the text. Either re-run a pass over the edited
   sections, or record the residue for audit — do not read a clean `sections_outstanding_
   review=0` as "the edited text was reviewed".
