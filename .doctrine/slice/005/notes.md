# Notes SL-005: Refusals that name the right thing — a contradiction refused and a misplaced operator explained

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · slice phases 2/2 complete, gates green, `verify-vt` 7/7 PASS · PHASE-02 landed at 9727c1c · base 9727c1c (+docs)

### Produced

- **PHASE-01 implementation** (`7347dff`): `contradiction` and `refusal` as pure functions over
  the parsed AST, `parse` split into a public wrapper over the private recursive
  `parse_predicate`; `Reject::ContradictoryFieldConditions` with its `Display` arm, its boundary
  row (`[(Error, u8); 25]`) and a whole-rendered message assertion; seven unit tests pinning the
  seven sentences, the seven demand positions, the fixed order and the three silences; four
  contradiction rows in the binary refusal table (exit 2, empty stdout, no store).
- **PHASE-02 implementation** (`9727c1c`): `parse_predicate`'s predicate-level `$` arm split
  into a five-operator or-pattern (`"$eq" | "$ne" | "$in" | "$exists" | "$contains"`) raising
  `Reject::MisplacedPredicateOperator` and a `_` arm keeping `UnknownPredicateOperator`;
  `parse_condition`'s `other =>` arm untouched, so a clause used as a condition still reads
  *unknown*; the two arms carry cross-referencing comments so a sixth operator cannot be
  added to one only. `Reject::MisplacedPredicateOperator { operator }` with one doc comment,
  one `Display` arm interpolating the operator into **both** slots of
  `predicate operator {operator} belongs inside a field's condition object: {"<field>":
  {"{operator}": <operand>}}`, and a boundary row (`[(Error, u8); 25]` → `; 26`). Pins:
  two whole-sentence `assert_eq!`s on `.to_string()` (`$ne`, `$in`) and one new binary test
  (`a_misplaced_operator_is_refused_naming_its_position_and_never_a_field`) asserting exit 2,
  empty stdout, the whole `smut: …` line, no `status`, no `"x"` operand and no store — plus
  the unknown-name whole-line pin `smut: unknown predicate operator: $foo`. Two comments the
  split falsified repaired, assertions byte-identical (`tests/predicates.rs:328-330`, the doc
  in `parse_refuses_an_unknown_operator_at_either_level`).
- **`plan.toml` PHASE-01/VT-2 keyword repaired** (`[(Error, u8); 25]` → `; 26`), one token,
  authorised by the seat (see § Autonomy log). The `expects` prose stays true; only the
  stale array-length literal moved. Flagged for the audit seam.
- **DEC-027** — amended four times: the six-form enumeration → a decidability criterion
  → `$contains` completeness → demanded position. Now the governing rule.
- **DEC-030** — new: both refusal messages' shape and wording, the never-invent-a-field
  constraint, and the split of the predicate-level `$` arm into two variants.
- **RV-026** — 20 findings across two hostile passes, all answered and verified and
  given routes; pass concluded `conducted`.
- **CHR-002** — `src/error.rs` cites DEC-006 for the duplicate-key refusal; the
  authority is DEC-001 rule 1. Out of this slice's scope.
- `design.md` — seven sections, materialised at revision 43, all seven attested.
- `plan.toml` + `plan.md` — two phases cut by defect (contradiction rule; `$`-message
  split), every obligation mapped to a criterion, hardened by two independent reviews
  (obligations; implementation surface) before dispatch. Slice moved `plan → ready`;
  PHASE-01's runtime sheet is expanded in the state tree, unexecuted.
- **DEC-030 settled `accepted`** and its body's misplaced-operator example corrected to
  `<operand>`; **DEC-027's position rule corrected in both tiers** (the `$not`-parity
  shorthand was too narrow: an `$or` that must not hold hands its children `false`, so a
  `$not` beneath it restores demand).
- **CHR-003** — six `design.md` sentences the implemented rule falsifies, parked for the
  audit/reconcile seam (the design run is locked; editing materialised prose would desync
  its fingerprints). Also carries the `required-position` vs `demanded` terminology fix.
- `research/research.md` — baseline re-stamped at plan time with a refresh section; its
  Thread 2 hotspot map is what the three selectors were drawn from.

### Learned

