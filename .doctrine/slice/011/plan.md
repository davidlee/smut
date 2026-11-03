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

### Where this plan supersedes design.md during execution

design.md is locked and is not amended until reconcile (user direction,
2026-11-03: carry design changes to reconcile, provided the correct design is
clear during execution). Until then, **where a row below applies, the plan's
criterion is the design**; everywhere else design.md governs.

| # | design.md says | correct design (governs execution) | plan home |
| --- | --- | --- | --- |
| C1 | § Refusals: "two name `q` as the subject" | three: `src/dsl.rs` ~319 (`so q cannot tell…`), ~581 (`q carries one {spelling}…`), `src/error.rs` ~387 (`q takes field=value tokens…`); all five rows change anyway | PHASE-02/EX-1, EX-2 |
| C2 | § Verification spine: the filter recipe is byte-identical across spellings | set-equal only: roots answer in the order written, `$in` in `smut_id` order (SL-010, RV-031 F-6) | PHASE-04/EX-5 |
| C3 | § Verification: "the other two hot paths" | three: `title~graph`, `type=question status=open --select smut_id,title`, `--has body --order title:desc` | PHASE-04/EX-5 |
| C4 | § Verification divergences: "both asserted", no seam | the terminal half (`MissingInput` vs `{}`) is reachable only in-process (`parse_with(…, true)` in `src/cli.rs`); the contract harness always pipes stdin | PHASE-03/EX-5, VT-3, VT-5 |
| C5 | Non-goal "`render` and `query` do not gain `-j`" — no verification | asserted: both still exit 2 with clap's `error: unexpected argument '-j' found` (observed 2026-11-03) | PHASE-03/EX-7, VT-2 |
| C6 | § Command line / R2: hyphen-leading ids are `--`'s; DEC-037's `-weird-id` stays an error that teaches `--` | on `r`, a hyphen-led word starting with a display letter is silently a flag: `-title` is `-t itle`, exit 0 (observed on `render` today). Letters stay (DEC-008, IMP-007); the behaviour is pinned and the guide teaches `--` for hyphen-led ids on `r`. On `q`, `-j…` words are read as `-j`'s value, refused loudly unless valid JSON | PHASE-01/EX-6, PHASE-03/EX-3, PHASE-04/EX-3 |

Not a correction (contested in the hardening pass): "an empty `r` line never
waits on stdin rests on an unstated `dsl::compile` property". `compile` takes no
stdin; the property is `SelectionArgs::specification` reading stdin only in its
`-j` branch, which design § The command line prints, and PHASE-03/EX-2's piped,
ignored spec tests it.

### Test-design notes carried into the phases

- The existing help-list tests assert `text.contains(named)`; for `r` that is
  vacuous. PHASE-03/EX-6 requires a line-anchored match.
- `!has_store()` alone is non-discriminating (IMP-015); PHASE-03/EX-3 pairs it
  with a `declare` in the same scratch directory, as SL-010 did — which shows the
  scratch admits a store but leaves IMP-015 open, because the ordering claim
  rests on the exit 2 each row asserts.

### Reconcile obligations

As design § Reconcile obligations: amend DEC-034, DEC-008, DEC-039, DEC-040 via
`doctrine knowledge edit decision`, comma-free arrays; and amend design.md for C1–C6 above.

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
