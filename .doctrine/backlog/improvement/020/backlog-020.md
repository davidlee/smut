# IMP-020: A skipped unreadable-layer row reports as passing at the gate, so the skip is not evidence

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found as RV-032 F-6 during the `SL-008` PHASE-03 review, and deliberately not fixed in that
phase — the remedy lives in the gate's test recipe, not in any file the phase owned.

`EN-3` for PHASE-03 required that the unreadable-layer fixture not depend on the identity of
the process running the suite: a layer that exists but cannot be read as a directory is the
portable, root-proof primary row, and a `mode 000` directory is at most a *second* row, because
mode bits do not stop root. The criterion's wording asked that where the mode-000 row is
skipped, "the test says so in its own message".

It does — via `eprintln!` (`src/conventions.rs:1782-1815`). The default test harness captures
stdout and stderr and prints them only for a failing test, or with `--show-output`. So a gate
run as root reports the row as **passing** while asserting nothing, and the announcement that
was supposed to make the skip honest is invisible.

That is EN-3's letter without its purpose. The class is not unguarded — the sibling row
`an_unreadable_layer_is_an_error` carries it — so this is a visibility defect, not a coverage
hole. But a skip that reports as a pass is the same shape as an assertion that cannot fail, and
this repository already treats that as a defect worth removing.

Two remedies, in preference order:

1. Make the skip *fail* unless running as root — `assert!(is_root(), …)` in the skip branch, or
   a `nix`-free check of `geteuid()`. Then a non-root run that skips is a red test, and a root
   run's skip is announced by the root check itself rather than by a captured stream.
2. Add `--show-output` to the gate's test recipe if a flag-level remedy is preferred. Cheaper,
   but it only prints when someone reads the log, and it lengthens the gate's output.

Do 1 if it can be done without a dependency; 2 otherwise.
