# DEC-037: The grammar stays inside clap's derive

`smut q` puts arbitrary positional tokens and flags in any order, which nothing in this tree had
asked clap to do before. A twelve-case spike on the pinned clap 4.6.7 settled it: **a bare
`Vec<String>` positional carries the whole grammar with no `#[arg]` attributes at all**, including
flags interleaved *after* a positional has started, with source order preserved.

What made the spike worth running is not the answer but the alternatives. Three of the five
candidate attributes misparse **silently** — exit 0, a plausible struct, the wrong query:

| attribute | what it does |
| --- | --- |
| `trailing_var_arg` | every flag after the first token becomes a token: `q session --backlinks part-of --depth 2 --ids` parses with four flags inside `tokens` |
| `allow_hyphen_values` | the same swallowing, plus a literal `--` leaks into the tokens (`q a -- --ids` → `["a","--","--ids"]`), plus `-weird-id` becomes legal, deleting the guard `--` exists for |
| `last` / `value_terminator` | breaks `--`, or requires a literal one before any token |

So the rule is not "clap works" but "clap works *because nothing was added*": the minimal spelling
is also the only silent-free one.

Two grammar consequences follow, and are stated rather than discovered. A token directly after a
value-taking flag is that flag's value — `--follow session` binds `session` — so examples place
values where they read as values. And `smut q` with nothing at all is a valid parse — it is
`query {}`, which the specification already defines as legal — so this layer deliberately has
**no** no-query refusal. The earlier reading, that the parse being valid put such a refusal in
this layer, is superseded: the layer would own it, and the decision is not to have one (SL-010
design §3; RV-034 F-6, amended 2026-11-03).

## Amendment — 2026-11-03 (SL-011 reconcile, RV-037 F-3)

The bare positional's `-weird-id` guard is clap's, and holds only for a word led by a letter the
verb does not own. SL-011 narrowed it. `--` reaches a hyphen-led id on every verb, and nothing else
does. Without it, on `q` and `r` a `-j…` word is `-j`'s value and, unless it is JSON, is refused in
the JSON reader's voice (`malformed JSON: …`), which names neither the word nor `--`. On `r`, a word
clap reads as a cluster of `-s`/`-i` optionally ending in `-t<value>` is silently display flags,
exit 0 (`-title` is `-t itle`, `-site` is `-s -i -t e`), and a `-f…` or `-l…` word is refused as a
format or a heading level. Accepted, not fixed: recognising the word needs the argv inspection this
record rejects, and DEC-008/IMP-007 keep the letters. The guide teaches `--` for hyphen-led ids on
`r` (SL-011 design § The command line, plan C6).
