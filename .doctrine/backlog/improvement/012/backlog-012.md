# IMP-012: `smut learn` teaches what the query surface does and never what it cannot do

The guide is thorough about the nine keys, the predicate, traversal, the envelope
and the recipe. It never says where the surface *stops*, and the four stops are
the ones a real analysis walks into:

- **No comparison operators.** `$gt` refused by name, exit 2 — an honest refusal,
  but the first news. Measured while answering a clinical-trial question
  (`deviations after the live amendment`) and a water-network one (`highest
  supply junction`); `$gt` was the first guess in two of three domains.
- **No field-to-field comparison.** `visits_completed >= visits_expected` has no
  spelling. Client-side, after fetching both fields.
- **No absence-of-incoming-reference.** `$exists:false` is *field* absence, and
  the nearest-looking test answers yes to the wrong question: asking which
  participants have no adverse event with `{"reported":{"$exists":false}}`
  returns all eleven, where the truth is six. Compare the field-absence question
  — which modules have no governing document — which works perfectly. Two
  similar-looking questions, one correct answer and one silently wrong one, and
  nothing in the guide tells them apart.
- **Chains cost a call per edge type.** `follow` takes several field names but
  all outgoing; `backlinks` likewise. Three hops across three relation types is
  three calls, and each hop's projection returns references, so a neighbouring
  entity's *label* costs a `get`. The two-call recipe is taught as direction
  discipline; the binding constraint is edge type.

## The two sentences this item now owes the decision

DEC-027 settled the refusal question, and it obliges the guide to say two things it
would otherwise be free to leave out:

- **Scope the refusal.** Smut refuses *some* provably-false conditions within one
  field's conditions and nothing across `$and`. Without that sentence, absence of a
  refusal reads as assurance and the guide has created a new wrong belief.
- **A field no entity carries fails every operator**, exactly as a missing field on
  one entity does. This is the trap that made "which participants have no adverse
  event" answer *all eleven*; it belongs beside the missing-versus-null paragraph,
  not in a section of its own.

## What a cure has to be

One paragraph and a short list, in the shape the guide already uses — dense,
declarative, no tutorial. Two idioms carry part of the weight and belong beside
the predicates rather than scattered:

- **`order` + `limit:1` is the comparison.** A max or min is two clean queries
  (`{"where":{…},"order":[{"field":"elevation_m","direction":"desc"}],"limit":1}`),
  and it is documented under `order`, far from `where` where the need arises.
- **Set difference is the caller's.** Absence of an edge and any range test are
  answered by fetching both sides and subtracting; the guide should say so in one
  sentence rather than leave it to be discovered.

Constraints:

- **Do not imply a refusal exists where it cannot.** Absence-of-edge is not
  detectable from a predicate: the predicate is true as written. Saying "smut
  will tell you" would be worse than the silence it replaces.
- **Present tense, no promises either way.** Whether the stops are permanent or a
  roadmap is undecided and deliberately so (operator, 2026-11-02: "future roadmap
  is informed by usage reports"). Write what the surface cannot do *today*, with
  no "never" and no "coming soon" — the paragraph then survives a widening
  without contradicting itself, and only the tense changes if one arrives.
- **Keep it short.** The guide is 13.5 KB already, and its length is the reason
  agents stop re-reading it (IMP-008). This item is content, IMP-008 is doors; the
  paragraph must survive a section split without becoming its own section.
- **Do not add operators in this item.** Range tests, cross-field comparison and
  edge-absence predicates are each a design decision (see IMP-010 for the
  diagnostic question they share); the guide stating the boundary is not the same
  work as moving it.

## Neighbours

IMP-011 (the discriminator and the substring trap) is the other half of "what
`where` cannot express". IMP-008 gives this paragraph a door. IMP-010 owns the
diagnostic channel that would soften two of the four cases.

## Provenance

Usability exercise RFC-001: 18 questions across three independently authored
stores; every claim above was measured on the built binary, 0.2.0. The tense rule
records the operator's answer to RFC-001 § Outcome Q3.

## Measured addition (2026-11-02): `$contains` changes meaning with the value's shape

Found while designing a conventions set (`epistemic-records`, SL-006), on a probe
store, binary 0.2.0. Four records, one field holding a string in one entity and an
array in two others:

| request | result |
| --- | --- |
| `{"decided_by": "trev from the disco"}` | the string row only — an array row is not equal to a string |
| `{"decided_by": {"$in": ["trev from the disco"]}}` | the string row only |
| `{"decided_by": {"$contains": "trev"}}` | the **string** row (substring) and the array row whose element is exactly `"trev"` — but not the array row whose element is `"trev from the disco"` |
| `{"decided_by": {"$contains": {"id": "pty-cfo"}}}` | both array rows; the object element is matched by whole-value equality |
| `{"decided_by": {"$eq": {"id": "pty-cfo"}}}` | nothing — the field is an array, not that object |
| `{"order":[{"field":"decided_by",…}]}` | refused: `an array has no ordering, and a sort value must be null, a boolean, a number or a string` |

So `$contains` is a substring test on a string and an exact-element test on an
array, with nothing in the response saying which one ran, and one field cannot be
sorted at all if any row holds an array. This is a fifth case for the "what the
surface cannot do" paragraph, and a sharper one than the other four because the
operator *works* — it silently answers a different question per row rather than
failing or over-answering.

It also settles a conventions question from the other side: a store that writes one
field in two shapes pays in query shape, and the payment is invisible.
