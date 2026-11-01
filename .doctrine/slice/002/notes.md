# Notes SL-002: smut 0.2 — graph release: predicates and traversal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · ready (design locked at run rev 62; plan repaired
round 1, 7 phases materialised, 25 VT mandates) · commit c3e5a49

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

### Open

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
  an auditor reads a ruling, not an omission.
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
