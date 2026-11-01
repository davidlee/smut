# Raw thread 3 — doctrine epistemic record schema and typed facets

Producer: `worker` subagent, delegated by the orchestrating agent. Read-only.
`doctrine` is the installed binary (0.47.0, `/home/david/.local/share/doctrine/bin/doctrine`);
there is no doctrine source checkout, so every claim below is from CLI output, a
library asset read through the CLI, or a file read of this repo's own store.

## What was run

- `doctrine --help`, `doctrine knowledge --help|new|edit|status|settle|inspect|list|show|paths --help`,
  `doctrine knowledge edit <kind> --help` for all seven kinds.
- `doctrine --version` (0.47.0); `doctrine design contract --format prompt`
  — the one schema-printing surface this build has (see § 2 and § 10). The bare
  `doctrine contract` form does not exist; the verb is design-scoped.
- `doctrine library tree`; `doctrine library show` of `reference/glossary.md`,
  `reference/using-doctrine.md`, `templates/knowledge-{assumption,decision,question,constraint,evidence,hypothesis,concept}.toml`,
  `templates/memory.toml`.
- `doctrine memory --help|record|status|verify|backlinks|list --help`,
  `doctrine memory show mem.signpost.doctrine.knowledge`, `.storage-model`,
  `.entity-engine`, `.relating-entities`, `.conventions`.
- `doctrine observation --help` and each of its subverbs' `--help`.
- `doctrine link|unlink|needs|after|supersede|tag|validate|doctor|catalog|export|graph|relation|search|blockers|estimate|backlog new --help`;
  `doctrine catalog scan | jq`; `doctrine relation census`; `doctrine doctor --json`;
  `doctrine validate`; `doctrine show|knowledge inspect|knowledge list` on real records.
- File reads of `.doctrine/knowledge/decision/0{01,14}/*`, the `014-<slug>` symlink,
  and `interview.local/.smut` through `target/release/smut` (read-only queries).

## Verification legend

- ✓ — verified this session by a command or a file/asset read, cited inline.
- unmarked — cited from a doctrine document but not independently checked; or a
  claim the read-only constraint made unobservable.

No claim is marked ✓ on plausibility. Where a probe would have needed a write,
the claim is left unmarked and the missing probe named.

## 1. Record kinds

Doctrine does not have *a* record kind with a type field. It has one **kind per
file family**, each with its own prefix, directory, status vocabulary, verbs and
facet schema. `doctrine validate` prints the full list it scans:

```
SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE, RV, REC,
ASM, DEC, QUE, CON, EVD, HYP, CPT, CM, REV, RFC          ✓ doctrine validate
```

The epistemic subset is `knowledge_record` — seven kinds ✓ `lib:reference/using-doctrine.md`
§ Which home for which record, ✓ `doctrine knowledge new --help` (`possible values:
assumption, decision, question, constraint, evidence, hypothesis, concept`).

Doctrine refuses to fold adjacent kinds into the same one, and says why: a
`knowledge_record` is "not work; not the backlog" (backlog is `issue, improvement,
chore, risk, idea` with lifecycle `open|triaged|started|resolved|closed`); an ADR
is a domain (`proposed → accepted → superseded`); a memory is a durable fact
(`concept|fact|pattern|signpost|system|thread`, `--type` **required**) ✓ `doctrine memory record --help`
✓ `lib:reference/using-doctrine.md` § Which home for which record. There is a
membership test rather than a shared type enum: capture in the wrong home is the
failure mode the kinds exist to prevent, and the test arbitrates rather than the
schema.

**Consequence for smut:** the *number of kinds* is cheap in doctrine because it is
a storage-layer fact (directory + prefix + verb family). In smut it can only be a
`type` value, i.e. a reader-held convention.

## 2. Typed facets per kind

Every knowledge record is `record-NNN.toml` (structured) + `record-NNN.md` (prose) ✓
`.doctrine/knowledge/decision/014/` listing. The `.toml` carries a common spine
plus a kind-specific `[facet]` table:

