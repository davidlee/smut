# Review RV-027 — reconciliation of SL-005

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-005 (`Refusals that name the right thing`) at the `audit`→`reconcile`
crossing. Two phases complete: PHASE-01 `7347dff` (the contradiction rule), PHASE-02
`9727c1c` (the predicate-level `$` split). Surface reviewed: the primary tree at
`2470576`, code delta `84ff2e0..9727c1c` over the three declared selectors
(`src/predicate.rs`, `src/error.rs`, `tests/predicates.rs`). Conformance is clean —
3 conformant, 0 undeclared, 0 undelivered.

Lines of attack, in the order they were probed.

**1. Does the behaviour match the locked design (§2–§5), not merely the tests?**
The rule's four steps and their fixed order; demand propagation (`And` passes demand,
`Not` flips it, `Or` judges children only when the `Or` itself is demanded false);
one evaluation path (`refusal` called once by the public `parse`, never at a
recursive return). The pins that protect the silences (`$or` single disjunct,
unbounded `$contains`, `$and`-spanning) must still accept.

**2. Is the message a contract or a coincidence?** DEC-030 fixes both sentences.
Both interpolation slots of the misplaced sentence carry the same operator;
`<field>` and `<operand>` are placeholders — no concrete field name is ever printed,
because a predicate-level `$` key carries none. `UnknownPredicateOperator`,
`MixedPredicateObject` stay byte-identical (RV-026 F-2, F-5, F-15).

**3. Do the process-level promises still hold?** Exit 2 and no store created for a
refusal (DEC-004); no success envelope and no success-path stderr (DEC-019); the
hand-maintained boundary table (26 rows) still covered by its own test.

**4. Is the evidence honest?** Red-first claims, VT keyword signal (one keyword had to
be corrected because the plan pinned one array at two lengths), scope discipline, and
the phase sheets' VA evidence — against the artefacts, not the narrative.

**5. Governance drift.** `CHR-003` (six locked-design sentences the implemented rule
falsifies, plus `required-position`/`demanded`), `CHR-002`, design §4's now-stale
`src/predicate.rs:72`/`:173` pointers, and design §6's "SL-004's three phases are all
still `planned`" claim (PHASE-01 completed before this slice landed).

**6. Code-review lens (pre-close).** The whole accumulated delta for quality:
duplication between the five-operator list and `parse_condition`'s match arms, test
brittleness, whether the binary pin is whole-line or substring theatre, and whether the
new test's negative assertions can false-pass.

## Synthesis

**Overall: acceptable — one documentation defect, three prose defects, no behavioural
defect.**

The slice does what it said: two refusals that each name the right thing. The contradiction
rule is four tests over at most five conditions, every decision routed through
`Condition::holds` — the same evaluator the query itself uses — so a refusal cannot disagree
with the answer it withholds; the `1`/`1.0` seam is decided by evaluation in both directions
(`the_sentence_is_chosen_by_the_fixed_order` refuses `{"$eq":1,"$in":[1.0]}` exactly as
`structural_equality_is_serde_json_equality` says it must). The demand walk is a dual, not a
parity count: `refusal` fires once from the public `parse`, `And` propagates demand, `Not`
flips it, and `Or` hands its children `false` only when the `Or` itself must not hold. The
two shapes that would expose a walk installed at a recursive return are accepted, and the
shapes refused under a negated `$or` really are false for every entity. The misplaced
sentence is a contract, not a coincidence: the operator interpolates into both slots, no
field and no operand literal is printed, and it is pinned whole at message level twice and
at binary level against the `smut:`-prefixed line. `UnknownPredicateOperator` and
`MixedPredicateObject` are byte-identical to their pre-slice text.

**Closure story.** Seven findings, all terminal. Five were true of shipped artefacts and
none was behavioural: one stale module doc (repaired in the audit, `b746bac`), three false
or dated design sentences resolved onto the reconcile surface, one plan keyword that could
not be satisfied by construction (already corrected in `1ebe94e`), and one pre-existing
citation error already owned by CHR-002. Two latent risks are captured rather than fixed
(IMP-016's duplicated operator set, IMP-015's non-discriminating `!has_store()` idiom),
plus one semantic gap outside the delta (ISS-005: `{"$not":{}}` empties the query at exit 0).

