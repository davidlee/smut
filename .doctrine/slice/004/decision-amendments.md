# SL-004 — the six-decision amendment proposal (PHASE-03/EX-1, VA-1)

**Status: proposal only. No decision record has been edited.** This is the mapping
`/audit`'s reconciliation brief consumes; `/reconcile` is the sole later writer of
DEC-002, DEC-005, DEC-007, DEC-010, DEC-012 and DEC-016 (DEC-025, design §4
*Accepted decision amendments*).

Every "current" passage below was read from the record with `doctrine show <id>`
on 2026-11-02 against commit `963a788` — facet and prose tier both — not from the
design table, a review report or an earlier paraphrase. Quotation is verbatim
except where an ellipsis is marked. Line/section anchors name the prose heading the
passage lives under, because record bodies are prose, not line-numbered artefact
files.

**Mechanism.** Facet fields and prose are both changed through the knowledge
editing surface (`doctrine knowledge edit` / the skill's editing verbs), never by
hand-editing `record-0NN.toml`. The facet and the prose of one record land in the
same act, so an updated body cannot coexist with a contradictory facet choice
(design §4, last paragraph).

**Amendment note** to attach to each of the six records — same text shape, dated
and citing both authorities, as EX-1 requires:

> Amended 2026-11-02 by SL-004 under ADR-001 (identity is `smut_id`, a reserved key
> in a permissive store) and this slice's DEC-025: structural identity is spelled
> `smut_id`; an ordinary `id` is caller data at every depth. Only the passages
> listed in SL-004 PHASE-03's amendment proposal changed; the rationale, durable
> ids and unrelated consequences are retained verbatim.

Two records additionally need a sentence of their own in the note body (DEC-005's
reversal, DEC-010's version figure); those are given per record below.

---

## DEC-002 — declaration equality, per-field conflict, canonical key order

**Facet: no identity spelling occurs in any facet field; no facet change.** Verified
field by field: `context` speaks of "equality … coalescing and conflict inside a
declaration batch, and how it treats key order, null, and numbers"; `choice` of the
Set/Drop algebra, structural equality, `1 != 1.0` and `BTreeMap`-backed objects;
`alternatives` of numeric cross-type equality, `preserve_order` and entity-level
conflict; `consequences` of `1` vs `1.0`, lexical key order, the `DROP` sentinel and
batch rejection; `rationale`/`decided_by`/`decided_on`/`shapes` untouched. None
names the wire key. Recording "no facet change" rather than inventing one is the
point of EX-1's per-record mapping.

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *The operation algebra*, coalescing example | `[{"id":"q","refs":[{"id":"t","label":"x"}]}]` and `[{"id":"q","refs":[{"id":"t"}]}, {"id":"t","label":"x"}]` | both lines with `"smut_id"` in place of `"id"` — the example's whole point is that normalisation replaces the nested declaration with an identity-only reference; a key that is now data would make the example assert the opposite |
| *Consequences pinned by the brief* | "Explicit id plus only `Drop` fields, on an absent entity → **created**, id-only. A bare id-only object never creates anything." | "Explicit `smut_id` plus only `Drop` fields, on an absent entity → **created**, identity-only. A bare `smut_id`-only object never creates anything." (identity sense; `ids`/`changes` container names retained) |
| *Consequences pinned by the brief* | "`previous` is captured … the full entity for `updated`" | unchanged — no key spelling; retained |
| *Consequences pinned by the brief* | "`missing` is computed against the **post-batch** id set" | retained — `id` here is internal vocabulary for opaque id *values*, not the JSON member (design §4: "Internal `Id`, `Change.id`, map-key and basename variable names need not become `smut_id`") |
| *Consequences pinned by the brief* | "`ids` echoes the top-level input order including repeats" | retained — container name, not the member |

**Unaffected rationale, retained explicitly.** Set/Drop algebra and its fold table;
structural `serde_json::Value` equality with no coercion; `1` distinct from `1.0`
and its rejection consequence; `BTreeMap` object ordering and the lexical
first-encounter consequence; the `DROP` sentinel boundary and "cannot be assigned";
`null`-versus-`Drop`; batch rejection being total; `unchanged` meaning; `missing`
uniqueness and encounter order; both rejected alternatives.

**Amendment-note addendum:** none beyond the common note.

---

## DEC-005 — references: one predicate, hoisting with label, preorder, error classes

**This one is a reversal, not a respelling.** The current record says of a
`Value::Object` with a member `id`: "Presence is the entire test"; and in its own
*Documented consequence* section: "An embedded object carrying `id` cannot be inline
metadata in 0.1. Stated in the README and in `learn`: it is the one convention a new
user will trip over." After SL-004 that proposition is false: `id` is ordinary data
and an embedded object carrying `id` **can** be inline metadata. The amendment must
free the ordinary `id` explicitly, not respell the examples and leave the
consequence sentence teaching the reversed rule.

**Facet changes.**

| Field | Current (verbatim) | Proposed |
| --- | --- | --- |
| `context` | "The brief makes any embedded object with an id an entity node, which both declaration and 0.2's traversal must obey." | "The brief makes any embedded object with a `smut_id` an entity node, which both declaration and 0.2's traversal must obey. An ordinary `id` is data at every depth (SL-004, ADR-001)." |
| `choice` | "labels are the containing entitys non-id field name after hoisting; request-time id errors are exit 2 while stored-data id errors are corruption exit 1" | "labels are the containing entity's non-`smut_id` field name after hoisting; request-time identity errors are exit 2 while stored-data identity errors are corruption exit 1, enforced eagerly at load (DEC-021)" |
| `consequences` | "An embedded object carrying id cannot be inline metadata in 0.1 (documented in README and learn); scan_refs must never be called on raw request JSON; declaration order is parent-before-child regardless of finishing order." | "**An embedded object carrying `smut_id` cannot be inline metadata — that spelling is structural at every depth — while an ordinary `id` may be inline metadata anywhere, because it carries no identity meaning.** The old restriction applied to `id` and is withdrawn (SL-004, DEC-021); scan_refs must never be called on raw request JSON; declaration order is parent-before-child regardless of finishing order." |
| `rationale`, `alternatives` | "A single definition is what stops 0.2 from growing a second, divergent notion of a reference…" | retained verbatim; the arguments are about *one* predicate and *error-class split*, neither of which moves |

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *One definition, three surfaces*, code block | `fn reference_id(v: &Value) -> Result<Option<&str>, Error>;   // Ok(None) ⇒ not an entity node` | `fn reference_id(v: &Value) -> Result<Option<&str>, Reject>;   // Ok(None) ⇒ not an entity node` — **factual correction**: `src/entity.rs:32` returns `Reject`, and `Error` is not the type; the next line already spells `extract` as returning `Reject`, so the record was internally inconsistent |
| *One definition, three surfaces* | `fn scan_refs(entity: &Value) -> impl Iterator<Item = (&str /*label*/, &str /*target*/)>;` | retained — accurate as shipped (`src/entity.rs:72`), deliberately elided lifetimes |
| *What counts* | "A `Value::Object` with a member `id` whose value is a nonempty string. Presence is the entire test — field count and field names are irrelevant." | "A `Value::Object` with a member `smut_id` whose value is a nonempty string. Presence is the entire test — field count and field names are irrelevant. An ordinary `id` is never inspected." |
| *What counts* | "`id` present but not a nonempty string ⇒ rejection. `{"id": null}` is an error, not \"no id, please generate\"." | "`smut_id` present but not a nonempty string ⇒ rejection. `{"smut_id": null}` is an error, not \"no identity, please generate\"." |
| *What counts* | "Id generation applies **only to top-level declare objects that lack `id` entirely**. A nested object without `id` is an embedded value and is never given one." | same sentence with `smut_id` for the member (twice); "Id generation" may stay, or read "Identity generation" — a wording choice, not a rule change |
| *What counts* | "Ids are opaque, case-sensitive, unnormalised; a whitespace-only id is a valid nonempty id." | retained — about id *values*, and true after the rename |
| *Declaration vs reference* | "An object with `id` and nothing else is a pure reference. An object with `id` plus any other field is a declaration: hoisted to top level, its occurrence replaced with `{"id": …}`." | "An object with `smut_id` and nothing else is a pure reference. An object with `smut_id` plus any other field is a declaration: hoisted to top level, its occurrence replaced with `{"smut_id": …}`." |
| *Labels* | "`scan_refs` starts at each **non-`id` field** of the entity …" | "`scan_refs` starts at each **non-`smut_id` field** of the entity …" |
| *Labels*, examples | `{"id":"a","refs":[{"id":"b","note":{"id":"c"}}]}` twice, and the edges sentence "`a.refs → b` and `b.note → c`" | entity keys move to `smut_id`; the edge sentence is unchanged (labels and targets are values) |
| *Error classification*, table | rows "invalid `id` in a declaration (a request)" / "invalid `id` inside persisted store data" | "invalid `smut_id` in a declaration (a request)" / "invalid `smut_id` inside persisted store data"; the exit classes 2/1 are retained |
| *Missing references* | "Only references appearing **in the request** are resolved, against the **post-batch** id set." | retained — id *set*, internal vocabulary |
| *Documented consequence* | "An embedded object carrying `id` cannot be inline metadata in 0.1. Stated in the README and in `learn`: it is the one convention a new user will trip over." | **Replaced.** "An embedded object carrying `smut_id` cannot be inline metadata: the key is structural at every depth, so a payload that uses that spelling cannot be stored verbatim. The converse is now true and belongs beside it: an embedded object carrying `id` **is** ordinary data and may be inline metadata anywhere. Stated in `learn` (DEC-027); the README statement is the operator's 0.2 rewrite and lands outside this slice (CHR-001)." |
| *Alternatives considered* | "`scan_refs` returning `Result` and skipping malformed ids — rejected: silent skips are how references disappear; the precondition makes the failure impossible instead." | retained verbatim — the precondition now has an enforcement point (DEC-021), which strengthens rather than alters the argument |
| *Alternatives considered* | "One error type for request and stored data — rejected: conflates exit 2 with exit 1." | retained verbatim |

**Unaffected rationale, retained explicitly.** One definition with three surfaces;
preorder declaration recording and its parent-before-child consequence; label =
containing-entity field name after hoisting, and the rejection of the request path
as a label; the single recursive pass over finite JSON and the rejection of a
fixpoint loop; `scan_refs` non-fallible with a validated-entity precondition;
request/stored error-class split and the `Error`-type-must-not-dictate-classification
sentence; missing-reference resolution against the post-batch id set only, with
no store-wide integrity scan; all four rejected alternatives.

**Amendment-note addendum:** "This amendment withdraws the reservation that an
embedded object carrying `id` cannot be inline metadata; that restriction now
applies to `smut_id` only. The eager loader rule is DEC-021's."

---

## DEC-007 — projection and envelope, id-first canonical order

**Facet changes.**

| Field | Current (verbatim) | Proposed |
| --- | --- | --- |
| `choice` | "…through one shared Entity wrapper whose subject is the entity node (an object with a nonempty string `id`) wherever it appears — an entity node names `id` first with lexical remaining keys, every other object is lexical, arrays in order; pretty files and compact stdout." | "…whose subject is the entity node (an object with a nonempty string `smut_id`) wherever it appears — an entity node names `smut_id` first with lexical remaining keys, every other object is lexical, arrays in order; pretty files and compact stdout." The rest of `choice` (borrowing `Selection`, pure `project`/`envelope`, flat envelope) is retained |
| `consequences` | "Two byte forms are specified separately (pretty files, compact stdout); the select set never determines key order; renderers consume structured Selection with serialisation not between selection and rendering." | retained verbatim — none of it names the key |

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *Envelope* | "`id` in every object regardless of `select`; `select: []` yields `id` only." | "`smut_id` in every object regardless of `select`; `select: []` yields `smut_id` only." |
| *Envelope* | "`select` duplicates are tolerated as a set; naming `id` is a no-op; naming an unused field is not an error." | "…naming `smut_id` is a no-op; naming an unused field is not an error. Naming an ordinary `id` selects a data field like any other." |
| *Serialisation happens at the final boundary*, canonical ordering | "- entity root: `id` first, remaining keys lexical;" | "- entity root: `smut_id` first, remaining keys lexical;" |
| *Serialisation clarification* | "an object whose `id` is a nonempty string names `id` first *wherever it appears*" | "an object whose `smut_id` is a nonempty string names `smut_id` first *wherever it appears*" |

**Unaffected rationale, retained explicitly.** The borrowing `Selection` and its one
lifetime; `envelope` in the core so query/get tests need no filesystem; "the store
reference is not permission to widen the result"; flat `objects`, never inlining a
referenced entity; `missing`/`dangling`/`truncated`/`truncation_reasons` always
present; missing fields omitted rather than `null`; the two byte forms table;
"Never serialise and then re-parse"; the `Entity`-wrapper-is-proportionate scope
note; all four rejected alternatives; the `78a7eb1` history in the serialisation
clarification (it is a record of what happened, not a rule about the key) and the
`tests/cli_contract.rs::a_projected_entity_is_serialised_by_the_same_wrapper_as_a_stored_one`
pin, whose name does not name the key.

**Note.** The paragraph "**The store reference is not permission to widen the
result.** … 0.2's Mermaid output …" and "Rendering consumes structured `Selection`"
are retained; Mermaid ownership does not move (this slice does not authorize it).

---

## DEC-010 — documentation set: README, versioned store format, learn

**Facet changes.**

| Field | Current (verbatim) | Proposed |
| --- | --- | --- |
| `choice` | "docs/store-format.md versioned as format_version 1 covering layout, serialisation, journal states and fsync obligations, recovery, reserved names and the commit-before-acknowledgement ambiguity; learn limited to the 0.1 surface." | "`docs/store-format.md` versioned as format_version 2 covering layout, serialisation (structural identity `smut_id`, an ordinary `id` as data), journal states and fsync obligations, recovery, reserved names, the nested-identity corruption refusal and the commit-before-acknowledgement ambiguity; learn limited to the shipped surface." |
| `consequences` | "Unknown format_version is exit 1 with no migration; the README states Linux as the acceptance platform and flock as advisory." | "An unknown or older format_version is exit 1 with no migration, compatibility reader or repair path (DEC-022); the README states Linux as the acceptance platform and flock as advisory." |
| `context` | "SL-001 answer to inq-10: what is documented, and where each piece lives." | retained |
| `rationale` | "The guarantee has to be quotable verbatim to stay honest; a format doc that drifts from the protocol is worse than none." | retained verbatim — STD-001 is the same argument |

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *README.md*, sharp-edge list | "- an embedded object carrying `id` cannot be inline metadata;" | "- an embedded object carrying `smut_id` cannot be inline metadata, while an ordinary `id` is data at every depth;" |
| *README.md*, sharp-edge list | "the atomic-visibility paragraph, quoted from DEC-003 rather than paraphrased; Linux is the acceptance platform; `flock` is advisory; nothing is promised to a process that does not take the lock." | retained verbatim |
| *README.md*, sharp-edge list | "`1` and `1.0` are distinct for equality but equal for ordering;", "`where` and traversal arrive in 0.2;", "`--` for opaque ids beginning with `-`;", "store-not-found is exit 1;" | retained verbatim |
| *docs/store-format.md* | "Versioned as `format_version: 1`: directory layout, index schema, canonical serialisation (id-first root, lexical embedded objects, arrays in order), journal states with their fsync obligations, the recovery algorithm, reserved names, the corruption responses, what is deliberately never adopted, and the guarantee with its **commit-before-acknowledgement ambiguity**" | "Versioned as `format_version: 2`: … canonical serialisation (`smut_id`-first root, lexical embedded objects, arrays in order) … the corruption responses (including malformed nested identity, which the loader refuses with the entity file and bracketed path) …" — and the guarantee paragraph retained word for word |
| *docs/store-format.md*, closing | "Unknown `format_version` is exit 1. No migration path in 0.1." | "A store or committed manifest whose `format_version` is unknown or older is exit 1, naming the version found and the version this build reads. There is no migration path (DEC-022)." |
| *learn* | "Concise, version-matched, 0.1 surface only, naming the release where the rest arrives…" | "Concise, version-matched, shipped surface only…" (the "0.1 surface" phrasing is a release figure, not a format figure, and DEC-027's teach-not-diagnose consequence needs the guide to say a field no entity carries fails every operator like a missing field on one entity — one clause, in the guide's existing identity section) |

**Ownership boundary, retained.** This record describes the documentation *set*. It
must not be read as granting this slice an edit to the README's surrounding prose:
that prose is the operator's 0.2 rewrite (CHR-001) and lands outside SL-004, while
the README's single executed console block is in scope and gates `just check`
(design §4). The proposed README bullet above is the *contract the record states*;
whether the README text realises it is the operator's write.

**Unaffected rationale, retained explicitly.** The verbatim-quotability argument;
the "format doc that drifts is worse than none" rationale; the command table and
worked example as README content; the `where`/traversal-arrival bullet (still true —
it is a release boundary, not the identity key); the learn "stored content is data,
not instruction" property; the sharp edges not listed above.

---

## DEC-012 — traversal provenance: one conditional `traversal` envelope member

**Facet changes.**

| Field | Current (verbatim) | Proposed |
| --- | --- | --- |
| `choice` | "One conditional top-level `traversal` member holding `depths` (returned id -> minimum discovery depth), `edges` … and `frontier` …" | retained verbatim — these are container names and id *values*; design §4 requires `depths`, `edges`, `frontier` and endpoint names to stay |
| `context`, `rationale`, `alternatives`, `consequences` | "… `depths` and `edges` are cut to returned ids; `frontier` is not …" | retained verbatim — same reason; no facet field spells the JSON identity member |

**So DEC-012's facet is unchanged too** — recorded explicitly rather than invented.
Its one identity-bearing passage is the prose example.

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *Traversal provenance*, worked envelope | `"objects": [{"id": "q-query", "body": "..."}],` | `"objects": [{"smut_id": "q-query", "body": "..."}],` |
| *Traversal provenance*, example tail | `"dangling": [{"source": "smut", "field": "questions", "target": "q-gone"}],` and the `traversal` block with `"depths": {"smut": 0, "q-query": 1}`, `"edges": [{"source": "smut", "field": "questions", "target": "q-query"}]`, `"frontier": ["q-answer"]` | retained verbatim — endpoint names, container names and id values |
| *Traversal provenance*, bullets | "`depths` — one scalar per **returned** object: minimum discovery depth in the mixed graph (both directions) …"; "`edges` — the induced edge set among **returned** objects, deduplicated by `(source, field, target)`, in result order …"; "`frontier` — discovered but never expanded, in discovery order (DEC-013)." | retained verbatim |
| *Traversal provenance*, closing | "`depths` and `edges` describe the result and are cut to selected ids after the limit; `frontier` describes the walk and is not cut." | retained verbatim |

**Unaffected rationale, retained explicitly.** The whole record: conditional
presence iff `follow`/`backlinks` is supplied; the frontier as the only fact that
survives the process; edges as the single derivation; the 0.1-envelope compatibility
argument (DEC-019); the three rejected alternatives; the "scalar is not sugar over a
node list" argument; the `traversal` member's cut-to-returned-ids rule.

---

## DEC-016 — the two-call idiom is normative in `learn`

**Facet changes.**

| Field | Current (verbatim) | Proposed |
| --- | --- | --- |
| `choice` | "`learn` carries the recipe (traverse for ids with select:[id], then filter those ids as roots) plus the 0.2 surface." | "`learn` carries the recipe (traverse with `select:["smut_id"]`, then filter those returned ids as roots) plus the shipped surface." |
| `consequences` | "The 0.1 tests asserting 0.2 is unadvertised invert, A label vocabulary and bounded-render advice enter the guide, The guide stays one document, The recipe includes its completeness guard …, The recipe includes its direction …" | retained verbatim |
| `context`, `rationale`, `alternatives` | "One normative copy of a procedure avoids drift; learn is version-bound and guaranteed to match the release; conventions remain advisory." | retained verbatim |

**Prose changes.**

| § | Current (verbatim) | Proposed |
| --- | --- | --- |
| *The two-call idiom's home*, recipe block | `1. smut query '{"from":"session","backlinks":["part-of"],"depth":2,"select":["id"]}'` and `2. smut query '{"from":[<ids>],"where":{"status":"open"},"select":["title"]}'` | line 1 becomes `…"select":["smut_id"]}`; line 2 is retained unchanged (`from` takes id values; `select` names a data field) |
| *The two-call idiom's home* | "`select: [\"id\"]` keeps the first call cheap; the second is an ordinary root filter because `from` takes an explicit id list." | "`select: [\"smut_id\"]` keeps the first call cheap; the second is an ordinary root filter because `from` takes an explicit id list." |
| *The two-call idiom's home* | "The direction is the first thing a reader gets wrong … `follow:[\"part-of\"]` from a session returns the session and nothing else …" | retained verbatim |
| *The two-call idiom's home* | "What the composition does **not** give away is completeness …" (whole guard paragraph, both retry bounds, "Repetition alone never establishes completeness") | retained verbatim — the guard is the reason the record exists |
| *The two-call idiom's home*, closing | "The conventions library states the **principle** … It does not repeat the recipe." and "The guide also covers, in the same revision: … `traversal` provenance member …" | retained verbatim |
| *The two-call idiom's home* | "(The store this release is built from carries the inverted example in `finding-smut-18`; copying an example out of the store is how the error travels.)" | retained verbatim — historical quotation |

**Unaffected rationale, retained explicitly.** One normative copy; learn as
version-bound and guaranteed to match the release; conventions holding the principle
and not a second recipe; the `where` stays roots-only boundary; the completeness
guard and its two truncation reasons with their retry bounds; the direction rule and
the inverted-`finding-smut-18` illustration; the closing list of what the guide also
covers.

---

## Cross-record checks the audit should apply

1. **No record is superseded wholesale.** Every record retains its rule; six
   targeted passages moved. DEC-005 is the only one whose *rule* changes, and it
   changes by explicit withdrawal with a pointer to DEC-021.
2. **No durable id is renumbered.** `DEC-002/005/007/010/012/016` keep their ids and
   `shapes` (`SL-001`, `SL-002`).
3. **Container and value names stay.** `ids`, `changes`, `deleted`, `missing`,
   `frontier`, `depths`, `edges[].source/.target`, `dangling[].source/.target`,
   `Change.id`, `Id`, `known_entities` — nothing in any proposal renames one.
4. **Response containers unchanged.** DEC-007's envelope keys and DEC-012's
   `traversal` member are untouched; only member values that were entity nodes move.
5. **Five passages are factual corrections, not preferences:** DEC-005's
   `reference_id` return type (`Reject`, `src/entity.rs:32`), and the four
   descriptions of the `id`-cannot-be-metadata rule (DEC-005 ×2, DEC-010 ×1,
   DEC-002 example ×1 whose example would otherwise assert the opposite of the
   shipped behaviour).
6. **Facet/prose coherence.** DEC-002 and DEC-012 change no facet field; DEC-005,
   DEC-007, DEC-010 and DEC-016 change facet fields whose wording *did* name the
   key or the version. Each pair is landable together with no residue: after the
   amendment no facet field and no prose passage in any of the six records says an
   ordinary `id` carries identity, or that the format is version 1.
7. **The README claim DEC-005 reverses is outside every record's write scope.**
   `README.md`'s surrounding prose still teaches "An embedded object carrying `id`
   is a reference, not metadata… there is no way to store inline metadata under a
   field named `id`" (lines 247-250 of the operator's uncommitted 0.2 rewrite). This
   slice may not edit it (CHR-001); the finding is carried in `notes.md`
   § Harvest/Open and in § Implementation run → PHASE-03, for the operator and the
   audit.
