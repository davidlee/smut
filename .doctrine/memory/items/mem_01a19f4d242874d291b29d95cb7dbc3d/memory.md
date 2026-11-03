Measured on the pinned toolchain (`rustc 1.100.0-beta.7`) with a three-line probe:

    HOME=None                 home_dir=Some("/home/david")
    HOME=Some("")             home_dir=Some("/home/david")
    HOME=Some("/tmp/fakehome") home_dir=Some("/tmp/fakehome")

The call is stable (no `#[allow(deprecated)]` needed) and resolves in three steps:
`$HOME` when set **and non-empty**, else the passwd entry, else `None`.

# Consequences

- **Test hygiene.** A fixture that wants "no home" must *set* `HOME` to a scratch
  directory, never `env_remove("HOME")` — unsetting it reads the developer's real
  home, which is exactly the ambient-state flake a clean container does not catch.
  SL-008's spawn helper sets `HOME` and `XDG_CONFIG_HOME` explicitly for this reason
  (PHASE-04/EN-4).
- **Design prose.** "else `$HOME/.config`" is not the same rule as
  `home_dir() + "/.config"`: a container with no `$HOME` still gets a home from
  `/etc/passwd`, so a "container with no home has no user layer" promise is false
  unless the passwd lookup also fails. SL-008's design §2 promised the former while
  the binary did the latter (RV-035 F-1).
- **The same care applies to every home-derived path** (`XDG_CONFIG_HOME` is the
  exception — that one is read directly with `var_os`, so an empty value really is
  empty).
