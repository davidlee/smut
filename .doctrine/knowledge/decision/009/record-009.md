# Markdown rendering

Settled 2026-11-01.

## Layout

Entities in selection order, separated by one blank line.

```
## Query design

Keep selection small.

- **type:** question
- **tags:** ["storage"]
```

- Heading: `#`×level + the heading field's value, only when that field is
  selected, is a string, and is non-empty. Defaults: field `title`, level 2.
  Newlines inside heading text collapse to spaces — structural, not escaping.
- Body: when `body` is selected and is a string, emitted verbatim as its own
  block. Body bytes are never trimmed or escaped; Markdown content is preserved
  and never executed.
- **Other fields appear only with `--show-fields`**, as `- **name:** value`
  (strings raw, everything else compact JSON).
- The heading field and `body` are excluded from that attribute list **only when
  they were actually rendered specially**. A non-string `body` is not rendered as
  a body block, so it stays displayable as JSON under `--show-fields`.
- `--heading-field body` renders that value once: as the heading. It is not also
  emitted as a body block or attribute.

## Identity and the fallback

With `--show-ids`, identity renders separately, before the entity's content, as a
safely delimited code span.

**Fallback:** when nothing else would be *visible* — no heading, no non-empty
body block, no shown fields — identity is rendered instead, so a bodyless entity
never silently vanishes. An empty or whitespace-only body does not count as
visible, so it cannot suppress the fallback.

## Encoding

- Raw Markdown is preserved in body text and ordinary string values — never
  escaped, never executed.
- **Generated field labels are escaped**, and **ids are delimited safely**. A
  single-backtick code span is not sufficient, because public ids are opaque and
  may contain backticks: use a fence longer than the longest backtick run inside
  the id, and pad with spaces when the id itself begins or ends with a backtick,
  per CommonMark's code-span rule.
- Ids are never joined ambiguously: each is its own delimited span.

## Separators and verbatim bytes

Separators are added **without trimming body bytes**. A body ending in newlines
therefore may produce extra blank lines between entities. Verbatim fidelity wins
over uniform spacing; the README says so once rather than the renderer trimming.

## Diagnostics

In the document, after a blank line, never only on stderr:

```text
> truncated: result_limit; additional entities omitted
> missing: `a`, `b`
```

No `12 of 40 shown` form: that needs a pre-limit count the selection does not
retain, and this cut does not warrant widening `Selection` for a number.
Missing ids use the same safe delimited encoding as everywhere else.

Zero entities: no entity output; diagnostics still emitted when they exist.
