# Notes SL-002: smut 0.2 — graph release: predicates and traversal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · reviewing (run rev 57, pass RV-015 concluded as a
rebinding pass, RV-012 F-4 answered awaiting raiser verification) · commit
603c718 + this session's ledger/run acts

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
- DEC-018: a fresh-agent privacy read-through precedes the fixture commit.
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
