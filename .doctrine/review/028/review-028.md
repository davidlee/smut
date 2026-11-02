# Review RV-028 — reconciliation of SL-004

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-004 as delivered at `64e5f4b` (three phases completed, lifecycle
`started`), read against its locked design (rev 47), `plan.toml`, ADR-001, DEC-020
… DEC-028, STD-001 and STD-002. Self-audit: the same session drives raiser and
responder.

**Surface audited.** The primary tree in `/workspace/smut`, at the committed
`64e5f4b` tree. The working tree also carries the operator's in-flight README
rewrite and its harness edit (`README.md`, `tests/readme_example.rs`); no phase
staged them and they are not this slice's write scope (CHR-001). No source, test
or document file under the slice's design targets differs from `64e5f4b`.

**Lines of attack.**

1. **The identity contract actually shipped** (ADR-001, design §1–§2, scope
   objectives 1–2): `smut_id` structural at every depth; ordinary `id` data at
   every depth, including `null`, empty, number, array and object; references and
   nested declarations; `changes[].smut_id` and `changes[].previous`; the
   conventions JSON seed; `smut_id`-first canonical bytes; `select: []`; a
   request-level malformed identity still exit 2. Probe the binary, not the tests.
2. **The loader refuses what it cannot trust** (IMP-005, DEC-021, design §3,
   STD-002): the precondition `scan_refs` assumes is established eagerly at load;
   a hand-edited nested `smut_id` is corruption at exit 1 naming the entity file
   and the bracketed path, for `get`, traversal, `render` and the write path, with
   no panic and no repair; root checks keep their precedence.
3. **The format break is loud and total** (scope objective 3, DEC-022): both
   versioned documents at 2; a version-1 index and a version-1 committed manifest
   refused by version before replay; no migration, compatibility reader or repair
   advice anywhere it would appear.
4. **The documents are true of the binary** (STD-001, scope objective 6): the
   format record's version, key, canonical bytes, corruption contract, path
   notation and worked diagnostics; the guide's taught rules and its executed
   examples; the normative brief's identity/version state with its 1.0 scope
   retained. And the fixture is spelling-only — DEC-024's inverse check.
5. **The conformance registry is an honest record.** Every undeclared and
   undelivered cell dispositioned; no retro-fitting of the locked design.
6. **The reconciliation handoff is complete** (DEC-025, scope objective 5): every
   stale identity or version passage in the six accepted decision records is named
   with its exact change, retained rationale identified, ids preserved; and no
   *other* binding record (ADR-001, STD-001, STD-002, plan, slice scope) is left
   stating the reversed rule.
7. **Boundaries held:** README's surrounding prose untouched by the slice; no
   minting/rename verb, short flag, Mermaid, dependency or API change; no test
   removed or weakened.

**Not this review's question:** whether the operator's 0.2 README rewrite is
finished (CHR-001 owns it), and whether the slice should have included ISS-003 or
IDE-002 (both excluded by direction).

## Synthesis

SL-004 is implemented as designed. The identity break is real, loud and complete at
the boundary the operator asked for, and the loader now refuses what it used to
panic on. Eight findings were raised; none is a blocker, five are wording or
registry defects, one is a governance record the break made stale, and one is work
that is already owned elsewhere.

### What was independently re-established (not read off the handback)

- **Gate.** `doctrine check gate` exit 0 on the delivered tree — build, clippy in
  both feature configurations with warnings denied, `cargo fmt --check`, default and
  fault-injection suites — 370 lib / 383 fault-injection passing, zero warnings.
  Re-run after the audit's own one-word repair (F-5) and still green.
- **`verify-vt 4`.** PHASE-01 VT-1..3 and PHASE-02 VT-1..8 all PASS. Treated as
  source-presence evidence only; the probes below are the behavioural evidence.
