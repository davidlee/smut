# Review RV-021 — reconciliation of SL-002

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-002 (`smut` 0.2) at implementation-complete, 7/7 phases,
HEAD `a9aee11`, branch `main`, tree clean. Mode: **conformance**; facet:
**reconciliation**. Surface reviewed: the primary worktree — the implementation
ran on this tree, so the reviewed artefact *is* `main` (no dispatch candidate
exists; `review/*` and `phase/*` are not dispatch evidence refs here).

**What this audit holds the slice to**

- the scope's 11 objectives (`slice-002.md`) and the brief's query contract in
  full (`smut-implementation-brief.org` § Query contract) — the brief wins where
  anything disagrees;
- the locked design (`design.md`, run `dr-01a196a8`, rev 62) — §5 grammar, §6
  predicates, §7 traversal and determinism, §8 envelope/provenance/truncation,
  §9 conventions and guide, §10.1 invariants, §10.3 edge cases, §13.3 scenarios,
  §14.1/§14.2 touch-set and fence;
- the plan's per-phase EN/EX/VT (`plan.toml`) as the slice's own acceptance
  criteria, ids immutable;
- governance: `DEC-011` (0.1 surface stays valid), `DEC-012`…`DEC-019` (the
  slice's settled decisions), `DEC-001`…`DEC-010` (inherited, unchanged).

**Lines of attack**

1. *Objectives against the delivered binary, behaviourally.* Every predicated
   claim was checked by running `target/debug/smut` against a scratch store and
   against the committed fixture, not by reading the design back to itself:
   grammar refusals (types, wildcard spelling, `budget: 0`, beyond-`u64` depth,
   unknown keys naming no release), the five operators and three clauses with
   their identities, disambiguation and the `$eq` escape, traversal in both
   directions with `[]`/`*`/named fields, depth 0/1 and the frontier, budget as
   expansions, truncation reasons in order, induced edges (unrequested field,
   unexpanded endpoints, `select` hiding a reference), dangling dedupe and
   walk-scoping, `get` never traversing, the Markdown notice (dangling-only and
   `traversal_budget`), and the conventions command end to end including the
   seed pipe, re-install replacement of an amended body, the surviving user
   field, `previous`, and the refusal of the deferred set.
2. *The six §13.3 acceptance scenarios in the suites that own them*, plus the
   two-call fixture recipe re-run independently (66 ids reached from `session`
   by `backlinks:["part-of"]` at depth 2; exactly eight open questions on the
   second call) rather than taken from `verify-vt`.
3. *PHASE-07's release surface* — the only phase with no independent review
   pass. README's nine-key paragraph, worked transcript, traversal-cost and
   provenance sections, conventions section, sharp edges and install line
   against the binary; `learn`'s claims one by one; `0.2.0` in `Cargo.toml` and
   `Cargo.lock` with no dependency movement.
4. *The relationships between phases* — whether a later phase falsified an
   earlier one's posture: PHASE-03's roots-only `where` under PHASE-07's README
   claims; PHASE-01's envelope keys under PHASE-06's guide; PHASE-04's fixture
   properties under PHASE-06's shipped example under PHASE-07's transcript.
5. *The known weak spot of `verify-vt`* — a keyword that passes is not a
   behaviour that is tested. Sampled: PHASE-01 VT-1 (`self-reference`,
   `depth boundary` occur in comments in `src/walk.rs` — but the behaviours are
   genuinely asserted there, `a_self_reference_is_discovered_once_and_does_not_requeue`
   and `the_depth_boundary_is_frontier_not_truncation`), PHASE-03 VT-4's
   `where_accepted` token, PHASE-04/05/07's keyword sets. Also checked every
   test-module comment that names a test: the only namesakes are deliberate
   DEC-019 re-cut references to deleted tests.
6. *Claims vs code* — every durable sentence that asserts behaviour: the guide,
   the conventions prose, `PROVENANCE.md`, the store-format document, and the
   run record's own statements about what was pushed and what was restored.
7. *Mechanical conformance* — `doctrine slice conformance 2` (26 conformant, 0
   undelivered, 4 undeclared, each dispositioned) and the §14.2 fence proven by
   `git diff` across the whole slice rather than per phase.

**Gates re-run at this seat on the final tree** (not taken on trust):
`cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings` clean;
`cargo test` 16 suites / 506 tests green; `doctrine check gate` exit 0 (both
clippy configurations, the test matrix and the fault-injection cadence);
`doctrine slice verify-vt 2` 25 PASS / 0 Fail / 0 unattributable;
`doctrine slice conformance 2` as above.

**Priors honoured.** The review lane's four ledgers (RV-017…RV-020: 23 findings,
0 blocker, 0 major, 0 design contradiction) are treated as evidence, not
re-audited; the recorded decisions on GAP-1, DEF-1…3, RV-017 F-2, RV-018 F-4,
RV-020 F-2 and the four conformance cells are accepted or contested explicitly
where this audit has new evidence, never silently reopened. The three locked-
design sentences the operator deferred to reconcile (source line numbers, the
"mutually-referencing clusters" claim, `learn`'s version-bound self-description)
are not re-litigated as design defects — except where this audit found the same
claim in a tier the reconcile list does not name.

## Synthesis

**Closure story.** The delivered binary meets the scope's eleven objectives and
the brief's query contract, and I checked that by running it rather than by
reading the design back to itself. Predicates hold as a closed interpreter: the
five operators, the three clauses with their empty-array identities, sibling
keys conjunctive, missing-fails-everything-except-`$exists`, present-null
present, structural equality with no coercion, `$in` and `$exists` operand
refusals, the mixed-object refusal naming its field, and the `$eq` escape for a
literal object carrying `$` keys. Traversal is a breadth-first walk over full
entities that consumes `entity::scan_refs` — grepping `src/` for a second
reference interpretation finds none, and the induced edge set is derived from
returned entities, not from expansions. The envelope stays additive: five keys
when nothing was traversed, a sixth only when `follow` or `backlinks` was
supplied, including as `[]` — `get`, `query {}` and every 0.1-shaped call keep
0.1's shape, and `tests/compatibility.rs` compares whole key sets rather than
spot-checking members. `depths` and `edges` are result-scoped, `frontier` and
`dangling` walk-scoped; the budget counts expansions and is checked before
expanding; a depth boundary is selection, not truncation; reasons are unique and
ordered. Determinism is behaviour, including the shuffled-store case. The
conventions command is pure text with one set, a fixed-id versioned seed, an
upsert that replaces the seeded body while keeping a user's field and reporting
the old one under `previous`, and a refusal for the deferred set by name. The
guide carries the two-call recipe with its direction, its four guard clauses,
the no-propagation limit, and the example the fixture suite executes.

Evidence that this is not a prose opinion: fmt and clippy clean in both feature
configurations, 16 suites / 506 tests, `doctrine check gate` exit 0,
`verify-vt` 25 PASS, `conformance` 26 conformant / 0 undelivered / 4 explained
(re-run at this seat), and the two acceptance facts re-derived by hand — the
fixture reached by `backlinks:["part-of"]` at depth 2 returns all 66 ids with the
`follow` variant returning `session` alone, and the second call names exactly the
eight open questions.

**Where the six findings landed.** Two are record defects, not artefact defects:
eight settled decisions carry `proposed` (F-1), and the falsified
"mutually-referencing clusters" claim lives in four tiers rather than the one the
run's reconcile list names (F-2). Two are stale run-record claims that would
mislead the next seat — "nothing pushed" against a public branch, and a
"byte-identical" restoration that is not byte-identical (F-3) — and one is
ledger hygiene the route axis hands to this seat: three design-stage ledgers
left non-terminal, with the one finding that awaited a raiser verifiable on
evidence (F-4). The fifth is a version-scoped document the design fence forbids
touching (F-5), and the sixth is four conformance cells that are the review
lane's own artefacts (F-6). No finding is a blocker; nothing must not ship
unreconciled.

**Standing risks.**

- **The fixture's vocabulary is stale by design** — `vocab` says `projects` is
  unused while `a-focus` uses it, and `session` says nine questions are open
  while the array holds eight. Tests assert against entities, never prose, and
  the staleness is documented. It is evidence of the practice the hygiene set
  warns about, not a defect to repair.
- **`backlinks` is unbounded per hop** and the budget does not bound it. Stated
  in the guide and the README with the exact boundary; this release makes no
  total-work claim and should not be read as making one.
- **A prose pin is a substring match.** `verify-vt` still certifies keywords,
  and the guide's pins are whitespace-flattened substrings — they can survive a
  claim that moved and they cannot distinguish a reworded sentence from a
  deleted one. The one structural pin (`the_guide_teaches_exactly_the_nine_keys`
  reading the key block) is the pattern that would fix the rest, and it is
  applied only there.
- **The store-format document speaks in 0.1's voice** (F-5). A reader of 0.2
  gets true statements whose scope they must infer.
- **Landing is a force-push** (F-3). Local `main` forked from the public tip; a
  fast-forward push will be refused at close.

**Tradeoffs consciously accepted.** The seeded `body` is replaced on re-install
and the store owns its copy only as an ordinary entity — the mechanism that makes
re-install an upsert also makes a hand-amended `body` perishable, and the reply's
`previous` is the only recovery. `where` stays roots-only, so the two-call
composition and its guard are the idiom rather than a defect. Four conformance
cells stay undeclared because declaring them would make the review lane a design
target. The store-format document keeps a fence over a stale voice. The design's
own three deferred sentences stay deferred.

## Reconciliation Brief

### Per-slice (direct edit)

- **`design.md` §2 and §13.1 (Fixture row), and `slice-002.md` objective 10** —
  RV-021 F-2. The acceptance store holds **no mutually-referencing cluster**; it
  holds one mutually-referencing pair (`finding-smut-17` ↔
  `prop-conventions-library`) and five clusters that are one-directional, each
  `cluster → session`. Replace the phrase in all three places with what the array
  actually holds; keep "22 entities with no incoming reference", which is
  measured and true (66 entities). `plan.toml` PHASE-04 EX-3's same clause is
  off-surface and needs no edit — its obligation is met by the pair.
- **`design.md` §1/§2 and §9.4** — the three sentences the operator deferred at
  G0, confirmed at this seat with the exact reading:
  - §1 cites `src/entity.rs:60` for `scan_refs`, §2 cites `src/entity.rs:31,60`
    and `src/learn.rs:113`; the actual lines are `:31`, `:71` and `:111`.
  - §9.4's "remains descriptive and version-bound": the guide's banner carries no
    version and its only release word is "in 0.2" inside the `$`-reserved limit,
    so "version-bound" should be restated as release-bound — no build number, one
    release-specific clause.
  - §2's "mutually-referencing clusters" is covered by the F-2 entry above.

### Governance/spec

- **`DEC-012`…`DEC-019`** — RV-021 F-1. All eight are `proposed`; 0.1's
  decisions were created `accepted` (`DEC-011` at `bcd1389`). Record them
  accepted, citing the operator's assent to the eight as 0.2's settled decisions
  — the records already say `decided_by: operator and agent`, so a status flip
  asserts that assent and should not be written without it. Verb:
  `doctrine knowledge status <ID> <STATE>`. No prose changes to the records and
  no new mechanism.

### Not reconcile edits — escalated to the operator

- **`docs/store-format.md`** — RV-021 F-5, tolerated. The fix is a second scope
  change inside a §14.2-fenced file; every statement stays true of
  `format_version` 1. Either authorise the one-line "unchanged since 0.1"
  rewording, or leave it to IMP-002's documentation pass.
- **Landing** — RV-021 F-3. `git push --force-with-lease=main:dfc2887…` at
  close, not a fast-forward. The operator ruled: wait, then push the finished
  slice.
