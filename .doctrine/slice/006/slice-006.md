# The epistemic-records set — doctrine's vocabulary, and only the gaps it cannot fill

## Context

`smut conventions` ships one set, `hygiene`. A second set, `epistemic-records`, was
drafted in SL-002 and deferred to IMP-004 (DEC-017) because every review finding
about it concerned *content*, not mechanism.

IMP-004 names three preconditions for resuming: 0.2 shipped, `hygiene` used on real
stores, and IMP-013's verdict on whether a project can ship a domain set. None is
met as of 2026-11-02: `Cargo.toml` is 0.2.0 with no release tag and CHR-001 open
before `cargo publish`; IMP-013 is design-settled but unimplemented; SL-004 is
mid-flight on the identity break. This slice was opened at operator direction to
progress the **content design** while those gates clear, on the operator's framing:

> the last draft was over-anchored on vocabulary an agent improvised in one
> conversation, against doctrine's types, which have been evolved in response to
> sustained use. Start from what doctrine's types cannot express.

That framing is the design. The draft's 13 types came from one 66-entity interview
store written before any convention existed; doctrine's seven epistemic kinds are
the product of a corpus that runs to thousands of records and dozens of slices.
Where the two disagree, doctrine is the evidence and the store is an anecdote.

## Scope & Objectives

Decide the set's content, and draft its text:

1. **The backbone: doctrine's seven epistemic kinds, adopted by name** — assumption,
   decision, question, constraint, evidence, hypothesis, concept — with their
   per-kind status vocabularies, their field names, and the `*_by`/`*_on` stamps.
2. **The additions: only what doctrine cannot express.** Each addition below names
   the thing doctrine has no way to say, and why a permissive store can.
3. **The shipped text** (`src/conventions/epistemic-records.md`), ready to embed
   when the shipping layer allows it.

