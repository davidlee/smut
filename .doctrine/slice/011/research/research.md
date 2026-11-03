# SL-011 research — Markdown from a `q` line

**Producers.** Both threads were run inline by the orchestrating agent on 2026-11-03 at `cb402c3`:
this project defines no research agents (`arch.local.md` and `.doctrine/project-orientation.md`
carry no *Research agents* section — the latter is still the unedited install template), so this
is the graceful-degradation path the research skill names. `raw/thread-1-governance.md` and
`raw/thread-2-code-map.md` hold the commands and the observations that did not survive
distillation.

**Baseline.** `.doctrine/slice/011/research/baseline.toml`, stamped by `doctrine slice research 11`
against `slice-011.md`. The tree was not clean at round time: `.doctrine/slice/011*/` is untracked
and `.doctrine/backlog/improvement/021` is staged — neither touches this surface.

**Standing on the scope.** The round does not re-open the scope doc; it collects what constrains a
*second* verb reaching one grammar. SL-010 is `done`, so R4's precondition ("design should not
start until SL-010's reconcile settles") is cleared, and `src/dsl.rs` is final as SL-010 left it.
The round's own point is narrower than SL-010's: the compiler is not the work — the operand
dispatch, one verb-named refusal, and a test harness the slice assumes already exists are.

## Verification legend

✓ read or grepped during this round. Unmarked = cited from a document only. Design and plan may
load-bear ✓ rows, or verify an unmarked row at point of use.

---

## Thread 1 — governance applicability

### Binding, and load-bearing here