**The finding that mattered** is F-3, and it was not in the code. `!scratch.has_store()`
cannot fail for a `query`, because `Query::creates_store()` is `false` and `Store::run`
returns `Absent` without touching the filesystem — so the assertion records as DEC-004
evidence something that carries no ordering information at all. The ordering is genuinely
carried, by exit 2 rather than exit 1; the defect is that design §5 attributes the promise
to the wrong assertion. That is the failure mode this audit exists to catch: a green test
whose *name* promises more than its body proves, absorbed into a design table as if the
promise had been checked.

**Standing risks, consciously accepted.**

- **The rule is deliberately partial, and the boundary is pinned.** Four silences are named
  in design §7 and each carries a test whose comment says breaking it is a decision, not a
  cleanup: unbounded `$contains`, anything outside demanded position, `$or` contents never
  analysed, and a contradiction spanning `$and` clauses. `a_single_disjunct_or_is_not_refused`
  and `an_unbounded_contains_is_not_refused` fail if a later reader widens the rule, so the
  limit is a contract rather than an oversight. A satisfiability oracle was considered and
  rejected as a second implementation with no independent truth to appeal to.
- **Two operator lists, coupled by a comment (F-2/IMP-016).** Latent until a sixth operator
  is added to `parse_condition` alone, at which point the message lies about a defined
  operator. Fixing it properly is a design change to a module this slice just settled.
- **The plan's verification keyword was stale by the plan's own design (F-6).** The
  hardening missed cross-phase keyword lifetime; the class is now recorded in
  `mem.fact.doctrine.vt-inert-keywords`. A process question for the operator: hardening
  could reject a keyword that a later phase of the same plan must invalidate — offered, not
  filed.
- **The design run is locked (rev 43).** F-3, F-4 and F-5 are prose repairs owed on a locked
  artefact; reconcile must use the run's own path or record why it cannot. That is a real
  constraint on this slice's cleanliness, not a formality.

**What earns no raise.** `the_sentence_is_chosen_by_the_fixed_order` is the best test in the
delta: nine rows that kill a `BTreeMap`-order implementation, both `$in` array orders, and
the "second semantics" temptation in one table. The pre-close review discarded nine
candidates on evidence, including the two that looked most promising (`contradiction`
disagreeing with evaluation; the walk refusing satisfiable predicates).

## Reconciliation Brief

### Per-slice (direct edit) — `.doctrine/slice/005/design.md`

- **F-4 / CHR-003 — design prose that is false of the shipped rule.** Six sentences plus a
  terminology split. Where a rewrite is needed, the shipped shape is the authority:
  1. §1's diagram label (`each Field is judged only when reached through And`) — the root
     `Field` is judged with no `And`; make the label say *demanded position*, not a path.
  2. §2's "Shape of the check" and §6's `src/predicate.rs` row say the walk is "called once
     at the end of `parse`". Replace with the shipped shape: a private recursive
     `parse_predicate` and a public `parse` that calls `refusal` once over the finished AST.
  3. §5's binary-level row — give both strings: the message-level pin is the bare sentence;
     the binary-level pin carries the `smut: ` prefix.
  4. §6's "including the four pins" — §5 defines three; name them.
  5. §7's "`$or` contents are not analysed" — narrow to *a must-hold `$or` stops the walk;
     an `$or` that must not hold hands its children `false`, so a `$not` beneath it restores
     demand*. DEC-027 already states it this way; the design should agree with the decision.
  6. §3's second sentence (`$exists: false beside any other condition`) names one operator
     and no empty list; DEC-030 asks each sentence to name the two operators or the empty
     list it is about. State the two presence facts.
  - Terminology: §1's "required-position" → "demanded", the word §2–§6 and the plan use.
  - **Lock caveat:** the design is under a locked managed run (`dr-01a19a1e`, rev 43). If the
    run's verbs refuse post-lock prose revision, record that against CHR-003 and leave it
    open with the reason. Do not hand-edit materialised prose to fake a clean reconcile;
    close CHR-003 only when the design genuinely reads true.

