# Design-run mechanics worth knowing before improvising

Hit while driving SL-001's managed design run end to end. Each cost a refused
write or a stale gate; none is obvious from `design show`.

## 1. An inquiry cannot be resolved by lifecycle alone

`declare: [{"subject":"inq-1","lifecycle":"resolved"}]` is refused:
*"inq-1 cannot resolve without a disposition"*. Disposition lives on a
checkpoint node, which either adopts an existing record or creates one:

```json
{"declare":[{"subject":"cp-1","disposes":"inq-1",
             "dispose":{"form":"adopt","record":"DEC-001"}}]}
```

Forms: `adopt {record}`, `create {kind,title,body,…}`, `unresolved {note}`,
`non-durable {note}`. Also note `provenance` is **inert once held** — a later
submission carrying it is refused rather than ignored, so decide it at creation.

## 2. Editing a section during `reviewing` stales the pass

Entering `reviewing` opens a pass over the current materialisation watermark.
Integrating findings by editing sections in place leaves the envelope reporting
`review_pass RV-NNN STALE — it no longer covers current content`, and
`review-disposed … conducted: {review: RV-NNN}` will not be admissible over it.
`review prime` does not clear this; it is not the cache-staleness signal.

The mechanism's answer is the back-edge: regress to `drafting` with a reason,
then advance to `reviewing` again. Re-entry opens a **new** pass (RV-002 in our
case) over the settled content; the old ledger keeps its history. Cheap and
honest — do not try to force the disposition against a stale pass.

## 3. The research baseline hashes the slice scope document

`.doctrine/slice/NNN/research/baseline.toml` stores a hash per intent doc,
including `slice-NNN.md`. Any scope edit — re-scoping, retitling prose,
correcting a claim — makes `doctrine verify research-current` stale. Run
`doctrine slice research <id> --restamp` after editing, and re-check with
`doctrine verify research-current --slice <id> -p <root>`.

Related: `doctrine slice selector add … --intent design-target` **upserts by
path**, so re-adding a path already carrying `scope-relevant` replaces its
intent rather than duplicating the row.
