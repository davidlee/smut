<!-- doctrine:section sec-1 -->
## The shape of a set, its layer, its origin

A **set** is loaded advice:

    Set { name, title, summary: Option<String>, version: Option<String>, body }

`name` is the filename stem for a file set and `hygiene` for the shipped one. `title` is
the file's first level-1 heading, verbatim, falling back to `name`. `summary` and `version`
are the frontmatter's two keys and are absent when the file omits them. `body` is the file's
text below the frontmatter — the bytes that print — with trailing newlines trimmed so the
shell's single newline is the only one.

An **entry** is one file's outcome:

    Entry { name, origin, outcome }
    Origin { layer: Layer, path: PathBuf }        // path is the file's own path
    Layer  { Project, User, Binary }
    Outcome { Loaded(Set), Failed(reason: String) }

The unit of storage and of listing is the **file**, not the name, because a name can exist
at three layers and a listing that dropped the losers would be lying about what is on disk.
Resolution is a view over the entry list, computed per name (`set_in_force`), and it is the
only place precedence appears.

`Layer` decides precedence and prints as `project`, `user`, `binary`. The binary layer's
`Origin.path` is the embedded file name, `conventions/<name>.md` — the same string every
layer reports as `source`, so the shipped set is described in the same terms as a file
without pretending to be one.

<!-- doctrine:section sec-2 -->
## Finding the roots

Three roots, resolved once, in the binary. The loader receives them.

**Project root.** From the current directory, walk up; the first ancestor holding a `.git`
entry is the root. A `.git` **directory** is a clone; a `.git` **file** (`gitdir: …`) is a
worktree or a submodule, and either answers the only question being asked — *is this a
repository?* When the walk reaches the filesystem root without one, the project root is the
current directory itself and there is no walk-up (DEC-028).

    project_root(start: &Path) -> PathBuf     // pure over the filesystem, no environment

Nothing shells out to `git`. The binary has no subprocess in its shipped path today, and a
PATH dependency plus a process spawn to answer a `stat`-shaped question is a poor trade. A
broken `.git` entry is treated as absent; a `.git` entry that is neither file nor directory
is not a repository.

**User root.** `$XDG_CONFIG_HOME` when set and non-empty, else `$HOME/.config`; a config
home yields `<config-home>/smut/`. When neither variable resolves — a container with no
home — there is no user root and the user layer is simply absent. That is not an error: the
project and binary layers still answer, and failing a read-only command for want of a home
directory would be a worse outcome than a layer that is not there.

    user_root(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf>

Both functions are pure over values handed to them. **`main` is the only code that reads the
environment**, which is what keeps the suite from inheriting a developer's real
`~/.config/smut/` (DEC-032).

**Two roots in one process.** The store stays at `<cwd>/.smut` (DEC-004) while the sets walk
up, so a cwd inside a repository and not at its root reads the repository's sets and writes
its own store. That disagreement is intended — a store is data placed deliberately, advice
is policy belonging to the project — and it is tested rather than documented away.

<!-- doctrine:section sec-3 -->
## Reading one set file

A **candidate** in `<layer>/smut/conventions/` is a regular file whose extension is `.md` and
whose name does not begin with a dot. Directories, other extensions and dotfiles are not
sets: they are not listed and not reported, because a directory the user controls should not
produce noise. A symlinked `.md` resolves through the link and is an ordinary set.

The name is the stem, exactly as written — case preserved. Name resolution is exact-match;
two files differing only by case are two different sets, which is what a case-sensitive
filesystem means and is not worth smoothing.

**Frontmatter** is optional, delimited, and flat:

    ---
    summary: general structure and relations
    version: 1.2
    ---

    # General hygiene
    …

| key | class | behaviour |
| --- | --- | --- |
| `summary` | known | becomes the set's summary (the listing's third column) |
| `version` | known | becomes the set's version (the seed's `version`) |
| `title`, `name`, `id`, `type` | derived | **refused by name**, with the message saying where each fact comes from |
| anything else | unknown | **ignored**, so a set written for a later smut still loads |
| a line that is not `key: value` | unparseable | refused, naming the line |
| an opening `---` with no closing `---` | unterminated | refused |

Frontmatter is never printed as prose, never addressable and never seeded into a body.

**Title and body.** The first line beginning `# ` is the title, verbatim, falling back to the
name. The body is everything after the frontmatter, with trailing newlines trimmed: what
prints is the body plus the shell's one newline. Nothing transforms an author's title — the
shipped set's own heading reads `# Conventions: general hygiene` so the prefix a reader sees
in a store is the file's choice, not a rewrite in code.