| block | fields | source |
| --- | --- | --- |
| spine (all kinds) | `schema`, `version`, `id`, `slug`, `title`, `record_kind`, `status`, `created`, `updated`, `tags[]` | ✓ `lib:templates/knowledge-decision.toml`, ✓ `.doctrine/knowledge/decision/014/record-014.toml:1-11` |
| `[evidence]` (all kinds) | `supports[]`, `contradicts[]`, `notes[]` — commented "free-text citations" | ✓ same, `record-014.toml:22-25` |
| `[relationships]` (all kinds) | `supersedes[]`, `superseded_by[]` — commented "verb-written" | ✓ same, `record-014.toml:27-29` |
| `[[relation]]` (repeatable) | `label`, `target` | ✓ `record-014.toml:31-32` |

Facets, verbatim from `doctrine knowledge edit <kind> --help` ✓ and the shipped
templates ✓ (closed sets are the template comments):

| kind | prefix | facet fields | closed value sets |
| --- | --- | --- | --- |
| assumption | ASM | `claim`, `confidence`, `basis`, `validation_plan`, `validated_by`, `validated_on`, `invalidated_by`, `invalidated_on` | confidence `low/medium/high`; basis `observation/prior-art/design-inference/external-source/operator-judgement` |
| decision | DEC | `context`, `choice`, `alternatives[]`, `rationale`, `consequences[]`, `decided_by`, `decided_on` | — |
| question | QUE | `question`, `why_matters`, `answer`, `answered_by`, `answered_on` | — |
| constraint | CON | `statement`, `source`, `applies_to[]`, `waiver_reason`, `waived_by`, `waived_on` | source `canon/adr/external/technical/legal/compatibility/operator` |
| evidence | EVD | `datum`, `provenance`, `confidence` | provenance `inspection/experiment/reproduction/citation`; confidence `low/medium/high` |
| hypothesis | HYP | `proposition`, `predicts` | — |
| concept | CPT | **none** | — |

