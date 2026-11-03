# DEC-036: Roots are bare words, and `--from=ID` is the escape

The grammar has to separate a root id from a condition without consulting the store, and the
obvious alternatives are a sigil and a guess.

**A sigil** (`@field=value`) makes every bare word an id — total, unambiguous, honest. It also
taxes every hot-path line with a keystroke, and the benefit only appears where a slug-shaped id
collides with a field name.

**A guess** — resolve `a=b` as an id because such an id exists — is refused outright. It is
correct until the next `declare`, which makes the grammar non-deterministic, unauditable, and
dependent on state the caller cannot see.

So: bare means a thing, `field=value` means a filter. `--from=ID` reaches a root carrying `=`,
`~`, a comma or a leading `-`; `--` reaches a hyphen-leading id, as it already does for `get`. A
token containing an operator is never resolved as an id, whatever the store holds.

The cost is that a field name containing `=`/`~`/`,` and a value containing a comma are outside the
DSL's domain. They refuse, naming `-j` (amended 2026-11-03 by SL-011, DEC-045: they named `smut
query`) — which is the sentence that makes the front-end's
boundary a teaching moment rather than a wall.