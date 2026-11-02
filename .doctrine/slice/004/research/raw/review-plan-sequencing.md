# SL-004 — Review Seat B: sequencing, assumptions, implementation-surface fit, design attack

Reviewer: Seat B (plan/design adversary). Read-only seat; no implementation, no tracked file touched.
Repo state reviewed: `4f70859` ("Refine SL-004 recovery controls and materialise ready plan"), working tree `MM README.md`.
Baseline probes run: `cargo fmt --check` → exit 0; `cargo test --test store_corruption` → 12 passed;
`cargo test --test readme_example` → 5 passed; `just --dry-run check` → 7 commands (full `cargo test` twice).
No full suite run (per brief).

Verdict in one line: **the three-phase split is a seam that a format break should not have**; PHASE-01
changes the acceptance set of `format_version` 1 while leaving the version at 1, then PHASE-02 rewrites
three of PHASE-01's four exit artefacts — and the plan's two load-bearing risk statements
(`reference_id`-independence, and the traversal/selection envelope coupling) are falsified by measurement.

---

## (a) Phase-by-phase sequencing ledger

### PHASE-01 — "Establish the loaded-identity invariant" (key = `id`, version = 1)

Entrance criteria: all satisfiable as written.

| EN | presumption | produced by? | verdict |
|---|---|---|---|
| EN-1 | design §1–3, §5, DEC-021, STD-001 read; "the existing `Store::load` root checks and `reference_id` rule match their current source" | n/a (reading) | OK — both match: `src/store/mod.rs:216-238` root checks, `src/entity.rs:31` `reference_id` |
| EN-2 | baseline green, README staged+unstaged preserved | n/a | OK but **hollow**: `readme_example` executes only ```` ```console ```` (README:95); the operator's ` ``` console ` blocks (README:19/30/52) are never executed, so "green" says nothing about the half-written draft |
| EN-3 | tests can build malformed stores in scratch with `ENTITY_ID` | pre-existing (`tests/store_corruption.rs::Scratch`) | OK |

Exit criteria: all achievable. `Store::run` loads the snapshot for *every* `Request`
(`src/store/mod.rs:296` → `src/cli.rs:69`), and `Get`/traversal both consume `snapshot.entities()`,
so EX-3's "both `get` and traversal refuse" is reachable exactly as written. `reference_id` is
reusable at a load boundary (`pub fn reference_id(&Value) -> Result<Option<&str>, Reject>`), and
`scan_refs` is non-fallible (`src/entity.rs:71`, panics at `:93`).

**What PHASE-01 actually writes (and what PHASE-02 must therefore rewrite)**

| PHASE-01 writes | keyed to | PHASE-02 must | rework |
|---|---|---|---|
| `validate_identity_tree` + call site in `src/store/mod.rs` | key-agnostic (uses `reference_id`) | nothing | **0** — the one genuinely invariant artefact |
| `scan_refs` panic text "an `id` that is not a nonempty string is corruption" (`src/entity.rs:94`) | `id` | respell again | 1 string, written twice |
| `tests/store_corruption.rs`: nested identity under objects/arrays, JSON-escaped paths, lexical/positional first-failure, unchanged entity/index bytes, root-check precedence — built on `INDEX` (`format_version: 1`, `:17`) and `ENTITY` (`{"id":"alpha",…}`, `:18`) | `id` + v1 | respell `INDEX` version 1→2, `ENTITY` key, and every literal in the new tests | ≥5 edit sites |
| `tests/cli_contract.rs`: `malformed_stored_identity` for `get` + `follow` (exit 1, empty stdout) + request-analogue exit 2 | `id` + v1 | respell | ≥3 edit sites |
| `docs/store-format.md`: new "nested identity" corruption rule, `$["payload"]["tags"][2]["id"]` path notation, worked diagnostic, new bullet in the §Corruption list | `id` | rewrite the key and the path component | ~3 passages, written twice |
| nothing about versions (VA-2 asserts they "have not yet moved") | — | — | — |

PHASE-02 additionally rewrites **six pre-existing** `docs/store-format.md` passages that PHASE-01
left alone: intro version line `:4`, index example `:55`, index bullet `:62`, manifest example `:86`,
two canonical-bytes "`id` first" sentences `:106`/`:111`, and both diagnostic lines `:249`
(`format_version 2 … reads only 1`, which becomes backwards) and `:252` (`holds id "b"`).

