# Research — SL-001 smut v1

Pre-design research round for `SL-001` (smut v1 — persisted mutable entity graph
CLI). Artefact tier: runtime working evidence, gitignored in place; durable
findings harvested at close.

- **Producers:** both threads run by the orchestrating agent (degraded mode — the
  project defines no research agents in `AGENTS.md`, `.doctrine/project-
  orientation.md` or `.doctrine/doctrine.toml`). Read-only; no files written by
  the threads themselves.
- **Baseline:** `.doctrine/slice/001/research/baseline.toml`, stamped by
  `doctrine slice research 1`; `doctrine verify research-current --slice 1`
  reports *current*.
- **Raw output:** `raw/thread-1-governance.md`, `raw/thread-2-crates.md`.

## Verification legend

- ✓ — verified by the consuming agent (a command run or a file read in this
  session, cited inline).
- unmarked — researcher claim: cited, not independently checked. Design and plan
  may only lean on ✓ rows, or must verify them at point of use.

## Thread 1 — governance applicability

**Binding constraints**

| Source | How it binds | Status |
| --- | --- | --- |
| `smut-implementation-brief.org` (repo owner, 2026-11-01) | The authoritative statement of v1 intent and of every observable semantic: command set, envelopes, defaults, exit codes, storage protocol, test obligations. Conflicts with SL-001 resolve in its favour. | ✓ read in full this session |
| `AGENTS.md` | Process rules only (TDD red/green/refactor, lint-as-you-go, 2-space indent, DRY, no parallel implementation, professional/dry tone). No product or architecture content. | ✓ in context |
| SL-001 scope doc | Scope, objectives, non-goals; explicitly defers to the brief. Adds one hard out-of-scope: no Doctrine coupling in smut code, no trust enforcement around stored content. | ✓ read this session |

**Checked, not applicable (with reasons)**

- **Project specs** — none exist. `doctrine spec list` empty ✓. The brief is the
  only product-intent document, and the slice defers promoting it to a spec until
  after v1 ships.
- **ADRs** — none exist. `doctrine adr list` empty ✓.
- **Policies / standards** — none exist. `doctrine policy list`, `doctrine
  standard list` empty ✓.
- **Doctrine framework rules** — bind this repository's *process*, not smut's
  *behaviour*. They cannot decide a JSON envelope. The brief additionally
  forbids importing that project's machinery into the tool.
- **`.doctrine/project-orientation.md`** — still the install scaffold, every
  section an unfilled HTML comment ✓. No project principles to inherit.
- **`.doctrine/doctrine.toml`** — `[install]` plus commented defaults ✓. Nothing
  steers conduct, dispatch or review for this repo.
- **Backlog** — empty ✓. No latent work items bearing on this slice.

**Revision candidates**

None — the governing picture is empty rather than stale. The consequence is
recorded under Design-input deltas: this design creates the project's first
binding choices, and they should be recorded as design-run decisions (DEC)
rather than invented as ADRs mid-design.

## Thread 2 — substrate and prior-art map

Greenfield repository: `src/main.rs` prints `Hello, world!`, `Cargo.toml` declares
zero dependencies, edition 2024 ✓. There is therefore no hotspot map; the
equivalent is the mechanism map for the decisions the brief delegates to the
implementer.

**Change surface** — the slice's selectors already name it: `src/**`,
`tests/**`, `Cargo.toml`, `README.md`, `docs/**`, `justfile` ✓.

**Crate candidates**, verified on crates.io this session ✓:

| Crate | Version | Note |
| --- | --- | --- |
| `serde` | 1.0.229 | derive for the few typed structs (index, journal) |
| `serde_json` | 1.0.154 | `Value` is the entity representation; MSRV 1.71 |
| `nanoid` | 0.5.0 | generated public IDs |
| `fs4` | 1.1.0 | `flock(2)` via rustix, no libc; MSRV 1.75 |
| `clap` | 4.6.7 | derive CLI, MSRV 1.85 |

