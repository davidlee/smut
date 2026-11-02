# IMP-021: A schema gate is a write seam in the shell, not a validator beside it: one shell, two fronts, and no schema kind of smut's own

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The brief excludes this by name: "No domain schemas, type enforcement,
referential integrity…" (`smut-implementation-brief.org:10`, restated at `:225` as
"schema enforcement and integrations", deferred "only when concrete use
demonstrates the need"). IMP-011 is that demonstration: three independently
briefed authors wrote three 34-entity stores, and **0 of 3** followed the one
piece of shipped advice that decides whether `where` is usable later. Advice that
must be sought at a moment nobody is looking is advice that is not applied.

This item records the design that survived a session's worth of investigation, so
the next conversation does not re-derive it. It is a *proposal with open
decisions*, not an approved design. Nothing here is scheduled.

It also contradicts a standing non-goal: any slice descending from this item must
amend the brief at both lines rather than quietly depart from it — the brief is
tier-2 authority, and it is excluded from the published crate, so the amendment
has to be made where it lives.

## The claim

Schema enforcement, if it exists, is a **write-time gate inside the shell** — not
a validator beside smut, not a second frontend, and not schema semantics inside
the engine. (The ladder gains a file kind; the engine gains nothing that knows
what a schema *means*.) The arrangement:

- one shell, moved from `src/main.rs` into the library, parameterised by one seam;
- `smut` = the shell with a schema gate that is **default-on when the project
  ships a schema and absent otherwise**;
- `smut-core` = the same shell with the gate off — the explicit escape;
- schema files discovered through the existing project ladder, as a *different
  kind* from `conventions`.

The agent's surface is unchanged: the same grammar, the same `learn` text, the
same response shapes, the same exit codes. There is nothing new to learn, because
nothing is wrapped.

## The seam already exists

`Store::run` is `lock → recover → load → recompute → commit`
(`src/store/mod.rs:277`). A refusal inside `recompute` therefore happens under
the lock, before `commit`, with nothing written — and if the pre-check against the
empty snapshot refuses, the store is never created. `declare`'s `recompute` is
three lines (`src/cli.rs:95`):

```
let batch = declare::normalize(snapshot.entities(), &self.input, &mut self.ids)?;
let outcome = declare::apply(snapshot.entities(), batch)?;
```

Validate the returned write set **after `apply`**, before returning it: the
entities there are already patch-merged, hoist-resolved and `"DROP"`-resolved, so
the gate never re-implements those three rules. That was the objection that killed
the project-script facade, and this design does not inherit it.

Nothing needs to become public that is not public already: `src/lib.rs` is all
`pub mod`, and `Store`, `Store::run`, `Snapshot`, `Command`,
`declare::{normalize, apply, Batch, WriteSet, Entities}`,
`conventions::{Roots, project_root}` are all reachable today. A read-only or
write-gating `Command` is the shape `Query` and `Delete` already are
(`src/cli.rs:123`, `:153`).

The gate's refusal is a `Reject` — "invalid input", exit 2, stderr, no success
envelope. `Error` stays "exactly these two classes, and no third"
(`src/error.rs:295`) and the 0/1/2 table is untouched.

## Why not the alternatives

**TypeScript/zod as the mechanism.** smut is one self-contained Rust binary; a
sanctioned schema mechanism that needs node in the loop contradicts its whole
posture. Worse, the output envelope carries no version member (0.1's five keys plus
optional `traversal`), so a separately versioned package cannot detect a wire
mismatch. TypeScript retains a *producer* role only: author in zod/typebox if you
like, emit JSON Schema (`z.toJSONSchema()` drops refinements and transforms), and
the engine consumes the file. One sentence in the docs.

**A project-local facade script.** It must re-implement `declare` to validate
*input*: patch-and-preserve, the `"DROP"` sentinel, and hoisting. Three engine
rules re-derived, drifting from the source of truth. Structurally doomed. What
killed it was not the indirection, it was the re-implementation.

**A read-only auditor binary as the whole answer.** It can only ever be a
*detector*: `declare && check` detects after the write, `check && declare` checks
stale state, and neither prevents. Prevention must live in the writer. A read-only
auditor remains a good idea for a different reason (see IDE-003, the census), but
it is not this item.

**`--schema` as a flag on `declare`.** Opt-in per call, so the one thing it exists
to prevent is one forgotten flag away. It also drags a keyword profile into
`learn` and the README as conditional prose.

**A second `[[bin]]` in the same crate.** `cargo install smut` builds and installs
*all* targets, so everyone pays for the validator dependency and gets a binary
they did not ask for. Isolation requires a separate, separately installed binary —
which fails the "nothing to learn" test unless that binary *is* the one the agent
runs. Hence the inversion: the gated front is the product, the raw engine is the
fallback.