**Budget/acknowledgement verdict.** The double-write is *acknowledged in substance* — PHASE-01/EX-4
says "using the phase's current identity spelling in the same change", and PHASE-02/EX-5 requires
`docs/store-format.md` to "consistently describe the new identity/version rules". It is **not
enumerated and not budgeted**: no PHASE-02 criterion names the PHASE-01-authored corruption passages,
and the only PHASE-02 VT that targets `tests/store_corruption.rs` is VT-3
(keywords `format_version`, `smut_id`, `UnknownFormatVersion`) — a *source-presence* mandate, per
plan.md's own admission. An implementer who reads PHASE-01's doc text as "already done under STD-001"
can leave the format document with two identity spellings and stay green.

**Assertions weak enough to pass in both phases.** PHASE-01/VT-1 mandates that the new tests be
written using `ENTITY_ID`. Because `ENTITY_ID` is a `&str` const, a test that injects
`{ENTITY_ID: null}` compiles and *passes identically* before and after the rename while its semantic
target silently flips from "an `id` payload is refused" to "nested `{"id": null}` is ordinary data
that must load at exit 0". No criterion anywhere flips that direction — see finding F-4. (By
contrast, the "unchanged entity/index bytes" assertion inside VT-1 stays valid in both phases; that
one is fine.)

### PHASE-02 — "Move structural identity and the store format together"

Entrance: EN-1 (PHASE-01/EX-1..4 hold) ✔ produced by PHASE-01. EN-2 (record the pre-edit Git
revision; confirm no pre-existing `smut_id`) ✔ — verified: the fixture is committed at HEAD and
contains **zero** `smut_id`. EN-3 (README coordination) ✔ but see H4.

Exit criteria: all achievable; EX-4's "inverse comparison recovers the original parsed data" is
realisable with `serde_json` equality (fixture has 66 roots + 157 nested identity keys, all spelled
`"id"`, all keys (not prose) — a structural, not textual, rewrite is required and safe).

**Destroyed/reworked output of PHASE-01**: as enumerated above — 3 of 4 exit criteria produce
artefacts PHASE-02 rewrites. **New destruction PHASE-02 causes:** it makes the operator's live
`interview.local/.smut` (66 entities, `format_version: 1`) unreadable, with no repair path by
design. Measured: the live store's id set is *identical* to the committed fixture
(`{live: 66, fixture: 66, onlyLive: [], onlyFx: []}`), so no data is lost — but the plan contains no
pre-flight step that says so or that gives the operator a re-declare path (finding F-2).

### PHASE-03 — "Assemble integrated evidence and the reconciliation handoff"

**Entrance/exit mismatch (real defect):** EN-1 requires "PHASE-02/EX-1 through **EX-6** hold", but
EN-3 requires "PHASE-02/**EX-7** paired recovery discrimination and actual version-2 index/manifest
evidence". EX-7 is therefore presumed by an entrance criterion that EN-1's completion list omits —
the criterion that guarantees the evidence exists is the one EN-1 does not enumerate. (EN-2's claim
that "RV-023 contains no routed instrument findings" is verified: `doctrine show RV-023` →
`findings: 0`.)

PHASE-03 destroys nothing (evidence only) and correctly defers the six DEC amendments to
reconciliation.

### Ranked sequencing hazards

* **H1 (worst).** PHASE-01 narrows `format_version` 1's acceptance set in place. Measured today:
  a v1 store holding `{"id":"alpha","payload":{"tags":[{"id":null}]}}` serves `smut get alpha` at
  **exit 0** and panics at **exit 101** on `follow` — exactly RV-022 F-3. PHASE-01 makes both exit 1
  *as corruption, at version 1*, while `FORMAT_VERSION` and `MANIFEST_FORMAT_VERSION` stay 1 and
  PHASE-01/VA-2 explicitly certifies that they "have not yet moved". Design §6's claim that "Old
  on-disk stores are different: their version supplies an unambiguous refusal boundary" is therefore
  false for the intermediate tree. Nothing is released between phases, which limits the blast radius
  to "any build of the intermediate commit", but the ordering is still backwards: the corruption
  contract moves before the version that is supposed to explain it.
