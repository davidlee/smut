# Notes SL-002: smut 0.2 — graph release: predicates and traversal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · implementing (PHASE-01 `45983c7`; PHASE-02 `1324dae` +
`4a192e7`; R1 repaired `b499a60`; PHASE-03 `21688c8`; PHASE-04 `fddbe33`;
PHASE-05 `cb46e37`; PHASE-06 `46c6699` + `a6ddb25`; PHASE-07 planned) · design
locked at run rev 62; plan repaired round 1

### Produced

- SL-002 scope `slice-002.md` — 11 objectives, the boundary, and the settled
  decisions table replacing the old open questions.
- Research artefact + raw threads; baseline restamped after the design.
- DEC-012…DEC-019 from the eight inquiry dispositions; DEC-014 amended to the
  induced subgraph of returned entities.
- 15-section `design.md`, unchanged at revision 57; review passes RV-007…RV-014
  across four adversarial windows, every finding terminal except RV-012 F-4
  (answered, awaiting the raiser's verification). RV-015 is the rebinding pass
  opened by the stage round trip, concluded with its basis recorded.
- 16 `design-target` selectors matching §14.
- IMP-001 moved to `started`.
- `IMP-004` — the epistemic-records convention set, deferred out of 0.2 with its
  draft taxonomy, the five open questions and the doctrine comparison
  (`originates_from SL-002`, `after SL-002`); DEC-017 narrowed to one set.
- `plan.toml` / `plan.md` — seven phases, 25 VT mandates each carrying
  `test_file` + `keywords`, plus VA/VH rows; sequencing is the scope's risk order
  with the walker first.
- Plan review round 1 — two read-only reviewers (obligation coverage over 236
  design obligations; sequencing against the tree, verdict BLOCK), integrated:
  38 findings, 36 conceded, 2 escalated, 0 rejected. Two sequencing blockers
  repaired (README console block re-recorded at PHASE-01; `FUTURE_KEYS` retires
  at PHASE-03). Escalations: the privacy gate's negative outcome, and design
  line-number drift.
- **PHASE-01** (`45983c7`, C1) — `src/walk.rs`: `Fields`, `Walk`, `Dangling` and
  the BFS consuming `entity::scan_refs`, with 14 unit tests (VT-1). `src/spec.rs`
  accepts seven keys, `FUTURE_KEYS` holds only `where` and `budget`, and
  `Fields::Named` dedupes at parse. `select::run` took one private `Request`
  value (EX-13); `Selection.walk`, the conditional `traversal` member, induced
  `edges` in result order and the walk's `dangling` landed in `envelope`.
  `src/render_markdown.rs`'s notice trigger grew the dangling case with one line
  per triple (EX-14). New `tests/traversal.rs` (25 tests, VT-2/3/4) plus
  VT-5…VT-9 in the existing suites. The README's console block was re-recorded
  against the built binary and its falsified prose re-cut in the same commit
  (EX-11, plan blocker B1). DEC-019 re-cuts: `src/spec.rs`'s five-key loop and
  its exhaustive-claim names, `tests/compatibility.rs`'s end-to-end example (now
  asserts the brief's closing dangling step), `tests/readme_example.rs`'s
  0.2-refusal test (now the traversal assertion). Nothing outside §14.1 touched;
  the §14.2 unchanged set is unmodified.

- **PHASE-03** (`21688c8`, C3) — `src/predicate.rs`: the §6.1 AST, the §6.2
  parser (literal = `$eq`, operator-only = ANDed, mixed refused naming the field,
  `$` reserved, `$and`/`$or`/`$not`) and `Predicate::holds` over one entity's
  top-level fields, with 11 unit tests (VT-1). `select::run` filters roots
  between `missing` and the walk, through the same single `Request` value
  (EX-13). `where` is the ninth key, so `FUTURE_KEYS`,
  `Reject::FutureSpecificationKey` and its display arm and exit-code row retire
  (EX-8); three predicate refusals (`MixedPredicateObject`,
  `UnknownPredicateOperator`, `InvalidPredicateValue`) join `Reject`, all exit 2
  before store access (EX-6). New `tests/predicates.rs` (12 tests, VT-2/VT-3 +
  the EX-6 CLI case); DEC-019 re-cuts in `tests/compatibility.rs`
  (`where_accepted_…`, DEF-3), `tests/cli_contract.rs`, `tests/selection.rs`,
  `src/spec.rs` (future-key test deleted with its apparatus, acceptance set →
  nine), `src/error.rs` (two assertions) and the README's key-refusal sentence.
  Gates green in both feature configurations; five negative controls each turned
  the owning test red. `src/learn.rs` untouched (GAP-1).

- **PHASE-04** (`fddbe33`, C3) — `tests/fixtures/interview/entities.json`: one
  normalised array of the first-use store's 66 entities, read through `smut
  query '{"limit":400}'` with the store directory as cwd, sorted by id, each
  entity in the store's own serialisation order (`id` first, remaining keys
  lexical), values verbatim. Declared into an empty directory it creates 66
  entities and reads back an identical array, so it rebuilds the store rather
  than sampling it; it is the only copy of the data and nothing reads `.smut`
  internals. `PROVENANCE.md` records the source snapshot, the operator's G1
  privacy ruling (read-through waived 2026-11-02, consent from recall, nothing
  removed, no redaction, resolved by the owner), the committing phase's residual
  scan, the normalisation rules and the array's own counts (66 entities, 28
  statused, 44 reference targets, 22 with no incoming reference, 5 clusters, 0
  retracted, 8 open questions). `tests/fixture.rs`: 9 tests over a store declared
  from that array in a copied `Scratch` (no `tests/common`) — non-empty and
  read-back count, an unreferenced entity, `finding-smut-16` superseded in place,
  a settled finding, a mutually referencing pair, exactly the five clusters, no
  retracted record, and the two-call acceptance query by id
  (`backlinks:["part-of"]` depth 2 from `session`, then those ids as `where`
  roots filtered to `open` = the eight). Red observed on the absent file;
  negative control amputating one cluster's `part-of` edge turned exactly the two
  containment assertions red. Gates green in both feature configurations; no
  source path touched, `src/learn.rs` untouched (GAP-1).

- **PHASE-05** (`cb46e37`, C4) — `src/conventions.rs`: the one-set registry
  (`Set`, `HYGIENE`, `SETS`), `Ask` (`List`/`Prose`/`Seed`) resolved before it is
  carried so the answer is infallible text, `listing()`, `seed()` under the fixed
  id `conventions-hygiene` with `type: convention`, the crate's version and a
  title derived from the prose's own heading. The prose is
  `include_str!("conventions/hygiene.md")`, so §9.2's eight items — item 7's
  `retracted`/`superseded`/`supersedes`/`vocab` among them — ship as one copy
  printed three ways. `src/cli.rs` grows `conventions [NAME] [--format
  text|json]` and `Invocation::Conventions`, routed in `src/main.rs` on the
  no-store chain; `src/error.rs` gains `UnknownConvention` (naming the sets) and
  `UnknownConventionFormat`, both exit 2. `--format json` with no name is a usage
  error. New `tests/conventions.rs` (14 tests, VT-1/VT-2 plus the §13.3
  scenario-5 pipe). `src/learn.rs` untouched (GAP-1); no §14.2 path touched.

- **PHASE-06** (`46c6699`, C5) — `src/learn.rs`: the guide rewritten as the
  normative home of 0.2. It teaches the nine keys with defaults and the
  roots-only rule, the predicate summary with missing-vs-null and the `$`
  namespace limit, traversal labels as top-level fields in both directions with
  `depth`/`budget` and what `budget` does not bound (the per-hop incoming store
  scan, the whole-store root scan), the conditional `traversal` member with
  `depths`/`edges`/`frontier`, `dangling`, both truncation reasons in their
  order, the two-call recipe **with its direction** (containment child →
  container, so the session question is `backlinks:["part-of"]` at depth 2 while
  the `follow` variant is a different question that returns `session` alone), the
  completeness guard clause by clause, the client-side limit that keeps the key
  set at nine, a minimal label vocabulary with bounded-render advice, and one
  `smut conventions` pointer. 0.1's "arrive in 0.2" notice and its
  dangling-arrives sentence are gone (GAP-1 closes); the banner lost its version
  because the crate version is PHASE-07's. New `EXAMPLE_TRAVERSE`,
  `EXAMPLE_FOLLOW` and `EXAMPLE_FILTER` constants are printed in the guide and
  pinned by `GUIDE.contains(..)`; `tests/fixture.rs` (which had never imported
  the library) gained a guard helper (`Guarded`, both fields read before any id
  is extracted) and five tests that execute those constants: `smut learn` prints
  them, `backlinks` at depth 2 from `session` plus the `where` over the returned
  ids is the eight open questions, `follow` reaches `session` alone, both
  truncation reasons are named on a truncated first envelope, and a
  `truncated: false` second envelope over a capped first call is no evidence.
  The 12 inherited red assertions were adopted unchanged; only the re-cut test's
  stale doc cross-reference was corrected, in `a6ddb25`, and `rustfmt` reflowed
  the inherited `KEYS` list. Gates green in both feature configurations;
  `verify-vt` PHASE-06 VT-1/VT-2 PASS with PHASE-01…05 still PASS.

### Learned

- `entity::scan_refs` is the walker's whole reference vocabulary; `Entities`,
  the index and `serde_json::Map` are `BTreeMap`s, so only named-field query
  order must be imposed by the walker.
- The 0.1 compatibility pin compares the whole parsed envelope, so provenance
  must be conditional on the traversal keys being supplied.
- Edges must be the induced subgraph of returned entities, derived after the
  limit — not the walk's crossed-edge subset — or 1.0's renderer derives them a
  second time. DEC-014 was amended for this.
- `materialise` orders sections lexically by id; document order was fixed by
  mapping bodies to ids in lexical order (mapping in this file).
- The first-use store's `vocab` is stale: it says `projects` is unused, but
  `a-focus` uses it. Tests assert against entities, not vocab prose.
- Presence of `traversal` keys on the *request* (either key supplied, empty
  array included), never on the root set; `from: []` therefore still carries it.
- Edge order is **result** order, not discovery order: edges are induced after
  ordering and the limit, so DEC-012 lagged DEC-014 until the review repair.
- Fixture counts are pre-redaction source measurements; the test pins properties
  plus the committed array's own counts, so the privacy gate cannot redden it.
- Seed ids are fixed **per set** (`conventions-hygiene`, and one per later set),
  so an upsert cannot touch an entity with a different id: the store's `vocab` is
  evidence of the practice of amending convention prose, *not* a record this
  pipeline overwrites. The valid overwrite counterexample is editing `body` on an
  installed `conventions-hygiene` and re-running its own seed.
- The scope split is **one filter, not two maps**: `Walk.depths` records every
  discovered id, and `envelope` keeps only the returned ones. A second depth map
  would be a second account of the walk.
- `edges` must iterate `selection.ids` (result order). Discovery order looks
  identical in any fixture without an `order`, so the only test that can tell
  them apart has to reverse one against the other — that is the negative control
  that keeps DEC-014 honest.
- Presence of `traversal` keys on the *option*, never on its contents: treating
  `follow: []` as absent is the one wrong implementation a plain edge fixture
  cannot distinguish from a correct one, so the negative control supplies
  exactly that mutation.
- `entity::scan_refs` reports the **top-level** field as the label even when the
  reference sits in a nested object or an array element; that is what lets the
  walker and the envelope's induced-edge derivation share one interpretation
  (EX-8/VA-1 verified by grep: `scan_refs` is the only reference reader in
  either).
- A test can be vacuous while a key is still refused by name: 0.1's
  `FutureSpecificationKey` already exits 2, names the key and touches no store,
  so an assertion about a *bad value* must also assert the shape the accepted
  key names (`nonnegative`) and not the release. Caught and rebriefed.
- `doctrine slice verify-vt` attributes results only inside a **closed**
  conformance boundary: a phase's mandates read `UNATTRIBUTABLE` until the phase
  flips to `completed` (which stamps `code_end_oid`), so run it after the
  transition, not only after the commit.
- The Markdown notice reads `selection.walk` directly; no second field was added
  to `Selection`, so the JSON `dangling` and the rendered lines cannot disagree.
- A VT keyword is a **string match in a mandated file**, so a claim can be true
  in prose and still read FAIL: `VT-3`'s `roots-only` failed while the suite said
  "roots only". Put the mandate's exact token in the test that owns the claim,
  and re-read a phase's keyword set against the *finished* file, not the plan.
- The predicate's placement is one line, and its correctness is entirely about
  *where* it sits. The discriminating tests are the frontier of a filtered
  root's walk (empty) and a discovered entity that fails the predicate
  (returned): both mutations — filter after the walk, and filter discovered ids
  too — are caught only by those two assertions.
- `serde_json::Value` is `Eq`, so `Spec` keeps its derives with an
  `Option<Predicate>` inside it; the cost is `cli::Invocation` crossing clippy's
  200-byte `large_enum_variant` threshold, silenced once with a documented allow
  rather than boxing a field §5 declares unboxed.
- `$`-prefix scanning is the whole disambiguation: `operators == 0` is a literal,
  `operators == members.len()` is an operator object, anything between is a
  refusal — so `{}` parses as `Eq({})`, never as an empty operator set.
- `doctrine slice record-delta --start <commit>` takes the phase's entering HEAD
  itself, not `<commit>^`: passing the parent pulls a foreign commit (the R1
  repair batch) into the phase's delta.
- A data fixture earns its red at its own layer: with `entities.json` absent, all
  nine tests failed on the unreadable file — a red for the right reason. Its
  non-vacuity check is an **amputation**, not a prose mutation: delete one
  cluster's `part-of` edge from a copy and exactly the two containment assertions
  go red (`cluster-mind` missing from `traversal.depths`; `q-split` missing from
  the eight) while the entity assertions stay green. A fixture suite that
  survives an impoverished store proves nothing about the graph.
- The normalisation is **order-only**, so the array round-trips through `declare`
  byte-identically and the fixture can prove it rebuilds the store. Re-sorting or
  re-projecting values would have made that check impossible.
- `session`'s prose says nine questions are open while the array holds eight —
  the second stale claim in the store's own text, after `vocab` saying `projects`
  is unused. A fixture assertion on a count must derive it from the entities; the
  prose entities are not authority.
- The privacy ruling is **recorded, not performed**: `PROVENANCE.md` cites the
  operator's waiver so an auditor reads a ruling with a basis, and the phase's
  residual scan is written up as a check on that waiver's premise, never as a
  second verdict. That separation is what keeps `VH-1` a human act.
- `doctrine slice record-delta --commit <S>` refuses to narrow an auto-bound solo
  boundary: `--status completed` had already recorded `2ea81fa..fddbe33`, and the
  safe mode's narrower `[S^, S]` span would have silently un-attributed the paths
  in between.
- A guide pin is a **substring match inside one wrapped literal**, so a clause
  split across a line break reads as absent: `contains("different question")` and
  `contains("no evidence")` both failed while the prose said exactly that across
  two lines. Keep an asserted phrase on one line, or the pin measures the wrap.
- A guard whose only observable is "did it raise" certifies nothing. `Guarded`
  returns the ids **and** the reasons, so a guard that skips `truncation_reasons`
  fails on the reason content and one that ignores `truncated` fails on the
  truncation case; both mutations were run. The real capped call (`limit: 3` on
  the fixture graph) is what makes the guard non-vacuous rather than the
  synthetic envelope alone.
- `tests/fixture.rs` was a black-box suite that had never linked the library.
  Importing `smut::learn` is what lets the shipped example be the executed one,
  and it is additive: the `Scratch`/`Declared` harness is untouched. A guide whose
  example lives only in prose cannot be tested this way — the constants exist to
  be importable.

### Open

- **DEF-1 (PHASE-01 VT-9).** "the reason order when both reasons occur" cannot
  be asserted in PHASE-01: `Reason::TraversalBudget` does not exist until
  PHASE-02, and PHASE-02's own `VT-2` mandates that assertion in the same
  `tests/traversal.rs`. PHASE-01 asserts uniqueness and `["result_limit"]`, with
  the re-cut's reason in the test module. No obligation is dropped — the
  deferred clause is owned by PHASE-02's criterion — so this is recorded, not
  escalated. The seat should confirm the ownership at C2.
- **DEF-2 (PHASE-01 EX-5).** Its "absent for … a query supplying only `depth` or
  `budget`" and "only from/where/select/order/limit" clauses are unobservable
  while `budget` (until PHASE-02) and `where` (until PHASE-03) are refused by
  name: those queries have no envelope at all. PHASE-01 asserts the satisfiable
  set — `get`, `query {}`, from/select/order/limit-only and depth-only — and the
  remaining two become assertable in the phase that accepts each key.
- **DEF-3 (found at PHASE-01 closure; repair belongs to C3).** PHASE-03's `VT-4`
  keywords are `["additivity", "where"]` over `tests/compatibility.rs`. Both
  words are now in that file — `where` from the pre-existing refusal pins,
  `additivity` from PHASE-01's legitimate traversal-additivity comment — so
  `doctrine slice verify-vt 2` reports PHASE-03 `VT-4` **PASS** before PHASE-03
  has written its where-*accepted* pin. The mandate certifies nothing, which is
  exactly the vacuity class the plan review claimed to have swept (it read
  `UNATTRIBUTABLE` only because the file was not yet in the slice's delta).
  Repair under envelope #6: retarget the keyword to something only a
  where-accepted pin produces. C3 must not trust the keyword, and PHASE-03's
  `EX-7` still owns the assertion itself. PHASE-01's own nine mandates all read
  `PASS` after the phase closed its conformance boundary.
- **DEF-2 discharged at C3.** PHASE-01 `EX-5`'s last unobservable clause was the
  `where`-only case ("`traversal` absent for a query supplying only `where`"):
  PHASE-03's `where_accepted_adds_a_predicate_without_disturbing_the_pinned_query`
  asserts exactly the five 0.1 members for a `where`-only query. `budget`-only
  was PHASE-02's. Nothing outstanding.
- **DEF-3 discharged at C3.** `VT-4`'s keywords are `["additivity", "where_accepted"]`
  and the token now sits in the re-cut test's own name and module doc; the PASS is
  earned by the pin that computes a predicate, not by the word `where` being in
  the file since PHASE-01. The pin also refuses to be vacuous: it asserts a
  second query over a field the store *does* hold, so "accept and ignore" fails.
- **RV-012 F-4 is contested** and awaits the responder seat: the raiser accepted
  the §9.3/§9.2/scope/§13.1 repairs but rejected the response on two points —
  (1) DEC-017's prose was listed as swept while it still said only that the store
  owns its copy, and (2) the finding's `vocab` counterexample is impossible
  because `vocab` has a different id from any seed. Point (1) is now satisfied by
  the deferral refactor (DEC-017 carries the supplied-field/body-replacement
  warning); point (2) needs the durable answer corrected, not a design change.
- Section review of `design.md` (all 15, none attested), then the pass
  disposition (`conducted`), then `design-accepted` to lock; `/plan`, `/phase-plan`.
- IMP-004: the epistemic set's own design conversation (not this slice's work).
- DEC-018: the fresh-agent read-through is **waived by the operator** and the
  verdict is *publish* — the ruling and its basis are under
  `## Operator rulings (G0)`. PHASE-04's provenance note records the waiver, so
  an auditor reads a ruling, not an omission. **Discharged at C3**: the array is
  committed (`fddbe33`), nothing removed, no redaction, and the residual scan
  found nothing materially private — see `## C3 — PHASE-04 landed`.
- The design run state is gitignored but live; resume with `doctrine design resume
  SL-002`.

## Design surface triage (exploring, 2026-11-01)

Evidence: `.doctrine/slice/002/research/research.md` (baseline current),
`IMP-001`, the brief §§103–163/184–220, SL-001 design §§4–6/8–9, and the
first-use store.

### Open questions → inquiry map

| Inquiry | Question | Blocking |
| --- | --- | --- |
| inq-1 | What provenance does a traversal result carry, and where does it live in the envelope? | yes |
| inq-2 | What exactly is the frontier, and how do depth, budget and limit affect it? (parent inq-1) | yes |
| inq-3 | Which edges does the result carry, and in what order? (parent inq-1) | yes |
| inq-4 | How is the traversal budget accounted, and when is `traversal_budget` set? (needs inq-2) | yes |
| inq-5 | Where does the two-call idiom live, and what must `learn` say? | yes |
| inq-6 | Which conventions ship, and how is a piped declaration owned/versioned? | yes |
| inq-7 | Is the first-use store promoted as a fixture, and in what form? | yes |
| inq-8 | How does 0.2 stay additive for 0.1 callers, given the pinned envelope? | yes |

Predicates, label semantics and determinism are **pinned by the brief**, not
open: the brief's closed operator list, missing-field semantics,
operator-vs-literal rule, and six determinism rules are acceptance criteria.

### Risks

- A second reference definition in the walker (mitigate: consume
  `entity::scan_refs`; test walker/declare agreement).
- Envelope widening breaking the 0.1 pin (compatibility test fixes the whole
  parsed envelope, including its key set).
- `backlinks` is O(store) per hop and not bounded by `budget`; document it.
- The fixture is a real personal record; promotion is a design decision with
  provenance, and committing it is outward-facing.
- Conventions content is unendorsed; ship only what has evidence (hygiene +
  epistemic records) and keep the two epistemics separate.
- Determinism tests that only exercise lexical order pass without testing the
  named-field rule the code must impose.

### Assumptions

- Store fits in memory; whole-store scans are acceptable at folk-database size
  (SL-001 §8.2, still true: 66 entities).
- No new crate is needed; `serde_json::Value` + `BTreeMap` are the substrate.
- The on-disk format does not change in 0.2 (`format_version` 1).
- Human-only review policy remains the run's default.

### Shaping decisions (to record as DEC-012+)

- Provenance is a conditional envelope member, present exactly when traversal
  was requested, keeping non-traversing 0.1 envelopes byte-identical.
- One derivation: induced edges are the single edge source; per-object depth is
  a scalar map; the frontier is a separate id list.
- Edges are filtered to selected ids after the limit, derived from full
  entities before projection.
- Conventions ship as embedded text, printed only, with a fixed seed id so a
  re-install is an update of that entity; never validated, never selected.
- The fixture is a normalised single JSON array declared into a scratch store,
  with a provenance note; not `.smut` internals.

### Constraining governance

- Brief: closed operator set, roots-only `where`, budget does not bound initial
  scans/backlinks, depth is selection not truncation, envelopes flat.
- `criterion-scope-smut`: no schema, no modelled containment, no in-store audit.
- DEC-011: 0.1 invocations stay valid; compatibility is tested, not assumed.
- IMP-001 "Do not build": no validating schema, containment, in-store audit,
  doctrine promotion.
- IMP-002 (Mermaid/1.0) and IMP-003 (0.1.x fixes) are out of scope.

## Design draft notes (2026-11-01)

- Section ids are not numerically aligned with the document: `materialise` orders
  sections lexically by id, so to get a readable 1–15 order the bodies were
  assigned to the ids in lexical order. Mapping: §1 sec-1, §2 sec-10, §3 sec-11,
  §4 sec-12, §5 sec-13, §6 sec-14, §7 sec-15, §8 sec-2, §9 sec-3, §10 sec-4,
  §11 sec-5, §12 sec-6, §13 sec-7, §14 sec-8, §15 sec-9.
- Self-review correction before the human pass: `edges` is the induced subgraph
  of the returned entities, derived in the envelope after the limit — not the
  subset of edges the walk crossed. DEC-014 was amended accordingly; the walker
  no longer carries an `Edge` list, and a returned-but-unexpanded node still
  contributes its edges. This makes 1.0's renderer able to project the envelope
  rather than derive edges a second time.
- Review pass RV-005 was opened at revision 25 and went stale when the
  correction changed section fingerprints; a fresh pass is opened from the
  corrected materialisation.
- RV-007 review pass (agent pass, operator disposes and accepts). Four findings,
  all `fix-now`, repaired in place rather than carried:
  - F-1 `major` (dedupe) — §10.3 suppressed the `traversal` member for `from: []`,
    contradicting §8.1 and invariant 5. Presence keys on the request only; the
    row now says the member is present with empty `depths`/`edges`/`frontier`.
    Owner: §8.1 + invariant 5 + DEC-012; the sweep found no other exception.
  - F-2 `major` (refresh) — the scope said edges are in *discovery* order while
    §7.2/§8.2 and DEC-014 say *result* order; DEC-012 said encounter order.
    Resolution: result order wins. Edited `slice-002.md` (objective 5 and the
    settled table) and DEC-012 (choice facet and body).
  - F-3 `minor` (dedupe) — §14.1 gave the walker an `Edge` type §4 and DEC-014
    abolished. Removed; the row now points at §7.2.
  - F-4 `minor` (review) — §13.1 pinned 66/44/22 from the pre-redaction store
    while DEC-018's privacy gate may change the fixture. §13.1 now pins the
    acceptance properties plus the committed array's counts, measured at the
    committing phase; §2 keeps the figures as source evidence.
- Section fingerprints changed with the repairs, so every section attestation
  must be recorded after them, never reused from before.
- RV-007 went stale on those repairs, as RV-005 did before it. The run opens its
  own pass on entering `reviewing`; opening one by hand (RV-008) does not rebind
  it. The repair is a stage round trip (reviewing → drafting → reviewing) and the
  pass the run opens there (RV-009) is the one to conclude. Do not renumber or
  reconcile the stale ledgers: RV-005…RV-008 are the pass history.

## What a further review pass would probe (RV-009)

If this design came back for another pass, it would go at these, in order — none
is a known defect, each is the place a later reader is most likely to be wrong:

1. **The induced-edge derivation and `select`.** §8.2 derives edges from full
   entities of returned ids. The claim "`select` cannot erase connectivity" is
   true for edges; check that it is also true for what a Markdown client does
   with them, since the renderer does not print edges.
2. **Budget accounting under a `where`-filtered root set.** The budget counts
   expansions, not roots; a filtered root list changes what is expandable, so
   the boundary tests in §13 must construct the queue state, not reason about it.
3. **Two accounts that are legitimately separate.** `frontier` is walk-scoped
   and `depths`/`edges` result-scoped; a pass would re-test that no helper
   function quietly cuts one with the other's bound.
4. **The conventions seed's upsert claim.** §9.3 leans on 0.1's patch semantics
   for "fields the user added survive"; that is 0.1 behaviour, so a pass would
   confirm it with the existing patch tests rather than a new fixture.
5. **`from: []` and `missing`.** The repaired presence rule makes the member
   unconditional on the request's keys; the remaining risk is the interaction
   with `missing`, which is untouched.

Nothing here blocks the lock: each item is a check for the phase that builds it,
and each has a place in §13.1's test map.

## RV-009 — adversarial pass (codex), 2026-11-01

Independent second pass by the operator's codex reviewer, after the RV-007
repairs. Three findings, all verified against the text and all `fix-now`:

- F-1 `major` (review) — **the two-call recipe laundered truncation.** The guide
  recipe extracted ids from a first response without reading `truncated`; with
  101 discovered entities and the open question ordered last, the second call
  reports "no open questions" in a `truncated: false` envelope. The acceptance
  fixture is under the default limit, so the suite could not have caught it.
  Repaired: §9.4 normative guard, invariant 13 ("composition does not launder
  incompleteness"), §12 risk row, §13.1 Guide + new Composition row, §13.3
  scenario 6 (over-cap graph and budget-cut variant), scope objective 9, and
  DEC-016's body + consequences.
- F-2 `minor` (refresh) — §10.3's `follow: []` row promised "no edges, empty
  frontier", contradicting §7.2/§8.2 (induced edges) and the depth/budget
  frontier rules. Repaired to "no outgoing discoveries", with the same
  qualification on the named-field row. Sweep: every no-edge promise.
- F-3 `minor` (dedupe) — §9.2 defined `retracted` and then said `superseded` is
  the status for a retraction. The clause goes; retracted = withdrawn,
  superseded = replaced (the `supersedes` edge). §2 and §13.1 already used the
  distinction.

Lesson on the machinery: repairs to a materialised design cost the open pass its
fingerprints, and the run only rebinds its pass on re-entering `reviewing`. So
the order that wastes least ceremony is **raise → dispose → repair everything →
adopt → one stage round trip → one pass → conclude**. Repairing between passes
buys a fresh stale ledger each time (RV-008, RV-009, RV-010 all paid for this).

## RV-011 — fixture-claim correction (found while checking thread 3)

- F-1 `minor` (refresh) — §2 and the §13.1 fixture row claimed the acceptance
  graph holds "a retracted finding". It holds none: 28 statused entities with no
  `retracted` anywhere, and the one retired finding (`finding-smut-16`) is
  `superseded`. The store's own `vocab` says `superseded` retired it "with its
  retraction intact" — the same conflation RV-009 F-3 found in the draft, sitting
  in the evidence the draft was written from. Repaired in §2, §13.1, the scope
  objective and §9.2, which now names the conflation as the drift the distinction
  corrects.
- Consequence for planning: the fixture property list is **superseded finding +
  settled finding + mutual cluster + unreferenced entities**, and no retracted
  record. Do not manufacture one to satisfy an old sentence.

## Research thread 3 (delegated)

`.doctrine/slice/002/research/raw/thread-3-doctrine-epistemic-schema.md` — what
doctrine's own record model does (24 kinds, 7 knowledge kinds with kind-specific
facets, per-kind status vocabularies, derived liveness, outbound-only relations,
supersession that requires a successor) and a comparison table against the §9.2
draft. `.doctrine/slice/002/research/raw/thread-3-synthesis-proposal.md` — its
own synthesis for what the set should say.

Corrected by the orchestrator: the findings file first claimed no verb prints a
schema; `doctrine design contract --format prompt` does (embedded
`knowledge::RecordKind`), it is just design-scoped. Everything else spot-checked
against the store and the CLI.

## RV-012 — adversarial pass 2 (codex), 2026-11-01

Four findings (three from the operator's codex pass, one raised here), all
`fix-now`, all verified before repair:

- F-1 `major` (dedupe) — **DEC-016's recipe walked containment the wrong way.**
  `follow:["part-of"]` from `session` returns the session alone (clusters point
  *at* the session), and the second call then answers "no open questions" in a
  complete-looking envelope — the guard cannot help, because nothing was
  truncated. `backlinks:["part-of"]` at depth 2 reaches the eight open questions
  (verified: `part-of` child→container, 5 clusters → session, 8 open questions).
  Repaired in DEC-016's body, §9.4, §13.1 Guide, §13.3 scenario 2 and the scope:
  the direction is stated in the recipe, the guide's example is the one the
  fixture test runs, and the test asserts the eight ids *and* that the `follow`
  variant reaches none. The store's `finding-smut-18` carries the same inverted
  example — data, untouched; the lesson is recorded: never copy an example out of
  the store without running it.
- F-2 `minor` (review) — the composition control had no executable subject. §13.1
  now names the discriminating fact (target stored and open, absent from the
  truncated first result, second envelope complete-looking and empty), the naive
  helper kept in the suite as the wrong answer, and the guarded conclusion that
  must differ. No prior-response input, no propagation: the guard is client-side
  and §9.4 says so.
- F-3 `minor` (refresh) — DEC-018 and the §14.1 row still fixed the fixture to
  pre-redaction numbers and a "retracted claim". Both repaired: counts name the
  source snapshot, the committed array records its own counts in PROVENANCE.md,
  the retirement case is the superseded finding, and provenance records redaction
  scope and who resolved it.
- F-4 `minor` (refresh, raised here) — §9.3 called the seed upsert "the store owns
  its copy" while the seed supplies `body`: a hand-amended convention body is
  silently replaced on re-install. The operator's own `vocab` is exactly that
  amendment. Repaired in §9.3, §9.2's printed-prose note, the scope and the
  conventions test row.

## Scope reduction: the epistemic-records set leaves 0.2 (2026-11-01)

Operator direction: the epistemic schema is not converged and deserves its own
design conversation. Removed from the design, not repaired again.

- **What ships now:** `smut conventions` with **one** set, `hygiene`, plus the
  mechanism unchanged — fixed-id seeds, `--format json`, pipe through `declare`,
  never validated, never auto-selected. Adding a set later is a file plus a match
  arm, so the deferral costs 0.2 nothing.
- **What moved:** the draft taxonomy (13 types, 8 statuses, 6 relations, `scope`),
  the five open questions (per-kind statuses, the missing disposition pair,
  `source`/`refutation` as types, does the list survive a query, whose claim) and
  the doctrine comparison now live on **IMP-004**
  (`doctrine show IMP-004`), linked `originates_from SL-002`, `after SL-002`,
  tags `conventions`/`epistemic`/`design-deferred`. The evidence stays where it
  was gathered: `research/raw/thread-3-doctrine-epistemic-schema.md` and
  `thread-3-synthesis-proposal.md`.
- **DEC-017 narrowed** (title, context, choice, consequences): one set in 0.2;
  the seed-body replacement caveat folded in; the epistemic set recorded as
  deferred. Its slug still reads `conventions-two-sets-...` — durable, left
  alone.
- **Kept in 0.2 because it is lifecycle advice, not a taxonomy:** the
  retracted-vs-superseded distinction moved into `hygiene` item 7, with the
  store's own `vocab` as the evidence. Four review findings landed around that
  taxonomy; this one was about a real drift in the operator's record.
- **Deltas:** §3 caveat, §9.2 (rewritten, one set), §9.3 ids, §10.2 assumption
  dropped, §11 residual gate replaced by a pointer to IMP-004, §12 risk row,
  §13.2 gate dropped, §13.3 scenario 5 rewritten (hygiene round-trip + the
  deferred name refused by name), §14.1 file list, §15 DEC-017 row; scope
  objective 8, out-of-scope, risks row, settled table, verification item 5.
- **Gate removed:** DEC-017 no longer requires operator approval of a schema
  before a set ships, because no such set ships. §9.2's review is now ordinary
  review, not approval.

## Rebinding, and the F-4 dispose (2026-11-01)

Two ledger acts, no design change. Nothing was built or run this session; every
claim below is a reading of the design, of DEC-017, or of a committed ledger.

- **F-4 disposed (RV-012).** Contested → answered, `fix-now`, route `refresh`
  kept. The response concedes the raiser's point 2 — the finding's own instance
  is impossible, because seed ids are fixed per set (`conventions-hygiene`) and
  an id-keyed upsert cannot touch `vocab`, which is evidence of the *practice* of
  amending convention prose rather than a record this pipeline overwrites — and
  records point 1 as satisfied after the fact by `603c718`, which rewrote
  DEC-017's body, choice and consequences to say the seeded `body` is replaced
  on re-install. No mechanism added. **Verification of F-4 is the raiser's act,
  not the responder's**: F-1…F-3 were self-verified only because that seat raised
  them.
- **Rebinding (revision 55 → 57).** The refactor was adopted after RV-014 had
  opened, so the run reported its pass as `STALE`. Rebinding is a stage round
  trip — `reviewing → drafting → reviewing` — after which the run opened **RV-015
  bound to the current revision**; opening a ledger by hand does not rebind it.
  The same lesson as RV-008/RV-010/RV-013, and the reason those three were paid
  for and superseded.
- **RV-015 is a rebinding pass, not a fifth window.** Concluded with its basis
  stated: the whole adopted delta of `603c718` plus a cross-reference sweep for
  references to a shipped second set or an operator schema approval. Nothing new
  found, nothing raised. The substantive examination of this design remains
  RV-007…RV-014.
- **Open exposure, already governed.** §2's first-use-store paragraph and §9.2
  item 7 both describe the operator's store's `vocab`. If the DEC-018 privacy
  read-through changes it, §11's gate already rules: the affected section is
  revised before the phase that depends on it. Not a finding — a dependency the
  privacy phase carries.

## F-4 checked against the code, and the clause it earned (2026-11-01)

The contest was about the accuracy of a durable answer, so the answer was
checked against the mechanism rather than against its own prose.

- **Holds as a mechanism.** `src/declare.rs:270-306`: `apply` keys the upsert on
  `declaration.id`, clones the stored fields, then applies `Op::Set` per declared
  field — so a supplied `body` overwrites and an unnamed field survives. That is
  §9.3's pair exactly.
- **The instance does not hold.** The live store's `vocab` is `"id": "vocab"`
  (`interview.local/.smut/…`), carrying the second-session amendment in its body;
  no entity in the store has an id beginning `conventions`, so a
  `conventions-hygiene` seed is a creation. Id-keyed upsert cannot reach it.
- **The precision found by looking.** The replacement is not invisible:
  `src/declare.rs:116` emits `previous` for an updated entry unless `--compact`,
  and `src/learn.rs:72-76` documents it. §9.3 said "without a warning", which was
  right, but stopped short of saying the old body comes back in the reply.
- **Delta (operator-directed, revision 58):** §9.3 gains that clause and §13.1's
  Conventions row gains the assertion that `previous` carries the replaced body
  and `--compact` omits it. Adopted at rev 58; the pass went stale as it must,
  so the stage round trip ran again (rev 59, 60) and **RV-016** is the bound
  pass, concluded on the delta.
- **Lesson, third time paid for:** any edit adopted mid-pass stales that pass.
  Edit while in `drafting`, or accept the round trip and a fresh ledger — the
  ordering is cheaper than the ledger.

## Plan review round: obligations and sequencing (2026-11-02)

Two read-only reviewers audited the plan as authored (commit `ab1011b`) against
the locked design and the tree; an integrator adjudicated every finding on
evidence and was the sole writer. Design untouched, design run not reopened.

- **Reviewer A — obligation coverage.** 236 obligations enumerated across
  §1–§15, the scope objectives and the closure items: COVERED 197, WEAK 35,
  VACUOUS 1, MISSING 2, MISTRANSCRIBED 1; findings F1–F18. Verdict on the plan:
  not executable as-is.
- **Reviewer B — sequencing and codebase fit.** Verdict **BLOCK**: B1 (accepting
  `follow` falsifies the executed README transcript, which design §14.1 assigns
  to PHASE-07, so PHASE-01's "tree green" was unreachable) and B2 (`FUTURE_KEYS`
  retired at PHASE-02 while `where` was still refused *by name* as a future key,
  breaking six asserting sites and contradicting the shipped guide). Majors
  M1–M8, minors m1–m10; 20 premises verified TRUE against the source.
- **Dispositions:** 38 findings, CONCEDED 36, ESCALATED 2, REJECTED 0. Nothing
  was accepted on a reviewer's authority: each premise was re-checked against
  `design.md` or the source (the "already green" keyword sets, `FUTURE_KEYS`,
  `select::run`'s five parameters and the absent `clippy.toml`, the notice's
  `None` return, the five clusters in the source store, the `where`-refusal
  pins).
- **Repaired (plan only).** B1: the README console block is re-recorded at
  PHASE-01 (`EX-11`) and `tests/readme_example.rs` re-cut beside it, so PHASE-01
  can end green; PHASE-07 extends the example. B2: `FUTURE_KEYS` retires at
  PHASE-03, the phase accepting its last key; PHASE-01 keeps `where` and
  `budget` by name, PHASE-02 keeps `where`. M1/M3/M8: every assertion a phase
  falsifies is named in a criterion — `src/spec.rs`'s five-key loop, the
  `where`-refusal pins in `tests/compatibility.rs`, `tests/cli_contract.rs` and
  `tests/selection.rs`, the `FutureSpecificationKey` rows in `src/error.rs`, and
  `learn`'s 0.1-promise test — re-cut, never deleted (DEC-019). M6: the
  `select::run` interface decision is now taken at PHASE-01 (`EX-13`). M7: the
  notice's trigger, line order and literal triple form are PHASE-01's (`EX-14`),
  dangling-only case included; PHASE-02 adds only the `traversal_budget` line.
  m1: the walk is unbounded (`u64::MAX`) until `budget` exists, so PHASE-01
  cannot truncate silently. Also: edge dedupe and result order, sibling `where`
  keys ANDed, §13.3 scenarios 1/3, §12:788's blanket row obligation and the four
  §10.3 rows it was missing, the guide's remaining guard clauses and its
  no-propagation limit, the `$`-prefixed field limit, the label vocabulary,
  `--format text`, the seed's version source (`env!("CARGO_PKG_VERSION")`),
  `src/conventions/hygiene.md`, duplicate-label collapse, `Walk`'s derives, and
  §14.2's unchanged set proven by diff where no selector watches the path.
  Appended, never renumbered: PHASE-01 `EX-11…EX-14`, `VT-7…VT-9`; PHASE-03
  `EX-8`, `VT-4`; PHASE-06 `EX-7`; PHASE-07 `VT-3`.
- **Vacuous mandates fixed.** Every VT whose keywords already existed on the
  entering tree was retargeted or re-keyed (`README.md` prose → the suite that
  executes it; `pub fn walk`/`pub enum Predicate` → `#[test]` plus a behavioural
  token; the two pre-green guide and compatibility sets). Every row written this
  round was checked the same way: no VT is satisfiable by its own keywords on
  the current tree.
- **Escalated, not written into the plan.** (1) The privacy read-through's
  negative outcome (§11, DEC-018): a redacted fixture, a synthetic one, or no
  committed fixture changes what the release ships, so PHASE-04 `EX-2` only
  records that the decision blocks PHASE-06/PHASE-07. (2) `design.md`'s line
  references drift from the source: `§1`/`§2` cite `src/entity.rs:31,60` where
  `scan_refs` is at `:71`, and `§9.4` cites `learn.rs:113` where the dangling
  sentence is `:111`. Both are design-text corrections for the next design
  revision; no plan criterion cites them.
- **Verification.** `doctrine slice phases 2` re-materialised all seven sheets
  (state tree regenerated to force it; only the objective lines changed) and
  `doctrine slice verify-vt 2` parsed all 25 mandates — FAIL/UNATTRIBUTABLE
  only, which is correct before implementation. Nothing was built; no `cargo`
  command was run, because the 0.1 suite says nothing about the plan. The full
  disposition table is in `/tmp/sl002-plan-review-integration.md` (session
  artefact, not committed).

## Implementation run — proposed 2026-11-02 (awaiting operator adaptation)

**Status: proposed, revised after the operator's G0 reply; not started.** No phase
is open; nothing in `src/` has been touched for 0.2; HEAD `3e5b6ae`, tree clean,
branch `main`. Revisions at G0: the privacy read-through is waived (below), and a
**review lane** with batched repairs is added to the drive (below). One operator
decision is still outstanding: the design's line-number drift.

## Operator rulings (G0)

Two rulings, recorded here because the phase that depends on each cannot carry
the authority itself. Neither is mine.

- **Privacy (DEC-018 / §11) — consent to publish; read-through waived.** The
  operator's words: "it's probably overcooked in this instance… the only thing I
  can think of was I mentioned watching Slow Horses on tv which I can live with
  the world knowing. I consent to the fixtures, skip the audit." Read as a waiver
  of the *procedure* §11 names (a fresh-agent read-through) and a verdict of
  **publish, nothing removed**: §11 does not assume what a read finds, so no
  section is revised, and `VH-1` is answered before the fixture commit rather than
  satisfied by it. Consequence for PHASE-04: EX-2's provenance note records source
  snapshot, `read-through: waived by the operator <date>, nothing removed, no
  redaction, resolved by the owner`, plus the array's own counts (EX-4).
  **Residual, deliberately kept:** consent was given from recall, not from
  inspection. If the normalised array turns out to contain something materially
  private — named third parties, health, addresses, credentials — I stop at G1
  and show it before committing. That is not a re-litigation of the waiver; it is
  the difference between "nothing I can think of" and "nothing there", and the
  commit is the irreversible act. No secret ever reaches an RV ledger.
- **Code review is a lane, not a gate.** Explicit review was missing from the
  first draft. The operator's direction: retrospective review runs *in parallel*
  with new work, may batch a few phases, and covers maintainability and quality
  as well as correctness; repairs batch too. Adopted as the review lane below.

### Routing and the seat

Governing skill: **`/super-orchestrator`** (pi-native tiering in the main
worktree), with `/phase-plan` + `/execute` as the per-phase stage under it.
`/dispatch` was the alternative and is **unavailable in this checkout**, not
merely unchosen: `bwrap` is absent and `.agents/skills/dispatch-spawn`'s sole
spawn path (`scripts/spawn-confined.sh`) does not exist, so dispatch fails
closed (`bwrap-unavailable`) with no unconfined fallback. Reported, not routed
around. `/capsule-driver` is the same tiering shaped for a Claude harness
(`CLAUDE_CODE_*` env, `TaskStop`, `AskUserQuestion`); the pi equivalents are
installed (`sub-orchestrator` with `allowNestedSubagents`, plus `worker`,
`delegate`, `scout`, `reviewer`, `researcher`), so the pi-native sibling is the
honest choice.

| tier | seat | does |
| --- | --- | --- |
| seat | this session, operator attached | charters, reads handbacks, holds gates, never implements |
| sub-orchestrator | fresh `sub-orchestrator` per charter | `/phase-plan`s the sheet, owns lifecycle, runs gates, commits owned paths |
| worker | `worker`/`delegate` per bounded task | tests (red first) and implementation |
| reviewer | fresh read-only `reviewer` per review batch | raises on its own RV, verifies its own repairs, concludes the pass |

If nested spawning is denied to a child, that is a capability blocker to report,
not permission to implement inline.

### Environment facts (probed, not assumed)

- `cargo 1.100.0-beta.7`; gates are `justfile`: `fmt --check`, clippy
  `--all-targets -- -D warnings` **and** `--all-features`, `cargo test`, then
  `cargo test --features fault-injection`.
- Suite present: `tests/{cli_contract,compatibility,concurrency,durability_crash,readme_example,render,roundtrip,selection,store_corruption}.rs`.
  `tests/fixtures/` does not exist yet (PHASE-04 creates it).
- `DOCTRINE_WORKER` unset; single worktree; **one writer at a time** is a
  precondition of the green-tree invariant, not a preference.
- The plan's entrance criteria make the seven phases a **total serial chain** —
  every phase names its predecessor — so no batch is parallelisable. Charters
  group phases; they never overlap.
- `doctrine dispatch status --slice 2`: `dispatch branch not found` (expected);
  `dispatch plan-next` reads `next: PHASE-01`.
- Fixture lead time is the schedule risk: the DEC-018 read-through is the long
  pole and gated work, so it is requested at G0, not at the start of PHASE-04.

### Charter schedule

| charter | phases | why this boundary | review batch | reviewed while |
| --- | --- | --- | --- | --- |
| C1 | PHASE-01 | Heaviest (14 EX, 9 VT) **and** interface-setting: `Walk`, the `select::run` input value (EX-13), the notice trigger (EX-14), the README re-record (EX-11). Everything after consumes these. | R1: 01 | C2 implements |
| C2 | PHASE-02 → PHASE-03 | Two ordinary phases, serial inside the charter. Ends where the query surface is complete and `FUTURE_KEYS` has retired. | R2: 02+03 | C3 implements |
| C3 | PHASE-04 | Alone: it carries the plan's only `VH`, and the fixture is the run's one irreversible commit. | R3 (starts here) | — |
| C4 | PHASE-05 → PHASE-06 | Two ordinary phases; ends where behaviour and its sentences agree and the guide's example is the one the fixture test runs. | R3: 04+05+06 | C5 implements |
| C5 | PHASE-07 | Alone: the release proof (six scenarios, four gates, conformance diff, version, §14.2 diff). | R4: 07 + all repair commits | seat halt → audit |

Boundaries are committed, green, and handed over (fresh seat context) — never
mid-phase. PHASE-01 alone in C1 is the deliberate cost of its interface weight;
splitting it remains an **escalation**, never a resequencing.

**Concurrency cap: one writer and one reviewer alive at a time.** Two writers in
one worktree make the green-tree invariant untestable; a reviewer is read-only and
may overlap a writer. Repairs are writes: they land when the writer is idle.

### Review lane

Explicit, batched and parallel to implementation — the operator's addition, and
the run's answer to *maintainability and quality*, not only correctness.

- **A batch is a charter's delta.** A fresh read-only `reviewer` child runs
  `/code-review` over the batch's commit range and raises on **one new ledger per
  batch**: `doctrine review new --facet code-review --target SL-002 --title
  "code review of SL-002 PHASE-0N…0M" --raiser reviewer --responder implementer`.
  Findings detail their phase. The whole-delta pre-close lens remains `/audit`'s,
  so R4 covers 07 and the repair commits only — no duplicate inventory.
- **Review over correctness *and* design.** The brief names the axes: duplicated
  reference interpretation, coupling between `walk` and `select`, cohesion of the
  new modules, test brittleness versus behaviour, and whether the code honours the
  letter and spirit of §4–§10. A VT going green is not an answer to any of those.
- **Roles stay separate.** The reviewer raises; the implementer disposes on
  evidence; the reviewer verifies its own findings and concludes the pass
  (RV-012's lesson: verification is the raiser's act). I do not verify a finding I
  raised, and nothing is disposed on assertion alone.
- **Repairs batch, with one exception.** `blocker` halts and is repaired before
  the next phase opens — a wrong foundation costs more than the batch saves.
  `major`/`minor`/`nit` land as **one repair commit per batch** at the charter
  boundary, tree re-gated green, each DEC-019 re-cut named in it. No `follow-up`
  disposition to dodge work; a pathology outside the slice goes to `backlog new`.
- **Tripwires stay per-phase** and are not deferred to the batch: a deleted or
  renamed test without a DEC-019 reason, a waived or uncheckable VT, an
  out-of-scope path touched, or `Deviations: NONE` beside a real divergence.

### Verification cadence

Per phase, at the sub-orchestrator tier and spot-checked at the seat:

1. `/phase-plan` expands the runtime sheet; `slice phase 2 PHASE-NN --status
   in_progress` (flag, not positional).
2. Contract changes: tests written from the criterion and observed **red for the
   right reason** before implementation; the reason is quoted in the sheet.
3. Gates on the phase tree: fmt, both clippy configurations, `cargo test`, and
   `--features fault-injection` (non-vacuity asserted at PHASE-07 `VA-1`).
4. `doctrine slice verify-vt 2` after the phase commit. `FAIL`/`UNATTRIBUTABLE`
   for unstarted phases is expected; a `Fail` on a *completed* phase halts.
5. Adversarial-checks pass: the new suite is run against a deliberately wrong
   implementation where the criterion is a negative control (induced-edge order,
   budget-before-expansion, roots-only `where`), so a green is not vacuous.
6. Commit: path-limited, one commit per phase (trailing knowledge/notes commits
   allowed), sheet flipped `completed`, harvest refreshed in this file.
7. Review: the batch reviewer's raises are triaged at the seat into
   `blocker` (halt) or the next repair commit; the repair commit re-runs steps 3–5
   before it lands, so a batch never ends on a tree the review has not seen.

Seat review is **not** every diff. Mandatory seat review on a tripwire: a deleted
or renamed test without a DEC-019 reason, a waived or uncheckable VT, an
out-of-scope path touched, `Deviations: NONE` on a delta that plainly deviates,
or a green whose evidence names no command. PHASE-01's envelope tests and
PHASE-04's provenance note are read at the seat regardless — one is the release's
central claim, the other is outward-facing personal data.

### Operator gates

| gate | when | what the operator decides |
| --- | --- | --- |
| G0 | now | Adapt/assent this schedule **and** the envelope below; dispose the two open decisions (privacy negative outcome; design line drift) or defer them to G1. Also: who performs the DEC-018 read-through, and with what model exposure — raw store contents must not reach a subagent's context without an explicit ruling. |
| G1 | before PHASE-04's fixture commit | **Answered 2026-11-02**: consent to publish, read-through waived (see `## Operator rulings`). Residual: if the array holds something materially private despite that recall, I stop here and show it before committing. Blocks PHASE-06/07 no longer. |
| G2 | before PHASE-07's commit | The release surface: README claims, the conformance table's unexplained cells, and `0.2.0` as it will be described. |
| halt | after PHASE-07 | The seat stops; `/audit` is a separate stage. |

RV-012 F-4's verification is the raiser's act and is **not** in this run's
envelope; it surfaces at audit if still open.

## Authority envelope (SL-002, 2026-11-02 — binding for this run)

An auditor tests later decisions against this. Two lists, and the boundary is
the test: *can I reverse it inside the phase, with no change to what the release
means or ships?*

### Decide, record, continue — I act, then log under `## Autonomy log`

1. **Implementation anatomy inside the design's named surface**: module
   decomposition, private types and helpers, field and variant names beyond the
   members §4/§8 enumerate, and where a helper lives (`src/walk.rs` vs
   `src/select.rs`). No new public API, no new module outside §14.1's touch-set.
2. **Function signatures**: the shape of `select::run`'s single input value
   (EX-13 fixes *one value*, not its fields), `Walk`'s constructor, the
   interpreter's entry point. Signatures are mine; the pipeline's order is the
   design's.
3. **Test anatomy**: test-function names, module layout within the mandated
   `test_file`, and whether a new suite copies `tests/cli_contract.rs`'s
   `Scratch` (the plan's answer: copy it; no `tests/common` is created in 0.2).
   VT `test_file` and keyword sets are fixed by the plan.
4. **Commit granularity and message shape**: one commit per phase, path-limited,
   plus trailing notes/knowledge commits; repo's `type(sl-002): …` convention.
5. **Appended criteria**: new `EX-`/`VT-`/`VA-` ids appended (never renumbered,
   never reused), and appended task rows in the runtime sheet.
6. **Criterion-text precision**: a wording change that preserves the obligation
   (naming the helper, the exact literal, the assertion site), and correcting a
   VT path/keyword that the tree proves wrong — including a keyword that would
   certify nothing. A wording change that *narrows* the obligation is #4 below.
7. **Re-cutting a falsified test** (DEC-019) where a criterion already names it:
   the test is renamed/re-asserted with its reason in the module, never deleted.
   A falsified test **no** criterion names is a plan defect: I record it, fix it
   the same way, and report it in the handback rather than pretending it was
   covered.
8. **Fixture mechanics that are not the verdict**: array normalisation and sort
   order, `PROVENANCE.md`'s shape, the declaring helper's form, which of EX-3's
   properties get their own assertion. The verdict and the redaction content are
   the operator's (G1).
9. **Additive strictness**: running clippy `--all-features` and the
   fault-injection suite at every phase, not only PHASE-07. Adding checks is
   never an escalation; removing one always is.
10. **Prose the plan mandates** (`learn`, README, conventions prose, provenance
    note): wording is mine where the sentence asserts only behaviour that exists
    (`VA-1` is the check); the recipe's direction, the guard's clauses and the
    no-propagation limit are fixed by §9.4/`DEC-016` and are not rewritable.
11. **Worker operation**: brief composition, model inheritance from the
    operator's configuration, one focused rebrief per worker, and recording a
    worker miss. Two failed attempts on one phase ends #7 below.
12. **Review operation**: batch boundaries, reviewer briefs, which findings join
    which repair commit, and a finding's disposition when the repair preserves the
    design's behaviour (naming, cohesion, duplication, test shape, error text).
    Review adds a second reader, not a second authority.

### Stop and escalate — I do not decide these

1. **Any design deviation or semantic change**, and anything that would need the
   locked design run reopened (locked, no outbound stage edge: it is an operator
   act). Includes "the design says X and the code needs Y".
2. **Scope change**: a behaviour no criterion asks for; touching a §14.2
   unchanged path (`src/entity.rs`, `src/order.rs`, `src/store/**`,
   `docs/store-format.md`, `justfile`, `flake.nix`); a new crate or dependency;
   `format_version` or the on-disk format; projecting the edge set into a
   renderer (IMP-002's work).
3. **Plan shape**: phase insertion, split, merge, reorder or deletion
   (PHASE-01's split included), or renumbering/deleting/retitling criteria.
4. **Weakening**: dropping or waiving a criterion for schedule; marking a phase
   complete on a red, partial or unrun gate; relaxing a bound, determinism rule,
   guard or negative control; accepting a `verify-vt` `Fail` on a completed
   phase.
5. **The two outstanding operator decisions**: the privacy gate's negative
   outcome (redacted subset / synthetic store / no committed fixture — changes
   what ships) and reopening the design run for the §1/§2/§9.4 line drift.
6. **Outward-facing or hard-to-reverse acts**: committing the personal fixture
   (the G1 verdict itself is the operator's), pushing, publishing, or describing
   0.2 anywhere outside this repo.
7. **A red I cannot attribute**: the same phase failing after two
   sub-orchestrator attempts, a conflicting writer in a phase-owned path, a CLI
   refusal that needs a new user act, or a criterion that cannot be satisfied as
   written. Report with the evidence; do not reinterpret the criterion.
8. **Review-ledger acts**: disposing, verifying, contesting or concluding any
   `RV` finding (RV-012 F-4 included) is the raiser's/operator's seat. My
   surface is this envelope, the phase sheets and this file.
9. **A finding the envelope forbids**: a review may not launder what #1–#4
   forbid. A finding whose repair would be a design deviation, a scope change, a
   dependency, or a weakened guard is escalated with its evidence — never quietly
   repaired because a reviewer asked. A `tolerated` disposition on a finding that
   contradicts the locked design is likewise escalated.

**The invariant protected throughout**: every phase ends on a green tree, and a
charter never opens on a red or half-landed one.

## Autonomy log

One row per envelope decision actually taken: phase · decision · reason ·
decided / deferred / escalated. Empty until a phase opens.

| date | phase | decision | class | reason |
| --- | --- | --- | --- | --- |
| 2026-11-02 | — | Review lane adopted: one `code-review` RV per charter batch, review parallel to the next writer, repairs batched to the boundary | decided (operator-directed) | Explicit review was missing; maintainability and quality need a seat that is not the implementer |
| 2026-11-02 | — | DEC-018 read-through waived; PHASE-04's provenance note must record the waiver, not omit the step | decided (operator ruling) | Owner's consent, given from recall; recorded verbatim so the gate reads as satisfied-by-ruling |
| 2026-11-02 | PHASE-01 | `Fields`, `Walk`, `Dangling` live in `src/walk.rs` and `src/spec.rs` imports `walk::Fields` | decided (#1, #2) | §14.1 assigns the three types to `walk.rs`; `Spec` already holds `order::SortKey`, so the same direction of dependency |
| 2026-11-02 | PHASE-01 | `select::run` takes one private `Request<'a>` value (roots, fields, keys, limit, follow, backlinks, depth); `select`/`get` stay the only constructors; no `budget` field | decided (#2, EX-13) | EX-13 fixes *one value*, not its fields; PHASE-02 adds `budget` and PHASE-03 adds `where` to the same value, so clippy's argument threshold is never the design constraint. `get` still passes `Limit::unlimited` and adds no traversal |
| 2026-11-02 | PHASE-01 | The walk is called with `u64::MAX`; no budget-shaped parameter or field is added | decided (#2, EX-3, plan m1) | `budget` is refused by name until PHASE-02, so this phase must not be able to truncate; `exhausted` is false throughout |
| 2026-11-02 | PHASE-01 | `src/walk.rs` implements the §7.1 budget check even though this phase never exercises it | decided (#1, #2) | §4's signature and `Walk::exhausted` are locked design, and `exhausted` cannot be computed without the check; PHASE-02 only makes the key real. The key itself stays refused |
| 2026-11-02 | PHASE-01 | Wildcard-in-array and bad-`depth` refusals reuse `Reject::InvalidSpecificationValue`; no new variant, `src/error.rs` untouched | decided (#6) | §14.1's error additions list no traversal-form variant, and making one public would be a new public API |
| 2026-11-02 | PHASE-01 | `Fields::Named` collapses duplicates at parse (first occurrence) *and* the walker tolerates duplicates | decided (#1, #6) | §5 says duplicates are a set for edge selection that never reorder the first occurrence; VT-7 asks for the collapse to be a grammar-level assertion, so it is enforced where the list is built |
| 2026-11-02 | PHASE-01 | `Walk.depths` records every discovered id; `envelope` filters to returned ids; the notice reads `selection.walk` directly | decided (#1, #2) | §8.2 makes `depths` result-scoped and the frontier/dangling walk-scoped; one map plus one filter keeps a single account of the walk |
| 2026-11-02 | PHASE-01 | Dangling triples deduped with a `BTreeSet` guard beside an encounter-order `Vec` | decided (#2) | §7.2 "deduped by `(source, field, target)` … in encounter order"; a set alone loses the order and a vec alone repeats |
| 2026-11-02 | PHASE-01 | `depths()` panics on a returned id the walk has no depth for, rather than omitting it | decided (#1) | A silently omitted depth is an envelope lying about a returned object; it matches `project_one`'s existing precondition style |
| 2026-11-02 | PHASE-01 | Test work split into three file-disjoint red waves (`tests/traversal.rs` VT-2; then VT-3/VT-4 appended; then selection/render/cli_contract/compatibility), then four implementation briefs, one writer at a time | decided (#3, #11) | The green-tree invariant needs one writer; only genuinely disjoint files ran in parallel. New suites copy `tests/cli_contract.rs`'s `Scratch`; no `tests/common` and no third harness fork |
| 2026-11-02 | PHASE-01 | One worker rebrief: `a_bad_depth_is_refused_before_any_store_access` was green for the wrong reason | decided (#11) | 0.1's by-name refusal already exits 2, names `depth` and touches no store. The rebrief added the assertions only an accepted-and-validated key can satisfy (`stderr` contains `nonnegative`, not `0.2`). Worker miss recorded |
| 2026-11-02 | PHASE-01 | DEC-019 re-cuts: `src/spec.rs`'s five-key loop → the two keys still future; its two exhaustive-claim tests renamed to seven keys / `the 0_1_keys`; `tests/compatibility.rs`'s end-to-end example renamed and extended with the brief's closing dangling step; `tests/readme_example.rs`'s 0.2-refusal test → the traversal assertion | decided (#6, #7, EX-12) | EX-12 names the five-key loop and the compatibility example; the other two names asserted something this commit makes false. Renamed/re-asserted, never deleted, each with its reason in the test module |
| 2026-11-02 | PHASE-01 | README's console block re-recorded by executing the built binary; five prose spots re-cut (0.1/0.2 boundary, the block's intro sentence, "what the transcript shows", the `## Commands` key list, the sharp-edge bullet) | decided (#10, EX-11) | EX-11 requires the block to be the binary's own transcript in this commit; the `## Commands` key list is falsified by accepting three keys and is covered by EX-11's general "prose this commit falsifies" clause. The `## Build` version line stays — the crate is still 0.1.1 |
| 2026-11-02 | PHASE-01 | DEF-1 (VT-9's both-reasons clause) and DEF-2 (EX-5's `where`/`budget` clauses) recorded, with the obligations left to PHASE-02/PHASE-03's own criteria | deferred, not escalated (#6, #7) | Neither drops a slice obligation and neither narrows a bound: each clause is unobservable until a later phase accepts the key that makes it expressible, and the later phase's criterion already owns it. Recorded in the sheet and in this file's `### Open`, and reported in the C1 handback |
| 2026-11-02 | PHASE-01 | Inherited dirty `.doctrine/slice/002/slice-002.toml` (`ready`→`started`, a predecessor's lifecycle write) adopted into the trailing notes commit | decided (lifecycle artefact) | It is the slice's own lifecycle state, not unrelated work; it rides the notes commit rather than being staged into the phase commit, and no `git add .` was used |
| 2026-11-02 | PHASE-06 | The seat wrote the phase in-session; no worker child was chartered | decided (#11; charter) | The parent's charter names this seat the sole writer, and a worker would have needed the example constants handed to it first, which is the whole phase |
| 2026-11-02 | PHASE-06 | The examples live as `pub const EXAMPLE_TRAVERSE`/`EXAMPLE_FOLLOW`/`EXAMPLE_FILTER` beside `GUIDE`, printed as escaped literals inside the guide and pinned by `GUIDE.contains(..)` | decided (#10, EX-5) | The plan's own T3 answer. `concat!` would need a macro to splice a literal into a `const &str`; the pin closes drift by test, and `tests/fixture.rs` then executes the same bytes through `smut learn` and `smut query` |
| 2026-11-02 | PHASE-06 | `EXAMPLE_FILTER` keeps `IDS` as a documented placeholder for the id array the first reply returned; the fixture substitutes it | decided (#10, EX-5) | The guide cannot print ids it does not have. Substituting the ids is the one step the recipe leaves to the caller, and `GUIDE.contains(EXAMPLE_FILTER)` still holds the request byte for byte |
| 2026-11-02 | PHASE-06 | The banner lost `0.1`, and "`--format markdown` is the only format in 0.1" became "Markdown is the only render format. A Mermaid diagram arrives in 1.0." | decided (#10, EX-4) | The crate version is PHASE-07's to move, so a figure in the guide is false in one direction or the other; the Mermaid-as-1.0 claim is design (§14.1) and stays |
| 2026-11-02 | PHASE-06 | Two assertions added beyond the inherited set: `a_complete_looking_second_envelope_is_no_evidence_about_the_first` and `the_guide_teaches_exactly_the_nine_keys` | decided (#3, #9, VA-1) | Additive strictness. The first proves the guard is not vacuous on a real capped call (the fixture's own graph at `limit: 3`); the second is the plan's "teaches exactly nine and no other" row, read off the guide's own key block rather than from a second list |
| 2026-11-02 | PHASE-06 | One inherited doc cross-reference corrected in a separate `docs(sl-002)` commit (`a6ddb25`) instead of folded into the phase commit | decided (#4) | RV-019's review commit landed on top while this seat worked, so HEAD had moved and an `--amend` landed on the reviewer's commit; that was undone from the reflog before anything left the machine and the reviewer's commit is byte-identical to its original (`991a831`). The correction therefore rides its own commit and the phase boundary stays `46c6699`. Recorded as the run's one operational miss |

### G0 closed (2026-11-02)

Operator assented to the revised run record. Two further rulings:

- **Design line-number drift — deferred to reconcile.** Operator: "deal with it at
  reconcile". Recorded, not repaired mid-slice; the locked run stays closed. The
  audit/reconcile stage owns either the `REV` that corrects §1/§2/§9.4's line
  references or the recorded decision to leave them. This is a ruled deferral, not
  an omission: no criterion depends on the drift, and nothing in the run reads
  those lines.
- **Assent to the charter schedule and the authority envelope** as recorded:
  five charters (C1…C5), the review lane with per-batch `code-review` RVs and
  batched repairs, gates G1 (answered) / G2 / halt-before-audit.

C1 (PHASE-01) opens on this basis.

### C1 — PHASE-01 landed (2026-11-02)

Charter: C1, PHASE-01 only, one writer, stop before PHASE-02. Entering HEAD
`b47dd79`; phase commit `45983c7`; trailing notes commit after it. Phase flipped
`in_progress` at `b47dd79` (so `code_start_oid` is the entering tree) and
`completed` at `45983c7`.

Cost: **ten worker children** (three red-test briefs + one rebrief, four
implementation briefs, one documentation/test brief; the rebrief is the only
repeat on a single area). Every worker's green claim was re-run by the
sub-orchestrator. Gate and negative-control evidence is in the phase sheet and
in the C1 handback; the sheet is disposable, so the durable facts are in
`## Harvest` above and the decisions in the table above.

What the seat should look at: the two `blocker`-class risks this phase owns were
checked by mutation rather than by assertion — edges in discovery order instead
of result order, and `follow: []` treated as absent both turn the suite red —
and the dangling-only notice trigger is the third. `VA-1` was checked by grep:
`scan_refs` is the only reference reader in `src/walk.rs` and in the envelope's
induced-edge derivation in `src/select.rs`.

## C1 close — seat verification (2026-11-02)

The charter child was killed by the harness's 30-minute wall **after** it had
committed the phase, flipped the sheet, closed the conformance boundary and run
`verify-vt`; it died mid-way through writing this file. Nothing was rerun and
nothing discarded: the trailing notes write was inspected and adopted, and the
in-flight `slice-002.toml` (`ready`→`started`) is the seat's own lifecycle write
from G0, not the child's.

**Verified at the seat, not taken on report** (the rules say a worker's green is
advisory):

| check | result |
| --- | --- |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test` | 301 + 9 + 32 + 10 + 4 + 0 + 3 + 9 + 4 + 8 + 12 + 25 + 0, all pass |
| `cargo test --features fault-injection` | 314 + the same suites, all pass — the gated suite ran 13 more tests, so the feature is not vacuous |
| `doctrine slice verify-vt 2` | PHASE-01 VT-1…VT-9 **all PASS**; PHASE-02…07 FAIL/UNATTRIBUTABLE as expected |
| `git show --stat 45983c7` | 12 files, +2726/−75; `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix` **untouched** — §14.2 proven by diff, not by selector |
| boundary record | `.doctrine/state/slice/002/boundaries.toml`: PHASE-01 `b47dd79..45983c7`, provenance `solo` |

`PHASE-01` is `completed`; lifecycle `started`; phases 1/7.

### The child's three defects — adjudicated

All three are the same class: a mandate that cannot fail, or cannot be satisfied,
because a later key does not exist yet. That is the vacuity class the plan review
claimed to have swept, so each is recorded here rather than absorbed silently.

- **DEF-1 — PHASE-01 `VT-9`'s "reason order when both reasons occur".**
  Unobservable in PHASE-01: `Reason::TraversalBudget` does not exist until
  PHASE-02. **Ratified as deferred, obligation intact**: PHASE-02's own `VT-2`
  mandates exactly that assertion in the same file, so C2 discharges it. No edit
  to a completed phase's criterion — a retroactive rewrite of a mandate that has
  already been verified is worse than a recorded hole.
- **DEF-2 — PHASE-01 `EX-5`'s `where`-only and `budget`-only clauses.**
  **Ratified as deferred, obligation intact**: `budget`-only ("accepted and
  inert, no `traversal` member") is PHASE-02 `EX-1` verbatim; the
  `where`-only case is PHASE-03's. PHASE-01 asserted the satisfiable set
  (`get`, `query {}`, from/select/order/limit-only, depth-only) — depth is
  already real, so that half was genuinely testable and was tested.
- **DEF-3 — PHASE-03 `VT-4` was vacuous, and this one was repaired.**
  Its keywords were `["additivity", "where"]` over `tests/compatibility.rs`.
  Both words are present *now* — `where` from the pre-existing refusal pin,
  `additivity` from PHASE-01's legitimate DEC-019 comment — so `verify-vt`
  reported PHASE-03 `VT-4` **PASS** three phases early. Retargeted to
  `["additivity", "where_accepted"]`, a token only the where-*accepted* pin can
  carry, and the criterion text now names the token so PHASE-03's implementer
  knows the mandate is contracted to it (envelope #6: a keyword the tree proves
  wrong is mine to correct; the obligation is unchanged and PHASE-03 `EX-7` still
  owns the assertion). C2 is chartered with this fact.

**Lesson for the rest of the run:** a VT keyword can go vacuous *because an
earlier phase legitimately used the word*. Vacuity is not a property of the plan
at authoring time; it has to be re-checked at every phase boundary against the
tree as it now is. From C2 on, the seat re-reads the next phase's keyword set
against the post-phase tree before chartering, and DEF-3 is the reason.

## C2 close — seat verification (2026-11-02)

The charter was `PHASE-02 → PHASE-03`. It delivered **PHASE-02 only**: the
harness's 30-minute wall killed the child after it had committed the phase,
recorded the conformance boundary (`515c6f8..4a192e7`) and run `verify-vt`, and
before it wrote any notes. Same failure shape as C1 — **two charters, two
timeouts, one phase each** — so the run's charter size changes from here: one
phase per child. Nothing was rerun; the phase's own commits stand.

**Verified at the seat:**

| check | result |
| --- | --- |
| `cargo fmt --check` / clippy ×2 | clean |
| `cargo test` | 13 suites, all pass (429 tests total; `tests/traversal.rs` 30, `cli_contract` 35) |
| `cargo test --features fault-injection` | green (run by the child; the seat re-ran the default suite) |
| `doctrine slice verify-vt 2` | PHASE-01 9/9 and PHASE-02 4/4 **PASS** |
| §14.2 fence | delta touches `README.md`, `src/select.rs`, `src/spec.rs`, `tests/{cli_contract,render,traversal}.rs` — no `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix` |
| end-to-end smoke (seat, hand-built store) | see below |

**The smoke test found the semantics right, and one thing the suite already
knew.** With `a→{b,c}`, `b→a`, `c→d`:

- `depth 3` unbounded: `depths {a:0,b:1,c:1,d:2}`, four edges, `frontier []`.
- `budget 1`: `a` expanded, `b`/`c` discovered but declined → `frontier ["b","c"]`,
  `truncated true`, `reasons ["traversal_budget"]` — so the check precedes the
  expansion (EX-2) and the declined node *and the rest of the queue* are in the
  frontier in discovery order (EX-4).
- `budget 2`: `b` expanded, `c` declined → `frontier ["c"]`.
- `depth 0`: `frontier ["a"]`, `truncated false` — the root is discovered but not
  expanded, so the depth cut puts it in the frontier and is not truncation
  (DEC-013/EX-4). Counter-intuitive on first read, correct on the wording.
- `budget` without `follow`/`backlinks`: accepted, inert, **no** `traversal`
  member (DEF-2 discharged).
- `budget: 0` and `budget: -1`: exit 2, "budget must be a positive integer",
  before any store access.

**DEF-1 and DEF-2 are discharged**, not merely deferred:
`both_reasons_appear_once_in_the_order_they_can_occur` exists in
`tests/traversal.rs`, and PHASE-02's `EX-1` inert-budget case is asserted.
`FUTURE_KEYS` is now `["where"]` — one entry, and PHASE-03 retires it.

**A design-consistent non-change worth recording:** PHASE-02's `EX-6` (the notice
gains the `traversal_budget` line) required **no edit to
`src/render_markdown.rs`**. PHASE-01 made the notice join
`selection.reasons` generically, so the new variant prints as
`> truncated: traversal_budget; additional entities omitted` with no second
account of reasons. The criterion is satisfied by construction; a phase that had
added a bespoke line would have created the duplication this run exists to
avoid.

## R1 — the repair batch, and one position flipped (2026-11-02)

RV-017 (`6d71911`) raised five prose findings against PHASE-01; all five are
disposed and the batch landed as `b499a60`:

- **F-1, F-3, F-4 fixed.** `tests/traversal.rs`'s module doc no longer claims the
  file is expected to be RED; the README distinguishes this tree from the
  published 0.1.1 instead of advertising traversal beside `cargo install smut`;
  `tests/selection.rs`'s comment names PHASE-02's both-reasons test inline
  instead of citing a bare `DEF-1`.
- **F-5 was already closed by PHASE-02**, which rewrote the contradicting doc
  block when it added the budget field. Dispositioned fixed with the phase as its
  agent.
- **F-2 flipped from my first position to the reviewer's.** I had overridden the
  reviewer's "tolerate until PHASE-06" on the theory that `learn`'s stale text was
  two clauses and cheap to fix. Reading the guide falsified that: the stale text
  is the key list, the 0.2 notice, the envelope description **and** the dangling
  sentence — the section PHASE-06 rewrites wholesale, with its own re-cut test and
  VT. A partial repair now would be rewritten in two phases and would leave a
  half-truthful guide in between. Tolerated, routed to PHASE-06 EX-1/EX-4, with
  the window and its owner named. **Evidence flipped the position; the review's
  route was right and mine was a guess about scope.**

### Plan gap recorded (not papered over)

**GAP-1.** No criterion in PHASE-01, PHASE-02 or PHASE-03 re-cuts `src/learn.rs`
prose, although each phase falsifies some: PHASE-01 accepts three keys and starts
reporting dangling references, PHASE-02 accepts `budget`, PHASE-03 will retire the
by-name refusal entirely. EX-11's rationale ("accepting `follow` falsifies it in
this commit") is scoped to the README, and §14.1 assigns `learn` to PHASE-06 — so
a shipped guide overstates what is refused for three phases by design, not by
accident. PHASE-06 EX-1/EX-4 close it; PHASE-06's charter carries the explicit
check ("no sentence in `learn` claims a working key is refused"). Recorded here so
the auditor sees a named gap with an owner rather than a discovery at audit.

### Autonomy log additions

| date | phase | decision | class | reason |
| --- | --- | --- | --- | --- |
| 2026-11-02 | C2 | **One phase per charter** from here; the plan's charter grouping survives only as the review-batch boundary | decided (#11, charter mechanics) | Two children, two 30-minute walls, one phase each. Grouping phases in one child bought nothing and cost the notes write both times. Phases, order and criteria are unchanged — this is charter size, not plan shape |
| 2026-11-02 | PHASE-01 | RV-017 repair batch landed by the seat (`b499a60`) with no child active | decided (#7, #10, #12) | No writer was alive; four fixes were prose and test comments with the evidence in hand; leaving known-false statements in the tree until C3 opened served nothing |
| 2026-11-02 | PHASE-01 | F-2 tolerated → PHASE-06, over my own earlier fix-now position | decided (#12) | The guide's staleness is the whole section PHASE-06 rewrites; a partial repair would be replaced in two phases |
| 2026-11-02 | PHASE-02 | PHASE-02's own autonomy decisions are **unrecorded**: the child was killed before writing them | gap, disclosed | The diff, the criteria and the seat's verification stand in their place; the sheet (gitignored) may hold its own notes. An auditor should read this as a missing record, not a missing decision |
| 2026-11-02 | C2 | C2 committed `515c6f8` (IDE-001, smut vs cordage) mid-charter | noted | A backlog idea captured through the project's own sink, unrelated to PHASE-02 source; harmless, but it is why the PHASE-02 boundary's `code_start_oid` is a doc commit |

### C3 — PHASE-03 landed (2026-11-02)

Charter: C3, **PHASE-03 only** (one phase per charter, ruled at C2). Entering
HEAD `2f49644`, tree clean; phase flipped `in_progress` at `2f49644`; phase
commit `21688c8` (11 files, +1430/−120); boundary `2f49644..21688c8`,
provenance `solo`; trailing notes commit after it.

Cost: **two worker children** — one red wave (12 integration tests + three
DEC-019 re-cuts) and one implementation (the module, the retirement, the wiring).
No rebrief, no repeat on an area. Every worker's green claim was re-run at the
sub-orchestrator tier, and all five gates were run by me on the final tree.

**Verified here, not taken on report:**

| check | result |
| --- | --- |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test` | exit 0 — 14 suites: lib 313, `predicates` 12, `cli_contract` 35, `traversal` 30, `selection` 10, `render` 12, `compatibility` 9, … |
| `cargo test --features fault-injection` | exit 0 — lib 326 and `durability_crash` 44, so the feature is not vacuous |
| `doctrine slice verify-vt 2` | **PHASE-03 VT-1…VT-4 all PASS** (PHASE-01 9/9, PHASE-02 4/4 still PASS; PHASE-04+ fail/unattributable as expected) |
| §14.2 fence | `git show --stat 21688c8` touches `README.md`, `src/{cli,error,lib,predicate,select,spec}.rs`, `tests/{cli_contract,compatibility,predicates,selection}.rs` — no `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix`, and no `src/learn.rs` |

**Red observed before green**, all four targets, all for the same right reason:
the pre-existing by-name `where` refusal (`FutureSpecificationKey { key: "where" }`
/ "where is not supported in 0.1; it arrives in 0.2"). The suite was written from
§6.2/§6.3/§7.1 and drove `spec::parse` + `select` only, so it could not lean on
the implementation's shape; the implementation then satisfied it with **no test
edits**.

**The phase's two genuine risks were controlled by mutation, not asserted:**
`where` moved after the walk, and `where` also filtering discovered ids, both
turn `where_filters_roots_before_the_walk_and_the_limit` red on "a discovered
entity failing `where` is returned: roots only". Present-null-as-absent,
`$contains` on a non-container, and the inverted disambiguation rule each turn
their own test red. Each mutation was restored and the suite re-run green before
the next.

**One finding from `verify-vt`, fixed inside the phase commit:** `VT-3` read
`FAIL — keyword roots-only absent` because the suite spelled it "roots only". The
token was moved into the VT-3 banner and the module doc — the same class as
DEF-3 inverted (a claim whose *evidence* was there but whose *mandate string* was
not). No criterion was narrowed.

### Autonomy log additions (C3)

| date | phase | decision | class | reason |
| --- | --- | --- | --- | --- |
| 2026-11-02 | PHASE-03 | Interpreter entry points fixed as `predicate::parse(&Value) -> Result<Predicate, Reject>` and `Predicate::holds(&Value) -> bool`; the AST is §6.1 verbatim | decided (#1, #2) | §6.1 locks the AST and EX-5 locks the placement; the entry point is implementation anatomy, so it is named in the sheet before briefing a worker |
| 2026-11-02 | PHASE-03 | Three new `Reject` variants (`MixedPredicateObject`, `UnknownPredicateOperator`, `InvalidPredicateValue`); a non-object `where` reuses `InvalidSpecificationValue { key: "where" }` | decided (#6) | §14.1 lists "bad predicate/operator/type" as refusals without naming variants; a wrong operand type is not a specification *value* of a key, so it earns its own class rather than a misused existing one |
| 2026-11-02 | PHASE-03 | `#[allow(clippy::large_enum_variant)]` on `cli::Invocation`, documented at the lint site | decided (#1, #9) | `Spec` grew by `Option<Predicate>` (locked by §5) and the `Request` variant crossed clippy's 200-byte threshold — verified by removing the allow and reading the lint. Boxing the field would contradict §5; the lint is about a stack copy, not a correctness guard, so it is suppressed once with its reason, never a check removed |
| 2026-11-02 | PHASE-03 | `tests/predicates.rs` copies `tests/cli_contract.rs`'s `Scratch` rather than creating `tests/common` | decided (#3) | The plan's answer to the same question, applied; a third harness fork would be the drift the review lane looks for |
| 2026-11-02 | PHASE-03 | `VT-3`'s `roots-only` token moved into the test file inside the phase commit, then the commit amended and the boundary re-recorded (`record-delta`) before `verify-vt` | decided (#6, #7) | The obligation was met; only the mandate's literal was not. Amending keeps the phase one contiguous commit so `code_end_oid` is the tree the mandate was verified against |
| 2026-11-02 | PHASE-03 | Research advisory (`slice research 2` reports drift from the added `plan.md`/`plan.toml`) left unrestamped | deferred, recorded (not escalated) | The drift is a plan-stage artefact added after the design, not design drift; the baseline is a design/plan-stage artefact and PHASE-01/02 ran under the same advisory. Restamping mid-phase would be an unrequested write outside this charter |
| 2026-11-02 | PHASE-03 | `src/learn.rs` left untouched although it still refuses `where` in prose (GAP-1) | deferred (recorded decision, upheld) | PHASE-06 EX-1/EX-4 rewrite that section wholesale; the C2 ruling and R1's F-2 disposition stand |

## C3 — PHASE-04 landed (2026-11-02)

Charter: C3, **PHASE-04 only** (one phase per charter, ruled at C2). Entering
HEAD `2ea81fa`, tree clean; phase flipped `in_progress` at `2ea81fa`; phase
commit `fddbe33` (3 files added, +1319/−0 — the array is 1164 lines); boundary
auto-bound at completion as `2ea81fa..fddbe33`, provenance `solo`; trailing notes
commit after it.

Cost: **one worker child** — the test suite only. The fixture itself was built by
this seat with a scripted read of the binary's public query surface and a `jq`
sort, because the raw store must not reach a subagent before the array is the
consented artefact; the worker read the committed array, not the store. No
rebrief, no repeat. Every worker green was re-run here, and all five gates were
run by me on the final tree.

**Verified here, not taken on report:**

| check | result |
| --- | --- |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean; non-vacuity proven by injecting an `unused_must_use` into `tests/fixture.rs` (failed), then reverting |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test` | exit 0 — 15 suites, `fixture` 9/9, lib 313 |
| `cargo test --features fault-injection` | exit 0 — 15 suites, lib 326, `durability_crash` 44 |
| `doctrine slice verify-vt 2` | **PHASE-04 VT-1 PASS**; PHASE-01 9/9, 02 4/4, 03 4/4 still PASS; PHASE-05…07 fail/unattributable as expected |
| §14.2 fence | `git show --stat fddbe33` is `tests/fixture.rs`, `tests/fixtures/interview/entities.json`, `tests/fixtures/interview/PROVENANCE.md` — no `src/**` at all, so `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix` and `src/learn.rs` are untouched |
| VA-1 | `git status` shows only the three intended paths; `interview.local/.smut` is matched by `.gitignore:58`; no store internals tracked |

**EX-1…EX-5 and VH-1.** EX-1: one normalised array, declared by the helper in
`tests/fixture.rs`, the only copy — and it rebuilds the store (66 declares, full
query reads back an identical array). EX-2: `PROVENANCE.md` carries the source
snapshot, the waiver and its basis, who resolved it, and what the outcome was.
EX-3: the nine assertions, on entities only. EX-4: the counts are the array's
own, in both the sheet and the note. EX-5: green in both feature configurations.
VH-1: satisfied by the operator's recorded ruling of 2026-11-02, which this phase
transcribes; the residual it left (consent from recall) was honoured by scanning
before the commit — nothing materially private found, so nothing was escalated
and nothing was removed.

**The residual was checked, and this is what it found.** Titles and bodies were
scanned for credentials, contact details, URLs, postal codes, financial and
health identifiers and mid-sentence proper nouns; the personal-content entities
(`session`, `a-focus`, `convention`, `a-navigation`, `q-focus`) were read in
full. The only personal mention is the operator's own *Slow Horses* viewing,
which the ruling names as inside the consent. `finding-smut-10` quotes opaque
store filenames as legibility examples — identifiers, not credentials. The
`PROVENANCE.md` text says this is a check on the waiver's premise, not a second
verdict.

**Two things worth the seat's attention.** (1) `session`'s prose says nine
questions are open while the array holds eight — the second stale claim inside
the store's own text, and the reason EX-3 forbids asserting against vocabulary
prose. (2) Two **review-lane** commits (`9e89d24`, `157ecf1`) landed between the
phase's entering HEAD and its own commit, both under `.doctrine/review/**`; they
ride in the boundary's range as foreign commits in the undeclared cell. Not a
conflicting writer — no source or test path was involved — but it is why
`record-delta --commit` refused to narrow the auto-bound span, and that refusal
was correct.

### Autonomy log additions (C3 · PHASE-04)

| date | phase | decision | class | reason |
| --- | --- | --- | --- | --- |
| 2026-11-02 | PHASE-04 | Array element field order is the store's own wrapper order (`id` first, then lexical) rather than a re-sort; the array is sorted by id and nothing inside an entity is reordered | decided (#8) | Fixture mechanics, not the verdict. Re-serialising values would break the declare round-trip, and the round-trip is what lets the fixture prove it rebuilds the store |
| 2026-11-02 | PHASE-04 | The array was built by this seat from the public query surface; only the *test suite* was delegated | decided (#11) | The G0 ruling flags raw store contents reaching a subagent. Building it here keeps the store in one context and gives the worker the consented, publishable artefact instead |
| 2026-11-02 | PHASE-04 | The privacy residual was scanned by keyword, structure and full reads of the personal-content entities; nothing materially private found, so nothing was escalated, removed or redacted | decided (#10; residual not triggered) | Consent came from recall, so the premise was checked before the irreversible act. The check is recorded in `PROVENANCE.md` as a check — the verdict stays the owner's |
| 2026-11-02 | PHASE-04 | Redeclared the cluster ids and the eight question ids as literals in `tests/fixture.rs` and `PROVENANCE.md` | decided (#8) | They are acceptance properties from §13.1/§13.3 scenario 2, not counts of the fixture; asserting by id is what makes the composition claim testable. No *total* count is hardcoded — the suite compares against the array's own length |
| 2026-11-02 | PHASE-04 | Boundary left as the auto-bound `2ea81fa..fddbe33` after `record-delta --commit fddbe33` refused to narrow it | decided (lifecycle artefact) | The refusal is right: the two review-lane doc commits sit between start and end and appear in conformance's undeclared cell, which is honest. Narrowing would have un-attributed nothing useful and moved those paths silently |
| 2026-11-02 | PHASE-04 | Research advisory left unrestamped (same reason as C3/PHASE-03) | deferred, recorded (not escalated) | Plan-stage drift, not design drift; no charter authority to restamp mid-phase |

## C4 — PHASE-05 landed (2026-11-02)

Charter: C4, **PHASE-05 only**. Entering HEAD `306b721`, tree clean, PHASE-05
already `in_progress` — a predecessor child had expanded the runtime sheet and
died before writing source. The sheet was re-read, found sound, and kept as
authored; the build started at the tests. Phase commit `cb46e37` (7 files,
+945/−5): `src/conventions.rs` 281 lines, `src/conventions/hygiene.md` 46,
`tests/conventions.rs` 538, and 85 changed lines across `src/cli.rs`,
`src/error.rs`, `src/lib.rs`, `src/main.rs`.

Boundary: auto-bound at completion as `77bb56a..cb46e37`, which swept in two
foreign commits that landed on the entering HEAD while this seat worked —
`23c8980` (review lane, `.doctrine/review/**`) and `045c54c` (the research
baseline restamp). Neither touches source or tests, so this was not a
conflicting writer, but neither is this phase's work; `record-delta --start
cb46e37^ --end cb46e37` narrowed the record to the phase's own commit. (The
inherited `.doctrine` dirt recorded at charter time resolved itself: `045c54c`
is that restamp, and `doctrine verify research-current --slice 2` reports
current on the closing tree.)

Cost: **no worker child**. The whole surface is ~330 lines of new source in one
coherent area, and a worker would have needed all of §9 plus the 0.1 patch
semantics to produce it — the failure mode the predecessor demonstrated. Built
by this seat; envelope #11.

**Verified here, not taken on report:**

| check | result |
| --- | --- |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test` | exit 0 — 16 suites, 481 tests; `conventions` 14/14, lib 320 |
| `cargo test --features fault-injection` | exit 0 — 16 suites, 538 tests; `store::fault` 3/3 executed, so the injected run is not vacuous |
| `doctrine slice verify-vt 2` (after the phase was flipped `completed`) | **PHASE-05 VT-1 PASS, VT-2 PASS**; PHASE-01 9/9, 02 4/4, 03 4/4, 04 1/1 unchanged; PHASE-06 (VT-2 `follow` absent) and PHASE-07 (VT-1/2/3) fail as expected for unbuilt phases |
| §14.2 fence | `git show --stat cb46e37` is `src/{cli,conventions,error,lib,main}.rs`, `src/conventions/hygiene.md`, `tests/conventions.rs` — no `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix`, no `design.md`, no `src/learn.rs`, no `Cargo.*`, no README |
| VA-1 | grep over `src/conventions.rs` for `Store`/`Snapshot`/`lock()`/`std::fs`/`std::path` in code returns nothing (the only hits are the module doc saying there is none); the prose is `include_str!("conventions/hygiene.md")`, so it is the file the `src/conventions/**` selector matches |

**Red was observed before the command existed**, for the right reason: 14 tests,
all failing with `smut: unknown command: conventions` and exit 2 — not a compile
error. One test passed vacuously in that state (`--format json` with no name also
exits 2 when the command does not exist), so it was tightened to assert the
refusal names `json` and is *not* the unknown-command refusal; it then failed,
and 0/14 were green before implementation.

**EX-1…EX-6, VT-1, VT-2, VA-1.**

- EX-1: listing is one line for one set (`hygiene  general structure and
  relations`), prose for a named set, unknown name exit 2 naming `hygiene`
  (`epistemic-records` explicitly, so the deferred set is not half-shipped),
  `--format text` byte-identical to no flag for both forms, `--format json` with
  no name exit 2, and `dot`/`yaml`/`JSON`/`markdown` each a usage error naming
  themselves.
- EX-2: one object, `id: conventions-hygiene`, `type: convention`,
  `version == env!("CARGO_PKG_VERSION")` (prints `0.1.1` on this tree — PHASE-07
  owns the bump, which is why it is not pinned), `title: Conventions: general
  hygiene` derived from the embedded heading, `body` the embedded text, one
  compact line ending in one newline. Five fields, no more.
- EX-3: `tests/conventions.rs` runs the real pipe — seed → `declare`, seed again
  (`unchanged`, still one entity), amend `body` + add `owner`, seed again
  (`updated`, `previous.body` = the amendment, `previous.owner` = `ops`), and
  the stored `body` back to the seeded text with `owner` intact; `--compact`
  omits `previous` and the replaced text appears nowhere. 0.1's patch semantics
  did this already: nothing was re-implemented.
- EX-4: no store is created by any of the four forms, and all four still answer
  correctly with a decoy file sitting at `.smut` — every command that opened it
  would exit 1. No validation, no re-install, no heuristic set choice.
- EX-5: eight numbered items, asserted by count and order; item 7 carries
  `retracted`, `superseded`, `supersedes` and `vocab`; the prose says the seeded
  body is advice a re-install replaces and points at `smut learn` for the
  two-call recipe while containing no query JSON at all.
- EX-6: `src/lib.rs` exports `conventions`; both feature configurations green.
- VT-1: `tests/conventions.rs` carries `conventions-hygiene` and
  `epistemic-records`, and its JSON round-trip is the `json` case.
- VT-2: the re-install test carries `previous` and `compact` and asserts exactly
  the pair §13.1 promises (amended seeded field replaced, user field kept).
- VA-1: the two rows above.

**The quoted prose.** Item 7: *"Distinguish a withdrawn record from a replaced
one. `retracted` means the content is withdrawn and kept as a record of its own
error; `superseded` means something newer replaced it, and `supersedes` is the
edge that makes it so. A store that says `superseded` retired something "with
its retraction intact" has collapsed the two, and a `where` on either status then
lies — the store this release is built from does exactly that, and its own
`vocab` is the evidence that it was needed."* The re-install caveat: *"Re-installing
replaces every field the seed supplies — `body` included — and leaves the fields
you added alone. The seeded body is therefore advice, not a record: keep an
amendment by editing under a different id or by keeping it outside the seeded
fields, or expect the next install to replace it. The replacement is reported
rather than warned about: the reply to that `declare` carries the entity as it
stood before the batch, `body` included, under `previous`, and `--compact` drops
that field."*

**Negative controls** (each mutated, observed red, restored, gate re-run):

| control | mutation | red |
| --- | --- | --- |
| the command opens a store | `main.rs`'s `Conventions` arm calls `Store::in_directory(..).load()?` and takes its lock before answering | 4 tests failed with `smut: no store at …/.smut` (exit 1 vs 0) |
| `previous` omits the replaced body | `declare.rs`'s `Change::to_json` drops the `previous` insert | `a_re_install_…_reports_the_previous` failed: `previous` `Null` vs `"Our house rule: one label per relation."` |
| one of §9.2's eight items dropped | item 5 deleted from `hygiene.md` | the eight-item test failed, listing 7 items with the gap at 5→6 |

After restoring, `git diff` on all three files was empty and every gate above
was re-run on the restored tree.

**Two things for the seat.** (1) `Reject::InvalidUsage` is reused for
`--format json` with no name — the design names two new refusals and this is a
usage error, not a third; its message carries the `smut: ` prefix and a trailing
newline itself, because that arm prints the message verbatim. (2) The seed's
JSON keys serialise lexically (`body` first, `id` second) because
`serde_json::Map` is a `BTreeMap`; the entity is order-insensitive, `declare`
reads it either way, and §9.3's printed example is illustrative rather than a
byte contract.

### Autonomy log additions (C4 · PHASE-05)

| date | phase | decision | class | reason |
| --- | --- | --- | --- | --- |
| 2026-11-02 | PHASE-05 | Everything built by this seat; no worker child chartered | decided (#11) | One coherent area of ~330 new lines; briefing a worker needed all of §9 plus 0.1's patch semantics, which is the context the predecessor child lost. The plan's split (tests, then source) was followed by hand, tests first |
| 2026-11-02 | PHASE-05 | `Invocation::Conventions(conventions::Ask)`; `Ask` resolved (and refused) at parse time, so `main.rs`'s arm is infallible text like `Learn` | decided (#1, #2) | Keeps DEC-004 step 1 whole — every refusal happens before a store could exist — and keeps the no-store chain structural rather than a promise |
| 2026-11-02 | PHASE-05 | Two new refusals only: `UnknownConvention { found, sets }` and `UnknownConventionFormat { format }`; `--format json` with no name reuses `Reject::InvalidUsage` | decided (#1, #6) | The design names exactly two new refusals. The name-carrying refusal keeps `error.rs` free of the registry, and a missing name is a usage error, which is what `InvalidUsage` already means |
| 2026-11-02 | PHASE-05 | `hygiene.md`'s H1 is `# General hygiene`; `title` is that heading in sentence case behind `Conventions: `, `body` is the file text less one trailing newline | decided (#1, #6) | §9.3's example fixes the title and the body's derivation; deriving both from the one embedded string is what stops a second copy drifting |
| 2026-11-02 | PHASE-05 | The prose also says the replay is reported under `previous` and names the deferred `epistemic-records` set as deferred | decided (#10) | Both are behaviour that exists (§9.3, the refusal) and neither repeats the recipe; the second keeps a reader who expected the second set from reading its absence as an oversight |
| 2026-11-02 | PHASE-05 | The vacuously-green refusal test was tightened (assert the message names `json` and is not the unknown-command refusal) rather than accepted | decided (#6) | A test satisfied by the command's non-existence certifies nothing; DEF-3's rule applied to a test written in this phase |
| 2026-11-02 | PHASE-05 | Boundary narrowed to `cb46e37^..cb46e37`; research baseline left as `045c54c` left it (already current) | decided (lifecycle artefact) | Two foreign commits landed on the entering HEAD mid-phase. Narrowing attributes the phase's own commit only; the neighbouring review/doc commits keep their own cells and stay visible in conformance |

## C5 — PHASE-06 landed (2026-11-02)

Charter: C5, **PHASE-06 only**. Entering HEAD `84a8d5f` with an **inherited
uncommitted partial** in `src/learn.rs` — a predecessor child killed by a
provider budget error after writing ~155 lines: the twelve red 0.2-coverage
assertions plus the DEC-019 re-cut of
`the_guide_names_the_release_the_rest_arrives_in` into
`the_guide_teaches_the_0_2_keys_and_the_release_notice_is_gone`. The partial was
adopted as the spec and not edited: `cargo test --lib learn::` was run first and
showed **12 failures, every one "the guide omits …"** — the contract, written
red on purpose. Phase commit `46c6699` (2 files, +607/−41); docs follow-up
`a6ddb25`; boundary `84a8d5f..46c6699`, provenance `solo`, auto-recorded at the
completion flip and therefore not carrying RV-019's later review commit.

Cost: **no worker child**. The phase sheet's charter named this seat the sole
writer, and a worker would have needed the example constants handed over first,
which is the phase.

**Verified here, not taken on report:**

| check | result |
| --- | --- |
| `cargo fmt --check` | clean (after `cargo fmt` reflowed the inherited `KEYS` list) |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo test` | exit 0 — 16 suites; lib 333, `fixture` 14, `compatibility` 10, `traversal` 30, `conventions` 14, … |
| `cargo test --features fault-injection` | exit 0 — lib 346 and `durability_crash` 44 executed, so the injected run is not vacuous |
| `doctrine slice verify-vt 2` (after `--status completed`) | **PHASE-06 VT-1 PASS, VT-2 PASS**; PHASE-01 9/9, 02 4/4, 03 4/4, 04 1/1, 05 2/2 unchanged (22 PASS total); PHASE-07's VT-1…VT-3 fail as expected for an unbuilt phase |
| §14.2 fence | `git show --stat 46c6699` is `src/learn.rs` and `tests/fixture.rs` only — no `src/entity.rs`, `src/order.rs`, `src/store/**`, `docs/store-format.md`, `justfile`, `flake.nix`, no `design.md`, no README, no `src/conventions/**`, no `Cargo.*`, and the fixture array is untouched |

**EX-1…EX-7, VT-1, VT-2, VA-1.**

- EX-1: the guide's key block is read by a test (`the_guide_teaches_exactly_the_nine_keys`)
  and is exactly the nine of §5 in order; `depth: … default 1. 0 is roots-only`;
  the predicate summary is complete (`$and`/`$or`/`$not` plus `$eq`, `$ne`,
  `$in`, `$exists`, `$contains`) and carries "present null is present"; labels
  are stated as `top-level` field names in both `outgoing` and `incoming`
  directions; the `traversal` member names `depths`/`edges`/`frontier` plus
  `dangling` and both reasons; the label vocabulary is "a handful" with "render
  in bounded chunks"; `$`-prefixed names "cannot be filtered at all in 0.2"; one
  `smut conventions` pointer line under `Conventions` (the `Commands` block lists
  the command, which is the 0.1 surface, not a second pointer).
- EX-2: the recipe states containment `child -> container`, names the question,
  prints the `backlinks` call at depth 2 **and** the `follow` variant beside it
  with "a different question, not a wrong answer", and explains that the session
  stores no `part-of` reference so the outgoing walk returns it alone.
- EX-3: the guard is a numbered part of the recipe: read `truncated` and
  `truncation_reasons` before any id; `retry once` with a bound the store can
  meet; stop and answer `partially` naming the reason and the refused bound;
  `repetition` alone never establishes completeness; a truncated first response
  makes a `truncated: false` second envelope `no evidence` at all.
- EX-4: the 0.1 notice is gone (`!GUIDE.contains("arrive in 0.2")`,
  `!GUIDE.contains("refused here")`) and the guide states what `budget` `does not
  bound`: the per-hop `store scan` for backlinks and the whole-store `root scan`
  for an absent `from` or a `where`. The falsified test is re-cut, not deleted.
- EX-5: `EXAMPLE_TRAVERSE`/`EXAMPLE_FOLLOW`/`EXAMPLE_FILTER` are `pub const`s
  printed in the guide (pinned by `GUIDE.contains(..)` in `src/learn.rs`) and
  executed by `tests/fixture.rs`, which also asserts `smut learn` prints them.
- EX-6: the gate table above; the compatibility pin is part of `cargo test`.
- EX-7: "The guard is client-side … smut adds no prior-response input to the
  second call and performs no propagation … the specification stays at the nine
  keys above". Nothing was added to `Spec` (`src/spec.rs` still has nine arms).
- VT-1 (`src/learn.rs`): `$in` in the operator list, `roots-only` in the key
  block, `smut conventions` in the pointer line — all three tokens in the
  mandated file, and the mandate's clauses are the individual tests above.
- VT-2 (`tests/fixture.rs`): `follow` (the variant test), `eight` (`OPEN_QUESTIONS`
  and the assertion message), `guard` (`Guarded`, `guarded`, and the guard
  test's name), with the eight ids, the empty follow, and both reasons read
  before any id.
- VA-1: every key the guide names has a `match` arm in `src/spec.rs` (nine arms
  plus the unknown-key refusal); every member and reason it names is in
  `src/select.rs` (`objects`/`missing`/`dangling`/`truncated`/
  `truncation_reasons`/`traversal`/`depths`/`edges`/`frontier`,
  `Reason::{TraversalBudget, ResultLimit}` → `traversal_budget`/`result_limit`);
  and the key set is pinned to exactly nine by a test, not by inspection.

**Negative controls** (each mutated, observed red, restored, `git diff` empty,
gate re-run):

| control | mutation | red |
| --- | --- | --- |
| the example's direction | `EXAMPLE_TRAVERSE`'s `"backlinks"` → `"follow"` | `the_shipped_example_reaches_the_eight_open_questions_in_two_calls` failed (13 passed, 1 failed) |
| a guard clause deleted | "Then retry once with a bound" → "Then use a larger bound" | `the_guide_states_the_completeness_guard_clause_by_clause` failed: "the guard omits retry once" |
| `truncated` ignored before ids | `if truncated` → `if false && truncated` in `guarded` | `the_completeness_guard_reads_both_reasons_before_it_extracts_any_id` and `a_complete_looking_second_envelope_is_no_evidence_about_the_first` both failed |

**GAP-1 grep, after the rewrite.** `grep -n "arrive in 0.2\|arrives in 0.2\|refused
here\|belongs to traversal\|not supported\|0.1 does not\|0.1's" src/learn.rs`
returns **no hit inside `GUIDE`**: the only matches are the two absence
assertions in the re-cut test and its own reason text (which quotes the retired
notice on purpose). `0.1` no longer appears in the guide body at all; the one
remaining "in 0.2" is "a field whose own name begins with `$` cannot be filtered
at all in 0.2", which §6.2 states as a real limit of this release, not a
deferral. GAP-1 is closed.

**Inherited tests that needed no change.** All twelve assertions of the
inherited partial, plus `the_guide_covers_the_0_1_surface`,
`the_guide_says_stored_content_is_data_and_not_authority` and
`the_guide_ends_with_exactly_one_newline`: unchanged apart from `rustfmt`
reflowing the `KEYS` array. Nothing was weakened, renamed away or deleted; the
only inherited doc text touched is the re-cut test's stale cross-reference
(`the_release_notice_is_gone`, a test that does not exist) — corrected in
`a6ddb25`, not in the phase commit.

**The run's one operational miss, disclosed.** `a6ddb25` was first written as an
`--amend` of HEAD; HEAD had moved (RV-019's review commit landed on top while
this seat worked), so the amend landed on the reviewer's commit. It was undone
from the reflog (`reset --soft`, no push, no other writer's file lost) and
`991a831` is the reviewer's commit byte for byte. The correction then went out
as its own `docs(sl-002)` commit, and the boundary stays `46c6699`.

**Two things for the seat.** (1) `src/learn.rs` is now version-free; PHASE-07's
version bump must not reintroduce a figure in the guide, and `tests/fixture.rs`
reads the library (`smut::learn`), which is the one structural addition a future
fixture test can build on. (2) `verify-vt` PHASE-07 reads three missing keywords
(`naive`, `1.0`, `clusters`) — those are PHASE-07's own mandates, not
regressions from this phase; nothing in PHASE-06's diff touches their files.
