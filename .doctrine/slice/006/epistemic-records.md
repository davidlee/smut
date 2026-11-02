---
summary: doctrine's epistemic record vocabulary, carried into a permissive store
version: 1.0
---

# Epistemic records

What you write down when you learn something, decide something, or are wrong about
something — using doctrine's own words for it. These are advice, not a schema: smut
prints them and never checks them.

The vocabulary is doctrine's, not this tool's invention. Same kind names, same status
words, same field names, same dates, so a record written here would be a doctrine
record if doctrine's machinery were present. Nothing is enforced, so every rule below
is a request you may ignore — and a store that ignores them still stores data.

General discipline lives in `hygiene`: one direction per relation, a status only where
there is a lifecycle, a vocabulary small enough to hold in your head. This set carries
only what is specific to records about what is true.

## The kinds

One table. Statuses in **bold** are terminal — the record keeps its body and its
references, and is never deleted. `‡` is a reference to a `party`; `→` is a reference
to another record.

| `type` | what it is | statuses | fields |
| --- | --- | --- | --- |
| `question` | something you want to know | `open` **`answered`** **`obsolete`** **`dropped`** | `why_matters`, `answer`, `answered_by`→, `answered_on` |
| `hypothesis` | a proposition under test | `proposed` **`confirmed`** **`refuted`** **`dropped`** | `proposition`, `predicts` |
| `assumption` | taken as true without proof | `held` `testing` **`validated`** **`invalidated`** **`obsolete`** **`dropped`** | `claim`, `basis`, `confidence`, `validation_plan`, `validated_by`‡, `validated_on`, `invalidated_by`‡, `invalidated_on` |
| `evidence` | a datum you got, and how | `captured` `disputed` `confirmed` **`retracted`** **`superseded`** | `datum`, `provenance`, `confidence`, `reliability`, `reported_by`‡ |
| `finding` | something learned, not yet one of the above | `open` **`settled`** **`dropped`** | `confidence`, `reported_by`‡ |
| `decision` | a choice, with what it was chosen over | `proposed` **`accepted`** **`rejected`** **`superseded`** | `context`, `choice`, `alternatives`→, `rationale`, `consequences`, `decided_by`‡, `decided_on` |
| `option` | a mutually exclusive candidate for a question or a need | `proposed` **`chosen`** **`rejected`** **`superseded`** **`dropped`** | none — the body argues it; `addresses` names the problem |
| `constraint` | something that binds what you may do | `active` **`waived`** **`superseded`** **`retired`** | `statement`, `source`, `applies_to`, `waiver_reason`, `waived_by`‡, `waived_on` |
| `concept` | an idea, nameable and reusable | `draft` `active` **`retired`** | none — the body is the whole record |
| `party` | the actor behind a report | `active` `discredited` **`retired`** | none — the body carries what you know about them |

`confidence` and `reliability` are `low`, `medium`, `high`. `basis` is `observation`,
`prior-art`, `design-inference`, `external-source`, `operator-judgement`. `provenance`
is `inspection`, `experiment`, `reproduction`, `citation`. A constraint's `source` is
`canon`, `adr`, `external`, `technical`, `legal`, `compatibility`, `operator` — and if
you need a value that list lacks, add it and record it in `vocabulary`.

Three status words do work you might not expect. `captured` is *you have it and have
not weighed it*. `confirmed` is *it survived a challenge and can be challenged again* —
evidence `confirmed` is deliberately not terminal. `disputed` is *challenged and not
settled*. `dropped` is the one doctrine has no word for: abandoned without a verdict,
which is not `refuted` and not `obsolete`.

Two discriminations the table cannot carry:

- **`finding` or `evidence`?** Evidence names one datum and how you got it. A finding is
a conclusion no single datum carries — a summary, an inference, or something someone
told you that you have not yet weighed.
- **What is this set's own?** The kinds `finding`, `option` and `party`; the field
  `reliability`; and `confidence` on a `finding`. Everything else is doctrine's,
  spelled as doctrine spells it, including the closed vocabularies — except
  `answered_by`, below, which keeps doctrine's name and changes what it points at.

## States

A kind's statuses are its own. There is no shared list, so `status: "retracted"` on a
`party` is not expressible — and that is the point: a status every kind can take tells
you nothing.

A record with no status has no lifecycle. A stated fact is not a process, and most
records in a real store carry none. Never write a status for completeness.

## The labels

A relation is a field whose value is a reference. Write one direction only and let
`backlinks` answer the other. The direction is a cost decision: write the one whose
question you will ask most.

| field | direction | asks |
| --- | --- | --- |
| `part-of` | child → container | what is inside this |
| `about` | record → subject | what do we have on this |
| `answered_by` | question → the record that answered it | is this answered, and by what |
| `addresses` | option → question or constraint | what was on the table |
| `supersedes` | new → old | what replaced this |
| `supports` / `disputes` | evidence → the record it bears on | what rests on this, and what argues against it |
| `cites` | record → a record it rests on | what does this assume |
| `reported_by` | record → party | whose word is this |

