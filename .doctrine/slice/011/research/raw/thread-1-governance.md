# Thread 1 — governance applicability (raw, verbatim round output)

Run inline by the orchestrating agent on 2026-11-03 at `cb402c3`. Read-only:
`doctrine show` / `doctrine search` / `doctrine memory retrieve` only. Nothing written.

## What the round swept

| command | what it returned |
| --- | --- |
| `doctrine status` | 2 active slices, 10 total; 24 open backlog rows; RFC-001 open |
| `doctrine show SL-011` | scope doc; `proposed`; `after: SL-010`; OQ1 + R1–R4 |
| `doctrine show SL-010` | `done` — R4's precondition is cleared |
| `doctrine search "render"` | 25 rows, incl. IMP-007 (resolved), SL-007 (abandoned), IMP-002 (open), DEC-034/009 |
| `doctrine search "DEC-034"` | 62 rows; the decision corpus is `.doctrine/knowledge/decision/001…041` |
| `doctrine adr status` | only ADR-001 exists |
| `doctrine show DEC-0xx` | every decision cited in the artefact, read in full facet + prose |
| `doctrine show STD-001/002, REV-003, RV-034, IDE-006, CHR-001, SL-007, IMP-002, IMP-007` | as cited |
| `doctrine memory retrieve "render markdown cli grammar clap" --tag cli` | 1 hit: the knowledge-facet comma-splitting footgun |
| `doctrine memory retrieve --path-scope src/learn.rs` | 5 hits: refusal-pin shape, doc-comment-hides-contradiction, entity-engine |
| `doctrine memory show mem.pattern.smut.refusal-pins-hold-shape-not-truth` | read in full |

## The decision corpus, scanned for applicability

041 decisions exist. Those read in full and the verdict each got:

- **034** verb + rationale — load-bearing, and the one record OQ1(a) narrows.
- **035** value typing — inherited unchanged; the DSL's `value()` is shared by definition.
- **036** bare roots / `--from=` escape — inherited unchanged.
- **037** clap shape — load-bearing twice: the shared positional must stay attribute-free,
  and the record demands any future attribute be argued against it.
- **038** emit-and-`spec::parse` — the seam render must reuse; also the reason refusal *voice*
  is a design obligation rather than a nicety.
- **039** refuse only what the spec cannot spell — load-bearing; render inherits every sentence.
- **040** learn teaches the DSL, `query` stays complete — load-bearing; its `EXAMPLE_*` +
  fixture duty is the one place where "shared grammar" costs new test machinery.
- **041** `--ids`/`--select` refuse each other, `--brief` is IMP-002's — inherited unchanged, and
  now applies on render too.
- **008, 004, 001, 033** input/validation/exit-code spine — inherited unchanged.
- **009** Markdown rules — untouched by a verb change (render's output is the settled half).
- **010** the documentation set — `learn` is the delivery channel.
- **011, 019** compatibility as a tested property — `tests/compatibility.rs:272` is the guard.
- **016** the two-call idiom — normative, pins the JSON spelling; render is not in that recipe.
- **027, 030** contradiction refusal and its pinned sentence — inherited through `spec::parse`.
- **007, 012, 013, 014, 015** envelope/projection/traversal — reached only via `select::select`.
- **002, 003, 005, 006, 017, 018, 020–026, 031, 032** — checked, not applicable (reasons in the
  artefact: declaration equality, journal, hoisting, ordering, conventions layering, fixtures,
  identity assertions/wire contract, old-format stores, the identity sweep, layer origin,
  loader roots).

## Observations that did not survive distillation

1. **DEC-034 says "`q` is the only human front-end and has no alias."** That is the sentence
   OQ1(a) narrows. It is not a contradiction *today* — render is not a human front-end today —
   but the moment render accepts tokens it is false as written. The authority model calls two
   binding sources that disagree drift; a scope objective against a decision is the same shape
   at a smaller altitude, and the cure is a knowledge-record amendment, not silence.

2. **REV-003 is the precedent for how such an amendment lands.** A `revises` target must be
   SPEC/PRD/REQ/ADR/POL/STD; a DEC is a knowledge record, so no REV can carry it. SL-010
   amended four decisions through `doctrine knowledge edit decision` plus a direct prose edit,
   and abandoned the empty revision it had opened. So this slice's "revision candidates" section
   is empty for a *reason* worth recording rather than an omission.

3. **`doctrine knowledge edit decision --alternatives/--consequences` are comma-separated array
   setters** (`mem.fact.doctrine.knowledge-facet-array-comma-splitting`). A sentence containing
   a comma is silently split into fragments. Any amendment of DEC-034/008/034-adjacent prose
   must keep those two fields comma-free or rewrite comma-free.

4. **CHR-001 is `resolved · done`** and the tree carries no README modification. SL-011's
   non-goal still calls the README "operator-owned (CHR-001)" — the warrant is superseded even
   though the boundary stands. Meanwhile `README.md:275` prints `smut render [JSON]` and
   `tests/readme_example.rs:487` executes a README step that begins `smut render `. So the
   README is a *live* coupling to this slice's usage line, not the dead one the non-goal
   implies.

5. **SL-007 is abandoned and its two children were resolved elsewhere** (IMP-007 for render's
   short flags, IMP-018 for the `follow: []` spelling). Consequence: render's `-f -t -l -i -s`
   were chosen for render alone, with no global letter scheme. A merged CLI surface must not
   assume one exists, and must not take a short letter on anything shared.

6. **RFC-001 is open and is evidence, not authority.** Its ranking puts silent wrong answers in
   Tier 1 and "costs calls or reading" in Tiers 2–3. A second spelling of an existing verb buys
   Tier 2–3 only — and can *add* a Tier-1 risk if the operand dispatch ever guesses. That is the
   exact trade OQ1 must argue.

7. **No policy exists; STD-001 and STD-002 both have their trigger without their subject** —
   nothing lands in `.smut/`, no stored-validity rule tightens, no format version moves.