Out of scope: adding the set to the binary (IMP-013's match arm), any check or
validation of stores (`criterion-scope-smut`: the moment it is checkable it is a
schema), and re-litigating IMP-004 §B (retraction vs supersession; seed replaces
body — both settled).

## Evidence

### Doctrine, verified this session (0.47.0)

`doctrine validate` scans 24 kinds. `glossary.md` gives the per-kind status
vocabulary; `doctrine knowledge edit <kind> --help` gives the facet fields and the
stamp pairs; `knowledge edit concept --help` prints `Refuse: concept records carry
no facet fields`.

| kind | status vocabulary | the fields | who/when |
| --- | --- | --- | --- |
| assumption | `held` `testing` `validated` `invalidated` `obsolete` | `claim`, `basis`, `confidence` | `validated_by`/`_on`, `invalidated_by`/`_on` |
| decision | `proposed` `accepted` `rejected` `superseded` | `context`, `choice`, `alternatives`, `rationale`, `consequences` | `decided_by`/`_on` |
| question | `open` `answered` `obsolete` | `question`, `why_matters`, `answer` | `answered_by`/`_on` |
| constraint | `active` `waived` `superseded` `retired` | `statement`, `source`, `applies_to` | `waived_by`/`_on` |
| evidence | `captured` `disputed` `confirmed` `retracted` `superseded` | `datum`, `provenance`, `confidence` | — |
| hypothesis | `proposed` `confirmed` `refuted` | `proposition`, `predicts` | — |
| concept | `draft` `active` `retired` | none, by refusal | — |

Closed vocabularies doctrine already owns and smut should reuse rather than invent:
`confidence` `low|medium|high`; `basis`
`observation|prior-art|design-inference|external-source|operator-judgement`;
`provenance` `inspection|experiment|reproduction|citation`; `source`
`canon|adr|external|technical|legal|compatibility|operator`. Shared across kinds:
`created`, `updated`, `tags`, prose body; the evidentiary edges `supports` and
`disputes`; `supersedes`/`superseded_by`; and free-text citations in a notes
channel — doctrine's answer to "how do I cite" is **not** a source entity.

Two facts about doctrine's design that the draft got backwards: `EVD` `confirmed` is
deliberately non-terminal (reopenable), and every stamp is a date written by a verb,
which is why doctrine does not suffer the "a change records neither who nor when"
problem that §C.2 of IMP-004 named as the draft's largest omission.

### The live store, re-censused (2026-11-02)

`interview.local/.smut`, 66 entities, exact counts via `jq` over the entity files:

| fact | measurement |
| --- | --- |
| distinct types | 15 — of which 7 map onto a doctrine kind, 8 do not |
| dominant type | `finding`, 18 of 66; `observation` 3 — neither is a doctrine kind |
| statuses in use | 7 improvised spellings (`open` `answered` `settled` `current` `active` `recorded` `superseded`); `retracted` has 0 instances |
| entities with no status | 38 of 66 |
| relation labels | `part-of` 65, `about` 48, `answers` 5, `projects` 1 |
| unlisted fields | `severity` 14 (doctrine's backlog/risk facet), one-off `stages`, `history`, `edges` |
| provenance or date fields | none — no stamps of any kind in 66 entities |
| citation practice | 1 of 66 bodies names another entity |
| vocabulary upkeep | the store keeps its own `convention` entity (`vocab`), hand-amended |

So the store's 15 types decompose as: **7 doctrine kinds** (`assumption` absent,
`claim` standing in for it), plus `finding`, `observation`, `option`, `cluster`,
`session`, `index`, `model`, `project`, `theme`, `convention`, `criterion` — a mix
of things doctrine cannot express and things that belong to another doctrine home.

### Supporting, from outside this repo

RFC-001 / IMP-011: 0 of 3 independently briefed authors used the advised `type`
spelling. Advice does not deliver itself; this slice can only make the advice
correct and short.

## The design

### 1. The backbone is doctrine's, adopted by name

The set lists doctrine's seven kinds with doctrine's statuses, field names and
stamps. The purpose this gives the set — the purpose the draft lacked — is that a
record written in smut under this convention **would be a doctrine record if
doctrine's machinery were there**: same kind names, same status words, same field
names, same dates. One vocabulary across two tools, no translation table, and a
store that can be imported when the store is what you have and doctrine is what you
need.

This is also the honest answer to "what is this set for". Not a taxonomy argument
invented for smut, and not recipes for querying (DEC-016 keeps the recipe in
`learn`): it is doctrine's record model, carried into a store that has no facets,
no verbs and no validation.

Carried into JSON, a kind's fields are ordinary top-level fields, and a status is a
value of `status`. Nothing validates them — that is the trade, not a gap.

### 2. The additions — what doctrine cannot express

Each of these is a decision. The evidence for each is doctrine's inability plus, in
most cases, the store reaching for it.

**A. A record before it has a kind — `finding`.** Doctrine classifies at creation:
`knowledge new <kind>`, and the kind is the directory and the id prefix. At the
moment of noticing you do not yet know whether what you have is evidence, an
assumption, or a decision; you have a finding. Doctrine's nearest is `EVD`, but
evidence asserts provenance and confidence from birth, and most findings have
neither yet. *Store evidence:* `finding` is the single largest type (18 of 66), and
`observation` adds 3. This is the strongest addition in the list — the thing
doctrine cannot say is the thing that was actually being said most often.

**B. An option as a thing, not a string — `option`.** Doctrine's alternatives are a
comma-separated string inside the decision's facet. A string cannot carry its own
evidence, cost or status, cannot be superseded, and cannot be linked to the
evidence that killed it. When options are worth weighing they are worth being
records: `option` + `cites`/`supports` evidence, and the decision's `alternatives`
names the ids it chose between. *Store evidence:* 3 `option` entities. *Cost:*
one more type, and the decision's `alternatives` field becomes a list of ids rather
than prose.

**C. An inquiry that ended without a verdict — a status, not a type.** Every
doctrine status is a verdict: `refuted`, `invalidated`, `rejected`, `obsolete`,
`retired`, `withdrawn`. There is no word for "we stopped caring". Writing `refuted`
on an abandoned line of inquiry is a false statement, and `obsolete` means
superseded by circumstance, which is different. Add one: `dropped`, terminal,
content kept, no successor required. *Honest weakness:* this is argued from
expressiveness, not from the store — no entity in the reference store records an
abandoned inquiry, possibly because there was no word for it.

**D. Changing your mind about what a record is — a rule, not a type.** In doctrine
the kind *is* the identity, so promotion costs a new id and a supersession edge. In
smut `type` is an ordinary field and `declare` patches fields (omit a field and it
keeps its value), so promotion is one call and identity survives: `finding` →
`evidence` → `decision`, same id, history intact. This is the one capability the
permissive store has and doctrine structurally lacks, and it is worth saying out
loud because it is the reason `finding` is safe to add — nothing written as a
finding is stranded there.

**E. The container of an inquiry — a rule, not a type.** Doctrine's containers are a
slice (work, with phases and a gate) and a memory typed `thread`; neither means "the
conversation that produced these records". Smut needs no new type for it: `part-of`
to a container entity, where the container is a `question` or `decision` when it is
one and a `concept` otherwise. *Store evidence:* `part-of` on 65 of 66 entities, and
a `cluster`/`session` pair invented to be the container. The rule subsumes both.

**F. Staleness horizon — a field, with an honest limit.** Memories carry `review_by`,
`lifespan` and `trust`; knowledge records carry none, so a decision from a year ago
asserts as strongly as today's. Smut can carry `review_by` (`YYYY-MM-DD`) as an
ordinary field, **but** the predicate surface has no ranges — `$eq` `$ne` `$in`
`$exists` `$contains` only — so "which claims are overdue" is not selectable, and
the only available query is `order` plus `limit` for the oldest. Include the field;
claim nothing the tool cannot do.

**G. Confidence on something that is neither an assumption nor evidence — a field.**
Doctrine puts `confidence` on exactly those two kinds. Often the thing worth
recording about a finding or a decision is how sure you are. One shared field, using
doctrine's own `low|medium|high`.

### 3. What the store invented that belongs to another home

Not additions; the convention should say where each goes rather than enumerate it:

| store type | where it belongs |
| --- | --- |
| `index` | a signpost — a container, `part-of` root or a `concept` |
| `theme`, `project` | grouping labels, not kinds — `part-of` a container |
| `model` | a `concept`, with a concept map when its relations matter |
| `convention` | a standard (`STD`); in-store it is the vocabulary record |
| `criterion` | a `constraint` when it binds, a requirement when it measures |
| `severity` | a backlog/risk facet, not an epistemic field |

### 4. What the draft has that doctrine does not, and is dropped

- `source` as a type — doctrine cites in notes and prose, never as an entity.
- `refutation` as a type and `refutes` as an edge — doctrine says it once, as
  `HYP refuted` / `EVD disputed` / `disputed` status, and as `supports`/`disputes`
  edges from evidence to what it bears on. Two encodings of one fact is the drift
  the store's own `vocab` amendment was needed to repair.
- `scope` as an enum — doctrine splits the question across `source`, `decided_by`
  and `provenance`, and never into a claim-author enum; the store answers it with
  `about`, and doctrine answers it with the `about`-shaped thing (the record names
  its subject).
- `answered` `active` `recorded` as extra status words — each is a doctrine status
  under a different name (`answered` = the question's `answered`; `active`/`recorded`
  = `accepted`/`answered`/`current` depending on kind).

### 5. The decisions, in plain terms

1. Adopt doctrine's seven kinds, per-kind statuses, field names and stamps as the
   set's backbone. *(recommend yes)*
2. Add `finding` (A). *(recommend yes — the store's dominant use, and promotion in
   place makes it safe)*
3. Add `option` (B). *(recommend yes, at the cost above)*
4. Add the terminal status `dropped` (C). *(recommend yes; weakest evidence)*
5. State the promotion rule (D) and the container rule (E) as rules, no new types.
   *(recommend yes)*
6. Include `review_by` (F) with its honest limit, and shared `confidence` (G).
   *(recommend yes)*
7. Drop `source`, `refutation`, `refutes`, `scope`, and the three extra status
   words (§4). *(recommend yes)*
8. `cites`: name it as the single optional label for a record resting on another
   record, since in a schemaless store an unnamed label costs spelling drift
   (`cites`/`source`/`refs`) which is IMP-011's failure class. *(recommend yes —
   one line; the alternative is "provenance is prose", which is what the store
   actually does)*

