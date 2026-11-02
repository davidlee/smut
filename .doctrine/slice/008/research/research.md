# SL-008 research — layered conventions

**Producers.** Both threads were run inline by the orchestrating agent on 2026-11-02:
the project defines no research agents, so this is the graceful-degradation path the
research skill names. `raw/governance.md` and `raw/code-map.md` hold the commands and
their output.

**Baseline.** `.doctrine/slice/008/research/baseline.toml`, stamped by
`doctrine slice research 008` against `slice-008.md` at commit `d3705e7`.

**Standing on accepted decisions.** The design is settled before this round: DEC-026
(where sets live), DEC-028 (what a set is), DEC-029 (the name), on DEC-017 (the seed).
This round does not re-open them; it collects what implementing them touches, and the
five places where two of them, or a decision and the code, do not quite meet.

## Verification legend

✓ verified by this agent — the cited site was read or grepped during the round.
Unmarked = cited, not independently checked.

---

## Thread 1 — governance applicability

### Binding, and load-bearing here

| authority | constraint | state |
| --- | --- | --- |
| DEC-026 | one relative suffix, three roots, project root is the **walk-up terminus** (git root; else cwd with no walk-up); project beats user beats binary; same-named sets never merge; origin always printed; an environment override exists in the choice, unnamed | ✓ |
| DEC-028 | one markdown file per set; filename = name; optional flat `summary`/`version` frontmatter; derived keys (`title`/`name`/`id`/`type`) refused by name; unknown keys ignored; unparseable line refused; a malformed file never breaks another layer; seed id stays `conventions-<name>`, gains `source`, `version` is the author's for files and the crate's when shipped; sections are a pinned grammar **implemented later** | ✓ |
| DEC-029 | the word `conventions` spans command, leaf directory, `type: "convention"` seed and the `conventions-<name>` id prefix | ✓ |
| DEC-017 | the seed is an upsert under a fixed id, and the reply reports the replaced body under `previous`; conventions never validate a store | ✓ |
| DEC-004 | the store is `<cwd>/.smut` and is **never searched for in a parent** — the process has exactly one store root, and it is the cwd | ✓ (`src/store/mod.rs:176`, `src/main.rs:98-101`) |
| DEC-008 | grammar order: flags are validated before operands, and all of it in step 1 | ✓ (`src/cli.rs:311`, `src/conventions.rs:76-79`) |
| STD-001 | `docs/store-format.md` is the store format's only record; conventions put **nothing** inside `.smut/` | ✓ (`docs/store-format.md:49-56`) |
| ADR-001 | identity is `smut_id`; a container-named id *value* keeps its own name | ✓ (already landed; the seed emits `smut_id` via `ENTITY_ID`, `src/conventions.rs:126-129`) |
| IMP-013 | the implementation brief: seven work items | ✓ |
| IMP-011 / RFC-001 | advice compiled into a binary reaches an author only if they run and read it; 0 of 3 briefed authors used the advised `type` | ✓ |

### Checked, not applicable

- **DEC-012 / DEC-008 amendment** — format names and future-format refusals belong to
  `render`; `conventions --format` keeps `text|json` only (`src/error.rs:151-155`).
- **IDE-004** (no edge attributes) — the seed gains `source` as an ordinary field, not
  an edge attribute; unaffected.
- **ISS-003** (pre-minted ids) — the seed's id is fixed by construction.
- **IMP-008** — owns the section selector and the `--with-conventions` flag; explicitly
  downstream, and DEC-028 says not to pin it before layers exist. Confirmed out.
- **CHR-001** — owns README prose; two paragraphs describe the compiled-in behaviour and
  are now stale by design, but the item is open and the file is another thread's.

### Revision candidates

1. **`docs/store-format.md:53-55`** — "the binary shipped today embeds its one set" is
   true only until this slice lands. STD-001 makes that file the store format's only
   record, so the sentence should be amended (one clause) in the landing commit rather
   than left to drift. ✓
2. **`src/learn.rs:250-253`** — "a set never validates, never selects and never touches
   a store" survives untouched; the sentence before it ("`smut conventions` prints
   them") gains the layer fact. Pinned by `src/learn.rs:484`, so the test moves with it. ✓

---

## Thread 2 — code map

### Hotspots

