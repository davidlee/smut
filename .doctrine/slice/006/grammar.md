# Proposed grammar — `epistemic-records`

**Status: this file is superseded as the working artefact by
`epistemic-records.md` — the set text itself.** It is kept because it carries what the
set cannot: the reasoning, the measurement, the accounting against doctrine's facets, and
the alternatives that were considered and dropped. Read the set to know what to write;
read this to know why.

One place, one read. This is the **proposal**, not shipped text: it is what
`src/conventions/epistemic-records.md` would say, arranged so the whole thing can be
judged at once. Decisions taken by the operator on 2026-11-02 are marked ✅; what is
still open is marked ○ and collected at the end.

The principle: **doctrine's record model, carried into a store that has no facets,
no verbs and no validation**, plus the handful of things doctrine has no way to say.

---

## 1. Shape of a record

Every entity is a JSON object. Under this grammar a record carries:

| field | requirement | notes |
| --- | --- | --- |
| `smut_id` | required | identity, permanent, never reused. Opaque to the tool, not to you: a readable string is fine, a string you will want to change is not. (`id` until SL-004 lands — see §12.4.) |
| `type` | required | one of §2's ten kinds |
| `title` | required | what a rendered result must be readable as |
| `body` | required | the prose: the argument, the caveat, the thing that does not fit a field |
| `status` | per kind | only the words its kind's row lists; omit for a record with no lifecycle |
| `tags` | optional | cross-cutting labels, never a substitute for `type` |
| kind fields | per kind | §2; omitted fields keep their value on re-declare |
| dates | on transition | `YYYY-MM-DD` (§6 rule 7) |

Not carried from doctrine: `schema`, `version`, `slug`, `created`, `updated`. The
first two are envelope; the slug is a lookup alias doctrine needs because its id is
the kind, and git already holds the timestamps.

## 2. The kinds

Seven adopted verbatim from doctrine, three added. Statuses in **bold** are terminal
(no further status change; the record is kept, never deleted).

### Adopted

| kind | what it is | statuses | fields |
| --- | --- | --- | --- |
| `assumption` | something taken as true without proof | `held` `testing` **`validated`** **`invalidated`** **`obsolete`** **`dropped`** | `claim`, `basis`*, `confidence`*, `validation_plan`, `validated_by`‡, `validated_on`, `invalidated_by`‡, `invalidated_on` |
| `decision` | a choice, recorded with what it was chosen over | `proposed` **`accepted`** **`rejected`** **`superseded`** | `context`, `choice`, `alternatives`→, `rationale`, `consequences`, `decided_by`‡, `decided_on` |
| `question` | something we want to know | `open` **`answered`** **`obsolete`** **`dropped`** | `why_matters`, `answer`, `answered_by`→, `answered_on` |
| `constraint` | something that binds what we may do | `active` **`waived`** **`superseded`** **`retired`** | `statement`, `source`*, `applies_to`, `waiver_reason`, `waived_by`‡, `waived_on` |
| `evidence` | a datum we got, and how | `captured` `disputed` `confirmed` **`retracted`** **`superseded`** | `datum`, `provenance`*, `confidence`*, `reliability`†, `reported_by`‡ |
| `hypothesis` | a proposition under test | `proposed` **`confirmed`** **`refuted`** **`dropped`** | `proposition`, `predicts` |
| `concept` | an idea, nameable and reusable | `draft` `active` **`retired`** | none — the body is the whole record |

`*` closed vocabulary, doctrine's own (§5).
`†` the one addition to an adopted kind: doctrine is weak on *who* (§7).
`‡` a **reference to a `party`**, never a name as text — `decided_by`, `validated_by`,
`invalidated_by`, `waived_by`, `reported_by` (§6 rule 16).
`→` a **reference to another record** — `answered_by` (the record that answers the
question), `alternatives` (the `option` ids the decision chose between). `_by` is not a
promise about the referent: doctrine's own `superseded_by` names a record while
`decided_by` names a person.

