# CLI grammar and input handling

Settled 2026-11-01.

## Parser

`clap` v4 derive. Its usage exit code is 2, which matches the brief without
special-casing.

```
smut declare [JSON] [--compact]
smut query   [JSON]
smut render  [JSON] [--format markdown] [--heading-field <name>] [--heading-level <1-6>]
                    [--show-ids] [--show-fields]
smut get    <ID>...
smut delete <ID>...
smut learn
```

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
