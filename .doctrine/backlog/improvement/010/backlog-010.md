# IMP-010: A provably contradictory field condition answers with the empty set and says nothing

Trial friction 1, and the one the reporter was most careful about:

> `{"status":{"$eq":"open","$ne":"open"}}` is a legal ANDed predicate, returns
> `[]`, says nothing. Every other malformed input gets a named error; this one is
> a well-formed question with a known-false answer. It's not wrong — it's
> *unhelpful* at exactly the point where an agent is most likely to have made a
> typo.

Measured on 0.2.0:

```
❯ smut query '{"where":{"status":{"$eq":"open","$ne":"open"}}}'
{"dangling":[],"missing":[],"objects":[],"truncated":false,"truncation_reasons":[]}
(exit 0)
```

It is not a bug. A `query` returns a set, and this set is empty.

## Why no refusal can be general

- Contradiction is undecidable at parse time in the general case: `$in` sets can
  overlap, `$contains` compounds, and `$not`/`$and` compose. No refusal can mean
  "this can never match".
- The empty result of `{"where":{"nonesuch":"x"}}` is legitimate and, after the
  fact, indistinguishable from the contradictory one. Only a syntactic check can
  tell them apart, which makes this a parse-time question — not a "say when
  nothing matched" feature, which would need a store scan and would be noise on
  every honest empty answer.

## The subset that is decidable, and sound

One field's condition list is enough for the always-false forms:

- `$eq v` with `$ne v`, for any `v` compared structurally;
- `$eq v` with `$in [...]` not holding `v` structurally;
- `$in []`.

Each is false for every store, so any diagnostic built on it cannot produce a
false positive. `$in []` is the sharpest: it is a syntax-level always-false, and
smut already teaches `$or: []` as documented always-false — the vocabulary for
"this cannot hold" exists.

## The tension worth deciding, not dodging

`Reject::MixedPredicateObject` refuses a value mixing literals and operators
*because the design gives it no behaviour* (`parse_field`'s doc comment,
`src/predicate.rs`). A provably false conjunction **has** behaviour — false. So
refusing it on the "no behaviour" rationale would be inconsistent, and the
consistency is the interesting part:

- **(a) Accept, say nothing.** Today. Consistent, unhelpful.
- **(b) Accept, diagnose where a caller can see it.** Needs a channel the
  envelope does not have — the conversation IMP-009 owns, which is why this item
  does not invent one.
- **(c) Refuse the always-false forms, with a second rationale** — "a predicate
  that cannot hold is a typo, not a question" — accepted as a judgement rather
  than derived from the design, and stated in `smut learn` §Querying beside the
  missing-versus-null rule, which is the same kind of sentence.

Note that (c)'s line has to be drawn honestly: structural equality, arrays, and
`$contains` all compound. A partial check that refuses only what it can prove is
defensible; one that refuses on a heuristic is not.

## Not in this item

- Any envelope or diagnostic-channel work — IMP-009.
- A general "your query matched nothing" note; see above, it cannot distinguish
  the case that matters.
- Predicate error *messages* — ISS-004 owns the refusal text on the same surface.

## Ready to pick up when

A decision on response-surface diagnostics exists (IMP-009's conversation), or
only the `learn` sentence is wanted, in which case (c) is the cheap answer and the
item is small. Status `design-deferred`.

## Provenance

0.2.0 rc trial, friction 1. The reporter's framing — "not wrong, unhelpful" — is
the premise; the sound subset, the inconsistency argument and the options are
this repo's reading.
