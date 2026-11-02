# DEC-035: A bare value is a JSON scalar when it parses as one

The predicate layer compares structurally with no coercion, so somebody has to decide what a bare
token means. Both candidates fail; they fail differently, and that asymmetry decides it.

**Always-string** fails *silently* on the two shapes a typed store is used for: a boolean flag and
a numeric rank. `done=true` compares the string `"true"` against a stored boolean and answers empty
at exit 0 — the failure class this repository's usability work ranks first.

**Scalar-typing** fails only where a string happens to look like a scalar, and the field where that
bites is the identity key. That failure is rare, nameable, and escapable.

So a value that is a JSON scalar is that scalar: `true`, `false`, `null`, a number. Everything else
is the string it looks like — `v1.2`, `007`, `open,resolved`. A JSON number has no leading zero,
so `007` is a string; a comma list is a list, not a value.

The wart is documented where it bites. `smut_id` is identity (ADR-001), so a numeric-looking id
compared with `=` compares a number; `--ids`, `--from=ID` and `smut query` are the three ways out,
and the guide names them beside the rule rather than in a footnote.