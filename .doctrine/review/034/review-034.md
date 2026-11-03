# Review RV-034 — reconciliation of SL-010

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-010 *A humane query surface* at `8386aeb` (the handover commit), against
design r35 (locked), the four phase criteria in `plan.toml`, and the governance the
slice named for itself in `notes.md` § Design surface triage.

**What this audit probes.** Not whether the twelve flags exist — the suites say they
do, and `verify-vt 10` reports every VT PASS. It probes the three things a green suite
cannot see:

1. **Whether a refusal sentence is true of the line it answers.** The surface's whole
   claim is that every refusal names what the caller typed and never invents a field
   (DEC-030). A sentence can be pinned, rendered whole, exit 2, and still be false
   about the caller's own words. RV-031 found that class in the contradiction voice;
   this audit looks for its siblings in the shape refusals and the flag refusals.
2. **Whether the recorded evidence is what the notes say it is.** `slice conformance`
   reads recorded deltas, so a boundary recorded with the wrong end silently moves
   attribution from "foreign commit" to "slice span" without reddening anything.
3. **Whether the prose the design owns survives being read by its reader.** The
   guide is the delivery channel (DEC-010); a sentence can be design-faithful,
   test-faithful and still refuted by the next paragraph of the same section.

**The invariants it holds the slice to.**

- `q` adds no semantics: `compile` → `spec::parse` → `Request::Query`, one engine
  (DEC-038). Byte identity for the traversing recipes; selected-set agreement for the
  filtering pair, and no claim wider than that (RV-031 F-6).
- The front-end is stricter or silent, never different (design §3).
- Every refusal exits 2, is one line, names what was typed, invents no field or flag
  (DEC-030), and selection is deterministic (EX-10).
- Nothing lands inside `.smut/`; no stored-validity rule moves (STD-001, STD-002).
- `README.md` is operator-owned (CHR-001) and is not restaged.
- Design §6's unchanged list is a claim with a census: one base-diff, not nine
  per-file checks (VA-1).

