<!-- doctrine:section sec-1 -->
## The shape of a set, its layer, its origin

A **set** is loaded advice:

    Set { name, title, summary: Option<String>, version: Option<String>, body }

`name` is the filename stem for a file set and `hygiene` for the shipped one. `title` is the
file's first level-1 heading, verbatim, falling back to `name`. `summary` and `version` are
the frontmatter's two keys and are absent when the file omits them. `body` is the file's text
below the frontmatter with trailing newlines trimmed; printing adds back exactly one.

An **entry** is one file's outcome:

    Entry { name, origin, outcome }
    Origin { layer: Layer, path: PathBuf }        // the file's own path
    Layer  { Project, User, Binary }
    Outcome { Loaded(Set), Failed(Cause) }
    Cause  { line: Option<usize>, reason: String }

`Cause` is a type rather than a formatted string because the line number has to survive into
the refusal: the error variant carries the same `Cause`, so nothing re-parses a message to
find out where a file went wrong.

The unit of storage and of listing is the **file**, not the name, because a name can exist at
three layers and a listing that dropped the losers would misrepresent what is on disk.
Resolution is a view over the entry list, computed per name (`set_in_force`), and it is the
only place precedence appears.

`Layer` decides precedence and prints as `project`, `user`, `binary`. The binary layer's
`Origin.path` is `conventions/<name>.md` — the same string every layer reports as `source`,
so the shipped set is described in the same terms as a file without pretending to be one.

A file whose **name is not UTF-8** never becomes an `Entry` with a name: it is a failed row
whose reason says so. A `String` is never derived from a lossy conversion, and nothing in the
loader unwraps.

<!-- doctrine:section sec-2 -->
## Finding the roots

Three roots, resolved once, in the binary. The loader receives them and reads no environment.

**Project root.** From the current directory, walk up; the first ancestor whose `.git` entry
is a **directory** or a **regular file** is the root — a clone, or a worktree or submodule
(`.git` as a `gitdir:` file), and either answers the only question being asked: *is this a
repository?* Any other kind of `.git` entry (a dangling symlink, a socket, a broken link to
nothing) is not a repository and the walk continues. When the walk reaches the filesystem
root without one, the project root is the current directory and there is no walk-up
(DEC-028).

    project_root(start: &Path) -> PathBuf        // walks the filesystem, reads no environment

Nothing shells out to `git`: the binary has no subprocess in its shipped path today, and a
PATH dependency plus a process spawn to answer a `stat`-shaped question is a poor trade.

