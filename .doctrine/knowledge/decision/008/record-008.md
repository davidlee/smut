# CLI grammar and input handling

Settled 2026-11-01.

## Parser

`clap` v4 derive. Its usage exit code is 2, which matches the brief without
special-casing.

```
smut declare [JSON] [-c|--compact]
smut query   [JSON]
smut render  [JSON] [-f|--format markdown] [-t|--heading-field <name>] [-l|--heading-level <1-6>]
                    [-i|--show-ids] [-s|--show-fields]
smut get    <ID>...
smut delete <ID>...
smut learn
```

## Amendment — 2026-11-02: one-letter spellings

Every long flag gained a short form (IMP-007), because the display flags are
typed by hand and often in one line:

| long | short |
| --- | --- |
| `--format` | `-f` |
| `--show-ids` | `-i` |
| `--show-fields` | `-s` |
| `--heading-field` | `-t` |
| `--heading-level` | `-l` |
| `--compact` | `-c` |

Lowercase throughout, one letter per noun where the noun is unambiguous: `-t`
for the heading field because it defaults to `title` and is usually set to
`type`, and `-s` over `-F` because a case-only distinction from `-f` is a letter
to mistype. Nothing about this changes a behaviour, a response shape or an exit
code — the short form parses to the identical request, which is how it is tested.

Two constraints held: `-h` and `-V` stay help and version, and `--` still ends
flag parsing, so `smut get -- -i` names the id `-i` rather than the flag. Both
are pinned in `src/cli.rs`'s parsing tests.

## Input rules

- Exactly one JSON operand. `-` reads stdin explicitly.
- Operand absent: on a TTY → usage help, exit 2; on a pipe → read stdin.
- Trailing non-whitespace after the JSON value → exit 2. No newline-delimited
  stream mode.
- **`--` terminates flags**, so an opaque id beginning with `-` is usable
  (`smut get -- -weird-id`). Documented in the README and `learn`, because ids are
  opaque and may legitimately start with a hyphen.
- **Spec types and flag combinations are validated before any store access** —
  extending DEC-004's step 1 from "parse the JSON" to "validate the whole
  request". A malformed spec must not create, lock or recover a store.

## `--format`

`markdown` is the default and the only value in 0.1. `mermaid` is refused with
exit 2 and the message *"not supported in 0.1; planned for 1.0"*, supplemented —
not replaced — by the tracking item. User-facing copy names a release, never a
bare item id.

## Exit codes

Usage and invalid input are 2, including clap's own argument errors; storage and
runtime failures are 1; reported missing ids and truncation are success, 0.
