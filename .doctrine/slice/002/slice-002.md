# smut 0.2 — graph release: predicates and traversal

## Context

`smut` 0.1 (SL-001, closed) delivers the notebook round trip: declare, patch,
get, query, Markdown render and an embedded guide. It answers "read these named
entities back"; it cannot ask a question *across* the graph.

The authoritative statement of intent remains the repository-root brief
`smut-implementation-brief.org` (v1, dated 2026-11-01). This slice delivers the
brief's **Query contract** — the whole of it — which 0.1 deliberately narrowed
to `from`, `select`, `order` and `limit` (DEC-011). Where the brief and any other
document disagree, the brief wins; a divergence is recorded, never silently
resolved.

The charter is `IMP-001` (`doctrine show IMP-001`). Its *Trigger met* section is
the design input: the operator's first real 0.1 use, kept as a smut store at
`interview.local/.smut` (66 entities), settles what 0.2 must answer and what it
must not build. The release sequence is unchanged:

| Release | Capability | Where |
| --- | --- | --- |
| 0.1 (shipped) | Write linked thoughts, read them back as Markdown, never lose a batch | SL-001 |
| **0.2 (this slice)** | Ask questions across the graph — predicates, traversal and their provenance | SL-002 |
| 1.0 | Present the graph (Mermaid) and document the limits | IMP-002 |

`smut` remains a standalone complement to doctrine, not a replacement. It owns
identity, persistence, retrieval and presentation; domain structure and meaning
stay with the agent. `criterion-scope-smut` excludes a validating schema, modelled
containment and an audit mechanism *by intent*, not by deferral.

## Scope & Objectives

Deliver a working `smut` 0.2 binary that implements the brief's query contract in
full, consumes 0.1's mechanisms rather than restating them, and answers the
questions the first-use store actually asked.

1. **Predicate language** — `where` over roots only, exactly as the brief
   defines it: `$eq`, `$ne`, `$in`, `$exists`, `$contains`; `$and`, `$or`, `$not`
   with their empty-array identities; field conditions and boolean clauses
   ANDed; the operator-vs-literal disambiguation rule (`$eq` to compare a
   literal object containing operator keys); multiple field operators ANDed;
   missing-field semantics (every operator fails except the appropriate
   `$exists`); no coercion, regex, JSONPath or nested-field syntax. Unknown
   operators and mixed literal/operator objects are errors.
2. **Traversal** — `follow` and `backlinks`, each an array of top-level field
   names or `"*"`, both directions simultaneously allowed, `depth` (nonnegative,
   default 1, zero = roots only) and `budget` (positive, default 10000), applying
   the same rules at every hop. Cycles terminate; each entity appears once.
   Incoming and outgoing edges are labelled by the originating top-level field.
3. **Reference identity is single-sourced** — the walker consumes
   `src/entity.rs`'s reference predicate and scan (`reference_id` /
   `scan_refs`), not a second interpretation. The label filter is applied to
   that scan's `(label, target)` pairs.
4. **Determinism** — the brief's six rules are test targets: outgoing before
   incoming; named fields in query order; wildcard fields and embedded object
   keys lexical; array items in order; incoming sources in ID order with stable
   field ordering; identical results regardless of filesystem iteration order.
   `src/store/index.rs` iteration order is normalised, not trusted.
5. **Traversal provenance** — the result carries what the walk knew, in one
   conditional top-level `traversal` member (DEC-012): `depths` (returned id to
   minimum discovery depth), `edges` (induced among returned objects, deduped,
   discovery order, DEC-014) and `frontier` (discovered but never expanded, DEC-013).
   The member is present exactly when `follow` or `backlinks` is supplied, so a
   non-traversing 0.1 envelope is unchanged. The frontier is the information that
   exists nowhere else once the process exits; `truncated: true` alone says
   something was cut and not where.
6. **Truncation and diagnostics** — `traversal_budget` joins 0.1's
   `result_limit`, with the brief's ordering: budget counts entity expansions and
   stops the walk; limit caps ordered output; a depth boundary is selection, not
   truncation (DEC-015). `dangling` reports references encountered during
   requested traversal (deduplicated, `{"source","field","target"}`), never a
   store-wide integrity audit.
