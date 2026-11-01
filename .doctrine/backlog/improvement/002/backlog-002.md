# IMP-002: smut 1.0 — presentation release: Mermaid, display flags, docs and hardening

Third of three releases delivering the v1 brief
(`smut-implementation-brief.org`). Prerequisites: SL-001 (0.1) and IMP-001 (0.2).

## What it delivers

The capability sentence: *present the graph to a reader, and know the limits.*

- Mermaid renderer: fenced directed graph, generated internal node symbols,
  escaped labels, `#quot;` entity escaping, edges only where source and target
  are both selected, duplicate `source/field/target` collapsing, external nodes
  omitted. Label from the selected heading field, else ID.
- Display flags completed: `--show-ids`, `--show-fields`, `--heading-field`,
  `--heading-level` behaviour in both formats, and the documented list of
  format-specific ignored flags.
- README with command/response examples covering the whole surface.
- Portability limits section (Linux primary; `flock` advisories; fsync
  behaviour on network filesystems).
- Modest generated-store smoke benchmark for scan/query/render practicality.

## Why it is last

Mermaid is the only place where output correctness is *silent* when wrong
(mis-rendered graphs look plausible), so it wants a stable entity and query
surface underneath it. Documentation that promises durability and portability is
also safer written against a finished store protocol than an in-flight one.

## Inherited design questions

Deferred from SL-001's inquiry map:

- Markdown/Mermaid layout, fallback and escaping rules in full (SL-001's inq-9
  covers Markdown only);
- visible incompleteness disclosure in both formats;
- what is documented, and where (SL-001's inq-10 covers README + on-disk format;
  portability and the benchmark are additions here).

## Ready to pick up when

0.2 is closed and the query surface has stopped moving.
