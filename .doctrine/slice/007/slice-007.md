# Short flags, and an edge-set spelling that is not a trick

## Context

Two open backlog items, the same two files, the same class of change: CLI
surface, no behaviour.

- **IMP-007** — `render`'s five display flags are long-only, and so is
  `declare --compact`. IMP-007's own note defers the letter choice to "a wider
  CLI surface review" if one arrives.
- **IMP-018** — a `where`-only query carries no edge set, because `traversal` is
  conditional on a traversal being requested (DEC-012). Supplying `"follow": []`
  yields the complete induced edge set for free — no expansion, no budget,
  `depths` uniformly `0` — but nothing in `learn` or the README joins the two
  facts, so the capability reads as a trick rather than a surface.

They are one slice because they are one decision: IMP-018 may add a sixth flag,
and choosing letters for five flags while a sixth is undecided produces a scheme
that gets re-cut. That is IMP-007's stated condition for pairing.

Ground facts, all measured on the current binary:

- `query` takes **no flags at all** today; `render` has five long-only;
  `declare` has `--compact`; `conventions` has `--format` (a different domain).
- The edge set is already delivered: `select::envelope`'s `fn edges` is
  result-scoped and walk-independent (it scans the full stored entity of every
  returned id and keeps a pair when the target is returned too). The walk's only
  role in `follow: []` is to make `traversal` exist.
- `learn` states the grammar (`:131`) and the README states the conditionality
  (`:224`); neither states the consequence.

The governing constraints are DEC-008 (clap derive grammar, `--` terminates
flags, the whole request is validated before any store access), DEC-009
(rendering), and ADR-001 — none of which moves. DEC-012 and DEC-019 pin the
envelope's conditionality and the 0.1 key set; this slice moves neither.

## Scope & Objectives

O1. **One short-flag scheme, chosen across the CLI and applied once.** Covers
`render`'s `--format`, `--heading-field`, `--heading-level`, `--show-ids`,
`--show-fields`, `declare`'s `--compact`, and O3's flag if it lands. A short form
parses to the **identical** `Invocation` as its long form; `--help` and `learn`
both list it; nothing about behaviour, output or exit codes changes.

O2. **The guide joins the two edge-set facts.** One passage in `learn`, with the
measured example: supplying `follow: []` asks for the induced subgraph of the
result without traversing; `edges` is cut by `limit` exactly as `objects` is;
`depths` are all `0`; no expansion and no budget is spent.

O3. **Decide the spelling**, and deliver whichever is decided — a CLI flag, or
prose alone. Named as an open question below rather than assumed here, because
the merge rule against an existing `follow` is a real design question, not a
formality.

O4. **README carries the external-renderer example.** The join in O2 is only
worth reading because a fifteen-line shell converter (`smut query … | jq … |
<renderer>`) is a legitimate alternative to a Rust formatter. The README example
is what makes that true in practice.

## Non-Goals

- `--format mermaid`, or any second output format (IMP-002).
- Any envelope change. DEC-012's conditionality and DEC-019's key-set pin stay
  exactly as they are; IMP-009 owns any successor.
- `learn` section selection (IMP-008).
- Any change to selection, traversal, ordering, limits or Markdown output, and
  any change to a response shape — `tests/compatibility.rs` must not move.
- Environment variables, config files, or aliases: the surface is flags only.
- `conventions --format` is not renamed, re-lettered or unified with `render`'s
  `--format`; sharing a long name across two subcommands with different domains
  is not a collision this slice resolves.

## Affected surface

`src/cli.rs` (derive attributes, `parse`, and the parsing tests it already
carries), `src/learn.rs` (command block, prose, and its content pins),
`README.md`, and `tests/readme_example.rs` if the README example is executed by
the existing harness. Possibly `tests/cli_contract.rs` for the equivalence and
`--`-termination pins. Nothing under `src/store/`, and no edit to
`src/select.rs`, `src/walk.rs` or `src/render_markdown.rs`.

## Risks & assumptions

- **R1 — the first flag on `query`, and the first flag that edits a request.**
  If O3 lands a flag, it is the first flag `query` has ever had, and the first
  flag whose job is to *amend the request document*. Composition with a request
  that already supplies `follow` or `backlinks` needs an explicit rule (refuse,
  merge, or override) rather than an emergent one.
- **R2 — a short flag nobody can remember is worse than the long one** (IMP-007's
  own test). `--show-fields` is the hard case (`-F` vs `-s`), and the two heading
  flags only read well as a pair or not at all.
- **R3 — `learn` is a bounded, hand-written guide with content-pin tests.** Two
  items both edit it; a new paragraph pressures both its length and its pins, and
  a pin that has to be relaxed to admit new prose is a pin that was doing work.
- **R4 — validate-before-store.** DEC-008 validates the whole request before any
  store access. Request sugar must be applied *before* validation, or a rejection
  can arrive after a lock and a recovery.
- **A1 — `--` termination is not weakened by short flags.** `smut get -- -weird-id`
  must keep working, and no short-flag parsing may reach past `--`. Verified by
  test, not by argument.
- **A2 — no consumer depends on `--format mermaid`'s refusal message.** It is
  untouched here; IMP-002 removes it.

## Open questions

- **OQ-1** — flag or prose for the edge set? If a flag: on `query`, on `render`,
  or both, and what is the composition rule against a supplied `follow`?
- **OQ-2** — the letter scheme: do the two heading flags pair (`-H`/`-L`) or not
  (`-t`/`-l`), and what does `--show-fields` take?
- **OQ-3** — does the short form belong in `learn`'s command block only, or in
  its worked examples too? (The examples are what a reader copies.)

## Verification / closure intent

- Every short flag pinned by an **equivalence** test — short form parses to the
  same `Invocation` as the long form — plus a `--`-termination test proving a
  short flag cannot be smuggled past `--`, and one that `get`/`delete` still take
  `-`-leading ids as literals.
- `learn` content pins for the short forms and for the edge-set join; `--help`
  pinned once.
- The README example is either executed by the existing harness or deliberately
  excluded with a comment saying why.
- `cargo fmt`, `cargo clippy` with zero warnings, full suite green;
  `tests/compatibility.rs` untouched — the proof that this is surface only.
- Both IMP-007 and IMP-018 resolve on close.

## Summary

Surface only: two spellings and one joined passage. The interesting decision is
O3 (whether a trick becomes a flag), not the letters.

## Closure — abandoned 2026-11-02, before design

The scope above is accurate and stays as the record of what was considered. The
lifecycle did not: this is six clap attributes, one possible flag and three
paragraphs, and the operator's direction (tier 1, this session) was that a change
of that size should not carry a design interview, adversarial review rounds, a
plan, phase sheets and an audit. Costed honestly, the ceremony was larger than
the change.

It is abandoned rather than shrunk because the two backlog items already carry
the intent, the constraints and the ground facts; the work proceeds directly
against IMP-007 and IMP-018 under TDD, fmt and clippy, with the equivalence pins
described above. Nothing here forecloses a slice if the letter scheme turns out
to need one.
