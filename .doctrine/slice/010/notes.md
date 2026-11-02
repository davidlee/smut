# Notes SL-010 — the humane query surface

Durable per-slice scratchpad. The disposable phase sheets live under
`.doctrine/state/`; anything that must survive `rm -rf` lands here.

## Design surface triage (exploring, 2026-11-02)

### Constraining governance

| authority | what it fixes here |
| --- | --- |
| brief § Query contract (`smut-implementation-brief.org:105-165`) | the machine surface: ten keys and their defaults, the pipeline order, five operators with their exact semantics, the six-member envelope. A human surface may be **stricter only** |
| DEC-011 + DEC-019 | a 0.1/0.2 `query` call stays valid and additive; `query {}` stays byte-identical; compatibility is a **tested** property (`tests/compatibility.rs`) |
| DEC-004 | the whole request is validated before any store exists; the store is `<cwd>/.smut`, never searched upward |
| DEC-008 | clap v4 derive owns the command grammar; `--` terminates flags for opaque ids; its `alternatives` line **rejects hand-rolled argument parsing** |
| DEC-001 rules 1, 4 | a core `Reject` is exit 2 because the core can express only user error; the parsed `Limit` has no user-reachable unlimited form |
| DEC-027 | a field condition list that is provably unsatisfiable is refused at parse time by **demand + decidability**; a field no entity carries is **taught, never refused** |
| DEC-030 | the contradiction refusal is one variant, one pinned sentence `field {field} cannot be satisfied: {contradiction}`; a refusal never invents a field and never prints an operand literal; one line, no epigram |
| DEC-016 | the two-call recipe is normative in `learn` and its **JSON spelling is pinned**; the recipe must carry its direction and its completeness guard |
| DEC-007, DEC-012, DEC-013, DEC-014, DEC-015 | envelope and traversal members, all inherited unchanged through `Request::Query` |
| DEC-010 | `learn` is the delivery channel and is limited to the shipped surface |
| ADR-001 | identity is `smut_id`; it decides what "the identity field" means for value typing and for `--ids` |
| STD-001, STD-002 | nothing lands inside `.smut/`; no stored-validity rule changes, so no format bump |
| RFC-001 | the harm ranking: this slice buys Tiers 2–3 (calls and reading), and buys nothing in Tier 1 (silence) |
| IMP-014 | no `Reject` census exists; new variants land unpinned by default |
| IMP-016 | the five operators are already enumerated twice in `src/predicate.rs` |

No specs, no policies; one ADR and two standards, of which only ADR-001 bears.

**The two that actually bite.** **DEC-016** pins the recipe's spelling in prose, in
`src/learn.rs` constants, and in an executed test (`tests/fixture.rs:547-600`), so
changing the recipes to DSL lines obliges a decision amendment. **DEC-008**'s
alternatives line rejects hand-rolled argv parsing, so a failure of the clap spike is a
decision change rather than an implementation detail. Everything else constrains
without colliding.

### Shaping decisions — settled before this run, not re-opened

IDE-006 (`humane query DSL`) is the design: the grammar is defined over **argv tokens**,
not over a string; `parse(argv) -> Spec` and the DSL **adds no semantics**; `from` is
demoted to bare words and `where` to infix conditions (`=`, `!=`, `~`, comma list);
every other specification key is promoted to a flag of the same name; shell-active
characters are banned from the grammar, which is why the wildcard is `any` and `$exists`
is `--has`/`--missing`; `smut query` stays the complete machine surface forever.

The run's work list is the research artefact's **design-input deltas D1–D7**
(`research/research.md` § Design-input deltas), not the mechanism.

### Open questions

Leans are recorded before inquiring and are kept as the prior, not as the answer.

