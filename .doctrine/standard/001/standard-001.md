# STD-001: docs/store-format.md is the store format's only record — review it whenever the format, its version, its identity key or its corruption contract changes

## Statement

`docs/store-format.md` is the single record of the on-disk store format. Any change
to the format is not complete until that document carries it, in the same change.
The four things it owns, and the trigger for each:

| The document owns | Review it when |
| --- | --- |
| the directory layout, reserved names and journal states | a file, directory or reserved name moves, appears or disappears |
| the format version and what it means | `FORMAT_VERSION` changes, or a store of a given version stops being readable |
| the identity key and canonical bytes (entity-first ordering, `smut_id`-first within an entity) | the identity key's spelling changes, an ordering rule changes, or the serialiser's shape changes |
| the corruption contract (what is refused, never repaired) | a validation is added or removed, or an exit-class boundary moves |

A change that lands code without the document is a defect in the change, not a
follow-up.

## Rationale

Two failure modes this exists to prevent, both observed in this repository:

- **Prose has no test.** Every other format-bearing surface is mechanically pinned —
  the identity constant is read by code, `smut learn`'s sentences are pinned by
  substring tests, the guide's examples and the README's transcript are *executed*
  against the built binary by `tests/fixture.rs` and `tests/readme_example.rs`.
  `docs/store-format.md` is read by nothing. It was written once, in one slice's
  voice, and drifted (RV-021 F-5) precisely because no gate held it.
- **The format is the thing callers cannot re-derive.** A wrong `smut learn`
  sentence is caught by running the command; a wrong store document is caught by
  someone implementing a reader against it. Its audience is a future implementer, and
  that implementer is usually an agent with no way to tell a stale sentence from a
  current one.

The title is the enforcement: doctrine loads standard titles into every agent's boot
context, so the trigger is in front of an agent before it touches `src/store/`.

## Scope

- **Applies to** any change under `src/store/`, and any change that moves the
  identity key, the envelope's entity serialisation, the format version, or the
  refusal classes — including a change that only *reads* the format differently.
- **Includes** the document's worked diagnostics: the example error messages are part
  of the contract and are quoted by callers.
- **Excludes** the human-facing command documents — `smut learn` and the README — which
  are pinned by execution and by prose tests instead, and `smut-implementation-brief.org`,
  which is an input to a release rather than a record of the format.

## Verification

No mechanism asserts this; it is a review obligation, not a lint rule, and it is
discharged at two points:

- **The slice's audit**, where the store document is a `scope-relevant` selector
  whenever `src/store/` is in play, so `slice conformance` shows it as delivered or
  undelivered.
- **Where the change is a format break**, the document moves in the same batch as the
  version bump, because a reader of the store has no other way to learn that the old
  version is refused.

A change that should have moved the document and did not is a review finding, and the
existing code-review conventions already carry it as one.

## References

- DEC-020 — where the identity key lives and which surface each pin holds, including
  why this document is governed by a standard rather than a test.
- ADR-001 — identity is `smut_id`; the format break this standard's first application
  covers.
- DEC-010 — the documentation set (README, versioned store format, `learn`), which this
  standard refines for the store document specifically.
- SL-004 — the slice that moves the identity key and the format version.
- RV-021 F-5, RV-022 F-3 — the two reviews that found the store document and the loader
  drifting apart from the code.
