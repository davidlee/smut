# What happens

A managed design run that has locked (`stage locked (revision N)`) refuses `doctrine design
adopt`:

    Error: run is locked; adopt refuses. To correct a run still governing execution,
    regress to reviewing first; at reconcile, edit design.md directly and do not adopt

That message is the answer to the question reconcile would otherwise have to guess at. Three
distinct postures, by lifecycle stage:

- **run still governing execution** (the slice is being implemented from it) — regress to
  `reviewing` first, then revise through the run;
- **at reconcile** — edit `design.md` directly, and **do not adopt**. The run is not
  re-entered, so there is nothing to desync;
- **`adopt` exists for** the case it names: a document edited outside the run that the run
  must take back as its baseline (DEC-279).

# What not to infer from the run's display

`doctrine design show <slice> --format status` keeps reporting `changes N since the declared
baseline` and `review_pass RV-NNN STALE — it no longer covers current content` after a
reconcile edit. That staleness is a statement about the *old* design review's coverage, not a
gate: re-conducting a design review at reconcile is out of that seam's scope. Record the
repairs and let the design-review pass's staleness stand.

# The honest failure mode

Hand-editing materialised prose *and then* forcing an adopt to hide the divergence would be
the thing to avoid. Here the CLI names the lawful path, so take it and say so in the
reconciliation outcome rather than leaving the reader to wonder why the run looks changed.
