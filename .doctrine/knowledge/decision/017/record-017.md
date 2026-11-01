# Conventions: two sets, fixed-id seeds

`smut conventions` ships **two** sets in 0.2:

- `hygiene` — general structure and relation advice: give every entity `type`
  and `title`; give lifecycle entities a `status` and record the values in use;
  choose one direction per relation by the question you will ask; model
  containment explicitly with `part-of` before writing; keep the label
  vocabulary small, define each in one line, normalise when adding; state whose
  claim a claim is; render in bounded chunks, because render is a human channel.
- `epistemic-records` — the one domain with evidence (the first-use store is an
  instance): types, status values, and the edge vocabulary with its directions
  (`part-of`, `about`, `answers` answer-to-question, `supersedes` new-to-old,
  `refutes`, `cites` citing-to-source), plus the whose-claim rule.

No social-graph or dependency set in 0.2. Neither has evidence, and a set is
shipped only when its edge vocabulary and directions differ from hygiene's.

`--format json` emits **one entity** with a fixed id (`conventions-hygiene`,
`conventions-epistemic-records`), `type: "convention"`, `title`, `body` and
`version` (the smut release). Piping it to `declare` upserts that entity, so a
re-install updates the seed rather than duplicating it. After installation the
store owns its copy: the binary never re-installs, never reads it back, never
validates it and never selects a set heuristically. `smut conventions` with no
argument lists the sets and their one-line descriptions.

`learn` keeps its one pointer line and its separate epistemology: descriptive
and version-bound, where conventions are advisory and domain-bound.


## Operator caveat (sufficiency acceptance, 2026-11-01)

The operator wants to see and opine on the epistemic-records schema before it
ships. The set's types, status values, edge vocabulary and direction rules are
drafted in the design and shown in review; the set does not ship until that
schema is approved.
