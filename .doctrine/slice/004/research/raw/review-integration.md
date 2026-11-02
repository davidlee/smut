# SL-004 — pre-implementation review integration

**Scope:** adjudication of `/tmp/sl004-obligations-review.md` (Seat A) and
`/tmp/sl004-sequence-review.md` (Seat B) against the locked design, `plan.toml`,
`plan.md`, the runtime sheets and the code, and the plan repair that follows.

**Surfaces written:** `.doctrine/slice/004/plan.toml` (primary),
`.doctrine/slice/004/plan.md` (rationale paragraph only),
`.doctrine/slice/004/notes.md` (appended `## Pre-implementation review`).
**Not written:** `design.md` (locked), `README.md`, `src/**`, `tests/**`,
`docs/**`, `.doctrine/state/**`, `.doctrine/slice/004/phases/*` (regenerated only
through the CLI).

**Verification performed (all commands re-run here, not trusted from the seats):**

| claim | command | result |
|---|---|---|
| `ENTITY_ID` is `"id"` | `rg -n ENTITY_ID src/entity.rs` | `17:pub const ENTITY_ID: &str = "id";` ✓ |
| conventions seed is a raw literal | `sed -n '115,135p' src/conventions.rs` | `("id", id_of(set))` at `:126`; no `ENTITY_ID` in the file ✓ |
| receipt subject is a raw literal | `sed -n '100,120p' src/declare.rs` | `:110 entry.insert("id".to_owned(), …)` ✓ |
| identity-naming diagnostics | `rg -n 'holds id\|invalid entity id' src/error.rs`; `rg -n 'no .id.' src/store/mod.rs`; `sed -n '70,100p' src/entity.rs` | `error.rs:282`, `error.rs:389`, `store/mod.rs:505`, panic at `entity.rs:93` ✓ |
| render suppression is keyed on the constant | `sed -n '180,215p' src/render_markdown.rs` | `is_selected` forces `ENTITY_ID` (`:186`); `attribute_lines` filters `*field != ENTITY_ID` (`:211`) ✓ |
| format constants | `rg -n FORMAT_VERSION src/store/{index,journal}.rs` | `index.rs:28 = 1`, `journal.rs:35 = 1` ✓ |
| version literals outside the constants | `rg -n format_version tests/store_corruption.rs tests/traversal.rs` | `:17,121,130,145,185` and `traversal.rs:367`; `:130` uses **2** as the *unknown* value ✓ |
| v1 store with nested `{"id":null}` serves `get`, panics on `follow` | scratch `.smut` + `target/debug/smut get alpha` / `query '{"from":["alpha"],"follow":["payload"]}'` | `get` → exit 0 with the payload; `follow` → panic `src/entity.rs:93`, exit 101 ✓ |
| a stale `where` is silent | `smut query '{"from":["alpha"],"where":{"id":"nope"}}'` | `{"objects":[],…}` exit 0 ✓ |
| only `tests/fixture.rs` reads the fixture | `rg -l 'entities\.json\|FIXTURE' tests/` | `tests/fixture.rs`, `tests/fixtures/interview/PROVENANCE.md` only ✓ |
| `readme_example` matches one fence spelling | `rg -n console tests/readme_example.rs`; `rg -n '^\s*```' README.md` | `:78 inside = line.trim() == "```console"`; README has 3×"``` console" + 1×"```console" ✓ |
| learn prose pins | `rg -n 'cfg\(test\)' src/learn.rs`; `sed -n '65,95p' src/learn.rs` | test module at `:269`; guide still says "a nonempty string \"id\"" and "cannot carry \"id\" as inline metadata" ✓ |
| sweep counts | `rg -o --no-filename '"id"' src tests docs README.md \| wc -l`; `… \| rg -l \| wc -l`; brief | 974 / 27 ✓; `smut-implementation-brief.org` adds 10 → 984 repo-wide ✓ |
| RV-023 empty | `doctrine show RV-023` | `findings: 0` ✓ |
| `tests/render.rs`, `tests/store_corruption.rs`, `tests/cli_contract.rs`, `src/learn.rs` exist | `ls tests/` | all present ✓ |

---

## (a) Disposition table

