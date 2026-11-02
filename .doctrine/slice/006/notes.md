# SL-006 notes — the adversarial pass on the taxonomy

2026-11-02. IMP-004 recorded that four review passes on the earlier draft landed
*around* the taxonomy without attacking it. This is the pass that attacks it, run
against the drafted set (`.doctrine/slice/006/epistemic-records.md`) rather than the
grammar: the set text is what an author reads, so it is where the review earns its keep.

Verdicts: **fixed** (the text changed), **tolerated** (deliberate, with the reason),
**follow-up** (costed, needs a decision or a slice).

## Findings

**F-1 — `finding` and `evidence` were not separable.** Both are "I noticed something",
both carry `confidence` and `reported_by`, and nothing said when one is the other. An
author would pick by mood, and the two would drain into whichever word they met first —
which is how the one real store ended up with 18 findings and 3 observations for the
same job. **Fixed:** evidence names one datum and how it was got; a finding is a
conclusion no single datum carries.

**F-2 — a reader could not tell doctrine's vocabulary from this set's.** The doc claims
legibility ("a record written here would be a doctrine record"), but three kinds
(`finding`, `option`, `party`) and two things (`reliability`, `confidence` on a finding)
are additions, and nothing said so. An author moving the same records into doctrine
would discover the boundary by being refused. **Fixed:** one sentence naming exactly
what is this set's own and what is doctrine's.

**F-3 — `answered_by` silently redefines a doctrine field.** Doctrine's `answered_by`
holds a *person*; this set has it hold a *record*. That is the single place where the
legibility promise is broken, and it was broken without a word. **Fixed:** the rule now
says what changed and where the party went (`reported_by` on the answer).

**F-4 — `review_by` was prescribed as if it were queryable.** Rule 8 asked for a date
and said nothing about the fact that no range predicate exists, so an author would
reasonably assume "overdue" was a query. That is the silent-wrongness class RFC-001
ranked first. **Fixed:** one clause — it can be ordered, its oldest taken, the overdue
set cannot be selected.

**F-5 — the labels table let `part-of` and `about` be written to the same target.** The
worked example in `grammar.md` does exactly that (`q-schema` was both `part-of` and
`about` `ctx-query`), which is the one-fact-two-places defect the set spends a rule
forbidding. **Fixed in the set** with an explicit line; **fixed in `grammar.md`** by
removing the redundant `about` from the example.

**F-6 — the grammar's worked example inverted `supports`.** `opt-schema-in-binary`
carried `supports: [ev-loader-overhead]`, i.e. an option pointing *at* evidence, while
the label is defined evidence → record. An example that contradicts its own table is
worse than no example, because it is what people copy. **Fixed in `grammar.md`:**
`ev-loader-overhead` disputes the option, in the defined direction.

**F-7 — the seeded entity's `type` is `convention`, which is not one of the ten kinds.**
The install mechanism's own label collides with the set's vocabulary, and a reader
checking types would find a store that contradicts the set on day one. **Fixed:** the
sealing appendix says the seeded copy is machinery, its type is the mechanism's, and it
is not an epistemic record. **Follow-up:** the mechanism could seed `type: concept`, but
that changes DEC-017's emitted payload and gains nothing but tidiness — not worth it.

**F-8 — `party` `discredited` was marked terminal in the grammar and non-terminal in
the set.** Terminal is wrong: standing is a judgement that later evidence can revise,
which is the whole reason for keeping the party as a record rather than a string.
**Fixed in `grammar.md`** (bold removed from `discredited`).

**F-9 — the constraint `source` enum has no value for "our own store".** The draft this
set replaced had `scope: doctrine | smut | operator | external`, which at least admitted
a project; doctrine's enum has `canon`, `operator` and `external` but nothing for the
store itself. Doctrine's own answer is that the enum is closed, so the honest handling is
the one every other closed vocabulary gets. **Fixed:** the set says to add the value and
record it in `vocabulary`. **Tolerated:** not extending doctrine's enum in this set.

**F-10 — "the minimum record" reads as a requirement.** It is advice in a set that
cannot require anything, and three of its six clauses will be skipped by any hurried
author (`why_matters` most of all). **Tolerated:** the section header and the closing
clause say what it is, and softening it further would cost the only pressure the set
has.

