# STD-002: Recovery replays a committed manifest before the loader validates it — every stored-validity rule that tightens lands with a version bump, or proves it cannot refuse an in-flight manifest

<!-- Body sections reuse the tuned prior art from spec-driver/supekku
     templates/standard-template.md; its YAML frontmatter is dropped — metadata
     lives in the sister standard.toml (the storage rule). -->

## Statement

Recovery runs under the lock and **strictly before** store validation
(`src/store/mod.rs` calls `recover()` then `load()`; `docs/store-format.md` § Recovery).
`validate_staged` (`src/store/journal.rs`) validates a committed manifest's *shape* —
format version, basenames, uniqueness, `puts`/`removes` disjointness, reserved names,
presence, regular files, no symlink, no unlisted entry — and never parses a payload.

So a staged put whose payload a stricter loader would refuse is published, the journal is
renamed `done` and cleared, and the *next* load refuses a store whose committed batch has
already been applied. The store then refuses every command until an entity file is
hand-edited, and the pre-crash content of that entity is already overwritten.

The rule, therefore:

> **A change that adds or tightens a stored-validity rule does one of two things: it
> bumps the format version, so a manifest an earlier build committed is refused by
> version before replay; or it shows that no manifest a prior build could have committed
> becomes refusable by the new rule.** The proof lives in the change's design or notes.

The version gate is the whole defence. Which of the two obligations a given change carries:

| The change | Review it when |
| --- | --- |
| the set of payloads the loader accepts | any validation, predicate, or normalisation rule is added, tightened or moved earlier in the load path |
| the format version and its refusal path | `FORMAT_VERSION` is bumped, or an existing version stops being readable |
| the pre-replay contract in `validate_staged` | anything is added to what recovery refuses before its first replay byte |

A change that tightens a stored-validity rule with neither a version bump nor a proof of
non-refusability is a defect in the change, not a follow-up.

## Rationale

**The two mechanisms disagree by design, and that is deliberate.** Validating before
recovery would refuse a crashed-but-recoverable store as corrupt — bricking a store with
its first interrupted command. So recovery must run first, and the loader's opinion of a
payload is therefore always formed *after* that payload has been published.

**The window widens monotonically.** Recovery never parses a payload, so no loader rule
narrows what recovery accepts: its acceptance set stays strictly larger than the loader's,
always. Every rule the loader learns retroactively makes more of the manifests an older
build could have committed refusable-after-replay. Slice by slice, in other words, the
number of ways to arrive at a published payload the loader refuses only ever increases.

**Nothing in the repository can observe the violation.** No test can fail on "a rule was
tightened without a version bump" — the diff has no shape for it. The only surfaces that
can carry the obligation are prose and process, so the trigger belongs in a title that
rides every agent's boot context, exactly as STD-001's does for the format document. This
standard exists because the defence here is process, and unenshrined process is not a
defence.

**Observed rather than hypothesised.** RV-024 F-2 raised this against SL-004, was closed
`tolerated` on measured likelihood, and left the invariant unenforced — RSK-001 recorded
that gap. The likelihood is low for the hand-edit path (a validating build cannot stage
malformed identity; a mid-staging crash lands in the `prepare` state, which is discarded;
a cross-version manifest is already refused by version). It is not low for the
rule-tightening path, which is the ordinary business of this codebase.

## Scope

- **Applies to:** every change to the store's load path — `src/store/`, `src/entity.rs`'s
  identity and normalisation rules, and anything that decides whether a given stored
  payload is acceptable. Includes changes made for reasons that have nothing to do with
  the format, which are exactly the ones that forget it.
- **Recommended for:** a slice whose design adds a rule about *stored* data without
  touching the format version — read the two obligations and pick one explicitly.
- **Excluded:** payloads the *request* boundary refuses (they never reach a staged put),
  and the `prepare` state, which is discarded rather than replayed.
- **Not excluded:** the case where the writer instead of the loader is wrong. A write set
  is built from entities that already passed the same predicate through the same
  serialiser, so that leg belongs to the write-set tests, not to this standard.

## Verification

No mechanical check exists, and none is proposed: the obligation is on the reviewer of a
change, not on the code. The verification is that the trigger is in the boot context and
in the format document's pre-replay contract, so a slice that tightens a rule meets it
before implementation rather than after a store refuses to open.

## References

- `docs/store-format.md` § Recovery and § Corruption — the ordering, its rationale, and
  the pre-replay contract.
- STD-001 — the format document is the format's only record. STD-001 sends a change to the
  document; this standard says when the document's version must move with it, or why it
  need not.
- RV-024 F-2 — the finding that raised this, closed `tolerated` with the invariant as its
  repair.
- RSK-001 — the unenforced-invariant gap this standard closes.
- SL-004 — the exemplar: version 1 → 2 is the version-bump obligation discharged, with the
  old-manifest refusal pinned pre-replay (PHASE-02/EX-7, VT-6).
