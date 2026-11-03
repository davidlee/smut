# REV REV-003 — reconcile SL-010

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Created during SL-010's reconcile pass (RV-034) on the assumption that the four amended
decision records were `revises` targets. The engine refuses that outright:

```
Error: `revises` target must be one of [SPEC, PRD, REQ, ADR, POL, STD], got a DEC
```

A decision is a knowledge record, not governance/spec truth, so no REV can carry it. That
is the reason this revision is abandoned empty rather than misfiled.

Nothing else in the brief needed a revision either:

- **No ADR.** ADR-001 is the identity record (`smut_id`), and SL-010 changed nothing about
  identity — it reused the field the ADR already fixed.
- **No policy, no standard.** STD-001 and STD-002 are untouched: nothing landed inside
  `.smut/`, no stored-validity rule moved, and no format version was implied.
- **No spec, no requirement.** SL-010 declares no spec targets (`plan.toml` `[specs]` is
  empty) and mints none.

The four decisions were amended on their own surface instead — `doctrine knowledge edit
decision DEC-NNN` for the facet fields, and a direct edit of each `record-NNN.md` for the
prose, which is where DEC-016's recipe, DEC-035's narrowing, DEC-037's superseded
consequence and DEC-039's corrected illustration live. IMP-014 and IMP-016 are work
records, annotated in their own bodies.

Abandoned 2026-11-03 by SL-010's reconcile pass. Finding: RV-034 F-6.
