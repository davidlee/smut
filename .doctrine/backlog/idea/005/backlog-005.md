# IDE-005: A companion conventions set for actors — people, roles, organisations, contact points

Raised 2026-11-02 while designing the epistemic-records conventions set (SL-006),
where `party` entered as the target of `reported_by`.

## The pressure

The epistemic set needs exactly one question answered about a person: *how much
weight does this report carry, and who stands behind it*. It answers that with a
`party` kind whose whole definition is a title, a body, and a status
(`active|discredited|retired`), plus a `reliability` field on the report itself.

That is a deliberately thin actor. A store that has people in it at all will
immediately want more: which organisation a party belongs to, what role they hold and
when it changed, several ways to reach them, and the fact that a person's standing is
institutional ("the CFO is reliable on cost, evasive on dates") as well as personal.

## Why it is a separate set and not more fields here

The epistemic set's job is what a *record* is. An actor schema's job is what a *world*
is: identities, roles over time, affiliations, contact points. Merging them would
make every epistemic store carry a directory it does not need, which is the same
failure as shipping twelve domain sets in 0.2 (DEC-017).

The migration path is clean and worth stating now: if this set exists later, `party`
**moves to it**, and the epistemic set keeps only the edge — `reported_by` → a party,
with `reliability` staying where it is, on the report. The epistemic set would then
say "the actor is defined by the actors set; this set only points at it". That is a
§F-shaped outcome (IMP-004) at a smaller scale: a set that is a shipped example of a
domain another set owns.

## What it would have to cover

- identity: how a person is named and disambiguated (ids are permanent and opaque in
  smut, so a rename is an issue — see IDE-002)
- affiliation and role, including roles that end (`part-of` with dates? a status?)
- contact points as their own records, so an address is not copied into three bodies
- reliability and standing, if `party`'s notion is to be shared rather than re-invented
- the difference between an actor inside the store's organisation and one outside it —
  which is the distinction the epistemic set's `reported_by` actually cares about

## Not yet decided

Nothing. This is a captured idea, at the same weight as IMP-004's opening question:
before designing it, ask what it is *for* — a directory, a stakeholder map, or just
enough actor to make evidence attributable.

## Related

- SL-006 (where `party` and `reported_by` were born)
- IDE-004 (edge attributes — the alternative home for "reliability of this claim")
- IDE-002 (renaming with referential stability: a directory makes this sharper)
- IMP-004 / DEC-017 (a set is a file plus a match arm)