**User root.** `$XDG_CONFIG_HOME` when set and non-empty, else the home `std::env::home_dir()`
reports — `$HOME` when that is set and non-empty, otherwise the passwd entry. Either way the
layer root is `<config-home>/smut`; when no home resolves at all — no `$HOME`, no passwd entry —
there is no user root and the user layer is simply absent (DEC-032). Which variable answers is
the *binary's* question, not this function's: `user_root` is handed the result and joins
`smut` onto it. Not an error: the project and binary layers still answer, and failing a
read-only command for want of a home directory would be worse than a layer that is not there.

    user_root(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf>

**The two roots a layer is read from** are therefore the `…/smut` directories and nothing
deeper:

    Roots { project: PathBuf, user: Option<PathBuf> }

The declaration-kind segment is joined **exactly once**, inside `load` — `<root>/<kind>/` —
so no caller can join it twice and no root carries a kind.

The *compound* `.config/smut` segment, by contrast, is deliberately constructed twice, because
a config home already names `.config`: `Roots::under` joins it onto the repository root for the
project layer, and `user_root` builds it for the home fallback (`<home>/.config`, then `smut`).
"One join per layer root" is the rule; "one join for the segment" is not — and no caller of
`Roots::under` repeats it.

Both functions are pure over values handed to them. **The binary is the only code that reads
the environment**, and that is enforced by shape rather than by discipline: `execute` takes
the roots as a parameter, so the in-process unit tests pass their own and never see a
developer's `~/.config/smut/` (DEC-032).

**Two roots in one process.** The store stays at `<cwd>/.smut` (DEC-004) while the sets walk
up, so a cwd inside a repository and not at its root reads the repository's sets and writes
its own store. That disagreement is intended — a store is data placed deliberately, advice is
policy belonging to the project — and it is tested rather than documented away.

<!-- doctrine:section sec-3 -->
## Reading one set file

A **candidate** in `<root>/<kind>/` is a regular file whose extension is `.md` and whose name
does not begin with a dot. Directories, other extensions and dotfiles are not sets: not
listed, not reported, because a directory the user controls should not produce noise. A
symlinked `.md` resolves through the link and is an ordinary set.

A candidate whose metadata cannot be read — a dangling symlink ending in `.md` — is the *failed
row* below rather than nothing at all: the filter asks what a file is, and the failure table's
"cannot be opened or read" is the answer when the filesystem will not say. A directory whose
name is not UTF-8 stays silent, because the metadata test runs before the name is decoded and
a directory is not a set.

**Frontmatter** exists only when the file's **first line is exactly `---`** and a later line is
exactly `---`. Inside the fence, blank lines are skipped and every other line must be
`key: value`; outside it, `---` is a Markdown horizontal rule and not a fence at all. A key
written twice is the second value, silently: the frontmatter is a flat map, so a repeated
`summary` overwrites rather than refusing, and that is a named limitation rather than an
oversight — refusing a duplicate would need a message class DEC-028 does not carry. The fence
test is exact, so three shapes a Markdown author may reasonably write — `--- ` with a trailing
space, CRLF line endings, a leading blank line — demote the whole frontmatter to prose: the
file still loads, its `summary` is printed as advice, and nothing warns. Both limitations are
stated here rather than repaired, because repairing either changes the on-disk format.

| key | class | behaviour |
| --- | --- | --- |
| `summary` | known | the set's summary (the listing's third column) |
| `version` | known | the set's version (the seed's `version`) |
| `title`, `name`, `id`, `type` | derived | **refused by name**, each with its own sentence saying where that fact comes from |
| anything else | unknown | **ignored**, so a set written for a later smut still loads |

Frontmatter is never printed as prose, never addressable and never seeded into a body.

**Title and body.** The first line beginning `# ` is the title, verbatim, falling back to the
name. The body is everything after the frontmatter with trailing newlines trimmed. The trim is
deliberately asymmetric: trailing newlines go and a leading blank line after the fence stays,
so a set written the conventional way — the fence, a blank line, then the heading — prints and
seeds a body that opens with an empty line. Nothing
absorbs it, and §7's byte-for-byte row is what makes that deliberate rather than tolerated.
Nothing
transforms an author's title — the shipped set's own heading reads
`# Conventions: general hygiene`, so the prefix a reader sees in a store is the file's choice,
not a rewrite in code.

**Printing.** `answer` returns `body` followed by **exactly one `'\n'`**, and the binary writes
what it is given, verbatim — there is no shell newline to rely on. So
`smut conventions hygiene > hygiene.md` produces a file that ends in one newline; it loses the
frontmatter, which is the accepted trade (DEC-031).

**The shipped set** goes through the same reader over the same bytes, embedded with
`include_str!`. Its summary comes from its own frontmatter; its version is the crate's,
applied by `load` after `parse` because a byte reader knows nothing about releases.

**A failure is a reasoned row, never a panic.** The vocabulary is closed and small:

| class | reason |
| --- | --- |
| cannot be opened or read | the I/O error's own words, with the path |
| not UTF-8 | the file is not UTF-8 text |
| **empty body** | *a set that says nothing is a mistake, not advice* (DEC-028) — a frontmatter-only or empty file |
| malformed frontmatter | an unparseable line, an unterminated fence, or a derived key — naming the line where there is one |
| the filename is not UTF-8 | reported before any read, so no lossy `String` is ever produced |

The store loader's discipline (fail clean and loud, SL-004) is the precedent: unreadable
advice must never take the process down.

<!-- doctrine:section sec-4 -->
## Which set is in force

Resolution is by name over the entry list, in layer order: **project, then user, then binary**.
The first entry for a name is the set in force; everything after it is shadowed.

**Validity is not a tie-breaker.** A project `hygiene.md` that fails to load still wins the
name `hygiene`; the user's working copy is not silently promoted. The alternative — falling
through to the next layer when a nearer file is broken — makes the advice a caller receives
depend on a parse outcome they cannot see, and collapses "which file is broken" into "which
file is in force", two questions that must stay apart (DEC-031).