* **H2.** PHASE-02's fixture rewrite is guarded against a coupling that does not exist. Only
  `tests/fixture.rs` reads `tests/fixtures/interview/entities.json` (`rg -l 'entities\.json|FIXTURE' tests/`
  → `tests/fixture.rs` only). `tests/traversal.rs` and `tests/selection.rs` build their own synthetic
  stores (`declare_four`, `store(&[json!({"id":"a"})])`); their pins cannot be disturbed by fixture
  id order. The real fixture-coupled assertions are `tests/fixture.rs::CLUSTERS` (`:34`, five
  hardcoded ids) and the count-sensitive comment/assertions (`:697`), which the plan never names.
* **H3.** PHASE-02/EN-3 vs README. The operator's staged+unstaged hunks occupy README:16–32 — the
  exact region PHASE-02 must sweep — and already assert post-rename semantics
  ("Anything with a `smut_id` is upserted as a smut entity", README:30) plus a placeholder command
  (`smut declare ...`, README:31) inside a ` ``` console ` block that *no test executes*
  (`tests/readme_example.rs:78` matches `line.trim() == "```console"` only; README has three
  ` ``` console ` and one ` ```console `). So for the whole of PHASE-01 the human surface contradicts
  the binary and the gate stays green.
* **H4.** PHASE-02/EX-3 bricks the operator's only real store mid-slice (see F-2).
* **H5.** PHASE-03 EN-1/EN-3 EX-7 gap (above).

---

## (b) Measured-vs-claimed assumption table

| claim | verified how | actual | verdict |
|---|---|---|---|
| 974 literal `"id"` across 27 files under `src/`,`tests/`,`docs/`,`README.md` | `rg -o --no-filename '"id"' src tests docs README.md \| wc -l`; `rg -l … \| wc -l` | **974 / 27** exactly; top files `tests/fixtures/interview/entities.json:223`, `tests/traversal.rs:78`, `src/declare.rs:63` | **confirmed** |
| fixture has 223 `"id"` occurrences | `rg -o '"id"' …/entities.json \| wc -l`; `rg -o '"id":' …` | **223 / 223**; node walk: **223 objects carrying `id`** = 66 roots + 157 nested references; the 10 escaped `\"id\"` prose mentions do *not* match the quote form | **confirmed, and stronger than claimed** (all 223 are structural keys; a textual rewrite is unsafe for `id`-as-word, safe for `"id"`) |
| fixture is 66 entities | `jq length` → `66`; node id-set count → 66; PROVENANCE says 66 | **66** | **confirmed** |
| no pre-existing `smut_id` in the fixture | `grep -c 'smut_id' …/entities.json` | **0** | **confirmed** |
| "both format-version constants" | `rg -n 'FORMAT_VERSION\|MANIFEST_FORMAT_VERSION' src/` | `FORMAT_VERSION` (`src/store/index.rs:28`), `MANIFEST_FORMAT_VERSION` (`src/store/journal.rs:35`), plus two `FORMAT_VERSION_KEY` name consts. **Six literal version sites sit outside the constants**: `tests/store_corruption.rs` ×7 (`:17,121,130,145,185`), `tests/traversal.rs:367`, `docs/store-format.md` ×3, `smut-implementation-brief.org:82` | **partially false** — "both constants are 2" is necessary, not sufficient |
| `scan_refs` is non-fallible today | read `src/entity.rs:71,86-98` | `pub fn scan_refs(&Value) -> impl Iterator<Item=(&str,&str)>`; `panic!` at `:93` on a non-empty-string identity | **confirmed** |
| `reference_id` rule and signature | read `src/entity.rs:31-51` | `Result<Option<&str>, Reject>`; object + key present → nonempty string ⇒ `Ok(Some)`, else `Err(Reject::InvalidId{found})`; key absent ⇒ `Ok(None)`. DEC-005 documents `-> Result<Option<&str>, Error>` — drift | **confirmed, signature drift noted** |
| `reference_id` makes PHASE-01 "independent of the key's value" (plan.md) | code reading + semantics | the *code* is key-agnostic; the *verdict* is not: on `{"id":null}` it returns `Err` (corrupt) before the rename and `Ok(None)` (ordinary data) after. Every PHASE-01 expectation is key-dependent | **false as stated** |
| both `get` and `follow` reach stored data through the loaded snapshot (PHASE-01/EX-3 achievable) | `src/main.rs:94` → `src/cli.rs:69` → `src/store/mod.rs:296 self.load()`; probe | every `Request` loads first; `Get::recompute` and traversal both take `snapshot.entities()`. Probe: `get` exit 0, `follow` exit 101 panic at `src/entity.rs:93` | **confirmed; EX-3 achievable** |
| `just check` runs here, and what it executes | `just --dry-run check`; `cargo fmt --check`; two targeted suites | runs: `cargo build`; `cargo clippy --all-targets -- -D warnings`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo fmt --check`; `cargo test`; `cargo test --features fault-injection`. All toolchain present (just 1.58, rust 1.100.0-beta.7). Targeted baseline green | **confirmed** |
| `src/cli.rs` is "genuinely untouched" | `rg -n '"id"' src/cli.rs` + `#[cfg(test)]` at `:469` | 4 literal `"id"` sites, **all in `src/cli.rs` tests** (`:513,536,547,563`); production surface untouched, tests are not | **half true** — slice table says "nothing beyond the key", design §4 says "update structural examples/tests" |
| `docs/store-format.md` current content | read `:1-120`, `:240-260`; `rg -n 'id'` | 9 `id` lines; version stated **1** (`:4`, `:55`, `:62`, `:86`); canonical bytes "puts `id` first" (`:106`,`:111`); diagnostics `holds id "b"` (`:252`) and the now-backwards `format_version 2 … reads only 1` (`:249`); **§Recovery `:208-210` already states "Recovery runs … strictly before store validation"** | **confirmed** |
| DEC-005's text vs the slice's characterisation | `doctrine show DEC-005` | matches: "A `Value::Object` with a member `id` whose value is a nonempty string. Presence is the entire test" and "An embedded object carrying id cannot be inline metadata in 0.1 (documented in README and learn)". Slice's quote truncates "whose value is a nonempty string" but the substance and the reversal are accurate | **confirmed** |
| `src/declare.rs` "uses the constant when emitting a change receipt's subject" (design §2) | `rg -n 'ENTITY_ID' src/declare.rs` (7 uses) + line inspection | the subject is `src/declare.rs:110 entry.insert("id".to_owned(), …)` — a **raw literal**, not the constant | **FALSIFIED** |
| `src/conventions.rs` "uses it for the fixed-id declaration" (design §2) | read `src/conventions.rs:118-135` | `("id", id_of(set))` at `:126` — a **raw literal** | **FALSIFIED** |
| "A listed consumer that already reads `ENTITY_ID` needs no production rewrite solely to replace the spelling" (design §4) | same two measurements | the only two wire-key spells outside the constant are exactly these two, and both need writing | **FALSIFIED** |
| the traversal/selection pinned envelopes "were written against these ids' order and referrals" (slice Risks) | `rg -l 'entities\.json\|FIXTURE' tests/`; read `tests/traversal.rs`, `tests/selection.rs` | only `tests/fixture.rs` reads the fixture; traversal/selection use synthetic stores. Their pins are rename-affected but **fixture-independent** | **FALSIFIED** |
| 974 is "the sweep" | `rg -o '"id"' smut-implementation-brief.org \| wc -l`; repo-wide | brief adds **10** `"id"` + 4 `=id=`; repo-wide `"id"` = **984**. `PROVENANCE.md` has 0 quoted but 2 prose `id` mentions (`:75`,`:76`) | **undercount by 10+** (brief is in scope under DEC-023) |
| RV-023 has no routed findings | `doctrine show RV-023` | `findings: 0` | **confirmed** |