| site | fact | ✓ |
| --- | --- | --- |
| `src/conventions.rs:36-58` | `Set` is three `&'static str` fields; `HYGIENE` is a `const`; `SETS` is `[&Set; 1]` | ✓ |
| `src/conventions.rs:66-78` | `Ask` holds `&'static Set` — the lifetime exists precisely because sets are compiled in | ✓ |
| `src/conventions.rs:80-99` | `resolve(name, format)` is pure, returns `Result<Ask, Reject>`, validates the flag before the name | ✓ |
| `src/conventions.rs:100-116` | `answer` is infallible text; `listing` prints `name  summary` per set, one line | ✓ |
| `src/conventions.rs:124-139` | `seed` builds five fields; `version` is `env!("CARGO_PKG_VERSION")`; `source` does not exist yet | ✓ |
| `src/conventions.rs:147-165` | `title_of` derives the seed title from the prose's first `#` heading | ✓ |
| `src/cli.rs:44-45, 286-292, 314-317` | `Invocation::Conventions(Ask)`; `resolve` is called **inside `parse`**, i.e. DEC-004's step 1 | ✓ |
| `src/cli.rs:230-233` | `Cli` has no global options — there is no `-p/--path` root flag to reuse | ✓ |
| `src/main.rs:90-93` | only the `Request` arm constructs a `Store`; `conventions` is answered on the no-store chain | ✓ |
| `src/error.rs:144-155` | two convention refusals (`UnknownConvention{found,sets}`, `UnknownConventionFormat{format}`) — neither carries an origin or a path | ✓ |
| `src/error.rs:263-268` | `Reject` → exit 2; `Store` → exit 1. There is no third channel | ✓ |
| `src/learn.rs:48-51, 57` | the guide's grammar block and "Help, version, learn and conventions need no store" | ✓ |
| `tests/conventions.rs:29` | the test's `EMBEDDED` is `include_str!("../src/conventions/hygiene.md")` — the body asserted as byte-identical to what ships | ✓ |
| `tests/conventions.rs:125, 161, 187, 338` | four pinned behaviours: one listing line, prose verbatim, unknown name exit 2, no store created or read | ✓ |
| `tests/conventions.rs:81-90` | the binary is spawned with `current_dir(scratch)` and **the parent environment inherited** — no `HOME`/`XDG_CONFIG_HOME` isolation | ✓ |
| `src/store/mod.rs:960-961` | the only subprocess use is the `fault-injection` crate feature; nothing in the shipped path shells out | ✓ |

### Greps that returned nothing

- `grep -rn '\.git\b' --include=*.rs src/` — **no git-root detection exists** anywhere.
  DEC-026's terminus is new machinery. ✓
- `grep -rn 'home_dir\|XDG\|config_dir' --include=*.rs src/` — **no config-home reading
  exists**; the only `env::` uses are `temp_dir` in tests and the fault-injection
  variable. ✓

### Naming and shape precedents

- Roots are values, not ambients: `Store::in_directory(root)` takes the directory it was
  handed (`src/store/mod.rs:182-186`) and refuses to look upward. A `Set` loader shaped
  the same way is the precedent, not a `dirs`-style ambient lookup.
- The seed's fields are supplied as an array of pairs and hoisted by `ENTITY_ID`
  (`src/conventions.rs:124-139`) — gaining `source` is one more pair, not a restructure.
- Refusals carry their evidence (`sets`, `found`), and the message prints it
  (`src/error.rs:372-380`). An origin-bearing refusal is the same pattern with a path.

### Toolchain facts

- `rustc 1.100.0-beta.7`, edition 2024. `std::env::home_dir()` compiles and returns
  `Some("/home/david")` on this toolchain — no `dirs`/`home` crate is needed for the
  user layer. ✓
- No new dependency is required by DEC-028's frontmatter: two flat keys, hand-parsed. ✓

---

## Cross-thread findings

**1. Origin printing collides with byte-identity — the sharpest finding.** DEC-026's
cure for `.config`-in-a-repository is "origin printing *plus a set header naming its own
file*". But `src/conventions.rs:12-18` claims the prose, the seed body and the listing
are "one copy printed three ways and cannot drift", `tests/conventions.rs:29` asserts
the printed prose against `include_str!`, and the seed body is that string verbatim
(`src/conventions.rs:167-170`). An injected header makes printed prose ≠ the file an
author edits, and would make the seed's `body` ≠ its `source` file. Two of those three
truths must be chosen between — see delta 2.

