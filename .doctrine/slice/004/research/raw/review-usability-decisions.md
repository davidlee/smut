# SL-004 — Review Seat C: the usability decisions (DEC-026…029, RFC-001) against the locked design

Reviewer: Seat C (adjacent-decisions seat). Read-only; no tracked file touched except this report.
Repo state reviewed: `a1b901e` (`design.md` locked), working tree carried the operator's in-flight
`notes.md` / `plan.md` / `plan.toml` / `slice-004.md` edits at the time of reading.
Read: `design.md` §3, §4 (code-impact table, documentation contract, decision amendments), §5;
`plan.toml` PHASE-01/02 acceptance criteria and their EN/VT entries; `notes.md`
§ Pre-implementation review and § Escalated to the design.
Decisions read as the source of truth for the findings below: **DEC-026** (conventions layers),
**DEC-027** (a predicate that cannot hold is refused), **DEC-028** (a conventions set is one markdown
file), **DEC-029** (the name), plus **RFC-001** (the three-store usability exercise) and
**IMP-010/011/012/013**.
Probes run: `grep -n "holds id\|invalid entity id" src/*.rs src/store/*.rs`; `grep -n "src/error.rs"
design.md plan.md` (no match); `grep -n "reserved" design.md`; `grep -rn "/tmp/"
.doctrine/slice/004/*.md`; the selector list in `slice-004.toml`.

Verdict in one line: **the design is accurate against the adjacent decisions — the identity break and
the conventions mechanism do not interfere — but one file the plan edits is absent from the
code-impact contract, and one escalation (D1) was answered by a decision recorded after the design
locked.**

Seat C has no authority over this slice. Everything below is evidence with a proposed disposition;
the operator routes it, and `design.md` is locked until they unlock it.

---

## (a) Findings

### C-1 — `src/error.rs` is edited by the plan but absent from the code-impact contract

