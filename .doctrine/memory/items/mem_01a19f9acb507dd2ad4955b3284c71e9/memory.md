A decision is a **knowledge** record, not governance/spec truth, so no REV can carry it. `doctrine revision change add --target DEC-NNN` refuses outright:

    Error: `revises` target must be one of [SPEC, PRD, REQ, ADR, POL, STD], got a DEC

A reconciliation brief that says "DEC-NNN → REV modify" is therefore unpayable, and the mistake is cheap to make because a DEC reads like governance. The lawful route is two surfaces:

- **facet fields** (context/choice/alternatives/rationale/consequences, decided_by/on) → `doctrine knowledge edit decision DEC-NNN --choice/--rationale/...`. The prose body is *not* a facet.
- **the prose body** → direct edit of `.doctrine/knowledge/decision/NNN/record-NNN.md`, same as any authored `.md`.

**Facet list flags are comma-split, lossily.** `--alternatives` and `--consequences` split the value on *every* comma, so an item containing a comma becomes two items with no warning (`"…that flag's value, so examples…"` → two entries). Author each item comma-free, or the list quietly grows. Single-string facets (`--choice`, `--rationale`, `--context`) take commas happily.

REV rows that do land: `modify` on ADR/POL/STD/SPEC/REQ modifies *in place* and surfaces for manual landing; `status` rows auto-land. `create`/`introduce` need `--new-label` + `--member-of`, so they only make sense for requirement membership.

Observed on SL-010's reconcile (RV-034 F-6): three of six "governance" items were decisions and two were backlog items, so the pass used zero REVs and the created REV-003 was abandoned empty with the engine's refusal quoted in its rationale.
