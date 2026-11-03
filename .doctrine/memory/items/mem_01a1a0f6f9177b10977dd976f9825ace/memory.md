Probed 2026-11-03 (RV-037) in a scratch crate on clap 4.6.7, the version `Cargo.lock` pins:

- Variant with its own `///` doc + flattened struct with `///` doc → the variant's doc wins; `--help` and `<verb> --help` unchanged.
- Variant with no doc + flattened struct with `///` doc → the struct's doc becomes the verb's about in both the verb list and `<verb> --help`.

So in smut, where every `Sub` variant carries a doc comment, a `///` on `SelectionArgs`/`DisplayArgs` would be inert; the `//` comments there are defensive. A new variant added without a doc comment would silently take the struct's sentence as its about. `src/cli.rs`'s help test pins the top-level about line (`text.lines().next()`), not each verb's.
