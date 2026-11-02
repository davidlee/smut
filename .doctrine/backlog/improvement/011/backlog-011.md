# IMP-011: The `type` discriminator is advice an author never sees, and the fallback — filtering ids by substring — is unsound

Three independently briefed authors wrote three 34-entity stores for a usability
exercise (an engineering org, a clinical trial, a water network). Each was given
the declaration shape and nothing else — no guide, no conventions, no source.
Measured afterwards:

| store | discriminator present | usable as a class filter? |
| --- | --- | --- |
| engineering org | `kind` on all 34 entities | yes, but not the recommended spelling |
| clinical trial | none at all | no — every entity is indistinguishable by field |
| water network | `kind` on 7 of 34, holding *physical* node class (`junction`, `gate`, `reservoir`) | no — `kind:"pipe"` matches nothing, silently |

`hygiene` item 1 says exactly the right thing: "Give every entity `type` and
`title`. The discriminator is what makes `where` usable later." The advice is
correct and **0 of 3 authors followed it**, because it is reachable only by
running `smut conventions` and choosing to read it. `declare` never mentions it;
`learn` points at conventions once, in passing. Advice that must be sought at a
moment nobody is looking is advice that is not applied.

## The consequence: an unsound fallback

With no discriminator, the only handle on a class is the id, and the only string
operator is `$contains` — a substring test, not a prefix test. Measured on the
water store:

```
❯ smut query '{"where":{"id":{"$contains":"p-"},"in_zone":{"$exists":false}},"select":["id"]}'
["p-hillcrest-oak","p-oak-millbrook","pump-eastgate-lift","pump-hillcrest-boost"]
```

Pipes are `p-*`; pumps are `pump-*`, which contains `p-`. Four objects where the
question asked for two, exit 0, no warning, and the answer reads as complete.
This was the one wrong answer the exercise produced, and the tool supplied the
workaround that made it.

## What a cure has to be

- **Delivery, not text.** The sentence is already right. The question is whether
  an author meets it *before* writing: a `declare` reply that says it once for a
  store whose entities carry no discriminator, or a `learn` §Declaring clause
  beside the reference convention — the paragraph an author does read. Either is
  a behaviour change and needs the pointer pinned like any other surface.
- **`type` and `title` keep their names.** Two of three authors reached for
  `kind`, which is the vocabulary divergence `hygiene` item 5 warns about,
  observed rather than hypothesised. If `kind` is the word authors reach for,
  the recommendation is either wrong or unheard; decide which before rewriting it.
- **One slot, two axes.** The water store needed a class (`pipe`, `valve`) and a
  physical kind (`junction`, `reservoir`) at once, overloaded `kind`, and thereby
  broke both. The convention says nothing about a second classification axis. One
  line — `type` is the class; anything else is a field with its own name — would
  have prevented it.
- **Not a `$prefix` operator.** Substring filtering is a workaround for the
  missing discriminator. Adding a prefix test makes the workaround more
  comfortable and leaves the cause; if ids are the only handle, removing the need
  for the handle beats ordering it.

## Neighbours

IMP-012 (`learn` states what the query surface cannot do) is where the
`$contains` trap gets its sentence if it is taught rather than removed. IMP-010
(a contradictory condition answers silently) owns the sibling diagnostic — a
filter naming a field no entity carries — if a diagnostic channel is ever added.
IMP-004 owns *content decisions for a second conventions set*; this item amends
the shipped one.

## Provenance

Usability exercise RFC-001 § evidence: three stores, 34 entities each,
independently authored, 18 questions. Not a defect report — every behaviour above
is design-conformant.
