# The fact

Two different roots, two different rules:

| what | follows | set by |
|---|---|---|
| the store (`.smut/`) | the **process cwd** (DEC-004) | where you `cd` |
| the conventions layers | `HOME` / `XDG_CONFIG_HOME` (+ walk-up to a `.git`) | the environment |

SL-008's VA-2 transcript was meant to run "from a scratch directory" with a scratch `HOME` and
`XDG_CONFIG_HOME` exported — but the shell stayed in `/workspace/smut`, so `smut declare` created
`.smut/` in the repository root. `just check` and `git status` would not have caught it: `.smut`
is gitignored, and every test passes from the repository root because the fixtures do the `cd`
for themselves.

# The rule

- A hand-verified CLI transcript's fixture is **(cwd, env)**, both. `cd "$scratch"` in the same
  subshell that runs the command; exporting is half of it.
- After any hand run that declares or queries, check for a stray `.smut/` at the repository root
  (`ls -d .smut`) before trusting `git status` to be clean of it.
- Rust tests are already safe: the spawn helper sets the child's cwd. The trap is the transcript
  a human or agent types by hand.