**The shipped set** is read by the same function over the same bytes, embedded with
`include_str!`. Its `summary` comes from its own frontmatter, and its `version` is injected
from the crate, which is what DEC-028 says the shipped set's version is.

**Failure is a reasoned row, never a panic.** Five ways a file lets the reader down — cannot
be opened, cannot be read, is not UTF-8, has an unterminated fence, has an unparseable or
derived key — all become `Outcome::Failed` carrying one line that names the problem and, where
there is one, the line number. The store loader's discipline (fail clean and loud, SL-004) is
the precedent: an unreadable advice file must not take the process down.

<!-- doctrine:section sec-4 -->
## Which set is in force

Resolution is by name over the entry list, in layer order: **project, then user, then
binary**. The first entry for a name is the set in force; everything after it is shadowed.

**Validity is not a tie-breaker.** A project `hygiene.md` that fails to load still wins the
name `hygiene`; the user's working copy is not silently promoted. The alternative — falling
through to the next layer when a nearer file is broken — makes the advice a caller receives
depend on a parse outcome they cannot see, and turns "which file is broken" into "which
file is in force", two questions that must stay apart (DEC-031).

The consequence is deliberate and must be visible: with a corrupt project set and a good user
set, asking for `hygiene` refuses. The refusal names the project file, its path and its
reason, which is the whole diagnosis; the user's copy is still listed as `shadowed`, so
nothing about the state is hidden.

**A shadowed set is unaddressable.** Resolution stops at the winner, and the seed's id is
`conventions-<name>` with no layer (DEC-017), so there is no command that prints or seeds the
loser. The workaround is the listing's path: read the file. Layer-qualified names would have
to change the fixed id and are their own decision.

**An unknown name** is any name no entry carries. The refusal lists every resolved name with
its origin, failed entries included with their reason, in the listing's order — the refusal is
the listing, said in one sentence.

**A name whose winner failed** is not unknown: it is broken. Exit 1, with the reason. The
line between the two is DEC-033's: what the caller typed is theirs to fix, what the
filesystem supplied is not.

<!-- doctrine:section sec-5 -->
## What each command prints

**The listing** — `smut conventions` — is one row per *file*, layer-major, then by name
within a layer:

    hygiene  project  general structure and relations  /repo/.config/smut/conventions/hygiene.md
    hygiene  user     shadowed                        /home/d/.config/smut/conventions/hygiene.md
    mine     user     my own house rules              /home/d/.config/smut/conventions/mine.md
    hygiene  binary   general structure and relations  conventions/hygiene.md

Four fields, two spaces between them, one row per line. The third field is the summary when
the file loaded and is in no shadow; `shadowed` when a nearer layer holds the same name; and
`did not load: <reason>` when it failed. The fourth is the file's own path — absolute for a
file layer, because a row's job is to let a reader open it, and `conventions/<name>.md` for
the binary, because that is what "its own path" means for embedded bytes.

Layer-major ordering is the precedence story written down: the first row for any name is the
advice in force. The listing **exits 0** even when it reports a broken file: a report whose
content is a problem is still a successful report, the same way a query's reported missing
ids are (DEC-033).

**The prose** — `smut conventions <name>` — is the body and nothing else: no banner, no
heading injected, no footer, no frontmatter. A caller piping it anywhere gets the file's own
text. It is *not* a round trip: redirecting it back into a set file loses the frontmatter, and
that is accepted (DEC-031).

**The seed** — `smut conventions <name> --format json` — is one declaration:

    smut_id  conventions-<name>      fixed, layer-blind (DEC-017)
    type     "convention"
    version  the file's frontmatter version, or the crate's for the shipped set,
             or absent when a file declares none
    source   conventions/<name>.md   the file relative to its layer root
    title    the file's first heading, verbatim
    body     the file's bytes below the frontmatter, trailing newlines trimmed

`smut_id` prints first and the rest lexically, as every declaration does. Declaring it is an
upsert: a second install reports the body it replaced under `previous`, unchanged from today.

