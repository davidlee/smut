# ISS-005: A constant-false predicate `{"$not":{}}` empties the query at exit 0, indistinguishably from a genuine empty result

Found by the pre-close code review of SL-005 (RV-027 F-3's pass), 2026-11-02.

`{"$not":{}}` is `Not(And([]))` — the `$not` of a predicate object with no members.
`And([])` is true for every entity, so the negation is false for every entity, and the
query returns an empty result at **exit 0**. From the caller's side that is
indistinguishable from a query that legitimately matched nothing, which is exactly the
ambiguity DEC-027 set out to remove for field-level contradictions: *a condition that can
never hold is a typo, not a question.*

SL-005's rule does not reach it, because `contradiction` reads one field's condition list
and this shape carries no field. Design §7 names four silences (`$contains` unbounded,
anything outside demanded position, `$or` contents never analysed, cross-clause
contradiction); this is a fifth, and it is not named. That is the defect — not that the
rule is partial, but that the boundary it draws is undocumented here.

`{"$and":[]}` beside it is the dual and is the legitimate, documented spelling for
"match everything"; `{"$not":{}}` has no such defence, since no caller writes it
deliberately to select nothing (they would write `{"from":[]}`).

Pre-existing and unchanged by SL-005; the delta moved nothing here. Capture only.
