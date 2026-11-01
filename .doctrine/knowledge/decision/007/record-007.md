# Projection, the envelope, and the serialisation boundary

Settled with the operator on 2026-11-01, with the serialisation clarification.

## Selection

```rust
struct Selection<'a> {
    ids:     Vec<&'a Id>,          // ordered, deduped, post-limit
    store:   &'a Entities,
    fields:  Option<&'a [String]>, // `select`; None = all
    missing: Vec<&'a Id>,          // explicit roots absent, `from` order, unique
    truncated: bool,
    reasons: Vec<Reason>,          // [ResultLimit] in 0.1
}

fn project(&Selection<'_>) -> Vec<Value>;   // owned, field-filtered
fn envelope(&Selection<'_>) -> Value;       // pure; the shell only prints it
```

One lifetime covers the common usable duration: selected ids borrow from the
store, missing ids borrow from the request, fields borrow from the spec. The
signature borrows all inputs accordingly; separate lifetimes are not forced.

`envelope` lives in the core, so query/get tests need no filesystem. `get` is
`select` with `from` supplied, no limit, no `select` — one code path.

**The store reference is not permission to widen the result.** Renderers consume
`Selection` and render only `ids`; 0.2's Mermaid output will include only edges
whose both endpoints are selected. Having the whole store in hand must never let
a renderer bypass `limit`.

## Envelope

- `id` in every object regardless of `select`; `select: []` yields `id` only.
- Selected fields absent from an entity are omitted, not `null`.
- Selection addresses top-level fields only.
- `objects` is flat; referenced entities are never inlined.
- `missing`, `dangling`, `truncated`, `truncation_reasons` are always present,
  even when empty or false. `dangling` is structurally `[]` in 0.1 — kept so the
  envelope shape does not change in 0.2.
- `missing` and `truncation_reasons` are unique and ordered.
- `select` duplicates are tolerated as a set; naming `id` is a no-op; naming an
  unused field is not an error. **The deduplicated `select` set never determines
  JSON key order.**

## Serialisation happens at the final boundary

`Value` erases any ordering a wrapper imposed earlier: if `project()` builds
ordinary values and `envelope()` wraps them, a custom entity serialiser used
upstream has no say in the final bytes. So entity serialisation occurs **at the
final serialisation boundary**, through one shared wrapper used for stored entity
files and for projected entities alike.

Never serialise and then re-parse to arrange keys.

**Canonical ordering**, specified per level:

- entity root: `id` first, remaining keys lexical;
- embedded ordinary objects: all keys lexical;
- arrays: order preserved.

That is one ordering convention, not literally one textual representation. Two
byte-level forms, specified separately:

| surface | form |
| --- | --- |
| stored entity files, `index.json`, journal manifest | pretty-printed, 2-space indent, trailing newline (a human opens these) |
| command stdout | compact single line, trailing newline (machine-facing, pipe-friendly) |

Rendering consumes structured `Selection`; serialisation does not sit between
selection and rendering.

Scope: a shared `Entity` wrapper is proportionate. A general canonical-JSON
framework would not be — non-entity values serialise as ordinary `serde_json`.

## Alternatives considered

- Fully key-sorted entities (no `id`-first) — rejected: `id` is what a reader
  looks for first, in output and in files, for a small amount of code.
- Serialising entities upstream with a custom `Serialize` and re-parsing —
  rejected: `Value` erases the ordering, so the custom impl would be dead code.
- One textual form for files and stdout — rejected: pretty files and compact
  stdout are both right and cost nothing to specify.
- Linking the store into renderers as free access — rejected: it invites bypassing
  `limit` and selecting entities nobody asked for.

## Serialisation clarification (SL-001 audit, RV-004 F-1)

The wrapper's subject is the **entity node**, not the value's position in a
document: an object whose `id` is a nonempty string names `id` first *wherever it
appears* — inside a response's `objects` array and inside `changes[]` exactly as
in a stored entity file — and a value that is not an entity node stays all-lexical.
`78a7eb1` collapsed the two entry points into one wrapper, so the rule is
structural rather than a flag carried by the root; design §4.2 states it for the
slice, and
`tests/cli_contract.rs::a_projected_entity_is_serialised_by_the_same_wrapper_as_a_stored_one`
pins it against the spawned binary.