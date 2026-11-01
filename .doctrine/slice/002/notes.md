# Notes SL-002: smut 0.2 — graph release: predicates and traversal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-01 · reviewing (design run RV-007) · 33fc45e

### Produced

- SL-002 scope `slice-002.md` — 11 objectives, the boundary, and the settled
  decisions table replacing the old open questions.
- Research artefact + raw threads; baseline restamped after the design.
- DEC-012…DEC-019 from the eight inquiry dispositions; DEC-014 amended to the
  induced subgraph of returned entities.
- 15-section `design.md`, materialised at revision 33; RV-005/RV-006 are
  superseded stale passes, RV-007 is the review pass (F-1…F-4 raised,
  disposed fix-now, repaired).
- 16 `design-target` selectors matching §14.
- IMP-001 moved to `started`.

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

### Open

- User review of `design.md` (RV-007), then lock (`design-accepted`), `/plan`,
  `/phase-plan`.
- DEC-017: the epistemic-records schema needs operator approval before that set
  ships.
- DEC-018: a fresh-agent privacy read-through precedes the fixture commit.
- The design run state is gitignored but live at `.doctrine/state/slice/002/`;
  resume with `doctrine design resume SL-002`.

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