| authority | constraint | state |
| --- | --- | --- |
| **DEC-034** | a human grammar needed a **new verb** because "a bare positional on `query` cannot be distinguished from a JSON operand, so a grammar bolted onto the machine verb would have to guess". Its `choice` also says **"`q` is the only human front-end and has no alias"** — the sentence OQ1(a) narrows | ✓ |
| **DEC-037** | the tokens are a bare `Vec<String>` positional with **no `#[arg]` attributes at all**; three of five candidate attributes misparse *silently*; "any future attribute on the positional must be argued against this record"; a token directly after a value-taking flag is that flag's value; `q` with nothing at all is `query {}` and there is deliberately no no-query refusal | ✓ (`src/cli.rs:266-273`) |
| **DEC-038** | the compiler **emits the specification object and calls `spec::parse`**; DSL-only refusals are voiced in the DSL layer *before* that call; each case where a `spec::parse` message can still reach a human is **listed in the design**, not discovered | ✓ (`src/dsl.rs:75`, `src/spec.rs:121`) |
| **DEC-039** | refuse only what the specification cannot spell; every DSL-only refusal exits 2, names the defect, names **only flags and fields the caller typed**, and points at `smut query` | ✓ (`src/error.rs:378-410`) |
| **DEC-036** | a bare token with no operator is a root; a token carrying an operator is never resolved as an id; `--from=` is the escape | ✓ (`src/dsl.rs:75-135`) |
| **DEC-035** | a bare value is a JSON scalar when it parses as one (string literals deliberately excluded) | ✓ (`src/dsl.rs:237-250`) |
| **DEC-041** | `--ids` with `--select` refuses at exit 2 rather than resolving by precedence; `--brief` belongs to IMP-002 | ✓ (`src/dsl.rs:150`) |
| **DEC-040** | `learn` teaches the DSL for the hot path and keeps `query` as the complete surface; the `EXAMPLE_*` constants and the fixture harness gain a token-list path so printed recipes stay **executed** | ✓ (`src/learn.rs:111-152,367-377`, `tests/fixture.rs:628-690`) |
| **DEC-016** | the two-call idiom is normative in `learn`; its `choice` pins the JSON spelling of both calls, and the recipe must carry its direction and its completeness guard | ✓ (`src/learn.rs:264-276`, `tests/fixture.rs:536-611`) |
| **DEC-011 / DEC-019** | every existing call keeps its meaning **and its bytes**; compatibility is a *tested* property; growth is additive and the re-cut compatibility test is deliberate, not deleted | ✓ (`tests/compatibility.rs:272-289`) |
| **DEC-004** | the whole argument line, the operand and every flag validate **before any store** is opened, locked or created; the store is `<cwd>/.smut` | ✓ (`src/cli.rs:346-457,492`) |
| **DEC-008** | clap v4 derive owns the grammar; `--` terminates flags; one JSON operand with `-` for stdin and tty-detection for the absent case; hand-rolled argument parsing rejected. `--format mermaid` is refused *naming the planned release* | ✓ (`src/input.rs:28-35`, `src/cli.rs:505`) |
| **DEC-009** | render's Markdown is settled: heading from a selected non-empty string field, verbatim body, other fields only on request, identity as a safe fence, diagnostics **leading** | ✓ (`src/render_markdown.rs:30-56`) |
| **DEC-010** | the documentation set; `learn` is limited to the shipped surface and is the delivery channel — "a spelling it does not teach does not exist" | ✓ (`src/learn.rs:40-50`) |
| **DEC-001 r.4 / DEC-033** | the parsed `Limit` has no user-reachable unlimited form; a core `Reject` is always exit 2 because the core can express only caller error | ✓ (`src/dsl.rs:176-181`, `src/error.rs:298-308`) |
| **DEC-027 / DEC-030** | a provably unsatisfiable field condition is refused at parse time in **one** variant, rendered `field {field} cannot be satisfied: {contradiction}`, one line, no epigram, never inventing a field | ✓ (`src/error.rs:378-384`) |
| **ADR-001** | identity is `smut_id`, a reserved key in a permissive store; an ordinary `id` is data | ✓ |
| **STD-001** | `docs/store-format.md` is the store format's only record; nothing may appear in `.smut/` for a query feature | ✓ |
| **IMP-002** | the 1.0 presentation release owns Mermaid, the completed display-flag behaviour, and `--brief` | ✓ |
| **IMP-007** | resolved *in this tree*: render's five short letters `-f -t -l -i -s` exist and are pinned as short/long equivalence pairs. The shared flag struct must not consume one | ✓ (`src/cli.rs:312-335`, `tests/cli_contract.rs:988-1035`) |
| **IMP-014** | no `every_reject_refusal()` census exists; a new `Reject` variant lands unpinned by default | ✓ (`src/error.rs:618-800`) |
| **IMP-015** | `!has_store()` is non-discriminating in the query refusals; refusal tests copy that weakness | ✓ (`tests/cli_contract.rs`) |
| **IMP-016** | the five predicate operators are spelled in three places, and `dsl.rs` names its own table as the third; a shared *flag* set adds a fourth spelling of the **flag names** (in `cli.rs`) if the extracted struct carries them | ✓ (`src/dsl.rs:244-252`) |
| **IDE-007** | `$regex` is a JSON-only sixth operator. The DSL cannot spell it, so render-through-the-DSL cannot either | ✓ (`src/dsl.rs:244-251`) |
| **REV-003** | a `revises` target must be SPEC/PRD/REQ/ADR/POL/STD — **a DEC is a knowledge record, so no REV can carry a decision amendment**. SL-010 amended four decisions through `doctrine knowledge edit decision` and abandoned the empty revision it had opened | ✓ |
| **`mem.fact.doctrine.knowledge-facet-array-comma-splitting`** | `doctrine knowledge edit decision --alternatives/--consequences` are comma-separated array setters: a sentence containing a comma is silently split into fragments; repair needs a comma-free rewrite | ✓ |

### Evidence, not authority

- **IDE-006 §5** — the originating idea and the scope doc's warrant ("one selection grammar, two
  outputs", with `smut render --backlinks part-of --depth 2 session` as the target line). Resolved,
  so it shapes intent; it does not settle OQ1.
- **RFC-001** (open) — the harm ranking. Tier 1 is silent wrong answers; Tiers 2–3 are "costs calls
  or reading". A second spelling of an existing verb buys Tier 2–3 only, and **adds** a Tier-1 risk
  if the operand dispatch ever guesses. That is precisely the trade OQ1 must argue.
- **SL-007** (abandoned) — its two children were resolved elsewhere (IMP-007 here, IMP-018 in
  `learn`). Consequence: `-f -t -l -i -s` were chosen for `render` alone, with no global CLI letter
  scheme. A merged surface must not assume one exists.
- **CHR-001** (resolved · done) — the README rewrite landed and the tree is clean of README edits.
- **`mem.pattern.smut.refusal-pins-hold-shape-not-truth`** — a whole-line message pin buys "the
  rendered sentence cannot drift", never truth. Refusal text that names a *cause* is the
  highest-risk shape; text that names only what was typed is not.