---

## (c) Unlisted-surface table

Impact table = `.doctrine/slice/004/slice-004.md` "Affected surface". Anything not matched by
`src/…`, `tests/**`, `docs/store-format.md`, `README.md`, `smut-implementation-brief.org` is UNLISTED.

| file | structural id / version uses | why it matters |
|---|---|---|
| `src/error.rs` | `IdDisagreement` display `"the entity file {entry} holds id {found}"` (`:389`); `IdDisagreement.found` doc "The `id` the file itself holds" (`:198`) | **production diagnostic string**, and the exact text `docs/store-format.md:252` quotes as a worked example. Not in the table at all → PHASE-02 discovers it only when the doc review (VA-3) trips. |
| `src/entity.rs:94` panic text | "an `id` that is not a nonempty string is corruption" | design §3 says "update its spelling" but assigns no phase; if PHASE-01 edits it to keep `id`, PHASE-02 rewrites it (double-write). discovered-late: no |
| `src/store/mod.rs:505` `describe()` | returns `"no \`id\`"` | same class: production diagnostic wording, listed only as `src/store/*`. |
| `tests/store_corruption.rs` | `INDEX` `format_version: 1` (`:17`); `ENTITY` `{"id":"alpha",…}` (`:18`); 6 more `"format_version":1` literals; `an_unknown_format_version_is_refused_without_migration` uses `"format_version":2` as the *unknown* value (`:130`) | the test that must **invert its value** (2→1), not just respell. Covered only by the generic `tests/**` row. discovered-late: no (fails loudly) |
| `tests/traversal.rs` | rebuilds `index.json` with a hardcoded `"format_version": 1` (`:367`, inside `shuffle_store`) | PHASE-02 breaks it; nothing names it. discovered-late: yes (only at gate time) |
| `tests/compatibility.rs` | pins the change-receipt key set exactly: `BTreeSet::from(["id","previous","status"])` (`:190`) and `(["id","status"])` (`:212`) | the only *exact-key-set* pin on `changes[]`; will fail loudly, but it is the guard that would catch a missed `src/declare.rs:110` |
| `tests/selection.rs:20,31` | harness reads `entity["id"]` from the fixture and rebuilds `json!({"id": id})` | if missed, panics — but it is the only place that *derives* from fixture keys |
| `tests/fixture.rs:34,697` | hardcoded `CLUSTERS` (5 ids) + "the committed fixture is 66 entities" assumptions | the true fixture-coupling the slice attributes to traversal/selection |
| `tests/fixtures/interview/PROVENANCE.md` | prose `:75` "Entities sorted by `id`", `:76` "serialisation order: `id`" | tracked; design §4 says "changes to fixture provenance explain the key adaptation" in prose but the **table omits the path** → a reader of the table alone misses it |
| `docs/store-format.md:208-210` | "Recovery runs … strictly before store validation: validating first would refuse a crashed-but-recoverable store as corrupt" | PHASE-01 adds a *stricter* validation behind that sentence; the sentence becomes the reason a hand-edited committed journal can publish then be refused (see E-3) |
| `smut-implementation-brief.org:38` | "Consequently an embedded object with a field named =id= cannot be ordinary inline metadata in v1. Document this convention." | a **normative requirement** that reverses for `id` and survives for `smut_id` — not a respell. Generic brief row covers it, but nothing forces the semantic read |
| `interview.local/.smut/` (untracked) | 66 entity files + `index.json` at `format_version: 1` | the operator's live store; id set measured identical to the fixture, so recoverable — but no phase mentions it |
| `.doctrine/knowledge/decision/{002,005,007,010,012,016}` | listed in design §4, **absent from the slice table** | reconciliation obligation; deferred by PHASE-03 by design (fine) |
| `justfile`, `Cargo.toml`, `flake.nix`, `arch.local.md` | no structural id, no version | correctly untouched — checked, not assumed |

