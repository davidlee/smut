# README examples — the `q` section, proposed for reconcile

**Status: proposal, not an edit.** `README.md` is operator-owned (CHR-001); this
file is material for `/reconcile`, with every transcript below run and verified
against the fixture in § Fixture. Nothing here is a claim read off the design.

## Where it goes, and what it must not do

- **Placement:** a new `## Querying, quickly` between `## Worked example`
  (README:129–177) and `## Commands` (README:178). The worked example already
  builds the world `smut` / `q-query` / "Query design", so the new section meets
  the reader's own entities a second time instead of inventing a new cast.
- **The `Commands` table is missing `smut q`.** Add the row beside
  `smut query`: *select entities from argv tokens; the same specification, no
  quoting.* The table currently reads `declare`, `get`, `delete`, `query`,
  `render`, `conventions`, `learn`.
- **One sentence in the specification-keys paragraph** (README:190–199):
  `q` compiles to that same specification, so both verbs obey one pipeline,
  one envelope and one set of exit codes.
- **Division of labour.** README *shows*; `smut learn` *teaches*. Do not restate
  the twelve flags, the banned-character list or the refusal census — `learn`
  owns them and the ten-key pin is a test there. The README section earns its
  space by being short enough to read in one screen.
- **Do not claim more than is true.** Byte identity holds for the traversing
  recipes and `--ids`; the filtering pair agrees on the *selected set*, not on
  order or bytes. State the strong claim where it is verified, not generally.

## Fixture

Three declarations in an empty directory reproduce every example exactly. It is
the README's own worked-example world, extended by two entities.

```console
$ smut declare '[{"smut_id":"smut","title":"Smut","body":"Persist conversational structure.","questions":[{"smut_id":"q-query"}]}]'
$ smut declare '[{"smut_id":"q-query","type":"question","status":"open","title":"Query design","body":"Keep selection small."},{"smut_id":"q-limit","type":"question","status":"closed","title":"Does the limit stop the walk?","body":"Discovery is bounded separately."},{"smut_id":"d-escape","type":"decision","title":"JSON stays the complete surface","body":"Grouping and structural operands stay in query."},{"smut_id":"t-tags","type":"task","status":"open","tags":["storage","cli"],"title":"Show the short form in the README"}]'
$ smut declare '[{"smut_id":"q-query","part-of":[{"smut_id":"smut"}]},{"smut_id":"q-limit","part-of":[{"smut_id":"smut"}]}]'
```

Six entities, three of them linked to `smut`, two carrying `status`. No quotes
anywhere in the three declarations except the JSON's own.

## The examples

One paragraph of lead-in, then the transcript. Each block keeps its `why` on one
line. The envelope is abbreviated with `| jq -c .objects` except in example 1,
which shows it whole once; say that once in prose rather than per example.

### 1. The point, in one line

```console
$ smut q status=open --ids
{"dangling":[],"missing":[],"objects":[{"smut_id":"q-query"},{"smut_id":"t-tags"}],"truncated":false,"truncation_reasons":[]}
```

The envelope is the same one `smut query` returns — `missing`, `dangling`,
`truncated`, `truncation_reasons` — because `q` is a front end, not a second
engine. Compare what the same read costs as JSON:

```console
$ smut query '{"where":{"status":"open"},"select":["smut_id"]}'
```

### 2. Substring

```console
$ smut q title~design --select smut_id,title
[{"smut_id":"q-query","title":"Query design"}]
```

`~` is substring on a string, member on an array — the design's two warts are
`learn`'s to state, not README's.

### 3. One of a list

```console
$ smut q type=question,decision --select smut_id,type
[{"smut_id":"d-escape","type":"decision"},{"smut_id":"q-limit","type":"question"},{"smut_id":"q-query","type":"question"}]
```

A comma list on `=` is `$in`. No quotes, no `$` spelling of the operator.

### 4. Present and absent

```console
$ smut q --has status --ids
[{"smut_id":"q-limit"},{"smut_id":"q-query"},{"smut_id":"t-tags"}]
$ smut q --missing status --ids
[{"smut_id":"d-escape"},{"smut_id":"smut"}]
```