Two doctrine subtleties worth keeping: `evidence` `confirmed` is **not** terminal
(subsequent evidence may reopen or supersede it, doctrine says so deliberately), and
`concept` carries no fields *by refusal* in doctrine — in smut nothing refuses, so
the convention says it instead.

### Added ✅

| kind | what it is | statuses | fields |
| --- | --- | --- | --- |
| `finding` | something learned, before it is evidence or a decision | `open` **`settled`** **`dropped`** | `confidence`*, `reported_by`‡ |
| `option` | a **mutually exclusive** candidate that addresses a problem or a need | `proposed` **`chosen`** **`rejected`** **`superseded`** **`dropped`** | none — body carries the argument; `addresses` names the problem |

`finding` exists because doctrine classifies at creation (`knowledge new <kind>`; the
kind is the directory and the id prefix), and at the moment of noticing you do not
yet know what you have. `option` exists because doctrine's `alternatives` is a
comma-separated string that cannot carry evidence, cost, or its own status, and
cannot be superseded — and because options are mutually exclusive by definition, so
they can only all be wrong together, never all right.

### Added ✅

| kind | what it is | statuses | fields |
| --- | --- | --- | --- |
| `party` | an actor behind a report: a person, team, organisation, system | `active` `discredited` **`retired`** | none — body carries what is known about them |

See §7. It is the only kind that buys nothing from doctrine's vocabulary, because
doctrine has no actor kind at all.

**It is also the kind most likely to leave.** A store with people in it will want
roles, affiliations and contact points, which is a world schema and not a record one
(IDE-005). If an actors set ever exists, `party` moves to it and this set keeps only
the edge: `reported_by` → a party, with `reliability` left where it is, on the report.

### `dropped` ✅

One word, added to every kind whose lifecycle has no verdict word for abandonment:
`assumption` `question` `hypothesis` `finding` `option`. Every doctrine status is a
verdict — `refuted`, `invalidated`, `rejected`, `obsolete`, `retired`. There is no
way to say "we stopped caring", and writing `refuted` there is a false statement.
`dropped` is terminal, keeps the record, and requires no successor.

## 3. Which records are live

The query that decides whether the status vocabulary is any good. *Live* — still
capable of changing the store — is:

| kind | live statuses |
| --- | --- |
| assumption | `held` `testing` |
| decision | `proposed` |
| question | `open` |
| constraint | `active` |
| evidence | `captured` `disputed` `confirmed` |
| hypothesis | `proposed` |
| concept | `draft` `active` |
| finding | `open` |
| option | `proposed` |
| party | `active` `discredited` |

A record with **no** status has no lifecycle at all — it is a stated fact, not a
process. Absence never silently means "current" or "live".

## 4. The labels

In smut a relation is **a field whose value is a reference** (`{"id": …}` or an array
of them). There is no edge table, so a label is a field name, and one direction is
stored while `backlinks` derives the other.

| label | direction | means | the question it makes cheap |
| --- | --- | --- | --- |
| `part-of` | child → container | containment only | what is inside X (`backlinks`) |
| `about` | record → subject | what the record addresses | what do we have on X (`backlinks`) |
| `answered_by` | question → answering record | the one record that answers it | is this answered, and by what (`follow`) |
| `addresses` | option → question or constraint | which problem a candidate is a candidate for | what was on the table (`backlinks`) |
| `supersedes` | new → old | the old record is replaced; the successor is named by this edge | what replaced this (`backlinks`) |
| `supports` | evidence → record | this datum bears for it | what is this record resting on (`backlinks`) |
| `disputes` | evidence → record | this datum bears against it | what argues against it (`backlinks`) |
| `cites` | record → record | this reasoning rests on that record | what does this assume (`follow`) |
| `reported_by` | record → party | who or what told us | what rests on Paul (`backlinks` from the party) |

