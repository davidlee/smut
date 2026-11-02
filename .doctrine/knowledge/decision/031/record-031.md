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
2. **A set's identity is its origin.** Every listing row names the layer and the file.
   A caller can always answer "where did this advice come from" from the same command
   that gave them the advice.
3. **The body is the file's bytes.** `smut conventions <name>` prints the body — the file
   without its frontmatter — and nothing else. No banner, no footer, no injected heading.
   The seed's `body` is that same string, so a seeded entity and its source file diff
   clean.
4. **A shadowed set stays visible and says so.** The row for a set that lost its name
   prints `shadowed` in place of its summary: its summary is not what anyone is getting.
5. **A failed set is a row, not a silence.** It prints its path and its one-line reason,
   and asking for it by name refuses with that reason (DEC-033 governs the exit code).

The listing is therefore `name · layer · summary-or-verdict · path`, one row per file, in
precedence order:

    hygiene  project  general structure and relations  /repo/.config/smut/conventions/hygiene.md
    hygiene  user     shadowed                        /home/d/.config/smut/conventions/hygiene.md
    hygiene  binary   general structure and relations  conventions/hygiene.md

## What this costs

The `.config`-in-a-repository confusion DEC-026 accepted now has one cure instead of two:
a reader who wants to know whether their project is overriding the shipped set runs the
listing. Someone who never runs it will not be told. That is the price of keeping a set's
bytes identical to its file, and it is the smaller of the two prices.