Three of those read as provenance, and they are different: a **witness**
(`reported_by`) is an actor whose standing changes, not your record's content; a
**premise** (`cites`) is a record whose fall takes your reasoning with it; a **subject**
(`supports`, `disputes`) is a record your evidence is *about* — it loses weight and
keeps its content. `reported_by` and `cites` are written by the record that owns the
dependency; `supports` and `disputes` are written by the evidence.

`part-of` is containment and `about` is subject: do not write both to the same target,
which is one fact written twice.

## Rules

1. **Kind first.** Ten kinds, and if none fits it is a `finding` — that is what
   `finding` is for. A finding that earns a kind is the same id re-declared with the new
   `type` and the new kind's entry status (`held`, `captured`, `proposed` …). Identity
   survives; nothing else changes type.
2. **A question names its answer.** `answered_by` is singular and points at the record
   that settled it. Doctrine's field of the same name holds a *person*; here it holds
   the answer, and the party behind that answer is a `reported_by` on the answer. Two
   live answers is not a state this set allows: the newer answers `supersede` the
   older, or it is not an answer to that question — it is a `finding` `about` it, or a
   `supports`/`disputes` on the existing answer.
3. **An actor is a party, never a name.** Any field naming a person, team, organisation
   or system — `decided_by`, `validated_by`, `invalidated_by`, `waived_by`,
   `reported_by` — holds a reference to a `party`. Keep the few parties you actually
   have and reuse them; an actor not worth a record does not go in the field, and the
   sentence goes in the body.
4. **`confidence` is yours; `reliability` is the report's.** How sure you are of the
   datum, and what weight the report carries given who it came from. They compose:
   *certain Paul said it, and you do not believe it* is `confidence: high`,
   `reliability: low`.
5. **One shape per field.** An actor field is a reference in every record or prose in
   every record, never both; the same for a date and an id. A field holding a string in
   one entity and an array in another changes what `$contains` *means* per row, silently,
   and cannot be sorted at all — `order` refuses it outright. Prose that cannot be a
   reference belongs in the body.
6. **An edge carries no data.** A reference is the identity and nothing else; an object
   carrying fields alongside an identity declares an entity instead of annotating a
   link. So "strong evidence" or "partial" lives in the body, never on the link.
7. **One direction per relation.** Never write both ends by hand — that is two facts to
   keep in step with nothing to check them.
8. **Dates mark decisions, not existence.** `decided_on`, `answered_on`, `validated_on`,
   `invalidated_on`, `waived_on`, and `review_by` on anything whose truth decays. All
   `YYYY-MM-DD`. `created` and `updated` are already known to git and worse here. There
   is no range predicate, so `review_by` can be ordered and its oldest taken, and the
   overdue set cannot be selected — write the date anyway, and read it yourself.
9. **One fact, one place.** If a reader will filter or sort on it, it is a field; if it
   is the argument, it is the body. `status` says *answered*; `answered_by` says *by
   what*. A status that repeats an edge is a fact you now have to keep true twice.

## The minimum record

Nothing here is checked, so the only pressure is a shape a reader can recognise as
complete. An `assumption` with no `claim`, `evidence` with no `datum` or `provenance`,
a `hypothesis` with no `proposition`, a `constraint` with no `statement`, a `question`
with no `why_matters`, a `decision` with no `choice` and no alternative named — each is
a note in the wrong kind, and worth re-typing before it is worth reading.

## Record your own vocabulary

A store will outgrow this list, and then the store is the reference. Keep one
`concept` under the id `vocabulary` whose body holds the kinds in use with the statuses
each takes, the labels in use with their direction, every value you invented beyond
this set with one line on what it means, and your deviations — "we do not use `party`
here" is more useful than silence.

That record is a hand-written census, not a schema, because nothing can compute the
distinct values in a store. It will go stale, and nothing will notice. Keep it true
when you add a value, and read it before you trust it.

The copy of a set that a store installs is already a stamp: the entity is
`conventions-epistemic-records` and it carries the `version` it was installed at. You
do not need a second place to record it.

Seeding this set

  `smut conventions epistemic-records --format json` prints this set as one entity
  declaration, and piping it into `smut declare` stores it under the fixed id
  `conventions-epistemic-records`, so running the pipe again updates that entity instead
  of adding another. Re-installing replaces every field the seed supplies — `body`
  included — and leaves the fields you added alone. The seeded body is therefore advice,
  not a record: keep an amendment by editing under a different id or by keeping it
  outside the seeded fields, or expect the next install to replace it.

  The seeded entity's own `type` is `convention`. That is the mechanism's label for a
  stored copy of a set, not one of the ten kinds above — leave it as it is, and do not
  read it as an epistemic record.

  The general rules live in `hygiene`, and the query recipes live in `smut learn`. This
  set does not repeat either.