The consequence is deliberate and visible: with a corrupt project set and a good user set,
asking for `hygiene` refuses, naming the project file, its path and its reason — which is the
whole diagnosis — while the user's copy is still listed as `shadowed`. Nothing about the state
is hidden.

**A shadowed set is unaddressable.** Resolution stops at the winner, and the seed's id is
`conventions-<name>` with no layer (DEC-017), so no command prints or seeds the loser. The
workaround is the listing's path: read the file. Layer-qualified names would have to change
the fixed id and are their own decision.

**An unknown name** is a name no entry carries. Its refusal renders in the listing's order, one
clause per resolved *file* — the file is the unit of listing, so a name held at three layers
contributes three clauses — joined by `; `:

    unknown conventions set <name>; the sets are: hygiene (project); hygiene (user); mine (user, did not load: frontmatter line 2 is not a flat `key: value` line)

`name (layer)` for a loaded or shadowed set, `name (layer, did not load: <reason>)` for a
failed one — the same information as the listing, in the same order, so a caller can retry
from the message alone. An entry whose filename is not UTF-8 contributes no clause: the refusal
is a list of names to try, and the listing is where an undecodable file appears. Exit 2,
because the caller typed a name that resolves to nothing.

**A name whose winner failed** is not unknown: it is broken. Refused by name at exit 1 with the
path and reason. The line between the two is DEC-033's — what the caller typed is theirs to
fix, what the filesystem supplied is not.

<!-- doctrine:section sec-5 -->
## What each command prints

**The listing** — `smut conventions` — is one row per *file*, layer-major, then by name within
a layer:

    hygiene  project  general structure and relations  /repo/.config/smut/conventions/hygiene.md
    hygiene  user     shadowed                        /home/d/.config/smut/conventions/hygiene.md
    mine     user     -                               /home/d/.config/smut/conventions/mine.md
    hygiene  binary   shadowed                        conventions/hygiene.md

The binary row says `shadowed` here rather than printing its summary, because the third-field
rule below shadows *any* row whose name a nearer layer already holds, and the shipped set is
not exempt: the rule is about position, not about which layer supplied the bytes.

Four fields, two spaces between them, one row per line. The third field is:

- the **summary**, when the file loaded and is in no shadow;
- **`-`** when it loaded and is in force but declares no summary — the first set most people
  write is a bare markdown file, and it must have a row;
- **`shadowed`** when a nearer layer holds the same name;
- **`did not load: <reason>`** when it failed — and this **wins over `shadowed`**, so a broken
  user file behind a working project file still prints its reason rather than only its
  demotion (DEC-031 rule 5).

Those four states are the renderer's choice; the bytes are an author's. A summary is
interpolated verbatim, so a file declaring `summary: shadowed` renders a row shaped exactly
like a demotion, and a summary holding two spaces yields a five-field row. "No two states can
claim one row" is therefore a guarantee about the renderer, not about every line the listing
can print, and it stays a guarantee about the renderer: nothing is escaped or re-quoted,
because this is a report before it is a table, and an author's words reach the reader as
written (DEC-031 rule 3).

The fourth field is the file's own path: absolute for a file layer whose root is absolute —
the field is that root's join, and nothing rewrites a path to look absolute, so a relative
`$XDG_CONFIG_HOME` yields a relative row — and `conventions/<name>.md` for the binary, because
that is what "its own path" means for embedded bytes.

Layer-major ordering is the precedence story written down: the first row for any name is the
advice in force. The listing **exits 0** even when it reports a broken file — a report whose
content is a problem is still a successful report, as a query's reported missing ids are
(DEC-033).

**The prose** — `smut conventions <name>` — is the body followed by exactly one newline:
no banner, no injected heading, no footer, no frontmatter. A caller piping it anywhere gets the
file's own text. It is *not* a round trip: redirecting it into a set file loses the frontmatter,
and `smut learn` says so in one line — *prints the body; the frontmatter is metadata and is not
reproduced* — because losing a `summary` and a `version` silently is the silent-loss class this
repository ranks first (DEC-031).