---

## (d) Ranked findings

### F-1 — BLOCKER — PHASE-01 changes version 1's acceptance set without changing the version

`PHASE-01/EX-1..EX-3` add eager nested-identity refusal at the load boundary while
`PHASE-01/VA-2` certifies "the index/manifest versions and `ENTITY_ID` value have not yet moved".
A `format_version` 1 store that a shipped 0.2.0 build serves at exit 0 (measured: `smut get alpha` →
`{"…","objects":[{"id":"alpha","payload":{"tags":[{"id":null}]}}],…}`, exit 0) becomes exit 1
corruption with the version still reading 1. Design §6 and DEC-022 both rest on the version number
being the unambiguous refusal boundary; PHASE-01 breaks that property inside the slice's own tree, and
`docs/store-format.md` (written by PHASE-01/EX-4) will simultaneously say `format_version 1` and
document a corruption rule the released v1 reader does not enforce.

**Minimal repair (no renumbering):** append `PHASE-01/EX-5`: "The loader refusal is gated so it can
only fire on data the build that wrote it could not have produced, or the phase lands as part of
PHASE-02's change unit — no build of PHASE-01's tree is a candidate to run against any existing
`format_version` 1 store, including the operator's `interview.local/.smut`." Plus append
`PHASE-02/EN-4`: "PHASE-01's tree has not been released, tagged, or run against a live store."
Structural alternative (needs assent, E-1): merge PHASE-01 into PHASE-02.

### F-2 — MAJOR — PHASE-02 destroys the operator's only real store with no pre-flight step

