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
under the existing suite unedited (after a preparatory commit that adds the two
durable help pins EX-1 names), so a behaviour change cannot hide inside a field
move (VA-1 reads that commit alone). `-j` is then proved on `q` against
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
| PHASE-01 | clap struct move under a green suite, then one flag and one refusal | sustained frontier reasoning for the refactor commit, plus strict adherence for the field move: the flattened-struct failure modes — the parent's `about` text, the positional's behaviour — are pinned only by the two assertions EX-1 adds, so a competent shortcut can go green and still ship changed help |
| PHASE-02 | five sentences and their pins | sustained judgement — the truth reading (VA-1) is the work; the edits are trivial |
| PHASE-03 | one arm, many end-to-end proofs | careful adherence — the proofs are tables; the discriminating help and store checks are where a hurried pass goes vacuous; the two places a test passes for the wrong reason are the ordering (display flags validated before the selection) and the `-l` shadowing rows (`r -l --ids`, `r -l -j 3`) |
| PHASE-04 | prose, constants, a render runner, README | careful adherence — many pins move together; the complete-surface clause must not — plus prose judgement (the VA-1 reading: nothing taught that the CLI refuses), landed as three commits (constants and fixture runners; guide prose; README) so a red pin cannot be mistaken for a red behaviour |

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
- **§ Refusals, "two name `q` as the subject".** Three sentences do —
  `src/dsl.rs:319` (`so q cannot tell the field from the value`),
  `src/dsl.rs:581` (`q carries one {spelling} per field`) and `src/error.rs:387`
  (`q takes field=value tokens…`). All five rows are already PHASE-02/EX-1, EX-2
  and their pins, so no criterion changes; owed: correct the count in the design
  at reconcile, since it is the sentence a reader would use to decide which rows
  need a subject change.
- **§ Verification, the stated divergences "both asserted".** The design does not
  say where the terminal half can be asserted: the contract harness always pipes
  stdin (`tests/cli_contract.rs:96-124`), so `MissingInput` is unreachable from a
  spawned command and only `parse_with(…, terminal: true)` in `src/cli.rs` can
  observe it. PHASE-03/EX-5 splits the two seams and VT-3/VT-5 name both files.
  Owed: name the seam in the design.

### Test-design notes carried into the phases

- The existing help-list tests assert `text.contains(named)`; for `r` that is
  vacuous. PHASE-03/EX-6 requires a line-anchored match.
- `!has_store()` alone is non-discriminating (IMP-015); PHASE-03/EX-3 pairs it
  with a `declare` in the same scratch directory, as SL-010 did — which shows the
  scratch admits a store but leaves IMP-015 open, because the ordering claim
  rests on the exit 2 each row asserts.

### Reconcile obligations

As design § Reconcile obligations: amend DEC-034, DEC-008, DEC-039, DEC-040 via
`doctrine knowledge edit decision`, comma-free arrays; plus the corrections
above.

### Risks and mitigations (critical pass, 2026-11-03)

| risk | evidence / inference | mitigation |
| --- | --- | --- |
| A doc comment on a flattened `clap::Args` struct leaks into the parent command's help | inference: a known clap-derive behaviour class; the design's probe checked parsing and the `q` usage line, not the `about` text | PHASE-01/EX-1: three help captures byte-identical across the refactor, plus durable pins — the `about` line in `src/cli.rs`'s help test, `render`'s usage line beside `q`'s |
| The refactor hides a behaviour change | inference | refactor is its own commit, suite unedited, read alone (PHASE-01/VA-1) |
| The gate is red before the slice starts, on the version pin | fact, not inference: `tests/cli_contract.rs:547` asserts `smut 0.2.2`, `Cargo.toml:3` reads `0.3.0`, and `Cargo.lock`'s `smut` entry moved with it, so `help_and_version_exit_zero_and_touch_no_store` fails now | PHASE-01/EN-3: the operator decides before PHASE-01 — move the pin with the bump as CHR-004 did, or record a known-red baseline for that one test — and it is settled before PHASE-03, which edits the same file |
| Foreign work in the tree is disturbed | fact: `git status` shows `Cargo.toml`, `Cargo.lock` and `flake.nix` (a whitespace reformat of the `packages` attrset) already modified | PHASE-01/EN-3: all three neither committed nor reverted by this slice |
| The render recipe's equivalence row proves nothing | fact: `tests/fixtures/interview/entities.json` carries no `q-query`, so both spellings render only the missing notice and match whatever `r` does | PHASE-04/EX-5: the test declares `q-query` into a fresh scratch first and asserts the output contains `## Query design` |
| The terminal divergence cannot be asserted where the plan first put it | fact: `run` always pipes the child's stdin (`tests/cli_contract.rs:96-124`), so `MissingInput` is unreachable from a spawned command | PHASE-03/EX-5 and VT-5: the terminal half moves in-process to `src/cli.rs`; the pipe-reachable rows stay in `tests/cli_contract.rs` |
| `r` reads stdin when it has no words | inference: `render`'s arm does exactly that | PHASE-03/EX-2's empty row pipes a spec that must be ignored |
| A help or store assertion passes vacuously | evidence: existing `contains(named)` list; IMP-015 | PHASE-03/EX-6 line-anchored; EX-3's paired `declare` shows the scratch admits a store, and the ordering claim rests on the exit 2 |

Checked and clear: no `alias` or `-j` letter exists in `src/cli.rs`; clap is
4.6.7 in `Cargo.lock`, the version the design's probe used.
