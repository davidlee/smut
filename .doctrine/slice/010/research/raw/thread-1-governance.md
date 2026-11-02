# Thread 1 raw — governance applicability (SL-010)

Producer: orchestrating agent, degraded mode (no project research agents defined in
`AGENTS.md`, `.doctrine/project-orientation.md` or `arch.local.md`). Read-only.

Commands run:

    doctrine knowledge list
    doctrine show DEC-001 DEC-004 DEC-007 DEC-008 DEC-010 DEC-011 DEC-012 DEC-013 \
                  DEC-014 DEC-015 DEC-016 DEC-019 DEC-027 DEC-030
    doctrine show RFC-001 IDE-006 SL-008 CHR-001 IMP-009 IMP-014 IMP-016
    sed -n 105,165p smut-implementation-brief.org
    grep -rn "DEC-" --include=*.rs src/

Findings are distilled into `../research.md` § Thread 1. This file holds what was
read and the raw observations that did not survive distillation.

## Raw observations

- `doctrine knowledge list` shows 33 accepted decisions (DEC-001…DEC-033) and two
  open questions (QUE-001, QUE-002). The decisions relevant to a query *front-end*
  are not the ones a front-end's author would guess: DEC-016 (where the two-call
  recipe is normative), DEC-027 (a predicate that cannot hold), DEC-030 (what a
  predicate refusal may say) and DEC-019 (additive grammar) all bear, and all four
  are newer than the brief.
- The brief's § Query contract (`:105`) is the machine contract and is silent on
  any human surface. There is no brief clause a `q` verb contradicts; there is also
  none that authorises it — the authority is the operator's, via IDE-006.
- DEC-008's `alternatives` line names "hand-rolled argument parsing" as rejected.
  That is the only clause in the corpus that a failure of the clap spike would
  collide with.
- DEC-027's consequences state, in as many words, that "a where naming a field no
  entity carries is taught and never refused". The DSL's field-typo silence is
  therefore a *decided* silence, not an oversight. IDE-006 calls it "loud enough";
  DEC-027 calls it taught. Both are consistent; RFC-001 still ranks it Tier 1.
- DEC-016's `choice` pins the spelling of both calls in the recipe and its
  `consequences` pin "the recipe includes its direction". `src/learn.rs` prints
  three `EXAMPLE_*` constants; `tests/fixture.rs:547-600` executes them. So DEC-016
  is pinned in prose, in code and in a test.
- IMP-014 (no `every_reject_refusal` analogue) and IMP-016 (five operators in two
  places) are open improvements that this slice *widens* rather than creates.
- SL-008's decisions (DEC-026, DEC-028, DEC-029, DEC-031, DEC-032, DEC-033) are all
  accepted and its slice is `started`; three of its selectors are three of this
  slice's selectors.
