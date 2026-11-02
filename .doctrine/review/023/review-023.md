# Review RV-023 — design of SL-004

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Self-review of the six-section SL-004 design at run revision 34, after
materialisation at revision 32. This is the run-owned design review pass,
not an independent external review or a review of implemented code.

Probe consistency with ADR-001, STD-001 and DEC-020 through DEC-025; the ordinary
id/structural smut_id distinction; root versus nested generation; receipt and
projection spelling; shared predicate ownership; loader error classification;
diagnostic-path ambiguity; version checks before committed-journal replay;
recovery before normal validation; fixture equivalence independent of rewritten
expected outputs; and the code-impact/selector/documentation reconciliation map.

Compared the draft with current source in entity, declare, conventions, store
loading, index and journal parsing/recovery, and with the existing tests and
repository check recipe. The code-impact registry covers 44 tracked files after
glob expansion. The draft preserves unfinished 1.0 requirements and the separate
mint/rename/short-flag scope.

## Synthesis

Overall: acceptable for human design review.

No verified findings were raised in this self-review. The design makes the
remaining structural reservation and stale-client behavior explicit, preserves
the recovery protocol, and gives the new validation a precise owning layer and
failure shape. Verification separates new behavior, unchanged behavior, fixture
content, canonical bytes and no-replay evidence. This is a design assessment,
not implementation, runtime, crash-test or portability proof. The human section
reviews, final acceptance and choice of any additional adversarial pass remain
outstanding. No review waiver or final user acceptance was inferred.