**Exit codes.** 0 for success, including a listing that reports a broken file. 2 for what
the caller typed: an unknown set name, an unknown `--format`, `--format json` with no name.
1 for what the filesystem supplied: an unreadable layer directory, a file that cannot be
opened or decoded, unparseable or derived frontmatter — and asking by name for a set whose
winner failed. `smut learn`'s exit table gains the clause `or a conventions set could not be
loaded`.

<!-- doctrine:section sec-6 -->
## Code impact

**`src/conventions.rs`** is rewritten around the types in §1: `Layer`, `Origin`, `Set`,
`Entry`, `Outcome`, `Roots`, and the ships-one-set constant becoming a slice of
`(name, bytes)` pairs — today one entry, `("hygiene", include_str!("conventions/hygiene.md"))`.
The pure half:

    project_root(start: &Path) -> PathBuf
    user_root(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf>
    parse(bytes: &str, origin: Origin) -> Outcome          // frontmatter, title, body
    load(roots: &Roots) -> Vec<Entry>                      // the only I/O in the module
    set_in_force(entries: &[Entry], name: &str) -> Option<&Entry>
    listing(entries: &[Entry]) -> String
    seed(set: &Set) -> Value

`Ask` becomes the *question*, owned and resolvable without I/O: `List`, `Prose { name }`,
`Seed { name }`. `Answer` is either the text or the refusal, and `answer(roots, ask) -> Result<String, Error>` is the one entry point the binary calls. `HYGIENE`, `SETS`, the
`&'static` lifetime and `title_of`'s transformation all go.

**`src/cli.rs`** changes only in construction: `Sub::Conventions { name, format }` still
validates the flag before the operand (DEC-008) and now builds `Ask` from strings. `parse`
touches no filesystem, so its unit tests stay as they are.

**`src/main.rs`** gains the environment reads — `XDG_CONFIG_HOME`, `home_dir()` — and the
current directory it already computes, then calls the loader where it calls `Store::in_directory`:

    Invocation::Conventions(ask) => conventions::answer(&roots()?, ask)

**`src/error.rs`** gains one variant carrying the refusal (origin, path, optional line,
reason) whose `exit_code` is 1, so no diagnostic about a set claims a store. The existing
`UnknownConvention` keeps exit 2 and its shape, gaining origins in its rendered lists.

**`src/conventions/hygiene.md`** gains frontmatter with its `summary`, and its heading
becomes `# Conventions: general hygiene`; its body is otherwise untouched.

**`src/learn.rs`** gains the exit-table clause and one sentence in the Conventions section
saying a set is found in the project, then the user config home, then the binary.

**`tests/conventions.rs`** is rewritten: `Scratch` gains a repository marker and a config
home, and the spawn helper clears and sets the environment it needs instead of inheriting it.

**`docs/store-format.md`** gains one amended clause: the binary is no longer the only source.

<!-- doctrine:section sec-7 -->
## Verification

Each property names the test that can go red. Nothing here asserts an absence that no input
could falsify — the discipline is the memory `mem_01a19ac6c03b7da381fb6dfa6f137ceb` about
assertions occupying an evidence slot.

| property | the test that reddens |
| --- | --- |
| project beats user beats binary | the same name written with three different bodies at three layers; the project body prints |
| the user layer is really read | a set that exists *only* in the scratch config home resolves and prints |
| the walk-up happens | cwd = `repo/sub/dir/`, the set at `repo/.config/smut/conventions/`; it resolves |
| a worktree counts as a repository | `.git` as a *file* at the fake root; the set resolves |
| no repository means cwd, no walk-up | a scratch outside any repo whose own `.config/...` resolves, and whose parent's does not |
| a broken project set shadows a good user set | corrupt project file + valid user file → refusal at 1 naming the project path; the user row still says `shadowed` |
| the listing reports and succeeds | same fixture, listing exits 0 and carries `did not load:` with the reason |
| frontmatter is not prose | a file with `summary:`/`version:`; printed prose starts at the body's first line |
| derived keys are refused by name | `title:` in frontmatter → exit 1, message names the heading as its source |
| unknown keys are ignored | `future-key: x` loads normally |
| non-UTF-8 is a reasoned row | write invalid bytes; the listing names it and exits 0 |
| the seed is fixed and origin-bearing | `smut_id`, `source`, `version`, title verbatim, body equal to the file minus frontmatter |
| a second install still upserts | declare twice; `previous` reports the amended body |
| no store is created | pair the command with `get` in the same directory, which must fail for want of a store — the pairing is what discriminates |
| the two roots disagree on purpose | cwd in a subdirectory: `conventions` resolves the repo's set, `get` fails at exit 1 for the local store |

**Preserved behaviour** that must stay green without a rewrite: an unknown name at exit 2,
`--format json` with no name as a usage error, an unknown format refused before the operand,
and the seed's single-line output.

