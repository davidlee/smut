# IDE-006: humane query DSL

# The diagnosis: JSON isn't the problem, quoting is

The specification is already small and well-shaped: ten keys, five operators, two traversal directions. What makes it miserable at a prompt is not its size. It's that **every character the shell eats has to be paid for with a quote**, and the shell eats almost all of JSON's punctuation:

```
smut query '{"where":{"type":"question","status":"open"},"select":["smut_id","title"]}'
            └──────────────── 14 quote characters, one $ away from a variable expansion ───────────────┘
```

Three separate costs:

1. **Quote-balance fragility in readline.** One missing `'` and you're in a continuation prompt; nested braces are edited blind. A query is not a document, but you type it like one.
2. **`$` is shell-active.** `$eq`, `$in`, `$exists` are hostile inside double quotes and impossible to type casually. The single most important thing `where` teaches is the thing hardest to type.
3. **Reading it back is work.** `follow` vs `backlinks` — the guide itself says direction is "what is easy to get wrong" — is buried mid-object. Direction should be the most legible token in the line, not the eleventh.

So the target is not "a nicer query language". It's **a surface with no quoting**, where the grammar is visible in the tokens.

---

# The reframe: argv is the DSL

The strongest design move available: **don't define a string grammar at all.** Define the grammar over argv tokens. Then a bare word is a token, a space is a separator, and the shell has already done the lexing. Zero quotes for the common query.

The invariant that keeps this safe:

> `parse(argv) -> Spec`. The DSL **compiles to the existing ten-key specification and adds no semantics.** Anything that cannot be expressed as a `Spec` is not a DSL feature; it's a bite at the spec, argued there.

That preserves DEC-011 (0.1/0.2 calls stay valid), keeps `smut query` as the complete machine surface forever, and makes the DSL testable by one property: every DSL line equals its JSON spec.

---

# The grammar

```
smut q [ID...] [CONDITION...] [FLAG...]
```

```
ID          a bare word — a root smut_id          session
CONDITION   FIELD OP VALUE                        status=open
  OP  =     $eq                                   status=open
      !=    $ne                                   status!=open
      ~     $contains   (substring or array member)  title~graph
VALUE       a comma list with `=` is $in          status=open,resolved
FLAG        one per spec key that isn't demoted   --depth 2
```

That's the whole surface. Two spec keys are *demoted* because they're the hot path, and everything else is *promoted* to a flag with the same name:

| spec key | DSL | why |
|---|---|---|
| `from` | **bare words** | roots are what you type first and most |
| `where` | **`field=value` tokens** | the `$` namespace disappears entirely |
| `follow` | `--follow F[,F]` / `--follow any` | same word as the guide |
| `backlinks` | `--backlinks F[,F]` / `--backlinks any` | same word as the guide; **visible next to `--follow`**, which is the direction footgun's real fix |
| `depth` | `--depth N` | |
| `budget` | `--budget N` | |
| `select` | `--select F[,F]`, `--ids` | `--ids` = `select:["smut_id"]`, the recipe's exact shape |
| `order` | `--order F[:asc\|:desc][,F…]` | `title:desc` |
| `limit` | `--limit N` | |
| `edges` | `--edges` | |
| `"*"` | the word `any` | see shell-safety below |
| — | `--has F[,F]` / `--missing F[,F]` | `$exists` true/false, shell-safe |
| — | `--from=ID` | escape hatch for an id carrying `=`, `~` or a leading `-` |

**The one rule to learn:** *bare = a thing, `field=value` = a filter, `--word` = structure.* Hot path has no flags; structure is spelled out and thus readable.

---

# Worked examples — the guide's own queries