- **The identity contract, probed against the built binary.** The design §1 example
  (`{"smut_id":"note","external":{"id":7,…},"related":{"smut_id":"other"}}`) makes
  one entity, keeps `external.id` as data, and reports `other` missing. A nested
  `id` of number, `null`, object and array survives storage and is served at exit 0
  and renders as an ordinary attribute. `changes[].smut_id` present, `previous`
  entity-shaped. The conventions JSON seed names `smut_id` first. `select: []`
  yields `smut_id` only. A request-level malformed identity (`7`, `null`, `""`,
  and a nested `false`) is exit 2. A `where` on ordinary `id` discriminates only
  entities carrying `id` as data, and the old spelling returns an empty result at
  exit 0 — the consequence DEC-027 decided to teach rather than refuse.
- **IMP-005, re-run in its original shape.** Hand-editing a nested `smut_id` to
  `null` makes `get`, `follow`, `render`, a plain query, the write path and an
  unrelated `get` all refuse: exit 1, the entity file named, the path
  `$["payload"]["smut_id"]`. No panic, and the entity bytes are unchanged after
  the refusals — refused, not repaired.
- **The version break.** A version-1 index exits 1 with
  "format_version 1, and this build reads only 2"; version 3 the same with 3; no
  migration advice. A fresh directory still exits 1 with "no store" and creates
  nothing; an index-less `.smut/` holding entity files is refused rather than
  initialised over.
- **Fixture preservation (DEC-024).** `sed 's/"smut_id":/"id":/g'` over the
  committed fixture reproduces blob `f5b29de6…` **byte for byte** (75 147 bytes
  each, 223 keys), and a `/jq -S/` structural comparison after inverting only the
  identity key is equal. The rename changed no id, value or array order.
- **Conformance.** 31 selectors conformant. Undeclared: `src/predicate.rs` (F-1)
  plus the operator's and SL-006's `.doctrine` work that landed inside the recorded
  phase range and this slice's own slice artefacts. Undelivered: exactly the six
  decision records, which no phase may deliver — they are the reconcile write
  surface, and this review's brief carries them.
