# Thread 3 synthesis — what `epistemic-records` should say

Producer: `worker` subagent, delegated by the orchestrating agent. Read-only.
This is a proposal to the operator, not a change request. Evidence is in
`raw/thread-3-doctrine-epistemic-schema.md` (cited below as `T3 § n`). Store
counts are from `target/release/smut` over `interview.local/.smut` (66 entities);
query examples are 0.2 `where`/`backlinks` syntax from
`.doctrine/slice/002/design.md:248-300, 341-405` — the built binary is 0.1 and
rejects `where`, so no `where` example below was executed, only the counts.

## The one question

**Does `epistemic-records` ship the draft's vocabulary — 13 types, 8 statuses,
6 labels, a `scope` field — or a shorter one in which every value answers a
question that the first-use store already asks?**

Approving the draft accepts: a list that its own reference store already
contradicts (4 types in use are unlisted, 2 listed types are unused), three
statuses that no query reads, and a field no entity carries.
Approving the shorter set accepts: smaller advice, and two capabilities given up
(traversal over what a record cites, and an enum for whose claim it is).

## Observations that force a choice

Each is evidence, not argument. Disputing one does not commit you to the
proposal.

- **O1 — doctrine never puts kind in a field.** Twenty-four entity kinds, each a
  directory + id prefix + verb family + own status vocabulary; the kind is in the
  id (`ASM-007`), so `show` needs no kind argument. (T3 § 1, § 6. ✓)
  *Forces:* the draft's `type` field carries, in smut, the whole weight doctrine
  spreads across storage. It cannot be justified by analogy to doctrine; it has to
  be justified by queries.
- **O2 — doctrine gives each kind its own status list, and they do not line up.**
  A question is `open|answered|obsolete`; a decision is
  `proposed|accepted|rejected|superseded`; a concept cannot be `superseded`.
  (T3 § 3. ✓) *Forces:* one 8-value list over 13 types makes every combination
  expressible, therefore none informative. `status: "retracted"` on a `session`
  will never be wrong, and so will never mean anything.
