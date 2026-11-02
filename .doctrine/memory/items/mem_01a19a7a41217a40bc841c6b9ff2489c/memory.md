# Design-run reviewing mechanics

Hit while driving SL-005's review pass. Each cost a refused apply.

1. **Severe findings need a `route`.** `review dispose` accepts a disposition without
   one, but the design gate later refuses with "severe findings carry no route" —
   blocker and major findings must be given one of `review | demonstrate | probe |
   control | dedupe | refresh`. For a repair guarded by a pinned test, `control`.
2. **A routed finding must be reopened first.** `review amend` only works on an
   `answered` finding; once verified, the path is
   `reopen (raiser) → dispose --route (responder) → verify (raiser)`. Three calls per
   finding.
3. **Section review coverage comes from `att-` declarations**, not from the
   `section-reviewed` checkpoint act. Declaring `{subject: "att-N", attests: "sec-N",
   reviewer: "human"}` is what turns a section's `review=outstanding` into
   `review=current`; the act alone is refused ("its rule binds to per-section coverage
   and the act carries none").
4. **A reopen or raise clears the pass's concluded marker**, so `review conclude` must
   be called again afterwards — otherwise `review-disposed conducted` has nothing to
   point at.
5. **`review_pass RV-NNN STALE` in the design envelope is advisory.** It appears when
   the run's revision moves after the pass concluded (e.g. by runbook discharges), does
   not appear among the `forward` gate's unmet conditions, and does not block
   `review-disposed` or the stage advance. Prime the cache with `review prime` if the
   `cache` line is stale; do not reopen findings to "fix" the envelope line.

Also worth knowing: `design materialise` must be re-run after any section body change,
and a section whose body changes loses its attestation, so fix text before attesting.