### Checked, not applicable

- **DEC-003 / STD-002** (journal, recovery-before-validation, stored-validity tightening). `render`
  reads and writes nothing: it reaches the store as `Request::Render` and may not create one.
  No format version moves, so STD-002's trigger ("a stored-validity rule that tightens") has no
  subject.
- **DEC-005 / DEC-006** (reference hoisting, ordering). The selection pipeline is reused whole;
  `--order` compiles to the same `SortKey`s, so ordering semantics are not in play.
- **DEC-007, DEC-012…DEC-015** (envelope members, projection boundary, frontier, induced edges,
  budget). Reached through `select::select`; render prints none of the envelope.
- **DEC-002, DEC-018** (declaration equality, the first-use fixture). Render declares nothing and
  no fixture store changes.
- **DEC-017** (conventions layering). Render prints no conventions set.
- **DEC-020, DEC-021, DEC-022…DEC-025** (identity assertions, eager nested-identity validation,
  old-format stores, the identity-sweep reconciliations). Reached only through the store, which is
  not changed.
- **DEC-023** (the implementation brief stays normative). The brief's render section does not move;
  a verb spelling is not a brief clause.
- **No policy exists**; **ADR-001** is the only ADR.

### Revision candidates

**None.** No ADR, policy, standard, spec or requirement moves — SL-011 declares no spec targets,
nothing lands inside `.smut/`, and identity is untouched.

There *is* a knowledge-record amendment candidate, and it is not a revision: **DEC-034**'s
`choice` sentence "`q` is the only human front-end and has no alias" becomes false the moment
`render` accepts tokens. Following REV-003's precedent, that lands at reconcile through
`doctrine knowledge edit decision`, with **no commas** in `--alternatives`/`--consequences`.
DEC-008's "one JSON operand" is also narrowed in description (render keeps the operand *form*
and gains a token form), at lower severity.

---

## Thread 2 — code map

### Hotspots

| path | what is there | why it moves |
| --- | --- | --- |
| `src/cli.rs:239-344` | `enum Sub`: `Q` at 266–310 with **twelve inline flag fields**; `Render` at 312–335 with `json: Option<String>` (314) and five display flags, `--heading-level` carrying `allow_hyphen_values` (322) | the shared args struct lands here; nothing like it exists (`grep flatten\|clap::Args src/` → no hits) ✓ |
| `src/cli.rs:384-449` | the two parse arms: `Q` → `dsl::compile` → `spec::parse` (384–419); `Render` → `check_format`, `Options`, `operand`, `spec::parse` (421–449) | the OQ1(a) operand dispatch lives between them ✓ |
| `src/cli.rs:492-502` + `src/input.rs:28-35` | `operand()` → `input::decide`: `"-"` → stdin, text → operand, `None`+tty → `MissingInput`, `None`+pipe → stdin | render's existing `-`/stdin/absent contract; a token positional must not consume it ✓ |
| `src/dsl.rs:75-215` | `compile`: `{`-leading token → `Reject::JsonOperand` (85–86); flags read in a fixed order, so two defects report the same one ✓ |
| `src/dsl.rs:26-38, 40-72` | `Line<'a> { tokens: &[String], flags }`, `Flags` (twelve owned fields) | the sharing seam: `dsl.rs` has no clap import, so a `cli.rs` struct converting into `Flags` keeps it clap-free ✓ |
| `src/error.rs:105, 385-389` | `JsonOperand` declared, then voiced *"q takes field=value tokens, not a JSON specification; use smut query for a JSON spec"* | the only refusal in the tree that hard-codes a verb ✓ |
| `src/learn.rs:45-49, 111-160, 296-345` | the usage block (`smut render [JSON] …` at 47), the short-form section, the Rendering and Workflows prose with the JSON render recipe at 337 | both the usage line and the recipe move with DEC-040's executed-constant duty ✓ |
| `tests/cli_contract.rs:2028-2042` | pins `Usage: smut q [OPTIONS] [TOKENS]...`; its comment says `value_name`/`trailing_var_arg`/`allow_hyphen_values` "would redden this line" | a shared struct gives both verbs one positional display name; only `q`'s is pinned ✓ |
| `tests/cli_contract.rs:453-505` | render's one-trailing-newline and display-flag behaviour, driven as a process with `{}` | the compat surface a token positional must not disturb ✓ |
| `tests/compatibility.rs:272-289` | `render_pins_its_markdown_text` — exact bytes for `render '{"from":["alpha","beta"]}'` | the guard the slice may not edit ✓ |
| `tests/fixture.rs:615-690` | the guide's printed examples executed, and the two-call recipe in both spellings (`filter_over_words`, 603) | **no render harness exists** — `grep render tests/fixture.rs` → 0 hits ✓ |
| `tests/dsl.rs:29-63` | `argv()`: a second reader of a printed line — whitespace split plus a flag table that panics on an unknown flag | a printed render line needs a third reader that also knows the five display flags ✓ |
| `tests/readme_example.rs:490` | asserts a README step whose command starts `smut render `, contains `"follow"`, and whose output starts `## ` | README examples are executed, and `README.md:275` prints `smut render [JSON]` ✓ |
| `src/render_markdown.rs:30-56`, `tests/render.rs` | `Options` + `render(&Selection, &Options)`; the suite drives select → render across the lib boundary | insulated from the verb change ✓ |
| `src/spec.rs:121` | `pub fn parse(input: &Value)` — the one seam both verbs must funnel through | DEC-038's structural guarantee ✓ |