`--has`/`--missing` are `$exists` true/false. The obvious spellings — `body?`,
`!status` — are shell-hostile, which is why the words are words.

### 5. Direction is the token you read first (the headline)

```console
$ smut q smut --follow questions --depth 1 --ids | jq -c .objects
[{"smut_id":"smut"},{"smut_id":"q-query"}]
$ smut q smut --backlinks questions --depth 1 --ids | jq -c .objects
[{"smut_id":"smut"}]
```

Same root, same field, opposite direction, and the difference is the second
word. This is the footgun `learn` calls out — buried mid-object in JSON,
impossible to misread here.

The full envelope for the first line, worth showing once, because a traversal
answers more than a list:

```console
$ smut q smut --follow questions --depth 1 --ids
{"dangling":[],"missing":[],"objects":[{"smut_id":"smut"},{"smut_id":"q-query"}],"traversal":{"depths":{"q-query":1,"smut":0},"edges":[{"field":"part-of","source":"q-query","target":"smut"},{"field":"questions","source":"smut","target":"q-query"}],"frontier":["q-query"]},"truncated":false,"truncation_reasons":[]}
```

`depths` says how far each returned entity was found; `edges` is the induced
edge set; `frontier` is what the depth bound left unexpanded — a boundary, not
a truncation.

### 6. Projection and order

```console
$ smut q type=question --select smut_id,title --order title:desc
[{"smut_id":"q-query","title":"Query design"},{"smut_id":"q-limit","title":"Does the limit stop the walk?"}]
```

`--order title:desc` is one token where JSON wants a two-key object; the
direction defaults to `asc`.

## The two limits, shown honestly

A section that only shows wins reads as a salesman. Two examples, each the JSON
that does what `q` cannot — and each says what `q` does instead, so nobody
discovers the boundary by getting a silent empty answer.

### 7. A structural operand (an object inside an array)

```console
$ smut q 'questions~{"smut_id":"q-query"}' --ids
{"dangling":[],"missing":[],"objects":[],"truncated":false,"truncation_reasons":[]}
```

Exit 0, nothing returned. The operand is not a JSON scalar, so the value rule
keeps it as text and it matches no array member — the failure is *silent*, which
is exactly why the escape matters. The correct spelling is `smut query`'s:

```console
$ smut query '{"where":{"questions":{"$contains":{"smut_id":"q-query"}}},"select":["smut_id","title"]}'
[{"smut_id":"smut","title":"Smut"}]
```

`$contains` on an array is structural member equality, so "who references this
entity through this field" is a predicate, not a walk.

### 8. Cross-field grouping

```console
$ smut query '{"where":{"$or":[{"type":"question","status":"open"},{"type":"decision"}]},"select":["smut_id","type"]}'
[{"smut_id":"d-escape","type":"decision"},{"smut_id":"q-query","type":"question"}]
```

`$or`, `$and` and `$not` have no short spelling — a comma list covers "one of a
list on one field", which is the common case, and everything else stays JSON.
One sentence, and the reader knows where the wall is before hitting it.

## Prose facts worth one sentence each

Each is pinned by a test; none needs a transcript.

- **Same bytes.** For the traversing and `--ids` recipes, `smut q <line>` and
  `smut query <json>` produce identical stdout. The filtering pair agrees on the
  selected set; ordering is the `order` key's business in both.
- **Same rules.** One store, one lock, one recovery, one envelope, the same exit
  codes; `q` validates before any store exists, exactly as `query` does.
- **`--depth` needs a direction**, and says so; the JSON caller may generate a
  depth it does not use, a human may not.
- **There is no unlimited `--limit`** — `smut get` is the unlimited path.
- **`~` is case-sensitive**, and the answer never says whether it matched a
  substring or an array member.
- **A field name no entity carries answers empty at exit 0.** The typo is
  silent, in both verbs; `learn` teaches it, README should not hide it.

## What to leave out

- The twelve-flag table, the token-shape table, the character veto list, the
  five operators and their exact semantics: `smut learn` prints all of it, and a
  second copy in the README is a second thing to drift.
- `--from=ID` and `--` belong in `learn`; README mentions "hostile ids" at most.
- Anything about `render`. Its grammar has not moved (`q` stops at the query
  verb), and its own examples are already in the worked example.
