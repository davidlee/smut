# Equality, coalescing and conflict inside a declaration batch

Settled with the operator on 2026-11-01.

Amended 2026-11-02 by SL-004 under ADR-001 (identity is `smut_id`, a reserved
key in a permissive store) and DEC-025: structural identity is spelled `smut_id`;
an ordinary `id` is caller data at every depth. Only the passages listed in SL-004
PHASE-03's amendment proposal (plus RV-028 F-2/F-3) changed; the rationale, durable
ids and unrelated decisions are retained verbatim.

## The operation algebra

Normalisation yields, **per declared entity, per top-level field**, exactly one
operation: `Set(normalized_value)` or `Drop`. Two operations on the same field
fold as:

| left | right | result |
| --- | --- | --- |
| `Set(v)` | `Set(v)` equal | coalesce |
| `Set(v)` | `Set(v')` unequal | reject the whole batch |
| `Set(v)` | `Drop` | reject the whole batch |
| `Drop` | `Set(v)` | reject the whole batch |
| `Drop` | `Drop` | coalesce |
| — | `Drop` | no-op; the field is absent |
| — | `Set(v)` | assign |

Granularity is the top-level field, not the entity: disjoint fields of the same
id combine, which is what makes repeated patches in one batch useful.

Comparison is against the **normalised** value, which is why the whole request is
normalised before any conflict is detected. These two declarations coalesce
rather than conflict, because normalisation extracts the nested declaration and
replaces it with an identity-only reference in both cases:

```json
[{"smut_id":"q","refs":[{"smut_id":"t","label":"x"}]}]
[{"smut_id":"q","refs":[{"smut_id":"t"}]}, {"smut_id":"t","label":"x"}]
```

## Equality

Equality is the JSON data model's structural equality — `serde_json::Value`
`PartialEq` — with no coercion anywhere.

- **Objects**: key order is irrelevant. Obtained by keeping `Map`
  `BTreeMap`-backed, i.e. **not** enabling serde_json's `preserve_order`
  feature.
- **Numbers**: `1` and `1.0` are **distinct** values and therefore conflict if
  both are assigned to one field in a batch. Reasons: no i64/u64/f64 comparison
  ladder to document; no arbitrary choice of which representation to store when
  two forms compare equal; consistent with the no-coercion stance the brief
  already takes for predicates. The consequence is surprising but decidable, and
  it is documented in the format doc.
- **Arrays**: order-sensitive.
- **`null`**: an ordinary assigned value. `null` and `Drop` conflict — one is a
  value, the other is absence.
- Duplicate keys inside one input object are rejected at parse time (exit 2, per
  DEC-001 rule 1), so they never reach this algebra.

## Consequences pinned by the brief

- `"DROP"` is a sentinel **only** as a top-level entity field value. Inside
  embedded values and arrays it is ordinary data. A top-level field therefore
  cannot be assigned the literal string `"DROP"` in 0.1.
- Explicit `smut_id` plus only `Drop` fields, on an absent entity → **created**,
  identity-only. A bare `smut_id`-only object never creates anything.
- `Drop` of an absent field is a no-op and does not make an entity `updated`.
- `unchanged` means the normalised patch leaves the entity structurally equal.
- `previous` is captured once per affected entity from the snapshot taken before
  the entire batch: the full entity for `updated`, `null` for `created`, omitted
  for `unchanged`, omitted everywhere under `--compact`.
- `missing` is computed against the **post-batch** id set, so a reference to an
  entity created in the same batch is not missing. It is unique, in encounter
  order.
- `ids` echoes the top-level input order including repeats; `changes` contains
  each declared entity once, in first-encounter order.
- First-encounter order is positional through arrays and **lexical** through
  objects (a consequence of the `BTreeMap` choice), so `changes` order and
  stored-file bytes are canonical rather than textual.
- Batch rejection is total: nothing is written, and no `previous` values are
  reported.

## Alternatives considered

- Numeric equality with a cross-type comparison ladder — rejected: needs an
  arbitrary stored representation for equal-but-differently-typed numbers.
- `preserve_order` — rejected: makes logically identical entity sets differ
  byte-for-byte in the store, for author-order fidelity nobody asked for.
- Entity-level conflict detection (reject a whole entity on any difference) —
  rejected: the brief requires disjoint fields of the same id to combine.
