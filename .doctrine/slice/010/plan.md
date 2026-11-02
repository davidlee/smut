# SL-010 — the phase plan

Four phases, cut where the evidence changes kind rather than where the files
change. The compiler is a pure function, so it can be proved with a table before
anything runs — but it is two sessions, because the token and merge rules and the
flag and refusal vocabulary are different work with a seam between them. The verb
can only be proved by running two commands against one store. The guide can only
be proved once there is a command worth typing.

`PHASE-04` is the second session and sits between `PHASE-01` and `PHASE-02` in
the array. Phase ids are append-only, so the id sequence reads 01, 04, 02, 03 and
that is expected; its criteria keep the ids they were written with (`EX-6`…`EX-10`,
`EX-11` as the gate, `VT-3`), which is why they do not run from one, and `VT-4`
is new because the flag mappings had no test to move.

## Why this order

**PHASE-01 — the compiler's core.** Everything the design decides about *meaning*
lives here: what a token is, what a value is, when two conditions collide. It is
the phase with the most false-claim risk, because a rule that reads well can still
be wrong at a boundary — and the design review found three of exactly that kind.
It is also the phase that can be finished without a store, a command line or a
guide, so its suite is the contract the next three phases lean on. `value()` and
the merge rules are the two places to slow down.

**PHASE-04 — the flag and refusal vocabulary.** Twelve flags keyed to their
specification keys, one comma-list rule over seven consumers, two strictness
rules, two shorthand conflicts, six `Reject` variants with a sentence apiece.
Once the merge core exists this is mostly transcription — except the messages,
which is where care shows: every row has to name what the caller typed, invent no
field, and stand as one line. A sentence that reads well can still be a second
voice in the refusal vocabulary.

**PHASE-02 — the verb.** The clap shape is settled (DEC-037 and the spike), so
this phase is mostly wiring — but the wiring is where "one engine" is either true
or merely asserted. The byte-identity pair and the paired `declare` are the two
tests that keep it true. Nothing here may widen `spec::parse`, and nothing here
may touch `render`: the design's unchanged table is a claim this phase has to keep
rather than a note, and one `git diff --stat` over the whole of it is the check.

**PHASE-03 — the guide.** DEC-016 pins the recipe in three places — the decision,
the `src/learn.rs` constants, and an executed fixture — so teaching a new spelling
is an edit to all three at once, and the decision amendment is owed at reconcile
rather than paid here. This phase is last because its strongest test is a command
that must already work. Two placement facts are load-bearing rather than tidy: the
new section goes *before* the `Querying` heading, and the test that scans
`Querying`…`Traversal` (`src/learn.rs:599-616`) is not touched — a `name:` line
inserted inside that span would turn the ten-key pin red, and the obvious "fix"
would be to loosen it.

## Model requirements

| phase | what the work is like | model requirement |
| --- | --- | --- |
| PHASE-01 | a new module whose contract is a table: classification order, `value()`, three merge rules, `$exists` | **sustained frontier reasoning** — the merge rules are where a rule that reads well is silently wrong, and every sentence in the module is a contract |
| PHASE-04 | flag plumbing, one list rule, six variants and their fifteen sentences | **careful adherence** — the plumbing is mechanical, the message writing is the judgement, and each row names what was typed and nothing else |
| PHASE-02 | one `Sub` arm plus two proof-shaped properties: no store before `spec::parse`, byte identity | **careful adherence** — wiring is mechanical; the two properties are proofs, not edits, and a hurried pass flattens them |
| PHASE-03 | prose, section placement, constants and a token-list harness across DEC-016's three sites | **careful adherence** — mechanical at the edges, sustained at the constants pin and the extended sentence tests |

## What the design review routed here

Every routed finding from RV-031 was a `control` — a repair the design made, whose
proof is a pinned test rather than more prose. Each is transcribed into a criterion
of the phase that settles it, and cites the finding inline:

| finding | settlement |
| --- | --- |
| F-1 the JSON operand had no detector | PHASE-01 EX-2 |
| F-2 satisfiable repeats refused in the contradiction voice | PHASE-01 EX-4, research F1 amended |
| F-3 `status=open status!=resolved` is answered, not refused | PHASE-02 EX-4 |
| F-4 `--has`/`--missing` overwrote each other | PHASE-01 EX-5 |
| F-5 flag lists accepted empties and `any` | PHASE-04 EX-6 |
| F-6 the guide's twin was a different request | PHASE-02 EX-7, PHASE-03 EX-2 |
| F-7 numeric flags had no refusal | PHASE-04 EX-9, PHASE-02 EX-5 |
| F-8 shell-safety was claimed, not enforced | PHASE-02 EX-3, PHASE-03 EX-7 |
| F-9 the clap refusals were described as one line | PHASE-02 EX-6 |
| F-10 `smut render` was unaccounted for | PHASE-02 EX-9, PHASE-02 VA-1 |
| F-11 the variant census said five | PHASE-04 EX-8 |

