# Research — SL-002 smut 0.2 graph release

Pre-design research round for `SL-002` (smut 0.2 — predicates and traversal).
Artefact tier: runtime working evidence, gitignored in place; durable findings
are harvested at close.

- **Producers:** both threads run by the orchestrating agent (degraded mode — the
  project defines no research agents in `AGENTS.md`, project orientation or
  `.doctrine/doctrine.toml`). Read-only.
- **Baseline:** `.doctrine/slice/002/research/baseline.toml`, stamped by
  `doctrine slice research 2`.
- **Raw output:** `raw/thread-1-governance.md`, `raw/thread-2-code-map.md`.

## Verification legend

- ✓ — verified by the consuming agent in this session (a command run or a file
  read, cited inline).
- unmarked — researcher claim: cited, not independently checked. Design and plan
  may only lean on ✓ rows, or must verify them at point of use.

## Thread 1 — governance applicability

**Binding constraints**

| Source | How it binds | Status |
| --- | --- | --- |
| `smut-implementation-brief.org` §§103–163, 184–220 | The observable query contract: keys, predicates, traversal determinism, envelope, guide and test obligations. Brief wins any conflict. | ✓ read this session |
| `IMP-001` | The slice charter: what 0.2 delivers, the trigger evidence, and the explicit "do not build". | ✓ `doctrine show IMP-001` |
| `SL-002` scope doc | Objectives, boundary, affected surface, open design questions, closure intent. | ✓ authored this session |
| `DEC-011` | 0.1's four keys, future keys refused by name, compatibility tested. 0.2 widens the surface; the refusal tests invert, the 0.1 pins do not move. | ✓ `doctrine show DEC-011` |
| SL-001 design §§4, 5, 6, 8, 9 | The 0.1 mechanisms to consume: journal, pipeline, comparator, truncation, envelope, invariants, DEC-001…010. | ✓ read this session |
| `criterion-scope-smut` (first-use store) | Excludes a validating schema, modelled containment and an in-store audit *by intent*; smut complements doctrine, it does not rebuild it. | ✓ `smut get criterion-scope-smut` |

**Checked, not applicable (with reasons)**

- **Project specs / ADRs / policies / standards** — still none; `doctrine
  relation census` shows only SL-001's `shapes` and `reviews` edges ✓. The brief
  is the product authority, and SL-001 deliberately recorded binding choices as
  design-run DECs rather than ADRs.
- **Doctrine framework rules** — bind this repository's process, not smut's
  observable JSON. The brief forbids importing that project's machinery.
- **Project orientation doc** — still install scaffold, no project principles.
- **Backlog beyond IMP-001** — IMP-002 (1.0 presentation) and IMP-003 (0.1.x
  fixes) are separate slices; the charter says do not absorb them.

**Revision candidates**

None. New choices belong to SL-002's design run (DEC-012 onward), extending
DEC-001…DEC-011 rather than revising them.

## Thread 2 — code map

**Hotspots** (all ✓ by read)

| Site | What 0.2 must do with it |
| --- | --- |
| `src/spec.rs:26` `FUTURE_KEYS`, `:70` `Spec`, `:86` `parse` | Retire the future-key refusal; add five keys with strict types; keep whole-request validation before store access |
| `src/select.rs:23` `Selection`, `:41` `Reason`, `:79` `run` | Insert traversal between roots and ordering; add provenance and a second reason without widening what renderers may walk |
| `src/entity.rs:31` `reference_id`, `:60` `scan_refs` | The walker's only reference definition; `scan_refs` already yields `(top-level label, target)` in lexical/positional order |
| `src/order.rs:66` `sort_ids`, `:92` `compare` | Unchanged comparator; traversal-discovered ids sort exactly like roots |
| `src/store/mod.rs:200` `load`, `src/store/index.rs:75` | Load is index-driven and `Entities` is `BTreeMap<Id, Value>`, so whole-store scans and incoming sources are already ID-ordered |
| `src/render_markdown.rs:280` `completeness_notice` | Must disclose `traversal_budget` and `dangling`, not only `result_limit`/missing, still leading the document |
| `src/learn.rs:113` Workflows; guide tests at `:135+` | Replace the 0.1 release notice with the 0.2 surface and the two-call idiom; the "0.2 is not advertised" tests invert |
| `src/cli.rs:95,130,396`; `src/main.rs:84` | `query`/`render` share selection; `conventions` is a new read-only command, no store, no `--format mermaid` |
| `tests/compatibility.rs:316`, `default_query_pin()` | The `where` refusal test inverts; the 0.1 `query {}` pin fixes the envelope key set |
| `Cargo.toml` | No new crate is required for predicates or traversal ✓ |

