# REV REV-002 — reconcile SL-004

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-004 moved the store's identity key from `id` to `smut_id` and the store format
from version 1 to 2. STD-001 is the standard that tells a reader where the format
record's rules live — and its own trigger table still states one of them with the
old spelling:

```
| the identity key and canonical bytes (entity-first ordering, `id`-first within an entity) | … |
```

The shipped serialiser puts `smut_id` first (`src/entity.rs:201,217,311`;
`docs/store-format.md` § Canonical bytes). The standard's argument is unaffected:
its trigger fired correctly on this slice, which is why the row was read at all.
One parenthetical moves; the statement's other rows, the rationale, the scope and
the worked-diagnostics rule are untouched.

**Scope of this revision: one row.** The other reconcile items are not REV
subjects:

- The six decision records (DEC-002, 005, 007, 010, 012, 016) are knowledge
  records, and the REV target set is `SPEC | PRD | REQ | ADR | POL | STD` — the
  engine refuses a DEC target (the precedent is REV-001, abandoned for that
  reason). They landed through `doctrine knowledge edit`; see the Reconcile
  narrative below.
- `design.md` §3/§4, `docs/store-format.md`, `smut-implementation-brief.org` and
  the selector registry are per-slice artefacts and project documentation, edited
  directly.

## Before / after

```diff
-| the identity key and canonical bytes (entity-first ordering, `id`-first within an entity) | the identity key's spelling changes, an ordering rule changes, or the serialiser's shape changes |
+| the identity key and canonical bytes (entity-first ordering, `smut_id`-first within an entity) | the identity key's spelling changes, an ordering rule changes, or the serialiser's shape changes |
```

## Reconcile narrative

- **[RV-028 F-4]** STD-001's canonical-bytes parenthetical said `id`-first. Amended
  to `smut_id`-first (this REV). Nothing else in the standard changed: the
  trigger column already says "the identity key's spelling changes", which is
  exactly what happened.
- **[RV-028 F-2]** DEC-007's title, first rejected-alternative bullet and slug.
  Title and bullet amended; the slug (`…-entity-serialisation-id-first`) is
  tolerated — `doctrine knowledge edit` exposes no slug field — and the record's
  amendment note says so, so a later reader does not hunt a fix that cannot exist.
- **[RV-028 F-3]** DEC-010's `choice` was replaced as the full field text, keeping
  the leading README clause the amendment proposal's quotation dropped.
- **[RV-028 F-8]** `smut-implementation-brief.org` now says "identity/content
  disagreement", matching the format record.
- **The six decisions** (`doctrine knowledge edit`, both tiers, one act each):
  DEC-002 (coalescing example, created/identity-only consequence), DEC-005 (the
  withdrawal: `id` is ordinary data, `smut_id` is the reservation; facet
  `context`/`choice`/`consequences`; `reference_id` return type corrected to
  `Reject`), DEC-007 (facet `choice`, title, envelope, canonical ordering,
  serialisation clarification, alternatives), DEC-010 (full `choice`, `consequences`,
  README/store-format/learn passages), DEC-012 (one envelope example), DEC-016
  (facet `choice`, the recipe's `select: ["smut_id"]`). Each carries the dated
  SL-004/ADR-001/DEC-025 note; no durable id was renumbered and no unrelated
  rationale moved.