`PHASE-02/EX-3` sets both constants to 2. The operator's `interview.local/.smut` is
`format_version: 1` with **66 entity files**, and DEC-022 forbids any reader, migration or repair path
— so on the first command after the bump it exits 1 permanently. Measured id-set equality
(`onlyLive: [], onlyFx: []` against the committed fixture) means the data *is* recoverable by
re-declaring `tests/fixtures/interview/entities.json`, but no criterion says so, and the slice's own
Risk list describes the store as "one real store in the wild … outside this repo" without a
pre-flight.

**Minimal repair:** append `PHASE-02/EN-4`: "Before the version constants move, record the operator's
re-declare path for `interview.local/.smut` (its 66 ids are identical to the committed fixture) and
note the exact command; the operator has acknowledged that the live store stops loading at this
phase." (Name it in notes.md's `Open` list, which currently says only "coordinate the operator's live
README".)

### F-3 — MAJOR — the design's two production-literal claims are false, and they are the whole wire surface

`src/declare.rs:110` emits `changes[].id` from a raw `"id"` literal; `src/conventions.rs:126` seeds
the conventions declaration from `("id", id_of(set))`. Design §2 asserts both "use the constant", and
§4 concludes "A listed consumer that already reads `ENTITY_ID` needs no production rewrite solely to
replace the spelling." Measurement: the only production spellings of the key outside
`src/entity.rs:17` are **exactly these two plus three `src/learn.rs` example consts**
(`:256,262,267`). An implementer trusting §4 skips the two envelope literals; the receipt keeps `id`
while `previous` moves to `smut_id`.

**Minimal repair:** append `PHASE-02/EX-8`: "No production literal spells the identity key: a test
asserts `src/` contains exactly one occurrence of the wire spelling, in `src/entity.rs::ENTITY_ID`
(comments and test modules excluded), and `changes[].smut_id` / the conventions seed come from the
constant." Record the §2/§4 correction for audit.

### F-4 — MAJOR — nothing in the plan asserts the *new* load-boundary acceptance of ordinary nested `id`

PHASE-01 writes corruption tests against `ENTITY_ID` (VT-1's keyword list mandates it). After the
rename those assertions keep passing with unchanged code while their subject silently becomes
ordinary data — the exact inverse of what the test now needs to prove. PHASE-02/EX-2 speaks of
ordinary `id` surviving "at roots and at depth", but no criterion places an ordinary `id` in a
**stored** entity and requires the loader to serve it; the design's own table row ("ordinary `id` with
those values is valid data") has no home.

**Minimal repair:** append `PHASE-02/EX-9`: "One paired test with literal keys (not `ENTITY_ID`):
a stored entity whose nested `id` is `null`/`7`/`""`/`[]`/`{}` loads and is served at exit 0, while
the same entity with `smut_id` carrying those values is `StoreError::Corrupt` at exit 1 with the
bracketed path." (This is also the negative control that makes the mechanical respell of PHASE-01's
tests safe.)

### F-5 — MAJOR — the fixture risk statement names the wrong files

Slice Risks: "the pinned envelopes in `tests/traversal.rs` and `tests/selection.rs` were written
against these ids' order and referrals". Measured: only `tests/fixture.rs` reads the fixture.
`tests/traversal.rs` builds four synthetic entities; `tests/selection.rs` builds `{"id":"a"}`-style
stores. So the "spelling-only" imperative is enforced against surfaces with no such coupling, while
the real coupling — `tests/fixture.rs::CLUSTERS` (5 hardcoded ids, `:34`) and its count-sensitive
assertions (`:697`) — is unnamed.

**Minimal repair:** append `PHASE-02/EX-10`: "`tests/fixture.rs`'s hardcoded `CLUSTERS`, executed
guide constants and count-sensitive assertions are re-derived against the adapted fixture and named in
the phase evidence; traversal/selection pins are documented as rename-affected but
fixture-independent." Correct the slice Risk text in notes.

### F-6 — MAJOR — the human surface already contradicts the binary and the gate cannot see it

The operator's staged+unstaged README (README:16–32) asserts "Anything with a `smut_id` is upserted as
a smut entity" and contains a literal placeholder `smut declare ...` inside a ` ``` console ` block.
`tests/readme_example.rs:78` only recognises the fence exactly ```` ```console ````, so all three
` ``` console ` blocks are invisible; the suite passes (measured: 5 passed). PHASE-01 therefore
completes with `just check` green over a README that documents a feature the binary does not have, and
PHASE-02 must edit *inside* the operator's live hunk to fix it.

