# Provenance: the interview fixture

`entities.json` is a normalised copy of one real store — the first-use store
written during smut's own development — committed so that acceptance tests and
the guide's example run against real data in a clean checkout (DEC-018).

## Source snapshot

| field | value |
| --- | --- |
| source | `interview.local/.smut` — the first-use store, gitignored, never tracked |
| snapshot taken | 2026-11-02, from tree `2ea81fa` (0.1.1 plus SL-002 PHASE-01…03) |
| read through | `smut query`, the public surface, with the store directory as cwd |
| response | one envelope, 66 entities, `truncated: false` |
| entities | 66 |

What it is: a two-session interview notebook about smut itself — its design
questions, findings, options and proposals, organised into five clusters under a
`session`. It is product evidence, not defect evidence. Its entity types are
`answer`, `claim`, `cluster`, `convention`, `criterion`, `finding`, `index`,
`model`, `observation`, `option`, `proposal`, `project`, `question`, `session`
and `theme`.

What it is **not**: a schema, a validation fixture, or a vocabulary authority.
The store's own `vocab` entity is stale in two directions — it says `projects` is
unused while `a-focus` uses it, and `session`'s prose says nine questions are
open while the array holds eight. Tests therefore assert against *entities*,
never against vocabulary prose.

## Privacy: the verdict, and the waiver

The design (§11, DEC-018) requires a fresh-agent read-through before a personal
store is committed, with the operator resolving anything it finds. That
procedure was **waived by the operator on 2026-11-02**, who consented to
publication instead:

> "it's probably overcooked in this instance… the only thing I can think of was I
> mentioned watching Slow Horses on tv which I can live with the world knowing.
> I consent to the fixtures, skip the audit."

Recorded so that it reads as a ruling rather than as a missing step:

| field | value |
| --- | --- |
| read-through | **waived by the operator**, 2026-11-02 |
| basis | the owner's consent, given from recall of the store's contents |
| removed | **nothing** |
| redaction | **none** |
| resolved by | the owner (the operator), not by an agent |

**The residual, honoured.** Consent was given from recall, not from inspection,
so the committing phase (SL-002 PHASE-04) scanned the array before committing it:
every title and body was checked for credentials and secrets, contact details
(phones, email addresses, URLs), postal addresses and postcodes, financial and
health identifiers, and mid-sentence proper nouns. It read the personal-content
entities in full (`session`, `a-focus`, `convention`, `a-navigation`,
`q-focus`). Nothing materially private was found. The one personal mention is
the operator's own viewing of *Slow Horses*, which the ruling above names
explicitly as inside the consent. This scan is recorded as a check on the
waiver's premise, **not** as a second verdict: the verdict is the owner's.

One further note for a reader of the data: `finding-smut-10` quotes opaque store
filenames (`-AZOyWBemhRVhHYbmAiuv.json`, `EuFCJR-M-Pwd51pseMOlP.json`) as
examples of unreadable filenames. They are store-generated identifiers, not
credentials.

## Normalisation rules

The array is the only copy of this data in the repository. Nothing derived from
`.smut` internals — no index, no filenames, no journal or lock state — is
committed or referenced.

1. **One array.** A single JSON array of entity objects, top-level, `[]`-rooted;
   it is valid input to `smut declare`.
2. **Deterministic order.** Entities sorted by `id`, ascending, byte order.
3. **Field order.** Each entity keeps the store's own serialisation order: `id`
   first, then its remaining top-level keys lexically. Nested objects are
   lexically ordered and arrays stay positional, exactly as the store emits them.
4. **Values verbatim.** No field, value, entity or reference was dropped,
   rewritten, reordered within a field, or coerced. Numbers keep their literal
   form (`1` is not `1.0`); `null` is an ordinary present value.
5. **No projection.** Every entity is the complete stored entity — `body`
   included — because `select` would make the fixture a partial copy.

Reproduce it from the store (the store directory is the cwd, so `$PWD/.smut` is
the store found):

```sh
cd interview.local
../target/debug/smut query '{"limit":400}' | jq '.objects | sort_by(.id)' \
  > ../tests/fixtures/interview/entities.json
```

The normalisation is order-only, so the round trip is checkable: declared into an
empty directory, the array creates 66 entities and a full `query` reads back an
array structurally identical to this file.

## Adaptation for the identity break

The array was adapted once, after the normalisation above, spelling-only: 223
object keys `id` -> `smut_id`, because ADR-001 made `smut_id` the structural
identity key. Every id value, ordinary field, numeric literal, array position and
the 66-entity count is unchanged, and the inverse rename of the committed array
is byte-identical to the array as first committed (blob `f5b29de6`); a
type-strict structural comparison rejects both an altered value and a reordered
array. The rules above therefore hold as written, reading `smut_id` wherever they
say `id`: the array is still sorted by identity, still identity-first per entity.
Nothing else in this record moved. The source-snapshot facts, the counts and the
privacy ruling with its quotation are as they were, and the historical
quotations the array carries still read in their original spelling.

## Counts of the committed array

Measured from `entities.json` itself when it was committed (SL-002 PHASE-04), not
copied from the design's §2 source figures:

| measure | value |
| --- | --- |
| entities | 66 |
| entities carrying a `status` | 28 |
| distinct reference targets | 44 |
| entities with no incoming reference | 22 |
| clusters | 5: `cluster-friction`, `cluster-mind`, `cluster-schema`, `cluster-setup`, `cluster-thesis` |
| entities with `status: "retracted"` | 0 |
| statuses in use | `active`, `answered`, `current`, `open`, `recorded`, `settled`, `superseded` |
| open questions reached by the acceptance query | 8 |

Design §2's source figures (66 entities, 44 reference targets, 22 unreferenced)
agree with the array. Where a test needs a count it takes it from the array or
from an acceptance property, never from a document.

Two qualifications an auditor should read rather than re-derive:

- **No cluster references another cluster.** Every `type: cluster` entity's only
  reference is `part-of` to `session`, so the "mutually-referencing" property the
  suite pins is a pair of non-cluster entities — `finding-smut-17` and
  `prop-conventions-library`, verified in both directions. Design §2 and EX-3
  describe the source store as holding mutually-referencing clusters; the data
  does not, and the noun is recorded here rather than quietly changed. The design
  sentence is locked and is on the deferred-drift list for reconcile.
- **The round trip is structural, not arithmetic.** The suite asserts the store
  reads back the committed array entity for entity (not merely one entity per
  element), so a normalisation that dropped a field, coerced a value or reordered
  an array inside one entity fails the test rather than the note.

## References

- `design.md` §2 (the store as product evidence), §11 (the privacy residual),
  §13.1 Fixture row (what the suite proves), §13.3 scenario 2 (the acceptance
  query), §14.1 (`tests/fixtures/interview/**`).
- DEC-018 (fixture and privacy gate), DEC-016 (the two-call recipe the fixture
  answers).
- `tests/fixture.rs` — the only consumer committed today; later suites read this
  file with their own scratch store rather than committing a second copy.