### Cited facts and naming precedents

- **The flag cross-product is empty.** `q`'s twelve long names and render's five share no name.
  Near-pairs are semantic only (`--ids` vs `--show-ids`, `--select` vs `--show-fields`).
- **No short letter is free.** render owns `-f -t -l -i -s`, `declare` `-c`, `conventions` `-f`,
  clap `-h -V`.
- **`value_name = "ID"` is used on `get`/`delete`; `q`'s positional has no attribute**, so clap
  derives `TOKENS`; render's derives `JSON` from the field name.
- **`allow_hyphen_values` is on an option value, not a positional** (cli.rs:322), so R1's silent
  swallowing does not apply to the token list. The only interaction is loud:
  `--heading-level --ids` binds `--ids` as the level and refuses it as an invalid level.
- **`Reject` is voiced in the declarer's vocabulary** in one place (`error.rs:378+`), one line per
  refusal (DEC-030).

---

## Cross-thread findings

**CF-1 — the ambiguity DEC-034 named has a second half nobody has stated: the absent operand.**
The recorded ambiguity is "a bare positional cannot be distinguished from a JSON operand". Render's
positional *is* that shape (a lone `Option<String>` holding JSON, `src/cli.rs:314`), so OQ1(a) is
a real argument, not an inherited one. But there is a second collision: `render` with no argument
reads stdin when piped and refuses (`MissingInput`) on a terminal, while `q` with no tokens is
`query {}`. A `Vec<String>` positional makes "no operand" mean both, and DEC-011 protects the
current behaviour. `--from=` and `-` are settled; **the empty line is not**.

**CF-2 — one refusal is verb-named, and sharing `compile` forces it out of the compiler.**
`Reject::JsonOperand` is raised inside `compile` (`dsl.rs:85`) and voiced as a `q` sentence
(`error.rs:385`). Under OQ1(a) render must *accept* that token, so the refusal cannot stay in the
compiler: "which verb is being spelled" is a call-site fact. Every other DSL refusal is
sibling-correct for both verbs, because the escape genuinely is `smut query` for both. Corollary:
the slice's affected-surface table is understated in two rows — `src/dsl.rs` must change (lose a
refusal) and `src/cli.rs` must change (gain it), whatever `src/error.rs` does.

**CF-3 — "the guide's render recipe stays executed" is not a property to preserve; it is one to
build.** `tests/fixture.rs` has no render harness at all. The two-call recipe executes in both
spellings because SL-010 built `filter_over_words`; a DSL render line needs the same machinery plus
the five display flags. That is new test scope, not a preservation duty, and the design should size
it as such.

**CF-4 — the collision a shared struct creates is a *display name*, not a flag name.** No long
name collides. But `q`'s positional is pinned as `[TOKENS]...` and render's is `[JSON]`; one
flattened struct gives both one name, and only one of the two usage lines is pinned today. Since
`--heading-level` must keep `allow_hyphen_values` and the positional must keep *no* attributes
(DEC-037), the struct is two different things in one derive — which is exactly the shape DEC-037
warns about.

