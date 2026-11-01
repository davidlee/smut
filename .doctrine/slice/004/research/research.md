# SL-004 research — identity key `smut_id`, the format break, and IMP-005

Produced for SL-004 (identity is `smut_id`), before design. Two read-only threads
(governance applicability; code map) plus cross-thread synthesis, assembled by the
orchestrating agent.

Baseline: `baseline.toml` (minted by `doctrine slice research 4`).
Raw thread output: `raw/thread-1-governance.md`, `raw/thread-2-code-map.md`.

## Verification legend

- ✓ — claim independently verified by the consuming agent at write time (a grep, a
  read of the cited site, or a run of the built binary in a scratch store).
- unmarked — claim cited to its source but not re-checked here. Design and plan may
  only load-bear ✓ rows or verify the rest at point of use.

## Thread 1 — governance applicability

**Binding, and the reason this slice exists.**
- ✓ ADR-001 (accepted 2026-11-01) — identity is `smut_id` in entity objects and
  references, `changes[].smut_id`, container-named id values unchanged, the format
  break unmitigated, and the boundaries it leaves standing (`smut_id` structural at
  every depth; the `DROP` sentinel; `null` versus absence; `$` in predicates).

**Binding records whose letter carries the old key, and must be reconciled.** These
are tier-2 accepted decisions; the slice does not get to contradict them silently,
and `/reconcile` is the writer.
- ✓ DEC-005 — defines an entity node as "a `Value::Object` with a member `id` whose
  value is a nonempty string", and states as a consequence that "an embedded object
  carrying id cannot be inline metadata". That consequence *reverses* under this
  slice; the decision needs substance amendment, not respelling.
- ✓ DEC-007 — entity serialisation names `id` first, and `select: []` yields `id`
  only. Both become `smut_id`.
- ✓ DEC-010 — the documentation set repeats the nested-`id` limitation (a second
  statement of DEC-005's consequence, so it recants too).
- ✓ DEC-016 — `learn`'s two-call recipe spells `select:["id"]` and
  `where:{"id": ...}`. Both move, and the recipe is the guide's most-copied text.
- ✓ DEC-002, DEC-012 — examples embedded in the decision prose use `"id"`; respell.
- ✓ DEC-001 — "id-keyed entity snapshot" and "write sets in public-ID space": that
  is internal vocabulary about ids-as-values, not about the key name. Unaffected.
- ✓ DEC-008 — `--` for opaque ids and validate-before-store: unaffected.
- ✓ DEC-009 — Markdown rendering shows identity as a value, never as a key pair:
  unaffected.

**Not applicable, with reasons (checked, not assumed).**
- ✓ No product or tech specs exist (`doctrine spec list` is empty), so no REQ or SPEC
  requirement pins the key name.
- ✓ No policies and no standards are active (`doctrine policy list`, `doctrine
  standard list` both empty; boot snapshot agrees).
- ✓ No ADR other than ADR-001 exists.

**Prior art that contradicts ADR-001 and needs a decision outside doctrine.**
- ✓ `smut-implementation-brief.org:33` — "An entity is a JSON object with a nonempty
  string =id=." The brief is the v1 implementation brief (a repo file, not a
  doctrine entity; IMP-002 refers to it as the input for 1.0), and its six `"id"`
  occurrences are the examples the README was written from. It is a *revision
  candidate*: either the brief is amended where ADR-001 supersedes it, or it is
  marked as the v1 brief whose identity key was superseded. Flagged at design time
  rather than silently left standing.
- ✓ `smut-implementation-brief.org:47` — "Entity identity cannot be dropped or
  renamed", and the `DROP` sentinel cannot be assigned as a top-level field value.
  Consistent with ADR-001 § "The boundaries this leaves"; no conflict.

## Thread 2 — code map

**The identity constant and its readers (hotspots).**
- ✓ `src/entity.rs:17` — `pub const ENTITY_ID: &str = "id";` — the single definition.
  27 `ENTITY_ID` references across `src/`.
- ✓ `src/entity.rs:31` — `reference_id`: the one predicate that decides "is this an
  entity node or a reference". Reads the key, and refuses a non-string/nonempty
  value (the exit-2 paths ISS-002 lists).
- ✓ `src/entity.rs:71` — `scan_refs`: walks a *validated* entity for references;
  documents a precondition it panics without (see IMP-005 below).
- ✓ `src/entity.rs:159` `extract`, `:195` `hoist` — the recursive pass; `hoist` skips
  `ENTITY_ID` (`if key == ENTITY_ID { continue }`) so identity never becomes a
  field operation, which is why ids cannot be dropped or renamed.
