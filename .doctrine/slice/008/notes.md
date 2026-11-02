# SL-008 notes — layered conventions

## Design surface triage (exploring, 2026-11-02)

### Constraining governance

| authority | what it fixes here |
| --- | --- |
| DEC-026 | three roots, one relative suffix (`<root>/.config/smut/conventions/`), project root is the walk-up terminus, project beats user beats binary, same-named sets never merge, origin always printed, an environment override exists in the choice |
| DEC-028 | a set is one markdown file: filename = name, first `#` = title, flat `summary`/`version` frontmatter, derived keys refused by name, unknown keys ignored, unparseable line refused, a malformed file never breaks another layer, seed keeps `conventions-<name>` and gains `source`, sections are a grammar implemented later |
| DEC-029 | the word `conventions` spans command, leaf directory, seed `type` and id prefix |
| DEC-017 | the seed is an upsert under a fixed id reporting the replaced body under `previous`; a set never validates a store |
| DEC-004 | the store is `<cwd>/.smut`, never searched upward — the process has one store root, and it is the cwd |
| DEC-008 | flags validated before operands, all in step 1 |
| STD-001 | `docs/store-format.md` is the store format's only record; nothing lands inside `.smut/` |
| ADR-001 | identity is `smut_id`, reserved at every depth; a container-named id value keeps its own name |

**The two that actually bite.** DEC-004 and DEC-026 disagree about what "the project" is,
by design: the store never walks up, the sets do. And `docs/store-format.md:53-55` says
"the binary shipped today embeds its one set", which is true only until this lands —
STD-001 makes that file the format's only record, so the clause is amended in the landing
commit.

### Shaping decisions — settled before this run, not re-opened

DEC-026 / DEC-028 / DEC-029 on DEC-017, with IMP-013's brief as the work list. The design
run's job is the seven deltas in `research/research.md` § Design-input deltas, not the
mechanism.

### Open questions

| id | question | lean |
| --- | --- | --- |
| inq-1 | What does one resolved set carry, and what shape makes three layers one mechanism rather than three loaders? | one record per file: name, summary, version, source path, layer, body, and a loaded-or-reason outcome |
| inq-2 | How is the project root found — and what happens when there is none? | walk up for a `.git` entry (directory **or** file); terminate at the filesystem root as cwd, no shelling out to git |
| inq-3 | How is the user layer found without the loader reading the environment ambiently? | roots are parameters; only `main` reads `$XDG_CONFIG_HOME` / `$HOME` |
| inq-4 | What does a set that cannot be loaded exit with, and what does it print? | exit 1 with layer, path and reason; exit 2 stays for what the caller typed |
| inq-5 | What does the listing print now that one line per set becomes one line per file? | name, summary, layer, path, and a shadow/skip marker |
| inq-6 | Which directory entries count as set files? | `.md` only; anything else is not a set and is not reported as a failure |
| inq-7 | How does the seed express origin without breaking "the body is the file's bytes"? | `source` names the file; `version` is the author's for files, the crate's when shipped; prose stays byte-identical |
| inq-8 | Does the environment override ship in this slice? | no — defer, since test isolation is solved by parameters |
| inq-9 | How does the shipped set stay one code path with a file, and what happens to `Set`, `Ask` and `SETS`? | the `include_str!` bytes go through the same reader; `Ask` owns its set; `SETS` becomes the binary layer's contents |
| inq-10 | What proves the layering, given the suite must be rewritten? | discriminating assertions only: a fake home holding a real competing set, a worktree `.git` file, a corrupt project set — no assertion that no input can falsify |

### Risks

- **R1 — two roots in one process.** Intended, but it makes `smut conventions` and `smut
  declare` disagree about "the project" in a subdirectory. Test the disagreement.
- **R2 — step-1 I/O.** `conventions::resolve` runs inside `cli::parse` (DEC-004 step 1);
  the refusal channel for a bad set is not `Reject`'s two-value taxonomy today.
- **R3 — home is an ancestor of nearly every checkout**, so the walk-up terminus is
  load-bearing rather than cosmetic.
- **R4 — ambient environment in tests.** The helper inherits the parent environment
  (`tests/conventions.rs:81-90`); without isolation a developer's own `~/.config/smut/`
  silently changes the suite.
- **R5 — the suite rewrite can weaken evidence.** `tests/conventions.rs` is 582 lines
  pinned to the compiled-in shape, and the behaviour-preservation gate would normally
  call a rewritten seam suspect. This seam is deliberately changing, so the discipline
  moves to the *assertions*: each one must name the change that reddens it.
- **R6 — `.config` in a repository already means "a home directory"** (DEC-026 accepts
  this; origin printing is the cure, and it now has to land in the listing).

### Assumptions

- **A1** — no new dependency: frontmatter is two flat keys (hand-parsed) and
  `std::env::home_dir()` exists on the pinned toolchain (checked: rustc 1.100).
- **A2** — "is this a repository?" is the whole question; the `.git` entry's *contents*
  are never needed, so a `.git` file (worktree, submodule) is enough.
- **A3** — a set file is `<name>.md`; the name is the stem, case preserved as written.
- **A4** — text is the human channel and `--format json` the piped one; the json
  declaration's shape does not change beyond `source`.

## Harvest

Nothing yielded yet — the slice is in design. Collected at close.
