# STD-003: Tests must pass on macOS CI — no non-UTF-8 filenames on disk, no unresolved temp-dir paths in assertions

## Statement

CI runs the suite on `ubuntu-latest` and `macos-latest` (`.github/workflows/ci.yml`).
Linux is the acceptance platform; macOS is portability evidence, but a red macOS job is
still a red build. Two Linux-only assumptions have already broken it. A test must not make
either:

1. **Do not create a file whose name is not UTF-8.** APFS (the macOS filesystem) refuses
   such names with `EILSEQ` (`Os { code: 92, "Illegal byte sequence" }`), so the fixture
   panics before reaching any assertion. A test that needs one is gated
   `#[cfg(all(unix, not(target_os = "macos")))]`, with a comment saying why.
2. **Do not compare against an unresolved temp path.** On macOS `std::env::temp_dir()` is
   under `/var`, a symlink to `/private/var`. The binary reports the resolved cwd, so an
   expected `/var/folders/…` never equals the printed `/private/var/folders/…`. Build
   fixture roots from a canonicalised path (`Scratch::new` in `tests/conventions.rs` does
   this); do not canonicalise only the expectation.

```
test builds path ──► /var/folders/…/scratch        (symlink)
binary resolves cwd ► /private/var/folders/…/scratch
assert_eq!(expected, actual)  ✗ on macOS, ✓ on Linux
```

## Rationale

Nothing local catches these: both pass on Linux, where agents develop. They surface only
after a push, as a red CI job. Each was fixed once (`2fdb479`, `a894e5c`); the same
patterns recur whenever a new test reads a scratch path or exercises byte-level filenames.

## Scope

- **Applies to:** every test under `src/**` (`#[cfg(test)]`) and `tests/`.
- **Not covered:** `flock`/`fsync` semantics on APFS. macOS CI proves nothing about them
  (`docs/store-format.md` § Portability).

## Verification

Mechanical, but only on CI: the macOS job. Before pushing a test that touches filenames
or reports a path, check it against the two rules above.

## References

- `.github/workflows/ci.yml` — the OS matrix and why macOS is there.
- `tests/conventions.rs` `Scratch::new` — the canonicalised scratch root.
- `src/conventions.rs` — the three non-UTF-8 filename tests and their gate.
