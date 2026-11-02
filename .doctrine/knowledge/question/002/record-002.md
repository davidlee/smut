# Which environment variable points at one config home

DEC-026's choice includes "an environment override points at one config deliberately", and
names neither the variable nor what it names. SL-008's design deliberately does not ship
one (DEC-032): the two callers who want it — a monorepo whose conventions live in one
shared checkout, and a dotfiles owner who keeps sets outside `~/.config` — can be served
later without breaking anything, and a variable is a compatibility commitment the moment
it exists.

Three surfaces would need deciding together:

1. **The name.** `SMUT_CONFIG_HOME` mirroring the XDG variable is the obvious candidate
   and collides with nothing in the current environment surface; `SMUT_HOME` and
   `SMUT_CONVENTIONS` are narrower and less obviously a root.
2. **What it names.** A *config home* (so `$SMUT_CONFIG_HOME/conventions/<name>.md`) keeps
   one suffix and stays structurally identical to the other two layers, which is DEC-026's
   whole point. A *set directory* would break that equivalence for one caller's
   convenience.
3. **Its rank.** Above the project layer, or between project and user? An override that
   loses to the project layer is what a monorepo wants; one that beats it is what a
   debugging session wants, and the two are different features.

Nothing here blocks SL-008, and no test needs it: the loader takes its roots as arguments
(DEC-032), so a caller that wants a different root gets one by construction.
