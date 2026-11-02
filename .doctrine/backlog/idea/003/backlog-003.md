# IDE-003: A store has no census: an agent surveys an unfamiliar store by scanning every entity

An agent's first act in a store it did not write is a survey — what classes
exist, which fields appear, which relations are used, how many of each. `smut`
has no verb for it. The only route is `query {}` with a `select`, then reading
keys off the objects.

Measured cost is small and scaling: three stores of 34 entities took one query
each (the default `limit` is 100), so the survey was cheap and eyeballed. At five
hundred entities it is five pages of output to answer "what is in here".

The census is also what turns a missing discriminator from a nuisance into a
blocker: with `type` populated (IMP-011), `{"where":{"type":"pipe"}}` *is* the
census for the one question that matters most.

## Open questions

- Is a census (a) a new verb, (b) a specification extension (`{"count": …}`,
  `{"fields": …}`), or (c) nothing: an idiom the guide teaches, keeping the
  surface at nine keys?
- If (a), what does it report — id classes by prefix, fields with counts,
  relation labels used, entity total? Each is a different answer to "what is in
  here", and id-prefix classing is exactly the unsound inference IMP-011
  complains about.
- Does a count belong at all, given the envelope already answers "how many" for
  a selection the caller wrote?

## Status

Idea, unscheduled. Nothing in the exercise was blocked by the lack of a census;
it is a cost, not a failure, and the ranking in RFC-001 places it last.

## Provenance

Usability exercise RFC-001: three stores, 34 entities each, surveyed by prefix and
field inspection.