| finding | seat | verdict | where |
|---|---|---|---|
| A-F1 stale `where` on `id` is silent | A | repaired + design-escalated | PHASE-02/EX-8, VT-2, VT-7; D1 |
| A-F2 identity-naming diagnostics undecided | A | merged with A-F5, B-F-8 (diagnostic leg); repaired + design-escalated | PHASE-02/EX-9; D2 |
| A-F3 `learn` prose pins have no criterion | A | repaired | PHASE-02/VT-7, EX-5 |
| A-F4 ordinary `id` becomes visible in render | A | repaired + design-escalated | PHASE-02/EX-10, VT-8; D3 |
| A-F5 exact diagnostic text softened | A | merged with A-F2; repaired | PHASE-01/EX-2, PHASE-02/EX-9, VT-3 |
| A-F6 VTs pass on source presence | A | merged with B-F-4 (literal-key leg); repaired | PHASE-01/VT-1, VA-3; PHASE-02/VT-3 |
| A-F7 `DROP` as ordinary data has no criterion | A | repaired | PHASE-02/EX-2, VT-2 |
| A-F8 conventions seed contradicts design §2 | A | merged with B-F-3; design-escalated + plan strengthened | PHASE-02/EX-1, VA-2; D4 |
| A-F9 PHASE-01 entrance omits §4/§6 | A | repaired | PHASE-01/EN-1 (and PHASE-02/EN-1 → §1-6) |
| A-F10 `learn` version-pinning collision | A | repaired in plan (versionless phrasing) + design-escalated | PHASE-02/EX-5; D5 |
| A-F11 `previous`/`--compact` unnamed | A | merged into A-F15; repaired | PHASE-02/VT-2 |
| A-F12 path allocation / linear walk unheld | A | repaired | PHASE-01/EX-1, VA-3 |
| A-F13 D§3-6 / D§3-8 cases unnamed | A | repaired | PHASE-01/EX-1, VT-1 |
| A-F14 `$` root marker absent | A | repaired | PHASE-01/EX-2 |
| A-F15 container vocabulary not named | A | repaired | PHASE-02/VT-2 |
| A-F16 no identity module / no new public API mid-slice | A | repaired (module/alias leg); remainder rejected | PHASE-02/EX-1; dependency/public-API stays PHASE-03/EX-4 |
| A-F17 byte forms / opaque ids unnamed in VA-2 | A | rejected | already held by unchanged suite + PHASE-02/EX-6 |
| A-F18 PROVENANCE unexecuted recipe | A | merged with B-F-9; repaired | PHASE-02/EX-5, VA-2 |
| A-F19 `src/order.rs` row overstates the key | A | design-escalated | D4 |
| A-F20 PHASE-02 VT-6 looks stray | A | repaired | comment above VT-6/VT-7/VT-8 |
| A-F21 design says "a proposed exact reason" | A | design-escalated (plan now pins the text) | PHASE-02/EX-9; D2 |
| B-F-1 PHASE-01 narrows v1's acceptance set in place | B | repaired (sequencing) | PHASE-01/EX-4, EX-5; PHASE-02/EX-5 — §c(i) |
| B-F-2 operator's live store bricks with no pre-flight | B | repaired | PHASE-02/EN-4 |
| B-F-3 design's production-literal claims are false | B | merged with A-F8; repaired + design-escalated | PHASE-02/EX-1, VA-2; D4 |
| B-F-4 nothing asserts the *new* load acceptance of nested `id` | B | merged with A-F6; repaired | PHASE-02/EX-8, VT-3; PHASE-01/VT-1 |
| B-F-5 fixture risk statement names the wrong files | B | repaired (plan) + evidence correction recorded | PHASE-02/EX-4; notes.md |
| B-F-6 README contradicts the binary; gate blind | B | operator-surface (CHR-001); plan reduced to a coordination boundary | PHASE-02/EN-3, EX-5, EX-10, VA-2; PHASE-03/EX-4, VA-4; `slice-004.md`; §f |
| B-F-7 PHASE-03 entrance/exit enumeration | B | rejected | EN-1 + EN-3 already cover EX-1…EX-7 |
| B-F-8 doc version literals; backwards diagnostic | B | repaired | PHASE-02/EX-3, EX-5, VT-3 |
| B-F-9 sweep undercount (984, brief in scope) | B | merged with A-F18; repaired | PHASE-02/VA-2 |
| B-F-10 DEC-005 signature drift | B | repaired | PHASE-03/EX-1 |

No finding was accepted on the seat's word alone; each `where` cites a criterion
whose text changes only after the file:line or probe above reproduced.

---

## (b) Repairs, per criterion

### PHASE-01 — "Establish the loaded-identity invariant"

**EN-1** (sharpen, restores design §4/§6 onto the phase that edits `src/store/mod.rs`; A-F9)
> old: `The locked design sections 1-3 and 5, DEC-021 and STD-001 are read; the existing Store::load root checks and reference_id rule match their current source.`
> new: `The locked design sections 1-5, section 6's linear-work and path-allocation constraints, DEC-021 and STD-001 are read; this phase introduces no new public error variant; the existing Store::load root checks and reference_id rule match their current source.`

**EX-1** (sharpen; D§3-6 non-stop walk, D§3-8 ordinary-`id` descent, D§3-9/§6-4 no re-read/clone; A-F12, A-F13)
> old: `…walks parsed objects/arrays using reference_id after root checks and before snapshot insertion; scan_refs remains non-fallible.`
> new: `…walks parsed objects/arrays using reference_id after root checks and before snapshot insertion; it does not stop at a valid reference-shaped object, it still descends into the values of an ordinary id, and it neither re-reads the file nor clones entity subtrees; scan_refs remains non-fallible.`

**EX-2** (sharpen; D§3-11 `$` marker and D§3-16 exact reason; A-F14, A-F5)
> old: `…naming the entity basename and JSON-quoted bracket path to the identity member; request malformation remains exit 2.`
> new: `…naming the entity basename and the JSON-quoted bracket path to the identity member from a $ root marker (for example $["payload"]["tags"][2]["smut_id"]), with the reason reading identity at <path> must be a nonempty string; request malformation remains exit 2.`

