# Review RV-009 — design of SL-002

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Independent second pass over SL-002 against the implementation brief, current
reference scanning and selection code, DEC-013 through DEC-015, and RV-007.
Probe multi-root BFS, depth/budget boundaries, induced edges, predicate
disambiguation, composition completeness, convention consistency, and acceptance
coverage. RV-007 repairs are present; do not duplicate its findings. This is a
design/source review; counterexamples below are deductions from the specified
pipeline, not executions of a 0.2 implementation.

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->