**Open, not recommended either way:** whether the set should also carry
`retracted`/`superseded` guidance at all, or defer to `hygiene` rule 7, which
already states it because of this domain. Deferring keeps one copy; carrying it keeps
the set readable alone.

### 6. Decisions taken (second pass, 2026-11-02)

All eight above were accepted, with these additions and resolutions:

- **`option` is defined by problem, not by shape** — a *mutually exclusive* candidate
  that addresses a problem or a need. Mutuality is the property that earns the type:
  options can all be wrong together, never all right, which is why they carry
  `addresses` and not `supports`.
- **`party`, `reported_by`, `reliability` are in** (the *who* gap; §7 of the grammar).
  `reliability` sits on the report, not the reporter, because the same witness is
  strong on what they own and weak on what they were told; `confidence` (how sure we
  are of the datum) and `reliability` (what weight the report carries) are distinct and
  compose.
  **Anything naming an actor is a reference to a `party` and never text** — every `*_by`
  field and `reported_by`, with no prose fallback. The fallback was measured and
  closed: a field that is sometimes text and sometimes a reference changes what
  `$contains` means per row (substring versus exact element, silently) and cannot be
  sorted at all (`order` refuses: "an array has no ordering"). An actor not worth a
  record is not written into the field; the sentence goes in the body. `answered_by`
  is dropped for the same reason it existed — doctrine's field named a person, `answers`
  names the answering record, and the pair read as one fact written twice.