`about` is settled: doctrine's word for this is a *role* on `references`, not a label,
so it does not transfer intact, and `about` is what an agent reaches for unprompted.

**`answered_by` points the way it does for a reason smut cannot argue with.** Absence of
a record's *own* field is a predicate (`{"answered_by": {"$exists": false}}`); absence
of an *incoming* reference is not queryable at all. So with the field on the question,
"which questions are open?" is one `where`; with the direction reversed the same
question is unanswerable, which is exactly why the one real store duplicates the fact in
`status` — and why that duplication has already drifted (below). Singular, too:
re-answering is supersession, not a second edge.

The store's practice, censused 2026-11-02, is the reverse direction and shows both
costs:

| fact | measurement |
| --- | --- |
| direction | the answering record holds `answers` — 5 records, 7 references, 6 questions |
| one answer, many questions | `a-verdict-focus` answers `q-mode`, `q-state` and `q-smut-test` |
| one question, many answers | `q-smut-test` is answered by both `a-verdict-focus` and `a-error-verdict`, both `settled`; nothing says which is current |
| status without an edge | `a-friction-survey` is an `answer`, `settled`, with no `answers` field at all |
| what the type bought | an `answer` is not a kind in this grammar: the answering record is a `finding`, `decision`, `evidence` or `concept` that a `question` points at — which is why the store needs an `answer` status vocabulary to say what the question already knows |

One answer serving many questions costs nothing in the new direction: three questions
`answered_by` the same record. What the inversion forbids is a question with two live
answers — the second must `supersede` the first, or it is not an answer to that question
at all (a `finding` `about` it, or a `supports`/`disputes` on the existing answer).

Two things to notice, both learned from doctrine's own counts rather than invented:

- **Direction is a cost decision, not a style.** `about` is written 48 times in the
  one store that has practice; `part-of` 65. Write the direction whose question you
  will ask most, because the other direction costs a store scan every time.
- **Doctrine's labels that are not here are not here for a reason.** `governed_by`,
  `needs`, `after`, `fulfils`, `shapes`, `spawns`, `reviews`, `related` are
  governance, work sequencing and process. They are other domains; a store that
  needs one should say so in its vocabulary record rather than borrow a word.

### Three questions that all read as provenance

Three labels answer "where did this come from", and the difference is what sits at the
far end. Learn them as **witness, premise, subject**:

| asked | label | far end | means |
| --- | --- | --- | --- |
| whose word is this? | `reported_by` | a `party` | the actor behind the report; what changes is their standing, not the record's content |
| what does this rest on? | `cites` | a record | a premise. If the premise falls, this record's reasoning falls with it |
| what does this bear on? | `supports` / `disputes` | a record | the subject. The evidence is *about* it; if the evidence falls, the subject loses weight and keeps its content |

Two things follow, and they are what stop the three collapsing into one:

- **Direction is not symmetric.** `reported_by` and `cites` are written by the record that
  owns the dependency, pointing away from itself. `supports`/`disputes` are written by the
  **evidence**, pointing at its subject. So from any record, what it used is outbound
  (`follow: ["cites"]`) and what argues for it is inbound
  (`backlinks: ["supports","disputes"]`) — one call each, neither duplicating the other.
- **A witness is reusable; a premise is per-record.** One `party` stands behind many
  reports, and reliability is a property of each report rather than of the party (§7).
  One record may be the premise for many — and that is a real dependency, not a shared
  witness.

### Where a store keeps its own vocabulary (rule 10)

The set delegates upward in exactly one place: a store may need a kind, a status or a
label this set does not name, and then the set cannot be the reference. `hygiene` already
asks for the values in use to be recorded; without a home, every store invents one (the
one real store has an entity with the id `vocab`, typed `convention`).

