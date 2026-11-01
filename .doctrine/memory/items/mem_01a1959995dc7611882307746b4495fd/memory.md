# Review finding classes that keep recurring in smut

Observed across RV-001…RV-003 on SL-001 plus one independent plan review. Each is
a class, not an instance: when one is reported, sweep for its siblings.

1. **Stated obligation vs declared API.** The rule says generated ids avoid
   existing entity ids; the function signature has no snapshot to know them. A
   decision can be internally contradictory and still read well. Check that every
   rule a record states is expressible through the surface it declares.
2. **A guarantee untrusted content can absorb.** A completeness notice appended
   after verbatim bodies disappears the moment a body ends in an unclosed HTML
   comment or fence: disclosed in the bytes, hidden on the page. Put guaranteed
   notices where no stored byte precedes them, and state the guarantee's exact
   boundary instead of denying the residue.
3. **The recipe narrower than the claim above it.** The prose said "every staged
   byte"; the numbered recipe said "every staged entity file and manifest",
   silently dropping the staged index that every commit writes. Implementers
   follow the recipe. Compare a general claim against the concrete recipe under
   it, element by element.
4. **Cfg-gated test suites pass vacuously.** Injection code gated out of default
   builds means `cargo test` executes zero tests from that file while every gate
   stays green. Run the feature matrix in the gate and assert a non-zero
   executed-test count.
5. **Integration tests need a lib target.** Rust `tests/*.rs` cannot reach a
   binary-only crate except by spawning it. A design that puts durability or
   corruption layers under `tests/**` needs `src/lib.rs`; check the module map
   carries one before planning phases that depend on it.
6. **Criteria only polite input reaches.** "Both notices render when the
   selection is empty" is unmeetable when truncation implies at least one row.
   A criterion whose precondition the pipeline cannot produce gets either
   fabricated or flagged — both cost a round trip.