- **Process.** Zero `#[test]` lines removed; the two removed `fn` lines are
  renames that keep their original claim. `ENTITY_ID` is the single definition of
  the wire spelling; production code that constructs or reads the envelope goes
  through it (the only raw spellings are test helpers in `#[cfg(test)]` modules
  and the guide's own statements of the spelling). No migration verb, minting
  verb, short flag, Mermaid, dependency or public API appears in the diff.

### The standing risks, ranked

1. **The six decisions are still unamended.** Scope objective 5 is not delivered
   until `/reconcile` writes them. F-2 and F-3 are the audit's contribution: the
   proposed mapping is precise but DEC-007's title, slug and first rejected
   alternative are outside it, and DEC-010's field quotation is a substring.
   Following the proposal as written would land six amendments and leave one
   record still naming the reversed canonical-order rule.
2. **A governance record states the reversed rule** (F-4). STD-001's own trigger
   table says the canonical bytes are `id`-first. It is a one-parenthetical fix,
   but it sits in the boot context of every future agent, so it is the highest
   leverage item in the brief.
3. **The format record states one mechanism that is not shipped** (F-6). The
   negative rule is right and load-bearing; the positive location is DEC-026/028's
   decision, not the loader's behaviour (IMP-013 tracks it).
4. **The README's committed prose contradicts the shipped binary** (F-7). The
   operator's in-flight rewrite already carries the corrected rule; the work is
   landing it, and CHR-001 owns it. The slice correctly refused to touch it.
5. **`src/predicate.rs` is undeclared** (F-1). Not a code defect — a registry and
   design-table gap, so a later conformance read cannot distinguish it from scope
   creep.

### Tradeoffs consciously accepted

- **Recovery publishes before the loader judges** (RV-024 F-2, tolerated). This
  slice discharges STD-002 by example — both versions move, an old manifest is
  refused before a byte replays — rather than by adding pre-replay payload
  parsing. The residual leg is a writer bug or a hand edit inside
  `journal.commit/` during the commit window.
- **`smut_id` stays reserved at every depth** (ADR-001's boundary, design §1/§6).
  A payload that uses that spelling cannot be stored verbatim, and the guide now
  says so. `id` is freed; not every identity-shaped field is.
- **Old stores are refused, with no migration** (DEC-022). The operator's
  direction; nothing here is a compatibility gap left by oversight.
- **The README's surrounding prose landed outside the slice** (CHR-001). The
  executed fence moved with the binary; the prose did not, by direction.

### Not this audit's work

ISS-002 and IMP-005 are the slice's originating items and resolve at `/close`;
ISS-003 (pre-minting) and IDE-002 (rename with referential stability) stay open by
direction. RV-024 retains six major, three minor and one nit findings that its own
pass left open, and RV-025 retains eight minor and two nit findings answered but
not verified; neither carries an unresolved blocker, and neither is a condition of
this audit — but both read `active`, so `/close` will want them terminal.

## Reconciliation Brief

Source of record for the six decision amendments:
`.doctrine/slice/004/decision-amendments.md` (per-record facet and prose mapping,
retained rationale, the dated ADR-001/SL-004 amendment note). This brief adopts it
with the three corrections below and adds the non-decision items.

Mechanism: facet fields and prose move through the knowledge editing surface
(`doctrine knowledge edit <ID>`, `doctrine knowledge edit decision <ID>`), never by
hand-editing `record-0NN.toml`. Each record's facet and prose land in one act, and
every amended record carries the common note ("Amended 2026-11-02 by SL-004 under
ADR-001 …") with DEC-005's withdrawal addendum and DEC-010's version-figure
addendum.

### Per-slice (direct edit)

- **[F-1] Selector registry — load-bearing, do this first.**
  `doctrine slice selector add 4 src/predicate.rs --intent design-target` with the
  note "SL-004 audit: field predicates only; the inline fixtures that name
  structural identity were respelled with the key. No production identity logic."
  A prose-only change leaves `slice conformance` red; the registry is what it
  reads.
- **[F-1] `design.md` §4, mirror of the row above.** Add to the code-impact table:
  "`src/predicate.rs` | Field predicates only — no identity read. Update the inline
  fixtures that name structural identity; keep the operator grammar unchanged."
- **[F-6] `design.md` §3, conventions sentence.** Keep the true negative rule
  ("The store layout reserves nothing for conventions … never inside `.smut/`") and
  re-time the positive claim: `.config/smut/conventions/` in the project, user and
  binary layers is **DEC-026/028's decided home for the layered loader IMP-013
  tracks**, while the shipped binary embeds its one set. No behaviour change.
- **[F-6] `docs/store-format.md` § "Conventions live outside the store"** — the
  same correction, in the document's own voice (STD-001: the record states what
  the reader does; a decided-but-unshipped mechanism is named as such).
- **[F-8] `smut-implementation-brief.org:89`** — "ID/content disagreement" →
  "identity/content disagreement", matching `docs/store-format.md`.

**Off-surface, do not write these.** `plan.toml`'s `EN-/EX-/VT-` and `PHASE-NN`
ids are immutable-append: nothing above changes a criterion. `README.md` is
CHR-001's write scope (below). `src/store/index.rs` is already repaired at audit
(F-5) and needs no reconcile edit.

**Procedural note.** `design.md` is delivered by a **locked** design run
(rev 47, one change already outside the run). After editing it, re-baseline with
`doctrine design adopt 4` — the sole lawful crossing of a document edited outside
the run — and expect the run's `review_pass RV-025 STALE` advisory to persist; it
is an optimisation signal, not a gate.

### Governance/spec (REV)

- **[F-4] STD-001 — `REV modify`.** In the statement's table, the row
  "the identity key and canonical bytes (entity-first ordering, `id`-first within
  an entity)" → "`smut_id`-first within an entity". Nothing else: the trigger
  column, rationale, scope and the standard's argument are unaffected, and the
  trigger it names has just been exercised correctly by this slice.
- **[F-2, F-3] DEC-007 and DEC-010, amending the proposal's texts.**
  - **DEC-007** gains three passages the proposal omits, in the same act as its
    mapped changes:
    - `title`: "0.1 projection and envelope: borrowing selection, final-boundary
      entity serialisation, `smut_id`-first canonical order";
    - *Alternatives considered*, first bullet: "Fully key-sorted entities (no
      `smut_id`-first) — rejected: `smut_id` is what a reader looks for first, in
      output and in files, for a small amount of code.";
    - the `slug` (`…-entity-serialisation-id-first`) is **tolerated** —
      `doctrine knowledge edit` exposes no slug edit — and the amendment note
      records that, so a later reader does not hunt a fix that cannot exist.
  - **DEC-010**: apply the `choice` amendment as the **full** field text, keeping
    the leading clause the proposal's quotation drops — "README with worked
    example, command table and sharp edges quoting the DEC-003 guarantee;
    `docs/store-format.md` versioned as format_version 2 covering layout,
    serialisation (structural identity `smut_id`, an ordinary `id` as data),
    journal states and fsync obligations, recovery, reserved names, the
    nested-identity corruption refusal and the commit-before-acknowledgement
    ambiguity; learn limited to the shipped surface."