**Cited facts** (✓ by read unless marked)

1. `scan_refs` labels each occurrence with the containing entity's **top-level**
   field, does not descend into a reference occurrence, iterates objects
   lexically and arrays positionally (`src/entity.rs:60–100`). It is therefore
   the walker's edge enumerator and already supplies the label.
2. `Entities = BTreeMap<Id, Value>` (`src/declare.rs:26`) and `Index` is a
   `BTreeMap` (`src/store/index.rs:75`); `load` iterates the index
   (`src/store/mod.rs:200–241`). ID order is structural, not incidental.
3. `serde_json::Map` is a `BTreeMap` under the default feature set (no
   `preserve_order` in `Cargo.toml` ✓), so embedded-object key order is lexical
   already. The determinism rule that is *not* free is **named fields in query
   order**: `scan_refs` returns lexical order, so a named-field traversal must
   iterate the requested labels and select from the scan.
4. `Selection.ids` is the only thing renderers may walk (`src/select.rs:23`,
   SL-001 design §6.3 invariant 9). The induced edge set must therefore be
   filtered to selected ids after the limit, even though it is derived from full
   entities before projection.
5. The 0.1 compatibility pin compares the whole parsed envelope
   (`tests/compatibility.rs`, `default_query_pin`), so an always-present new
   envelope member breaks it. Provenance members must either be absent on a
   non-traversing request or the pin must be deliberately re-cut.
6. `learn.rs` currently asserts the guide does **not** contain `$in` and that
   the line naming 0.2 mentions `where` (`src/learn.rs:135+`). Those are 0.1
   tests, not compatibility pins; they invert in 0.2.
7. The brief requires both Markdown and JSON to disclose truncation, missing
   roots and dangling diagnostics visibly (§163); 0.1's notice covers only the
   first two (`src/render_markdown.rs:280`).

**Naming precedents**

`Reject` variants name the offending key; `Reason` is a closed enum with an
`as_str` spelling; `Selection` + pure `project`/`envelope`; commands are structs
implementing `Command` with `recompute`/`prepare_output`; embedded text lives in
`src/conventions/*.md` via `include_str!` (the pattern `learn.rs` uses).

## First-use store as fixture

Verified against `interview.local/.smut` with the 0.1 binary and a node scan:

- 66 entity files, 66 distinct ids ✓.
- 44 distinct reference targets; **22** ids have nothing pointing at them ✓.
- Top-level reference labels across entities: `part-of` 65, `about` 48,
  `answers` 5, `projects` 1 ✓.
- `vocab` claims four labels in force and that `projects` is unused; `a-focus`
  carries `projects: [{id:"doctrine"}]` ✓. The prose is stale relative to the
  data, which is why tests must assert against entities, never against `vocab`'s
  prose.
- Entry points `start-here` → `audit-handoff`; five clusters by `part-of`;
  one retracted finding (`finding-smut-16`, status `superseded`); one answer
  pointing up against `part-of` (`answers`). ✓

The store is untracked (`.gitignore` carries `.smut`), so tests cannot read it
from a clean checkout. Promotion into the repo is a design decision.

## Cross-thread findings

1. **The walker is nearly free, the order is not.** `scan_refs` supplies labels
   and edge targets from the one definition; `BTreeMap` supplies ID order for
   roots/incoming sources and lexical order for wildcards. The only ordering the
   code must impose is named-field query order and outgoing-before-incoming, and
   the only place a second definition could creep in is re-walking the JSON.
2. **The compatibility pin forces a conditional envelope.** Adding `graph` (or
   equivalent) unconditionally would break `default_query_pin`; emitting
   provenance only when traversal was requested keeps every 0.1 call's shape
   identical and makes the new members predictable from the request.
3. **The limit sits after ordering; edges must be filtered after the limit.**
   Provenance derived before projection (so hidden reference fields still yield
   edges) must be cut to selected ids afterwards, or the envelope leaks nodes the
   result cap removed and renderers could draw them.