- **`review_by` stays**, deliberately, as dogfood: the field is honest even where it
  is not queryable, and "which records are overdue" being unselectable (no range
  predicates) is the pressure that may push the predicate surface.
- **`about` stays** rather than doctrine's `concerns`, which is a *role* on
  `references` and does not transfer as a label; `about` is also what an agent reaches
  for unprompted.
- **Concept maps: companion set**, not a section of this one.
- **The `hygiene` overlap is resolved in favour of one copy.** This set states only
  the kind-specific forms — `dropped`, terminal-per-kind, promotion in place, dates,
  no-data-on-edges, roles-into-labels — plus one line deferring the general discipline
  to `hygiene`. `hygiene` rule 7 then shrinks to a sentence, because the detail it
  carries is this domain's own. Two sets stating one rule is the drift DEC-016 refused
  between `learn` and conventions.
- **Ids: the kind prefix is a label, never identity semantics.** Doctrine's prefix *is*
  the kind because the kind *is* the identity (directory + prefix), so a kind change
  costs a new id. Here identity is permanent and `type` is a promotable field, so a
  kind-prefixed identity would be a lie the first time a `finding` became `evidence`.
  Rule: the prefix is an optional reading aid, never a discriminator (`$contains` as a
  class filter answers a superset at exit 0 — IMP-011), and after a promotion it is a
  memento of what the record began as. Zero-cost placement: choose it in the identity
  string. The alternative — an ordinary `label` field, rewritten on promotion — costs a
  second handle that no traversal uses, and `title` already serves display.
- **The set is written against the post-SL-004 format.** ADR-001 moves identity from
  `id` to `smut_id`, which makes `id` ordinary data; the set says `smut_id` for
  identity and must not ship before the rename, or must say which spelling is in force.
  This is the one place where the set's text depends on another slice's landing.

### 8. What this design says about schemas

Two of the set's rules are schema rules written as advice: **one shape per field per
store** and **an actor is always a party reference**. Both were measured, not preferred —
a mixed field changes what `$contains` means per row and cannot be sorted — and neither
can be checked by anything.

