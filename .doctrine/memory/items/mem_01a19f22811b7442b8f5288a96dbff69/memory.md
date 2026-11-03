# A green gate says nothing about its own assertions

Three phases running in SL-008 (2026-11-02), a run declared its suite clean and the review found
an assertion that no input could falsify:

- PHASE-02: three tautologies in the same test file.
- PHASE-03: an unreachable tie-break — the row was green on arrival because the code already had
  the property, and it would have stayed green under the mutation it claimed to catch.
- PHASE-04: an `assert_ne!` nested under an `assert_eq!` that already pinned the same value.

Reviewing "is this correct?" finds none of them. Reviewing "what mutation turns this red?" finds
all three, and the answer is checkable in one edit.

## The practice

For each property a phase claims, name one concrete mutation of the implementation and run the
suite. SL-008 PHASE-05, whose criterion was "`source` is the same literal at every layer":

| # | mutation | reddened |
|---|---|---|
| 1 | `source` ← `set.name` | 5 rows |
| 2 | `source` ← `conventions/<version>/<name>.md` | the same 5 |
| 3 | `version` ← the crate's release for every set | 3 rows |
| 4 | `body` dropped | 9 rows |
| 5 | `title` ← `Conventions: {title}` | 7 rows |
| 6 | `source` right for the project layer, wrong for the user layer | exactly 1 row |

Rules that make the probes worth their cost:

- **Revert byte-identically and prove it** (sha256 of the file before and after the last probe).
- **One probe per property, not one per test.** A probe that reddens nine rows exercised a broad
  property; the probe that reddens exactly one row is the interesting evidence.
- **A row with no probe has no evidence.** Rows green on arrival — "preserved behaviour" — become
  evidence only through the probe that reddens them.
- **A probe that reddens nothing** means the test is decoration, or the property is untested.
  Fix the test before claiming the criterion.
- **A "same X at every level" rule needs levels that differ elsewhere.** SL-008's probe 6 is the
  only control that isolates the two-layer comparison: if both fixture layers carried the same
  body and version, the row would pass because everything matched, not because the rule held.

Related: `mem.fact.smut.vacuous-negative-assertions` (the assertion-level version) and
`mem.fact.doctrine.vt-inert-keywords` (the substring-level version). All three are one defect: a
check whose signal is nil, recorded in a narrative that assumes signal.

## Four more constraints on the probes

- **A mutant that does not compile is not a control.** SL-008 PHASE-02's first "version applied
  inside `parse`" mutant inserted a duplicate struct field; the run went red because the crate
  did not build, which proves nothing. Check that a mutant reddens a **test**, not the compiler.
- **A wide blast radius can be honest.** The probe that deleted the project layer reddened 14
  rows; only `the_kind_is_joined_once` names the defect, the rest are collateral. Report the
  breadth. Engineering a mutant so exactly one row reddens usually proves less, not more.
- **A claim about ordering needs a fixture where the two orders disagree.**
  SL-008's `an_unknown_format_is_still_refused_before_the_operand` passes with the operand
  validated *first*, because its operand resolves: the format is still the only thing refused and
  the message is identical. The property holds (a real run proves it), the pin would not go red.
  When a test's name claims an order, its fixture must make the orders observably different.
- **A handback's green claim is a lead, not evidence.** A worker reported
  `cargo clippy --all-targets -- -D warnings` clean while the gate failed on `clippy::cmp_owned`
  in the new test. Re-run the gate; the gate is the evidence, the handback is the hypothesis.
