# DEC-043: The selection flags and the display flags are each one flattened clap struct shared by every verb that takes them

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amendment — 2026-11-03 (SL-011 reconcile, RV-037 F-3)

One consequence was not listed. The flatten puts the display letters beside the id positional on
`r`, so a hyphen-led word clap reads as a cluster of `-s`/`-i` optionally ending in `-t<value>`
(`-title`, `-site`) is silently display flags at exit 0. That is the accepted cost of keeping the
letters (DEC-008, IMP-007); the guide teaches `--` for hyphen-led ids on `r`, and DEC-037 records
the narrowed guard.
