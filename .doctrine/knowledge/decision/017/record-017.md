# Conventions: one set, fixed-id seeds

`smut conventions` ships **one** set in 0.2:

- `hygiene` — general structure and relation advice: give every entity `type`
  and `title`; give lifecycle entities a `status` and record the values in use;
  choose one direction per relation by the question you will ask; model
  containment explicitly with `part-of` before writing; keep the label
  vocabulary small, define each in one line, normalise when adding; state whose
  claim a claim is; distinguish a withdrawn record from a replaced one; render in
  bounded chunks, because render is a human channel.

`epistemic-records` was drafted as the second set and **deferred** to `IMP-004`
on 2026-11-01. It was the only domain with evidence, but every review finding
about it concerned its content — which statuses exist, whether a citation needs
an entity, whose a claim is — and those are product decisions for one narrow
domain, not the mechanism this release had to settle. 0.2 settles the mechanism
and ships one set; the draft, the evidence against it and the questions it must
answer are preserved in `IMP-004`, which needs its own design conversation.

No social-graph or dependency set in 0.2 either: neither has evidence, and a set
ships only when its edge vocabulary and directions differ from hygiene's.

`--format json` emits **one entity** with a fixed id (`conventions-hygiene`,
and the same pattern for any later set), `type: "convention"`, `title`, `body`
and `version` (the smut release). Piping it to `declare` upserts that entity, so
a re-install updates the seed rather than duplicating it. It also replaces every
field the seed supplies, `body` included: the store owns its copy only as an
ordinary entity it may edit or delete, and an amendment to the seeded body does
not survive the next install. The binary never re-installs, never reads the copy
back, never validates it and never selects a set heuristically.
`smut conventions` with no argument lists the sets and their one-line
descriptions.

`learn` keeps its one pointer line and its separate epistemology: descriptive
and version-bound, where conventions are advisory and domain-bound.
