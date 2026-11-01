# Ordering, the numeric comparator, and truncation

Settled with the operator on 2026-11-01. The comparator was rejected once, for a
real defect: switching comparison domains per pair breaks transitivity.

## Pipeline

Roots → `order` → `limit` → project.

- **Roots**: `from` supplies them in array order; absent, every id in ID order.
  Dedupe keeps the first occurrence. That order is *discovery* order only;
  supplying `order` replaces it wholesale.
- `order: []` is equivalent to omitting `order`: discovery order is preserved.
- **Missing roots** are excluded from results and reported in `missing`, in
  `from` order, unique. They never consume `limit`.

## The comparator

Entries are compared in array order. Per key:

| situation | outcome |
| --- | --- |
| both missing | continue to the next key |
| exactly one missing | that record sorts **later**, regardless of direction |
| both present | compare values; apply direction **only to a non-equal result** |

All keys tying ⇒ **ID ascending** (UTF-8 byte order), which makes the whole thing
a total order independent of sort stability or discovery order.

Missing-last is **per key, not a final partition**: with
`order: [priority, title]`, two records missing `priority` must still be ordered
by `title`. Appending missing-valued records in ID order would discard every
secondary key.

**Sort-field values are validated for every candidate before sorting begins**, so
an array or object in an order field rejects reliably even when there is one
candidate, or when an earlier key would decide every comparison.

## Scalar type order

`null` < `false` < `true` < number < string.

- `null` is a **value** that sorts first, not an absence.
- Arrays or objects in an order field ⇒ explicit error, exit 2.
- Strings: UTF-8 byte order (codepoint order). No collation, no locale, no
  normalisation.
- `id` is an ordinary top-level field and may be named in `order`.

## Numbers: exact comparison, never domain-switching

Integers and floats are compared **exactly**, against the value each actually
represents — a small, separately tested numeric comparator that decomposes the
float's IEEE-754 representation rather than rounding the integer to a float.

The rejected alternative was "integers compare exactly in i128; mixed pairs
compare as f64". It is not merely imprecise, it is **cyclic**:

| id | value |
| --- | --- |
| `z` | integer 9007199254740992 |
| `a` | integer 9007199254740993 |
| `m` | float 9007199254740992.0 |

`z < a` (exact integers); `a < m` (the integer rounds down to the float, tie
broken by id); `m < z` (equal values, tie broken by id). A cycle. Precision loss
can be documented; a cycle cannot be sorted with.

If exact comparison were ever judged disproportionate, the only coherent fallback
is **always-f64** with its precision limit documented. Never switch comparison
domains based on the pair.

Consequences to state in the docs:

- `1` and `1.0` are **unequal** for structural equality (DEC-002) but **compare
  equal** for ordering — different relations. Also `0`, `0.0` and `-0.0` compare
  equal, and order ties are broken by ID.
- The numeric comparator is property-tested for **totality and transitivity**
  over a corpus including ±2⁵³, i64/u64 extremes, subnormals, signed zeros and
  large integral floats. Property tests are the point: this is the one comparator
  where a plausible implementation can be silently wrong.

## Truncation

`limit` is a positive integer (0 rejected), default 100, applied after ordering.
`truncated: true` and `truncation_reasons: ["result_limit"]` appear only when rows
were actually omitted; a result of exactly `limit` rows is not truncated. `get`
takes no spec and no limit, so it never truncates (DEC-001 rule 4). Missing roots
do not consume `limit`.

## Alternatives considered

- Missing-values appended as a final partition — rejected: destroys secondary
  keys.
- Domain-switching numeric comparison — rejected: cyclic, so not a total order.
- Always-f64 — coherent, retained only as the fallback if exact comparison proves
  disproportionate.
- Making missing values sort first under `desc` by reversing the whole
  comparator — rejected: "regardless of direction" is not what a reversing
  comparator does, and the difference is invisible until someone writes `desc`.