4. **The brief's "depth boundary is not truncation" splits the frontier from the
   reasons.** A frontier node can be unexpanded by depth or by budget; only the
   budget case sets `truncated`/`traversal_budget`. One frontier definition, two
   causes, one truncated flag that depends on the cause.
5. **`backlinks` is a scan over `Entities`, bounded by neither `depth` nor
   `budget`.** At 66 entities this is cheap; the documentation must say what the
   budget does not bound rather than implying a total-work limit.
6. **Conventions are data, not behaviour.** The store's own `vocab` is the
   exemplar and it is stale; shipping a printed set carries the same risk, so the
   guide must keep the two epistemics distinct (`learn` descriptive/version-bound,
   conventions advisory/domain-bound) and never validate.
7. **The first-use store's `projects` inconsistency is evidence for the
   conventions design, not a smut defect.** A label can look live and be stale;
   a convention that states "define and check labels" is advice, and the tool
   still refuses to check.

## Design-input deltas

What this research changes about the intended design:

- **Envelope shape.** Provenance is a conditional member, not an unconditional
  one, to preserve the pinned 0.1 envelope. The design must state the condition
  precisely (traversal keys present) and what appears when they are absent.
- **Evidence artefact flow.** Acceptance tests build a scratch store from a
  committed, normalised copy of the first-use entities (single JSON array plus a
  provenance note), rather than committing `.smut` internals or leaving the
  fixture untracked. The alternative — tests against `interview.local` — cannot
  pass in a clean checkout.
- **Guide test inversion.** The planned guide update is a required, testable
  deliverable: the 0.1 tests asserting 0.2 is unadvertised are replaced, not
  deleted silently.
- **Determinism test design.** The risky rule is named-field query order, not
  filesystem order; the shuffled-order test should shuffle the fixture's index
  and entity filenames, and separately assert label-order independent of the
  scan's lexical order.
- **No new dependency.** Predicate evaluation is a small closed interpreter over
  `serde_json::Value`; traversal is a BFS over `Entities`. Adding a graph crate
  would be a parallel model.

## Post-inquiry update (2026-11-01)

The eight design questions are settled as DEC-012…DEC-019 and drafted in
`.doctrine/slice/002/design.md`. Three research conclusions are now decisions
rather than recommendations:

- the provenance shape is a conditional `traversal` member (DEC-012), kept
  additive for 0.1 callers (DEC-019);
- the conventions ship with fixed-id seeds (DEC-017);
- the fixture is a normalised array plus provenance, gated by a privacy
  read-through before commit (DEC-018).

### Later revision (2026-11-01) — the conventions scope reduced

Two changes landed after the decisions above, and this section records them
rather than leaving the advisory describing a design that no longer exists:

- **One set ships, not two** (DEC-017 narrowed; `IMP-004` created). The
  `epistemic-records` set was removed from 0.2 because every open question about
  it was a *content* decision for one domain, not mechanism: which statuses
  exist, whether a citation needs an entity, whose a claim is. The draft, the
  thread-3 evidence and those five questions are preserved on `IMP-004`, which
  gets its own design conversation. The mechanism the research endorsed is
  untouched — embedded prose, fixed-id versioned seed, `--format json`, pipe
  through `declare`, never validated, never auto-selected — and adding a set
  later is a file plus a match arm. The **operator schema-approval gate is gone
  with the set**, so the conventions section now gets ordinary review rather
  than approval; the epistemic question lives on `IMP-004`.
- **What survives the deferral, and why it is not taxonomy.** The
  retracted-versus-superseded distinction moved into the shipped set as
  lifecycle advice, with the store's own `vocab` as the evidence — exactly the
  drift finding 6 above named, kept because it is advice for any store rather
  than a schema for one domain.
- **Seed ownership, stated precisely.** The seed supplies `body`, so a
  hand-amended body is replaced on re-install; the design now says so and points
  at the mechanism's own report of it: `declare`'s reply carries the prior
  entity under `previous` (absent under `--compact`), so a client can recover
  what the overwrite took. Verified against `src/declare.rs` and `src/learn.rs`,
  not assumed from prose.

Nothing in the pre-design evidence is retracted, and the deferral removes
content rather than evidence. The baseline is restamped after this update so
planning starts from a current advisory.
- The edge set was corrected during review to the induced subgraph of the
  **returned** entities (DEC-014 amended), derived after the limit. That applies
  the research finding — the renderer should project the envelope, not derive
  the same structure a second time — rather than introducing a new premise.
