# Implementation Plan SL-011: Markdown from a q line — render reachable through the humane grammar

Prose companion to `plan.toml`. Narrative only — the phase list, criteria and
verification live in the TOML.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Four phases. Each lands only claims that are already true when it lands:

```
PHASE-01  structs + -j on q        ── -j exists ──┐
PHASE-02  escape sentences → -j    ◄──────────────┘  (a sentence may now point at it)
PHASE-03  the verb r               ── inherits final sentences; parity pins them once
PHASE-04  guide + README           ── every printed line is a command that works
```

## Sequencing & Rationale

**PHASE-01 — two shared structs, and `-j` on `q`.** The refactor and the new
flag are one phase because `-j` lives on `SelectionArgs` and has nowhere to go
until the struct exists — but they are two commits. The refactor lands first,
under the existing suite unedited, so a behaviour change cannot hide inside a
field move (VA-1 reads that commit alone). `-j` is then proved on `q` against
`query`, the verb whose output is already pinned, before a second verb
multiplies the cases. `JsonBesideSelection` is born here because `specification`
cannot be written without it.

**PHASE-02 — the escape pointers.** Small, but it is the judgement phase: five
sentences whose truth a pin cannot check
(`mem.pattern.smut.refusal-pins-hold-shape-not-truth`). It sits after PHASE-01
because "use -j" is false until `-j` exists, and before PHASE-03 so `r`'s parity
table pins the final sentences once rather than twice. The guide's escape prose
is left alone: "smut query" stays true there, and the guide is PHASE-04's.

**PHASE-03 — the verb `r`.** With both structs and the final vocabulary in
place this is a short arm and a lot of proof: byte identity across spellings and
display flags, refusal parity with `q`, the two stated divergences from `render`,
and the help surface. The parity table is what catches a copy-paste between the
`Q` and `R` arms.

**PHASE-04 — the guide and the README.** Last, because DEC-040 (a spelling the
guide does not teach does not exist) is satisfied by an executed recipe, and the
recipe needs a working `r`. Between PHASE-03 and PHASE-04 `r` exists untaught;
that window closes inside the slice.

The equivalence spine is split by where its evidence lives: CLI-level rows
(small `cli_contract` store, every spelling) in PHASE-03; recipe-level rows
(the guide's constants, the acceptance fixture) in PHASE-04.

## Model requirements

| phase | what the work is like | model requirement |
| --- | --- | --- |
| PHASE-01 | clap struct move under green, then one flag and one refusal | careful adherence — the refactor must change nothing; DEC-037's positional must survive the flatten |
| PHASE-02 | five sentences and their pins | sustained judgement — the truth reading (VA-1) is the work; the edits are trivial |
| PHASE-03 | one arm, many end-to-end proofs | careful adherence — the proofs are tables; the discriminating help and store checks are where a hurried pass goes vacuous |
| PHASE-04 | prose, constants, a render runner, README | careful adherence — many pins move together; the complete-surface clause must not |

## Notes

### Corrections owed to the design

- **§ Verification, equivalence spine, filter row.** The design lists
  `EXAMPLE_FILTER_Q` among the recipes whose `r` and `render` output is
  byte-identical. It cannot be: the line asks for the ids as roots, answered in
  the order written; `EXAMPLE_FILTER` asks for them through `$in`, answered in
  `smut_id` order. SL-010 established this (RV-031 F-6) and
  `tests/fixture.rs` `the_two_call_recipe_runs_in_both_spellings` compares only
  the set. PHASE-04/EX-5 claims set equality for that pair and byte identity for
  the rest. Owed: amend design § Verification's spine row at reconcile.
- **§ Verification, "the other two hot paths".** `src/learn.rs`'s five hot paths
  minus the two already twinned (`IDS status=open --ids`, the traverse line)
  leave three, not two. PHASE-04/EX-5 covers all three.

### Test-design notes carried into the phases

- The existing help-list tests assert `text.contains(named)`; for `r` that is
  vacuous. PHASE-03/EX-6 requires a line-anchored match.
- `!has_store()` alone is non-discriminating (IMP-015); PHASE-03/EX-3 pairs it
  with a `declare` in the same scratch directory, as SL-010 did.

### Reconcile obligations

As design § Reconcile obligations: amend DEC-034, DEC-008, DEC-039, DEC-040 via
`doctrine knowledge edit decision`, comma-free arrays; plus the spine correction
above.

### Risks and mitigations (critical pass, 2026-11-03)

| risk | evidence / inference | mitigation |
| --- | --- | --- |
| A doc comment on a flattened `clap::Args` struct leaks into the parent command's help | inference: a known clap-derive behaviour class; the design's probe checked parsing and the `q` usage line, not the `about` text | PHASE-01/EX-1: three help captures byte-identical across the refactor |
| The refactor hides a behaviour change | inference | refactor is its own commit, suite unedited, read alone (PHASE-01/VA-1) |
| A concurrent change (uncommitted 0.3.0 bump, observed at plan time) reddens the gate | evidence: `git status` | PHASE-01/EN-3: never commit or revert it; stop and ask if it reddens the gate |
| `r` reads stdin when it has no words | inference: `render`'s arm does exactly that | PHASE-03/EX-2's empty row pipes a spec that must be ignored |
| A help or store assertion passes vacuously | evidence: existing `contains(named)` list; IMP-015 | PHASE-03/EX-6 line-anchored; EX-3 paired `declare` |

Checked and clear: no `alias` or `-j` letter exists in `src/cli.rs`; clap is
4.6.7 in `Cargo.lock`, the version the design's probe used.