| today | as typed |
|---|---|
| `smut query '{"where":{"title":{"$contains":"graph"}}}'` (54 ch) | `smut q title~graph` (**18**) |
| `smut query '{"where":{"smut_id":{"$in":IDS},"status":"open"},"select":["smut_id"]}'` (83) | `smut q IDS status=open --ids` (**28**) |
| `smut query '{"from":["session"],"backlinks":["part-of"],"depth":2,"select":["smut_id"]}'` (88) | `smut q session --backlinks part-of --depth 2 --ids` (**50**) |
| `smut query '{"where":{"type":"question","status":"open"},"select":["smut_id","title"]}'` (87) | `smut q type=question status=open --select smut_id,title` (**55**) |
| `smut query '{"where":{"body":{"$exists":true}},"order":[{"field":"title","direction":"desc"}]}'` (95) | `smut q --has body --order title:desc` (**36**) |

Quote characters in the left column: 8–16. Right column: **0**, always.

The two-call recipe becomes two lines a human can read aloud:

```console
$ smut q session --backlinks part-of --depth 2 --ids
["q-query","q-other"]
$ smut q q-query q-other status=open --ids
```

And the guide's deliberately-wrong direction stays wrong, but is now *visibly* different rather than a word buried at column 40:

```
smut q session --backlinks part-of --depth 2 --ids     # the questions under the session
smut q session --follow     part-of --depth 2 --ids    # the session, alone
```

---

# Rules that keep it honest

1. **Refuse ambiguity — never guess.** A token carrying two operators (`status~a=b`) is refused with the field named, not split at the first one. A field name containing `=`, `~`, `,` or a value containing a comma is out of the DSL's domain: the refusal says *use `smut query`, which is the complete surface*. Same for ids: `--from=ID` is the escape.
2. **No new semantics.** `status!=open,resolved` is refused because the spec has `$ne`, not `$nin`. `--limit all` is refused because the parsed `Limit` has no publicly reachable unlimited form (DEC-001 rule 4) — `get` is the unlimited path, not a flag.
3. **Duplicate fields are refused**, mirroring the JSON duplicate-key refusal: `status=open status=resolved` → *"two conditions on `status`; write `status=open,resolved` for either"*. Silent contradiction-as-empty is the failure mode to prevent.
4. **The human surface is allowed to be stricter than the machine surface — in strictness only, never in meaning.** `--depth 2` with no direction is accepted-and-ignored by the spec; `q` should refuse it: *"--depth needs a direction: add --follow, --backlinks, or drop it"*. A human typed it and meant it; the JSON caller may be generating keys.
5. **Every refusal names the fix and the sibling.** Unknown flag → nearest known flag. A JSON-looking positional (`q '{"where":…}'`) → *"`q` takes `field=value`; use `smut query` for a JSON spec"*. That single refusal is the whole migration story.
6. **Same envelope, same exit codes, same store rules.** `q` is a front-end, not a second engine.

---

# Shell-safety is the design constraint (and it vetoes things)

The character set the DSL may use for *bare tokens* is exactly what bash doesn't touch:

**Banned, and what happens if you forget:** `*` (glob — `--follow *` silently becomes every file in `cwd`), `?` (`body?` single-char glob), `[` `]` (glob class), `$` (`$eq` → empty string), `!` at word start (history expansion), `( )` (syntax error), `| & ; < >` (pipe/redirect), `#` at word start (comment), `~` at word start (tilde expansion), backtick, backslash.

**Consequences for the design, all deliberate:**

- **No `*` wildcard.** The word is `any`. (`--follow any` = `"*"`.) This is the single biggest hidden win: the obvious spelling is the broken one.
- **No `?` exists operator.** `--has` / `--missing` instead.
- **No `:` as an infix operator.** It's shell-safe, but ids like `session:1` are plausible and `smut q session:1` would silently parse as *"field `session` contains `1`"* — a silent wrong answer, the worst failure class. `~` is safer in ids and unambiguous. Shell-verify: `echo title~graph status!=open` prints both intact.
- **No `!field` negation.** `!` at word start is history expansion. Field-level negation is `field!=value`; predicate-level negation is JSON, which is where it belongs.
- **No grouping, no `or`.** `(` `)` need quoting, `|` needs quoting. Fortunately the most common disjunction is `$in`, which the comma list covers: `type=question,idea status=open`. Cross-field `$or`, `$not` of a group, `$and` of groups → `smut query`. This is also exactly the brief's "prefer two simple calls over expanding the query language."
- **`--` still ends flags**, so a hyphen-leading id remains reachable: `smut q -- -weird-id`.
- **Quoted values still work** (quotes are removed by the shell before smut sees them): `smut q title~"two words"` passes one token with a space — so values with spaces need one pair of quotes, which is unavoidable and correct.

