# REV REV-001 — reconcile SL-001

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Opened to carry RV-004's one governance item: the clarification of DEC-007's
mechanism clause after finding F-1 (the shared wrapper was applied at the stored
file root only, so the same entity serialised two ways depending on where it
appeared; fixed in `78a7eb1`).

**Abandoned without a change row.** The revision engine refuses the target:
`doctrine revision change add REV-001 --action modify --target DEC-007` →
*"`revises` target must be one of [SPEC, PRD, REQ, ADR, POL, STD], got a DEC"*.
A decision is a knowledge record, not a governed artefact in the REV target set —
and the engine's refusal is not something an instruction can override. The
sanctioned surface for a decision is `doctrine knowledge edit` (facet fields and
prose body), so the clarification lands there instead, and this REV is left as the
record of why the ordinary route did not apply.

No other governance target is affected: no ADR, policy, standard, spec or
requirement states the wrapper rule. The slice-level statement of it is design
§4.2, which is a per-slice direct edit.

## Reconcile narrative

- **[RV-004 F-1]** DEC-007's mechanism clause said one wrapper was used "for
  stored entity files and for projected entities alike" while the build applied it
  at the file root only. The clarification — the wrapper's subject is the entity
  node, not the value's position in a document — was landed in
  `.doctrine/knowledge/decision/007/` by `doctrine knowledge edit` (facet
  `choice` plus an appended body section), because the REV target set does not
  admit a DEC. Design §4.2 carries the same rule for the slice.