**Minimal repair:** append `PHASE-01/EN-4`: "The operator's README draft region (README:16–32) is
left byte-identical; the contradiction it introduces (`smut_id` prose + placeholder command, no gate
covers it) is recorded in notes.md and raised with the operator at PHASE-02/EN-3." Raise a backlog item
for the fence-variant blind spot in `tests/readme_example.rs` (the test's own doc comment claims
"exactly one `console` block").

### F-7 — MINOR — PHASE-03 presumes an exit it does not enumerate

`PHASE-03/EN-1` requires PHASE-02 `EX-1…EX-6`; `PHASE-03/EN-3` requires `PHASE-02/EX-7`. Repair:
append `PHASE-03/EN-4` restating EN-1 as "PHASE-02/EX-1 through EX-7 hold" (ENTRY append) or add the
`EX-7` reference to EN-3's basis. This is the only entrance/exit gap in the chain.

### F-8 — MINOR — `docs/store-format.md` version literals and the backwards diagnostic

`docs/store-format.md:249` currently prints `the store index has format_version 2, and this build reads
only 1`. After the bump this example is exactly inverted. PHASE-02/EX-5 covers it in substance; the
risk is that PHASE-01/EX-4 already touches the same document and a reader treats the file as done.
Repair: append `PHASE-02/EX-11` requiring the format document to hold exactly one identity spelling and
one version pair at phase exit, with the four version sites (`:4,:55,:62,:86`) and both diagnostics
(`:249,:252`) named.

### F-9 — MINOR — the measured sweep undercounts

974/27 excludes the brief's 10 `"id"` occurrences and `PROVENANCE.md`'s 2 prose `id` mentions; the brief
also carries a normative requirement that *reverses* (`:38`). Repair: PHASE-02/VA-2's residual
classification should cite 984 repo-wide as the starting count, with the brief and PROVENANCE in the
classification set.

### F-10 — MINOR — DEC-005 documents a signature the code does not have

DEC-005 shows `fn reference_id(v: &Value) -> Result<Option<&str>, Error>`; the code returns `Reject`
(`src/entity.rs:31`). Since DEC-005 is being amended under DEC-025 anyway, the amendment should correct
the type, and PHASE-03/EX-1's amendment mapping should say so explicitly (it currently only covers the
predicate/hoisting/label prose).

---

## (e) Design-level defects needing operator assent

**E-1 — The three-phase split is the wrong seam for a format break.** The split's stated justification
(plan.md: "The helper's use of `reference_id` makes this independent of the key's value") is true of the
helper's *code* and false of its *semantics*: `reference_id`'s verdict on `{"id":null}` is `Err` before
the rename and `Ok(None)` after, so PHASE-01's entire test and documentation surface is key-dependent
and is rewritten at PHASE-02. What the split buys is a smaller diff, not a smaller risk — and it costs a
window in which the loader is stricter than the version number says. **Recommendation:** one atomic
rename+validate+bump phase (PHASE-01 and PHASE-02 executed and verified as a single change unit, phase
ids unchanged), with PHASE-03 untouched. Requires operator assent because it changes the plan's phase
boundaries and verification cadence.

**E-2 — Recovery-before-validation plus no-repair-path can brick a store the slice itself repaired.**
Design §3 deliberately declines "a separate semantic parser for journal entity bytes", and
`docs/store-format.md:208-210` states recovery runs strictly before validation. Consequence: a
committed manifest whose staged put carries malformed structural identity is *published* by recovery,
then refused as corruption by the load that follows — an unloadable store with, by design, no repair
path. Narrow (a v2 manifest can normally only be written by this build) but it is the one path where
the loader's refusal promise and the no-repair promise compose into permanent loss. **Decision needed:**
either validate staged put payloads before replay (a new check in manifest validation), or state in
`docs/store-format.md` and the design that recovery may publish bytes the loader will then refuse. This
is a design amendment, not a plan repair.

**E-3 — `format_version` must be made to mean one acceptance set, or PHASE-01 must not exist.** The
corollary of F-1: if the operator wants the version number to remain the refusal boundary (DEC-022,
design §6), the corruption contract cannot move under a version that does not move. Either fold the
loader change into the bump, or accept that "version 1" means "whatever build you have" — which
contradicts the design's stated boundary.