**Why the record exists at all** is mechanical, and worth saying because it is also why
the record will rot: smut cannot compute distinct values. "Which kinds does this store
use?" is not a query — there is no aggregation and no census (IDE-003), so it is a scan
and an eyeball. The vocabulary record is a **hand-written census**: derived data, written
by hand, in deliberate defiance of rule 12. It goes stale and nothing notices. If a
census verb ever lands, rule 10 should shrink to "ask the store", and this whole
subsection goes with it.

**Which kind is it.** None of the ten. The store's `convention` type has no home here —
a convention is a standard, and standards are another domain. The nearest kind is
`concept`: an idea, nameable and reusable, whose doctrine facet set is empty, so the body
is the whole record. A three-column table in a `concept` body is the natural form.

**How a reader finds it.** Three shapes:

| shape | found by | cost |
| --- | --- | --- |
| fixed id (`vocabulary`) | `smut get vocabulary` — one call, no query to know first | spends a permanent id; a store that already spent that name cannot take it back (IDE-002) |
| fixed title + predicate (`{"type":"concept","title":"Store vocabulary"}`) | one `where` the reader must already know | nothing consumed; can match zero or many; absence is less legible than a `missing` id |
| no home — "record it somewhere" | a scan | this is the failure the home exists to prevent |

Recommendation: **the fixed id**, with the title as the human signal. It is the same
shape DEC-017 and DEC-028 already use for seed ids, and an agent's first move in a
strange store is then `smut get vocabulary`, where `missing` is itself the answer.

**The tempting wrong home.** The set's own seeded entity is the obvious place to
accumulate, and it is unsafe by a settled finding: re-installing a set replaces every
field it supplies, `body` included (`hygiene`'s appendix says so; IMP-004 §B records it
as settled). A store's vocabulary put there is destroyed by the next install.

**What it must contain**, and nothing more: the kinds in use with the statuses each
takes; the labels in use with their direction; every value invented beyond this set, one
line each (`hygiene` rule 5 already asks for the one-liner); and the deviations — "we do
not use `party` here" is more useful than silence.

**Two documents, one authority each.** The *set* prescribes what should be; the *store's
record* describes what is. When they disagree, the record wins for "what is in this
store" and the set wins for "what to write next" — and the installed seed
(`conventions-epistemic-records`) already says which set is in force, so the record need
not restate it.

## 5. Closed vocabularies (doctrine's own)

| field | values |
| --- | --- |
| `confidence` | `low` `medium` `high` |
| `reliability` | `low` `medium` `high` |
| `basis` | `observation` `prior-art` `design-inference` `external-source` `operator-judgement` |
| `provenance` | `inspection` `experiment` `reproduction` `citation` |
| `source` (constraint) | `canon` `adr` `external` `technical` `legal` `compatibility` `operator` |

`confidence` is how sure **we** are of the datum; `reliability` is how much weight
**the report** carries given who it came from. They compose and are not
interchangeable: *"I am certain Paul said it, and I do not believe it"* is `confidence:
high`, `reliability: low`.

## 6. Rules

1. **Kind first.** Every record gets one of the ten kinds. If none fits, it is a
   `finding` — that is what `finding` is for.
2. **A kind's statuses are its own.** Only the words its row in §2 lists. There is no
   shared status list; `status: "retracted"` on a `party` is not expressible.
3. **No status means no lifecycle.** Never write one for completeness. Most records in
   a real store have none.
4. **Terminal is not deleted.** A record in a terminal status stays, with its body and
   its edges. Supersession names the successor; retraction says the content was wrong;
   `dropped` says we stopped.
5. **Promotion in place.** A `finding` that earns a kind is the same id re-declared
   with the new `type` — identity survives, edges survive, history is git's. Promotion
   is one way (`finding` → a kind); no other record changes type.
6. **One direction per relation.** Write the direction you will ask about; `backlinks`
   answers the inverse. Never write both ends by hand — that is two facts to keep in
   step with nothing to check them.