That makes this slice the first *instance* of what QUE-001 said would answer its question
("users authoring conventions sets that are, in substance, schemas … expressed as prose
because nothing else exists"). Recorded there with the operator's disposition shift
alongside it and no decision attached: `declare` still refuses nothing, and
`criterion-scope-smut` still says the cure for traversal cost is a filter, not a schema.
The option space worth naming is validation **on demand** (a verb that reports
deviations, never the write path) versus validation on write; nothing here depends on
which, because the text is advice either way.

### 9. Where the store's own vocabulary lives

Decided: a `concept` under the fixed id `vocabulary`, its body a table of kinds, statuses,
labels and deviations. The reasoning — why a fixed id, why `concept`, why the record is a
hand-written census, and why the set's own seed is the wrong home for it — is in
`grammar.md` §4.

Two consequences worth carrying:

- The installed copy **is** the version stamp (`conventions-epistemic-records` carries
  `version`), so no second registry is needed or wanted. What is genuinely missing —
  nothing compares an installed copy's version with the version the loader would install
  now — is backlogged as **IMP-017**, deliberately as a read-only check and not as an
  emitter, because a store-level registry would need merge semantics `declare` does not
  have.
- The `vocabulary` record is derived data kept by hand, which this set otherwise
  forbids. It is the one deliberate exception, and IDE-003 (a store has no census) is the
  item that would retire it.

## Affected surface

- `.doctrine/slice/006/epistemic-records.md` — **the set text, drafted here rather than
  in `src/`**. It is inert until IMP-013 adds the match arm, and a file in
  `src/conventions/` that nothing can reach is dead weight plus a contradiction with
  `hygiene`'s "deferred" paragraph. On the landing it moves to
  `src/conventions/epistemic-records.md`, and the paragraph and rule 7 in `hygiene.md`
  change in the same commit.
- `.doctrine/slice/006/grammar.md` — the reasoning, the measurements and the accounting
  against doctrine's facets. Superseded as the working artefact, kept as the record of
  why.
- `.doctrine/slice/006/notes.md` — the adversarial pass on the taxonomy and its
  dispositions.
- `src/conventions/hygiene.md` — two edits at landing: the paragraph naming the set
  deferred, and rule 7 shrinking to a sentence with the detail living here.
- `src/conventions.rs` — untouched by this slice. IMP-013 owns the arm.

## State and gate

Design decided; text drafted and adversarially reviewed (`notes.md`). Implementation is
gated on two other slices, and nothing in the design waits on them:

| gate | what it fixes | state |
| --- | --- | --- |
| SL-004 — identity is `smut_id` | the set says `smut_id`; operator direction is that the rename ships first | started, 1/3 |
| IMP-013 — layered sets and the loader | the `conventions` match arm that makes the file reachable at all | design-settled, unimplemented |
| CHR-001 / 0.2 publish | IMP-004's original trigger, and the evidence base for the *next* revision | unpublished |

No plan and no phases: a plan for a slice that cannot start is a plan written twice. The
slice stays at `design` until the gate opens; SL-004 and SL-005 are the work in the tree.

## Risks and assumptions

- **R1.** The one store is an instance, not a sample. Its counts support *which* gaps
  matter, not how often.
- **R2.** Advice does not deliver itself (IMP-011). This slice can make it correct and
  short, not read.
- **R3.** ~~Four review passes landed around the taxonomy.~~ **Addressed** — `notes.md`
  records the pass that attacked it, eleven findings, three of them text-changing and two
  of them defects in this slice's own worked example. A formal `RV` ledger is still owed
  if the set is to ship with a review trail.
- **A1.** Doctrine is the vocabulary reference; where smut's store and doctrine disagree,
  doctrine wins the name and the store wins nothing.
- **A2.** Doctrine's *machinery* (facets, verbs, refusals) is not available, so a doctrine
  field name in smut is a convention, not a facet — prose that says so is part of the set,
  not a caveat to it.

## Verification and closure intent

The set is judged by the questions a reader can answer, not by its length:

| question | answerable under the design |
| --- | --- |
| what kind of record is this? | one of ten kinds, or a `finding` before it is one |
| what state is it in? | its kind's own status list, not a shared one |
| when did it change, and who moved it? | `*_on` and a `*_by` party reference, `YYYY-MM-DD` |
| what answered this question? | `answered_by`, one hop |
| what does this rest on? | `cites` (premise), `supports`/`disputes` (subject), `reported_by` (witness) |
| was it replaced or withdrawn? | `superseded` with the successor named by edge, vs `retracted` |
| may I change what kind it is? | yes — one `declare`, identity survives |
| may I add a kind? | as a gap doctrine cannot fill, recorded in the store's `vocabulary` |

Closure: this is the design half of IMP-004, done and written down. The remaining work is
gated, listed above, and named in IMP-004 so the next reader starts from here.