- The concept kind carries no facet fields and this is enforced by refusal, not
  omission: `doctrine knowledge edit concept --help` prints the description
  `Refuse: concept records carry no facet fields` ✓, and the template's facet
  table exists but is empty ("currently empty; seeded for scaffold-order
  invariant") ✓ `lib:templates/knowledge-concept.toml`.
- **No facet field is required.** Templates seed every facet field empty with the
  comment "every field typed; seeded empty (captured later)" ✓. `doctrine
  knowledge new <kind> <title>` takes only `--slug` and a title ✓ `knowledge new --help`:
  a record exists with all facets blank and is filled in later by `edit`.
- Scalars vs arrays is a per-field decision, and arrays are comma-separated on the
  CLI (`--alternatives`, `--consequences`, `--applies-to`) ✓ `knowledge edit decision --help`.
- `[facet]` is a doctrine-wide concept, not a knowledge-record one: `doctrine
  estimate`, `doctrine value`, `doctrine risk`, `doctrine compare`, `doctrine tag`
  attach facets to other kinds, and they feed the advisory worklist ✓
  `lib:reference/using-doctrine.md` § Facets — what feeds the worklist.
- Facets are rendered by both `doctrine show` and `doctrine knowledge inspect`;
  `show` adds the prose body, `inspect` does not ✓ (`knowledge inspect --help`:
  "metadata only (no prose body)", observed on DEC-001).

The facet/prose split is a *rule*, not an option: "structured, queried, or
derived data goes in TOML; prose goes in MD; **never** put queried or derived data
in a prose body" ✓ `doctrine memory show mem.concept.doctrine.storage-model` ✓
`lib:reference/using-doctrine.md` § Storage tiers. A prose body may be empty by
design, and judging an entity hollow from prose alone is named a false reading ✓
same doc, § Reading entities.

**Observation — the prose tier is opaque and stored verbatim.** DEC-014's body is
17 bytes containing the literal string `/tmp/dec14-new.md` ✓
`.doctrine/knowledge/decision/014/record-014.md`, ✓ `doctrine show DEC-014 --json`
(`"body": "/tmp/dec14-new.md"`), while its facet table is fully populated ✓
`record-014.toml:13-21`. Nothing dereferences a body; nothing validates it.

## 3. Status and lifecycle

**Status is a stored, authored, per-kind-closed field.** ✓ `status = "proposed"` in
`record-014.toml:8`; ✓ `lib:reference/using-doctrine.md` § Storage tiers: "Lifecycle
fields (e.g. a `status`) live in the TOML and are hand-edited there."

Each kind has **its own** vocabulary, and there is no shared list ✓
`lib:reference/glossary.md` § knowledge record lifecycle vocabulary, ✓
`doctrine memory show mem.signpost.doctrine.knowledge`:

| kind | status vocabulary | seeded by `new` |
| --- | --- | --- |
| assumption | `held/testing/validated/invalidated/obsolete` | `held` |
| decision | `proposed/accepted/rejected/superseded` | `proposed` |
| question | `open/answered/obsolete` | `open` |
| constraint | `active/waived/superseded/retired` | `active` |
| evidence | `captured/disputed/confirmed/retracted/superseded` | `captured` |
| hypothesis | `proposed/confirmed/refuted` | `proposed` |
| concept | `draft/active/retired` | `draft` |

The vocabularies deliberately do not line up. A question cannot be `accepted`; a
decision cannot be `answered`; a concept cannot be `superseded`. `confirmed` on an
EVD is explicitly *non-terminal* ("it can be reopened or superseded by subsequent
evidence") ✓ both sources above — terminality is per value, not a property of the
word.

Doctrine also names a **terminal hide-set** per kind: `doctrine knowledge list`
hides it by default and `-a/--all` reveals it; `-s/--status <STATUS>` takes
multi-values and "any value reveals the hide-set" ✓ `knowledge list --help`. The
membership of each hide-set is **not observable** without a write or a corpus that
contains a terminal record (see § Limits).

**Transitions split into two verbs by what they must capture.** `doctrine
knowledge status <ID> <STATE>` sets an ordinary state; `doctrine knowledge settle
--by <BY> <ID> <STATE>` is "the resolving verb", and its `<STATE>` is restricted to
`answered`, `validated`, `invalidated`, `waived` — "Every other state stays with
`knowledge status`" ✓ `knowledge settle --help`. `settle` also takes "The date is
`today`, never taken from the caller" ✓ same. This is why the facet tables carry
`answered_by/answered_on`, `validated_by/…`, `invalidated_by/…`,
`waived_by/…` pairs — the disposition is a *facet write*, so status and its
justification cannot separate.

**Counter-example: one doctrine corpus derives status.** Observations are
append-only and never edited in place; `supersede` and `retract` are themselves
observation *records* (`--kind` accepts `friction, measurement, supersession,
retraction`), `observation show --raw` is documented as "the raw record (**no
resolution applied**)", and `list/search --history` include "all records including
inactive and controls" ✓ `doctrine observation * --help`, ✓
`lib:reference/using-doctrine.md` § Capturing friction. So doctrine runs *both*
models: stored-status for authored knowledge records, resolved/derived liveness for
the append-only observation log. **The brief's framing "doctrine derives status
rather than storing it" is true for observations and false for knowledge records.**
No observation corpus exists in this repo (`.doctrine/observations/` absent) so the
resolution step itself was not observed ✓ (`ls .doctrine/`, `grep -rl retract`
finds nothing outside the smut store).

Trust and severity are a third axis on memories, orthogonal to status:
`[trust].trust_level`, `[ranking].severity`, `[review].verification_state` ✓
`lib:templates/memory.toml`, ✓ `.doctrine/memory/shipped/*/memory.toml`. Status,
trust and verification are three fields because they answer three questions.

## 4. Evidence and verification

Three separate mechanisms, and the draft's one `scope` field sits across all of
them:

1. **`[evidence]` on every knowledge record** — `supports[]`, `contradicts[]`,
   `notes[]`, commented "free-text citations" ✓ `record-014.toml:22-25`. **Every
   `[evidence]` block in this corpus is empty** ✓
   `grep -A4 '^\[evidence\]' .doctrine/knowledge/*/*/record-*.toml` — declared,
   unexercised.
2. **Typed evidentiary edges** — the same two words reappear as a validated
   relation: "EVD and HYP records may be linked to other knowledge records via
   `supports` / `disputes` edges (`doctrine link EVD-001 supports DEC-002`)" ✓
   `lib:reference/glossary.md`, ✓ `mem.signpost.doctrine.knowledge`. Legal by
   `(source-kind, label)` pair, not by label alone (§ 5).
3. **Epistemic qualifiers as closed vocabularies in facets** — `confidence`
   (`low|medium|high`) on ASM and EVD, `basis` on ASM, `provenance`
   (`inspection|experiment|reproduction|citation`) on EVD, `source`
   (`canon|adr|external|technical|legal|compatibility|operator`) on CON ✓ § 2.
   Enforcement at write time is **unmarked**: `--confidence` documents "one of the
   confidence vocabulary; `""` clears", which would need a refused write to confirm.

Verification is a *separate field with its own verb*, on memories: `[review]
verification_state = "unverified"` in the shipped template ✓
`lib:templates/memory.toml`; `doctrine memory verify` "Attest a memory against the
current working tree. Stamps its verification axis (**refuses a dirty tree — no
false attestation**)" ✓ `memory verify --help`; `doctrine memory validate` checks
"dangling relations, stale verification, and draft expiry" ✓ `memory --help`. So
"is this still true?" is answered by a stamp tied to a git state
(`[git].verified_sha`, `base_commit`, `checkout_state_id`) ✓ `lib:templates/memory.toml`,
plus a `review_by` expiry ✓ `memory record --help --review-by`. Nothing in smut's
draft has an expiry concept.

Structured provenance exists where an entity can be named:
`--provenance-source KIND:REF` (repeatable) ✓ `memory record --help`;
`--repo` minting `kind: explicit, confidence: high` ✓ same.

## 5. Relations

- **Typed edges, not free text, for authored entities.** "The CLI is the validated
  seam — hand-editing relation rows skips the legality check"; "**do not**
  hand-author `[[relation]]` rows into a `.toml`" ✓ `lib:reference/using-doctrine.md`
  § Relating entities. The legal `(source, label) → target` set lives in a
  `RELATION_RULES` table that is *deliberately not transcribed* in the docs: "ask
  it rather than guess" ✓ same. Contents not observable (§ Limits).
- **Storage is outbound-only; reciprocity is derived.** `link` writes outbound,
  `inspect`/`show` render both directions ✓ same doc; ✓ `doctrine inspect DEC-014`
  prints `outbound: shapes: SL-002`; ✓ `doctrine show DEC-001` prints
  `shapes: [SL-001]`. The draft's "write one direction per relation, never both"
  is doctrine's *storage invariant*, not its advice.
- **Label vocabulary is small and mostly closed.** In this corpus
  `doctrine relation census` reports exactly three validated labels —
  `shapes` (19), `reviews` (9), `fulfils` (1) — plus 124 **raw** memory labels ✓
  `relation census`, ✓ `doctrine doctor` ("Raw Label: 124 memory edge(s) use raw
  labels (expected)"). `doctrine catalog scan` models this as
  `label: {Validated: "Shapes"} | {Raw: "…"}` and
  `target: {Resolved: "SL-002"} | {Unresolved: …}` ✓ — i.e. a validated enum with a
  named escape hatch, and target resolution reported rather than assumed.
- **Label sub-typing is by flag and gated.** `--role implements|originates_from|concerns`
  is "Required for `references`; refused for label-only labels";
  `--degree partial|full` "Only valid for the `fulfils` label; refused for all
  others"; `--descriptor` "Only valid on a `references --role concerns` edge;
  refused for all others. Non-empty" ✓ `doctrine link --help`. Free-text targets
  are legal only for the `drift` label ✓ same.
- **Not every relation is a `link` label.** `doctrine supersede` is its own verb ✓
  `supersede --help`; `doctrine needs` (hard prerequisite) and `doctrine after`
  (soft sequence) are their own verbs with their own target gates ("source must be
  a dep/seq-authoring kind; target must be work-like") ✓ `mem.signpost.doctrine.relating-entities`.
- **Reverse traversal is a first-class read**: `doctrine memory backlinks`,
  `doctrine blockers <ID> --transitive`, `doctrine graph <FOCUS> --depth/--label` ✓
  each `--help`. Someone asking "which answers point at this question" is an
  expected query, not a script.
- **The free-text second register persists.** `[evidence].supports` and
  `[evidence].contradicts` are arrays of *citations* (strings) on the same record
  that can also carry a typed `supports` edge. Doctrine tolerates the duplication
  because one is prose-grade and the other resolvable.

## 6. Identity and citation

- Identity is the **prefixed, zero-padded id**; "the id is identity; the slug is
  never authoritative" ✓ `lib:reference/glossary.md` § reference forms, ✓
  `lib:reference/using-doctrine.md` § Edit-preserving rules. `doctrine show <ID>` —
  "the prefix selects the kind", so no verb restates the kind ✓ `show --help`.
- **The kind lives inside the id.** An id is `ASM-007`, not `007` with a type
  field: `doctrine knowledge edit --help` documents its `<ID>` as "`ASM-007`,
  `DEC-012`, `CPT-001`" ✓ `knowledge edit --help`.
- Slug is a filesystem alias only: `.doctrine/knowledge/decision/014-<slug> -> 014`
  is a symlink ✓ `ls -ld .doctrine/knowledge/decision/014*`; the glossary documents
  the same pattern for ADRs (`nnn-slug` symlink alias) ✓.
- **Document-local enumerations are a separate, artefact-scoped namespace** —
  `OQ-1`, `D1`, `R1`, `Q1`, `C1`, bare (prefix + integer, no zero-pad), and "A
  doc-local id means nothing outside the artefact that holds it", so first use must
  be qualified by the artefact's durable id ✓ `lib:reference/glossary.md` § reference
  forms. Criterion ids (`EN-1`, `VT-1`, `VA-1`, `VH-1`) are immutable and bare ✓ same.
- Ids are immutable, `reseat` exists only as a repair verb ("Renumber an entity's
  canonical id … never a bare id") ✓ `reseat --help`; immutability is stated as a
  rule ✓ `lib:reference/glossary.md`, asserted by doctrine's own memory ✓
  `mem.pattern.doctrine.conventions`.
- Library citations are a different form again: `lib:<address>`, no anchors, valid
  only if `doctrine library tree` lists it ✓ `lib:reference/using-doctrine.md`
  § Publication. Three reference grammars — entity id, doc-local id, library
  address — each with its own resolution rule.

## 7. Supersession versus deletion

- **There is no delete verb.** `doctrine --help` offers no `delete`/`remove` for
  knowledge records; `doctrine knowledge --help` is
  `new, list, show, inspect, edit, status, settle, paths` ✓. The only removal verbs
  are `unlink` (edge), `tag clear` (tags), and `estimate clear` — nothing removes a
  record.
- **Supersession names both ends.** `doctrine supersede <NEW> <OLD>` ✓
  `supersede --help`; memory refuses the state without a successor —
  "`--by <OTHER>` is required for superseded (records the successor relation) and
  forbidden otherwise" ✓ `memory status --help`, and `memory edit --status` "refuses
  superseded — use `memory status superseded --by`" ✓ `memory --help`.
- **The terminal status is derived from the kind, not chosen by the caller.**
  "It covers ADR, POL, STD, ASM, QUE, DEC, CON and EVD; the old record flips to
  `superseded`, except an ASM/QUE, which flips to `obsolete`" ✓
  `mem.signpost.doctrine.relating-entities`, corroborated by "which terminal status
  the superseded record takes is decided from its kind, so ask the CLI rather than
  hand-editing a `supersedes` row" ✓ `lib:reference/using-doctrine.md`. Unmarked: no
  record in this corpus is superseded, so the flip was not observed.
- `supersede` is idempotent, and supersession is a chain on the record: both
  `supersedes` (new side) and `superseded_by` (old side) are verb-written ✓
  `record-014.toml:27-29`, ✓ `mem.signpost.doctrine.relating-entities`
  ("Idempotent").
- Observations go further: correction is another record with a `--reason`, and the
  original stays readable with `--raw` ✓ § 3.

## 8. Where doctrine refuses

Refusal is a designed surface: several flags exist *only* to refuse.

| refusal | verb | evidence |
| --- | --- | --- |
| a concept record has facet fields | `knowledge edit concept` | subverb description is `Refuse: concept records carry no facet fields` ✓ |
| an illegal `(source,label)→target` pair | `link` | "`link` rejects an illegal pair" ✓ doc; `RELATION_RULES` contents unobservable |
| `--role` on a label-only label; `--degree` off `fulfils`; `--descriptor` off `references --role concerns` | `link` | ✓ `link --help` |
| `superseded` without a successor; `--by` on any other state | `memory status` | "required for superseded … forbidden otherwise" ✓ |
| settling to a non-resolving state | `knowledge settle` | "Every other state stays with `knowledge status`" ✓ |
| a caller-supplied date for a disposition | `knowledge settle` | "The date is `today`, never taken from the caller" ✓ |
| an inline body on a new memory | `memory record` | `--body-mode` is "Not valid on `record` … Present (not omitted) so the refusal is explicit, worded, and clap-visible" ✓ |
| capturing an observation from a throwaway worktree | observation capture | "Doctrine refuses it rather than accept it, and names the broker to use" ✓ doc |
| a low-trust high-severity memory reaching agent context | `memory retrieve` | "Applies the trust holdback (non-bypassable)" ✓ `memory --help` |
| an attestation on a dirty tree | `memory verify` | "refuses a dirty tree — no false attestation" ✓ |

Two structural refusals matter more than any flag:

- **Prose cannot carry queried data.** A body is not a place for status, counts or
  edge lists ✓ § Storage tiers. This is what makes the facet tier trustworthy.
- **The engine refuses to invent.** `doctrine doctor` / `validate` report; they do
  not rewrite. `validate` scans ids only and printed `corpus clean` ✓ — it does
  **not** validate facets or status values, so the schema's authority is the verbs,
  not a validator.

## 9. Comparison with the smut `epistemic-records` draft (design.md § 9.2)

Draft elements read from `/workspace/smut/.doctrine/slice/002/design.md:550-589`.
Real-store counts from `target/release/smut query` over `interview.local/.smut`
(66 entities).

| draft element | doctrine equivalent | same / different / absent | consequence for approving the draft |
| --- | --- | --- | --- |
| 13 `type` values on one entity kind | 24 entity kinds, 7 knowledge kinds; kind is directory + prefix + verb family, encoded **in the id** | different | A type list is the only mechanism smut has; doctrine's equivalent is free because storage carries it. Nothing in doctrine argues for 13 values over 8 — the list is smut's, and needs its own evidence. |
| `session`, `cluster`, `index`, `claim`, `question`, `answer`, `finding`, `observation`, `proposal`, `criterion`, `convention` | nearest: session/cluster/index/claim → slice/spec/spec; question→QUE; answer→QUE.answer facet; finding→EVD/memory; observation→observation corpus; proposal→RFC/backlog.idea; criterion→`EN/EX/VT/VA/VH` ids; convention→policy/standard | different | 11 of 13 match types in live use (see next row); the mappings are one-to-many, so they justify the *shape* (a domain type list) and nothing about the values. |
| `source` | no source entity kind; `provenance` vocabulary + `--provenance-source KIND:REF` + `[evidence]` free-text citations | different | Doctrine never mints an entity to hold a citation. `source` as a type forces one entity per cited thing; the `cites` edge then has nowhere else to point. |
| `refutation` | HYP status `refuted`; EVD `disputes` edge; EVD status `disputed` | different | Doctrine treats refutation as a status/edge pair on an existing record, not a record type. A `refutation` type plus a `refutes` edge states the same fact twice. |
| 8 shared `status` values | 7 per-kind vocabularies, disjoint, with per-value terminality and a hidden terminal set | different | Doctrine's model says a *session* cannot be `retracted` and a *cluster* has no lifecycle at all. One shared list makes every combination expressible and none meaningful; the cost lands on the reader. |
| `open`, `answered` | QUE `open/answered/obsolete` | same | Exact match for the one kind that has it. |
| `settled`, `recorded`, `current`, `active`, `superseded` | answers/sessions settle; `active` on CON/CPT; `superseded` on DEC/CON/EVD | different | Doctrine spreads these across kinds rather than one enum. In the live smut store they are used on 5 types with no per-type rule — the drift doctrine prevents by typing. |
| `retracted` | EVD `retracted`; memory `retracted`; observation `retract` verb | same label, different home | Zero `retracted` **status** values in the live store. design.md § 2's "a retracted finding" is `finding-smut-16` by count (the only retired finding), and its stored status is `superseded`; the retraction survives only as prose in `vocab`. The value ships without an instance. |
| `part-of` | no containment edge — doctrine expresses containment through directory layout and namespace (`REQ` members, `descends_from`, `parent`) | absent in doctrine | Doctrine's containment is storage. smut cannot model containment (constraint), so the label is the only option — but the draft should say it is *the reader's* tree, not the store's. |
| `about` | no equivalent | absent in doctrine | Doctrine never needs it: every entity already sits in a directory and a namespace. Cheapest label in the store (48 entities, 84 refs) and the one that carries the most meaning per character. |
| `answers` (answer → question, "the only edge pointing upward against `part-of`") | question's own `answer`/`answered_by`/`answered_on` facets | different | Doctrine stores the answer *in* the question. In the live store `q-schema` is `answered` **and** `a-schema-resolved` carries `answers → q-schema`: one fact, two encodings, agreeing today and kept in step by nothing. |
| `supersedes` (new → old; old keeps `superseded`) | `doctrine supersede NEW OLD`, both sides written, terminal status kind-derived, successor **required** | same direction, weaker enforcement | The draft's direction matches. What it lacks is the requirement that a successor be named and the successor edge on the old record; without it `superseded` is a status with no pointer. |
| `refutes` (refutation → claim) | `disputes` (EVD/HYP → knowledge record), legal only from two kinds | same intent, different label and gating | Doctrine gates the pair by source kind. The draft's ungated label lets any type refute any type. |
| `cites` (claim/finding → source) | `references --role concerns\|implements\|originates_from` + `lib:<address>` citations + `[evidence]` free text | different | Doctrine has three citation registers so that a bare URL or file path needs no entity. smut has one: every citation must be an entity id, so `cites` + a `source` type is one mechanism doing what doctrine does with three. |
| `scope` (`doctrine/smut/operator/external`) | CON `source` vocab `canon/adr/external/technical/legal/compatibility/operator`; `decided_by`; EVD `provenance`; memory `[scope]` = paths/globs/commands/repo | different | Doctrine's `scope` on memories means *where it applies*, not *whose claim it is*. The draft's four values collapse doctrine's `source` (origin), `decided_by` (author) and `provenance` (method) into one field. `doctrine`/`smut` are project names, not claim-authorship categories, and will date. |
| "a retracted record is kept with its retraction intact rather than deleted; `superseded` is the status for that" | no delete verb exists; supersede is a verb; observations are append-only | same discipline, doctrine enforces by *not shipping the verb* | smut **does** ship `delete` ("Remove entities in one batch; incoming references are left untouched") ✓ `smut --help`. The advice is guarding a real affordance, so it earns its place — but the draft conflates retraction and supersession in one sentence: it names `superseded` as the status for a *retraction*. |
| "write one direction per relation; never write both" | outbound-only storage, reciprocity derived | same | Doctrine enforces by having no inbound field; smut cannot, so this is advice that a `backlinks` query silently punishes when broken. Keep. |
| advice only — printed, never validated, never auto-selected | doctrine *does* validate: `link` rejects illegal pairs, `settle` restricts states, `supersede` requires a successor | absent in draft by design (DEC-017) | The draft is strictly weaker than doctrine on every enforcement axis. That is the deliberate trade; it means the convention's only teeth are the reader's. |
| — | kind-specific facet tables (context/choice/rationale/basis/confidence/…) | absent in draft | The largest omission. Without facets a convention cannot say what a `claim` needs beyond a body; the draft says "state whose claim a claim is" and stops. |
| — | disposition pairs (`decided_on`, `answered_by`, `invalidated_on`, `waived_by`) | absent in draft | A status change in smut records neither who nor when. A reader cannot tell a settled claim from a stale one. |
| — | `[evidence]` free-text citation register | absent in draft | Every citation in smut must be an entity. |
| — | verification/expiry axis (`verification_state`, `review_by`) | absent in draft | Nothing in the draft can say a `finding` has expired. |
| — | per-kind terminal set, hidden from default listings | absent in draft | The draft has no notion of a status being terminal, so `where` cannot express "still live". |

**Fields the draft leaves out that doctrine carries, in one list:** facet tables
per kind; `decided_by`/`decided_on` and the other disposition pairs; a free-text
citation register beside the typed edges; a verification/expiry field;
per-value terminality; and a successor pointer written on the superseded record.

**Fields the draft asserts that doctrine does not have:** a `type` field on a
record (kind is storage); a shared status enum; a single `scope` enum; `part-of`;
`about`; `answers` as an edge (doctrine stores the answer in the question);
`refutation` as a type; `source` as a type; and `cites` as the only way to cite.

**Live-store counts that bear on the values** ✓ `target/release/smut query` plus
`jq` over `interview.local/.smut`, and `smut get vocab`:

- types in use (15): `index, session, cluster, convention, question, answer,
  claim, finding, observation, model, option, proposal, project, theme, criterion`
  — the draft's list shares 11, adds `source`/`refutation` (0 uses), and omits
  `model`/`option`/`project`/`theme` (6 entities between them).
- statuses in use (7): `open, answered, settled, recorded, active, current,
  superseded`. `retracted` never; `superseded` once. 38 of 66 entities carry no
  status at all.
- labels in use (4): `part-of` (65 entities / 65 refs), `about` (48 / 84),
  `answers` (5 / 7), `projects` (1 / 1). 157 refs total under 4 labels.
  The store's own `vocab` entity retired `theme`, `subject`, `links` into `about`
  and notes `projects` is defined and unused.
- the store already maintains its own convention record and **amended it by hand**
  when `superseded` was introduced ✓ `interview.local/.smut` → `vocab`
  ("Amendment, second session"). The draft's fixed-id seed formalises exactly this
  practice, and the practice predates the decision.

## 10. Limits of an opaque binary

Not claimed, because not observable read-only:

- **The canonical schema, as a type definition.** `doctrine design contract
  --format prompt` prints the `design apply` payload contract, and embedded in it
  is `extern knowledge::RecordKind` with every kind's facet keys, required-ness
  and closed value sets ✓ (orchestrator ran it this session). It is not a
  knowledge-specific verb and it does not print the status vocabularies or the
  relation legality table, so the *complete* schema is still only reconstructible
  from templates + `edit <kind> --help` + `knowledge status`.
- **`RELATION_RULES` contents** — which `(source, label) → target` triples are
  legal, and which labels are link-writable at all. The docs say to ask the CLI;
  the CLI does not print the table.
- **Write-time enforcement of facet vocabularies** (`confidence`, `basis`,
  `provenance`, `source`) — the template comments define the sets; whether an
  out-of-set value is refused needs a write, so it is `unmarked`.
- **Per-kind terminal hide-sets** — the mechanism is documented and the flag
  exists; the membership of each set is not printed.
- **The supersede status flip** — no knowledge record in this corpus is
  `superseded`, so the ASM/QUE→`obsolete` rule is a document claim only.
- **Observation resolution** — `.doctrine/observations/` does not exist here, so
  derived liveness was read from help text, not from a record.
- **Non-decision knowledge kinds** — this corpus holds 19 DEC records
  (`doctrine knowledge list --json` → 19 rows; each is reachable twice through its
  `nnn-<slug>` symlink, hence 38 `.toml` files) and no ASM/QUE/CON/EVD/HYP/CPT
  records (`ls .doctrine/knowledge/` → `decision`, `question`; `question/` is
  empty). The ASM/QUE/CON/EVD/HYP/CPT facets are
  therefore template-and-help-only. `EVD`/`HYP` edge behaviour is unobserved.
- **`[evidence]` non-empty behaviour** — every block is empty.
- Whether doctrine has any *field-predicate query*: none is documented and none
  appeared in `doctrine --help`; the closest are `knowledge list -s/--status`,
  `--regexp`, `search`, and the per-kind reports. That absence is `unmarked` as a
  negative claim — I did not enumerate every verb's flags.
