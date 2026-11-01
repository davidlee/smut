# smut 0.1 — notebook: persist, revise and read back

## Context

`smut` lets an agent persist, revise, retrieve and recombine a mutable graph of
linked thoughts during a conversation, with little more effort than emitting
Markdown. The repository currently holds only scaffolding (`Cargo.toml` with no
dependencies, a `src/main.rs` printing `Hello, world!`, `flake.nix`, `justfile`).

The authoritative statement of intent is the repository-root brief
`smut-implementation-brief.org` (v1, dated 2026-11-01), supplied by the
repository owner. Where the brief and this document disagree, the brief wins;
divergences are recorded here rather than silently resolved.

`smut` is deliberately standalone: it must not import the wider Doctrine
project's governance machinery, event sourcing, or schemas.

**This slice is the first of three releases that together deliver the brief's
v1.** The brief is a full product specification; delivering it as one change
would mean nothing is usable until nearly all of it exists, and would leave the
query language to be designed before the store has ever been used.

| Release | Capability | Where |
| --- | --- | --- |
| **0.1 (this slice)** | Write linked thoughts, read them back as Markdown, never lose a batch | SL-001 |
| 0.2 | Ask questions across the graph — predicates and traversal | IMP-001 |
| 1.0 | Present the graph (Mermaid) and document the limits | IMP-002 |

The brief remains the authority for every one of them; each release implements
the corresponding sections of it in full, so nothing is delivered twice and
nothing is dropped.

## Scope & Objectives

Deliver a working `smut` 0.1 binary with these observable capabilities, matching
the brief's contracts exactly:

1. **Entity model and declaration** — JSON objects with a nonempty string `id`;
   recursive reference/declaration extraction inside embedded values;
   collision-checked NanoID generation for ID-less top-level objects; the
   documented consequence that an embedded object carrying an `id` cannot be
   ordinary inline metadata.
2. **Patch semantics** — top-level field patching, omission preserves, wholesale
   replacement of arrays/embedded objects, the `"DROP"` sentinel,
   whole-batch normalise-then-write with conflict rejection and per-entity
   pre-batch `previous` values.
3. **Store** — `$PWD/.smut` index + per-entity files, internal UID/public ID
   separation, path safety, deterministic readable JSON, a single exclusive
   lock, a durable journal/staging recovery protocol with idempotent pre-commit
   and post-commit recovery, and the documented guarantee it licenses: atomic
   visibility through locking and recovery, carrying its
   commit-before-acknowledgement ambiguity.
4. **Commands** — `declare`, `get`, `delete`, `query`, `render` (Markdown only),
   `learn`, plus `--help`/`--version`, with the brief's JSON envelopes, exit
   codes, stdout/stderr separation and stdin/argument ingestion rules.
5. **Selection subset** — the query specification keys `from`, `select`,
   `order`, `limit`; documented scalar type order, per-key missing-values-last,
   ID tiebreak, and an exact numeric comparison; `result_limit` truncation
   reporting. `where`, `follow`, `backlinks`, `depth` and `budget` are rejected
   in 0.1 with a message naming 0.2. Rejection keeps the *syntax* additive — a
   working 0.1 call cannot become invalid when the key set widens — but it does
   not by itself guarantee unchanged behaviour; representative 0.1 invocations
   are kept as compatibility tests.
6. **Markdown renderer** — heading field/level, verbatim string body,
   `--show-ids`, `--show-fields` with compact JSON, ID fallback when nothing is
   visible, and visible disclosure of truncation and missing roots.
7. **Embedded `learn` guide** — concise, version-matched instructions covering
   the 0.1 surface only: commands, patch/reference conventions, `DROP`, batch
   responses, query defaults, missing versus null, and the write/amend/render
   workflows. The guide must not advertise 0.2 behaviour.
8. **Documentation** — `README.md` with command/response examples, and a
   versioned on-disk-format document carrying the durability model, the
   commit-ambiguity statement and the lock/concurrency model.
9. **Tests** — the 0.1 subset of the brief's mandated coverage: declaration and
   normalisation (single/array, generated IDs, forward/nested references,
   embedded values, id-only missing references); patch semantics (omitted
   fields, null, `DROP`, replacement, nested declarations inside replacements);
   conflict and rejection without partial mutation; duplicate JSON keys;
   filename/public-ID separation, escaping paths, malformed index/entity data,
   absent store; concurrent processes; injected interruption at every journal
   and publication boundary with recovery yielding a coherent post-commit batch
   or the pre-commit state; ordering and invalid-query behaviour; `get` ordering
   and non-cascading delete; render escaping and visible incompleteness; and the
   0.1 round-trip workflow end to end, including stdin and stdout/stderr
   separation.

## Deferred to later releases

Named here so the boundary is a decision, not an oversight:

- **To 0.2 (IMP-001):** the `where` predicate language in full, `follow`,
  `backlinks`, `depth`, `budget`, `dangling` reporting, `traversal_budget`, and
  the remaining query-spec keys.
- **To 1.0 (IMP-002):** the Mermaid renderer, `--heading-*` interaction with
  non-Markdown formats, the portability-limits section, and the generated-store
  smoke benchmark.

Everything the brief lists under "Deliberate follow-ups, not v1" stays out of
all three releases: Lua, query pipelines, nested-field predicates, pagination,
secondary indexes, daemon/watch mode, arbitrary document editing, integrity
repair, history, optimistic revision checks, first-class edge types, schema
enforcement and integrations.

## Non-Goals

- Any reusable public library API (a binary plus internal modules only).
- Doctrine integration: no `.doctrine` coupling in `smut` code, no governance
  concepts in the tool, no trust enforcement around stored content.
- Non-Linux acceptance testing. Portability *limits* are documented in 1.0, not
  solved.
- Licensing decisions (repository-owner call).
- Changes to the brief's observable semantics without an explicitly recorded
  rationale.

## Summary

One release delivering the notebook round trip: persist a batch of linked
thoughts, revise them by patch or `DROP`, read a named set back as Markdown, and
be able to trust that an interrupted command leaves either the whole batch or
none of it. The bulk of the engine work — normalisation, patch application, the
store and its journal, selection, the Markdown projector, the CLI and the learn
guide — lands here. Its densest design component is the commit protocol; its
least forgiving is conflict detection, which silently loses or silently rejects
assignments if the equality rule is wrong.

## Follow-Ups

- IMP-001 (0.2 graph release) and IMP-002 (1.0 presentation release) carry the
  remaining brief sections and the design questions deferred with them.
- Product/tech spec extraction: the brief currently lives as an `.org` file in
  the repository root. Promoting its durable intent into `.doctrine/spec/` is
  deliberately deferred until all three releases ship, so the spec describes
  what was actually built.
- Store format evolution policy beyond `format_version` rejection (migration
  tooling) — only when a v2 format is actually proposed.
