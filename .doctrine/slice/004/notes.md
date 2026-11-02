# Notes SL-004: Identity is smut_id — the format break, and a loader that refuses what it cannot trust

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · planning · 540cf0b

### Produced
- SL-004 — six-section materialised design and code-impact selectors
- RV-023 — design self-review and verification limits
- SL-004 — implementation plan and phase boundaries

### Learned
- DEC-020 — identity constant and per-surface pins
- DEC-021 — eager stored-identity validation
- DEC-022 — clean version refusal without compatibility handling
- DEC-023 — normative brief updated in place
- DEC-024 — fixture equivalence and semantic residual review
- DEC-025 — targeted amendments of six identity-bearing decisions

### Open
- SL-004 — formal audit and the six-decision reconciliation handoff
- SL-004 — coordinate the operator's live README during execution

## Working mode (operator)

Inquiry is taken **one question at a time**: the agent drafts the answer, the
operator nods or pushes back, and only then is it recorded in the run. Do not
answer two nodes in one turn, and do not record an answer the operator has not
seen. The operator also reshapes the tree freely — "just reshape the tree as you
need to".

`graph-reviewed` was recorded at rev 8 and invalidated when the inquiry graph was
declared at rev 9, as designed. Both it and `sufficiency-accepted` need the
operator's assent again once the graph is final — lay out what each is assenting to
before asking.

## inq-2 — loader validation of nested identity (IMP-005)

Recorded as DEC-021 in run revision 12, after the operator replied "agree" to
the presented eager-loader proposal. This assent covers inq-2 alone.

- **Where.** `Store::load`'s entity-file path (`src/store/mod.rs:218-238`), after the
  existing is-an-object and root-identity checks. The file is already parsed there;
  validation adds a linear walk over the parsed value, not another read or parse.
- **What.** A recursive walk over the loaded entity establishing exactly what
  `reference_id` already decides, so one rule and not two: any object carrying
  `smut_id` must carry a nonempty string there. A payload's ordinary `id` is never
  inspected — after this change it is data.
- **Failure shape.** `StoreError::Corrupt { entry, reason }`, exit 1, naming the file
  and the offending path (e.g. `payload.tags[2].smut_id`). Corrupt rather than a new
  variant: the store's contract is that a store it did not write is refused, never
  repaired, and `Corrupt` is that contract's spelling. Consistent with the
  failure-taxonomy memory: internal invariants assert, user data is refused, and a
  malformed store is user data.
- **Why the loader, not `scan_refs`.** Making `scan_refs` return `Result` threads an
  error through every traversal call site for a condition that must not exist in a
  loaded store, and converts corruption into a mid-query failure with partial results
  already built. `scan_refs` keeps its non-fallible validated-precondition signature;
  its panic stays as an assertion and stops being reachable from a loaded store.
- **Accepted trade.** The walk is
  eager, once per entity at load. Trivial for the 66-entity fixture; the same order as
  parsing for a large store. The alternative is lazy validation at `scan_refs` call
  sites — the operator chose eager validation at load over threading Result through
  traversal.

## inq-3 — old-format refusal

DEC-022 records the operator's agreement to the existing found/expected version
diagnostic and exit 1, with the corresponding manifest version check before
replay. No migration advice or compatibility layer. The operator's rationale:
"there are essentially no legacy consumers i care about, so fail clean and loud."
Their download-count recollection and bot estimate are context, not verified
adoption measurements or acceptance prerequisites. The format document moves
under STD-001. Graph review and inquiry sufficiency still require separate assent.

## inq-4 — brief remains normative

The operator asked how much of the brief remains before deciding whether to mark
it historical. Source inspection: core declarations/patch/storage/get/delete,
query predicates/traversal and Markdown rendering are implemented; Mermaid is
explicitly refused in src/cli.rs. IMP-002 still owns Mermaid, cross-format display
flag behavior/ignored-flag documentation and a generated-store smoke benchmark.
Markdown flags and visible incompleteness already exist; portability limits are
already described in README.md and docs/store-format.md, so those backlog bullets
need reconciliation rather than assuming they are wholly unimplemented.

The brief's full Mermaid end-to-end example therefore remains unfulfilled.
SL-001 explicitly keeps the brief authoritative across all three releases and
defers extracting a product/technical spec until they ship. Recommendation to
mark the whole brief historical was premature: prefer keeping it live and
annotating the identity/version changes and remaining delivery scope. The operator
agreed to updating the brief in place rather than extracting specs, to retain one
clear normative truth document. DEC-023 records this choice at run revision 14;
the documentation edits follow the design and plan. No full conformance test run
was performed for this coverage check.

