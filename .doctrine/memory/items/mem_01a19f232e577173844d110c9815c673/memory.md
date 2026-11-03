# What the row actually asks

SL-008 PHASE-04/VA-2: "record that `std::env::` appears in `src/main.rs` only inside `roots()`."
It appears six times:

| hit | module | property? |
|---|---|---|
| `XDG_CONFIG_HOME`, `HOME` reads | `roots()` | yes — the phase's property |
| `args_os()` | `main` | no — argv, not environment |
| `current_dir()` | `current_directory` | adjacent, deliberate |
| `temp_dir()` | a test helper | no |
| a doc comment mentioning `std::env` | — | no |

The honest report is the **classification** plus the load-bearing negative — `dispatch` and
`execute` hold no environment read — not "it appears only once", which is simply false.

# The rule

- Rewrite "X appears only in Y" as "every hit of X is classified; the modules the criterion cares
  about hold none". Then show the classification.
- A grep row whose stated literal is stricter than its truth is the same defect as an assertion no
  input can falsify: it cannot pass honestly, so it either gets quietly softened or reported as a
  failure that says nothing about the code.
- Prefer a mechanical discriminator (`rg -n 'std::env::' src/main.rs` output pasted) over a claim.
