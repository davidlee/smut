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