7. **Dates mark decisions, not existence.** `decided_on`, `answered_on`,
   `validated_on`, `invalidated_on`, `waived_on`, and `review_by` are worth writing;
   `created`/`updated` are already known to git and are worse there than here.
   `review_by` is kept deliberately (§12.6): it is honest even where it is not
   queryable, and that gap is the pressure worth dogfooding.
8. **An edge carries no data.** A reference is `{"id": …}` and nothing else — a
   reference object with extra fields declares that entity instead of annotating the
   edge. So a qualification ("strong evidence", "partial") lives in the body, or in a
   field on one endpoint, never on the link.
9. **Roles and degrees become labels or prose.** Doctrine's `references --role
   {implements, originates_from, concerns}` and `fulfils --degree {partial, full}` and
   `--descriptor` have nowhere to live here; promote the role into a distinct label
   if it is load-bearing, or say it in the body if it is not.
10. **Record the vocabulary in force.** The kinds, statuses and labels a store actually
    uses live in one `concept`-typed record whose body is the table. Doctrine compiles
    its vocabulary in; a permissive store has to write it down, and this is the one
    place where a store's own word beats doctrine's.
11. **Nothing here is checked.** The tool prints this set and never reads the store
    back. The only pressure is a query answering wrong later, which is the whole point.
12. ✅ **Storage discipline.** If a reader will filter or sort on it, it is a field; if
    it is the argument, it is the body. Doctrine's storage rule adapted: a fact
    written in prose cannot be queried, and a queryable field cannot argue.
13. ✅ **The minimum record.** Since nothing is checked, the only pressure available is
    a shape a reader can recognise as complete. One line each: an `assumption` with no
    `claim`, `evidence` with no `datum` or `provenance`, a `hypothesis` with no
    `proposition`, a `constraint` with no `statement`, a `question` with no
    `why_matters`, a `decision` with no `choice` and no alternative named — each is a
    note in the wrong kind.
14. ✅ **A kind prefix is a label, never a discriminator.** Doctrine's id prefix *is*
    the kind, because there the kind *is* the identity, and a kind change costs a new
    id. Here identity is permanent and `type` is promotable, so a kind-prefixed
    identity is a lie the first time a `finding` becomes `evidence`. So: prefix ids if
    they read better (`q-schema`, `ev-loader-overhead`), never filter a class by
    substring (`$contains` answers a superset at exit 0 — IMP-011's measured wrong
    answer), and read a prefix as a memento of what the record began as. An ordinary
    `label` field rewritten on promotion was considered and rejected: a second handle
    no traversal uses, when `title` already serves display.
15. ✅ **One shape per field, per store.** An actor field is a reference in every
    record or prose in every record, never both; the same for a date and for an id.
    Measured, not argued (probe store, 0.2.0): with `decided_by` a string in one entity
    and an array in another, `$contains` is a *substring* test on the string row and an
    *exact-element* test on the array rows, with nothing in the response saying which
    ran; `$eq` against a reference finds nothing because the field is an array; and
    `order` refuses the field outright (`an array has no ordering`). Two shapes are
    expressible — nothing validates — and they cost the queries the store exists to
    answer. Prose that cannot be a reference belongs in the body.
16. ✅ **An actor is a party, always.** Any field naming a person, team, organisation or
    system holds a reference to a `party` — `decided_by`, `validated_by`,
    `invalidated_by`, `waived_by`, `reported_by` — and never a name as text. A store
    keeps the few parties it actually has and reuses them; an actor not worth a record
    is not written into the field at all, and the sentence goes in the body. This is
    rule 15 applied to the one field family where the temptation to write prose is
    strongest.
17. ✅ **A question names its answer; the answer does not name the question.**
    `answered_by` (singular) on the `question`, pointing at the record that settled it.
    Not a style choice: an own-field absence is queryable and an incoming-reference
    absence is not, so this direction is what makes *which questions are open?* a
    predicate rather than a scan. Two live answers is the one thing it forbids — the
    newer `supersede`s the older, and either way the question moves its pointer.

## 7. The who — doctrine's thinnest side ○

Doctrine's `decided_by`, `validated_by`, `waived_by` and `answered_by` are free text,
and its `source` is an origin *class*, not a person. Nothing in doctrine can say:

- *this evidence came from Paul's email* — a durable actor, not a string;
- *between us, Paul is an unreliable witness* — a credibility that accumulates and
  applies to a class of reports, not one;
- *this is straight from the FudgeCorp CFO, so it carries more weight than the
  project manager's diet of distractions* — weight that is a property of the standard
  behind the report, not of how sure we are.

The proposal is two axes, because they answer different questions:

1. **The reporter is an entity** (`party`), so "what rests on Paul" is a `backlinks`
   query over `reported_by`, and Paul's track record accumulates as evidence about
   Paul.
2. **Reliability is a field on the report** (`reliability`), not on the reporter,
   because the same witness is strong on what they own and weak on what they were
   told. A party's *aggregate* standing is the `party` status (`discredited`), and the
   per-report `reliability` is where the nuance lives.

Cost, stated plainly: one kind, one field, one label, and it is the only part of this
grammar with no doctrine precedent — so it is the part most likely to be wrong. The
cheaper alternative is the two fields and no kind (`reported_by` as prose,
`reliability` as a field), which loses the traversal and the accumulated record.

## 8. Concept maps — companion set or a section here ○

Doctrine's concept map (`CM`) is a spec-tier artefact: a graph of concepts with
labelled relations. Nothing in the grammar above stops someone drawing one badly —
five undirected edges with five synonyms for the same relation, a "map" that is
really a list, concepts that are actually claims with no evidence.

What a map convention would say, in about six rules:

1. A map is one `concept` record; its body states the question the map answers.
2. Its members are `concept` records, each `part-of` the map. A concept in two maps
   is `part-of` the one it belongs to and `cites`d from the other — one home, no
   tangle.
3. Every relation inside a map is drawn from a **named vocabulary defined once** in
   the map's body, one line each, direction stated. A map with an undefined label is
   not a map.
4. A map carries **no** status, no dates, no confidence. It is arrangement, not
   assertion — the moment it asserts, it is a `claim`-shaped record and belongs in a
   kind.
5. `parent`/`child`, `part-of`, and `about` are not map relations — a map that needs
   a hierarchy is a taxonomy and is stated as one.
6. A map is never "the answer". It is the shape of the question.

**Recommendation: a companion set (`concept-maps`), not a section of this one.** Three
reasons: the epistemic set is already the size of `hygiene`; a map is arrangement
rather than record-keeping, so carrying both means reading status vocabularies to
draw a graph; and DEC-017 claims a set costs "a file plus a match arm", which a third
set would test cheaply. If one file is preferred instead, DEC-028 makes sections
addressable, so `smut conventions epistemic-records --section concept-maps` would
carry it at no mechanism cost.

## 9. A worked example

Seven records, exercising promotion, an option set, a witness, a supersession and a
drop. (Terse; bodies elided.)

```
{"id":"ctx-query","type":"concept","title":"The query surface",
 "status":"active","body":"What an agent can ask of a store, and what it cannot."}