- **O3 — the only real store already keeps its own convention entity, and the
  draft contradicts it.** `vocab` records 15 types and 7 statuses in force, and
  was **amended by hand** when `superseded` was introduced ("Amendment, second
  session"). (T3 § 9; `smut get vocab` → `interview.local/.smut`.) The draft's 13
  types share 11 with it, add `source`/`refutation` (0 entities) and omit
  `model`/`option`/`project`/`theme` (6 entities). *Forces:* shipping this list
  tells the authors of six entities they were wrong after the fact.
- **O4 — 38 of 66 entities carry no `status` at all.** (T3 § 9. ✓) *Forces:* the
  convention must say when to omit the field, or it will be applied for
  completeness and 66 entities become 66 statuses. Doctrine states the same rule
  for its own facets: "Set a facet where it changes a triage decision, not for
  completeness." (`lib:reference/using-doctrine.md` § Facets. ✓)
- **O5 — three of the draft's 8 statuses are one fact spelled three ways.**
  `active` (2: a `project`, a `theme`), `current` (2: both `index`es), `recorded`
  (1: an `answer` that is the current focus). (T3 § 9. ✓) *Forces:* a reader
  cannot tell whether `active` and `current` are the same state on different types
  or different states. The list has to pick one word.
- **O6 — answeredness is stated twice.** `q-schema` is `answered` **and**
  `a-schema-resolved` carries `answers → q-schema`; all six answered questions
  have at least one incoming `answers` edge and one (`q-smut-test`) has two; but
  `a-friction-survey` is an `answer` with **no** `answers` edge. (T3 § 9; `smut get`
  over the store. ✓) *Forces:* today the two encodings agree, so this is latent
  drift, not a present bug — but the pair carries two different facts (resolved?
  by what?), and `answered` collapses them into one word the writer must keep in
  step with an edge.
- **O7 — `retracted` ships with zero instances, and the draft's own sentence
  conflates it with supersession.** "A retracted record is kept with its
  retraction intact rather than deleted; `superseded` is the status for that"
  (`design.md:586-587`). The one retired record is `finding-smut-16`, status
  `superseded`; doctrine keeps `retracted` and `superseded` as distinct values on
  EVD and memory and makes supersession *require* a named successor. (T3 § 3, § 7.
  ✓) *Forces:* if the two are the same word, one of them should not ship.
- **O8 — doctrine's "whose claim" is never an enum.** It is `[facet] source`
  (`canon|adr|external|technical|legal|compatibility|operator`) on constraints,
  plus `decided_by`, plus `[facet] provenance` on evidence — three fields, none of
  them a claim-author enum. And the live store answers the question already, by
  traversal: `criterion-scope-smut` and `a-doctrine-payoff` both carry
  `about: doctrine`, and no entity carries a `scope` field. (T3 § 2, § 4, § 9. ✓)
  *Forces:* the draft's single `scope` enum has 0 instances in the only store that
  could have used it, and the question it answers is already answerable.
- **O9 — doctrine enforces what the draft can only advise.** `link` rejects an
  illegal `(source,label)→target`; `memory status superseded` refuses without
  `--by`; `settle` restricts the resolving states to four and refuses a
  caller-supplied date; `concept` has no facets by refusal rather than omission.
  (T3 § 8. ✓) *Forces:* smut enforces nothing, so an element earns its line only
  if a query the reader will actually run depends on it.

## Hard constraints — and why each is not negotiable

- **C1 — smut is a standalone complement; it must not re-implement doctrine.**
  `criterion-scope-smut`: "smut is **not** intended to replace doctrine's stronger
  domain-specific types and their supporting tooling … What smut must be good at
  is therefore the opposite of doctrine's strength: cheap writing, free vocabulary,
  no validation … The cure for the traversal cost they impose is a filter, not a
  schema." ✓ `smut get criterion-scope-smut`; the brief's own boundary, as
  recorded in `design.md:18-21`, puts a validating schema, modelled containment
  and an in-store audit out of scope. *Not negotiable because* doctrine is the answer for exactly
  this domain, and a convention with teeth is a validating schema arriving by the
  back door. This is also why copying doctrine's facet tables (T3 § 2) would be
  cargo-culting rather than learning: their data is only worth what `edit <kind>`
  and `settle` do with it, and smut has no such verbs.
- **C2 — the set is printed advice, never validated, never auto-selected.**
  `design.md:541-548`, `DEC-017`: "the pipe is the whole mechanism". *Not
  negotiable because* a set that could check a store would need a schema engine
  (C1), and a set that selected itself would make the tool a policy engine — it
  would decide the store's ontology for the writer, which is the one decision
  smut exists to leave open.
- **C3 — the seed is one entity.** `conventions-epistemic-records`, one
  `convention` entity with a `body`, upserted by a fixed id (`design.md:591-605`).
  *Not negotiable because* it is DEC-017's mechanism. Its consequence is the
  budget: cost at install is zero, cost at use is one read per write. A list of 13
  types is free to ship and expensive to apply; every value is a decision the
  reader has to make at entity 3.
- **C4 — smut has no typed facets; every field is an ordinary JSON field.** Any
  `declare` payload is legal, nothing restricts a value, no verb stamps a date.
  (T3 § 2, contrast.) *Not negotiable because* it is the tool's design, not a gap.
  Consequence: the convention may name `type` and `status` because queries are
  built on them, but a convention that blesses `decided_on` or `basis` would be
  recommending a field nothing can honour — the writer cannot tell whether it is
  enforced, so it is enforcement theatre.

## Proposal

Keep the shape of § 9.2 — a small table plus one paragraph of discipline — and
change its contents. The draft's five elements become twelve rows: two changed,
one added, three dropped, six kept. Two of the changes cut more than they add.

| element | verdict | what it says | tradeoff in one line |
| --- | --- | --- | --- |
| `type` | **change values** | the 15 in use: `session`, `cluster`, `index`, `claim`, `question`, `answer`, `finding`, `observation`, `proposal`, `criterion`, `convention`, `model`, `option`, `project`, `theme`. Drop `source` and `refutation`. | The list grows by 4 and shrinks by 2; it stops contradicting its reference store, at the cost of two more values to choose from. |
| `status` | **cut 8 → 5** | `open`, `settled`, `current`, `superseded`, `retracted`. `open` not yet resolved; `settled` resolved; `current` the live version of a replaceable thing (index, convention); `superseded` replaced, successor named; `retracted` withdrawn as wrong, kept. Drop `answered` (a settled question), `active` and `recorded` (spellings of `current`/`settled`). | One word per fact, so `where` cannot return a contradiction — at the cost of three words a writer already reaches for. |
| `status` — omission rule | **add** | A type with no lifecycle carries **no** `status`. Only `superseded` and `retracted` are terminal, and a terminal record is never deleted. | Says when to omit, which is what keeps 38 of 66 entities unstatused — at the cost of a sentence a hurried writer will skip. |
| `part-of` | **keep** | child → container; the only containment edge. | smut does not model containment (C1), so a broken `part-of` is a dangling ref like any other — the label promises a tree the tool will not defend. |
| `about` | **keep** | entity → the subject it addresses. | Cheapest label in the store (48 entities, 84 refs); it is also the answer to "whose claim" (below), which means one label does two jobs. |
| `answers` | **keep** | answer → question; the only edge pointing up against `part-of`. | With `answered` gone this stops being redundant: `status` says resolved, the edge says by which answer — and one question may have two (`q-smut-test`). |
| `supersedes` | **keep, tighten** | new → old; the old record keeps `superseded`, the edge names the replacement, nothing is deleted. | Two writes (status + edge) instead of one, which is the only way the replacement is findable from either end — doctrine refuses the status without a successor (T3 § 7). |
| `scope` | **drop the enum, keep the rule in prose** | "State whose claim a claim is" — and do it by `about`-ing the entity that owns it (`doctrine` and `smut` are already entities or can be), with the basis in the body. | `where {scope: "external"}` becomes a traversal, and "external" (no entity behind it) has to be prose. |
| `refutes` | **drop** | A refutation is `retracted` on the refuted record, with the refuting record `about` it and the reason in the body. | "Who refuted it" is prose rather than a traversal — and there is no `refutation` type left to be the source of the edge. |
| `cites` (+ `source`) | **drop from 0.2** | Cite entity ids and prose in the body. | Gives up traversal over what a record rests on; if that is wanted, `source`+`cites` is the pair to add later — DEC-017's file-plus-match-arm is the path, and it is cheap. |
| direction rule | **keep, sharpen** | Write one direction per relation; `backlinks` answers the inverse; never write both. | Advice, not enforcement (C2) — and a `backlinks` query silently punishes a broken pair, which is the only pressure available. |
| "advice only" | **keep** | The tool prints this and refuses to check it. | Strictly weaker than doctrine on every enforcement axis (O9); the convention's only teeth are the reader's, which is the property being bought (C1). |

Net: two fields (`type`, `status`) and four labels (`part-of`, `about`, `answers`,
`supersedes`), plus three rules — each value carrying at least one instance in the
reference store. What the draft has and this does not: `source`, `refutation`,
`answered`, `active`, `recorded`, `refutes`, `cites`, `scope`. What this has and
the draft does not: 4 types already in use, the omission rule, the terminal rule,
and the successor requirement on `supersedes`.

## Where this disagrees with the draft, and the query under each reading

Each disagreement names the evidence that defeats the draft, then the same
question asked both ways against the first-use store.

**D1 — `answered` duplicates the `answers` edge.** (O6)

- Draft: `{"where": {"type": "question", "status": "open"}, "select": ["id"]}`
  → 8 questions (`q-audit-cost`, `q-channel-fix`, `q-fix-order`, `q-impact-query`,
  `q-more-friction`, `q-promotion`, `q-split`, `q-variance-mechanism`); the six
  answered ones are excluded by a status that repeats what their `answers`
  backlink already says.
- Proposal: the identical query returns the identical 8 ids, because a resolved
  question is `settled`. Same answer, one encoding. When the operator wants
  *which* answer: `{"from": ["q-schema"], "backlinks": ["answers"],
  "select": ["id", "title"]}` → `a-schema-resolved`, under both readings.
- The evidence that decides it: the two encodings are not two facts. Resolved and
  *by what* are separable, and only the edge carries the second. Keeping
  `answered` asks the writer to update a status and an edge that can drift apart,
  and there is no validator to catch it (C2) — and the store's own `vocab` records
  that it has paid this kind of bill once already, for six overlapping labels (O3).

**D2 — three statuses for one state.** (O5)

- Draft: `{"where": {"status": {"$in": ["active", "current", "recorded"]}},
  "select": ["id", "type"]}` → 5 entities: `doctrine`, `theme-graphs`,
  `audit-handoff`, `start-here`, `a-focus`. The query only works if the operator
  already knows all three spellings, which is the definition of a vocabulary that
  costs a reader.
- Proposal: `{"where": {"status": "current"}, "select": ["id", "type"]}` → 4 of
  the 5, in one value: `a-focus` is a resolved answer, so it becomes `settled`;
  `doctrine`/`theme-graphs` are the live versions of replaceable subjects, so
  `current`.
- Evidence: 5 instances spread over 3 words, and nothing in the draft says whether
  `active` on a `project` and `current` on an `index` differ (O5). Doctrine's
  answer to the same problem is a separate closed list per kind (O2); a single
  shared list cannot do that, so it must at least not spell one state thrice.

**D3 — the type list contradicts its own reference store.** (O3)

- Draft: `{"where": {"type": {"$in": ["model", "option", "project", "theme"]}},
  "select": ["id"]}` → 6 entities that the shipped list does not name, including
  the store's only `project` (`doctrine`); and
  `{"where": {"type": {"$in": ["source", "refutation"]}}}` → 0.
- Proposal: both become 6 and 0 of a *listed* set, i.e. the query and the list
  agree. The point is not the count — it is that a convention embedded in the
  binary and contradicted on first use teaches the writer to ignore it.
- Evidence: the store maintains its own convention entity and keeps it truthful by
  hand (O3). The draft's fixed-id seed formalises that practice but arrives with a
  stale list, which is the one failure mode the practice exists to prevent.

**D4 — `scope` asks a question the store already answers by traversal, and has
zero instances of its own.** (O8)

- Draft: `{"where": {"type": "claim", "scope": "external"}, "select": ["id"]}`
  → 0. The store's single `claim` carries no `scope`; `doctrine`/`smut`/`operator`
  are proper nouns frozen into an enum, and `external` is the only value with a
  real meaning.
- Proposal: `{"from": ["doctrine"], "backlinks": ["about"], "select": ["id",
  "type"]}` → `criterion-scope-smut`, `a-doctrine-payoff` — "whose claim" answered
  for any subject, including ones not yet enumerated, and answered the way the
  store already writes it.
- Evidence: doctrine splits the same question across three fields and never into an
  author enum (O8); the live store's 2 `about: doctrine` entities show the edge is
  the register in use. Cost, stated plainly: "external" has no entity to point at,
  so an external claim's provenance lives in the body only.

## The test query, under both readings

"Which open questions are unanswered?"

| reading | query | result today |
| --- | --- | --- |
| draft | `{"where": {"type": "question", "status": "open"}, "select": ["id", "title"]}` | the 8 open questions; "unanswered" is carried by `open`, so a second call is optional. |
| proposal | the same query — a resolved question is `settled` | the same 8 ids, from one fact. |

**Both readings pass the test, so the test cannot be the tiebreaker.** What
separates them is the next question an operator asks with the same store: *which
open question already has a draft answer?* Under the draft the operator cannot
tell whether `status: "open"` is trustworthy while an `answers` backlink exists —
two fields, one truth, no check. Under the proposal `open` means unresolved and the
edge means by what, and the query is
`{"from": [<the 8 ids>], "backlinks": ["answers"], "select": ["id"]}`: any returned
answerer is an open question with a partial answer, and an empty `traversal.edges`
for an id is an unanswered one. One call, no contradiction possible.

## What I would not decide for you

- **Whether `model`, `option`, `project`, `theme` belong in the list at all**, or
  are one `cluster`-like subject type misnamed in a store written before the
  convention existed. Folding them shrinks the list to 11; keeping them makes it
  truthful. The convention should not be the place that decides a store's
  ontology retroactively (C2).
- **Whether giving up `cites`/`source` is acceptable.** It is the only element of
  the draft whose loss removes a capability rather than a spelling.
- **Whether 5 statuses is too many.** If a smaller list is wanted, `current` is
  the next to go (2 instances, both indexes, and the absence of a status already
  means "not retired").

Approving a change here is an edit to `design.md:550-589` before the text is
embedded, because § 9.2 *is* the shipped string — there is no second copy to drift
(`design.md:591-594`).

## Limits of this synthesis

- The counts are from one store, 66 entities, written before any convention
  existed; they are an instance, not a sample. Deleted entities are unobservable —
  `smut delete` leaves no tombstone I could find — so "how often retraction was
  attempted and abandoned" cannot be known.
- `where` and `backlinks` do not exist in the built binary, so no example above was
  executed; every result shown is derived by hand from the same entities the
  queries would read.
- No claim here rests on doctrine's facet *enforcement* (T3 § 10) — only on its
  documented refusal surface.
