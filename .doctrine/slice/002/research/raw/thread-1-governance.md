# Raw thread 1 — governance applicability (0.2)

Producer: orchestrating agent, degraded mode (no research agents defined in
`AGENTS.md` / project orientation). Read-only.

Commands used: `doctrine show IMP-001`, `doctrine show DEC-011`, reads of
`smut-implementation-brief.org`, `slice-002.md`, SL-001 `design.md`,
`interview.local/.smut` via the built 0.1 binary.

- IMP-001 charter: full `where` operator set, `follow`/`backlinks`/`depth`/
  `budget`, determinism, `dangling`, `traversal_budget`, grammar widening.
  Trigger-met section names the first-use store and the four decisions to settle
  (`prop-result-provenance`, roots-only `where`, conventions shape, work order).
  "Do not build": validating schema, modelled containment, in-store audit,
  doctrine promotion.
- Brief §§103–163 (Query contract, Keys, Predicates, Traversal, JSON envelope)
  are the observable authority; §184 (guide) and §204 (verification) bind the
  guide and tests. Brief wins over any other document.
- SL-002 scope records 11 objectives and six open design questions.
- DEC-011: 0.1's four keys, future keys refused by name, compatibility tested.
  It shapes 0.2 because the refusal-by-name tests invert here, while the 0.1
  query pins must not move.
- SL-001 design §§4.1–4.7, 5, 6.1–6.3, 8.1, 9: consumes commit/recovery,
  comparator, truncation and envelope rules; DEC-001…DEC-011 are the decisions
  to extend, not re-open.
- `criterion-scope-smut`: no validating schema, no modelled containment, no
  in-store audit, no doctrine coupling.
- No project specs/ADRs/policies/standards exist (SL-001 research Thread 1, still
  true: `doctrine relation census` shows only `shapes`/`reviews`).

Not applicable: Doctrine framework rules bind this repo's process, not smut's
observable JSON; the brief forbids importing that machinery. The project
orientation doc is still install scaffold. Backlog carries IMP-001/002/003 only;
IMP-002 and IMP-003 are separate slices.

Revision candidates: none. The design-run DECs are the write surface for 0.2's
new choices.
