# SL-010 research — the humane query surface

**Producers.** Both threads were run inline by the orchestrating agent on 2026-11-02:
the project defines no research agents, so this is the graceful-degradation path the
research skill names. `raw/thread-1-governance.md` and `raw/thread-2-code-map.md` hold
the commands and the observations that did not survive distillation.

**Baseline.** `.doctrine/slice/010/research/baseline.toml`, stamped by
`doctrine slice research 10` against `slice-010.md` at commit `87fe2df`.

**Standing on the design.** IDE-006 is the design and this round does not re-open it.
What the round collects is (a) the decisions written *after* the brief that constrain a
front-end at all, and (b) the four places where IDE-006's own rules and those decisions
describe the same mechanism twice. The second is the point of the round: three of
IDE-006's seven open questions are self-contained, but **rule 3 is not** — the tree
already owns that ground.

## Verification legend

✓ read or grepped during this round. Unmarked = cited from a document only.
Design and plan may load-bear ✓ rows, or verify an unmarked row at point of use.

---

## Thread 1 — governance applicability

### Binding, and load-bearing here

| authority | constraint | state |
| --- | --- | --- |
| brief § Query contract (`smut-implementation-brief.org:105-165`) | ten keys and their defaults; pipeline `select roots → traverse → dedupe → order → limit → project`; five operators with their exact semantics (`$contains` = substring on a string, structural member on an array); traversal determinism; the six-member envelope | ✓ |
| DEC-011 | a 0.1 call stays valid; compatibility is a **tested** property, protected by tests rather than by refusal alone | ✓ (`tests/compatibility.rs:1-22`) |
| DEC-019 | grammar and envelope grow **additively**; `query {}` stays byte-identical; the compatibility test was re-cut, not deleted | ✓ |
| DEC-004 | the argument line, the input and every flag are validated **before any store exists**; the store is `<cwd>/.smut`, never searched for upward | ✓ (`src/cli.rs:1-8,296`) |
| DEC-008 | clap v4 derive owns the grammar; `--` terminates flags so an id may begin with `-`; whole request validated in step 1. Its `alternatives` line explicitly **rejects hand-rolled argument parsing** | ✓ (`src/cli.rs:237-290`) |
| DEC-001 rules 1, 4 | a core `Reject` is always exit 2 because the core can express only user error; the parsed `Limit` has no user-reachable unlimited form | ✓ (`src/spec.rs:52-80`) |
| DEC-007, DEC-012, DEC-013, DEC-014, DEC-015 | envelope members are `smut_id`-first, `traversal` is conditional, frontier ≠ result limit, `edges` is induced among returned objects, budget counts expansions | ✓ (inherited unchanged by `Request::Query`) |
| **DEC-016** | **the two-call idiom is normative in `learn`, and its `choice` pins the JSON spelling of both calls**; the recipe must carry its direction and its completeness guard | ✓ (`src/learn.rs:287-310`, `tests/fixture.rs:547-600`) |
| **DEC-027** | a field condition list that is **provably unsatisfiable is refused at parse time**, naming the field and the contradiction, decided by *demand* (position) and *decidability* (`$eq`/`$in`/`$exists:false` bound the field; `$contains` alone does not). The absent-field case is **taught, never refused** | ✓ (`src/predicate.rs:60-110`) |
| **DEC-030** | the contradiction refusal is **one** variant `ContradictoryFieldConditions { field, contradiction }`, rendered `field {field} cannot be satisfied: {contradiction}`; a refusal **never invents a field** and never prints an operand literal; message text is a pinned surface, one line, no epigram | ✓ (`src/error.rs`, `src/main.rs:114-116`) |
| DEC-010 | the documentation set; `learn` is limited to the shipped surface and is the delivery channel | ✓ |
| ADR-001 | identity is `smut_id`, a reserved key in a permissive store; an ordinary `id` is data | ✓ |
| STD-001 | `docs/store-format.md` is the store format's only record; nothing may appear inside `.smut/` for a query feature | ✓ |
| RFC-001 | the harm ranking: three findings fail silently (Tier 1), the rest "cost calls or reading" (Tiers 2–3). A front-end buys Tiers 2–3 only | ✓ |
| IDE-006 | the design this slice implements | ✓ |
| IMP-014 | no `every_reject_refusal()` census exists; new `Reject` variants land unpinned by default | ✓ (`src/error.rs:457-470`) |
| IMP-016 | the five operators are already enumerated in two places; a sixth must be added to both | ✓ (`src/predicate.rs:94-102`) |
| SL-008 (DEC-026, DEC-028, DEC-029, DEC-031, DEC-032, DEC-033) | accepted, and its slice is `started`; **three of its selectors are three of this slice's selectors** | ✓ (`src/conventions.rs` modified in the tree at round time) |
| CHR-001 (resolved) | the README is operator-owned; the working tree carries an in-flight README edit no agent may restage or resolve | ✓ (`git status`) |

