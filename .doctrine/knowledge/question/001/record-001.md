# Does smut ever gain user-definable typed schemas?

An open question, recorded rather than resolved. The operator's position is
deliberate: "I don't have a strong position on the difference between 'not at all'
and 'not yet'." The decision taken around it (DEC-029) is to make the naming robust
to either answer instead of predicting one.

## What turns on it

- **Vocabulary.** Conventions are named for *not* being enforced. If schemas arrive,
  that word must keep marking the difference; if they never do, it is simply the
  right word. Today's naming hedges, and the hedge stops being visible the moment
  nobody remembers what it was for.
- **The loader.** DEC-028 specifies it as generic over declaration kind, which costs
  nothing and would let a schema artefact reuse the ladder. Whether that genericity
  is a hedge or a scaffold is this question.
- **The posture.** DEC-017 settled "never validated, never auto-selected, never read
  back". That is a product stance, not an oversight, and a schema artefact would
  amend it rather than extend it.

## Pressure, as measured

Not speculation — the usability work supplies the shape of the demand. The
top-ranked finding across three independently authored stores was that **0 of 3**
were queryable for the same reason: no discriminator, because the advice that
requires one was never read. The cheapest operator reaction to that failure is to
declare what entities look like and have the tool insist. Conventions are the
schema-shaped answer that does not insist; if advice proves insufficient at scale,
enforcement demand follows.

## What would answer it

- Users authoring conventions sets that are, in substance, schemas — field lists,
  required keys, reference constraints — expressed as prose because nothing else
  exists.
- Agents continuing to diverge in vocabulary despite shipped and project sets.
- A request for validation from a caller who cannot enforce anything on their side.
- Equally: a year of real stores where conventions *are* enough. That answers it too,
  and is the answer worth hoping for, since it costs no enforcement machinery.

## Boundaries already fixed

- Conventions never acquire teeth: the moment a set can refuse a declaration, the
  store's oldest sentence — "whatever is stored there is data" — becomes false.
- A schema artefact, if it arrives, is a *second* kind with its own verb, its own
  precedence and the validation, loaded through the same ladder. It is not a
  promotion of conventions.
- Nothing is foreclosed now: no naming, no format and no behaviour decision depends
  on the answer.

## Revisit

Reopen when entity validation is first proposed, when user sets show the prose form
straining (field lists repeated across files), or after a year of use with enough
real stores to see whether advice was adopted. Until then this is a question, not a
backlog item.