- **A gate that reads the completion registry cannot run before completion.** `doctrine slice
  verify-vt` attributes each VT to the slice through the phase's delta row, which the
  `--status completed` transition upserts — so a pre-completion run reports every VT
  `UNATTRIBUTABLE` ("keyword present but `<file>` not modified by this slice") however green the
  work is. Remedy: `doctrine slice record-delta <slice> PHASE-NN --commit <S>` (the safe mode,
  recording exactly S's own patch) before the gate; the completion transition upserts the same
  range. Not a defect, a bootstrap order. (`mem.fact.doctrine.verify-vt-needs-delta-row`.)
- **Rust red-first needs the type to exist first.** A test naming an unimplemented `Reject`
  variant is a compile error, so "red" proves only that the name is absent. Land the variant and
  its `Display` arm, re-run, and read the behavioural red ("should have been refused") before
  writing the rule; otherwise the red-first claim is unfalsifiable.
- **Duplicate keys cap the search space.** A condition object carries each operator at
  most once, so two of the original six "impossible forms" were unexpressible and the
  rule is four tests over ≤5 conditions. Reachability must be probed, not assumed.
- **A per-field proof is not a proof about the query.** Position changes everything:
  a contradiction under `$not` matches everything, and one inside `$or` leaves the
  query answerable. Demand propagation, not parity, is the rule.
- **Doctrine reviewing mechanics** (now `mem.pattern.doctrine.design-review-mechanics`):
  severe findings need a `route`, which requires reopen → dispose → verify; section
  review coverage comes from `att-` declarations, not from the `section-reviewed` act
  alone; a reopen clears the pass's concluded marker, so re-conclude.
- **A post-parse check belongs in a wrapper over a recursive parser** —
  `mem.fact.smut.recursive-parse-post-check`. `parse` re-enters itself via
  `parse_all`/`parse_not`, so a walk at its end runs once per clause operand and refuses
  satisfiable predicates. The natural design sentence "call it once at the end of
  `parse`" is silently wrong for any recursive parser whose check carries context.
- **A verify-vt keyword already present in the file is an inert gate** —
  `mem.fact.doctrine.vt-inert-keywords`. Keywords are raw substring presence checks, so
  `INVALID_INPUT`, `has_store`, `$contains` and `UnknownPredicateOperator` all passed
  vacuously in the plan's first draft; key on a name or literal the change introduces
  (a new variant, `[(Error, u8); 25]`, a whole pinned line, a test fn name) and grep the
  tree to confirm it is absent before the phase runs.
- **A keyword that pins a value the *next* phase must change is a latent gate failure.**
  PHASE-01/VT-2 keyed on `[(Error, u8); 25]`; PHASE-02/EX-2 mandates the same array move to
  26, so the plan pinned one array at two lengths and PHASE-01's VT went red the moment its
  sibling phase landed (``verify-vt`` exit 1 with every test green). Remedied here by a
  one-token keyword edit (see § Autonomy log); the durable rule is: **pin presence, or a name
  the phase introduces — never a mutable literal a later phase of the same plan invalidates.**
  Plan hardening should check keyword lifetimes across phases, not only within one.
- **Raw string literals are how a pinned sentence survives the keyword gate.** A whole-line
  assertion written as an ordinary Rust literal needs `\"` escapes, and the escaped text no
  longer contains the plan's keyword, so `verify-vt` fails on a green tree. `r#"…"#` keeps the
  sentence byte-identical in the file and in the assertion.

### Open

- **Both phases are done** (`PHASE-01` 7347dff, `PHASE-02` 9727c1c); `doctrine slice
  verify-vt 5` is **7/7 PASS**, `CARGO_NET_OFFLINE=true just check` exit 0. The slice is
  ready for `/audit`; audit has not started.
- **Carried to the audit seam:** the `plan.toml` PHASE-01/VT-2 keyword correction (25 → 26)
  and CHR-003's six locked-design sentences. PHASE-02 owns RV-026 F-2, F-5, F-15 — all three
  discharged as written.
- **Candidate process improvement, not created silently:** plan hardening should reject a VT
  keyword that a *later phase of the same plan* is mandated to change; PHASE-01/VT-2 vs
  PHASE-02/EX-2 is the worked example. Worth a backlog item only if the operator wants it —
  the practical rule is recorded in § Learned above.
- Nothing open inside the design. Three silences are deliberate and named in §7
  (`$contains` unbounded; anything outside demanded position; `$or` contents never
  analysed — pinned, so breaking it is a decision).
- **Plan hardening disposed on `/feedback`.** DEC-030 is `accepted` and its body's
  misplaced-operator example now prints `<operand>` as its facet always said; DEC-027's
  position rule is corrected in both tiers (the `$not`-parity shorthand was too narrow —
  an `$or` that must not hold hands its children `false`, so a `$not` beneath it restores
  demand). The locked design's own six false sentences are parked on **CHR-003** for the
  audit/reconcile seam, because editing materialised `design.md` would desync the run's
  fingerprints for prose that changes no behaviour. The plan implements the correct
  behaviour in every case.
- Carried into the plan: `tests/fixture.rs:457` composes `{"id":{"$in": reached}}`
  from a computed list; an empty derivation now refuses. Check it at implementation.
- Ordering: SL-005 lands before SL-004's PHASE-02 (both touch `src/error.rs`); SL-004's
  phases are all still `planned`.
- CHR-002.
## Design surface triage — explore.triage (2026-11-02, exploring)

Evidence: `.doctrine/slice/005/research/research.md` (threads 1–2, cross-thread
findings 1–7). Constraining governance is DEC-027 (mechanism + six forms), DEC-019
(envelope must not move), DEC-006 (duplicate keys refused) and
`mem.fact.smut.failure-classes` (a caller's own error is `Reject`, exit 2 — never
`StoreError`, never a panic). ADR-001, STD-001 and STD-002 are checked and not
applicable; this project has no `spec` entities.

### Shaping decisions (provisional — design locks them)

- **D1** — one new `Reject` variant carrying the field and a contradiction
description, matching `MixedPredicateObject`'s shape. Five variants would
multiply `Display` arms and the boundary table for no caller-visible gain.
- **D2** — the check runs in `parse_field` over the assembled `Vec<Condition>`,
so it is key-order independent and reads nothing but the condition list (the
module's no-store/no-clock property holds).
- **D3** — contradiction is decided with `serde_json`'s `==`, the same equality
`Condition::holds` evaluates with; otherwise refusal and evaluation can disagree
about `1` versus `1.0`.
- **D4** — no guide change. `src/learn.rs:116` already says a malformed predicate
is exit 2 and writes nothing, and `src/learn.rs` is SL-004's selector.
- **D5** — ISS-004 keeps `UnknownPredicateOperator` and its printed name (both are
pinned); only the sentence gains position and a generic shape.

### Open questions

- **OQ-1** — Objective 2 says *six* forms, but `$exists:true` with `$exists:false`
is **unexpressible**: `src/input.rs`'s `Value` visitor refuses a repeated key in
any object, so DEC-006 refuses it before a `Value` exists. Present six and mark
one unexpressible, or present five? Recommend: six, with the reason stated — the
reader of DEC-027 needs the mapping either way.
- **OQ-2** — a scope tension. Objective 2 says the six forms *"and only these"*;
Risks says the subset *"is a floor for the decision, not a quota"*. The dual of
form 4 is sound and decidable but unlisted: `$ne v` with an `$in` whose every
member equals `v`. Recommend: do not widen (a decision fixed the subset); record
the gap beside IMP-012's scoping sentence. Needs the operator's assent.
- **OQ-3** — does DEC-027's epigram (*a condition that can never hold is a typo,
not a question*) ride in the message? stderr is one line (`src/main.rs:116`), so
it would appear on every instance. Recommend: it stays in the decision and the
guide; the message names the field and the contradiction and stops.
- **OQ-4** — the contradiction wordings themselves (one per reachable form, plus
the literal-`$eq` case). Message text is a surface: choose, then pin. Needs the
operator's ear.
- **OQ-5** — ISS-004's shape example. No field can be named, so the placeholder
must be visibly generic and must not read as the field-level `MixedPredicateObject`
diagnosis.
- **OQ-6** — `$exists:false` with a literal `$eq` is unsatisfiable too, but the
two cannot share a condition object (all-`$` or literal, never both), so form 1's
"any other condition" should be stated as a rule, not as a key list.

### Risks

- **R1** — a partial detector reads as assurance. Bounded only by IMP-012's
scoping sentence, which this slice does not ship.
- **R2** — runtime-composed predicates move from exit 0 to exit 2. Accepted with
DEC-027; the refusal's own test carries a sentence about it.
- **R3** — `src/error.rs` is shared with SL-004's PHASE-02/EX-9. Ordering, not
avoidance: land the refusal phase before that phase, or take a small rebase.
- **R4** — `MixedPredicateObject`'s message is pinned byte-identical; the new
wording must not be reached by editing it.

### Assumptions

- **ASM-1** — SL-004 does not rewrite `tests/predicates.rs` or `src/error.rs` in
its early phases; if it does, this slice waits or rebases.
- **ASM-2** — `predicate::parse` stays the only construction path for a
`Predicate::Field`, so the check cannot be bypassed by a caller building
conditions directly.

## Review passes (2026-11-02, reviewing)

Two hostile passes ran against `materialised design.md`, each by an independent
read-only reviewer. The first attacked the design as drafted and returned one blocker (a
contradiction inside `$or` or `$not` would have been refused although the query is
satisfiable), four majors and eight lesser findings. The second attacked the *repairs*
and returned three majors: an `$or` of one disjunct passing the walk, a parity shorthand
in the prose that is false wherever `$or` interrupts the negations, and an unspecified
message template whose operand example would itself be refused for two of the five
operators. All twenty findings are disposed on RV-026.

A further pass would probe, in this order:

1. **The implemented walk against the design's demand rule**, not the rule on paper:
   the seven shapes in §5 plus `$and`-under-`$or`, `$or`-under-`$and`-under-`$not`, and
   an empty `$and` beside a contradictory field.
2. **The two message templates rendered from real input**, since both carry two
   placeholders and a slot mismatch is invisible to a substring assertion.
3. **The `$or` boundary as shipped** — that `{"$or":[{"f":{"$eq":1,"$ne":1}}]}` is still
   accepted after implementation, i.e. that the pin was not "fixed".
4. **A computed-empty `$in` through the binary** — the one runtime shape DEC-027
   knowingly moves from exit 0 to exit 2.

None of that is worth a third pass before there is code to point it at: each item is
first-implemented-then-checked, and the design's remaining risk is prose drift, which
the plan's red-first tests catch.

### Drift after lock (2026-11-02)

SL-004 moved while this design was being reviewed: its **PHASE-01 is `completed`** at
`da4c404`, and PHASE-02 is next. Design §6 and the scope's risk note say "SL-004's three
phases are all still `planned`" — true when written, false now. The ordering constraint
they state is therefore **live rather than theoretical**: SL-005's refusal phase should
land before SL-004's PHASE-02, which respells two identity messages in the same
`src/error.rs`. Enforce it in the plan; do not reopen the design for a dated sentence.

## Implementation run

### Preflight (super-orchestrator seat, PHASE-02 charter)

- **Charter:** SL-005 PHASE-02 only (`Split the predicate-level $ refusal, keeping both
  sentences true`). The seat announces it; the sub-orchestrator drives it.
- **Ordering guard (PHASE-01/EN-2, PHASE-02/EN-2) holds:** SL-004's PHASE-02 is still
  `planned` in `.doctrine/state/slice/004/phases/phase-02.toml`, PHASE-03 too. PHASE-01
  of SL-004 is `completed`. No rebase needed; no race on `src/error.rs` at charter time.
- **Baseline green at `b544359`:** measured now rather than recalled — `cargo test
  --test predicates` **12**, `--lib error::` **6**, `--test cli_contract` **40**,
  `--test compatibility` **10**, `--lib predicate::` **18**; `CARGO_NET_OFFLINE=true just
  check` exit 0 (clippy `-D warnings` on both feature sets, `fmt --check`). No known
  baseline failure. Correction: this seat's earlier preflight and the charter listed the
  counts rotated (`predicates 40, cli_contract 10, compatibility 12`); the measured figures
  above are the ones on this tree, and every suite is green before and after PHASE-02.
- **Toolchain probed:** `cargo 1.100.0-beta.7`, `just 1.58.0`, `git` on PATH;
  `cargo build --offline` and `cargo test --offline` work. Gate command for this phase:
  `CARGO_NET_OFFLINE=true just check` after `cargo fmt`.
- **Research:** `doctrine slice research 5` — "baseline is current". No refresh.
- **verify-vt snapshot:** PHASE-01 `VT-1..VT-4` PASS; PHASE-02 `VT-1..VT-3` FAIL on
  their own absent names (`MisplacedPredicateOperator`, the pinned `$ne` line). Expected
  before the phase; the gate needs `record-delta` before it can attribute (see Harvest).
- **Paths not owned by this slice — never staged:** `README.md` (operator's 0.2 rewrite),
  and untracked other-writer work: `.doctrine/slice/006*/`, `.doctrine/slice/006-*`,
  `.doctrine/backlog/idea/004*/`, `005*/`. Commits use explicit pathspecs only.
- **Plan pointer drift, decided not escalated:** PHASE-02/EX-5 names
  `src/predicate.rs:397-399` for the falsified comment; that text actually lives in
  `parse_refuses_an_unknown_operator_at_either_level` (~`:527-530`), and the
  `tests/predicates.rs:328-331` pointer is accurate. The comment is identified by
  content, both hunks are diffed comment-only, assertions byte-identical.

## Autonomy log

Scope: the PHASE-02 charter, the seat's `need_decision` envelope and the seat's reply.
Every entry is a decision taken inside the envelope; none widened scope.

- **PHASE-02 · decide · task ordering.** Contract type first (`src/error.rs`: variant +
  `Display` arm + boundary row + message pins), binary pin and comment repair on
  `tests/predicates.rs` in parallel (disjoint files), then the parse split
  (`src/predicate.rs`). Reason: a Rust test naming an absent variant is a compile error, so the
  red has to be behavioural; the observed red was the binary still printing
  `smut: unknown predicate operator: $ne` with 12 other tests green. Status: **done**.
- **PHASE-02 · decide · stale pointer corrected.** PHASE-02/EX-5 names
  `src/predicate.rs:397-399`; the falsified prose is in
  `parse_refuses_an_unknown_operator_at_either_level` (~`:541`). Identified by content;
  `plan.toml` untouched for it. Status: **done**, flagged for audit.
- **PHASE-02 · decide · fixture/helper reuse.** The binary pin reuses `Scratch`/`smut`/`Ran`
  and adds a sibling test rather than restructuring the malformed-predicate table (its rows are
  2-tuples asserted with `contains`; a whole-line pin cannot ride them). Status: **done**.
- **PHASE-02 · decide · raw-string pinning.** Pinned sentences are written as `r#"…"#` so the
  plan's VT keywords exist verbatim in the file text, which `verify-vt` reads. Status: **done**,
  recorded in § Learned.
- **PHASE-02 · decide · baseline-count correction.** The charter's baseline figures
  (`predicates 40, cli_contract 10, compatibility 12`) are the measured ones rotated; recorded
  as measured (`predicates 12 → 13, cli_contract 40, compatibility 10, lib 359/372`). No suite
  lost a test; no guard changed. Status: **done**.
- **PHASE-02 · escalate (seat) · locked-plan keyword conflict.** `verify-vt 5` exited 1 after
  the code commit: PHASE-01/VT-2 keyed on `[(Error, u8); 25]` while PHASE-02/EX-2 mandates the
  same array at 26 — the plan pinned one array at two lengths. Escalated via
  `contact_supervisor` (`need_decision`) rather than editing a locked artefact unilaterally.
  Seat's decision: **A — authorised**, exact scope one token on `plan.toml:51`, no `plan.md`
  change, re-run the gate, tighten the boundary, log it. Applied exactly that; `plan.md`
  confirmed not to repeat the literal. Status: **decided, flagged for audit**.
- **PHASE-02 · defer (not actioned).** No out-of-scope defect found in the three selectors, so
  nothing new went to the backlog. One process improvement (cross-phase VT-keyword lifetime) is
  recorded in § Learned and offered to the operator in the handback rather than filed silently.
  Status: **deferred to the seat/operator**.