- **DEC-002, DEC-005, DEC-012, DEC-016 — as proposed, no correction.** DEC-005's
  withdrawal (with the DEC-021 citation and the `Reject` signature correction) is
  the only rule change; DEC-002 and DEC-012 change no facet field, which the
  proposal records rather than inventing an edit.
  Conformance reads these six records as six **undelivered** design targets until
  these REVs land — that is the expected state, not a conformance defect.

### Follow-up (owned elsewhere — capture, do not edit)

- **[F-7] CHR-001 — the README's surrounding prose.** Committed at `64e5f4b` it
  still teaches the reversed rule (`README.md:16, 65, 73, 77, 247-250`); the
  operator's in-flight rewrite already carries the correction
  (`README.md:249-255`). CHR-001 owns landing it and keeping the ` ```console `
  fence shape `tests/readme_example.rs` matches. The slice held its boundary and
  must not sweep or stage that prose.

### Outcome expected

After these writes: `slice conformance 4` shows `src/predicate.rs` conformant and
no undeclared slice-owned source path; every binding record states `smut_id` as
the structural key and version 2 as the format; the six decisions carry dated
amendments with their rationale and ids intact; no file inside `.smut/` is claimed
as a conventions home. Then `/close`.

## Reconciliation Outcome

Performed 2026-11-02 against the brief above, on the user's agreement ("agree").
Every finding was already terminal; nothing was re-dispositioned.

### Direct edits applied

- **Selector registry** — `doctrine slice selector add 4 src/predicate.rs --intent
  design-target` (finding F-1, the load-bearing change). `doctrine slice conformance
  4` now reads `src/predicate.rs ⟵ src/predicate.rs` under **conformant**.
- **`design.md` §4** — new row: `src/predicate.rs | Field predicates only — no
  identity read. Update the inline fixtures that name structural identity; keep the
  operator grammar unchanged` (F-1's human mirror).
- **`design.md` §3** — the conventions sentence re-timed: the layout reservation
  stays, `.config/smut/conventions/` is named as DEC-026/028's decided home for the
  loader IMP-013 tracks, and the shipped binary's one embedded set is stated (F-6).
- **`docs/store-format.md` § Conventions live outside the store** — the same
  correction in the record's own voice (F-6, STD-001).
- **`smut-implementation-brief.org`** — "ID/content disagreement" →
  "identity/content disagreement" (F-8).
- **`src/store/index.rs:1`** — repaired during the audit pass, not here (F-5);
  gate re-run afterwards.

**Procedural correction.** The brief told reconcile to `doctrine design adopt 4`
after editing `design.md`. The engine refuses: *"run is locked; adopt refuses. …
at reconcile, edit design.md directly and do not adopt"*. The design was edited
directly, per the engine's own guidance. The run's `review_pass RV-025 STALE`
advisory stands and is an optimisation signal, not a gate.

### REVs completed

- **REV-002** (`reconcile-sl-004`) — `done`, one `[[change]]` row: `modify STD-001`,
  landed by hand after apply surfaced it. STD-001's canonical-bytes parenthetical
  now says `smut_id`-first; the trigger column, rationale, scope and every other row
  are unchanged (covers finding F-4). Narrative and before/after in
  `revision-002.md`.

### Knowledge surface (not REV-targetable)

The six decision records were amended through `doctrine knowledge edit` — facet and
prose in the same act, each carrying the dated SL-004/ADR-001/DEC-025 note. The REV
engine's target set excludes a DEC (REV-001's precedent), so this is the sanctioned
route:

- **DEC-002** — coalescing example pair → `smut_id`; the created/identity-only
  consequence. Facet unchanged (no identity spelling in any field).
- **DEC-005** — the withdrawal: `id` may be inline metadata, `smut_id` is the
  reservation; facet `context`/`choice`/`consequences`; `reference_id` return type
  corrected `Error` → `Reject`; `What counts`, declaration/reference, labels, both
  examples, the error-class table and the documented consequence rewritten.
- **DEC-007** — facet `choice`, **`title`** (`smut_id`-first canonical order), the
  Envelope bullets, canonical ordering, **first rejected alternative**, the
  serialisation clarification (covers F-2). The `slug` is **tolerated** —
  `doctrine knowledge edit` has no slug field — and the record's own note says so.
- **DEC-010** — `choice` replaced as the **full** field text including the leading
  README clause (F-3), `consequences`, the README sharp edge, the version-2
  store-format description, the closing refusal sentence and the learn paragraph.
- **DEC-012** — one envelope example key; facet unchanged.
- **DEC-016** — facet `choice`; the recipe's `select: ["smut_id"]` in both places.

Verified after the writes: no `carrying \`id\``, `member \`id\``, `\`id\`-first`,
`non-\`id\``, `{"id"`-as-identity or `format_version: 1` claim remains in any of the
six records; the only surviving `id` mentions are the withdrawal sentence itself
and value/container vocabulary.

