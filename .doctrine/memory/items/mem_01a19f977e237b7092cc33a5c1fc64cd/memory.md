A reconcile brief that routes a **decision** (DEC-NNN) to the REV surface is unroutable, and
the refusal arrives late — at `doctrine revision change add`, after the REV has been created
and transitioned.

Measured: the REV target set is `SPEC | PRD | REQ | ADR | POL | STD`. `revision change add`
takes `--target` as "a live peer FK", and a DEC is not in the set. SL-001's REV-001 was
abandoned for exactly this (its narrative says so), and SL-004's REV-002 records the correct
path: "The six decisions ... landed through `doctrine knowledge edit`; see the Reconcile
narrative below."

# The rule

- **Decisions, questions, assumptions, constraints → `doctrine knowledge edit`**, never a REV.
  Facet fields are per-field replacements (`--choice`, `--rationale`, `--consequences`,
  `--context`) — passing one replaces the whole field, so reproduce the field text and edit
  inside it rather than appending a fragment.
- **`--consequences` and `--alternatives` are comma-separated**: a comma inside one item
  silently splits it into two. Rephrase to avoid commas (em dashes work).
- **Prose bodies** land through `--body` (whole-body replace) or `--body-mode append`; the
  convention in this repo is a dated `## Amendment (<slice> reconcile, RV-NNN)` section naming
  the finding that drove it.
- A revise-intent with **no** REV-eligible target needs **no REV at all**. Check before
  creating one: an empty REV that is later abandoned is noise in the corpus.
- A brief is a proposal, not authority about which verb to use. When the brief names a surface
  the engine will refuse, correct the surface and record the correction in the Reconciliation
  Outcome; do not force the brief's wording through and do not escalate a mechanical mapping
  error to `/audit` or `/consult`.

# A trap while drafting

Building the new body with `perl -0pe ... > /tmp/body.md` when `perl` is absent leaves an
**empty file**, and `doctrine knowledge edit --body "$(cat /tmp/body.md)"` then wipes the
record's body. It happened to DEC-026 in SL-008 and was recovered with
`git show HEAD:<path> > <path>` (not `git checkout`, which the shared-index discipline
forbids). Check the staging file is non-empty before feeding it to `--body`.