**EX-4** (rewritten — sequencing repair, see §c(i); B-F-1)
> old: `docs/store-format.md documents the new corruption check, path notation and worked diagnostic using the phase's current identity spelling in the same change; focused and repository checks are green.`
> new: `The corruption rule, path notation and worked diagnostic this phase introduces are recorded in the phase evidence as pending PHASE-02 material, and docs/store-format.md is left untouched while it still declares format version 1, so no version in the format document states a rule its released reader did not enforce; focused and repository checks are green.`

**EX-5** (NEW; sequencing repair, see §c(i); B-F-1)
> `No tree of this phase is a release candidate or a supported reader of any version-1 store: it is not tagged or published, it is run only against scratch stores, and the stricter loader enters service only with PHASE-02's version bump. The phase evidence records both the rule the tree enforces and the absence of any release or live-store use.`

**VT-1** (sharpen `expects`; A-F13 case + B-F-4's silent-flip mechanism)
> old: `Store corruption tests cover invalid nested identity under objects/arrays, JSON-escaped paths, lexical/positional first failure, unchanged entity/index bytes and preserved root-check precedence.`
> new: `Store corruption tests cover invalid nested identity under objects/arrays, a malformed identity reached inside an ordinary id-valued object, JSON-escaped paths, lexical/positional first failure, unchanged entity/index bytes and preserved root-check precedence; the injected malformed identity is written with the phase's literal key spelling rather than the constant, so its semantic target is visible in the diff and cannot flip silently when the key moves.`

**VA-2** (sharpen; the doc review now reviews recorded text, not a doc that has not moved)
> old: `Review the corruption-document change against actual diagnostics under STD-001; …`
> new: `Review the phase's recorded diagnostic text and path notation against the actual emitted errors under STD-001; …` (tail unchanged)

**VA-3** (NEW; A-F6 paper-VT backing, A-F12 allocation)
> `The inline mod tests in src/store/mod.rs drive validate_identity_tree over a parsed malformed value, so the shape mandate is backed by an executed test and not a signature; the walk stays linear over the already-parsed entity with no re-read or re-parse, and the diagnostic path allocates only the traversal path and the final error.`

### PHASE-02 — "Move structural identity and the store format together"

**EN-1** (sharpen; now that PHASE-01 gained EX-5, and design §6 binds the phase that adds the dependency surface)
> old: `PHASE-01/EX-1 through EX-4 hold and the loader regression suite is green; read design sections 1-5, …`
> new: `PHASE-01/EX-1 through EX-5 hold and the loader regression suite is green; read design sections 1-6, …`

**EN-4** (NEW; B-F-2)
> `Before either version constant moves, the operator's live store (interview.local/.smut, whose id set measured identical to the committed fixture) has an agreed re-declare path recorded in the phase evidence, and the operator has acknowledged that it stops loading at this phase.`

**EX-1** (sharpen; B-F-3/A-F8 production literals + A-F16 no alias)
> old: `…changes[].smut_id and the conventions JSON seed use it. Internal value/type names and response containers stay unchanged.`
> new: `…changes[].smut_id and the conventions JSON seed use it, and no production literal spells the wire key outside ENTITY_ID itself - the change-receipt subject and the conventions seed are raw literals today and are routed through the constant. Internal value/type names and response containers stay unchanged, and no identity module or alias is added.`

**EX-2** (sharpen; D§2-25 / A-F7)
> old: `…while id behaves as an ordinary queried field under existing patch rules.`
> new: `…while id behaves as an ordinary queried field under existing patch rules, including the top-level "DROP" sentinel applied to an ordinary id.`

**EX-3** (sharpen; B-F-8's six literal version sites and the inverted unknown-version case)
> old: `Both format-version constants are 2; old indexes fail cleanly by version…`
> new: `Both format-version constants are 2, every version literal in a store-building test is 2, and every old-version refusal fixture names 1 explicitly - including tests/traversal.rs's rebuilt index and the unknown-version case, which stops using 2 as its unknown value; old indexes fail cleanly by version…`

**EX-4** (sharpen; B-F-5)
> old: `…demonstrate rejection of an altered copy.`
> new: `…demonstrate rejection of an altered copy, and re-derive every assertion that reads the committed fixture - including the hardcoded cluster ids and count-sensitive expectations in tests/fixture.rs - against the adapted array.`

**EX-5** (sharpen; B-F-8 doc completeness, A-F18 PROVENANCE, A-F10 versionless refusal; README clause removed post-steering — see §f)
> old: `The normative brief, docs/store-format.md, learn, README examples, fixture provenance and affected test expectations consistently describe the new identity/version rules. …`
> new: `The normative brief, docs/store-format.md, learn, tests/fixtures/interview/PROVENANCE.md and affected test expectations consistently describe the new identity/version rules: the format document ends the phase with one identity spelling and one version pair, its version lines, index and manifest examples, canonical-bytes sentences and both worked diagnostics all moved; the guide states that a store written by an older build is refused without pinning a release figure. The brief's unfinished 1.0 requirements and historical fixture prose remain intact. README.md is the operator's 0.2 rewrite (CHR-001): it is neither edited nor gated here, and this phase must not be blocked on its content.`

**EX-8** (NEW; D§6-1's "new behavior cases" and the design's named-but-undecided silence risk; A-F1 + B-F-4)
> `A stored entity whose nested ordinary id carries null, empty string, number, array or object loads and is served at exit 0 by both get and traversal, while the same values under smut_id are corruption at exit 1 with the bracketed path; a where on ordinary id selects only entities carrying id as data, so a caller still sending the old spelling gets an empty result at exit 0, and the guide states that consequence in one sentence.`

**EX-9** (NEW; D§3-28-shaped obligation, STD-001 contract; A-F2/A-F5, B-F-8 diagnostic leg)
> `Every identity-naming diagnostic names smut_id: the Reject::InvalidId message, the StoreError::IdDisagreement message, describe()'s absent-identity text and the scan_refs assertion. docs/store-format.md quotes the updated strings verbatim, and the index refusal keeps the wording smut: the store index has format_version 1, and this build reads only 2.`

**EX-10** (NEW; the consequence of the constant-keyed renderer; A-F4)
> `Rendered output for an entity carrying both smut_id and an ordinary id is pinned: the suppression set is the identity constant, so the ordinary id now renders as an attribute under --show-fields and smut_id does not; the phase records the before/after render bytes and any rendered example in the guide agrees with the new output.`

**VT-2** (sharpen; A-F11 + A-F15 + A-F1 + A-F7, all one public-round-trip home)
> old: `…and the renamed change receipt while retaining container vocabulary.`
> new: `…the renamed change receipt with an entity-shaped previous and --compact, a where on ordinary id returning only id-as-data while the old spelling returns an empty result at exit 0, DROP applied to an ordinary id, and the unchanged frontier, depths, edges and dangling containers.`

**VT-3** (sharpen; replaces the vacuous clause a source-presence mandate could satisfy; A-F6/B-F-4, B-F-8)
> old: `Version-1 index refusal names found/expected versions and exits 1, while the new loader corruption and ordinary-id cases stay covered.`
> new: `Version-1 index refusal names found/expected versions and exits 1; literal-key paired cases prove a nested ordinary id of null, empty string, number, array or object loads and is served at exit 0 while the same values under smut_id are corruption at exit 1 with the bracketed path; the unknown-version case no longer uses 2 as its unknown value.`

**VT-7** (NEW, `test_file = "src/learn.rs"`; A-F3 — DEC-020 names these pins as the guide's only guard and no VT touched them)
> `The guide's prose pins assert that smut_id is structural, an ordinary id is data, identity predicates and projections use smut_id, a store written by an older build is refused, and a query on the old spelling returns nothing rather than an error.`

**VT-8** (NEW, `test_file = "tests/render.rs"`; A-F4)
> `An entity carrying both smut_id and an ordinary id renders the ordinary id as an attribute under --show-fields while smut_id stays suppressed.`

**VA-2** (sharpen; B-F-9, A-F18, B-F-3 production-literal check; README comparison removed post-steering — see §f)
> old: `…classify residual structural id uses across source, tests and current docs.`
> new: `Run just check, including default/fault-injection tests and the guide examples its fixture suite executes; … classify residual structural id uses across source, tests, current docs, the normative brief and tests/fixtures/interview/PROVENANCE.md, starting from the repo-wide occurrence count rather than the slice's earlier figure. Check that the wire spelling appears in production code only at ENTITY_ID. Confirm README.md carries no edit, sweep or staging by this slice: it is the operator's 0.2 rewrite (CHR-001), lands outside the slice, and is not evidence here.`

**VT-6 ordering** (A-F20): a three-line comment above VT-6/VT-7/VT-8 records that
the later ids append after VT-6 by the immutable-id rule, so no reader "fixes" the
order. No criterion text changed.

### PHASE-03 — "Assemble integrated evidence and the reconciliation handoff"

**EN-1** (sharpen; closes the entrance enumeration the new PHASE-02 exits opened, and keeps EN-3's distinct EX-7 evidence duty)
> old: `PHASE-02/EX-1 through EX-6 hold; …`
> new: `PHASE-02/EX-1 through EX-6 and EX-8 through EX-10 hold; …`

**EX-1** (sharpen; B-F-10)
> old: `DEC-005 explicitly frees ordinary id and cites DEC-021;`
> new: `DEC-005 explicitly frees ordinary id, cites DEC-021 and corrects its quoted reference_id signature to the code's Reject error type;`

### `plan.md` (rationale paragraph, mirrored by hand because its substance changed)

> old: `The helper's use of reference_id makes this independent of the key's value. The corruption document must change with that check under STD-001. This intermediate commit retains the old key/version; it is not a separate release or a reinterpretation of the final design.`
> new: `The helper's *code* is key-agnostic, but its verdict is not: reference_id rejects a nested {"id":null} before the rename and accepts it as ordinary data after it, so this phase's tests and expectations are key-dependent and PHASE-02 respells them. The corruption document therefore lands with PHASE-02 rather than here — a format document that still declares version 1 must not state a rule the released version-1 reader never enforced. This intermediate commit retains the old key and version; it is not a release, not a supported reader of any existing version-1 store, and not a reinterpretation of the final design.`

---

## (c) The two sequencing repairs

### (i) PHASE-01 narrows version 1's acceptance set in place — B-F-1 / H1

**Reproduced.** A `format_version: 1` store holding
`{"id":"alpha","payload":{"tags":[{"id":null}]}}` serves `smut get alpha` at
**exit 0** and panics at `src/entity.rs:93` (exit 101) on
`query '{"from":["alpha"],"follow":["payload"]}'` — the RV-022 F-3 shape. After
PHASE-01 both refuse at exit 1 **as version 1**, while PHASE-01/VA-2 certifies the
versions have not moved and PHASE-01/EX-4 would have written into
`docs/store-format.md` — the file STD-001 makes "the store format's only record" —
a nested-identity corruption rule under `format_version 1` that the released v1
reader never enforced.

**Chosen repair (two criterion edits, no renumbering, no phase merge):**
1. PHASE-01/EX-4 no longer writes the format document at all. The corruption rule,
   path notation and worked diagnostic are recorded as pending material in the
   phase evidence; the document moves in PHASE-02, in the same unit as the version
   that explains it (PHASE-02/EX-5 now names every passage that must move).
2. PHASE-01/EX-5 states the release boundary the tree actually has: no tree of the
   phase is a release candidate or a supported reader of a version-1 store, it is
   exercised only against scratch stores, and the stricter loader enters service
   only with PHASE-02's bump. Evidence must record both halves.

**Why a keeper of this design accepts it.** Design §6's "old on-disk stores …
their version supplies an unambiguous refusal boundary" and DEC-022 are claims
about *consumer-facing* builds. No consumer ever holds a PHASE-01 tree: the plan's
own note says it "is not a separate release", and the phase cannot be tagged or
published under EX-5. What PHASE-01 must not do is leave a *record* that says
version 1 enforced a rule it did not — and after the repair it does not. The design
also explicitly leaves phase boundaries to planning, so the split survives, with
its one real defect (the false format record) removed.

**Rejected alternatives.**
- *Merge PHASE-01 into PHASE-02* (Seat B's E-1 recommendation). Rejected: it
  changes locked plan phase boundaries and needs operator assent for a benefit the
  split's isolation (recursive walk + diagnostic-path tests against the current
  key, verified before the 27-file sweep) already delivers; and it would strand
  PHASE-01's exit criteria, since "the loader is isolated from the sweep" is the
  only reason those criteria exist as a unit.
- *Gate the check on `FORMAT_VERSION == 2`* so it cannot fire on a v1 store.
  Rejected: the check becomes dead code in PHASE-01, unexercisable by PHASE-01's
  own corruption tests, and the gate would be removed two commits later — a
  criterion that cannot fail plus a churn the design does not sanction.
- *Leave EX-4 as written and accept the false format record*, disclosed only in
  notes. Rejected: STD-001 makes the document the format's only record; a record
  that says v1 refused nested malformed identity is a false contract statement in
  the same tree that is certified green.

### (ii) PHASE-01 → PHASE-02 rework, enumerated

What PHASE-01 writes and PHASE-02 rewrites:

| PHASE-01 artefact | key/version dependence | PHASE-02 rework |
|---|---|---|
| `validate_identity_tree` + call site in `src/store/mod.rs` | key-agnostic (uses `reference_id`) | none — the one invariant artefact |
| `scan_refs` panic text (`src/entity.rs:93`) | `id` | respelled once more (now by PHASE-02/EX-9) |
| `tests/store_corruption.rs` — `INDEX` (`format_version: 1`, `:17`), `ENTITY` (`{"id":"alpha"}`, `:18`) and every new literal-key case | `id` + v1 | respell key, bump every version literal, invert the unknown-version value (`:130`) — PHASE-02/EX-3, VT-3 |
| `tests/cli_contract.rs` malformed-stored-identity cases | `id` + v1 | respell — PHASE-02/VT-2, VT-3 |
| pending doc text (evidence only, after the repair) | `id` | becomes the PHASE-02/EX-5 doc change |

**How the plan now acknowledges it:** the rework is named in the criteria that
own each artefact (EX-3, EX-9, VT-2, VT-3, EX-5), and the silent-flip hazard is
closed at its source — PHASE-01/VT-1 requires the *literal* key spelling in the
injected malformed identity, so a test cannot keep passing while its subject
silently changes from "an `id` payload is refused" to "nested ordinary `id` is
data". PHASE-02/VT-3's literal-key paired cases are the negative control that
makes the mechanical respell safe, and `plan.md` no longer claims the phase is
independent of the key's value.

**Rejected alternative.** *Retain `ENTITY_ID` in PHASE-01's tests* (the current
mandate): it compiles and passes identically before and after the rename, so the
respell is an unreviewed no-op and the one test that must invert is invisible.

---

## (d) Proposed design revision — awaiting operator assent (`design.md` untouched)

The design is locked and design-run rev 39 is the accepted artefact. Each item
below says exactly what a revision would add or correct; none has been written.

**D1 — §2 "Reading, projection and canonical bytes" (and §5's behaviour table):
the stale-spelling consequence is undecided.**
*Problem:* §2 decides only that `id` "may legitimately match a predicate"; the
slice body explicitly hands the question back ("the design should decide whether
that is a test or just a documented sharp edge"), and the locked design never
answers it. *Evidence:* `smut query '{"from":["alpha"],"where":{"id":"nope"}}'`
→ `{"objects":[]}`, exit 0, reproduced; `src/predicate.rs:219` — only `$exists`
holds of a missing field. *Minimal wording, after the sentence that a field name
addresses the actual top-level field:* "A caller still addressing identity by the
old spelling therefore selects nothing and receives an empty result at exit 0, not
an error; the guide states that consequence, and one behaviour case pins it."
*Assent also settles whether it is (a) a test, (b) a documented sharp edge, or
(c) both — the plan currently does (c).*

**D2 — §3 "Unambiguous diagnostic paths": the existing identity-naming diagnostics
are undecided, and the one proposed reason is not declared contractual.**
*Problem:* §3 decides the *new* corruption reason and says "update its spelling"
for the `scan_refs` assertion, and says nothing about `Reject::InvalidId`,
`StoreError::IdDisagreement` or `describe()` — all of which name `id` as the
identity. *Evidence:* `src/error.rs:282`, `src/error.rs:389`,
`src/store/mod.rs:505`, `src/entity.rs:93`; `docs/store-format.md:252` quotes the
second verbatim and STD-001 makes worked diagnostics part of the contract.
*Minimal wording, replacing "A proposed exact reason is:":* "The corruption reason
is exactly `identity at <path> must be a nonempty string`. Every diagnostic that
names identity names `smut_id`: the invalid-identity refusal, the entity-file
index disagreement, the absent-identity description and the `scan_refs`
assertion. `docs/store-format.md` quotes the resulting strings."
*Consequence of assent:* PHASE-02/EX-9 becomes a design obligation rather than a
plan reading of §3.

**D3 — §4 "Design targets", the `src/render_markdown.rs` row: "retain rendering
behavior" is unsatisfiable as written.**
*Problem:* the renderer's suppression set *is* the constant
(`src/render_markdown.rs:186`, `:211`), so "retain the behaviour" and "the key
moves" cannot both hold for a data field named `id`; the design decides neither
outcome. *Evidence:* probe — with `{"id":"a","smut_id":"x","body":"hello"}` the
row prints `- **smut_id:** x`; after the swap a data `id` renders and `smut_id`
does not. *Minimal wording, appended to the row:* "Retain the mechanism: identity
display remains keyed on `ENTITY_ID`, so an ordinary `id` renders as an ordinary
attribute and `smut_id` is the suppressed identity. Pin the new output."

**D4 — §2 and §4 factual claims contradicted by the code (and the `src/order.rs`
row).**
*Problem:* §2 asserts in the present tense that `src/declare.rs` and
`src/conventions.rs` already use the constant, and §4 concludes "A listed consumer
that already reads `ENTITY_ID` needs no production rewrite solely to replace the
spelling." Both are false, and they are the whole remaining wire surface.
*Evidence:* `src/declare.rs:110` `entry.insert("id".to_owned(), …)`;
`src/conventions.rs:126` `("id", id_of(set))`; `rg -n ENTITY_ID src/conventions.rs`
→ no matches; `rg -n ENTITY_ID src/order.rs` → no matches (ordering is over `Id`
values, so the "sort tiebreak" in the affected-surface table is an overstatement).
*Minimal wording:* in §2, change "uses the constant when emitting" / "uses it for"
to "must emit … from the constant"; in §4, replace the sentence with "A listed
consumer that already reads `ENTITY_ID` needs no change beyond inspection, except
the change-receipt subject in `src/declare.rs` and the conventions seed in
`src/conventions.rs`, which spell the key as literals today"; correct the
`src/order.rs` row to "verify no identity key enters ordering".

**D5 — §4 "Documentation contract": the guide must state old stores are refused,
while `src/learn.rs` is chartered never to pin a version.**
*Problem:* two obligations collide and the design picks neither. *Evidence:*
`src/learn.rs:3-13` — "Descriptive and version-scoped, never version-pinning … a
version figure would be PHASE-07's to change". *Minimal wording, appended to the
refusal sentence:* "stated without a release or format figure." The plan already
requires the versionless phrasing (PHASE-02/EX-5); assent makes it the design's
choice rather than the plan's.

**D6 — §6 "Risks …": the surviving `smut_id` reservation has no documentation
duty.**
*Problem:* §6 says the store "still reserves `smut_id` at every depth", but no
section assigns the sentence that tells a caller, so the shipped surface still
teaches that only `id` was reserved. ISS-002's failure mode survives re-spelled.
*Minimal wording, appended to that paragraph:* "The guide must say that a payload
field named `smut_id` is still interpreted structurally, so such a payload cannot
be stored verbatim; `smut_id` frees `id`, not every identity-shaped field."

**D7 — §3 "Version 2 and recovery": recovery-before-validation plus no-repair-path
can publish exactly what the loader then refuses.**
*Problem:* a committed manifest whose staged put carries malformed structural
identity is replayed by recovery, then refused by the load that follows — a store
with no repair path by design. Narrow (a v2 manifest is normally written only by
this build) but it is the one path where the refusal promise and the no-repair
promise compose into permanent loss. *Evidence:* `docs/store-format.md:208-210`
("Recovery runs … strictly before store validation"). *Decision needed:* either
validate staged put payloads before replay, or state in §3 and the format document
that recovery may publish bytes the loader will then refuse. Not a plan repair.

**D8 — §3 "Unambiguous diagnostic paths": the bracket notation silently supersedes
the recorded decision's dotted example.**
*Problem:* DEC-021 records "an unambiguous field/index path" with the dotted
`payload.tags[2].smut_id`; the design introduces `$["payload"]["tags"][2]["smut_id"]`
without saying it supersedes the worked example. *Minimal wording, appended to the
notation paragraph:* "This supersedes the dotted example in DEC-021's failure
shape; the bracket form is chosen because it escapes key text." (DEC-021 is not in
the amendment set, so the design is the only home for the note.)

**D9 — §3, the word "proposed".** Folded into D2: leaving "A proposed exact reason
is:" is what let the plan soften a contractual string. If D2 is rejected, this
stays a known ambiguity in the verification story (PHASE-02/VA-3 can only check
agreement between two texts, not with a decided one).

**D10 — §4 "Design targets" README row, §4 "Documentation contract" ("README and
`learn` describe the shipped surface") and §5 "Execute the README transcript and
guide examples": the README is no longer this slice's write scope.**
*Problem, post-steering:* the operator is rewriting `README.md` for 0.2 and will land
it before `cargo publish` (`CHR-001`). Three design obligations still bind the slice
to that file: the code-impact row ("Update declarations, response examples and sharp
edges while preserving the operator's active draft"), the teaching obligation on
"README and `learn`", and §5's executed-README-transcript requirement. Left as
written, the design gates the slice on a surface this plan no longer owns.
*Minimal wording:* replace the §4 row with "README.md is out of this slice's write
scope: the operator rewrites it for 0.2 and it lands separately (`CHR-001`). This
slice must not edit, sweep or stage it and is not gated on its content."; in the
Documentation contract, change "README and `learn` describe the shipped surface" to
"`learn` describes the shipped surface; the README is the operator's rewrite and
lands outside this slice"; in §5, keep "execute the … guide examples using their
existing tests" and drop the README transcript from the slice's evidence, noting that
the suite still runs it. §4's "README remains the operator's live draft" and §6's
preservation paragraph stay as written — they are the boundary.

---

## (e) Left open, and why

- **All design items in (d).** `design.md` is locked at rev 39; the plan carries
  the minimal sanctioned reading for each, and none of my criteria exceeds what the
  design already obliges. If the operator rejects D1/D3, PHASE-02/EX-8's guide
  sentence and EX-10's pinned render output are the two criteria to revisit.
- **`README.md`.** Now an explicit boundary (operator steering, CHR-001): the
  operator's 0.2 rewrite lands separately before `cargo publish` and the slice is not
  gated on it. My earlier draft (direct-comparison evidence, a README sentence in
  EX-5/EX-10) was withdrawn; see §f. The fence blind spot
  (`tests/readme_example.rs:78` matches only "```console") is recorded for the
  operator's rewrite, not as a slice criterion.
- **`.doctrine/slice/004/slice-004.md`.** Its Risks section misattributes fixture
  coupling to `tests/traversal.rs`/`tests/selection.rs`. The correction is recorded
  in `notes.md` rather than edited in, because the slice body is the operator's
  framing and PHASE-02/EX-4 now carries the substance.
- **`tests/readme_example.rs`, `tests/traversal.rs:367`, `docs/store-format.md`.**
  All are implementation-phase edits; naming them in criteria is the most a review
  pass may do.
- **Seat B's rejected F-7.** PHASE-03/EN-1 already requires `EX-1…EX-6` and EN-3
  requires `EX-7`'s evidence; the union is complete, so appending a restatement
  would add a criterion that cannot fail. I did close the *analogous* gap I created
  by adding PHASE-02/EX-8…EX-10 (PHASE-03/EN-1 now names EX-8…EX-10).
- **Seat A's rejected F-17.** Pretty/compact byte forms and opaque-id rules rest on
  the unchanged suite, and PHASE-02/EX-6 already forbids weakening it; naming them
  without a test home would be a restatement of the design.
- **Concurrent writers in the tree.** During this task `DEC-028`, `rfc-001`,
  `backlog-013` and `CHR-001` (`.doctrine/backlog/chore/001/`) appeared or changed
  under me (`13:00:55`–`13:01:27` and later). They are the orchestrator's/operator's
  writes, not slice changes: not touched, not staged, not reverted (no git mutating
  command was run), and not counted as dirt. My diff is confined to
  `.doctrine/slice/004/` (`plan.toml`, `plan.md`, `notes.md`, `slice-004.md`);
  `README.md`'s in-flight changes are the operator's and pre-date this task.

---

## (f) Operator update — README becomes a boundary (B-F-6 re-disposed)

Mid-run the operator moved `README.md` out of this slice: it is being rewritten for
the 0.2 release, lands separately before `cargo publish`, and is recorded as
**CHR-001** (linked to SL-004 as references/concerns). B-F-6 is therefore
**operator-surface**, not a plan repair. I withdrew the README-shaped criteria my
first pass had added and replaced them with a boundary.

**Criteria changed (and only these):**

| criterion | before | after |
|---|---|---|
| PHASE-02/EN-3 | "Coordinate the README passages with the operator before editing them … a missing coordination answer blocks that edit" | "README.md is out of this slice's write scope: the operator is rewriting it for the 0.2 release and it lands separately before cargo publish (CHR-001). The phase neither edits, sweeps nor stages it, does not gate completion on its content, and leaves the operator's staged and working-tree changes untouched." |
| PHASE-02/EX-5 | listed "README examples" among the surfaces that must "consistently describe the new identity/version rules" | "README examples" removed from the list; appended "README.md is the operator's 0.2 rewrite (CHR-001): it is neither edited nor gated here, and this phase must not be blocked on its content." |
| PHASE-02/EX-10 | "…any guide **or README** rendered example agrees with the new output" | "…any rendered example **in the guide** agrees with the new output" |
| PHASE-02/VA-2 | "executed README/guide examples"; "the README's agreement with the binary is also compared directly" | "the guide examples its fixture suite executes"; "Confirm README.md carries no edit, sweep or staging by this slice: it is the operator's 0.2 rewrite (CHR-001), lands outside the slice, and is not evidence here." |
| PHASE-03/EX-4 | "…or unrelated README change is introduced" | "…or any README.md edit is introduced: the README is the operator's 0.2 rewrite (CHR-001) and lands outside this slice." |
| PHASE-03/VA-4 | "Review the normative brief, learn/README and docs/store-format.md…" | "Review the normative brief, the guide and docs/store-format.md…; README.md is the operator's 0.2 rewrite (CHR-001) and is outside this review; this phase is not gated on its content." |
| `slice-004.md` §Scope 6, Affected surface row, Risks bullet, human-checked bullet | README listed as a swept surface with "coordinate before sweeping" / "the README examples match the binary's output verbatim" | README removed from the swept surface and listed as out of scope under CHR-001; the human-check now covers `smut learn`, the normative brief and the six DECs, and states the README is not human-checked here |
| `plan.md` Notes | "Coordinate the actual passages immediately before the rename phase" | "The operator's README is being rewritten for the 0.2 release and lands outside this slice, before `cargo publish` (CHR-001). No phase edits, sweeps or stages it and no phase is gated on its content; preserve both of the operator's layers…" |

PHASE-01/EN-2's "preserve the operator's staged and unstaged README changes and
keep them out of unrelated commits" is unchanged: it was always a prohibition, not a
content obligation.

**Why this is a boundary and not a softening.** A softening deletes an obligation
and leaves nobody owning it; this assigns an owner and a date. The README's 0.2
rewrite now belongs to the operator, lands before `cargo publish`, and is tracked as
CHR-001 — so the statements the design wanted made are still going to be made, by
the party who is already writing them. What the plan keeps is everything that is
falsifiable *here*: no phase edits, sweeps or stages the file; no phase is blocked on
its content; no phase claims it as evidence; and the byte-identical preservation of
the operator's two layers stays a condition (PHASE-01/EN-2, PHASE-02/EN-3,
PHASE-03/EX-4). What it drops is the claim that *this slice* verifies the README,
which it never could while the file was being rewritten in parallel — the old text
would have gated the slice on a moving target and, worse, would have been satisfiable
by a gate that cannot see the file's examples at all (below). The slice's own human
surface is unchanged and still checked: `smut learn` (PHASE-02/VT-7's prose pins plus
PHASE-03/VA-4) and `docs/store-format.md` under STD-001.

**For the operator's rewrite (report only, no slice action):**
`tests/readme_example.rs:78` matches an exact ```` ```console ```` fence
(`inside = line.trim() == "```console"`). The README's in-flight examples sit in
```` ``` console ```` blocks — with a space — so the harness never matches them and
`cargo test` stays green over examples nothing executes. Three such blocks at
README:19, :30 and :52; only README:95's ```` ```console ```` is read. The rewrite
should either use the exact fence spelling or (better, and CHR-001's own subject)
teach the harness to accept both.

**Ghost of the withdrawn repair.** If the operator later reverses this decision, the
minimal reinstatement is PHASE-02/EX-5's README clause plus a VT on
`tests/readme_example.rs`; nothing else in the first pass depended on it.
