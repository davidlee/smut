# IMP-007: `smut render` — short flags for its long flags

`render` carries five display flags and every one of them is long-only, so the
common invocation is a row of words:

```
smut render '{}' --show-ids --heading-field type --heading-level 3
```

The short forms exist for the same reason `--` exists: the flags are meant to be
typed, often, by hand. `declare`'s single flag (`--compact`) has no short form
either, so whatever is chosen should be chosen for the CLI once rather than for
`render` alone.

## What is free

`-h` is help and `-V` is version, so those are out. Current candidates, none
decided:

| long | plausible short | note |
| --- | --- | --- |
| `--format` | `-f` | obvious, and `--format markdown` is the only value today |
| `--show-ids` | `-i` | identity |
| `--show-fields` | `-F` or `-s` | `-s` collides with nothing yet but reads like "select" |
| `--heading-field` | `-t`? `-H`? | `t` reads as title but means any field; `-H` pairs less obviously with `-l` |
| `--heading-level` | `-l` or `-L` | `-l` and `-H` are a coherent pair; `-l` and `-t` are not |
| `declare --compact` | `-c` | consistency question, not a render question |

The decision worth making explicitly: whether the two heading flags form a pair
(`-H`/`-L`, or `-t`/`-l`) and whether `--show-fields` takes the letter a reader
would guess. A short flag nobody can remember is worse than the long one.

## Constraints

- **Clap derives these**, so the letters are declared on the derive attributes in
  `src/cli.rs`; nothing in the core or the renderer changes.
- **The grammar is documented in two places that can drift**: `smut learn`'s
  command block and `--help`. Both should list the short form once it exists, and
  a prose pin in `src/learn.rs` should hold it there.
- **`--` still ends flag parsing**, and a short flag must not weaken that: the
  guide's `smut get -- -weird-id` example is the reason the parsing order matters.
- **No behaviour changes.** This is pure surface, so it should not disturb a
  single response shape — and its tests belong in `src/cli.rs`'s parsing tests,
  not in the renderer.

## Neighbours

IMP-002 (the 1.0 presentation release) covers the *behaviour* of these flags in
both formats and the list of format-specific ignored flags; this item adds only a
second spelling for flags that already behave. Not sequenced after it.

## Ready to pick up when

Any time. The only reason to wait is if the letter choice would be settled better
alongside a wider CLI surface review — nothing in this change forecloses one.

## Resolution — 2026-11-02: done

Landed directly, without a slice (the operator waived lifecycle ceremony for a
change this size; SL-007 was created for this pairing and abandoned before
design, keeping its scope document as the record).

The scheme chose the "same letters as `render`" row from the table above, and
extended it to the two flags outside `render` that share a noun:

| long | short |
| --- | --- |
| `--format` | `-f` (`render` and `conventions`) |
| `--show-ids` | `-i` |
| `--show-fields` | `-s` |
| `--heading-field` | `-t` |
| `--heading-level` | `-l` |
| `--compact` | `-c` |

`-t`/`-l` over `-H`/`-L` because capitals need shift and read as an afterthought;
`-s` over `-F` because a case-only distinction next to `-f` is a letter to
mistype. Lowercase, one letter per noun.

All three stated constraints held. The letters are declared on the derive
attributes in `src/cli.rs`, so nothing in the core or the renderer changed. Both
places that document the grammar list the short form: `learn`'s command block
(with a prose pin in its rendering section) and the README's flag list/table.
`--` still ends flag parsing, pinned by a test that reads `-i` as an id after it.

Testing is equivalence rather than existence: one table parses seven short/long
pairs and compares the whole resulting request, including the invocation IMP-007
quotes in full, so a letter wired to the wrong field or dropping a value fails.
DEC-008 was amended with the letter table.
