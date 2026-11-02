# A shared working tree: commit path-limited, never amend

`/workspace/smut` is worked by more than one agent thread at once, on one branch and one
index. What that cost, on SL-008:

- Another thread's commit swept an **untracked** file of mine into itself (a
  `.doctrine/slice/NNN-<slug>` symlink created by `doctrine slice new`), because that thread
  staged everything. My follow-up `git commit --amend` then amended *their* commit, which
  was content-identical (a no-op) but re-wrote a commit I did not author.
- `git status` at the start of a step is not durable information: `Cargo.toml`, `README.md`
  and `tests/readme_example.rs` were repeatedly modified by other threads mid-session.

Rules that follow:
1. **Always commit path-limited**: `git commit -m … -- <paths>`, never `git commit -a`, and
   never `git add -A`.
2. **Never `git commit --amend`** in a shared tree — the commit you would amend may not be
   yours, and the index may hold someone else's staging.
3. **Never `git reset` past your own commit** to "fix" history; a soft reset moves HEAD for
   every thread in the tree. If a commit is contaminated, say so and leave it.
4. Treat a modified file you did not touch as evidence of another live thread, not as
   drift to clean up.

`doctrine slice new` writes both `<NNN>/` and a `<NNN>-<slug>` symlink to it; the symlink is
tracked, so a path-limited commit needs it named as well.