### Checked, not applicable

- **DEC-003 / STD-002** (journal, recovery-before-validation). `q` reads and writes
  nothing: it reaches the store as `Request::Query` and may not create one.
- **DEC-005, DEC-006** (reference hoisting, ordering). The selection pipeline is
  reused whole; `--order` compiles to the same `SortKey`s.
- **DEC-009, DEC-022, DEC-023…DEC-025** — Markdown rendering, old-format stores, and
  the identity-sweep reconciliations: untouched by a front-end.
- **DEC-020, DEC-021** (identity assertions, eager nested-identity validation at load):
  reached only through the store, which is not changed.
- **DEC-029** (the name `conventions`) — unrelated vocabulary.
- **QUE-001** (typed schemas) — this slice does not introduce field-name validation, and
  DEC-027 already closed the field-absence question the other way.
- **QUE-002** (config-home env var) — `q` reads no environment.
- **IDE-004** (edge attributes) — `--follow` gains no vocabulary.

### Revision candidates

1. **DEC-016 must be amended, not merely followed.** Its `choice` and `consequences`
   pin `smut query '{…}'` as the spelling of both recipe calls, and the recipe is
   *executed* (`tests/fixture.rs:547-600`). The moment `learn` prints `smut q` lines,
   a decision and the guide disagree. SL-008 set the precedent for paying this at
   reconcile rather than mid-slice ("Obligations owed at reconcile", its `notes.md`).
2. **DEC-008 needs a ruling if the clap spike fails.** Its `alternatives` line rejects
   hand-rolled parsing, so "read `env::args` ourselves" is a decision change, not an
   implementation detail.
3. **DEC-011's rationale is the reason `query`'s bytes must not move** — worth citing
   in the design rather than restating.