## The dependency decision

`jsonschema` 0.60.2 has ~19 normal dependencies (smut's lockfile is 82 packages);
`boon` 0.6.1 has 12 and supports 2020-12 only, last released January 2025. The
recommendation is neither: a **closed keyword profile**, roughly 250–400 lines,
zero dependencies. Reason is house style, not size — DEC-011's closed key set,
five predicate operators, "an unrecognised key or operator is reported as
unknown". "Mostly JSON Schema" is the anti-pattern: every unsupported keyword
(`remote $ref`, `format` semantics, regex dialect, `unevaluatedProperties`) is an
unwritten refusal and an agent footgun, and the cost is permanent prose.

Coverage of the profile is deliberately partial in another direction: it can
express engine rules JSON Schema cannot, and it should.

## Rules the profile must encode

1. **`smut_id` is implicit.** The engine owns it; otherwise every schema repeats
   it and `additionalProperties: false` fails on every entity.
2. **`number`, not `integer`.** smut keeps `1` and `1.0` structurally distinct
   (DEC-002) while ordering them equal; JSON Schema's `integer` would reject a
   stored `1.0`. Either drop the keyword or define it as "zero fractional part",
   explicitly.
3. **Coverage is counted, never assumed.** Type→schema resolution means entities
   with no `type` or an unknown one. Passing them silently is IMP-011's failure
   class in a new coat: the report says how many were validated and how many were
   skipped, by reason.
4. **The gate certifies the write, not the store.** `smut-core` exists and can
   write an entity that never passed a gate. That must be documented, or the gate
   becomes a false assurance about the store.

## What it costs

- **`main.rs` becomes library surface.** Moving `execute`/`dispatch`/`Rendered`
  (341 lines) into the lib is a semver change on a published lib crate, and the
  deny-warnings regime means the new public shell needs its own tests. A
  workspace-local `smut-shell` crate is the alternative if the lib surface should
  not grow.
- **A new kind and a new verb.** `<root>/.config/smut/schemas/<type>.json` via the
  existing ladder — the loader is generic over `kind` (`src/conventions.rs:430`),
  so the *paths* are free, but the *entry parser* is markdown-plus-frontmatter and
  a schema kind needs its own reader. A project also cannot discover its own
  schemas without a listing verb.
- **One tested sentence constrains the naming.** `learn` asserts, and a test
  enforces, "a set never validates, never selects and never touches a store"
  (`src/learn.rs:556`). Keep schemas a different kind from conventions sets and
  that sentence survives intact; put a schema in `conventions/` and the guide
  becomes false.
- **Read-only auditing is not in scope here.** Machine-readable violations on
  stdout with a failing exit cannot live in `smut` without breaking "a failure
  never carries a partial success envelope" (`src/main.rs:64`). That is a `check`
  verb's problem, in the front, with its own exit table.

## Open decisions, in order

1. **Does default-on gating fit the thesis?** Proposed: yes, because the gate is a
   property of the project's files — no schema in the project, no constraint, and
   the README stays true for every project that ships none. This is the decision
   the whole item rests on and it should be taken first, in writing (it falsifies
   "flagrantly permissive" for a project that ships a schema, and that should be
   said out loud rather than discovered in a refusal).
2. **Where the shell lives** — library (semver surface) or a workspace
   `smut-shell` crate.
3. **The front's name and the escape's name.** `smut` gated / `smut-core` raw is
   the proposal; the escape must be documented, deliberate, and leave a trace in
   shell history.
4. **The closed keyword list**, and how an unknown keyword is refused (by name,
   like every other closed surface here).
5. **The schema kind's name and listing verb**, given the `learn` sentence above.
6. **Whether a per-call override exists at all** (`--no-schema`), and if so what
   it reports on the way past.
7. **Bulk read stays out.** Full-store reading needs no new verb for the gate (the
   `Snapshot` is already in memory), and `query` still caps at 100
   (`src/spec.rs:38`) with no `offset`/`cursor`/`$gt` (`src/spec.rs:571`). If a
   `check` verb arrives later it reads the snapshot directly and inherits none of
   that.

## Suggested order of work

1. **Shell refactor with `AllowAll`.** Behaviour provably unchanged; a parity test
   asserts the front and the core produce identical bytes and exit codes for every
   command and every documented refusal. This slice is the risk sink — get it green
   before any schema code exists.
2. **Profile + gate.** Closed keyword list, type→schema resolution, coverage
   counts, `Reject` path, per-keyword refusal inventory (the IMP-014
   `every_reject_refusal` analogue).
3. **Discovery + listing + docs.** Schema kind in the ladder, listing verb,
   `learn` section, README paragraph stating the gate certifies writes and never
   the store.

`smut-core` ships in step 1, not step 3: the escape must exist before the gate
does.
