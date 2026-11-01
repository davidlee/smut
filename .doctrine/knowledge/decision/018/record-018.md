# Fixture: a normalised copy with provenance

The first-use store is promoted as a test fixture, but not as `.smut` internals:

- `tests/fixtures/interview/entities.json` — one JSON array of the 66 complete
  entities, public ids and fields preserved.
- `tests/fixtures/interview/PROVENANCE.md` — origin
  (`interview.local/.smut`), the two first-use sessions, the owner's ownership
  of the record, the copy date, the counts (66 entities, 44 distinct reference
  targets, 22 with no incoming reference; labels `part-of`, `about`, `answers`,
  `projects`) and the caveat that the store's `vocab` prose is stale because
  `projects` is used by `a-focus`.

Tests build a scratch store by declaring the array through the binary, then
exercise traversal, predicates and provenance against it. Internal UIDs, index,
lock and journal files are not committed: they are 0.1's private format and
would pin it. The fixture is acceptance evidence, not product data, and carries
no doctrine coupling.

Why promote at all: it is the only graph that contains the awkward cases the
release must answer — a superseded reference, a retracted claim,
mutually-referencing clusters, an edge pointing against containment, and a hit
and a miss for `backlinks` that were not invented to make a test pass.


## Operator caveat (sufficiency acceptance, 2026-11-01)

Before the fixture is committed, a fresh agent reads every entity for potentially
private content and reports it; the operator resolves anything found — remove,
redact, or do not commit the fixture. The normalised-array form stands either
way; the read-through is a phase obligation, not a design assumption.