- ✓ `src/declare.rs:181` `normalize`, `:400` `declaration_roots` — generation happens
  for a root without the key; `:110` builds the `changes[]` entry with
  `entry.insert("id", ...)` — **the one envelope key this slice changes**.
- ✓ `src/store/journal.rs:246` `entity_id` — the write path re-reads the key to
  validate a put; `:172`-region `translate` refuses a write set that contradicts §3.1.
- ✓ `src/store/mod.rs:226` — load validates only that the file is an object and that
  its identity matches the index key; it does not inspect nested identity.
- ✓ `src/store/index.rs:28` — `FORMAT_VERSION: u64 = 1`; `:111` refuses any other
  version with `UnknownFormatVersion { found, expected }`. The break is a one-line
  bump plus the existing refusal — no new machinery needed for "blow up loudly".

**The sweep, measured.**
- ✓ 974 literal `"id"` occurrences across 27 files under `src/`, `tests/`, `docs/`,
  `README.md`.
- ✓ `tests/fixtures/interview/entities.json` holds 223 of them: a real 66-entity
  declaration list (`PROVENANCE.md` beside it). `tests/traversal.rs`,
  `tests/selection.rs`, `tests/fixture.rs` pin envelopes computed from these ids, so
  the rewrite must be spelling-only — same ids, same values, same referrals.
- ✓ `docs/store-format.md:104-111` — the canonical-bytes section names `id` first and
  says the same wrapper serialises `objects[]`; `:70-72` records that an entity's
  *basename* is an internal uid, independent of the public id (so the key rename
  moves no file); `:250-252` shows diagnostics that say `holds id "b"`.
- ✓ `src/learn.rs:65-86` — the guide's Declaring and References paragraphs, including
  the two statements that a nested `id` cannot be inline metadata (`:79-82`) and a
  top-level id-only object is a reference (`:86`).

**The loader/panic seam this slice also carries (IMP-005).**
- ✓ `src/entity.rs:85-106` — `scan_refs` panics on a nested identity that is not a
  nonempty string.
- ✓ `src/store/mod.rs:218-238` — nothing validates nested identity at load, so a
  hand-edited store reaches that panic. RV-022 F-3 reproduced it end to end
  (`get` exits 0 returning the malformed value; a `follow` traversal exits 101).

**Naming precedents.**
- ✓ The store already separates public id from placement (`index.json` maps id →
  basename; basenames are `nanoid` uids), so nothing about the rename moves bytes
  between files.
- ✓ The codebase's convention for an unreachable internal invariant is
  `unreachable!`/assertion (`src/order.rs:136`), not a new error class — relevant
  because the loader fix must choose between "corruption, exit 1" (available, and
  correct here) and asserting.

## Cross-thread findings

1. **ADR-001 is the only accepted decision that can authorise this change, and six
   others must move with it** — with DEC-005 reversing rather than respelling. That
   is a reconciliation obligation, not a code task, and the slice's audit must carry
   it or the corpus will hold two identity rules.
2. **The brief is the one governance-adjacent artefact that contradicts ADR-001**
   (`:33`). It is not a doctrine entity, so no verb will flag it. Design should
   decide: amend it, or declare it historical in the one paragraph that matters.
3. **The format version is already a working refusal path**, so "blow up loudly"
   needs no new mechanism — which is exactly why folding IMP-005 into this slice is
   cheap: the loader is being touched anyway, and the new spelling is the moment to
   validate nested identity rather than respell around the panic.
4. **The fixture store is the trap.** A careless rewrite of
   `tests/fixtures/interview/entities.json` changes ids and therefore sort order and
   referrals, breaking pins that have nothing to do with the key. Spelling-only is a
   property the design must state and a test must hold.
5. **Silence is the failure mode to watch.** After the rename, any surface left
   spelling `id` fails *quietly*: a missing field fails every predicate operator
   except `$exists`, so a stale `where: {"id": ...}` returns an empty result at exit
   0 (verified in a scratch store). The guide and the tests are the only places that
   can make that loud.

## Design-input deltas

- Format version 1 → 2, with the existing `UnknownFormatVersion` refusal carrying
  the message; no migration path.
- Loader: validate nested identity at load (corruption, exit 1) as part of the same
  pass that respells it (IMP-005).
- `changes[].id` → `changes[].smut_id` is the single envelope key change; `ids`,
  `deleted`, `missing`, `frontier`, `depths`, `edges[].source`/`.target` stay.
- The fixture rewrite is spelling-only, and needs a check that holds it.
- Six DECs move (one in substance); the brief needs an explicit disposition.
- Minting (ISS-003) and the short flags (IMP-007) are out of this slice.
