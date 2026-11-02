# Implementation Plan SL-004: Identity is smut_id — the format break, and a loader that refuses what it cannot trust

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Implement the locked identity design without changing graph meaning or bypassing
the audit/reconcile seam. The authored phase objectives and criteria live in
`plan.toml`; this companion explains why the boundaries are chosen. ADR-001,
STD-001 and DEC-020 through DEC-025 supply the accepted constraints.

## Sequencing & Rationale

Establish the loader invariant first, against the current `ENTITY_ID`, so the new
recursive algorithm and its diagnostic-path tests are isolated from the larger
spelling sweep. The helper's *code* is key-agnostic, but its verdict is not:
`reference_id` rejects a nested `{"id":null}` before the rename and accepts it as
ordinary data after it, so this phase's tests and expectations are key-dependent
and PHASE-02 respells them. The corruption document therefore lands with PHASE-02
rather than here — a format document that still declares version 1 must not state
a rule the released version-1 reader never enforced. This intermediate commit
retains the old key and version; it is not a release, not a supported reader of
any existing version-1 store, and not a reinterpretation of the final design.

Then move structural identity, both version constants, every current data/example
surface and the committed fixture together. This is deliberately a wider phase:
splitting source, tests, guide and versioned format prose across independently
green commits would either leave a broken tree or document a format the code does
not read. Within the phase, preserve the original fixture before any transformation,
write new ordinary-id behavior cases first, adapt structural sites by meaning and
run focused checks before the complete gate. No global text replacement is safe.

The final implementation phase independently assembles the integrated evidence
and proposed decision amendments. It prepares the audit handoff; it does not edit
accepted decisions ahead of reconciliation. The audit's own RV and reconciliation
brief route those amendments to `/reconcile`, where their facet and prose changes
land before slice closure. This preserves design section 4's explicit timing.

Phase completion does not mean release readiness or formal closure. The code,
current documents and fixture can be verified before audit while the six accepted
decision amendments remain explicit reconciliation obligations.

## Verification choices

Behavioral tests establish what input and output mean. Structured VT mandates
provide source-presence signals and must not be reported as test execution.
Agent verification owns the original-fixture comparison, negative control,
document review, residual classification, independent scratch-store probes and
final gate results. Those require judgment or cross-artifact evidence that a raw
substring check cannot provide.

Fixture equivalence compares inverse-renamed structural keys with a recorded Git
original using `serde_json` equality, retaining numeric distinctions and array
order. Do not commit a second fixture or make installed builds depend on Git.
Record enough of the one-time verification command to repeat it at audit.

Version tests must construct old version values explicitly: changing both a
fixture's version and the reader constant can otherwise make an old-format test
quietly stop testing refusal. Recovery tests must inspect staged publication and
removal effects, not merely an error string or unchanged in-memory result.
Pair the old-manifest refusal fixture with an otherwise equivalent readable
manifest that actually publishes/removes its staged targets. Keep the underlying
store readable in both cases, so an unrelated index-version error or a fixture
that never reaches recovery cannot satisfy the refusal requirement vacuously.
Inspect actual versioned index/manifest bytes as well as their constants.

## Notes

The operator's README prose is being rewritten for the 0.2 release and lands outside
this slice, before `cargo publish` (CHR-001). No phase sweeps or stages it. Its single
exact-fenced console block is in scope and gates `just check` (`tests/readme_example.rs`
executes it), so it carries `smut_id` before the gate runs; preserve both of the
operator's layers and never use
a broad commit that absorbs unrelated work. Plan/design commits explicitly name the
Doctrine paths and exclude README.

The loader pass is linear over already-parsed values. Use one mutable path buffer
or equivalent bounded traversal state rather than repeatedly cloning ancestor
paths or subtrees. JSON-quoted path components are diagnostic notation, not a
query-language extension.

The final residual sweep permits ordinary payload `id`, ids-as-values, internal
identifier names, historical fixture quotations and explicit old-version refusal
fixtures. During implementation the six older decisions' structural uses remain
classified as pending reconciliation, not silently counted as already fixed.

No implementation dependency is added. An unexpected need for a new dependency,
different error class, new identity shape or changed recovery protocol returns
to design instead of being hidden inside an execution task.