## inq-5 — verify the targeted identity rename

DEC-024 records the operator's agreement at run revision 15: targeted structural
identity-key replacement; preserve id values, ordinary payload id and container
names; compare the fixture after reversing only the key rename with its original
parsed data; add ordinary-id versus smut_id behavior checks and run the existing
fixture/example/regression suites; classify residual id occurrences by meaning.
A blanket literal ban is invalid because ordinary id becomes legitimate data.

## inq-6 — targeted decision amendments

DEC-025 records the operator's "yes" at run revision 16: amend the affected
passages of DEC-005/007/010/016/002/012 in place during reconciliation, with dated
ADR-001/SL-004 notes. Preserve unrelated rationale and durable ids; explicitly
change DEC-005's reservation consequence rather than merely respelling examples.

All six inquiry answers have accepted records DEC-020 through DEC-025 shaping
SL-004; inquire.knowledge was discharged at revision 17. The scope was aligned
with those accepted answers, including the normative brief, eager loader
validation, old-format refusal, fixture equivalence and legitimate payload id.
The operator reviewed the exact revision-18 tree and replied "reviewed";
graph-reviewed was recorded at revision 19. Inquiry sufficiency still needs its
own presented summary and explicit assent; none is inferred from the individual
answer approvals or graph review.

The operator subsequently replied "proceed" to the sufficiency summary;
sufficiency-accepted was recorded at revision 20 and the run advanced to drafting
at revision 21. Section 1 (change and boundaries) is drafted through the run and
materialised for human review. The design workflow's fresh-context handover
option was offered before the drafting-ready act. The operator directed continued
drafting and batch approval at the end. No section-review assent or final design
acceptance is inferred from inquiry sufficiency.

The operator replied "yes" to section 1; att-1 records that review. Section 2
(identity rules and wire contract) is drafted and materialised for review.
Operational correction: submitting att-1 with top-level `acceptance` at revision
24 also caused the CLI to emit a design-accepted act, although the recorded basis
explicitly covered section 1 only. This is not operator assent to the whole
design. The subsequent section-2 amendment invalidates that whole-design act;
require fresh explicit whole-design assent at the actual acceptance gate. The
explicit section-reviewed checkpoint plus attestation was subsequently refused
because the checkpoint act carries no per-section coverage. The operator then
directed: "sure, we can batch approve at the end. proceed". Leave section 2 through
section 6 attestations pending for that final explicit approval; do not retry the
unsupported checkpoint or infer whole-design acceptance early.

## Draft and review packet

Sections 1–6 are materialised: boundaries, wire identity, loader/version refusal,
code/documentation ownership, verification, and risks. The design-target registry
mirrors section 4 (20 selectors, 44 tracked paths after expansion). The agent
readiness act and selector discharge are recorded; the run entered reviewing at
revision 34 and opened RV-023. Section 1 retains its earlier human attestation;
all remaining section attestations and final design acceptance await the operator.

RV-023's self-review concluded with no verified findings. It is not an independent
review or runtime verification. A further adversarial pass would most usefully
probe diagnostic-path escaping/determinism, the distinction between malformed
structural identity and ordinary payload id, replay-before-load boundaries,
fixture-comparison discrimination, and whether the normative brief plus both
facet/prose halves of the amended decisions can drift. This statement was written
after that pass. Offer an inquisition or an external review prompt as alternatives
to proceeding to planning on the self-review plus the operator's review.

## Operational hazards

## Design approval and planning

The operator replied "approve, lock & plan" to the completed design and RV-023
packet. The review disposition was recorded as conducted at revision 38 and all
six sections plus final design acceptance at revision 39; the run is locked.
The lifecycle moved to plan. This is explicit batch assent, replacing the earlier
invalid premature whole-design act; no additional independent review is claimed.

Planning rechecked the source symbols, both version gates, dependency direction,
current scope and all six decisions. Research was refreshed for STD-001 and the
accepted brief treatment; code premises remain unchanged. The planned order is
loader validation under the shared key, the coherent format/identity sweep, then
integrated evidence and amendment proposals for the formal audit/reconcile seam.
Accepted decision edits remain with reconciliation rather than phase execution.
The plan TOML parses; every VT has an existing file and nonempty keyword mandate.
These checks validate planning structure, not implementation behavior.

- **The operator's README draft is staged and uncommitted.** A bare `git commit`
  sweeps it into the commit; this happened twice and was repaired twice. Always
  commit with an explicit pathspec (`git commit ... -- .doctrine`).
- Minting a standard is the chosen mechanism for a prose surface that no test can
  hold: its title rides every agent's boot context (STD-001 is the first).