**The seed** — `smut conventions <name> --format json` — is one declaration:

    smut_id  conventions-<name>      fixed, layer-blind (DEC-017)
    type     "convention"
    version  the file's frontmatter version, or the crate's for the shipped set,
             or absent when a file declares none
    source   conventions/<name>.md   the file relative to its layer root
    title    the file's first heading, verbatim
    body     the file's text below the frontmatter, trailing newlines trimmed

`smut_id` prints first and the rest lexically, as every declaration does. Declaring it is an
upsert: a second install reports the body it replaced under `previous`, unchanged from today.

An id of `conventions-<name>` and a `source` of `conventions/<name>.md` are the same at every
layer, by DEC-017's design — so a stored record cannot say which layer its advice came from.
That is deliberate, and the listing is the answer: it names the file in force, and the record is
the advice, not its provenance.

**Exit codes.** 0 for success, including a listing that reports a broken file. 2 for what the
caller typed: an unknown set name, an unknown `--format`, `--format json` with no name. 1 for
what the filesystem supplied: a layer directory that **exists and cannot be read**, a file that
cannot be opened or decoded, unparseable or derived frontmatter — and asking by name for a set
whose winner failed. A directory that is simply **absent** is not an error: it has no entries
and produces no row, which is the state of every project that has not opted in. `smut learn`'s
exit table gains the clause `or a conventions set could not be loaded`.

<!-- doctrine:section sec-6 -->
## Code impact

**`src/conventions.rs`** is rewritten around the types in §1: `Layer`, `Origin`, `Cause`,
`Set`, `Entry`, `Outcome`, `Roots`, with the ships-one-set constant becoming a slice of
`(name, bytes)` pairs — today one entry, `("hygiene", include_str!("conventions/hygiene.md"))`.
The split, with the signatures the plan will be held to:

    project_root(start: &Path) -> PathBuf
    user_root(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf>
    parse(bytes: &str, origin: Origin) -> Outcome          // pure: frontmatter, title, body
    load(roots: &Roots, kind: &str) -> Result<Vec<Entry>, Error>
    set_in_force(entries: &[Entry], name: &str) -> Option<&Entry>
    listing(entries: &[Entry]) -> String
    seed(set: &Set) -> Value

`load` owns every filesystem read *of a layer*, and everything the reader cannot know: the two
roots' `<kind>` directories are all it reads (`project_root`'s `.git` walk above is the binary's
root discovery, and `parse` reads bytes rather than files). It joins `<kind>` once, skips an
absent layer silently, sorts each layer's entries by name (a directory's order is arbitrary,
and the listing is the diagnostic) — an entry whose filename is not UTF-8 sorts last, and two
such entries order by their raw filename, so the order is total whatever `read_dir` returns —
applies the crate's version to the binary layer after `parse`, and concatenates the layers in
precedence order. It returns a `Result` because an existing-but-unreadable directory is exit 1,
and a failure class with no channel is a failure class that becomes silence. That class has no
`Origin`, because there is no file: it reuses `StoreError::FileSystem` with a `step` naming the
layer read (`reading the <kind> layer`), which keeps this section's promise that no diagnostic
about a set claims a store. `parse` stays a reader of bytes; a byte reader that knows release
numbers is not a reader. It takes no name parameter: the name is the file's stem, which
`origin` already carries, and a filename that is not UTF-8 never reaches it because §3 reports
that row before any read.

`Ask` becomes the *question*, owned and answerable without I/O: `List`, `Prose { name }`,
`Seed { name }`. `answer(roots, kind, ask) -> Result<String, Error>` is the single entry point
the binary calls. `HYGIENE`, `SETS`, the `&'static` lifetime and `title_of`'s transformation all
go.

**`src/cli.rs`** changes only in construction: `Sub::Conventions { name, format }` still
validates the flag before the operand (DEC-008) and builds `Ask` from strings. `parse` touches
no filesystem, so its unit tests stay as they are.

**`src/main.rs`** gains the environment reads — `XDG_CONFIG_HOME`, `home_dir()` — and the
current directory it already computes, and its `execute` takes the roots as a parameter:

    execute(arguments, stdin_is_terminal, stdin, &roots)   // main computes roots; tests pass theirs

