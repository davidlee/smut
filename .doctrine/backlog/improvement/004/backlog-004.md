# IMP-004: smut conventions: the epistemic-records set — its own design conversation

Deferred out of SL-002 (smut 0.2) on 2026-11-01, at operator direction: the
schema was the least falsified part of the design after four review passes, whose
findings all landed *around* the taxonomy without attacking it. 0.2 ships the
`smut conventions` mechanism with the `hygiene` set only. The content below is
the work in progress, preserved so the later conversation starts from evidence
rather than from a blank page.

## Why it is not a patch to 0.2

Every review finding about the set was about its *content*: which statuses exist,
who a claim belongs to, whether a citation needs an entity. Those are product
decisions about one narrow domain, not mechanism, and the mechanism is what 0.2
needed to settle. Bucket B (below) is a design conversation with its own
questions, not three sentences of repair.

## A. The draft as it stood (SL-002 `design.md` §9.2, revision 54)

| Element | Values |
| --- | --- |
| types | `session`, `cluster`, `index`, `claim`, `question`, `answer`, `finding`, `observation`, `proposal`, `criterion`, `convention`, `source`, `refutation` |
| statuses | `open`, `answered`, `settled`, `recorded`, `active`, `current`, `superseded`, `retracted` |
| `part-of` | child → container (containment; the only structural edge) |
| `about` | entity → subject it addresses |
| `answers` | answer → question it answers (points upward against `part-of`) |
| `supersedes` | new record → record it replaces; the old one keeps `superseded` |
| `refutes` | refutation → claim |
| `cites` | claim or finding → source |
| `scope` | `doctrine`, `smut`, `operator`, `external` — whose claim it is |

Plus: direction discipline (write one direction per relation, chosen by the
question you will ask most), the retraction/supersession distinction, and the
never-enforced stance — printed, never validated, never auto-selected.

## B. What review established — settled, do not reopen

- **Retracted ≠ superseded.** Withdrawn vs replaced, and `supersedes` is the edge
  that makes a record superseded (RV-009 F-3). The store this was derived from
  conflates them: its `vocab` says `superseded` retired `finding-smut-16` "with
  its retraction intact". Zero entities in that store carry `retracted`
  (RV-011 F-1, verified by `jq` over `interview.local/.smut`).
- **A seed is not edit-protected.** Re-install replaces every supplied field,
  `body` included (RV-012 F-4). Any advice that invites amending the installed
  copy in place must say so.
- **The store's practice predates the vocabulary**, which is why the install
  mechanism (fixed id, upsert) is right and the *content* is the open question.

## C. What the doctrine comparison says must be settled

Source: `.doctrine/slice/002/research/raw/thread-3-doctrine-epistemic-schema.md`
(findings, with a line-by-line comparison table) and
`thread-3-synthesis-proposal.md` (a proposal, not authority).

1. **One shared status list or per-kind vocabularies.** Doctrine keeps disjoint
   per-kind sets with per-value terminality; the live store spreads 7 statuses
   over 5 types with no per-type rule. The taxonomy argument has not been made.
2. **The missing disposition pair.** A status change in smut records neither who
   nor when, so a reader cannot tell a settled claim from a stale one. Thread 3
   calls this the largest omission.
3. **Types doctrine does not mint.** `source` as a type forces one entity per
   citation; `refutation` as a type plus a `refutes` edge states one fact twice
   (doctrine uses HYP `refuted` / EVD `disputed`); `answers` as an edge competes
   with an `answer` facet — the live store holds both encodings for `q-schema`,
   agreeing today and kept in step by nothing.
4. **Does the type list survive a query?** `type:"question", status:"open"`
   returns eight questions today. "Which records are still live?" has no defined
   answer, because no status is declared non-terminal.
5. **Whose claim is it.** `scope` collapses doctrine's `source` (origin),
   `decided_by` (author) and `provenance` (method) into one field, and two of its
   four values are project names that will date.

## D. Constraints that carry over unchanged

- Printed advice; never validated, never auto-selected, no schema
  (`criterion-scope-smut`). The moment it is checkable it is a schema with a
  friendlier name.
- Smut is standalone: it does not re-implement doctrine's record model.
- The mechanism is already decided and shipped in 0.2: `smut conventions [NAME]`,
  `--format json` emitting one fixed-id entity, pipe through `declare`.
- Adding a set is a file plus a match arm (DEC-017) — the deferred set costs 0.2
  nothing.

## E. The question this conversation starts with

Is the set's job **structure** (types, statuses, relations — a taxonomy argument)
or **workflow** (recipes for asking questions of a store — closer to what `learn`
already does, and what the first-use store actually demonstrates)? Answering that
first decides whether §A's table is the right artefact at all.

Trigger: after 0.2 ships and the `hygiene` set has been used on real stores; the
operator's own use is the evidence that settled the last set of questions, and it
is the only evidence this one should lean on.