**F-11 — the vocabulary record is derived data kept by hand**, which rule 9 otherwise
forbids. **Tolerated:** it is named as a hand-written census, says it will go stale, and
says why it exists (nothing can compute distinct values). **Follow-up:** IDE-003 (census)
and IMP-017 (installed copies going stale) are the two items that would retire it.

## What the pass did not find

- No kind that the ten cannot express, among the records this store's own history
  contains. The `answer` type the store used is *not* needed: an answering record is a
  finding, decision, evidence or concept that a question points at.
- No status doing the same job as another within a kind. The three duplicates the draft
  carried (`answered`, `active`, `recorded`) are gone and nothing has re-appeared in
  their place.
- No label that earned its place by assertion rather than by a query someone will ask.
  `addresses` is the weakest of the nine and survived only because options need a
  problem to be options *of*.

## Not done here

- No formal review ledger (`RV`) was raised for this pass: it is one agent's hostile
  read, recorded in the artefact it reviewed rather than as a scored ledger. If the set
  is to ship with a review trail, that ledger is the next artefact.
- No phase plan. SL-006's implementation is gated on SL-004 (`smut_id`) and IMP-013
  (the layer loader and the match arm), so a plan now would be a plan for a slice that
  cannot start.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · design complete, no phase (implementation gated) · head 920ce82 · no managed design run exists for SL-006

### Produced
- SL-006 — scope and decision record (`slice-006.md`): ten kinds, nine labels, rules 1–17, the gate table
- SL-006 — `grammar.md`: the reasoning, the doctrine accounting (§10: 30 of 32 facet fields cross unchanged), the alternatives dropped; marked superseded as the working artefact
- SL-006 — `epistemic-records.md`: the set text, 177 lines, written against `smut_id`; deliberately in the slice folder, not `src/`
- SL-006 — `notes.md`: the adversarial pass, eleven findings, three text-changing, two of them defects in this slice's own worked example
- IMP-017 — an installed conventions copy goes stale and nothing compares its version
- IDE-004 — no edge attributes; the three possible shapes and the four questions to decide first
- IDE-005 — a companion actors set; `party` migrates to it, this set keeps only the edge
- QUE-001 — first-instance evidence appended: has-it, in-substance-a-schema, plus the disposition shift and the on-demand-versus-on-write option space
- IMP-012 — measured `$contains` shape-dependence appended as a fifth "what the surface cannot do" case
- IMP-004 — state note pointing at SL-006, with the delta from its §A draft
- memory `mem_01a19aeaa10b75819dbe007f963f2cfa` — smut field-shape traps

### Learned
- SL-006 — the vocabulary is doctrine's, not the store's: 7 kinds adopted by name, 3 added (`finding`, `option`, `party`), 1 status added (`dropped`); `source`/`refutation` types, `refutes` edge and `scope` enum dropped
- SL-006 — `type` is a field, not identity, so promotion in place is expressible here and not in doctrine; therefore a kind prefix is a label and never a discriminator
- SL-006 — one shape per field, measured: a mixed field changes what `$contains` means per row (substring versus exact element, silently), defeats `$eq`, and makes `order` refuse the field
- SL-006 — an actor value is a `party` reference, never text; `confidence` (of the datum) and `reliability` (of the report) are distinct and compose
- SL-006 — the question names its answer, singular, because absence of an own field is a predicate and absence of an incoming reference is not; that inversion is what made the one real store duplicate the fact in `status`
- SL-006 — the installed seed (`conventions-<name>`, `version`) is already the version stamp; a store-level registry would need merge semantics `declare` does not have
- SL-006 — the seeded entity's `type: convention` is the mechanism's label, not one of the ten kinds
- SL-006 — doctrine's edge machinery (roles, degrees, descriptors) has nowhere to live in smut, so a role becomes a label or prose

### Open
- SL-006 — implementation is gated, in order: SL-004 (`smut_id`) → IMP-013 (layer loader + `conventions` match arm) → one commit that also makes the two `hygiene.md` edits
- SL-006 — no formal `RV` ledger; the adversarial pass is in-artefact. Owed if the set should ship with a review trail
- SL-006 — set length 177 lines against hygiene's 46; trim pass undecided (merge rules 5/6 and 7/9, cut statuses doctrine already names well)
- SL-006 — `concept-maps` decided as a companion set, six rules sketched in `grammar.md` §8, not drafted
- IMP-004 — preconditions still unmet: 0.2 unpublished (CHR-001), `hygiene` never used on a real store, IMP-013 unimplemented
