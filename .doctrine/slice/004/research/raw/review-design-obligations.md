# SL-004 — Review Seat A: design obligations vs plan capture

**Artefact under review:** `.doctrine/slice/004/design.md` (locked, design run rev 39), against
`.doctrine/slice/004/plan.toml` (PHASE-01..03), the three runtime sheets, `slice-004.md`,
`notes.md`, `research/research.md`, and the binding neighbours (ADR-001, STD-001, SL-004,
DEC-002/005/007/010/012/016/020/021/022/023/024/025).

**Method:** every normative sentence in the design was inventoried and traced to a plan
criterion. Code premises were verified in `src/` with `rg`/`read`, claimed counts were
re-measured, and two behaviour claims were probed against the **current** binary in
`mktemp -d` scratch stores (analogue probes: today's `smut_id`-as-data is tomorrow's
`id`-as-data). Working tree untouched; no test suite run.

## Counts

| class | obligations | FAITHFUL | WEAKENED | STRENGTHENED | SPLIT | ABSENT | UNVERIFIABLE |
|---|---|---|---|---|---|---|---|
| BEHAVIOUR | 64 | 55 | 8 | 0 (1 noted inside a FAITHFUL row) | 0 | 1 | 0 |
| DOC | 13 | 7 | 4 | 0 | 1 | 1 | 0 |
| RECORD | 11 | 10 | 1 | 0 | 0 | 0 | 0 |
| CONSTRAINT | 36 | 19 | 10 | 0 | 0 | 7 | 0 |
| ASSUMPTION | 4 | 2 | 1 | 0 | 0 | 1 | 0 |
| BEHAVIOUR/DOC (joint) | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| **total** | **129** | **93** | **24** | **1** | **1** | **11** | **0** |

UNVERIFIABLE is 0 at ledger level by construction: where a VT aims at a real test file the
obligation is carried (and TASK 3 records that the *keyword* mandate is vacuous); the two
loader-shape claims that no test can assert are recorded as `ABSENT`/paper-only instead.
Overall: 72% FAITHFUL, 19% WEAKENED, 9% ABSENT-or-SPLIT. Every ABSENT row is named in a
finding below.

Paper-only VTs: 2 (`PHASE-01/VT-3`, `PHASE-02/VT-1`) + 1 partially (`PHASE-02/VT-5`).
VAs that cannot fail: 5 (`PHASE-01/VA-1`, `PHASE-02/VA-3`, `PHASE-02/VA-2` classification leg,
`PHASE-03/VA-1`, `PHASE-03/VA-4`).

---

# (a) Obligation ledger

Quotes are ≤15 words and paraphrase-tight to the design's letter. `captured by` names the
criterion that carries the obligation; `—` means no criterion carries it.

## §1 Change and boundaries

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§1-1 | `smut_id` is the structural identity key; ordinary `id` freed at every depth | BEHAVIOUR | PHASE-02/EX-1, EX-2; VT-2 | FAITHFUL |
| D§1-2 | identity key remains reserved (`smut_id` at every depth) | CONSTRAINT | PHASE-02/EX-1 ("stay unchanged") | WEAKENED — reservation itself untested/undocumented |
| D§1-3 | other value conventions, incl. top-level `"DROP"` sentinel, unchanged | CONSTRAINT | — (only by unchanged existing suite) | ABSENT |
| D§1-4 | rename covers entity files, declarations, references, returned entities, `previous`, conventions declaration, `changes[].smut_id` | BEHAVIOUR | PHASE-02/EX-1; VT-2, VT-5 | FAITHFUL |
| D§1-5 | container-named values keep names: `ids`, `deleted`, `missing`, `frontier`, `depths`, `edges[].source/.target`, `dangling[].source` | CONSTRAINT | PHASE-02/EX-1; VT-2 ("container vocabulary") | WEAKENED — `depths`/`edges`/`frontier`/`dangling` not named; held only by unchanged traversal tests |
| D§1-6 | store format 1→2; old stores **and** old manifests refused by version, exit 1, naming found and expected | BEHAVIOUR | PHASE-02/EX-3, EX-7; VT-3, VT-4, VT-6 | FAITHFUL |
| D§1-7 | no migration, compat reader, alias, repair path, release-specific advice | CONSTRAINT | PHASE-03/EX-4 | FAITHFUL (alias not named) |
| D§1-8 | eager recursive check that every structural `smut_id` is a nonempty string, before snapshot insertion | BEHAVIOUR | PHASE-01/EX-1; VT-1, VT-3 | FAITHFUL |
| D§1-9 | malformed stored identity = corruption, exit 1, naming entity file and offending path | BEHAVIOUR | PHASE-01/EX-2; VT-1, VT-2 | FAITHFUL |
| D§1-10 | `scan_refs` retains non-fallible interface and validated precondition | CONSTRAINT | PHASE-01/EX-1; VA-2 | FAITHFUL |
| D§1-11 | implementation brief remains normative, updated in place | DOC | PHASE-02/EX-5; VA-3, VA-4 | FAITHFUL |
| D§1-12 | `docs/store-format.md` single on-disk record, moves in the same unit | DOC | PHASE-01/EX-4; PHASE-02/EX-5; VA-3 | SPLIT (01 corruption prose, 02 version prose) |
| D§1-13 | layout, basenames, locking, journal publication, durability, query/patch/missing semantics unchanged | CONSTRAINT | PHASE-02/EX-3, EX-6; PHASE-03/EX-4 | FAITHFUL |
| D§1-14 | no minting/rename verb, short flags, Mermaid in this slice | CONSTRAINT | PHASE-03/EX-4 | WEAKENED — only asserted at the last phase |

