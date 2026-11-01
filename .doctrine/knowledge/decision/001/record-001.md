# Boundary contract — 0.1 pure core / imperative shell

Settled with the operator on 2026-11-01 (approved A and B, amended in five
particulars). This is the load-bearing shape of the whole design: everything
else in 0.1 is either above this line (semantics) or below it (persistence).

## The core's surface

```rust
type Entities = BTreeMap<Id, Value>;          // validated snapshot, public ids only

fn normalize(input: &Value, ids: &mut dyn IdSource) -> Result<Batch, Reject>;
fn apply(store: &Entities, batch: Batch) -> Result<Outcome, Reject>;
fn plan_delete(store: &Entities, ids: &[Id]) -> (Response, WriteSet);
fn parse_spec(input: &Value) -> Result<Spec, Reject>;
fn select<'a>(store: &'a Entities, spec: &Spec) -> Result<Selection<'a>, Reject>;
fn render_md(selection: &Selection<'_>, opts: &RenderOpts) -> String;
```

`Outcome { response, write_set }` — and nothing else. The core does not return a
second complete snapshot; no consumer needs one.

## What crosses, in both directions

| Crossing | It is | It is not |
| --- | --- | --- |
| in | `Entities` — every entity, keyed by public id | a store handle, index, uid or path |
| in | `Value` — an already-parsed request | bytes, or anything unparsed |
| in | `IdSource` — injected id allocation | an ambient RNG call |
| out | `WriteSet { put: Vec<Value>, remove: Vec<Id> }` | filesystem operations |
| out | `Selection<'a>` — borrowed full entities + ordered id list + field set + diagnostics | pre-projected owned objects |
| out | `String` — rendered text | stderr, logging |
| out | `Reject` | any storage or corruption failure |

## The rules

1. **User-input rejection is exit 2, whichever layer detects it.** The shell
   rejects malformed JSON, duplicate JSON keys and invalid CLI arguments — all
   exit 2. Storage or runtime failure is exit 1. `Reject` from the core is
   always 2, because the core is incapable of expressing anything but user
   error. The invariant is stated over the *class* of failure, not over which
   layer noticed it.
2. **The shell validates storage structure and identity, but never interprets
   patch, relationship or query semantics.** It checks that indexed ids match
   stored entity ids, paths stay inside `.smut`, the format version is known and
   the index and files agree. The journal record stays entirely content-blind:
   paths, bytes and a delete list.
3. **`IdSource` has an explicit collision contract.** A generated id avoids:
   (a) existing entity ids in the snapshot, (b) every explicit id anywhere in
   the request, including forward references, (c) every other id generated for
   the same batch. The core pre-scans explicit ids before generating, then draws
   candidates until one is unseen. A generated collision must never become an
   accidental upsert. Corpus-wide this makes `normalize` *deterministic with a
   supplied id source* — not strictly pure.
4. **`get` is not a defaulted query specification.** It reuses the selection
   engine through an internal constructor that (a) disables traversal and
   (b) bypasses the 100-result default. Passing a normal defaulted spec would
   silently truncate a `get` of 150 ids.
5. **`Selection` borrows; projection is downstream.** The selection carries the
   ordered ids and produces full entities by reference. Field projection happens
   after selection, so hidden reference fields remain available and 0.2's
   Mermaid edges are not lost to `select`.
6. **The write set is in public-ID space.** The shell resolves id→uid through
   the index, allocating a fresh uid for an unseen id, collision-checked against
   the filesystem. The core is name-blind; making it uid-aware would make it a
   function of storage layout rather than of entities.
7. **One reference predicate, defined once** (`is_reference`), used by
   declaration and — in 0.2 — by traversal. Central semantic definition, no
   extensibility framework around it.
8. **`get` is `select`** with an explicit `from` (see rule 4); `delete` shares
   the dedupe-in-first-occurrence-order rule. One pipeline, one ordering
   authority.

## Consequences

- The core's tests need no filesystem; the shell's interruption tests need no
  patch semantics. Each subsystem is independently testable.
- Independently testable is not the same as independently *correct*: a small set
  of integration tests must prove `normalize → write-set translation → commit →
  reload` preserves the same entities. The seam itself is untested by either
  side's unit tests.
- Store-not-found is exit 1 (an absent state, not invalid input).

## Alternatives considered

- Uid-keyed write sets — rejected: forces the index into the core.
- A flattened `Error` enum shared by core and shell — rejected: the core could
  then represent a storage failure, and the semantics tests would grow a
  filesystem.
- Returning the post-batch snapshot to the shell — rejected: no consumer.
- Cloning entities into `Selection` — rejected: ownership cost with no benefit,
  and it invites the projection-inside-selection fusion.
