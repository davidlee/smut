# IMP-001: smut 0.2 — graph release: predicate language and traversal

Second of three releases delivering the v1 brief
(`smut-implementation-brief.org`). Prerequisite: SL-001 (0.1 notebook) shipped.

## What it delivers

The capability sentence: *ask questions across the graph, not just read named
entities back.*

- Full predicate language: `$eq`, `$ne`, `$in`, `$exists`, `$contains`, plus
  `$and`/`$or`/`$not`, the operator-vs-literal disambiguation rule, and
  missing-field semantics.
- Traversal: `follow`, `backlinks`, `depth`, `budget`, with the brief's
  determinism rules (outgoing before incoming, named fields in query order,
  wildcards lexical, array items in order, incoming sources in ID order).
- `dangling` reporting for references encountered during requested traversal.
- `traversal_budget` truncation reason, alongside 0.1's `result_limit`.
- Widening the query-spec grammar with the keys 0.1 deliberately rejects.

## Why it is not in 0.1

The brief pins the store, declaration and patch semantics almost completely; it
does not pin the query language's *shape* in use. Designing the traversal and
predicate surface after the notebook exists and has been used is cheaper than
designing it up front, and the brief names the query language as the most likely
site of scope creep (nested-field predicates, pipelines, pagination).

## Inherited design questions

Deferred from SL-001's inquiry map because they belong to this slice's design
run, not to the notebook's:

- traversal discovery order, budget accounting and dedupe timing (pruned from
  SL-001 as inq-5's traversal half);
- how projection interacts with traversal so hidden reference fields still
  yield Mermaid edges (the Mermaid half of SL-001's inq-6);
- the scalar-field ordering rules extended to traversal-discovered roots.

## Ready to pick up when

`smut render` on a real notebook has been used in anger and shown which
questions the user actually asks — i.e. once `where`'s absence is a felt gap
rather than an anticipated one.

## Trigger met (2026-11-01)

The condition above is satisfied. The owner used 0.1 for real in two live
sessions and kept the transcript as a smut store (`interview.local/.smut`,
66 entities, entry points `start-here` → `audit-handoff`); RV-004's audit read
that store and found no 0.1 conformance defect in it, so it is product evidence
rather than defect evidence, and it is the design input this item asked for.

What it settles, with the store's own finding ids:

- **The questions actually asked.** With 0.1 the operator answered "which open
  questions hang off this session?" by dumping all 66 entities and filtering by
  hand — `finding-smut-05`, `finding-smut-18`. `where`'s absence is now a felt
  gap, not an anticipated one.
- **`where` stays roots-only.** `finding-smut-18`: the two-call composition
  (traverse for ids, then filter those ids as roots) answers the common graph
  question, because `from` takes an explicit id list. No change to 0.2's
  contract; the idiom goes in the guide and, better, in the conventions command.
- **Labelled traversal is already the planned surface.** `follow` takes an array
  of field names, so the label is the originating top-level field — the store's
  edge labels *are* its traversal vocabulary (`finding-smut-06`).
- **The one genuinely new design question: what a traversal result carries**
  (`prop-result-provenance`). Nodes plus the induced edge set plus the unexpanded
  frontier, with minimum depth as a per-object scalar. The frontier is the
  argument: `depth` and `budget` cut the walk and today the only residue is
  `truncated: true`, which says something was cut and not where. The edge set is
  also what 1.0's Mermaid renderer needs, so it is one derivation shared rather
  than two.
- **Where conventions live** (`prop-conventions-library`, shape endorsed by the
  owner): a separate read-only command — `smut conventions [NAME]` — prose by
  default, a pipeable declaration on request, never validated, never
  auto-selected, with `learn` kept descriptive and version-bound. Contents and the
  domain list are still drafts; epistemic records is the only domain with evidence
  behind it.
- **Recommended work order** (`finding-smut-19`): the label filter was never the
  cost. Risks are a second definition of "reference" (the walker must consume
  `src/entity.rs`'s, not restate it), the six determinism rules (with
  `src/store/index.rs` iteration order normalised rather than trusted), and
  `backlinks` scanning the store per hop. Sequence: walker and labels, then
  depth/budget/truncation and the envelope reasons, then predicates — which are
  independent.
- **Fixture.** The same store is a better acceptance fixture than anything
  synthetic: 66 entities, awkward cases included, 22 with nothing pointing at
  them, so a `backlinks` test has a hit and a miss without inventing either. It
  lives in the owner's working tree, untracked (`.gitignore` carries `.smut`), so
  promoting a copy into the repo is this item's decision, not SL-001's.

**Do not build** (settled by scope, `criterion-scope-smut`): a validating schema,
containment as a modelled feature, an audit mechanism inside the store, and a
promotion path into doctrine as a 0.2 requirement.