## §2 Identity rules and wire contract

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§2-1 | `ENTITY_ID` keeps its name, becomes `"smut_id"` | BEHAVIOUR | PHASE-02/EX-1; VT-1 | FAITHFUL |
| D§2-2 | literal wire-contract assertion pins its value (DEC-020) | BEHAVIOUR | PHASE-02/VT-1 | FAITHFUL (assertion real; VT itself paper-only, see TASK 3) |
| D§2-3 | no identity module or alias added | CONSTRAINT | — | ABSENT |
| D§2-4 | `reference_id`, `extract`, `scan_refs` signatures unchanged | CONSTRAINT | PHASE-01/EX-1; PHASE-03/VA-2 | FAITHFUL |
| D§2-5 | `reference_id` still distinguishes absence from invalid identity | BEHAVIOUR | PHASE-01/EX-2; VT-1 | FAITHFUL |
| D§2-6 | `declare.rs` uses the constant for the change-receipt subject | BEHAVIOUR | PHASE-02/EX-1; VT-2 | FAITHFUL (already true: `src/declare.rs:277`) |
| D§2-7 | `conventions.rs` uses it for the fixed-id declaration JSON | BEHAVIOUR | PHASE-02/EX-1; VT-5 | FAITHFUL (as requirement; design's present-tense claim is false — see F8) |
| D§2-8 | declaration-interpretation table: reference / hoist / refuse / generate / no-op / recurse | BEHAVIOUR | PHASE-02/EX-2; VT-2 | FAITHFUL |
| D§2-9 | whitespace-only identity valid; ids case-sensitive, opaque, unnormalised | CONSTRAINT | — (unchanged suite) | WEAKENED |
| D§2-10 | generation avoids stored ids, in-batch explicit ids, already-generated ids | BEHAVIOUR | PHASE-02/EX-1 ("root generation") | FAITHFUL |
| D§2-11 | ordinary payload `id` does not occupy the structural namespace | BEHAVIOUR | PHASE-02/EX-2; VT-2 | FAITHFUL |
| D§2-12 | root without key is generated; `{"id":"external"}` root stores `id` as data | BEHAVIOUR | PHASE-02/EX-2; VT-2 ("root generation versus nesting") | FAITHFUL |
| D§2-13 | no compatibility interpretation, no typo refusal | CONSTRAINT | PHASE-03/EX-4 | FAITHFUL |
| D§2-14 | `where`/`select`/`order`/`follow` address real top-level fields; identity via `smut_id` | BEHAVIOUR | PHASE-02/EX-2; VT-2 | WEAKENED — positive case only; the stale-caller silence is unhandled (F1) |
| D§2-15 | query grammar does not change | CONSTRAINT | PHASE-02/EX-1 (indirectly); existing suite | WEAKENED |
| D§2-16 | every projection retains `smut_id`; `select: []` returns identity alone | BEHAVIOUR | PHASE-02/EX-1; VT-2; PHASE-03/EX-1 | FAITHFUL |
| D§2-17 | ordinary `id` projected like any data field | BEHAVIOUR | PHASE-02/EX-2; VT-2 | FAITHFUL |
| D§2-18 | entity-shaped objects serialise `smut_id` first, rest lexical, everywhere | BEHAVIOUR | PHASE-02/VT-1 | FAITHFUL (paper-only, F6) |
| D§2-19 | embedded ordinary objects stay all-lexical; arrays keep order | BEHAVIOUR | PHASE-02/VT-1 | FAITHFUL |
| D§2-20 | file output pretty + trailing newline; stdout compact + trailing newline | BEHAVIOUR | — (unchanged suite) | WEAKENED |
| D§2-21 | `declare` response envelope: `changes[].smut_id`, `ids`, `missing` | BEHAVIOUR | PHASE-02/VT-2 | FAITHFUL |
| D§2-22 | `previous` stays the full pre-batch entity using `smut_id`; compact/status rules retained | BEHAVIOUR | PHASE-02/VT-2 ("renamed change receipt") | WEAKENED — `previous` and `--compact` not named (F11) |
| D§2-23 | response containers and traversal metadata keep §1 names | CONSTRAINT | PHASE-02/EX-1 | WEAKENED (same as D§1-5) |
| D§2-24 | verification must exercise root and nested cases separately: mixed objects, projection of each, malformed, nested hoisting, forward refs, non-trivial canonical order | BEHAVIOUR | PHASE-02/VT-1, VT-2 | FAITHFUL |
| D§2-25 | ordinary `id` follows field patch rules incl. top-level `"DROP"`; no new literal-assignment syntax | CONSTRAINT | — | ABSENT (F7) |

## §3 Loader validation and format refusal

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§3-1 | `Store::load` owns it; private `validate_identity_tree(entry, value)` in `src/store/mod.rs` after root checks, before insert | BEHAVIOUR | PHASE-01/EX-1; VT-3 | FAITHFUL (VT-3 paper-only) |
| D§3-2 | walk calls `reference_id` for objects, descends values/arrays, ignores scalars | BEHAVIOUR | PHASE-01/EX-1; VT-3 | FAITHFUL |
| D§3-3 | predicate decides validity; store maps failure to `StoreError::Corrupt` | BEHAVIOUR | PHASE-01/EX-1, EX-2 | FAITHFUL |
| D§3-4 | no storage type or filesystem dependency enters `src/entity.rs` | CONSTRAINT | — | ABSENT |
| D§3-5 | root-object and root/index-disagreement diagnostics keep precedence | BEHAVIOUR | PHASE-01/EX-3; VT-1 | FAITHFUL |
| D§3-6 | walk does **not** stop at a valid reference-shaped object | BEHAVIOUR | PHASE-01/VT-1 (objects/arrays only) | WEAKENED |
| D§3-7 | no target-existence check, no placeholders, no rewrite, no new normalization pass | CONSTRAINT | PHASE-01/EX-3 | FAITHFUL |
| D§3-8 | ordinary `id` never validated as identity, even as null/array/object; `smut_id` nested inside it still found | BEHAVIOUR | PHASE-01/VT-1 | WEAKENED (nested-inside-id case unnamed) |
| D§3-9 | linear work; no re-read or re-parse of the file | CONSTRAINT | — | ABSENT |
| D§3-10 | every store-backed read shares the boundary; `get` and traversal both refuse, even outside the followed field | BEHAVIOUR | PHASE-01/EX-3; VT-1, VT-2 | FAITHFUL |
| D§3-11 | path notation: `$` root, JSON-quoted bracketed keys, numeric array positions | BEHAVIOUR | PHASE-01/EX-2; VT-1 | WEAKENED — the `$` root marker never appears in the plan |
| D§3-12 | object-key text JSON-escaped so separators cannot be confused | BEHAVIOUR | PHASE-01/VT-1 ("JSON-escaped paths") | FAITHFUL |
| D§3-13 | diagnostic notation only, not query syntax | CONSTRAINT | — | ABSENT (low risk) |
| D§3-14 | keys lexical, arrays positional, first failure; deterministic per entity; index iteration decides entity order | BEHAVIOUR | PHASE-01/VT-1 ("lexical/positional first failure") | FAITHFUL |
| D§3-15 | error names indexed basename and the identity member's path | BEHAVIOUR | PHASE-01/EX-2; VT-1 | FAITHFUL |
| D§3-16 | exact reason text `identity at … must be a nonempty string` | DOC | PHASE-01/EX-2 (elements only) | WEAKENED (F5) |
| D§3-17 | invalid request identity exit 2; malformed stored identity exit 1; no success envelope | BEHAVIOUR | PHASE-01/EX-2; VT-1, VT-2 | FAITHFUL |
| D§3-18 | keep the internal `scan_refs` assertion; update its spelling | BEHAVIOUR/DOC | — | ABSENT (F2) |
| D§3-19 | both `FORMAT_VERSION` and `MANIFEST_FORMAT_VERSION` become 2 | BEHAVIOUR | PHASE-02/EX-3; VT-3 | FAITHFUL |
| D§3-20 | keep existing parsers and refusal classes; no permissive reader, no new error variant | CONSTRAINT | PHASE-02/EX-3; PHASE-01/VA-2 | FAITHFUL |
| D§3-21 | index refusal text `has format_version 1, and this build reads only 2` | DOC | PHASE-02/VT-3 (found/expected only) | WEAKENED (F5) |
| D§3-22 | manifest refusal names found and expected versions, exit 1 | BEHAVIOUR | PHASE-02/VT-3, VT-4 | FAITHFUL |
| D§3-23 | committed version-1 manifest fails before any staged put/removal replays | BEHAVIOUR | PHASE-02/EX-7; VT-4, VT-6 | FAITHFUL (+ plan adds replay control: STRENGTHENED, benign) |
| D§3-24 | do not move store validation ahead of recovery; bookkeeping may precede load | CONSTRAINT | PHASE-02/EX-3 ("existing recovery order remains") | FAITHFUL |
| D§3-25 | refusal promise is "no replay", not "zero bookkeeping mutations" | CONSTRAINT | PHASE-02/EX-3, EX-7 | FAITHFUL |
| D§3-26 | no separate semantic parser for journal entity bytes; normal loader validates | CONSTRAINT | PHASE-02/EX-3 | FAITHFUL |
| D§3-27 | `docs/store-format.md` describes version break, nested-identity rule, path notation, worked diagnostics | DOC | PHASE-01/EX-4; PHASE-02/EX-5, VA-3 | FAITHFUL |

## §4 Code impact and documentation ownership

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§4-1 | per-path code-impact contract (entity, declare, conventions, store, walk/select/order, render, cli, learn, tests, docs, README, brief, six records) | CONSTRAINT | PHASE-02/EX-1, EX-5, EX-6 | WEAKENED — `render_markdown.rs` and `learn` have no dedicated criterion (F3, F4) |
| D§4-2 | validator imports `reference_id`; dependency direction unchanged | CONSTRAINT | PHASE-01/VT-3 | FAITHFUL (grep-only) |
| D§4-3 | use `StoreError::Corrupt` and current display; no new public error variant | CONSTRAINT | PHASE-01/EX-2; VA-2 | FAITHFUL |
| D§4-4 | internal `Id`/`Change.id`/map-key/basename names need not change | CONSTRAINT | PHASE-03/EX-4 (indirectly) | FAITHFUL |
| D§4-5 | brief states product intent; identity edits must not claim shipped features | DOC | PHASE-02/EX-5; VA-3 | FAITHFUL |
| D§4-6 | store-format records actual on-disk contract; brief's version example agrees | DOC | PHASE-02/VA-3 | FAITHFUL |
| D§4-7 | README and `learn` describe the shipped surface | DOC | PHASE-02/EX-5; VA-4 | FAITHFUL |
| D§4-8 | guide must teach `smut_id` structural, ordinary `id` data, identity predicates/projections use the new key | DOC | PHASE-02/EX-5 ("consistently describe") | WEAKENED — no VT touches `src/learn.rs` (F3) |
| D§4-9 | guide must state that old stores are refused | DOC | PHASE-02/EX-5 ("version rules") | WEAKENED — and in tension with `learn`'s "never version-pinning" charter (F10) |
| D§4-10 | guide retains existing value boundaries, notably `DROP` | DOC | — | ABSENT (held only by unchanged prose pins) |
| D§4-11 | two-call recipe keeps completeness guard and backlink direction; only spelling moves | DOC | PHASE-02/EX-5; VA-2 (executed examples) | FAITHFUL (`tests/fixture.rs:556` executes the recipe) |
| D§4-12 | README: inspect staged+working changes, coordinate passages, no blanket replacement/overwrite/reset, no broad commit | CONSTRAINT | PHASE-02/EN-2, EN-3 | FAITHFUL |
| D§4-13 | fixture provenance explains the key adaptation, source-snapshot facts and privacy ruling unchanged | DOC | PHASE-02/EX-5 | FAITHFUL (reproduction recipe unexecuted, F18) |
| D§4-14 | amend decisions through the knowledge editing surface; never hand-edit record TOML | RECORD | PHASE-03/EX-1 | FAITHFUL |
| D§4-15 | each amendment dated, citing ADR-001 and SL-004 | RECORD | PHASE-03/EX-1 | FAITHFUL |
| D§4-16 | preserve durable ids and unrelated rationale | RECORD | PHASE-03/EX-1, VA-1 | FAITHFUL |
| D§4-17 | per-record amendment table (DEC-005 carve-out, 007, 010, 016, 002, 012) | RECORD | PHASE-03/EX-1, VA-1 | FAITHFUL |
| D§4-18 | ADR-001 and STD-001 need no amendment | RECORD | PHASE-03/EX-1 (implicitly) | FAITHFUL |
| D§4-19 | review prose and facet edits together; no contradictory facet/body | RECORD | PHASE-03/VA-1 | WEAKENED — review-only, no failure signal |

## §5 Verification and fixture preservation

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§5-1 | three distinct verification jobs; focused cases, not a mirror per branch | BEHAVIOUR | PHASE-02/VT-1..VT-6 (collectively) | FAITHFUL |
| D§5-2 | literal `ENTITY_ID == "smut_id"` assertion in entity tests | BEHAVIOUR | PHASE-02/VT-1 | FAITHFUL |
| D§5-3 | ordinary `id` survives: CLI round trips at root beside explicit/generated `smut_id`; nested string/empty/number/null/array/object; no unintended hoist | BEHAVIOUR | PHASE-02/EX-2; VT-2 | WEAKENED — the six nested value kinds are not enumerated |
| D§5-4 | structural identity still controls graph shape: refs, nested decls, forward refs, root generation, malformed refusal, collision avoidance | BEHAVIOUR | PHASE-02/VT-2; existing suites | FAITHFUL |
| D§5-5 | field ops distinguish identity and data: query both ways on mixed data, project identity alone and with `id`, preserve ordering + patch semantics | BEHAVIOUR | PHASE-02/VT-2 | FAITHFUL |
| D§5-6 | stored, projected and receipt bytes put `smut_id` first beside lexically earlier keys; embedded objects lexical | BEHAVIOUR | PHASE-02/VT-1 | FAITHFUL (paper-only) |
| D§5-7 | pin `changes[].smut_id`, entity-shaped `previous`, compact behaviour, JSON conventions declaration; containers keep names | BEHAVIOUR | PHASE-02/VT-2, VT-5 | WEAKENED — `previous`/`--compact` unnamed (F11) |
| D§5-8 | malformed stored identity refused: exit 1, basename + exact path, for `get` and traversal | BEHAVIOUR | PHASE-01/VT-1, VT-2 | FAITHFUL |
| D§5-9 | loader validates every loaded object: corruption under a field `follow` does not request still refuses | BEHAVIOUR | PHASE-01/EX-3; VT-1, VT-2 | FAITHFUL |
| D§5-10 | path notation unambiguous: dots, brackets, quotes, empty keys, escapes, array position | BEHAVIOUR | PHASE-01/VT-1 | FAITHFUL |
| D§5-11 | error classes: analogous request exit 2; ordinary `id` with those values is valid data | BEHAVIOUR | PHASE-01/EX-2; VT-1, VT-2 | FAITHFUL |
| D§5-12 | old index refused: build version-1 store explicitly; exit 1, found 1/expected 2, no success envelope | BEHAVIOUR | PHASE-02/VT-3 | FAITHFUL |
| D§5-13 | old committed journal not replayed: no staged file published, no target removed | BEHAVIOUR | PHASE-02/EX-7; VT-4, VT-6 | FAITHFUL |
| D§5-14 | new format recoverable: interruption/concurrency suites run against version 2 | BEHAVIOUR | PHASE-02/VT-4, VT-6; VA-2 | FAITHFUL |
| D§5-15 | capture entity/index bytes before the probe; show no repair; don't demand zero writes | BEHAVIOUR | PHASE-01/EX-3; VT-1 | FAITHFUL |
| D§5-16 | probes in scratch stores; repeat the IMP-005 shape; `get` and traversal fail with empty stdout and documented exit class | BEHAVIOUR | PHASE-01/EN-3, VT-2 | FAITHFUL |
| D§5-17 | fixture adaptation is spelling-only; preserve ids, targets, values, numeric distinctions, array orders, prose | BEHAVIOUR | PHASE-02/EX-4; VA-1 | FAITHFUL |
| D§5-18 | record the exact Git revision; read via Git; no second committed copy; no Git at runtime | CONSTRAINT | PHASE-02/EN-2, VA-1 | FAITHFUL |
| D§5-19 | inverse-map only identity keys; compare with structural JSON equality (`1` ≠ `1.0`) | BEHAVIOUR | PHASE-02/VA-1 | FAITHFUL |
| D§5-20 | reject changed target/value, missing entity, reordered array; exercise an altered copy as negative control | BEHAVIOUR | PHASE-02/EX-4, VA-1 | FAITHFUL |
| D§5-21 | keep baseline revision and command/result in slice evidence; no persistent mirror | RECORD | PHASE-02/VA-1, EX-4; PHASE-03/EX-3 | FAITHFUL |
| D§5-22 | re-run fixture, traversal and selection assertions and the executed guide examples | BEHAVIOUR | PHASE-02/VA-2; PHASE-03/EX-3 | FAITHFUL |
| D§5-23 | residual search over code/tests/fixture metadata/brief/README/learn/store-format/six DECs; classify by meaning; only missed structural use fails | BEHAVIOUR | PHASE-02/EX-6, VA-2; PHASE-03/EX-3 | FAITHFUL |
| D§5-24 | execute the README transcript and guide examples with their existing tests | BEHAVIOUR | PHASE-02/VA-2 | FAITHFUL |
| D§5-25 | inspect prose pins so they assert the new rule, not merely delete old sentences | BEHAVIOUR | — (only PHASE-02/VA-3 "learn as version-matched") | ABSENT (F3) |
| D§5-26 | review the brief as normative incl. unfinished 1.0; review store-format against code/workeds under STD-001; no grep substitute | RECORD | PHASE-02/VA-3; PHASE-03/VA-4 | FAITHFUL (unfalsifiable by design, see TASK 3) |
| D§5-27 | run `just check`: build, clippy ×2 at `-D warnings`, fmt, both test runs | BEHAVIOUR | PHASE-01/VA-1; PHASE-02/VA-2; PHASE-03/VA-3 | FAITHFUL |
| D§5-28 | inspect the complete diff for removed/disabled tests, weakened expectations, unrelated edits | BEHAVIOUR | PHASE-02/VA-2; PHASE-03/VA-3 | FAITHFUL |
| D§5-29 | scripted version-2 lifecycle: declare, query, render, delete, with both keys | BEHAVIOUR | PHASE-03/EX-2, VA-2 | FAITHFUL |
| D§5-30 | retain evidence of old-index refusal, no-replay, corruption refusal, fixture equivalence, executed examples, residual classification, six amendments | RECORD | PHASE-03/EX-2, EX-4 | FAITHFUL |
| D§5-31 | checks establish the documented Linux/recovery model; no new power-loss or portability guarantee | CONSTRAINT | PHASE-03/EX-4 (indirectly) | FAITHFUL |

## §6 Risks, assumptions and release boundaries

| D-ref | obligation | class | captured by | fidelity |
|---|---|---|---|---|
| D§6-1 | old-field caller changes meaning silently; rejection would contradict the feature — so document + behaviour cases + executed examples | ASSUMPTION | PHASE-02/EX-2, EX-5; VA-2 | WEAKENED — the *silence* leg (F1) is not carried |
| D§6-2 | old on-disk stores have an unambiguous version refusal boundary | ASSUMPTION | PHASE-02/VT-3 | FAITHFUL |
| D§6-3 | the pass adds work to every store-backed op; linear in parsed size; whole-store loading accepted; lazy/error redesign out of scope | ASSUMPTION | — | ABSENT |
| D§6-4 | diagnostic paths allocate only the path and error; never repeatedly serialise or clone subtrees | CONSTRAINT | — | ABSENT (F12) |
| D§6-5 | store still reserves `smut_id` at every depth; such a payload is interpreted structurally | CONSTRAINT | PHASE-02/EX-1 (indirect) | WEAKENED — no doc obligation assigned |
| D§6-6 | `DROP`, predicate syntax, missing-vs-null, opaque-id rules keep their boundaries | CONSTRAINT | — (unchanged suite) | WEAKENED |
| D§6-7 | no promise of literal storage for every payload | ASSUMPTION | — | FAITHFUL (as a limit, nothing to carry) |
| D§6-8 | fixture keeps historical prose/data while its keys adapt | RECORD | PHASE-02/EX-4, EX-5 | FAITHFUL |
| D§6-9 | README stays the operator's draft; nothing may discard or absorb its changes | CONSTRAINT | PHASE-02/EN-3; PHASE-03/EX-4 | FAITHFUL |
| D§6-10 | code without the doc/reconciliation obligations does not complete the slice | RECORD | PHASE-03/EX-1, EX-4; VA-4 | FAITHFUL |
| D§6-11 | no technical inquiry open; the added precision changes no identity policy | CONSTRAINT | — | FAITHFUL |
| D§6-12 | no migration, minting/rename verb, short aliases, Mermaid, chronology fields, filename redesign, extra convention sets, dependency changes, new public library API | CONSTRAINT | PHASE-03/EX-4 (partial) | WEAKENED — "no new public library API" and "no dependency" only at PHASE-03 |
| D§6-13 | OS/filesystem/recovery guarantees remain as documented | CONSTRAINT | PHASE-03/EX-4 | FAITHFUL |

## Verified code premises (commands and outputs)

| claim | command | result |
|---|---|---|
| `ENTITY_ID` is `"id"`, single definition | `rg -n "ENTITY_ID" src/` | 27 lines across 6 files (`entity.rs` 9, `declare.rs` 7, `select.rs` 4, `render_markdown.rs` 3, `store/mod.rs` 2, `journal.rs` 2) — 4 are `use` lines, so 23 read sites; DEC-020's "27 readers" counts the imports |
| `reference_id` distinguishes absence from invalid | `read src/entity.rs:31-47` | `Ok(None)` on no key; `Err(Reject::InvalidId)` when present and not a nonempty string ✓ |
| `scan_refs` non-fallible today | `read src/entity.rs:71-99` | returns `impl Iterator`, `panic!` on malformed nested identity ✓ |
| exactly two format-version constants | `rg -n "FORMAT_VERSION" src/store/` | `src/store/index.rs:28`, `src/store/journal.rs:35` — no third constant ✓ |
| loader checks root + index agreement only | `read src/store/mod.rs:201-243` | object check then `entity.get(ENTITY_ID)` vs index key; no nested walk ✓ |
| `IdDisagreement` message names `id` | `rg -n "holds id" src/` | `src/error.rs:389`: "the entity file {entry} holds id {found}, and the index maps it to {key}" |
| exit-2 identity message names `id` | `read src/error.rs:280-286` | "invalid entity id: {found}, and an id must be a nonempty string" |
| `describe()` names `id` | `read src/store/mod.rs:503-509` | `None => "no \`id\`"` |
| conventions seed hardcodes the key | `read src/conventions.rs:123-131` | `("id", id_of(set))` — **no `ENTITY_ID` reference anywhere in `src/conventions.rs`** |
| `order.rs` does not touch identity | `rg -n "ENTITY_ID" src/order.rs` | no matches (sorting is by `Id` values) |
| render suppresses the identity key | `read src/render_markdown.rs:184-213` | `is_selected` forces `ENTITY_ID` true; `attribute_lines` filters `*field != ENTITY_ID` |
| render mechanism is keyed on the constant | `smut declare '{"id":"a","smut_id":"x","body":"hello"}'` then `smut render '{"from":["a"]}' --show-fields` in scratch | attributes show `- **smut\_id:** x` while `id` is suppressed → swapping the constant makes a data `id` render |
| a stale field predicate is silent | `smut query '{"from":["a"],"where":{"absent":"x"}}'` | `{"objects":[],…}`, **exit 0** |
| sweep size | `rg -o '"id"' src tests docs README.md \| wc -l`; `rg -c … \| wc -l` | 974 occurrences in 27 files ✓ (design's number); fixture 223 ✓; brief has 6 more, outside the design's count |
| fixture entity count | `rg -c '^    "id": ' tests/fixtures/interview/entities.json` | 66 ✓ |
| six amended decisions exist and carry the old spelling | `doctrine show DEC-005/007/010/016/002/012` | all six quoted by the design are present in substance ✓ |
| `just check` composition matches the design | `read justfile` | build → clippy (`--all-targets`, and `--all-features`, both `-D warnings`) → `cargo fmt --check` → `cargo test` twice (default, `--features fault-injection`) ✓ |

---

# (b) Ranked findings

## BLOCKER

### F1 — The slice's own named failure mode (silent `where` on `id`) is decided by nobody
**Design:** §2 "Reading, projection and canonical bytes" decides only the *positive* case
(`id` "may legitimately match a predicate"); §6-1 says "documentation, new behavior cases and
executed examples make the new contract explicit"; `slice-004.md` Risks explicitly hands the
question to the design ("the design should decide whether that is a test or just a documented
sharp edge"); `notes.md`/`research.md` cross-finding 5 names "silence is the failure mode to
watch". **The locked design never answers it** — no section states the negative consequence,
and §4's documentation contract lists three teaching sentences, none of which is this one.
**Plan:** `rg -i "where" .doctrine/slice/004/plan.toml` → no criteria mention `where`; the
only related text is PHASE-02/EX-2 ("`id` behaves as an ordinary queried field").
**Evidence:** probe on the current binary —
`smut query '{"from":["a"],"where":{"absent":"x"}}'` → `{"objects":[],…}` at **exit 0**;
`src/predicate.rs:219` documents "Missing is distinct from present-`null`: only `$exists` can
hold of a missing field". The same silence applies to the post-rename stale `where:{"id":…}`.
**Why it matters:** after this slice, a caller still sending `id` gets an empty result at exit
0 from every operator but `$exists` — the exact quiet restructure the decision exists to end.
The design's §5 "three verification jobs" omits it, so the slice can close green with the
failure mode live and untested.
**Minimal repair (append-only):** PHASE-02 `EX-8` — "A `where` on ordinary `id` selects only
entities that carry `id` as data; a caller left on the old spelling gets an empty result at
exit 0, and the guide states that consequence in one sentence" + PHASE-02 `VT-7`
(`tests/cli_contract.rs`, keywords `where`, `ordinary_id`, `empty_result`, `smut_id`). Because
the design is locked, this needs the operator's assent to add D§2-26 (see (c)).

### F2 — Diagnostics that name `id` as the identity key are undecided and carried by nothing
**Design:** §3 decides exactly one message — the *new* corruption reason ("a proposed exact
reason is: `identity at … must be a nonempty string`") — and §3-18 says "keep the internal
`scan_refs` assertion … update its spelling". Nothing decides the three existing
identity-naming diagnostics, and §3-5 says only that root/index-disagreement *precedence* is
retained.
**Evidence:** `src/error.rs:389` "the entity file {entry} holds id {found}, and the index maps
it to {key}"; `src/error.rs:282` "invalid entity id: {found}, and an id must be a nonempty
string"; `src/store/mod.rs:504` `"no \`id\`"`; `src/entity.rs:90` panic "…an `id` that is not a
nonempty string is corruption". `docs/store-format.md:252` quotes the first verbatim, and
STD-001 makes worked diagnostics part of the format contract.
**Plan:** PHASE-01/EX-2 pins only the new reason; PHASE-02/VA-3 says doc diagnostic examples
must "agree with code" — which is satisfied by leaving both stale. No criterion names any of
these strings.
**Why it matters:** a locked design whose thesis is "`smut_id` expresses identity everywhere"
would ship a corruption message that names a field which is no longer identity, and a format
document that quotes it. That is a misleading diagnostic in the one surface STD-001 calls a
caller-quoted contract — and the plan's own review leg actively certifies it.
**Minimal repair:** PHASE-02 `EX-8` (or `EX-9` if F1 takes EX-8): "Every identity-naming
diagnostic (`InvalidId`, `IdDisagreement`, `describe`, the `scan_refs` assertion) names
`smut_id`; `docs/store-format.md` quotes the updated strings verbatim" + PHASE-02 `VT-7`
(`tests/cli_contract.rs` or `tests/store_corruption.rs`, keywords `holds smut_id`,
`invalid entity identity`). Design-level: needs the operator to add D§3-28 (see (c)).

## MAJOR

### F3 — `learn`'s prose pins are the design's only mechanical guard, and no criterion touches them
**Design:** §5-25 "Inspect prose pins so they assert the new rule rather than merely deleting
old sentences"; §4-8/4-9/4-10 add three teaching obligations; DEC-020 names "prose pins in
`src/learn.rs` tests" as what holds the guide, and DEC-020's consequence "smut learn can never
interpolate the key, so its pins stay prose".
**Evidence:** the plan's VT test files are `src/entity.rs`, `tests/cli_contract.rs`,
`tests/store_corruption.rs`, `tests/durability_crash.rs`, `tests/conventions.rs`
(`rg -n test_file .doctrine/slice/004/plan.toml`) — **`src/learn.rs` appears in no VT**. The
prose pins are real and load-bearing: `src/learn.rs:270-330` (`guide()` flattening, the
"covers the 0.1 surface" name list, `KEYS`); `src/learn.rs:65-86` carries both sentences this
slice reverses ("An entity is a JSON object with a nonempty string \"id\""; "an embedded object
cannot carry \"id\" as inline metadata").
**Why it matters:** PHASE-02/EX-5's "learn … consistently describe the new identity/version
rules" is an outcome statement with no test home; the only checks are PHASE-02/VA-3 ("learn as
version-matched") and PHASE-03/VA-4 — both judgement-only. A deleted sentence passes EX-5.
**Minimal repair:** PHASE-02 `VT-7` — `test_file = "src/learn.rs"`, keywords
`["smut_id", "nonempty string", "ordinary id"]` — and PHASE-02 `EX-8` requiring the three
teaching sentences to be present as *pins*, not prose.

### F4 — Render output changes silently: an ordinary `id` becomes visible in Markdown
**Design:** §4's `src/render_markdown.rs` row says only "Inspect identity/reference examples and
pins; retain rendering behavior and identity display as a value"; §1-4 mentions "rendered
examples". No section decides what a data field named `id` does in rendered output.
**Evidence:** `src/render_markdown.rs:184-190` forces `ENTITY_ID` selected; `:211` filters
`*field != ENTITY_ID` out of `attribute_lines`. Probe: declaring
`{"id":"a","smut_id":"x","body":"hello"}` and rendering with `--show-fields` prints
`- **smut_id:** x` — i.e. the suppression set is literally the constant, so after the swap a
payload `id` field starts rendering and `smut_id` stops. No criterion covers
`tests/render.rs`.
**Why it matters:** it is a user-visible output change in a shipped surface, produced by an
obligation the design leaves as "inspect", and nothing would fail if it were left wrong or
untested.
**Minimal repair:** append PHASE-02 `EX-8` sentence "a data field named `id` renders as an
ordinary attribute and `smut_id` remains suppressed" + `VT-7` on `tests/render.rs` (keywords
`ordinary_id`, `show_fields`, `smut_id`). Design-level precision needed (see (c)).

### F5 — Exact diagnostic text that STD-001 makes contractual is softened to "elements"
**Design:** §3 pins a verbatim reason and the verbatim index refusal
(`smut: the store index has format_version 1, and this build reads only 2`).
**Plan:** PHASE-01/EX-2 requires only "naming the entity basename and JSON-quoted bracket
path"; PHASE-02/VT-3 requires only that refusal "names found/expected versions and exits 1".
**Evidence:** `docs/store-format.md:249` currently reads "format_version 2, and this build
reads only 1" — the example must invert, and `:252` quotes the IdDisagreement string. The
word "only" comes from `src/error.rs:373`; nothing in the plan requires it to survive.
**Why it matters:** STD-001 declares worked diagnostics "part of the contract and … quoted by
callers"; VAs can only check agreement if a criterion fixes the text, otherwise PHASE-02/VA-3
drifts toward whatever the code happened to emit.
**Minimal repair:** PHASE-02 `EX-9` (append) pinning both strings verbatim, referenced from
`docs/store-format.md`; no renumbering.

### F6 — Two VTs are source-presence mandates over the very code they claim to verify
See TASK 3 below (`PHASE-01/VT-3`, `PHASE-02/VT-1`). The design's §5-1 asks for behavioural
cases; the plan's only mandate for the loader's *shape* and for canonical ordering is a grep,
and PHASE-03/VA-3 itself concedes "a source-presence mandate passing is not evidence that its
behavioral tests ran". Repair: keep the mandates but add PHASE-01 `VA-3` requiring the inline
`mod tests` in `src/store/mod.rs` to exercise the walk, and PHASE-02 `VT-7` on
`tests/roundtrip.rs` for canonical ordering beside a lexically earlier ordinary key.

### F7 — `DROP`-as-data and the "no literal-assignment syntax" boundary have no criterion
**Design:** §2-25 "Ordinary `id` follows the existing field patch rules, including the
top-level `"DROP"` sentinel"; §1-3 and §6-6 keep the sentinel boundary.
**Evidence:** `rg -n "DROP" .doctrine/slice/004/plan.toml` → no matches. After the rename,
`{"smut_id":"x","id":"DROP"}` becomes a legal request whose semantics (drop the `id` field, not
store the string) are decided by the design but unverified by the plan; existing `DROP` tests
never combine it with a data field named `id`.
**Minimal repair:** PHASE-02 `EX-8` clause + a `VT-7` case in `tests/cli_contract.rs` (keywords
`DROP`, `ordinary_id`).

### F8 — Design §2 asserts a code fact the code contradicts (conventions seed)
**Design:** §2 "`src/conventions.rs` uses it for the fixed-id declaration returned as JSON"
(present tense, listed under "One definition and unchanged interfaces", whose point is that
those consumers *already* read the constant).
**Evidence:** `rg -n "ENTITY_ID" src/conventions.rs` → no matches;
`src/conventions.rs:126` hardcodes `("id", id_of(...))`. The claim is only true as the *future*
requirement in §4's table.
**Why it matters:** an implementer reading §2 concludes the seed already follows the constant
and skips it; PHASE-02/EX-1 ("the conventions JSON seed use it") saves the outcome, so the
plan is faithful and only the design's sentence is wrong — but the sentence is what an audit
reads.
**Minimal repair:** no plan change needed; record as design erratum (see (c)). PHASE-02/VT-5
already pins the seed.

### F9 — Section §4/§6 constraints that govern PHASE-01's own work are absent from PHASE-01's entrance criteria
**Evidence:** PHASE-01/EN-1 reads "design sections 1-3 and 5" — §4 (code-impact contract,
"no new public error variant", "update format-bearing tests") and §6 (linear-work and
allocation discipline) are omitted, yet PHASE-01 is the phase that edits `src/store/mod.rs`.
**Minimal repair:** PHASE-01 `EN-4` — "Design sections 4 and 6 are read; §6's linear-work and
path-allocation constraints bind this phase" (append-only).

### F10 — `learn` must state old stores are refused, while `learn` is chartered never to pin a version
**Design:** §4-9 requires the guide to "State that old stores are refused"; `src/learn.rs:3-13`
declares the guide "Descriptive and version-scoped, never version-pinning" and says "A version
figure would be PHASE-07's to change". A sentence about a version break is exactly the kind of
release-figure statement the module doc forbids, and the design does not say how to phrase it
(a versionless "a store written by an older build is refused" would satisfy both).
**Why it matters:** the two obligations can collide at implementation time and the design
picks neither, leaving an implementer to choose between a charter violation and dropping a
design obligation.
**Minimal repair:** PHASE-02 `EX-8` phrasing clause: the guide states the refusal *without* a
release figure. Design-level question for the operator (see (c)).

### F11 — `previous`/`--compact` receipt pins are weakened to "renamed change receipt"
**Design:** §5-7 explicitly demands "Pin `changes[].smut_id`, entity-shaped `previous`, compact
behavior and the JSON conventions declaration"; §2-22 repeats the `previous` rule.
**Plan:** PHASE-02/VT-2 expects only "the renamed change receipt while retaining container
vocabulary". Existing `tests/cli_contract.rs:396-416` (`compact_omits_previous_everywhere…`)
will force the change, so the risk is procedural, not silent.
**Minimal repair:** extend PHASE-02/VT-2's `expects` text (criteria text may be amended in
place; ids immutable) or append PHASE-02/`VT-7` clause naming `previous` and `--compact`.

## MINOR

- **F12** D§6-4 (path allocation, no repeated subtree clone/serialize) and D§3-9/§6-3
  (linear, no re-read/re-parse) have no criterion at all — implementer discretion on a
  constraint the design states twice. Repair: PHASE-01 `VA-3` review line naming them.
- **F13** D§3-6/D§3-8: VT-1 covers "under objects/arrays" but not "the walk does not stop at a
  reference-shaped object" nor "`smut_id` nested inside an ordinary `id`-valued object is still
  found". Repair: add both cases to PHASE-01/VT-1's expects.
- **F14** D§3-11: the `$` root marker appears nowhere in the plan (EX-2 says "JSON-quoted
  bracket path"). Repair: name `$` in PHASE-01/EX-2's text.
- **F15** D§1-5/D§2-23: `frontier`, `depths`, `edges`, `dangling` are carried only by the
  generic "container vocabulary" phrase; held in practice by unchanged `tests/traversal.rs`.
  Repair: list them in PHASE-02/VT-2's expects.
- **F16** D§2-3 (no identity module or alias) and D§6-12's "no new public library API / no new
  dependency" have no mid-slice criterion (PHASE-03/EX-4 covers scope only at the end).
  Repair: add to PHASE-02/EX-1 or a new EX.
- **F17** D§2-20 byte forms (pretty file / compact stdout / trailing newline) and D§2-9
  (opaque, case-sensitive, whitespace-only ids) rest on the unchanged suite with no criterion;
  acceptable, but should be named in PHASE-02/VA-2 so the reviewer knows they were considered.
- **F18** `tests/fixtures/interview/PROVENANCE.md:75-90` carries an id-sorting rule and an
  unexecuted `sort_by(.id)` reproduction command; PHASE-02/EX-5's "fixture provenance …
  consistently describe" is the only guard and nothing executes that recipe. Repair: name
  `PROVENANCE.md` explicitly in EX-5.
- **F19** Design's Affected-surface table lists `src/order.rs` as taking "identity key in
  selection, edges, sort tiebreak"; `rg -n "ENTITY_ID" src/order.rs` → no matches (sorting is
  by `Id` values). Harmless, but the row invites a pointless edit; §4's own row is correctly
  conservative ("keep algorithms and internal ids-as-values unchanged").
- **F20** `plan.toml` order: PHASE-02's `verification` array is VT-1..VT-5, VA-1..VA-3, VT-6 —
  the late-append is legitimate under the immutable-id rule but reads as a stray; a trailing
  comment marking VT-6 as the critical-pass append would prevent a future renumber attempt.
- **F21** Design §3's "A proposed exact reason is:" uses "proposed", which is what lets the
  plan soften F5; if the text is contractual the design should say so (it does for the index
  message, not the reason).

---

# (c) Design-level defects needing operator assent (design is locked)

These cannot be repaired by plan append alone — the design says nothing, so a phase criterion
would be inventing policy:

1. **D§2-26 (new): the stale-`where` consequence.** Decide explicitly whether "a `where` on
   `id` returns empty at exit 0" is (a) a test, (b) a documented sharp edge in `learn` +
   README, or (c) both — and say so in §2 or §5. Today the design is silent on the shape the
   slice body explicitly asked it to decide. *(Feeds F1.)*
2. **D§3-28 (new): identity-naming diagnostic strings.** Decide whether `InvalidId`,
   `IdDisagreement`, `describe` and the `scan_refs` assertion name `smut_id`, and whether
   `docs/store-format.md`'s quoted worked diagnostic changes with them. *(Feeds F2, F5.)*
3. **D§4-20 (new): render output for a data field named `id`.** Decide that a data `id`
   renders as an ordinary attribute and `smut_id` stays suppressed (or that rendering is
   unchanged in some other defined way). *(Feeds F4.)*
4. **D§4-9 phrasing: `learn` and version figures.** Reconcile "the guide must state that old
   stores are refused" with `src/learn.rs`'s "never version-pinning" charter by deciding the
   versionless phrasing. *(Feeds F10.)*
5. **D§6-5 documentation duty.** The design states the `smut_id` reservation survives at every
   depth but assigns no documentation obligation; decide whether the README sharp edge and the
   guide must carry it (ADR-001 states it as a boundary, so a reader of the shipped surface
   otherwise learns the "reserved key" limit only from the ADR).
6. **D§2 erratum: `conventions.rs`.** §2's present-tense claim that `src/conventions.rs` "uses
   it" is false (`src/conventions.rs:126` hardcodes `"id"`). Reword to a requirement (the plan
   already requires it) or accept the sentence as inaccurate. *(Feeds F8.)*
7. **D§3-18 scope.** "Update the `scan_refs` assertion's spelling" is the only diagnostic the
   design rules on; it should either stay alone (and item 2 rejected) or be generalised. This
   is the same decision as item 2 and is listed so the operator sees the asymmetry.
8. **Design target registry.** The Affected-surface table's `src/order.rs` row (F19) is a
   factual overstatement; correcting it touches the locked design-target selectors, hence
   assent.

---

# TASK 3 — paper-only verification, in detail

## VTs whose keyword mandate can pass while the behaviour is broken

| criterion | test_file | why it can pass vacuously |
|---|---|---|
| PHASE-01/VT-3 | `src/store/mod.rs` | The keywords `validate_identity_tree` and `reference_id` are satisfied by the helper's *signature* and a `use` line. It also claims "no independent identity rule or traversal `Result` plumbing is introduced" and "calls the shared identity predicate" — statements no keyword can falsify. A helper that exists, is named, and validates nothing (or stops at a reference-shaped object, violating D§3-6) passes. The real behavioural cover is PHASE-01/VT-1, so VT-3 adds only a shape assertion. |
| PHASE-02/VT-1 | `src/entity.rs` | The keyword `smut_id` is satisfied by the constant's value, and `ENTITY_ID`/`wire_contract` by the assertion itself, so the mandate is green the moment the constant flips. The same VT also claims "structural canonical ordering, including `smut_id` beside lexically earlier ordinary keys" — `src/entity.rs:310` shows the ordering lives there, but a keyword grep cannot distinguish a real ordering assertion from a comment or an unrelated test. Canonical ordering needs a case in `tests/roundtrip.rs`/`tests/selection.rs` where a lexically earlier ordinary key (e.g. `aardvark`) coexists with `smut_id`. |
| PHASE-02/VT-5 (partial) | `tests/conventions.rs` | `smut_id` and `conventions-hygiene` are satisfied by a mention; the seed could still be emitted from a literal `"smut_id"` rather than the constant while the test passes (which is exactly what F8 is about, inverted). |

`doctrine slice verify-vt 4` (PHASE-03/VA-3) checks none of this — it checks that the mandated
file exists and holds the keywords. PHASE-03/VA-3's own text concedes the gap; the plan should
therefore pair each paper mandate with an explicit behavioural criterion, which is what F3/F5/F6
repair.

## VAs that cannot fail

| criterion | why unfalsifiable |
|---|---|
| PHASE-01/VA-1 | "retain actual test results separately from verify-vt's source-presence mandates" — a process instruction; no output can violate it. |
| PHASE-02/VA-2 (classification leg) | "classify residual structural id uses" — any list, including one that misses a stale `where:{"id":…}`, satisfies it. No negative control is named (contrast PHASE-02/VA-1, which correctly demands one for fixture equivalence). |
| PHASE-02/VA-3 | "Review `docs/store-format.md` … agree with code" — judgement only; and because F5 leaves no pinned text, "agreeing with code" is satisfiable by leaving a stale diagnostic in both. STD-001 concedes no mechanism exists, which makes the *criterion wording* the only lever, and it is currently unfalsifiable. |
| PHASE-03/VA-1 | "compare each proposed facet/prose amendment against [the record]" — no failure signal; an amendment that quietly drops unrelated rationale still "compares". |
| PHASE-03/VA-4 | "confirm … unfinished 1.0 scope is retained" — no observable; nothing distinguishes a retained paragraph from a reworded one. |

Falsifiable VAs for contrast (good practice to mirror): PHASE-01/VA-2 (versions/constant have not
moved — checkable by grep/read), PHASE-02/VA-1 (negative control on an altered fixture copy),
PHASE-03/VA-2 (exit classes and concrete output), PHASE-03/VA-3's `just check` leg.

---

# TASK 4 — attack on the design itself (beyond the plan-capture findings)

**Internal contradictions / tensions**

- **A1** §2 (present tense: conventions "uses it") vs `src/conventions.rs:126` (hardcodes `"id"`),
  and §2's consumer list omits the conventions seed while §4's table requires it. *(F8.)*
- **A2** §4-9 "the guide must state old stores are refused" vs `src/learn.rs:3-13` "never
  version-pinning … a version figure would be a later phase's to change". No phrasing decided.
  *(F10.)*
- **A3** §6-5 states the `smut_id` reservation survives at every depth; §1 says the change
  "closes" ISS-002. Both are true (the *caller's* `id` is freed; the reserved spelling merely
  moves), but the design never says which of the two a reader of the shipped surface should be
  told, and assigns no documentation duty to the residue — so a payload with a nested
  `smut_id` still hoists silently, which is ISS-002's failure mode re-spelled. That is the
  single most under-communicated consequence in the design.
- **A4** §5-1 "add focused cases rather than a test mirroring each implementation branch" vs
  PHASE-01/VT-3 mandating the helper's internals; the design's own instruction argues against
  the plan's paper mandate.
- **A5** §3-6 (walk does not stop at a reference-shaped object) and DEC-005/`scan_refs` (which
  *does* stop) are not in conflict, but the design never says why the loader is deliberately
  stricter than the scanner. It is a defensible choice that will read as an inconsistency at
  audit; one clause would close it.

**Obligations not derivable from ADR-001 + the DECs**

- **A6** The path notation (`$["payload"]["tags"][2]["smut_id"]`, JSON-escaped key components)
  is entirely new policy invented in the design. It is not in ADR-001, DEC-021 (which says only
  "an unambiguous field/index path") or DEC-025. The design is the right home, but it should
  carry its rationale explicitly — nothing in the ledger justifies the bracket form over the
  dotted form, and DEC-021's own example used the dotted `payload.tags[2].smut_id`
  (`notes.md`, inq-2 "Failure shape"). The design silently supersedes the recorded answer's
  notation without saying so.
- **A7** "Both `FORMAT_VERSION` **and** `MANIFEST_FORMAT_VERSION` move to 2" is a design
  choice DEC-022 sanctions ("the corresponding version check to committed journal
  manifests") but does not require to be the same number; pinning them equal is the design's
  own conservative reading. Fine, but worth a sentence, because it means a v1 manifest and a
  v1 index are refused by the same number and a future independent manifest bump loses that.

**Design decisions the code's reality contradicts**

- **A8** §4's `src/order.rs` row (F19): no `ENTITY_ID` reference exists in `src/order.rs`;
  ordering is over `Id` values.
- **A9** §4-1's `src/render_markdown.rs` "retain rendering behavior" is not achievable as
  stated: the behaviour *is* key-dependent (`:186`, `:211`), so "retain" and "the key moves"
  cannot both hold for a data field named `id`. This is the design contradicting itself, not
  just leaving a gap. *(F4.)*
- **A10** §4's `src/cli.rs` row ("nothing beyond the key: the grammar does not change") is
  accurate for grammar, but `src/cli.rs` is also where exit classes are assembled for
  diagnostics; if F2's diagnostic strings move, `src/cli.rs` test expectations
  (`src/cli.rs:531-571` use `{"id":"a"}` literals) move too, which the row's "retain grammar,
  flags and generation policy" does not mention.

**Boundaries claimed but enforced by no criterion**

- "no identity module or alias" (D§2-3), "no filesystem dependency in `src/entity.rs`"
  (D§3-4), "diagnostic notation only, not query syntax" (D§3-13), "linear, no re-read"
  (D§3-9), "path allocation discipline" (D§6-4), "no new public library API" (D§6-12).
  Each is a cheap review line or a `rg` check; none exists.

**Left to implementer discretion that a locked design should pin**

- The exact reason/refusal strings (F5), the question of whether `id` in rendered Markdown is
  data or silence (F4), whether `learn` may name a version (F10), whether the answer to the
  stale-`where` caller is a test or a sentence (F1), and whether `"DROP"` combined with a data
  `id` is stored or dropped (F7).
- Also: what `smut learn` should say about the *reserved* `smut_id` at depth (A3) — today the
  guide teaches the opposite limit for `id` and the design gives no replacement sentence.

**Hunt-list items checked and found adequately closed** (so the audit does not re-litigate
them): journal/manifest replay (D§3-23..26 → EX-3/EX-7/VT-4/VT-6, plus a readable-manifest
control the design did not demand — STRENGTHENED, benign); the conventions seed (D§2-7 →
EX-1/VT-5); the `ids`/`deleted`/`missing`/`frontier` container vocabulary (D§1-5 → EX-1/VT-2,
weakly); the sort tiebreak (no key involvement — verified); the refused-version message *shape*
(preserved by construction, `src/error.rs:373`); the fixture equivalence protocol
(D§5-17..21 → EX-4/VA-1, including the design's own negative-control requirement); the six
decision amendments (D§4-14..19 → EX-1/VA-1, with DEC-005's substantive reversal explicitly
required).

---

## Residual risks of this review

- No implementation was run and no test suite was executed; every behavioural claim is either
  read from source or probed with the **current** binary in a scratch store, i.e. the analogue
  of the post-change case rather than the case itself.
- Two probes needed the constant flipped to be exact (render visibility, stale-`where` silence);
  the probe shows the *mechanism* is keyed on `ENTITY_ID`, which is a sound but indirect
  argument.
- The `interview.local/.smut` live store was not read; after the bump it will be refused until
  re-declared, which the design intends (Non-goals) but which no phase verifies.
- Criterion-id proposals are append-only (`PHASE-01/EN-4`, `PHASE-01/VA-3`, `PHASE-02/EX-8`,
  `EX-9`, `VT-7`, `PHASE-03/*` unchanged); none renumbers an existing id.
