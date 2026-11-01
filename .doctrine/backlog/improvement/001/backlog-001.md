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