- **F-5 — dated and locational drift in the design.**
  - §4's pointers (`src/predicate.rs:72`, `:173`) now miss by ~15 and ~32 lines. Replace the
    line numbers with the constructs: the predicate-level `$` arm of `parse_predicate`, and
    `parse_condition`. Line numbers rot; names do not.
  - §6's "SL-004's three phases are all still `planned`" — restate as the constraint plus
    what happened: SL-005's refusal phase had to land before SL-004's PHASE-02 (which also
    respells identity messages in `src/error.rs`); it did — SL-004's PHASE-01 completed at
    `da4c404` before `9727c1c`, and SL-004's PHASE-02 followed.

- **F-3 — §5's evidence attribution.** The binary-level row claiming "the scratch directory
  holds no store after the refusal" as the DEC-004 ordering evidence: state that for a
  `query` the ordering is carried by **exit 2** (a request validated after the store step
  exits 1 with `no store at …`), and that the absent directory is a non-vacuity guard,
  because `Query::creates_store()` is `false`. Do not weaken the test in the same pass —
  IMP-015 covers the stronger option (a corrupt store seeded in the scratch directory).

### Governance/spec (REV)

- **None.** DEC-027 already states the corrected position rule in both tiers, and DEC-030's
  `choice` facet and body agree on the placeholder shape. No ADR, policy, standard, spec or
  decision needs a revision for this slice.

### Existing and new owning items (not reconcile write surfaces)

- **CHR-003** — closed by the F-4 edit above, or left open with the lock reason.
- **CHR-002** — unchanged; owns F-7 (`src/error.rs`'s `DuplicateKey` doc citing DEC-006
  where DEC-001 rule 1 is the authority). Do not fix from this slice: SL-004 is editing that
  file.
- **ISS-005** — `{"$not":{}}` is a constant-false predicate at exit 0 (outside the delta).
- **IMP-014** — `Reject` coverage is hand-maintained; no `every_reject_refusal` analogue.
- **IMP-015** — the non-discriminating `!has_store()` idiom across the suite.
- **IMP-016** — the duplicated five-operator set; one `Operator` parser as the fix (F-2).

## Reconciliation Outcome

RV-027's brief is resolved. Two surfaces were used: per-slice direct edits to
`design.md`, and one direct correction to `DEC-030`'s body prose. **No REV was authored,
and none was possible** — see "Governance prose" below.

### Direct edits applied — `.doctrine/slice/005/design.md`

- **§1 (F-4)** — the mermaid label `required-position walk: each Field is judged only
  when reached through And` becomes `demanded-position walk: each Field is judged only
  where demand reaches it`; the surrounding prose's *required* becomes *demanded*. The
  root field is judged with no `And`, and one word now wins across §1–§6.
- **§2 (F-4)** — "Two pure functions … called once at the end of `parse`" becomes the
  shipped shape: the private `parse_predicate` builds the tree and the public `parse`
  calls `refusal` once over the finished tree. That sentence was the recursive-return
  defect RV-026 F-1 raised, surviving in prose after the code stopped doing it.
- **§4 (F-5)** — `src/predicate.rs:72` / `:173` become the constructs (`parse_predicate`'s
  `$` arm; `parse_condition`). Line numbers rot; names do not.
- **§5 (F-3, F-4)** — the contradiction row now reads *exit 2, and the scratch directory
  holds no store*, with the reason: exit 2 is what proves the ordering (a request
  validated after the store step exits 1 with `no store at …`), and the absent directory
  is a non-vacuity check because `Query::creates_store()` is `false`. The two message
  rows now state **both strings** — the message-level pin is the bare sentence, the
  binary pin carries the `smut: ` prefix.
- **§6 (F-4, F-5)** — the `src/predicate.rs` row's call shape corrected, the `$` arm
  named by construct rather than line, "including the four pins" → three; the
  SL-004 ordering paragraph becomes the constraint *plus what happened* (`da4c404`
  before `7347dff`/`9727c1c`, PHASE-02 after).