7. **Grammar widening** — the query spec accepts `where`, `follow`, `backlinks`,
   `depth`, `budget`; malformed operators and bad types reject the whole request.
   Rejection remains syntax-additive: a working 0.1 call cannot become invalid
   when the key set widens.
8. **Conventions library** — a separate read-only command
   `smut conventions [NAME]`, no store required, prose by default and one
   pipeable `--format json` declaration on request, never validated and never
   auto-selected. Exactly two sets ship (DEC-017): `hygiene` and
   `epistemic-records`, the only domain with evidence. Each JSON seed carries a
   fixed id and the release version, so re-installing updates that entity and the
   store owns its copy afterwards. `learn` gains a pointer line and keeps its
   single, descriptive, version-bound document.
9. **The guide carries the idioms** — `learn` is the normative home of the
   two-call composition (DEC-016): traverse for ids with `select:["id"]`, then
   filter those ids as roots with `where`. It also teaches the label vocabulary
   and bounded rendering. Conventions state the principle and point at `learn`
   rather than repeating the recipe. A behaviour without its sentence in the
   guide is an unfinished fix (`finding-smut-17`), even when the behaviour needs
   no code.
10. **Acceptance fixture** — the first-use store is the acceptance graph: real,
    small enough to read whole, and already containing the awkward cases (a
    superseded reference, a retracted claim, mutually-referencing clusters) and
    22 entities with no incoming reference, so `backlinks` has a hit and a miss
    without inventing either. It is promoted as one normalised entity array plus
    a provenance note, declared into a scratch store by the tests; `.smut`
    internals are not committed (DEC-018).
11. **Documentation and tests** — README query examples extended to the new
    surface; the store format document unchanged unless a decision changes it;
    focused automated tests for predicates, traversal, determinism, truncation,
    cycles, dangling references and hidden reference fields; the 0.1
    compatibility suite still green; `cargo fmt` and clippy clean.

### Work order (from `finding-smut-19`)

The label filter was never the cost. Sequence the phases by risk:

1. walker and labels, consuming the existing reference definition;
2. depth, budget, truncation and the envelope reasons;
3. predicates, which are independent of traversal;
4. conventions and the guide, which depend on the settled label vocabulary.

## Deferred / out of scope

- **IMP-003** — the cheap 0.1.x fixes from first use: refuse the literal `DROP`
  rather than eating it, `created`/`updated` fields, and the legibility triple
  (id-named files, references rendered as ids/links, fenced stored headings).
  A separate slice; not absorbed here.
- **IMP-002** — the 1.0 presentation release: Mermaid renderer, heading-flag
  interactions with non-Markdown formats, portability limits, generated-store
  smoke benchmark. The edge set is returned here so that renderer can project
  it, but the renderer is not built here.
- **`where` beyond roots.** It stays roots-only. The two-call composition is the
  idiomatic answer, not a defect to fix.
- **A reverse index for `backlinks`.** It remains a whole-store scan per hop, and
  the traversal budget does not bound it; the documentation says so instead of
  implying a hard total-work bound.
- **Nested-field predicates, query pipelines, pagination, secondary indexes,
  first-class edge types, schema validation, modelled containment, an audit
  mechanism inside the store** — all stay out, the last three by intent.
- **A conventions schema, checker or auto-selector** — the moment a set is
  checkable it is a schema with a friendlier name.
- **A storage-format change.** 0.2 widens the query surface, not the on-disk
  format; `format_version` stays 1 unless design records a reason.

## Affected surface

Coarse fence; the exact touch-set is the design's job.