| claim | evidence |
| --- | --- |
| EX-9 requires respelling two identity messages | `plan.toml` PHASE-02/EX-9: "the `Reject::InvalidId` message, the `StoreError::IdDisagreement` message, `describe()`'s absent-identity text and the `scan_refs` assertion" |
| both messages live in `src/error.rs` | `src/error.rs:282` `"invalid entity id: {found}, and an id must be a nonempty string"`; `src/error.rs:389` `"the entity file {entry} holds id {found}, and the index maps it to {key}"` |
| that file is not in the design's table | `grep -n "src/error.rs" design.md plan.md` → no match; §4's table is missing the row |
| it is not a selector either | `slice-004.toml` selectors: entity, declare, store/**, walk, select, order, cli, learn, docs/store-format.md, tests/**, conventions, render_markdown, README, brief, six DEC records — no `src/error.rs` |

§4 introduces the table as "the code-impact contract". For this file it is not one, and the gap has a
second, non-local consequence: **SL-005** (refusals, cut 2026-11-02, IMP-010 + ISS-004 under DEC-027)
adds a `Reject` variant, its `Display` arm and tests to exactly that module. The overlap was invisible
from the selectors and was found only by reading the plan.

**Proposed disposition:** add the `src/error.rs` row to §4's table and a selector for it, or state in
§4 that identity-naming diagnostics in that file are in scope via EX-9. Either way record the ordering
rule: SL-005's refusal phase lands before PHASE-02, or takes a rebase.

### C-2 — D1 (the silent `where` on the old spelling) is already decided; it needs a citation, not an adjudication

D1 is escalated as "the design's own wording". The substance is settled:

- **DEC-027** decided this exact class: a `where` naming a field that no scanned root carries is
  **taught, never refused** — refusing would punish filtering as a way to probe a store, and the check
  is only sound when the roots are store-wide. The same reasoning covers an old-spelling `id` after
  the rename, since `id` becomes an ordinary field that no entity carries.
- PHASE-02/EX-8 already takes that position ("a caller still sending the old spelling gets an empty
  result at exit 0, and the guide states that consequence in one sentence"), so the plan and the
  decision agree. What is missing is only the design saying *why*.

The sentence is also drafted, in a form broader than identity (IMP-012): **a field no entity carries
fails every operator exactly as a missing field on one entity does.** That single sentence discharges
EX-8's requirement and one of DEC-027's two obligations to the guide.

**Proposed disposition:** un-escalate D1. State the teach-don't-diagnose rule in §4 or §6 with a
reference to DEC-027, and adopt the sentence above for EX-8 rather than paraphrasing "states that
consequence". No behaviour change, no diagnostic, no new refusal.

### C-3 — the guide rewrite must not restructure `learn`

EX-5 and VT-7 rewrite and pin guide sentences, which is correct and sufficient for this slice. But a
scheduled later slice makes the guide's sections addressable (`smut learn <section>`; IMP-008) and its
pins are prose-substring matches over whitespace-flattened text. A rewrite that renames headings,
reorders sections or merges topics would re-cut that work for no benefit here.

**Proposed disposition:** no change to the design's obligations; add one boundary sentence to §4's
`src/learn.rs` row — identity spelling, examples and the new refusal consequence only; no heading or
ordering changes — and note that each new identity rule gets exactly one home section, so a future
single-section read is self-sufficient.

### C-4 — v2 must reserve nothing for conventions

DEC-026 and DEC-028 place conventions sets in `.config/smut/conventions/`, outside the store, precisely
so that `docs/store-format.md`'s layout and reserved names (STD-001) stay untouched. The design says
nothing about conventions and is therefore correct; the risk is a later implementer "helpfully" adding a
store-side home. `grep -n "reserved" design.md` → one hit, design.md:10, about the identity key.

**Proposed disposition:** one sentence in §3 or §4's `docs/store-format.md` row stating that the store
layout reserves nothing for conventions, so a version-2 document does not imply a home it does not have.

### C-5 — a stale `/tmp` reference survived the move into `research/raw/`

`notes.md:293` reads "See `/tmp/sl004-integration-report.md` §d for wording". The operator moved the
reports into `.doctrine/slice/004/research/raw/` and corrected the other mention (`research/raw/
review-integration.md`, cited at notes.md:206); this one was missed. `grep -rn "/tmp/"` over the slice
directory finds only this line.

**Proposed disposition:** repoint to `research/raw/review-integration.md` §d.

### C-6 — optional: the version refusal is the only teaching channel some callers will meet

Not a defect and entirely the operator's call. The refusal names found and expected versions; DEC-022
decided there is no migration, migration advice or compatibility layer. An owner of a real 0.1 store
meets this message before they meet the guide, and the three-store usability exercise found that agents
read refusals and skip documentation. Adding ", and no migration is provided" converts the refusal into
the channel DEC-022 implies.

**Proposed disposition:** operator's choice; if declined, no record needed.

---

## (b) Checked and consistent — no action

- **The conventions seed.** §4's `src/conventions.rs` row says "without changing seed ids or content",
  and EX-1 routes the seed's raw `"id"` literal through `ENTITY_ID`. That matches DEC-028 exactly: the
  *key* renames, the value `conventions-<name>` stays, and the field list is already a `[(field, value)]`
  array so a later `source` field (IMP-013) is additive rather than a rewrite.
- **The envelope.** DEC-012's amendment retains `depths`, `edges`, `frontier` and endpoint names. That is
  consistent with the decision not to add a `roots` member (RFC-001, 2026-11-02), so the traversal
  surface is unchanged in both documents.
- **Refusals and predicates.** DEC-027's new refusal class reads one field's condition list and never the
  store, so nothing in this slice's identity change reaches it; and no 0.1 query can express a predicate,
  so neither decision moves the other's compatibility property.
- **The store layout.** Nothing in DEC-026/028/029 requires a file inside `.smut/`, and nothing in the
  v2 layout intersects the conventions paths.

## (c) Limits of this review

Seat C is the agent that produced DEC-026…029 and RFC-001, so it is reviewing its own conclusions for
compatibility with someone else's design. Findings are restricted to claims checkable by reading the
repository (line numbers, plan text, selector lists); nothing here re-opens a decision, and C-2 in
particular should be read as "a decision already covers this" rather than as new argument.