| id | question | blocking | lean |
| --- | --- | --- | --- |
| inq-1 | What is the verb called? | yes | `q`, with `query` keeping JSON forever (a bare positional on `query` would be ambiguous with a JSON operand) |
| inq-2 | How is a bare value typed? | yes | JSON scalar if it parses as one, else a string — documented, and `--ids`/`--from=` named as the escape where identity matters (ADR-001) |
| inq-3 | Is a root a bare word or a sigil-marked token? | yes | bare words, with `--from=ID` as the documented escape for an id carrying `=`, `~` or a leading `-` |
| inq-4 | Can clap's derive carry this argv grammar, or must `q` read `std::env::args` itself? | yes | **answered by spike** — see § Spike evidence below: clap 4.6.7 carries all of it with a bare `Vec<String>` and no attributes, so DEC-008 is untouched |
| inq-5 | How does the compiler reach `Spec`, and what may it share with `spec`/`predicate`? | yes | emit the specification object and route it through `spec::parse`, so "adds no semantics" is structural; DSL-only refusals are voiced in the DSL layer before that call, and the operator table derives from one `Operator` rather than being a third copy (IMP-016) |
| inq-6 | Which repeated conditions on one field are refused, and in whose voice? | no | only two `=` on one field, as a contradiction in DEC-027/DEC-030's family; everything else compiles for `predicate::parse` to judge |
| inq-7 | What does the guide teach, in which spelling, and what does that oblige? | no | `q` for the hot paths, `query` kept and marked complete; DEC-016 amended to name both spellings; the two warts (value typing, `~`'s two meanings) stated rather than hidden |
| inq-8 | Does `--ids` combine with `--select`, and is `--brief` in? | no | `--ids` and `--select` refuse each other (silent last-wins is the failure class this slice removes); `--brief` stays out — output shape is IMP-002's |

### Spike evidence — inq-4 (2026-11-02)

Run in a throwaway crate at `/tmp/smut-clap-spike`, `clap = "=4.6.7"` (the exact
`Cargo.lock` version), fully offline. 21 tests green in debug and release; re-run and
confirmed by the orchestrating agent, not just reported. The verbatim findings are kept
at `research/raw/spike-clap-argv.md`; the crate itself is disposable.

**Verdict: clap carries the grammar, with `pub tokens: Vec<String>` and no `#[arg]`
attribute at all.** Case 4 — flags *after* a positional has started — is accepted with
token order preserved: `q --ids session status=open --depth 2` gives
`tokens=["session","status=open"]`, `ids=true`, `depth=Some(2)`.

Five facts the design must carry rather than rediscover:

1. **Do not add `trailing_var_arg` or `allow_hyphen_values`.** Both are *silent*: the
   first turns every flag after the first token into a token (exit 0, four flags gone);
   the second does the same **and** leaks a literal `--` into the token list once a
   positional exists (`q a -- --ids` → `["a","--","--ids"]`), **and** makes
   `q -weird-id` parse as a valid token, deleting the guard `--` exists to give.
   `last`, `value_terminator` and `num_args` are all non-minimal or worse.
2. **A token immediately after a value-taking flag is that flag's value.**
   `q --follow session status=open` means follow=`session`. Standard CLI convention,
   but it means `--follow part-of` is not a query about a root named `part-of`, so the
   guide's examples must put the value where it reads as one (`--follow any`,
   `--follow=part-of`) and the refusal vocabulary must not imply otherwise.
3. **`q` alone is a valid parse** (all fields empty), so "no query at all" is this
   layer's refusal to make, not clap's.
4. **`q -weird-id` exits 2 naming `-w`**, not the token: clap splits the short-flag
   cluster. Loud and never inventing a field (DEC-030 holds), but inaccurate; echoing
   the real token needs a pre-scan of argv, which is a design choice, not a default.
5. **A repeated boolean flag errors** (`--ids --ids` → `ArgumentConflict`) rather than
   being idempotent; opting into `ArgAction::Count` is available if wanted.

### Risks carried from the scope

R1 (clap) is now **inq-4** and a running spike rather than an open risk. R2 (two ways to
enforce "no new semantics") is **inq-5**. R5 is answered by research F1 and needs only its
voicing decided (**inq-6**). R6 stands and is a guide obligation (**inq-7**): a typo'd
field still answers empty at exit 0, which RFC-001 ranks Tier 1 and DEC-027 decided to
teach rather than refuse. R7 (two engines) is answered structurally by reusing
`Request::Query`, and by the byte-identity test. R8 is **inq-8**. R9 (`--edges` beside a
named direction) needs no decision: the spec's own rule is inherited.

### Assumptions the design will carry

- **A1 — output does not move.** `q` returns the same envelope through the same
  `select::envelope`; any output work is a second slice wearing this one's name.
- **A2 — no new specification key.** If the DSL cannot express something, the answer is
  `smut query`, never an eleventh key.
- **A3 — shell-safety is a grammar property.** It is proved by the token classifier's
  tests, not by a shell harness: `tests/readme_example.rs:242` splits on whitespace and
  performs no glob, tilde or `$` expansion, so it can only prove token survival.
- **A4 — one release, no version gate.** `q` ships in the next release beside `query`;
  nothing feature-detects it.

### Obligations owed at reconcile (pre-recorded)

- **DEC-016** must be amended to name the DSL spelling of the two-call recipe, because
  `learn` prints and `tests/fixture.rs` executes it. (Contingent on inq-7.)
- **DEC-008** may need an amendment if inq-4's spike fails. (Contingent on inq-4.)
- **IMP-014**'s census gap widens with each new `Reject` variant; recording that here so
  closure does not read the missing census as coverage.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-02 · exploring · 04d3480

### Produced

- `research/research.md` — two threads, cross-thread findings F1–F7, design-input deltas
  D1–D7. Findings F1 and F2 revise the scope's objective 4 and add an obligation.

### Learned

- DEC-027/DEC-030 already own the same-field contradiction this slice's IDE-006 rule 3
  wanted to invent; the rule shrinks rather than gets reworded (F1).
- The guide's recipes are pinned three ways — prose decision, `EXAMPLE_*` constants, and
  an executed test — so teaching a new spelling is a decision amendment (F2).
- clap's exposure is narrower than the scope claimed: `=`, `!`, `~` are ordinary
  positional content; the untried thing is unbounded positionals interleaved with flags.
- **The spike settled it: clap 4.6.7 carries the whole grammar with a bare
  `Vec<String>` and no attributes.** The danger is not failure but the *silent*
  alternatives — `trailing_var_arg` and `allow_hyphen_values` both misparse with exit 0,
  and the second leaks `--` into the tokens. DEC-008 survives untouched; the spike is
  evidence for a decision, not a change to one.

### Open

- inq-1 … inq-8 (declared, unanswered). inq-4 has its evidence; the rest are open.
- Both exploring gate acts are outstanding: `governance-confirmed` and `graph-reviewed`,
  awaiting the operator's plain replies.
