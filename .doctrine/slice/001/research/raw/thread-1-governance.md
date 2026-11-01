# Thread 1 — governance applicability (verbatim notes)

Runner: orchestrating agent (degraded mode — no project research agents defined
in `AGENTS.md` / `.doctrine/project-orientation.md` / `.doctrine/doctrine.toml`).
Read-only. Date 2026-11-01.

## Commands run

    doctrine spec list      -> (no output)
    doctrine adr list       -> (no output)
    doctrine policy list    -> (no output)
    doctrine standard list  -> (no output)
    doctrine slice list     -> SL-001 design — smut v1 — persisted mutable entity graph CLI
    doctrine backlog list   -> (no output)

## Findings

1. No product spec, technical spec, ADR, policy or standard exists in this
   repository. The governance surface is empty; SL-001 is the only entity's
   work item.
2. The only binding sources in the repo are:
   - `AGENTS.md` (doctrine boot include; project conduct/process rules)
   - `smut-implementation-brief.org` (repository owner, dated 2026-11-01) —
     v1 intent and the authoritative statement of observable semantics
   - SL-001 scope doc, which defers to the brief explicitly
3. `AGENTS.md` carries no product/architecture rules — only process rules
   (TDD, lint, 2-space indent, DRY, no parallel implementation, commit scope).
4. The brief carries one governance-shaped constraint about governance itself:
   do not import the larger Doctrine project's machinery (no event sourcing,
   schemas, governance concepts, trust enforcement).
5. Prior art in-repo: none (greenfield: `src/main.rs` prints hello world,
   `Cargo.toml` has zero dependencies).

## Checked, not applicable (with reasons)

- Doctrine framework ADRs/policies/standards — govern *this repository's
  process*, not *smut's behaviour*; none bind the artifact's observable
  semantics. The brief is the product authority.
- `.doctrine/project-orientation.md` — still the install scaffold template
  (all sections are HTML comments). No project principles declared there.
- `.doctrine/doctrine.toml` — `[install]` and commented defaults only; nothing
  configuring conduct, dispatch or review for this repo.

## Revision candidates

None. Nothing in the governing picture is stale or in need of revision; the
picture is simply empty. Consequence recorded for the design: load-bearing
choices made in this design will be the first governance this project has, and
should be recorded as design-run decisions rather than invented as ADRs.