4. **IMP-014 widens.** The DSL adds several refusals and there is no census to catch a
   missing one. Either this slice adds the census (scope growth) or it records the gap
   explicitly (honest, and IMP-014's own remedy).
5. **DEC-027's "taught, never refused" sentence is the guide's obligation**, and
   IMP-012 already owns the guide sentences about the surface's limits. A field typo
   via `q` is the same silence, one keystroke cheaper.

---

## Thread 2 — code map

### Hotspots

| site | what is there | why it matters |
| --- | --- | --- |
| `src/cli.rs:237` | `enum Sub` — ten variants, clap derive | `q` is an eleventh variant; the whole grammar question lives in its `#[arg]` attributes |
| `src/cli.rs:296` | `pub fn parse(arguments, stdin_is_terminal, stdin)` | DEC-004 step 1 in full; the compiler must be called from here, before the store |
| `src/cli.rs:383-388` | clap `ErrorKind` → `Reject::UnknownCommand` / `Reject::InvalidUsage` | new clap-level refusals inherit `smut: {message}` and exit 2 for free |
| `src/cli.rs:103-133` | `Request::Query(Query { spec })` + `recompute` | the only place `q` should terminate: same variant, same `select::select`, same envelope |
| `src/main.rs:16,114-116` | `fn main`, one-line `smut: {error}` (+ usage for usage errors) | stderr is one line; DEC-030's "no epigram" is structural, not stylistic |
| `src/spec.rs:121` | `pub fn parse(&Value)` — the single ingress; `parse_*` helpers private `:167-296` | a compiler that builds `Spec` directly cannot reuse the helpers without widening them |
| `src/spec.rs:67` | `Limit::capped` is `pub const fn`; unlimited is crate-internal | `--limit all` has nowhere to land (DEC-001 rule 4) |
| `src/predicate.rs:60-110` | `parse` = build + one `refusal` walk; operator list written twice | the DEC-027 mechanism, and the third-copy hazard (IMP-016) |
| `src/error.rs:20-216`, `:457` | `enum Reject` (~28 variants); `every_store_refusal()` exists, no `Reject` counterpart | where new refusals land, and the gap IMP-014 names |
| `src/learn.rs:31,287-310` | `GUIDE` string; `EXAMPLE_TRAVERSE/FOLLOW/FILTER`; ~35 sentence-pinning tests | the guide is code; the recipes are constants that are *executed* |
| `tests/fixture.rs:27,541-600` | `IDS` substitution, `Declared::query(&Value)`, the shipped-example tests | a DSL recipe needs a token-list runner beside the `&Value` one |
| `tests/readme_example.rs:242-275` | `fn arguments` — whitespace/quote splitter | proves token survival in README transcripts; **not** a shell (no glob, tilde or `$`) |
| `tests/cli_contract.rs:1-8`, `:44` | spawned-process harness, scratch dir, `has_store()` | the no-store assertion must be paired with a store-creating command, never bare |
| `tests/compatibility.rs:1-22` | parsed-`Value` equality + explicit key sets | the guard that the machine surface did not move |

### Naming and shape precedents

- One concept per module, with a `//!` header stating the decision it implements and
  the refusal it makes — `src/spec.rs:1-24` is the model (`src/order.rs`,
  `src/numeric.rs` follow it).
- Every refusal lives in `src/error.rs` and carries exactly the data one sentence
  needs; `Display` is the single renderer.
- Text that must not drift is pinned by a *sentence* test, often through a
  `teaching(claim, &[&[wordings]])` helper (`src/learn.rs:614-651`).
- Flags typed often get a one-letter spelling beside the long one (`-f -i -s -t -l`);
  the guide states that as a convention, so `q`'s flags inherit it or must justify
  departing.
- `--` is already the escape for an opaque id (`get`, `delete`), and there is a test
  for it.

### Nothing found

- No prior second front-end, no `trailing_var_arg`, and `allow_hyphen_values` exactly
  once (`--heading-level`). **Nothing in the tree yet mixes unbounded positionals with
  flags**, so there is no in-repo precedent for `q`'s argv shape.
- No `every_reject_refusal()` (IMP-014 confirmed at the cited site).
- No helper that runs a *token list* rather than a JSON spec.

---

## Cross-thread findings

**F1 — DEC-027 already owns "two conditions on one field", and says it better.**
IDE-006's rule 3 refuses duplicate fields *in the DSL*. The tree refuses the
unsatisfiable subset already, at parse time, by a rule that is sound over an entire
store and whose wording DEC-030 has fixed to one pinned sentence. So
`status=open status!=open` needs no DSL refusal at all: compile it to
`{"status":{"$eq":"open","$ne":"open"}}` and `predicate::parse` refuses it in the
already-decided voice. What survives is the *one* shape JSON cannot spell — two `=`
on the same field — and that shape is itself provably unsatisfiable (two different
`$eq`s), i.e. the same defect class DEC-027 names. The DSL's rule 3 therefore shrinks
from "duplicate fields are refused, with a suggested comma list" to "the one
inexpressible shape is refused, as a contradiction", which deletes a message, keeps
the corpus coherent, and removes the half-wrong advice that IDE-006's own text hints
at ("`write status=open,resolved` for either" is only true when both conditions are
`=`).

**F2 — the guide's recipes are a three-artifact surface.** DEC-016 pins the recipe's
spelling; `src/learn.rs` prints it as constants; `tests/fixture.rs` executes those
constants with a `IDS` → JSON-array substitution. IDE-006's worked examples change the
recipes. So changing them costs: a guide edit, a fixture-harness edit (a token list
where the harness speaks `&Value`), and a DEC-016 amendment. The return is that the
DSL lines become *executed transcripts for free* — the strongest evidence form this
repo has, and exactly what `tests/fixture.rs` was built for.

**F3 — the message surface is a decided contract and the DSL is its newest client.**
DEC-030 constrains every refusal: one line, no epigram, never invent a field, never
print an operand literal. IDE-006's "every refusal names the fix and the sibling" is
compatible but is not free-standing: the "nearest known flag" suggestion may name only
flags that exist, and no DSL refusal may interpolate a field the caller did not type.

**F4 — R1 (clap) is narrower than the scope claims, and is a spike, not an
architecture risk.** A positional token containing `=`, `!` or `~` is an ordinary
`Vec<String>` element — clap only treats `-`-leading tokens as flags, and `--` already
reaches those. The genuinely untried thing is *unbounded positionals interleaved with
flags*, which nothing in the tree does. One small experiment decides it; the fallback
if it fails is a DEC-008 amendment, not a hand-rolled parser.

**F5 — a DSL operator table would be the third copy of the five operators.** IMP-016
exists because two copies already drift by hand. `=`, `!=`, `~` map one-to-one onto
`$eq`, `$ne`, `$contains`, so either the table is derived from a single `Operator` enum
(IMP-016's proposed cure, pulled forward) or the copy is recorded as a third with the
same "add to all three" comment. Escalating IMP-016 is scope creep; *forgetting* it is
how the drift starts.

**F6 — new refusals land unpinned, and that is a known gap.** No `Reject` census
(IMP-014), so the only guard is the per-variant message test discipline DEC-030 relied
on. Roughly six DSL refusals means roughly six message tests, each naming the change
that reddens it.

**F7 — SL-008 is in flight in the same three files.** `src/cli.rs`, `src/error.rs` and
`src/learn.rs` are selectors of both slices, and SL-008 is `started` with
`src/conventions.rs` modified at round time. The `after: SL-008` edge is a hard
practical constraint on this slice, not a preference: it is the `Reject` enum and the
guide that are being edited, which are two of the three surfaces `q` touches.

---

## Design-input deltas

**D1 — rule 3 shrinks (from F1).** Refuse only two `=` on one field, in DEC-027/DEC-030's
contradiction family; compile everything else on one field and let `predicate::parse`
judge. Removes a refusal, a message and a test; inherits a sounder one. Scope § objective
4's "duplicate field" item must be restated, and R5 in the scope is answered rather than
left open.

**D2 — the slice gains a coupled obligation (from F2).** DEC-016's amendment, the three
`EXAMPLE_*` constants, and the fixture harness's token-list path. Affected surface gains
no new file but gains a *decision* obligation; SL-008's precedent is to record it in
`notes.md` under "obligations owed at reconcile" and pay it at closure.

**D3 — every new message is written under DEC-030 (from F3).** Objective 4 gains the
constraint: name only flags that exist; never invent a field; one line.

**D4 — the spike shrinks (from F4).** One experiment: `Vec<String>` positional
interleaved with flags, plus `--` for a hyphen-leading id. The outcome decides whether
DEC-008 needs an amendment; do not design the fallback speculatively.

**D5 — the operator table's provenance is an explicit decision (from F5).** Derive from
one `Operator` enum, or record the third copy. Not to be left to whichever the
implementer reaches first.

**D6 — value typing meets identity (from ADR-001).** Under JSON-scalar-if-parsable,
`smut_id=1` compares the *number* 1 against a string id, and `smut_id` is the one field
ADR-001 makes structural. `--ids` (= `select:["smut_id"]`) and `--from=1` are the
escapes, so the guide must name them exactly where it names the wart. This slightly
strengthens IDE-006's OQ1 rather than settling it.

**D7 — `--brief` stays out (unchanged).** Nothing in the code map pulls output shape
into the slice: the envelope is produced by `select::envelope` inside `Query::recompute`,
which `q` reuses untouched. Touching it is a second slice wearing this one's name.