**E-4 — "Ordinary `id` becomes data" is bounded by a new reservation, and the slice's framing
understates it.** `smut_id` is reserved at every depth, so a payload whose own field is literally
`smut_id` (imported records commonly have one) still cannot be stored verbatim — it becomes a
reference and is reported `missing`, at exit 0. ADR-001 and design §6 accept this ("does not promise
literal storage of every possible JSON payload without conventions"), and the slice-004 context says
"closes this for a caller's own `id` — while reserving `smut_id` at every depth, which the ADR states as
a boundary rather than a disappearance." No action required unless the operator reads "closes ISS-002"
as "any payload's identity field is now storable"; if so, this belongs in `learn` as a documented sharp
edge (the plan currently has no criterion requiring it in the guide).

**E-5 — Silence is documented, not enforced.** The design says "the design should decide whether that
is a test or just a documented sharp edge" (slice Risks). PHASE-02's criteria contain no "a `where` on
`id` returns empty at exit 0" test, and design §5's behavior table does not carry it either. The
failure mode the research most emphasizes (stale `id` query → empty result at exit 0) therefore has no
gate. Recommend appending `PHASE-02/EX-12` (behavior case) rather than relying on `learn` prose.

---

## Appendix — commands run (exact)

```
rg -o --no-filename '"id"' src tests docs README.md | wc -l                       → 974
rg -l '"id"' src tests docs README.md | wc -l                                     → 27
rg -c '"id"' src tests docs README.md | sort -t: -k2 -rn                          → (per-file table)
rg -o '"id"' tests/fixtures/interview/entities.json | wc -l                       → 223
rg -o '"id":' tests/fixtures/interview/entities.json | wc -l                      → 223
rg -o '\\"id\\"' tests/fixtures/interview/entities.json | wc -l                   → 10 (prose, not key form)
jq length tests/fixtures/interview/entities.json                                  → 66
grep -c 'smut_id' tests/fixtures/interview/entities.json                          → 0
node -e '<walk fixture>'                                                          → {car:223, roots:66, nested:157}
node -e '<live-store vs fixture id sets>'  → {live:66, fixture:66, onlyLive:[], onlyFx:[]}
head -5 interview.local/.smut/index.json                                          → "format_version": 1
rg -n 'FORMAT_VERSION|MANIFEST_FORMAT_VERSION' src/                               → index.rs:28, journal.rs:35 (+2 key consts, 15 matches)
rg -n 'format_version' tests docs README.md smut-implementation-brief.org          → store_corruption.rs×7, traversal.rs:367, docs×3, brief:82
rg -c 'ENTITY_ID' src/                                                            → 27 refs across 6 files
awk '<pre-cfg(test) filter>' for '"id"' in src/                                   → declare.rs:110, conventions.rs:126, learn.rs:256/262/267, entity.rs:17
rg -n 'ENTITY_ID' src/declare.rs                                                  → 7 uses, none emits changes[] subject
sed -n '95,130p' src/declare.rs                                                   → :110 entry.insert("id".to_owned(), …)
rg -n 'fn reference_id' -A20 src/entity.rs                                        → Result<Option<&str>, Reject>
rg -n 'pub fn scan_refs' -A28 src/entity.rs                                       → non-fallible, panic at :93
sed -n '190,245p' src/store/mod.rs                                                → root checks, no nested validation
sed -n '260,320p' src/store/mod.rs; sed -n '60,110p' src/main.rs; sed -n '60,80p' src/cli.rs → load covers every Request
mktemp scratch store probe                                                         → get: exit 0 (serves {"id":null}); follow '*': exit 101 panic src/entity.rs:93
rg -l 'entities\.json|FIXTURE' tests/                                             → tests/fixture.rs only
rg -n '^\s*```' README.md; sed -n '60,105p' tests/readme_example.rs                → 3×"``` console", 1×"```console"; matcher is exact-"```console"
git status --short; git diff README.md; git diff --cached README.md                → MM README.md; hunk at :16-32 adds smut_id prose + `smut declare ...`
just --dry-run check                                                              → 7 commands (build, clippy×2, fmt, test×2)
cargo fmt --check                                                                 → exit 0
cargo test --test store_corruption                                                → 12 passed
cargo test --test readme_example                                                   → 5 passed
doctrine show RV-023                                                              → findings: 0
doctrine show DEC-005/DEC-021/DEC-022/DEC-023/DEC-024/STD-001/ADR-001             → read in full
```

Not run (out of scope by brief): full `cargo test`, fault-injection suite, `cargo clippy`.