**Cited facts (leaned on later)**

1. `serde_json::Value` **cannot** reject duplicate object keys — `Map` keeps the
   last one, so evidence of the duplicate is gone after parsing. Rejection must
   happen inside a custom `Deserialize` visitor that checks each key as
   `visit_map` yields it (serde-rs/json#1112).
2. `serde_json::Map` is a `BTreeMap` unless `preserve_order` is enabled, so
   `Value` equality is key-order-insensitive and serialisation is already key-
   sorted — both required by the brief's "compare structurally, not by object
   key order" and "deterministic serialization".
3. `clap` exits `2` on argument errors (`USAGE_CODE` in
   `clap_builder/src/util/mod.rs`), which is the brief's usage exit code.
4. A durable rename is temp-write → fsync file → rename → **fsync the parent
   directory**. File fsync alone does not make the name durable.
5. Recoverable multi-file commit has a stable prior-art shape: stage+fsync,
   journal+fsync, COMMITTING+fsync dir, apply, clear COMMITTING. Every crash
   instant lands in pre-commit (discard) or post-commit (roll forward).
   (`reldir::transaction` docs; `fs-transaction` journal.)
6. Mermaid quotes labels; an embedded `"` is the entity code `#quot;`.
7. `CARGO_BIN_EXE_<name>` lets integration tests execute the built binary — the
   route for concurrency and end-to-end tests.

**Rejected**

- `assay_evidence::json_strict` (strict `Value` parsing with duplicate-key
  rejection) — an unrelated product's crate; not worth a dependency for a
  behaviour that is ~40 lines of visitor.
- An atomic-write crate (`atomicwrites`, `atomic_replace`, `safe-write`) — each
  gives single-file atomicity, which the brief explicitly says is *not* batch
  atomicity. The journal must be ours.

## Cross-thread findings

- **Governance is empty, so the brief is the whole canon.** Nothing external can
  arbitrate a design disagreement here; every load-bearing choice needs a
  recorded rationale, because the next person has no other authority to consult.
- **The brief's two hardest clauses are the two with no crate to buy.** Batch
  recoverability (item 5) and duplicate-key rejection (item 1) are both
  implementable in-house in tens of lines, and both are exactly where the
  mandated adversarial tests point. That is the shape of the work: small code,
  sharp edges.
- **Structural equality has a sharp edge the brief does not mention.**
  `serde_json::Number` distinguishes `1` (u64) from `1.0` (f64). "Structural
  equality" therefore already answers differently from a naive numeric
  comparison. It must be pinned by test and documented, not left to discovery.
- **The prior art and the brief agree on the durability claim, and both refuse
  the stronger one.** Individual renames are not batch-atomic on any real
  filesystem; the honest guarantee is *recoverable*, with a documented
  post-commit/no-response ambiguity. Any design prose claiming atomicity would
  be false.

## Design-input deltas

1. The parse edge is a real component, not a call to `serde_json::from_str`:
   duplicate-key rejection forces a custom recursive visitor, and it lives on the
   *input* side of the pure core's boundary.
2. Crate set fixed at `serde`, `serde_json`, `nanoid`, `fs4`, `clap`; no
   atomic-write crate. Hand-roll the journal against the brief's protocol so its
   boundaries stay pinnable by tests.
3. Durability language in the design and README: *recoverable batch commit with a
   documented ambiguity window* — never "atomic".
4. Ordering: an ordered map type plus a documented scalar type order, so
   determinism is structural rather than defended by discipline.
5. Mermaid escaping rules (`#quot;`, quoted labels, generated node symbols) are
   pinned in the design, not improvised in the renderer.
6. Test harness: integration tests spawn the binary via `CARGO_BIN_EXE_smut`;
   crash injection needs an explicit, documented seam (env-var abort at named
   protocol boundaries) because the brief mandates interruption testing.
