# DEC-034: The human query surface is `smut q`

The brief's ten-key specification is the machine contract, and it is small and well shaped. What
makes it miserable at a prompt is the shell: every character it eats has to be paid for with a
quote, and `$eq`/`$in`/`$exists` are hostile inside quotes. So the human surface is a new grammar,
and the question was where to put it.

**Not on `query`.** A bare positional on `query` cannot be distinguished from a JSON operand —
`smut query open` is either a root id or a malformed specification — so a grammar bolted onto the
machine verb would have to guess. That is a back-compat trap as well as an ambiguity, and DEC-011
and DEC-019 require every existing call to keep both its meaning and its bytes.

**A new verb instead.** `smut q` is short enough for a hot path, unique among `declare`, `get`,
`delete`, `query`, `render`, `learn` and `conventions`, and it makes the division of labour legible:
`q` is the shortcut, `query` is the complete surface.

`smut query` therefore does not move — not its keys, not its refusals, not its compatibility
tests — and stays documented as the way to say what the DSL cannot. `find` was the discoverable
alternative and differs in nothing else, so the rename later is one subcommand variant plus guide
text: the door stays open until the guide ships.

## Amendment — 2026-11-03 (SL-011 reconcile, RV-037)

"`q` is the only human front-end and has no alias" is superseded in part by DEC-042. There are two
human verbs over one grammar: `q` answers the envelope, `r` prints the Markdown `render` prints, and
both take the same line through one compiler. Neither is an alias. `smut query` is unchanged and
stays the complete surface; `-j` now carries the whole specification on both human verbs.
