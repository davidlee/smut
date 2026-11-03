# DEC-040: The guide teaches the DSL for the hot path and keeps `query` complete

`smut learn` is the delivery channel (DEC-010) and the normative home of the two-call recipe
(DEC-016). Changing what it prints is therefore a governance act, not an edit — which is why this
decision exists rather than being left to whoever writes the prose.

The recipes become `smut q` lines, because a recipe a reader retypes must be the one they will
actually type, and every quote in the JSON form is friction paid at the prompt.

`smut query` stays in the guide, described as the complete surface rather than the old way. That is
the sentence that makes the front-end's boundary actionable: when the DSL cannot say something —
grouping, a field name carrying `=`, a value carrying a comma — the reader has somewhere to go,
and the refusal told them where.

Both warts are stated where they bite. Value typing (`true`/`false`/`null`/numbers are scalars,
and `smut_id=1` compares a number) names its three escapes: `--ids`, `--from=ID`, `-j` (amended
2026-11-03 by SL-011: the third was `smut query`). And
`~` inherits `$contains`'s two meanings — substring on a string, exact member on an array — which
no front-end can fix and no guide may hide.

Two obligations follow. The `EXAMPLE_*` constants and the fixture harness gain a token-list path,
so the printed recipes remain executed evidence rather than prose. And DEC-016 is amended in place
at reconcile to name both spellings, the way SL-008 deferred its governance edits.

## Amendment — 2026-11-03 (SL-011 reconcile, RV-037)

The division of labour gains `r` and `-j`. The guide teaches the short form on two verbs — `q` for
the envelope, `r` for Markdown — and `-j` as the way to hand either the whole JSON specification.
`smut query` stays named as the complete surface; the escapes the guide and the refusals name are
`--ids`, `--from=ID` and `-j`.
