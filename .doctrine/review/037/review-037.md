# Review RV-037 — reconciliation of SL-011

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Surface reviewed: the main worktree, `main`, at `ef76f19` (no dispatch; no candidate branch).
Slice base `046b360`; phase deltas PHASE-01 `1ddf284..0c37d52`, PHASE-02 `2603891..f0240d3`,
PHASE-03 `8076140..1fc3c24`, PHASE-04 `bb8a05b..5aae61c`.

Lines of attack:

1. **Registry truth (RV-034 F-7's class).** Each recorded boundary spans only this slice's commits;
   foreign commits in `046b360..HEAD` (`a894e5c`, `9d888fe`, `c30398d`, `ef76f19`) sit outside every
   span; conformance read against that.
2. **Design § The command line.** `SelectionArgs`/`DisplayArgs` flattened, the two methods,
   validation order (display flags → selection → no store), `Flags: PartialEq`, `dsl.rs` clap-free.
3. **Equivalence spine and refusals** (design § Verification, as corrected by plan C2–C5) are
   tested at the seams the plan names, and hold on the built binary.
4. **Truth of every sentence the slice wrote** — refusals (DEC-045), the guide and README — against
   every input that reaches it (`mem.pattern.smut.refusal-pins-hold-shape-not-truth`; RV-031 F-2 /
   RV-034's dominant class). Includes the autonomy log's three "decided" claims: the Sonnet-added
   `-j` sentence, the README's omission of the `r` hyphen rule, and the traversal twin that drops
   `select`.
5. **Hyphen-led words (C6).** `r -title` silent, `q -junk` read as `-j`; what each tells the caller.
6. **Stale prose** left behind by later phases (comments, module docs naming only `q`).
7. **Reconcile obligations** — plan C1–C6 and DEC-034/008/039/040 verified real and complete;
   anything else that became false.
8. **Surfaces that must not move** — `tests/compatibility.rs` and design §5's unchanged list have
   no diff since `046b360`; host gate `just check` and `doctrine check gate` green.

## Synthesis

**Verdict.** The implementation is what design.md, as corrected by plan.md C1–C6, asked for.
`SelectionArgs` and `DisplayArgs` are one flattened struct each; `SelectionArgs::specification`
refuses `-j` beside any token or non-default `Flags` before reading an operand and calls
`spec::parse` once; the `R` arm validates display flags, then the selection, and returns a
`Request::Render` before any store exists; `dsl.rs` gained only `PartialEq` and its sentences.
`tests/compatibility.rs` and design § Code impact's unchanged list have no diff since `046b360`.
Gates: `just check` exit 0 (build, both clippy configurations at `-D warnings`, `cargo fmt
--check`, both test configurations); `doctrine check gate` exit 0; `doctrine slice verify-vt 11`
14/14 PASS; conformance 9 conformant / 0 undelivered / 0 undeclared over an exact registry (F-1).

**None of the seven findings is against behaviour.** Every probe on the built binary matched its
test and its design row. What the audit found is the slice's recurring class one layer out — a
sentence written for the case in front of its author, left standing where a later change made it
false:

- the README's opaque-id bullet, true for `get` and `q -weird`, false for the new verb whose
  positional *is* an id (F-2, major — the only finding a user would feel);
- C6's record of hyphen-led words, which names `-title` but not `-site` or the JSON voice a
  `-j…` word gets (F-3);
- the obligations table, which amended the four records the design named and missed three that
  quote the escape this slice moved (F-4, major);
- comments written in PHASE-01/02, correct then, not revisited when PHASE-03 landed `r` (F-5).

**Tradeoffs consciously accepted (not findings).** `r -title`/`r -site` are silent, and `q -jane`
answers in the JSON reader's voice. Both follow from decisions that outrank them (DEC-008/IMP-007
keep the letters; PHASE-01/EX-5 binds a bad `-j` value to `query`'s message; DEC-037 rejects argv
inspection) and the guide teaches `--` as the rescue on every verb. F-3 asks only that the record
say so.

**Evidence vs inference.** PHASE-01's note that "a `///` on a flattened `Args` struct would reach
the parent's help" was probed in a scratch crate on clap 4.6.7: it leaks only into a variant that
has no doc comment of its own. Every smut variant has one, so the `//` choice was defensive rather
than necessary; VT-3's `about` pin still guards it. Not a finding.

**Standing risks.** IMP-014 (no `Reject` census) and IMP-015 (`!has_store()` non-discriminating)
remain open, named and not paid, as the design said. The silent `-t`/`-s`/`-i` read on `r` is
permanent while the letters stay; only the guide stands between a reader and it.

**Route for a human reader.** `src/cli.rs` `SelectionArgs::specification` (the one place `-j`
and the line meet) → the `R` arm in `parse` (order: `display.options()` then `specification`) →
`tests/cli_contract.rs` `r_refuses_without_creating_a_store` (the ordering row `r -l 7 --ids
--select a`, which the PHASE-03 negative control reddened) and `r_reaches_ids_the_short_form_cannot_carry_bare`
(the C6 characterisation). Decisions worth reading: `doctrine knowledge show DEC-042` (why a verb
and not render tokens), `DEC-044` (why one refusal and not clap's conflicts), `DEC-037` (the
hyphen guard F-3 narrows).

## Reconciliation Brief

Order: design prose first, then the mirrors copied from it, then the records — so every mirror
copies a sentence that already exists.

### Per-slice (direct edit)

- **design.md — C1–C6** (plan.md § Where this plan supersedes design.md): amend each row's
  section as the plan's "correct design" column says. F-7 confirms C1–C5 as written.
- **design.md § The command line / R2 — C6, extended (F-3):** on `r`, words clap reads as `-s`/`-i`
  clusters optionally ending in `-t<value>` (`-title`, `-site`) are silently display flags, exit 0;
  `-f…`/`-l…` words are refused as format/level; on `q` and `r` a `-j…` word is `-j`'s value and,
  unless JSON, is refused in the JSON reader's voice (`malformed JSON: …`). The rescue on every
  verb is `--`.
- **design.md § The guide and the README (F-2):** add a README bullet — README's opaque-id sentence
  ("Without `--` it is a usage error") is narrowed to be true of `get`, `delete`, `q` and `r`.
- **design.md § The guide and the README (F-6):** "the `q` and `r` rows note `-j`".
- **design.md § Reconcile obligations (F-4):** the table lists DEC-034, DEC-008, DEC-039, DEC-040
  (with its escape-list sentence), DEC-035, DEC-036, DEC-037, DEC-043.
- **slice-011.md** — `## Summary` / `## Follow-Ups` are close's; nothing owed from this audit.

### Mirrors (code and README — operator's ruling: paid in the reconcile pass, as RV-034's were)

Each copies the design sentence above; none is authored independently. Then `just check`.

- **README.md:476-478 (F-2)** — the opaque-id bullet, worded after src/learn.rs:159-161.
- **README.md:284 (F-6)** — `smut r [TOKEN]... [-j\|--json JSON]`.
- **src/cli.rs:237, :288, :441-442, :244-245; src/error.rs:108, :119; src/dsl.rs:1-2 (F-5)** —
  comments name both verbs; :244-245 copies C6's narrowed guard.

### Governance/spec (knowledge edits — DEC records, comma-free arrays)

- **DEC-034** — "q is the only human front-end and has no alias" → two human verbs `q` and `r`
  over one grammar; superseded in part by DEC-042.
- **DEC-008** — "one JSON operand" → the spec arrives as an operand on `query`/`render` or as `-j`
  on `q`/`r`.
- **DEC-039** — refusals "point at `smut query`" → point at `-j` (DEC-045).
- **DEC-040** — the division of labour adds `r` and `-j`; the body's "three escapes: `--ids`,
  `--from=ID`, `smut query`" → `-j` (F-4).
- **DEC-035 (F-4)** — choice and body: the exactness escape is `-j` (or `smut query`).
- **DEC-036 (F-4)** — consequence "refuse toward `smut query`" and body "naming `smut query`" → `-j`.
- **DEC-037 (F-3)** — the `-weird-id` guard holds only for letters the verb does not own; narrowed
  on `q` by `-j`, on `r` by `-j` and the display letters (SL-011 C6).
- **DEC-043 (F-3)** — consequences gain the accepted silent `-t`/`-s`/`-i` read on `r`, taught by
  the guide.

Backlog: none. IMP-014, IMP-015, IMP-016 touched and named, not paid.

## Reconciliation Outcome

Paid in the brief's order: design prose, then the mirrors copied from it, then the records.
No REV: every governance item is a DEC, and a DEC is not a `revision change add` target (SL-010
precedent, REV-003). The records were amended on the knowledge surface — `doctrine knowledge edit
decision` for facets (comma-free arrays), direct edits for bodies.

### Direct edits applied (design.md, locked; amended in place, run not resumed) — `3b1cb0b`
- § The surface, `render`/`query` non-goal: asserted (C5).
- § The command line: a "Hyphen-led words" paragraph — C6 as extended by F-3 (`-s`/`-i`/`-t…`
  silent on `r`, `-f…`/`-l…` loud, `-j…` in the JSON reader's voice on `q` and `r`, `--` the rescue,
  the pin named).
- § Refusals: three sentences name `q` as subject, each located (C1).
- § The guide and the README: the `r` row notes `-j` (F-6); a README opaque-id bullet carrying the
  narrowed sentence, true of `get`, `delete`, `q`, `r` (F-2).
- § Verification: three hot paths named (C3); the filter recipe set-equal (C2); the terminal half
  in-process (C4); `render -j`/`query -j` still clap's error (C5).
- § Reconcile obligations: eight records (F-4).

### Mirrors (operator ruling 2026-11-03: reconcile pays them, as RV-034's) — `3b1cb0b`
- README.md: the opaque-id bullet (F-2) copies the design sentence; the `r` commands row gains
  `[-j\|--json JSON]` (F-6).
- src/cli.rs: `SelectionArgs`/`DisplayArgs` comments name `r`; the positional's comment copies C6's
  narrowed guard; `specification`'s doc names both verbs. src/error.rs: `UnreadableToken` and
  `MissingDirection` docs say "short-form". src/dsl.rs: module doc names `q` and `r` (F-5).
- Comment/doc text only. `--help` for the root and all nine verbs byte-identical before/after.
  `just check` exit 0. `tests/compatibility.rs` untouched.

### Records amended (knowledge edits)
- DEC-034 choice + body amendment: two human verbs `q`/`r`, superseded in part by DEC-042.
- DEC-008 choice + Input rules: the operand on `declare`/`query`/`render`; `-j` on `q`/`r`.
- DEC-039 choice + body amendment: refusals point at `-j` (DEC-045).
- DEC-040 choice + body: `r` and `-j` added; the three escapes are `--ids`, `--from=ID`, `-j` (F-4).
- DEC-035 choice + body: exactness escape is `-j` (or `smut query`) (F-4).
- DEC-036 consequences + body: refusals name `-j` (F-4).
- DEC-037 consequences + body amendment: the guard is narrowed per C6 (F-3).
- DEC-043 consequences + body amendment: the accepted silent `-s`/`-i`/`-t` read on `r` (F-3).

### Withdrawn / tolerated
- F-1, F-7: aligned; nothing to write.
- Backlog: none. IMP-014, IMP-015, IMP-016 named, not paid.

`doctrine check gate` exit 0. Reconcile pass complete — handoff to /close.