### Withdrawn / tolerated

- **F-7 — follow-up, not written here.** CHR-001 owns the README's surrounding
  prose. The committed file still teaches the reversed rule; the operator's in-flight
  rewrite already corrects it, so the remaining work is landing it (and keeping the
  ` ```console ` fence shape the harness matches).
- **DEC-007's slug** — tolerated as an immutable identifier, rationale recorded in
  the record.

### Conformance note

`doctrine slice conformance 4` now reports `src/predicate.rs` conformant and no
slice-owned undeclared source path. Its **six undelivered** rows are the decision
records: those edits land after the last completed phase's recorded `code_start`/
`code_end` range, and no completed phase may be re-opened to absorb them. The
selectors are satisfied by the work; the cell is a registry-range artefact, not
unfinished scope.

Reconcile pass complete — handoff to `/close`.

### Gate state — resolved

`just check` at the operator's `4fecf9a` ("readme and such") failed three tests in
`tests/readme_example.rs` at `tests/readme_example.rs:232`:

```
the command line has balanced quotes: smut declare '
```

`README.md:53` opens `$ smut declare '` with the JSON wrapped to a closing quote on
its own line — valid shell syntax, which the rewritten harness could not read
because it took one console line per command and swallowed the continuation as
recorded output. Not SL-004's code, prose or gate: the slice's own tree was green
at `64e5f4b` and again at `58612ca`.

**Resolved on the user's direction ("fix the test so it passes with valid
multi-line shell syntax"), commit `37dda01`.** `transcripts` accumulates console
lines while a quote is open and refuses a block that closes before the quote does;
a focused test pins that a wrapped command is one step, that the whole quoted
operand reaches the binary as one argument, and that only the line after the
closing quote is output. `cargo test --test readme_example`: 8 passed.
`rustfmt --check` on the file: clean.

**Gate at this commit: green.** `just check` in a detached worktree at
`37dda01` — exit 0, zero warnings, 370 lib and 383 `--features
fault-injection`, every suite passing. It is read in a worktree rather than the
live tree because SL-007 is editing `src/cli.rs`, `src/spec.rs`, `src/walk.rs` and
`tests/traversal.rs` uncommitted in it; that work is the operator's and was not
touched.

Reconcile pass complete — handoff to `/close`.
