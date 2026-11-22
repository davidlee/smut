# IDE-008: SMUT_PWD: an environment variable a harness sets once to run every smut command as if started in one directory

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred 2026-11-21 from IMP-022 (`smut -C <dir>`), at the operator's call.

A `SMUT_PWD` variable would let a harness set the start directory once (e.g. the
`env` block of Claude Code's `settings.json`), so no command carries `-C`.

It only pays when the store lives at one fixed place away from where the agent
works — one notebook store used from many repositories. When the store is in the
repository being worked on, the agent's starting directory is already right.
Claude Code does not persist environment variables between Bash calls, so the
variable is useless unless the harness sets it.

If built: it is the default `-C` falls back to (precedence `-C` > `SMUT_PWD` >
cwd), resolved in the same one place `-C` is, read only in the binary (DEC-032).
Decide alongside QUE-002 (the conventions config-home variable), so the two
environment surfaces are named as a pair. A variable is a compatibility
commitment the moment it exists.