## Corrections owed to the design

These are recorded here because this plan is the only artifact that can carry
them; they belong in `notes.md`'s obligation list at reconcile, not paid in a
phase.

- **Design §4's struct block prints an attribute its own prose forbids.** The code
  block shows `#[arg(value_name = "TOKEN")]` on the positional; the paragraph
  below it, the observation table, and DEC-037 all say the positional carries no
  attribute and the usage line is `[TOKENS]...`. The prose is the decision and the
  block is the defect: the plan requires a bare `tokens: Vec<String>`, and §4 owes
  the deletion of that attribute line.
- **§3 has no row for a comma list on `~`.** A comma list means one of them, and
  `~` cannot express a disjunction; the plan refuses `title~a,b` in the shape
  `!=` already uses — `cannot read "title~a,b": a comma list means one of them,
  which ~ cannot express; write two conditions or use smut query` — and §3 owes
  the row.
- **§3's rows 1 and 15 both fire on §1's "more than one operator" branch**, with
  different sentences and no rule saying which a given token gets. The plan
  discriminates by kind: a token carrying two `=` reports the field-name row
  (`cannot read "a=b=c": a field name holding = is outside q's grammar; use smut
  query`), and a token whose operators differ in kind reports the both-operators
  row (`cannot read "status~a=b": it carries both ~ and =; use smut query …`). §3
  owes the sentence.
- **§3 has no row for two comma lists on `=`** *(found while executing PHASE-01,
  sheet decision D4)*. `status=a,b status=c,d` is satisfiable — the two lists may
  intersect — so the contradiction voice would be false of it; §2's third bullet
  puts the refusal in the caller's tokens, and PHASE-01 writes it as
  `cannot read "status=a,b status=c,d": q carries one = per field; use smut query`.
  §3 owes the row, and the correction count at reconcile is four, not three.
- **§2's strictness rule refuses the numeric rows before the specification can**
  *(found while executing PHASE-04, sheet finding F1)*. §3's `--depth=-1` /
  `--depth abc` rows and §7's `--depth abc` row promise the specification's
  sentence, but `--depth`/`--budget` without a direction are refused first by this
  front-end (`MissingDirection`). The specification is reached with a direction
  present (`--depth abc --follow any`); §3 and §7 owe the precondition or a
  direction in the example. The correction count at reconcile is five, not four.

The six reconcile obligations the design and the review already carry — DEC-016's
amendment, DEC-035's narrowed wording, DEC-037's superseded consequence, IMP-014's
widened gap, IMP-016's third site, and DEC-039's misdescribed illustration — stay
recorded in `notes.md` and are paid at reconcile.

## What is deliberately not a phase

- **Output shape.** `--brief` and one-line rendering are IMP-002's, and the
  envelope is the contract this slice preserves. No phase touches it.
- **`smut render`'s grammar.** Cheap later, because the compiler is a pure
  function over tokens, and out of scope now because render's operand order and
  display flags are their own migration. PHASE-02 pins that it did not move.
- **The operator table's provenance.** IMP-016's reconciliation is recorded, and
  the third copy lands in `dsl.rs` deliberately (DEC-038). Pulling the census
  forward is scope creep; the plan does not.
- **A refinement of `value()` for the flag text.** PHASE-04 does not add one: the
  numeric flags go through the same rule a token gets, so `--depth 2` is the
  number `2` and `--depth abc` is the string that reaches the specification's
  sentence. Anything else would be a second parser.
- **A per-phase diff over each unchanged file.** One `git diff --stat` over
  design §6's whole unchanged list replaces the two single-file checks, because
  the list is one claim and a base-relative absence check is the weakest form of
  it.

## What would send this back to design

- A byte-identity failure that cannot be fixed without special-casing the
  envelope for `q` — that is the two-engines bug the design forbids, not a test
  to relax.
- A refusal that turns out to need a specification change to voice, because the
  front-end can only be stricter, never different.
- A `{`-leading id in a real store, which would make the JSON-detector rule cost
  a legitimate root rather than an accident.

Doctrine memories for this file:
- [medium] Doctrine lifecycle start: route to slice to design to plan to phase to audit to close — mem_019e9a11e8337613bdf8e96f75a9e6b2
- [medium] Doctrine storage model and the storage rule — mem_019e9a1234ff7e619d865592b0042cb9
- [medium] Doctrine core loop — mem_019e9a12789f7ac39c0841f4d976503b
