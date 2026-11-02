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

## What was decided, and how it changes the cure

Operator direction, 2026-11-02:

- **`type` is the word.** The recommendation stands; the failure was delivery.
  (Two of three authors wrote `kind` — that is evidence the advice is unheard, not
  that it is wrong.)
- **Delivery runs through the `learn` surface**, three levers: a strong"follow on
  with conventions" recommendation in the guide's existing `Conventions` section
  at the foot; a flag so an `AGENTS.md` can instruct it once — `smut learn
  --with-conventions hygiene`; and section retrieval (`smut learn labels`, etc.).
- **Conventions are not injected into responses as a rule.** The one edge case
  worth keeping open: a stderr warning when a *declaration* carries `kind` but no
  `type`.

Those levers are specified in IMP-008, which therefore holds two jobs now —
delivery (this item's cure) and legibility (its own complaint) — and the delivery
job outranks the legibility one.

## What a cure has to be

- **Delivery, not text.** The sentence is already right; `learn` already carries a
  `Conventions` section (`src/learn.rs:218`) and a pin asserting it names the
  command and what it is not (`the_guide_points_at_smut_conventions_once`). What
  changes is strength and reach: a recommendation rather than a description, plus
  a flag and section names an `AGENTS.md` can encode. See IMP-008 for the shape
  and for the pin consequences of changing the guide's default print.
- **`type` and `title` keep their names** — decided above. The vocabulary
  divergence stays a warning (`hygiene` item 5), not a second spelling.
- **One slot, two axes.** The water store needed a class (`pipe`, `valve`) and a
  physical kind (`junction`, `reservoir`) at once, overloaded `kind`, and thereby
  broke both. The convention says nothing about a second classification axis. One
  line — `type` is the class; anything else is a field with its own name — would
  have prevented it.
- **The stderr edge case was illustrative** (operator, 2026-11-02), and is
  recorded here only because the constraint outlives the idea. A success-path
  warning would break a widespread pin: `stderr.is_empty()` is asserted on
  success in nine places, including the shared `assert_one_json_line` harness
  (`tests/compatibility.rs:85`) that most contract tests route through. The
  warning itself is cheap and sound — the declaration is in hand at `declare`
  time, so "`kind` present, `type` absent" needs no scan — but whether a
  successful command may ever write to stderr is a contract question, and the
  answer belongs with IMP-012's boundary work rather than smuggled in here.
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
is design-conformant. Operator decisions of 2026-11-02 recorded in RFC-001
§ Outcome.
