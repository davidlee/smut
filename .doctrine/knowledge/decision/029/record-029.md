# The concept keeps the name `conventions`

One decision, four surfaces: the command (`smut conventions`), the leaf directory
(`.config/smut/conventions/`), the seeded entity type (`"convention"`) and the seed
id prefix (`conventions-<name>`). They stay aligned under one word, and the word is
the one already in place.

## Why this word

It is already the word for **agreed practice rather than enforcement**, which is
precisely the axis a typed-schema future tests: `conventions` beside a later
`schemas` reads correctly on sight, with no explanation. Every other candidate either
weakens that marking or collides with something already in the surface:

- **`recipe`** — disqualified by collision: "the two-call recipe" is the guide's own
  name for a query procedure, twelve mentions and a section heading.
- **`read`** — collides with the store-reading verbs.
- **`magazine`**, **`trope`** — charming, and unrecoverable in `--help`.
- **`practice`** — reads as a verb phrase (`smut practice hygiene`).
- **`advice`** — hedges equally well, and is a rename for no gain.

The one objection that survives is that it is the only plural noun among actions,
which does not survive contact with real CLIs: noun subcommands are ordinary.

## Why now, and why it is cheap

Nothing has shipped on disk. Reopening after users hold files under
`.config/smut/conventions/` costs a migration rather than an edit, so the moment to
settle it is before the first release that reads that path.

**Reopen only if** schemas arrive and the vocabulary needs a finer distinction than
advice-versus-enforcement. That is the sole trigger, and it is recorded so the
question is not relitigated from taste.
