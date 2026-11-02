# The loader takes its roots as parameters

DEC-026 named three roots and DEC-028 bounded the walk-up at the git root, which raises the
question DEC-026 did not: what code decides which directory those roots are? The available
answers are the ones any process has — the current directory, the environment, or a flag —
and the choice matters twice over, because the same process already derives a *store* root
from the current directory under DEC-004.

## What decides

**The loader takes its roots as arguments.** It is a pure function over
`(project_root, user_root, embedded)`, and it reads nothing. Everything ambient is resolved
once, in the binary:

- **The project root** is the nearest ancestor of the current directory that holds a `.git`
  entry — a directory in a normal clone, a file in a worktree or submodule, and either is
  enough, because the question "is this a repository?" is the whole question. When there is
  no such ancestor the current directory is the project root and there is no walk-up.
  Nothing is shelled out to `git`: a PATH dependency and a subprocess are a real cost in a
  binary that has neither today.
- **The user root** is `$XDG_CONFIG_HOME` when it is set and non-empty, else `$HOME/.config`
  through the standard library. When neither resolves — a container with no home — the user
  layer is **absent**, not an error: the project and binary layers still answer, and a
  command that cannot run for want of a home directory would be a worse failure than a
  missing layer.

**No environment override ships with this.** DEC-026's choice mentions one and names no
surface; an override is what a monorepo or a dotfiles owner wants, and it is a
compatibility commitment once it exists. It is left recorded rather than invented — see
QUE-002.

## Where the read happens

`cli::parse` stays pure. It validates the flags and the shape of the request and returns
**the question** — which set was asked for, in which format — exactly as it returns a
`Request` for the commands that need a store. The binary then resolves that question
against the roots it has computed, next to where it already computes the store root, and
the answer becomes text.

So the shape mirrors the pipeline the crate already has: `parse` decides, `execute`
obtains, `answer` prints. `cli`'s unit tests keep needing no filesystem, and the one
impure read sits on the one line that owns impurity. The `Ask` type therefore holds names
rather than a borrowed `Set`, and `Answer` holds the resolved set or the refusal.

## Why parameters rather than ambient reads

Two reasons, and the second is the one that decided it.

The first is ordinary: a pure resolver is testable without a process, a temp directory or
an environment.

The second is that the suite *already* spawns the binary with the parent environment
inherited. If the loader read `$HOME` for itself, a developer with their own
`~/.config/smut/conventions/hygiene.md` would silently change what the tests observe, and a
clean container would not reproduce it. Passing the roots in makes that leak impossible
rather than unlikely.

## The two roots in one process

The store stays where DEC-004 put it: `<cwd>/.smut`, never searched upward. The sets now
walk up. They agree only when the caller runs at their repository's root, and in a
subdirectory `smut declare` writes a store beside the subdirectory while
`smut conventions` reads the repository's sets.

That is intended — a store is data you placed deliberately, and advice is policy that
belongs to the project — but it is a disagreement a reader can meet, so it is documented and
tested rather than smoothed. The tempting alternative, teaching the store to walk up as
well, would change what `declare` creates and is a different slice's decision.
