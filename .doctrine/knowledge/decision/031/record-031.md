# The set in force is chosen by layer and named by its origin

DEC-026 settled where sets live and promised two cures for the confusion it accepted:
"origin printing plus a set header naming its own file". DEC-028 settled what a set is
and, in passing, made the set's bytes load-bearing: the seed's `body` is the file's own
text, so a store record and the file an author edits are the same bytes, and an amendment
made in the store shows up as `previous` rather than as drift.

Those two promises do not both fit. Injecting a header into printed prose makes the printed
set differ from the file it came from, makes the seed's `body` differ from its `source`,
and turns every future assertion of "one copy printed twice" into a suffix match.

This decision keeps the bytes and moves the origin.

## The rules

1. **Precedence is by layer, never by validity.** Project beats user beats binary. A
   project set that fails to load still wins its name; the user set it shadows is not
   silently promoted. Validity is not a tie-breaker, because a fallback that depends on
   the brokenness of a nearer file makes the advice a caller gets depend on a parse
   outcome they cannot see.
2. **A set is identified, for display and resolution, by its origin.** Every listing row
   names the layer and the file, so a caller can always answer "where did this advice
   come from" from the same command that gave them the advice. This is the display and
   resolution identity only: the *stored* id stays `conventions-<name>`, layer-blind
   (DEC-017), so a seeded record is the advice in force and not a record of one layer.
3. **The body is the file's bytes.** `smut conventions <name>` prints the body — the file
   without its frontmatter — and nothing else. No banner, no footer, no injected heading.
   The seed's `body` is that same string, so a seeded entity and its source file diff
   clean. The command is therefore **a printer, not a round-trip**: regenerating a set
   file from it loses the frontmatter, deliberately.
4. **A shadowed set stays visible and says so.** The row for a set that lost its name
   prints `shadowed` in place of its summary: its summary is not what anyone is getting.
   It is also **unaddressable** — resolution stops at the winner, so the loser's content
   is reachable only by opening the path the listing prints. Qualifying a name by layer
   would change the seed's fixed id, and is its own decision, not a consequence of this
   one.
5. **A failed set is a row, not a silence.** It prints its path and its one-line reason,
   and asking for it by name refuses with that reason (DEC-033 governs the exit code).
   *Failed* covers every way a file can let the reader down, including bytes that are not
   UTF-8: a set file the loader cannot read or decode is a row with a reason, never a
   panic. A symlinked `.md` file resolves through the link and is not a failure.

The listing is therefore `name · layer · summary-or-verdict · path`, one row per file, in
precedence order:

    hygiene  project  general structure and relations  /repo/.config/smut/conventions/hygiene.md
    hygiene  user     shadowed                        /home/d/.config/smut/conventions/hygiene.md
    hygiene  binary   general structure and relations  conventions/hygiene.md

A row's path is **the file's own path** — absolute for a file layer, because the row's job
is to let a reader open it. The seed's `source` is the other question ("which file inside
this layer"), so it is layer-relative and identical for all three layers:
`conventions/<name>.md`.

## Titles are the author's

The seed's `title` is the file's first level-1 heading, verbatim. Nothing lowercases it,
prefixes it or reflows it: an author who writes `# Team rules for the graph` gets exactly
that in the store. The shipped set's own heading carries the `Conventions:` prefix
(`# Conventions: general hygiene`), so the prefix a reader sees in a store is the file's
choice rather than a transformation in code. The fallback when there is no heading is the
filename.

Two fields a file set cannot always supply, stated rather than invented: `summary` is not
a seed field at all (it exists to make the listing readable), and `version` is **omitted**
from the declaration when the file declares none — never the crate's, which would claim an
author shipped something they did not.

## What this costs

The `.config`-in-a-repository confusion DEC-026 accepted now has one cure instead of two:
a reader who wants to know whether their project is overriding the shipped set runs the
listing. Someone who never runs it will not be told. That is the price of keeping a set's
bytes identical to its file, and it is the smaller of the two prices.

DEC-028's sentence that "a malformed file never breaks the other layers" reads wider than
this decision: isolation is about listing and loading, not about which set a name resolves
to. That clause is owed an in-place revision at reconcile.