**2. Test isolation and the environment override are the same mechanism.** R4 says the
loader must not read the environment ambiently; the test helper
(`tests/conventions.rs:81-90`) inherits the parent environment today. If roots are
parameters, the *binary* reads cwd for the project root and `$XDG_CONFIG_HOME`/`$HOME`
for the user layer, and tests control both by `current_dir` + `env_clear`/`env` — no
override variable is needed for testability. That closes OQ-1's justification for
shipping it now.

**3. Step 1 now touches the filesystem, and the exit taxonomy has two entries.** DEC-004
orders flags → input → store, and `conventions::resolve` runs in `parse`
(`src/cli.rs:314`). Reading advice is not reading the store, so the order holds — but a
set that fails to load is neither a usage error (the caller typed nothing wrong) nor a
store failure (`src/error.rs:263-268`). Something must carry it: a new `Store`-side or
third `Error` arm at exit 1, or a widened `Reject` at exit 2. Unstated, it will be
decided by whoever writes the first `?`.

**4. The project root is a second root in a process that has one.** `Store` is `<cwd>`
by DEC-004; the loader walks up to the git root by DEC-026. They are the same directory
only when the caller runs at the repository root. This is intended, but it makes
`smut conventions` and `smut declare` disagree about "the project" in a subdirectory —
and it means the binary needs *two* root derivations, one of which it never had.

**5. Git-root detection is new code with a non-obvious edge.** `.git` is a directory in
a normal clone and a **file** (`gitdir: …`) in a worktree or submodule; no production
code shells out today and the only subprocess use is feature-gated
(`src/store/mod.rs:960-961`). Shelling out to `git rev-parse --show-toplevel` would add
a PATH dependency to a binary that has none; walking up for a `.git` entry (either kind)
is dependency-free, testable with a temp directory, and is what DEC-026's wording
describes.

**6. The listing changes shape, not just content.** One line per *set* becomes one line
per *file*, because a shadowed set must remain visible (DEC-026). The current format is
pinned at `tests/conventions.rs:125` and printed by `listing()` at
`src/conventions.rs:110-116`; "shadowed" needs a column or a suffix, and the shape has
to be decided here rather than invented per test.

**7. A directory holds files, and DEC-028 defines a set by filename.** The loader must
decide what a non-`.md` entry means: a `README.md` in the directory is a set named
`README`; a `.DS_Store` is not a markdown file at all. Ignored, refused, or listed as a
non-set — undecided, and visible to a user who keeps notes in that directory.

**8. Research artefacts are not gitignored in this repo.** `.gitignore`'s doctrine block
ignores `.doctrine/slice/*/phases` and `.doctrine/**/handover.md`, not
`.doctrine/slice/*/research/`; `git check-ignore` returns nothing for
`research/baseline.toml`. Recorded as friction, not acted on here — this slice leaves
the folder untracked.

---

## Design-input deltas

1. **OQ-1 (environment override) — defer, with a reason now.** Finding 2 removes the
   testability argument. Resolve at design as "not in this slice", and record the
   deferred surface so a later monorepo need has a home.
2. **OQ-2 (origin in the prose) — resolve to: printed prose is the file's bytes.**
   Finding 1 makes the alternative cost a shipped invariant (`VA-1`'s "one copy printed
   twice", the seed body's equality with its `source`). Origin is therefore printed by
   the listing and by the shadow notice, never by rewriting the body.
3. **OQ-3 (shadowing channel) — narrows to a format question.** Given delta 2, the
   channel is the listing; what stays open is the row shape (finding 6).
4. **New question — what a load failure exits with** (finding 3). Design must name the
   channel explicitly; the default is exit 1 with layer, path and reason, leaving exit 2
   for what the caller typed.
5. **New question — which directory entries are sets** (finding 7).
6. **New obligation — a git-root helper** (finding 5), with its own tests for a `.git`
   directory, a `.git` file, a nested repository and a directory under no repository.
7. **Confirmed in place:** the `include_str!` stays (reviewable bytes in the tree), the
   seed emits `smut_id` through `ENTITY_ID`, `Ask` loses its lifetime, and the shipped
   set is parsed by the same reader as a file rather than special-cased.