---

# Open questions, with my recommendation

**1. Value typing — the one real wart.**
Bare values must decide between string and JSON type, and matching is structural with no coercion. My recommendation: **parse each value as a JSON scalar if it is one (`true`, `false`, `null`, a number), otherwise treat it as a string.** This makes `done=true`, `priority=3`, `status=null` (null is a real stored value here) work, and matches intent for typed fields. The cost: `smut_id=1` compares number 1. That's rare, and `--from=1` is the root escape, `smut query` the exactness escape. The alternative — bare always string — turns every numeric query into a silent empty result, which is worse. Either way: **document the wart, don't hide it.**

**2. Verb name.** Recommend **`smut q`**, with `smut query` keeping JSON forever. Extending `query` with flags makes `smut query open` ambiguous (JSON string? root id?), so a new verb buys a grammar with no back-compat trap. `smut find` is the discoverable alternative if one-letter verbs feel too terse.

**3. Bare roots vs. a sigil.** The chosen grammar is total for slug-like ids and needs `--from=` for ids carrying `=`, `~`, `!`. The total-grammar alternative is a condition sigil (`@status=open` / `+status=open`) that makes bare words *always* ids — unambiguously complete, but taxes the hot path with a keystroke and reads worse. I'd take bare conditions and the documented escape; the failure is loud enough (an empty result on a garbage field) and the escape is two characters. **Explicitly rejected:** resolving `a=b` as an id because it matches a stored id — that makes the grammar store-dependent, non-deterministic, and unauditable.

**4. `--where JSON` passthrough.** Tempting for "one complex predicate plus bare roots", but it reintroduces quoting into `q` and creates a half-language. Recommend **omit in v0**; the escape is one command away.

**5. Output is the other half of the pain.** `q` should print the same envelope (pipeable, machine contract). But a human reading 100 objects wants one line per hit. Recommend a `--brief` (one line per returned entity: `smut_id  type  title`, honouring `--select`), and/or steer the human to `smut render` — which should accept **the identical selection tokens** (`smut render --backlinks part-of --depth 2 session`) with its existing display flags. One selection grammar, two outputs (JSON vs Markdown) is a coherent pair, and it costs nothing because the parse already exists.

**6. Completion is the real endgame.** The grammar is designed so it *can* be completed: `smut q --<TAB>` completes flags; `smut q <TAB>` completes field names and, with a store read, **distinct values** (`status=<TAB>` → `open resolved`). Given the guide's own advice to keep the label vocabulary to a handful, value-completion is the highest-value follow-on — it's IDEA-003's census wearing an ergonomics hat. Not v0; note it so the grammar doesn't accidentally preclude it.

**7. A REPL** (`smut shell`) is a thin skin over the same grammar — the reason to design over argv is that the same tokens work as a line. Worth keeping possible; not worth shipping first. It buys nothing the shell doesn't already give you except a saved `smut` prefix.

---

# What I'd cut from v0

Nested paths, regex, `$or`/`$not` grouping, aggregation, computed fields, pipelines, unlimited limit, `--where` passthrough, Mermaid, and any operator the spec doesn't have. Each is a new semantics or a new quoting problem. The brief bans a general query language; the discipline that keeps this from becoming one is rule 2 above: **the DSL may only reduce keystrokes, never add meaning.**

---

**One line:** demote `from` to bare words and each `$operator` to an infix sigil over `=`, `!=`, `~`; promote every remaining key to a flag of the same name; ban every shell-active character from the grammar. Ten keys become eight flags and two demotions, the hot path has no quotes and no flags, and `smut query` stays the complete surface for the ten percent that needs JSON.
