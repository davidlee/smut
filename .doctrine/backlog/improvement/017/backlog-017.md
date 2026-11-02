# IMP-017: An installed conventions copy goes stale silently — its version is stored and nothing compares it

Raised 2026-11-02 while designing the epistemic-records set (SL-006), out of a
discussion of "versioned stamps": should `smut conventions <name> --format json`
emit something that records *which* convention versions a store follows?

## What already exists

The install mechanism already puts a version stamp in the store. Verified on a
throwaway store, 0.2.0:

```
smut conventions hygiene --format json | smut declare
→ {"id":"conventions-hygiene","body":"# General hygiene\n…",
   "title":"Conventions: general hygiene","type":"convention","version":"0.2.0"}
```

Re-running the pipe answers `{"status":"unchanged"}`, so the entity is idempotent and
the version it was installed at is stable, readable and per-set. DEC-028 adds a
`source` field for layer file sets (not yet implemented — it lands with IMP-013).

So "which version of which set was installed here" is already answerable, one `get`
per set, without any new mechanism. What is *not* answerable is "**which sets does
this store have installed**": `smut conventions` with no argument lists the sets
*available* to load (the config side), not the ones in a store, and the only query
route is `{"id":{"$contains":"conventions-"}}` — the unsound substring filter IMP-011
measured. That is the census gap (IDE-003) at small scale, and it is the real
deficiency behind the stamp idea.

## The gap this item is about

Nothing compares the stored `version` with the version of the set the binary (or the
project's file) would install now. A store seeded from 0.2.0 keeps 0.2.0's advice
forever, and the difference between "advice we follow" and "advice we installed once"
is invisible. The version field exists and has no reader — which is exactly the
condition that makes a hand-written `vocabulary` stamp attractive, and exactly why
that stamp would be the wrong fix: it would be a second copy of a fact the store
already holds, with nothing to keep the two in step, which is the drift class the
conventions work exists to reduce (status versus edge, `answered` versus `answered_by`).

## What a fix would look like

- **Read what is there.** A check that compares each installed `conventions-<name>`'s
  `version` with the version the loader would produce now, and reports the sets that
  are behind. Exit zero, findings printed — the shape `doctor`/`validate` already take,
  and the shape that leaves DEC-017's posture untouched.
- **An enumeration to make it possible.** Either the same verb lists installed sets
  from the store, or `conventions` gains an explicit "installed" mode. Without one, the
  check cannot find what to compare.
- **Not this:** an emitter that writes a store-level registry of `{set: version}`
  entries. Installing a second set must then merge into an entity the first set owns,
  and `declare` replaces arrays wholesale with no read-modify-write — so the caller
  would have to read, merge and write, and skipping the merge fails silently. DEC-017's
  one-fixed-id-entity-per-set contract is what avoids that, and it should stand.

## Not decided

- Whether staleness is reported by a new verb, by `conventions` growing a mode, or as
  one more finding on an existing verb.
- Whether the report is a warning or only a printed line: nothing here should start
  refusing.
- Whether a stale copy matters at all when a project set (IMP-013) can shadow the
  binary's — a store deliberately pinned to a project file's older advice is a normal
  state, not a fault, and the check must not call it one.

## Neighbours

- DEC-017 (one set, fixed-id seeds, the store owns its copy, the binary never reads it back)
- DEC-028 (the set file format; `version` and `source` on seeds)
- IMP-013 (layered sets — the loader this check would compare against)
- IDE-003 (a store has no census — the same missing capability, one scale up)
- SL-006 (raised here; its `vocabulary` record is the hand-written version of this,
  and shrinks or disappears if this lands)