**CF-5 — R1 is smaller than the scope doc assumes, and the real clap risk is elsewhere.** The
`allow_hyphen_values` on `--heading-level` does not interact silently with a bare token list. What
*is* silent is the thing DEC-037 already found: any attribute added to the positional to "make
render's JSON operand work" swallows flags at exit 0. So the design's clap proof should target the
*positional*, and the `--heading-level` pair is a one-line test.

**CF-6 — the README is live coupling carrying a dead warrant.** SL-011's non-goal calls the README
"operator-owned (CHR-001)", but CHR-001 is resolved and the tree is clean. `README.md:275` prints
`smut render [JSON]` and `tests/readme_example.rs:490` executes a README `smut render` step. The
boundary can stand (SL-010 drew it too) but it is a scope choice now, not a chore's shadow — and
the command table's row goes stale the moment `render`'s usage line changes unless someone owns it.

**CF-7 — refusal parity has two different debts.** DEC-039 gives render every `q` sentence for
free *only because* `compile` is shared (DEC-038). What is not free: (i) a *new* refusal ("tokens
beside a JSON operand") needs its own sentence naming the operand and the fix, in the caller's
words, and `mem.pattern.smut.refusal-pins-hold-shape-not-truth` says the highest-risk shape is a
sentence that names a cause; (ii) new refusal tests will copy IMP-015's non-discriminating
`!has_store()` pattern, and new variants join an uncensused set (IMP-014). Both are known, named
debt — worth a line in the design rather than a discovery in audit.

---

## Design-input deltas

**D-1. OQ1(a) is answerable, and its answer is a dispatch rule with three arms, not one.** The
rule must name: exactly one positional that is `-` → stdin (DEC-011, DEC-008); exactly one
positional beginning `{` → JSON operand; otherwise → tokens. The fourth case — *no* positional —
needs an explicit decision, because `MissingInput` on a terminal and stdin on a pipe are both
protected today while `q` reads an empty line as `{}` (CF-1).

**D-2. `Reject::JsonOperand` moves to the `q` arm of `parse`, and its sentence is not reused.**
The compiler keeps "tokens in, specification out" as its whole contract (D-3 of DEC-038's
rationale); the JSON-operand refusal is a call-site judgement. If the merged surface also has to
refuse a JSON operand *beside* tokens, that is a new refusal with its own sentence (CF-2, CF-7).

**D-3. The shared args struct converts into `dsl::Flags`; `dsl.rs` stays clap-free.** It has no
clap import today, and keeping it that way is what makes "the compiler is one pure function"
checkable. The conversion is `cli.rs`'s.

**D-4. `tests/fixture.rs` gains a render harness** — or objective 5's claim is rewritten. As it
stands the claim is false (CF-3). The cheapest shape is a printed render line that uses **no
display flags**, so the new reader in the fixture does not need to learn five flags it is not
testing.

**D-5. The clap proof targets the positional, not `--heading-level`.** R1 as written is largely
pre-settled; the real risk is any attribute added for the JSON operand (CF-5). One test suffices:
a token list with flags interleaved after a positional, and `--heading-level 3` beside it.

**D-6. DEC-034 needs a knowledge-record amendment, not a revision, and not silence.** Choosing
(a) makes its "only human front-end" sentence false. Land the amendment at reconcile via
`doctrine knowledge edit decision` (REV-003's precedent), comma-free (the facet-splitting
footgun).

**D-7. The README boundary should be re-stated with its real warrant** and given an owner, or the
`smut render [JSON]` row silently becomes a lie (CF-6). A backlog row for the operator's README is
cheaper than a slice-edited operator file.

**D-8. Two measurement questions belong in the design, not the plan:** an equivalence table over
the guide's five hot-path lines plus a DSL twin of the guide's render recipe (DSL vs JSON,
byte-identical Markdown), and a refusal-parity table for the *shared* refusals
(`--ids`/`--select`, `--depth` without a direction, `--limit all`, `--from=` empty,
`--order title:user`, a two-operator token) run through `render` rather than `q` — the second is
the one that would catch a copy-paste in the parse arm. The JSON-operand refusal is deliberately
*not* in that table: `q` and render disagree about it by design (D-2).
