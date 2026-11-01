# The reference predicate and normalisation

Settled with the operator on 2026-11-01, with the label rule and three
implementation pins confirmed.

## One definition, three surfaces

```rust
// entity.rs — the only place "reference" is defined
fn reference_id(v: &Value) -> Result<Option<&str>, Error>;   // Ok(None) ⇒ not an entity node
fn scan_refs(entity: &Value) -> impl Iterator<Item = (&str /*label*/, &str /*target*/)>;
fn extract(v: &Value) -> Result<(Value /*replacement*/, Vec<Decl>), Reject>;
```

`normalize` (0.1) is built on `extract`. 0.2's traversal will be built on
`reference_id` and `scan_refs`. Neither restates the rule.

## What counts

A `Value::Object` with a member `id` whose value is a nonempty string. Presence is
the entire test — field count and field names are irrelevant.

- `id` present but not a nonempty string ⇒ rejection. `{"id": null}` is an error,
  not "no id, please generate".
- Id generation applies **only to top-level declare objects that lack `id`
  entirely**. A nested object without `id` is an embedded value and is never
  given one.
- Ids are opaque, case-sensitive, unnormalised; a whitespace-only id is a valid
  nonempty id.

## Declaration vs reference, and the single pass

An object with `id` and nothing else is a pure reference. An object with `id`
plus any other field is a declaration: hoisted to top level, its occurrence
replaced with `{"id": …}`.

Normalisation is **one recursive pass over finite JSON** — each occurrence is
visited exactly once, not an iterative fixpoint algorithm. Hoisted entities'
fields are visited by the same pass, so nesting inside a hoisted entity behaves
identically, including inside a wholesale-replaced array or object. Termination
is structural: every replaced occurrence becomes ID-only. Shared ids collide only
afterwards, and are reconciled under DEC-002.

**Declaration order is preorder** — a parent is recorded before the declarations
it contains. For `{"id":"a","refs":[{"id":"b","note":{"id":"c","body":"x"}}]}` the
order is `a, b, c`, even though recursive normalisation finishes `c` first.
Otherwise an implementation detail can silently reverse the promised
first-encounter order.

## Labels

`scan_refs` starts at each **non-`id` field** of the entity and carries that
field's name through embedded objects and arrays. On reaching an entity
reference it emits `(field, target)` and **stops descending into that
occurrence** — the referenced entity is scanned separately if it was declared in
this batch.

A label is therefore the field name on the entity that *holds* the reference,
after hoisting. For `{"id":"a","refs":[{"id":"b","note":{"id":"c"}}]}` the edges
are `a.refs → b` and `b.note → c`. The request path has no lasting semantic role.

`scan_refs` yields *occurrences*; deduplication is the consumer's — 0.1 dedupes
`missing` by target, 0.2 will dedupe edges by source/field/target.

**Precondition:** `scan_refs` is non-fallible because its input is a *validated,
normalised entity*. It must not silently skip a malformed id, and it must not be
called on raw request JSON.

## Error classification — request vs stored

The predicate is shared; its error type is not.

| situation | class | exit |
| --- | --- | --- |
| invalid `id` in a declaration (a request) | invalid input | 2 |
| invalid `id` inside persisted store data | corruption | 1 |

Reusing the predicate must not let its rejection type dictate how a storage
failure is classified. The store's loader wraps predicate failures as corruption;
the request path surfaces them as user error.

## Missing references

Only references appearing **in the request** are resolved, against the
**post-batch** id set. That includes top-level id-only requests as well as nested
references. Untouched stored content is never audited — that would be a
store-wide integrity scan, which the brief excludes.

## Documented consequence

An embedded object carrying `id` cannot be inline metadata in 0.1. Stated in the
README and in `learn`: it is the one convention a new user will trip over.

## Alternatives considered

- An iterative fixpoint loop — rejected as unnecessary machinery over finite
  JSON; a single recursive pass with preorder recording is equivalent.
- Label = request path (`refs.note`) — rejected: the path has no semantic role
  after hoisting, and 0.2's backlinks need the containing entity's field.
- `scan_refs` returning `Result` and skipping malformed ids — rejected: silent
  skips are how references disappear; the precondition makes the failure
  impossible instead.
- One error type for request and stored data — rejected: conflates exit 2 with
  exit 1.