{"id":"q-schema","type":"question","title":"Does smut gain typed schemas?",
 "status":"answered","why_matters":"Decides whether the conventions mechanism is load-bearing.",
 "answered_by":[{"id":"d-schema-in-file"}],"answered_on":"2026-11-02",
 "part-of":[{"id":"ctx-query"}]}

{"id":"opt-schema-in-binary","type":"option","status":"rejected",
 "title":"Compile the schema into the binary",
 "body":"Cheapest to ship; makes every store's ontology the binary's business.",
 "addresses":[{"id":"q-schema"}],"about":[{"id":"ctx-query"}]}

{"id":"opt-schema-in-file","type":"option","status":"chosen",
 "title":"A set file per domain, resolved by layer",
 "body":"Project, user, binary; local wins; no merge.",
 "addresses":[{"id":"q-schema"}],"about":[{"id":"ctx-query"}]}

{"id":"ev-loader-overhead","type":"evidence","status":"confirmed",
 "title":"A schema engine costs a validation pass per load",
 "datum":"78ms for 66 entities against a 4ms baseline",
 "provenance":"experiment","confidence":"high","reliability":"high",
 "disputes":[{"id":"opt-schema-in-binary"}],"about":[{"id":"q-schema"}]}

{"id":"f-cfo-cost","type":"finding","status":"open",
 "title":"Licence cost is three times what we assumed",
 "confidence":"low","reliability":"high",
 "reported_by":[{"id":"pty-cfo-fudgecorp"}],
 "body":"Stated in the call; no paper yet.",
 "about":[{"id":"ctx-query"}]}

