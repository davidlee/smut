# IDE-004: Smut has no edge attributes: a reference is an id and nothing else

Raised 2026-11-02 while designing the epistemic-records conventions set (SL-006).
It is an idea, not a request: nothing below is a proposal to build.

## The fact

In smut a relation is a field whose value is `{"id": ...}` or an array of them.
There is no edge table, no edge identity, and no way to say anything *about* a
relation. Worse than absent, it is actively disambiguated away: `learn` states that
"an object with an id plus other fields is declared, and its occurrence becomes a
plain reference" — so `{"id": "ev-1", "reliability": "low"}` does not annotate an
edge, it re-declares entity `ev-1`. There is no spelling of an annotated edge.

## Is it a problem?

The pressure showed up twice while designing a 40-line conventions set, and both
times it had a home without edge attributes:

| what wanted to live on an edge | where it went instead |
| --- | --- |
| `reliability` of a report ("Paul's on record but unreliable") | a field on the evidence record, because a report's standing is a property of the report |
| the role of a `references` edge (doctrine's `implements` / `originates_from` / `concerns`) | promoted into the label name, or stated in the body |

Doctrine has had attributes available for years and uses them barely: a `[[relation]]`
row is `label` + `target`, a role is folded into the label, and free-text qualification
lives in `--descriptor` on exactly one edge kind. That is weak evidence that the
pressure is real but low.

The honest counter-argument: the reason nobody has missed edge attributes may be that
everyone writing stores under smut is writing small ones, where an attribute is a
sentence in a body. A graph database without edge attributes is unusual, and "we
copied a permissive document store and drew arrows in it" may be the actual design.

## Is it fixable?

Mechanically there are three shapes, and they differ in what they cost:

1. **A reserved escape on the reference object.** Something the parser already
   reserves — `$` is reserved inside a *condition*, not inside an entity — so a new
   rule such as "a reference carrying a reserved key is an annotated edge" would need
   the same disambiguation the current rule already makes, with one exception added.
   Cheapest to write; every traversal, `select`, `backlinks` scan and render path
   then has to decide what to do with the extra keys.
2. **A separate edge collection.** Entities and edges both addressable, edges with
   their own ids, amendable and deletable. This is the real graph-database answer and
   the largest change: `backlinks` currently scans fields, `delete` currently leaves
   dangling references by design, and an edge with identity needs its own lifecycle
   and its own dangling-reference contract.
3. **A label whose name carries the attribute.** Not a fix; it is what the conventions
   set already does, and it is worth naming here so the idea does not get built twice.

## What would have to be decided first

- Does an edge have identity (can it be amended, deleted, cited)?
- Are edge attributes queryable (`where` on an attribute, `order` by one)?
- Does an attribute change traversal cost, or is `backlinks` still a field scan?
- Anything that changes the wire format lands with a version bump and the same
  recovery contract STD-002 names.

## Related

- SL-006 (the conventions set whose grammar forced the question)
- ADR-001 (identity is `smut_id`, not `id`) — the nearest precedent for a format break
- IDE-005 (the actors set, where "reliability on the report" was decided)