- **§7 (F-4)** — "`$or` contents are not analysed" becomes *a must-hold `$or` stops the
  walk*, with the negated-`$or` case stated beside it. This is DEC-027's own corrected
  wording; the design now agrees with the decision.

**CHR-003 is closed** — all six sentences are addressed (items 1, 2, 5, 6 in the design,
item 3 in §5, item 4 in §6) and the `required-position`/`demanded` split is unified.

### Governance prose — `DEC-030` body, and why not a REV

CHR-003's item 6 ("one of the two should move") resolved onto the **decision**, not the
design: §3's "each names the operators it is about" is true as written, and the seven
pinned sentences are the contract, so the false claim was DEC-030's generalisation —
*"each naming the two operators or the empty list it is about"* — which the
`$exists: false beside any other condition` row cannot satisfy (its dual is any
condition, not a second named operator). The sentence now reads: *each naming the
operators it is about: both sides of a conflicting pair, the single operator whose
conflict is with any other condition, or the empty list.*

No REV was authored, for three reasons, in order of weight: the reconcile brief recorded
no governance item (this was found while locating the edit point, not by re-auditing);
`doctrine revision change add --target` takes a live peer FK from the ADR/SPEC family,
and a decision is not a REV target; and this slice already corrected DEC-030's body once
during design under the same authored-truth honour model, so the mechanism matches the
precedent. The correction is body prose only — the `choice` facet, the seven sentences and
every pinned message are untouched.

### Locked design run — the brief's caveat, answered

`doctrine design adopt 5` refuses, and the refusal *is* the guidance: *"run is locked;
adopt refuses. To correct a run still governing execution, regress to reviewing first;
at reconcile, edit design.md directly and do not adopt."* So the direct edit is the
prescribed posture here, not a workaround, and there is no desync to repair because the
run is not re-entered. The run still reports `locked (revision 43)` with a stale RV-026
pass; that staleness is a statement about RV-026's coverage of the *old* text, and
re-conducting a design review at reconcile is out of this seam's scope.

### REVs completed

None — no governance/spec item remained after the applicability check above.

### Withdrawn / tolerated

None. All seven findings were `verified`; nothing was withdrawn or tolerated, so there is
no drift carried into close with a rationale-shaped excuse.

### Items touched outside the write surface

- **CHR-002** — its body now carries the verified detail of F-7 (a doc-only defect, both
  citations wrong: `§3.1` and `DEC-006`; the authority is DEC-001 rule 1), so the owning
  chore is actionable without reading this ledger (`dfb14cf`).
- **ISS-005, IMP-014, IMP-015, IMP-016** — capture-only, unchanged. The originating items
  ISS-004 and IMP-010 are resolved at close, not here.

### Close evidence — the gate, in a tree nobody else was writing

`doctrine check gate` could not be read green in the working tree at close: SL-004's
PHASE-02 is landing *in the same worktree*, with uncommitted edits to `tests/cli_contract.rs`,
`src/entity.rs`, `src/store/{index,journal}.rs` and two store test files. At the time of
close that tree was red on `cargo fmt --check` (an SL-004 file) and on five `entity::` /
`store::index::` unit tests — none of them SL-005's, none of them a regression this slice
introduced.

So the gate was run where it belongs: on the closing commit itself, in a detached
`git worktree` at `d75961c` with no uncommitted work in it, mirroring `justfile`'s `check`
recipe exactly — `cargo build`, `cargo clippy --all-targets -- -D warnings`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`,
`cargo test`, `cargo test --features fault-injection`.

**All six green, exit 0.** `--lib` 359 default / 372 fault-injection; `tests/predicates.rs`
13; `tests/cli_contract.rs` 40; `tests/compatibility.rs` 10; `tests/fixture.rs` 15; the rest
as recorded in `/tmp/sl005-gate.log`.

The distinction matters for the record: the gate is a property of a commit, and the commit
being closed is green. The shared tree returns to green when SL-004's phase lands — that is
SL-004's gate to pass, not a residual of this slice.
