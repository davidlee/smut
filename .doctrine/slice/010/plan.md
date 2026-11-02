# SL-010 — the phase plan

Three phases, cut where the evidence changes kind rather than where the files
change. The compiler is a pure function, so it can be proved with a table before
anything runs. The verb can only be proved by running two commands against one
store. The guide can only be proved once there is a command worth typing.

## Why this order

**PHASE-01 — the compiler.** Everything the design decides about meaning lives
here: what a token is, what a value is, when two conditions collide. It is the
phase with the most false-claim risk, because a rule that reads well can still be
wrong at a boundary — and the design review found three of exactly that kind. It
is also the phase that can be finished without a store, a command line or a
guide, so its suite is the contract the next two phases lean on. `value()` and the
merge rules are the two places to slow down.

**PHASE-02 — the verb.** The clap shape is settled (DEC-037 and the spike), so
this phase is mostly wiring — but the wiring is where "one engine" is either true
or merely asserted. The byte-identity pair and the paired `declare` are the two
tests that keep it true. Nothing here may widen `spec::parse`, and nothing here
may touch `render`: the design's unchanged table is a claim this phase has to keep
rather than a note.

**PHASE-03 — the guide.** DEC-016 pins the recipe in three places — the decision,
the `src/learn.rs` constants, and an executed fixture — so teaching a new spelling
is an edit to all three at once, and the decision amendment is owed at reconcile
rather than paid here. This phase is last because its strongest test is a command
that must already work.

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
| F-5 flag lists accepted empties and `any` | PHASE-01 EX-6 |
| F-6 the guide's twin was a different request | PHASE-02 EX-7, PHASE-03 EX-2 |
| F-7 numeric flags had no refusal | PHASE-01 EX-9, PHASE-02 EX-5 |
| F-8 shell-safety was claimed, not enforced | PHASE-03 EX-7, scope reconciled |
| F-9 the clap refusals were described as one line | PHASE-02 EX-6 |
| F-10 `smut render` was unaccounted for | PHASE-02 EX-9 |
| F-11 the variant census said five | PHASE-01 EX-8 |

## What is deliberately not a phase

- **Output shape.** `--brief` and one-line rendering are IMP-002's, and the
  envelope is the contract this slice preserves. No phase touches it.
- **`smut render`'s grammar.** Cheap later, because the compiler is a pure
  function over tokens, and out of scope now because render's operand order and
  display flags are their own migration. PHASE-02 pins that it did not move.
- **The operator table's provenance.** IMP-016's reconciliation is recorded, and
  the third copy lands in `dsl.rs` deliberately (DEC-038). Pulling the census
  forward is scope creep; the plan does not.
- **The five reconcile obligations.** DEC-016's amendment, DEC-035's narrowed
  wording, DEC-037's superseded consequence, IMP-014's widened gap, IMP-016's
  third site — and the sixth the review added, DEC-039's misdescribed
  illustration. None is paid in a phase; all are due at reconcile and are
  recorded in `notes.md`.

## What would send this back to design

- A byte-identity failure that cannot be fixed without special-casing the
  envelope for `q` — that is the two-engines bug the design forbids, not a test
  to relax.
- A refusal that turns out to need a specification change to voice, because the
  front-end can only be stricter, never different.
- A `{`-leading id in a real store, which would make the JSON-detector rule cost
  a legitimate root rather than an accident.