| Area | Paths | Nature |
| --- | --- | --- |
| Query grammar | `src/spec.rs` | five new keys, predicate parsing, `depth`/`budget` types |
| Selection pipeline | `src/select.rs`, `src/order.rs` | roots → traverse → dedupe → order → limit → project |
| Reference identity | `src/entity.rs` | consume `reference_id`/`scan_refs`; no second definition |
| New subsystems | `src/walk.rs`, `src/predicate.rs` (names indicative) | walker, frontier, edges, depth; predicate evaluation |
| Conventions | `src/conventions.rs`, `src/conventions/*.md`, `src/learn.rs`, `src/cli.rs` | new command, embedded sets, guide pointer |
| Envelope and diagnostics | `src/select.rs`, `src/render_markdown.rs`, `src/main.rs` | edges/frontier/depth, `traversal_budget`, `dangling`, Markdown disclosure |
| Rejections | `src/error.rs` | malformed predicates, bad types, budget/depth |
| Tests | `tests/**`, `tests/fixtures/**` | predicates, traversal, determinism, conventions, fixture acceptance |
| Docs | `README.md`, `docs/**` | query examples, backlink cost, conventions |

## Risks and assumptions

| Risk | Mitigation |
| --- | --- |
| A second definition of "reference" drifts from declaration's, silently losing edges | Walker consumes `src/entity.rs`; a test asserts the walker and `declare` agree on a nested/embedded reference |
| Six determinism rules, with `src/store/index.rs` iteration order the weak point | Normalise iteration at the boundary; assert byte-identical results across shuffled index orders |
| `backlinks` is O(store) per hop and unbounded by `budget` | State the cost in the guide and README; do not imply a total-work bound |
| Envelope widening breaks 0.1 clients | New members are additive; 0.1 calls keep their shape; `tests/compatibility.rs` pins representative invocations |
| Predicates grow into a query language | Brief's closed operator list; no nested paths, pipelines or pagination |
| Conventions content is unendorsed | Shape is endorsed, contents are drafts; keep hygiene + evidenced epistemic set, ship few |
| The fixture is a real personal record | Promotion is decided in design with provenance; it is the owner's own store |

Assumptions carried from 0.1: single user, single host, local filesystem with
working `flock`/`fsync`; the store fits comfortably in memory, so whole-store
scans for filtering and backlinks are acceptable; stored content is data, not
instruction.

## Settled by inquiry

The exploring-boundary map (inq-1…inq-8) is disposed; the answers are durable
knowledge records, not scope prose.

| Decision | Settles |
| --- | --- |
| DEC-012 | Provenance is one conditional `traversal` member: `depths`, `edges`, `frontier` |
| DEC-013 | Frontier = discovered but never expanded; depth and budget cut it, limit does not |
| DEC-014 | Edges are induced among returned objects, deduped, discovery order, dangling excluded |
| DEC-015 | Budget counts entity expansions; `traversal_budget` only on a real decline |
| DEC-016 | The two-call idiom is normative in `learn`; conventions carry the principle |
| DEC-017 | Two convention sets, fixed-id versioned seeds, the store owns its copy |
| DEC-018 | First-use store ships as a normalised array plus provenance, not `.smut` internals |
| DEC-019 | Grammar and envelope stay additive; compatibility tests are re-cut, not deleted |

Predicates, label semantics and determinism were never open: the brief pins
them and they are acceptance criteria.

## Verification and closure intent

"Done" is judged by:

1. the brief's mandated coverage for this release — missing/null predicate
   semantics, boolean composition, invalid queries, cycles, backlinks, depth,
   budget and result truncation, hidden reference fields — passing;
2. the six determinism rules asserted as behaviour, including a shuffled
   filesystem-order case;
3. the promoted first-use fixture, declared into a scratch store, answering the
   first-use questions in two calls (traverse for ids, filter those ids as roots)
   with correct `dangling`, provenance and truncation disclosure;
4. the 0.1 compatibility suite unchanged and green, plus `cargo fmt` and clippy
   with zero warnings;
5. `smut learn` and the README carrying the idioms the release introduces, and
   `smut conventions` printing its sets without a store and `--format json`
   round-tripping through `declare`;
6. the slice's selectors diffed against git actuals (`doctrine slice
   conformance`) with each non-conformant cell explained.

## Summary

0.2 turns the notebook into a graph a reader can interrogate: a closed predicate
language over roots, bounded and deterministic traversal in both directions, and
a result that carries its own provenance under one conditional `traversal`
member — induced edges, the unexpanded frontier and minimum depth — so a client
does not have to reconstruct what the walk knew. It adds the conventions command
as advice, never as a schema, and puts the idioms in the guide where a future
agent will find them. Every 0.1 call keeps its response.