**Where the bodies were expected.** The deferred residuals the seat handed to audit
(`--from=`'s sentence, `--edges --depth 2`, the F1 numeric ruling), the guide's "no
flags" sentence, and the conformance undeclared cell — read as metadata plus the
operator's `c8c2e7b` unless a recorded delta says otherwise.

## Synthesis

SL-010 delivers what it designed. Four phases, every VT green, `q` reaching the
specification through the existing ingress rather than beside it, byte identity pinned
where it holds and selected-set agreement where it does not, no store opened before the
specification validates, nothing in design §6's unchanged list moved, and nothing landed
inside `.smut/`. Nine findings, none against behaviour. Eight of the nine are the same
failure at different altitudes: **a sentence that was true of the case it was written
for, generalised to cases where it is false.** That is RV-031 F-2's class, and the fact
that it recurred one layer down — in the refusal catalogue and in the guide, rather than
in the design's merge rules — is the standing risk this audit records, not a defect it
invented.

**What the audit found that the suite could not.** A refusal can be pinned, rendered
whole, exit 2, and still be false about the caller's own words: `smut q 'url=a=b'` tells
a reader their *field name* holds an `=` when it is their URL. Three more sentences
(`--from=`'s comma list, design §2's blanket unsatisfiability, the guide's "no flags")
and one reachability row (§3/§7's numeric precondition) are the same shape. The suites
held the shape of every message and could not hold its truth — which is what a
whole-line pin buys, and what it cannot buy.

**The one evidence defect, settled during the audit.** PHASE-03's recorded boundary
ended at `108d1c2`, six commits past its own, importing the operator's foreign `c8c2e7b`
and both IDE-007 backlog commits into the phase span. Nothing reddened: `conformance`
reads recorded deltas, so the cell simply read 14 undeclared instead of 4. Re-recorded
as `f26f923..cd69acd`, conformance returns 8 conformant / 0 undelivered / 4 undeclared
and `verify-vt` stays green. The handover's premise was true of the intent and false of
the record; the audit's job was to check the record rather than inherit the intent.

**Tradeoffs knowingly accepted.** No blocker was raised. Every finding is either
prose-and-pins, paid at reconcile in one pass, or already remediated in place, so a
blocker would have been ceremony over a gate nothing is trying to dodge. That is not a
licence to ship a false sentence: the brief below names the exact sites, and reconcile
pays them before `/close`. Residual #2 (`--edges --depth 2`) closes as covered rather
than as drift; the four undeclared paths are design §6's own exclusion; `q`'s silence on
an unknown field and on an empty line are decisions (DEC-027, and design §3's
supersession of DEC-037), re-confirmed rather than re-litigated.

**Standing risks carried out of the slice.** (1) The refusal catalogue is guarded by
whole-line pins but not by truth — a pin records the sentence someone chose. (2) IMP-014's
missing census and IMP-016's third operator table are now wider, and both are annotated
at reconcile rather than fixed. (3) A recorded boundary is evidence, and `record-delta
--commit`'s contract (exactly `[S^, S]`) is what makes a range noticeable; the next audit
should read a range as the signal it is.

**The human's route through it, in four stops.** `src/dsl.rs` `compile` is the whole
surface — tokens in source order, then the twelve flags in the design table's order, into
a `BTreeMap`-ordered `Where` and a JSON object; `src/cli.rs`'s `Sub::Q` arm compiles it and
hands the `Spec` to the same `Request::Query` the machine verb builds — design §2's
structural seam, and the reason "adds no semantics" is a fact about the call graph rather
than a promise. `tests/cli_contract.rs`'s byte-identity pair, each paired with an
assertion on the returned ids, is the one-engine proof; `tests/dsl.rs`'s refusal census is
the message contract, whole line by whole line. The remaining risk worth your attention is
not in the code: the refusal vocabulary is now six variants wide with no census over
`Reject` (IMP-014), so the next variant lands unpinned unless someone remembers —
`doctrine show IMP-014` is the item.

Design decisions this rests on, if you want them at source: `doctrine show DEC-038` (the
emit-for-`spec::parse` seam: one engine, structurally), `doctrine show DEC-030` (refusals
name what was typed), `doctrine show DEC-040` (what the guide may teach).

## Reconciliation Brief

Twelve items across four surfaces. Ordered so that no surface is written before the one
that authors it: **design prose first**, then the code mirrors that copy it, then
governance.

> **Do not run `doctrine design resume 10`.** The design run is locked at revision 35 and
> these corrections are prose owed *to* it; resuming would treat them as design work and
> reopen a run that is closed. Pay the prose by direct edit to
> `.doctrine/slice/010/design.md`.

### 1. Per-slice (direct edit — `design.md`)

Six corrections. Five are already recorded in `plan.md` § Corrections owed; the sixth is
F-4, which this audit promotes from candidate to owed.

- **§3, the field-name row (F-1)** — the row asserts "a field name holding = is outside
  q's grammar" for every repeated-operator token, which is false when the second operator
  is in the value (`url=a=b`). Rewrite so it is true of both halves and names neither as
  the cause; keep one line, the quoted token, and the `smut query` pointer (DEC-030).
  Mirror: §1's and §5's "a field name holding =, ~ or ," lists gain the value case.
- **§3, a row for `--from=` (F-3)** — the escape is one id, so an empty one is an empty
  id, not an empty list member. Cite the sentence the code now speaks.
- **§2, bullet 1 (F-2)** — two `=` are unsatisfiable when both sides are single values;
  a comma list beside a single value is satisfiable and is refused because the condition
  object carries one `$eq` per field. Fold the list-vs-single case into bullet 2's
  "two comma lists".
- **§1, "The hot path therefore has no flags" (F-4, sixth correction)** — false of the
  two example lines printed directly beneath it. Reword to claim the token half, keeping
  the shell-quote claim the next sentence already makes.
- **§3's `--depth=-1`/`--depth abc` rows and §7's `--depth abc` row (F-5)** — the
  specification's sentence is reached only with a direction present. Fifth correction as
  recorded; this audit adds that **§7's row** needs it too, not only §3's.
- **§3's row for a comma list on `~`**, and **§3's discrimination sentence for two `=` on
  one field** — corrections 2 and 3 as already recorded in `plan.md`; unchanged.

Off-surface: `plan.toml`'s `EN-/EX-/VT-` and `PHASE-NN` ids are immutable-append. F-3
leaves EX-6 satisfied as written. Nothing in this brief edits a phase criterion.

### 2. Code mirrors (`fix-now` — source and tests, this unit)

Operator's ruling: (a) fix-now, so canon and code tell the same truth at close. Each
mirror copies the prose authored above; each is in a path this slice already owns as a
`design-target`, so conformance is unaffected.

- `src/dsl.rs` `ambiguous` (299-306) — the same-operator reason, per §3's rewritten row (F-1).
- `src/dsl.rs`'s `--from` arm (~200) — a reason naming the flag and its one-id nature (F-3).
- `src/learn.rs` 110-113 — the "no flags" sentence (F-4).
- `src/learn.rs` 141-143 — the unsayable list gains the value case (F-1).
- `tests/dsl.rs` 377-396 and 603-606 — the two pins carrying `a=b=c`'s sentence (F-1).
- `tests/dsl.rs` 596-599 — the `--from=` pin (F-3).
- Optional, and the better shape if taken: one whole-sentence pin inside the short-form
  span for the reworded §1 claim, so the guide cannot drift back (F-4). Not a substring.
- Then `just check` (build, clippy in both feature configurations, fmt, tests in both).
  **Path-scoped `git add` only** — SL-008 shares this worktree and is in audit itself.

### 3. Governance/spec (REV)

- **DEC-016** — amend to name the DSL spelling of the two-call recipe. The guide prints
  and `tests/fixture.rs` executes both, so the decision is the only place the short form
  is absent.
- **DEC-035** — narrow "a JSON scalar" to exclude JSON *string* literals, with design §2's
  double-unquoting reason.
- **DEC-037** — supersede the consequence "the no-query refusal belongs to this layer":
  the layer would own it, and design §3 decides not to have one.
- **DEC-039** — correct the `status=open status!=resolved` illustration; it is satisfiable
  and answered at exit 0. The decision's rule stands; only that sentence is wrong.
- **IMP-014** — annotate: six new `Reject` variants landed with no census to catch a
  missing one; the whole-line message test is the guard.
- **IMP-016** — annotate: `OPERATORS` in `src/dsl.rs` is the third spelling of the five
  predicate operators.

### 4. Operator-owned (`README.md`, CHR-001)

- Pay the `q` section from `.doctrine/slice/010/readme-examples.md` — placement between
  `## Worked example` and `## Commands`, the missing `smut q` row in the Commands table,
  and the one sentence in the specification-keys paragraph. Pay it; do not restage the
  file.
- The material's own limits hold: byte identity is claimed only for the traversing recipes
  and `--ids`; the filtering pair agrees on the selected set, not on order.

### Already paid in this audit

- **F-7** — PHASE-03's boundary re-recorded as `f26f923..cd69acd`; conformance 8/0/4
  verified, `verify-vt` re-verified green. Nothing left for reconcile.
- **F-8, F-9** — aligned; recorded so closure inherits the reading.

## Reconciliation Outcome

Twelve design edits, six code mirrors, four decision amendments, two backlog annotations, one
README section. Written in the order the brief set — prose first, mirrors copied from it — so no
surface authored a second wording. Commits `780903b` (design + mirrors) and `ce7fd82` (decisions,
backlog, README, REV-003).

### Erratum on this brief

The brief above says "six corrections" and then lists seven. The authoritative inventory is
**twelve design edits**, and all twelve were paid. The count was loose prose on the audit's side,
not a shortfall: `plan.md`'s five recorded corrections (incorporated by reference) + this audit's
F-1, F-2, F-3 and F-4 + three sites the recorded set named only in general terms (§2's `Flags` doc
comment, §3's second unsatisfiability sentence, §7's row). No finding was skipped, and nothing
outside the findings was written — the extra sites are the same findings at their remaining
addresses.

### Direct edits — `.doctrine/slice/010/design.md`, in place

The design run was **not** resumed; the prose was edited directly, which is what the audit's trap
list and the run's locked status require.

- **§1** — "the hot path therefore has no flags" became "the tokens need no flags and nothing the
  grammar invents needs a quote" (F-4); the unsayable list gained a *value* holding `=`, `~` or `,`
  (F-1's mirror).
- **§2** — two `=` are unsatisfiable *when both values are single values* (F-2); a second `=`
  carrying a comma list joins the caller's-words rule beside two comma lists (F-2); the `Flags` doc
  comment carries the direction precondition (F-5).
- **§3** — the field-name row became "a token carrying the same operator twice", true of `a=b=c`
  and of `url=a=b`; a `--from=` row and a `title~a,b` row were added; the discrimination sentence
  now says which branch fires on which kind (F-1, F-3, corrections 2 and 3); the specification-refusal
  rows carry a direction, with the sentence that says why (F-5); the two-`=` paragraph no longer
  calls every repeat unsatisfiable (F-2).
- **§4** — the struct block's `#[arg(value_name = "TOKEN")]` is deleted, replaced by the doc line
  that says why the positional is bare (correction 1).
- **§7** — `--depth abc --follow any` for the specification's sentence, plus a row for the
  directionless `--depth abc` the front-end answers first (F-5).

### Code mirrors — the same sentences, copied

`src/dsl.rs` (`ambiguous`'s reason and its doc comment, the `--from` arm, `Flags`'s doc comment),
`src/learn.rs` (110-113 and 141-143), `tests/dsl.rs` (the `a=b=c` pin, a new `url=a=b` case, the
`--from=` pin moved out of the list-flag loop into the census), and one whole-sentence pin in
`src/learn.rs`'s tests for the reworded claim. `just check` exit 0, every configuration.

Two mutation probes, each red then restored: the old same-operator reason reddened two tests, the
old `--from` reason reddened one. The pins bite the code, not just the prose.

### Governance — no revision was needed, and none was used

REV-003 `reconcile-sl-010` was created and then **abandoned empty**:

```
Error: `revises` target must be one of [SPEC, PRD, REQ, ADR, POL, STD], got a DEC
```

That is not a missing finding; it corrects the *surface* the brief named. A decision is a knowledge
record, not governance/spec truth, so no REV can carry one. Nothing else qualified either: no ADR
(ADR-001 stands untouched), no policy, no standard (STD-001/STD-002 are quiet — nothing landed in
`.smut/`, no stored-validity rule moved), and no spec or requirement (the slice declares no spec
targets). REV-003's rationale quotes the refusal verbatim so the next pass does not repeat it.

- **DEC-016** — the recipe gained its short-form spelling beside each JSON line; the JSON lines,
  the direction sentence and the completeness guard are unchanged, and both spellings are executed.
- **DEC-035** — the body states the JSON-string-literal exclusion and its double-unquoting reason;
  the `choice` facet already enumerated boolean/null/number, so only that sentence was owed.
- **DEC-037** — the consequence "the no-query refusal belongs to this layer" is superseded:
  `smut q` alone is `query {}`, so there is deliberately no refusal. Facet and body.
- **DEC-039** — the illustration is corrected: `status=open status!=resolved` is **answered**;
  `status=open status!=open` is the pair that reaches the pinned sentence, and it comes from
  `predicate::parse`. `choice` and `rationale` now say "single values", and the body carries the
  comma-list half too.
- **IMP-014** — annotated: six variants landed, the census gap is real, and the audit found the
  converse hole a census would not close (a row can be pinned and still be false).
- **IMP-016** — annotated: `src/dsl.rs` is the third site, deliberately not derived from the other
  two, so a sixth operator is now a three-site change.

### Operator-owned — `README.md` (CHR-001), paid not restaged

- New `## Querying, quickly` between the worked example and `## Commands`: three compact declares,
  the point (line and JSON twin, same bytes), substring and list, the direction pair, then the two
  limits with the JSON that does what `q` cannot.
- The `Commands` table gained the `smut q` row; the specification-keys paragraph gained the
  sentence that `q` compiles to the same specification.
- **Every transcript was re-run against a fresh store** rather than transcribed from the
  proposal, and the harness executes the new section: `tests/readme_example.rs` runs each section's
  console blocks in a store of its own, so the section is pinned exactly as the worked example is.
  Two proposal details did not survive that contact — a transcript cannot carry a pipe, so the
  envelopes are shown whole; and the cast is trimmed to what these examples need, which is also the
  proposal's own "short enough to read in one screen" rule.

### Deviations, and why

- **Decisions amended as knowledge, not by REV** — the engine refuses the target; see above.
- **IMP annotations appended to their bodies** — `backlog edit` transitions status only, so there
  is no prose verb for a work record.
- **Correction count** — see the erratum. Everything else was paid as briefed.

Ready for `/close`.
