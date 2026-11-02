# RSK-001: Version discipline is unenforced — a stricter stored-validity rule can land without a version bump

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The condition

Recovery replays a committed manifest **strictly before** store validation
(`docs/store-format.md` § Recovery; `src/store/mod.rs:295-6` calls `recover()` then
`load()`). `validate_staged` (`src/store/journal.rs:608`) checks the manifest's shape —
version, basenames, uniqueness, disjointness, regular files, no symlink, no unlisted
entry — and never parses the payload.

So a staged put whose payload the loader would refuse is published, the journal is
renamed `done` and cleared, and the *next* load refuses a store whose committed batch
has already been applied. The store then refuses every command until an entity file is
hand-edited; the pre-crash content of that entity is already overwritten.

## Why it does not bite today, and why it will come back

Three things close the normal paths, and they are worth naming because the residual is
exactly what they do *not* cover:

- the request boundary refuses malformed identity (`Reject::InvalidId`,
  `src/entity.rs:43/530/745`, `src/declare.rs:916`), so a validating build cannot stage it;
- a crash mid-staging lands in the `prepare` state, which `recover()` discards rather
  than replays (`src/store/journal.rs:493-6`);
- **a cross-version manifest is refused by version before replay** — SL-004's
  PHASE-02/EX-7 and VT-6.

That third one is the whole defence. The risk is that it is a *convention*, not a
mechanism: nothing fails if a future slice tightens a stored-validity rule and forgets
the version bump. Every loader tightening re-opens publish-then-refuse for manifests in
flight at upgrade time, and the window widens monotonically with the number of rules
the loader knows, because recovery's acceptance set never grows with the loader's.

RV-024 F-2 was closed `tolerated` on this reasoning: the invariant
("a stricter stored-validity rule lands with a version bump, or must prove it cannot
refuse an in-flight manifest") is the repair, and this risk tracks that nothing enforces
it. Deliberately **not** in scope: the writer-bug leg (write-set serialisation
disagreeing with the identity predicate) — the write set is built from entities that
already passed the same predicate through the same serialiser, and that leg belongs to
the write-set tests, not to a risk register entry.

## What would retire this

Any one of:

- a test or guard that fails when a stored-validity rule is added without a version bump
  (the diff gate, an assertion tying the rule set to the version, or a checklist item
  with teeth);
- recovery-side validation of staged put payloads — the option declined in RV-024 F-2
  as disproportionate to the likelihood;
- `validate_staged` gaining a parse step for any reason, which would close the window as
  a side effect.

Do not close this because SL-004 shipped. It is not about SL-004; it is about every
subsequent tightening of the loader.