{"id":"pty-cfo-fudgecorp","type":"party","status":"active",
 "title":"FudgeCorp CFO",
 "body":"Owns the budget line. Reliable on cost, evasive on dates."}

{"id":"pty-operator","type":"party","status":"active",
 "title":"The operator",
 "body":"Decides product posture; reachable in session."}

{"id":"d-schema-in-file","type":"decision","status":"accepted",
 "title":"Schemas ship as layered set files, not binary protocol",
 "context":"A schema engine is the obvious cure; the store is meant to stay permissive.",
 "choice":"A set file per domain, resolved project → user → binary, local wins.",
 "alternatives":[{"id":"opt-schema-in-binary"},{"id":"opt-schema-in-file"}],
 "rationale":"Costs a validation pass nobody has asked for; buys a vocabulary that ships with the work.",
 "consequences":["No enforcement; the set is advice","A domain set costs a file and a match arm"],
 "decided_by":[{"id":"pty-operator"}],"decided_on":"2026-11-02",
 "about":[{"id":"q-schema"}]}
```

Then promotion, in place, one call:

```
smut declare '{"id":"f-cfo-cost","type":"evidence","status":"captured",
               "datum":"Licence cost is 3× the assumed figure (call, 2026-11-01)"}'
```

And the queries the grammar is worth anything for:

| question | request |
| --- | --- |
| what is still live? | `{"where":{"$or":[{"type":"question","status":"open"},{"type":"finding","status":"open"}]},"select":["id"]}` |
| what rests on the CFO? | `{"from":["pty-cfo-fudgecorp"],"backlinks":["reported_by"],"select":["title"]}` |
| how was this question answered? | `{"from":["q-schema"],"follow":["answered_by"],"select":["id","title"]}` |
| what was on the table? | `{"from":["q-schema"],"backlinks":["addresses"],"select":["id","status"]}` |
| what did the operator decide? | `{"from":["pty-operator"],"backlinks":["decided_by"],"select":["id","title"]}` |

## 10. Accounting — how much of doctrine survived

Doctrine's facet fields, kind by kind, against this grammar:

| doctrine | here | verdict |
| --- | --- | --- |
| 32 facet fields across the seven kinds | 30 unchanged, same names | **intact** |
| `decision.alternatives` (comma-separated string) | ids of `option` records | **upgraded** |
| `question.question` (a field holding the question) | the record's `title` | **dropped** — redundant |
| spine: `id` `title` `tags` `status` | same | **intact** |
| spine: `record_kind` | `type` | **intact, different mechanism** — a field, not identity, which is what makes promotion possible |
| spine: `schema` `version` `slug` `created` `updated` | — | **dropped** — envelope, lookup alias, or git's job |
| `[evidence] supports[] / contradicts[]` | `supports` / `disputes` labels | **intact** (renamed to doctrine's own edge names) |
| `[evidence] notes[]` | the body | **dropped** as a field |
| `[relationships] supersedes / superseded_by` | `supersedes`, one direction | **collapsed** — the inverse is derived, which is strictly better |
| `[[relation]] label + target` | any reference field | **intact** |
| per-kind status vocabularies | identical, plus `dropped` | **intact + 1** |
| closed vocabularies (`confidence`, `basis`, `provenance`, `source`) | identical | **intact** |
| `references --role implements/originates_from/concerns` | — | **dropped** — no edge attributes; a role becomes a label |
| `fulfils --degree partial/full`, `--descriptor` | — | **dropped** — the qualification goes in the body |
| verb-written stamps | author-written dates | **kept in spirit, moved** — the convention asks for what the verbs used to do |
| validation and legal-pair refusal | nothing | **dropped by design** — the property being bought |
| `concept` facets = none, by refusal | none, by convention | **intact** |

By count: **30 of 32 facet fields cross unchanged**, all five closed vocabularies and
all seven status vocabularies cross unchanged, and what does not cross is doctrine's
bookkeeping (schema/version/slug/created/updated), its edge machinery (roles,
degrees, descriptors) and its enforcement. The single structural gain is that `type`
became a field rather than an identity, which is what makes promotion-in-place
expressible here and not in doctrine.

## 11. Deliberately not imported

Four of doctrine's adjacent families stay out, and the set should say so rather than
grow to meet them:

| doctrine family | why it is not here |
| --- | --- |
| requirements, specs, PRDs — normative truth | a different institution with its own tree, its own ids and its own coverage verb. Where one *binds*, it enters this grammar as a `constraint` whose `source` is `canon`, `adr` or `external`. Where one *measures*, it stays out. |
| `observation` (the friction ledger) | a different animal from a `finding`: adjudicated, faceted (`repository_context`, `product_surface`, counters), and about the *tool*, not the subject. Do not reuse the word `observation` here — the collision would make one store's census read as another's. |
| review ledgers, revisions, slices, RFCs | process. Their lifecycles are about work in flight, not about what is known. A permissive store that wants them has its own vocabulary record for them. |
| doctrine's `related` catch-all | present in doctrine's counts (25 edges) and deliberately absent here: it is the label you reach for when you have not decided what the relation is, and an unnamed relation is exactly the drift rule 6 exists to prevent. If a relation genuinely needs a new name, name it in the vocabulary record. |

## 12. Settled, and still open

Settled 2026-11-02: `party`, `reported_by` and `reliability` are in (§7); **every
field naming an actor is a party reference and never text** (§6 rule 16); **a question
names its answer** with a singular `answered_by`, so the `answers` label is gone and
re-answering is supersession (§6 rule 17); one shape per field, per store (§6 rule 15,
measured); `review_by`
stays (§6 rule 7); `about` stays (§4); concept maps are a **companion set** (§8); the
kind prefix is a **label, not identity** (§6 rule 14); the `smut_id` rename goes first,
so the set is written against the post-SL-004 format and states no fallback spelling;
and the `hygiene` overlap is resolved in favour of one copy — **this set keeps only the
kind-specific forms** (`dropped`, terminal-per-kind, promotion, dates,
no-data-on-edges, roles-into-labels) and defers the general discipline to `hygiene`,
whose rule 7 then shrinks to a sentence, because the detail it carries is this domain's
own. Two sets stating one rule is the drift DEC-016 refused between `learn` and
conventions.

Still open:

1. **Where the vocabulary record lives.** Analysed in §4 (recommendation: a `concept`
   under the fixed id `vocabulary`); still a decision rather than a draft.
2. **`review_by` as dogfood.** Kept on purpose: the field is honest, and "which records
   are overdue" being unselectable (`$eq` `$ne` `$in` `$exists` `$contains` — no ranges)
   is the pressure that may push the predicate surface toward ranges. If they land, this
   rule gains a query; if not, the field still tells a reader something.