so `execute` stays in-process testable without touching the developer's home, and the
`dispatch_of` unit test that today asserts on `conventions::HYGIENE` moves to the spawned suite.

**`src/error.rs`** gains one variant holding `Origin` and `Cause`, whose `exit_code` is 1, so no
diagnostic about a set claims a store. `UnknownConvention` keeps exit 2 and gains the
origin-bearing rendering of §4.

**`src/conventions/hygiene.md`** gains frontmatter with its `summary`, and its heading becomes
`# Conventions: general hygiene`; its body is otherwise untouched.

**`src/learn.rs`** gains the exit-table clause, one sentence in the Conventions section saying a
set is found in the project, then the user config home, then the binary, and the one line §5
requires saying a printed set does not reproduce its frontmatter. Its guide is pinned by tests,
so those move with it.

**`tests/conventions.rs`** is rewritten: `Scratch` gains a repository marker and a config home,
and the spawn helper clears and sets the environment it needs instead of inheriting it.

**`docs/store-format.md`** gains one amended clause: the binary is no longer the only source.

<!-- doctrine:section sec-7 -->
## Verification

Each property names the test that can go red. Nothing here asserts an absence no input could
falsify — the discipline is the memory `mem_01a19ac6c03b7da381fb6dfa6f137ceb` about assertions
occupying an evidence slot.

| property | the test that reddens |
| --- | --- |
| project beats user beats binary | the same name with three different bodies at three layers; the project body prints |
| the user layer is really read | a set that exists *only* in the scratch config home resolves and prints |
| the walk-up happens | cwd = `repo/sub/dir/`, the set at `repo/.config/smut/conventions/`; it resolves |
| a worktree counts as a repository | `.git` as a *file* at the fake root; the set resolves |
| **the walk stops at the repository** | a fake home holding `hygiene.md`, a repository inside it, `XDG_CONFIG_HOME` elsewhere, cwd in the repo: no row may report that set as `project` |
| no repository means cwd, no walk-up | a scratch outside any repo whose own `.config/…` resolves and whose parent's does not |
| a broken project set shadows a good user set | corrupt project file + valid user file → refusal at 1 naming the project path; the user row still says `shadowed` |
| **a failed file behind a working one still reports its reason** | valid project file + corrupt user file → the user row says `did not load:`, not `shadowed` |
| the listing reports and succeeds | the fixture above; listing exits 0 and carries `did not load:` with the reason |
| an absent layer is not an error | a scratch with no `.config/smut/conventions/` anywhere and a clean home: listing exits 0, one row |
| an unreadable layer is exit 1 | a `conventions/` directory with mode 000: exit 1, path named |
| **a set with no summary lists with `-`** | a bare markdown file, no frontmatter: row's third field is `-` |
| frontmatter is not prose | a file with `summary:`/`version:`; printed prose is `body` + one newline, byte for byte |
| **the body and the seed agree** | `prose == body + "\n"` and `seed["body"] == body`, asserted together |
| **all four derived keys are refused by name** | `title:`, `name:`, `id:`, `type:` each exit 1 with its own source sentence |
| unknown keys are ignored | `future-key: x` loads normally |
| an empty body is malformed | frontmatter only, and an empty file: both rows say a set that says nothing is a mistake |
| a non-UTF-8 file and a non-UTF-8 name | invalid bytes in the content, and in the filename: reasoned rows, no panic |
| the seed is fixed and origin-bearing | `smut_id`, `source`, `version`, title verbatim, body equal to the file minus frontmatter |
| **the shipped set is not special** | the same bytes as a file and as the embedded set resolve to equal entries apart from layer and origin |
| a second install still upserts | declare twice; `previous` reports the amended body |
| no store is created | `!scratch.store().exists()` after the command, *and* the paired `get` failing with its absent-store message — both, because the message is what discriminates a created-and-valid store from a created-and-corrupt one |
| the two roots disagree on purpose | cwd in a subdirectory: `conventions` resolves the repo's set, `get` fails at exit 1 for the local store |

**Preserved behaviour** that must stay green without a rewrite: an unknown name at exit 2,
`--format json` with no name as a usage error, an unknown format refused before the operand,
and the seed's single-line output.

