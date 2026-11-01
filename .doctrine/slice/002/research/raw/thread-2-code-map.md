# Raw thread 2 — code map (0.1 as built)

Producer: orchestrating agent, degraded mode. Read-only.

Hotspots (verified by read this session):

- `src/spec.rs:26` — `FUTURE_KEYS = ["where","follow","backlinks","depth","budget"]`;
  `Spec` at `:70` has `from`/`select`/`order`/`limit`; `parse` at `:86` rejects
  future keys by name at `:102`.
- `src/select.rs:57` `select`, `:74` `get`, `:79` `run` — roots (BTreeSet dedupe)
  → `order::sort_ids` → limit/`result_limit` (`:108–119`). `Selection` at `:23`
  carries ids/store/fields/missing/truncated/reasons; no traversal data.
  `envelope`/`project` follow.
- `src/entity.rs:31` `reference_id` (single entity-node predicate),
  `:60` `scan_refs` → `(label,target)` where the label is the originating
  top-level field, objects lexical, arrays positional, and a reference
  occurrence is not descended into. `src/declare.rs:26`
  `pub type Entities = BTreeMap<Id, Value>`.
- `src/store/mod.rs:200` `load`: `Index::read` → `refuse_unexplained_entries` →
  iterate `index.entries()` to build `Entities`. `src/store/index.rs:75`
  `objects: BTreeMap<Id, String>`; index bytes never come from `readdir`
  (`:72` doc).
- `src/select.rs` `Reason` at `:41` has one variant `ResultLimit`.
- `src/render_markdown.rs:280` `completeness_notice` leads the document with
  truncation reasons and missing ids; no dangling/budget-specific handling yet.
- `src/learn.rs:113` Workflows teaches declare/amend/render and "prefer two
  simple calls"; `GUIDE` tests at `:135+` assert 0.2 predicates are *not*
  advertised and a 0.2 line mentions `where`.
- `src/cli.rs:95` Query / `:130` Render; `:396` `check_format` allows only
  markdown and refuses mermaid as future. `src/main.rs:84` `execute`.
- `tests/compatibility.rs:316` refuses `where` by name and then pins the
  untouched 0.1 `query {}`; `default_query_pin()` fixes the envelope key set
  (`dangling`, `missing`, `objects`, `truncated`, `truncation_reasons`).
- `Cargo.toml`: serde/serde_json/clap/fs4/nanoid; proptest dev. No new crate is
  needed for predicates or traversal.

First-use store (`interview.local/.smut`, read with `target/release/smut`):

- `jq '.objects | length' index.json` → 66.
- Node scan over the 66 entity files: 66 distinct ids, 44 distinct reference
  targets, 22 ids with no incoming reference.
- Top-level reference labels by entity: `part-of` 65, `about` 48, `answers` 5,
  `projects` 1.
- `vocab` says four labels are in force and that `projects` is "defined and
  unused"; `a-focus` carries `projects: [{id:"doctrine"}]`. The prose is stale
  relative to the data.
- `criterion-scope-smut`, `audit-handoff`, `prop-result-provenance`,
  `finding-smut-17/18/19`, `prop-conventions-library`, `obs-rehydration` read in
  full this session.
