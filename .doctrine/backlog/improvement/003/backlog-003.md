# IMP-003: smut 0.1.x — silent loss, chronology and legibility fixes from first real use

Three clusters of friction, found by the operator using 0.1 for real (two live
sessions recorded as a smut store at `interview.local/.smut`, 66 entities; entry
points `start-here` → `audit-handoff`). Every one of them is **design-conformant**
today — none is an implementation defect, which is why they are here and not on
RV-004.

Scope: fixes that need no dependency on 0.2 and no change to the brief's
contract. IMP-001 owns the graph release, IMP-002 the 1.0 presentation work; this
item is the cheap, independent set that can ship at any time.

## The three clusters

**1. Silent loss** — `finding-smut-08`, `finding-smut-20`.

- The literal `"DROP"` cannot be stored as a top-level field value, and the
  attempt removes the field and reports `created` with no warning. The one place
  smut discards input without refusing it.
- A supplied array replaces the stored one, and the response reports only
  `updated`. A caller intending to add a reference silently loses the list — it
  caught the operator three times in one session.

Cures, cheapest first: refuse `DROP` as a literal through the refusal path that
already exists for trailing input; carry a count or a per-field change list in the
`changes` entry so a replacement is visible in the response (`previous` already
carries the old value, which is what made the damage recoverable); consider a
`$append`/`$remove` operator only if a fourth argument is ever needed — putting a
second operator in the value space is what `finding-smut-08` argued against.

**2. Time** — `finding-smut-09`.

Entities carry exactly the declared fields; `index.json` carries no sequence or
timestamp. Chronology cannot be reconstructed, `order` cannot sort by age, and the
filesystem mtime dies on copy. Every caller hand-writes its own timestamp, which
is the discipline a store exists to remove. smut already knows both facts on every
write; stamping `created`/`updated` is the smallest change with the widest reach.
Caveat to decide first: whether the stamps are fields the caller can see and
`select`, and what they do to `previous` and to the brief's declared field set.

**3. Legibility** — `finding-smut-10`, `finding-smut-11`, `finding-smut-13`.

Each repairs an argument the design makes rather than a behaviour:

- opaque filenames: `.smut/` is greppable but `ls` and `git log` name files by
  hash, which undercuts the inspectability that justifies the file-per-entity
  layout. A sanitised id as filename with the hash as fallback for unsafe ids
  costs an index that already exists.
- `--show-fields` prints references as raw JSON (`[{"id":"y"}]`) — the only place
  JSON reaches the human surface. Rendering ids, or links when the target has a
  title, makes the flag usable for what it is for.
- rendered documents have weak boundaries: an entity with no heading field renders
  as body only, and a stored body opening `# ` impersonates the generated
  structure. Fencing or indenting stored headings finishes a job the truncation
  notice already started.

## The rule that makes any of this durable

`finding-smut-17`: **a fix that is not in the guide is not fixed.** Each item above
has a behaviour half and a sentence half. The behaviour half alone leaves every
future agent re-deriving the convention (retain the whole reference list when
amending; render in bounded chunks). So each cluster lands with its sentence in
`smut learn`, or — for advice rather than specification — in the conventions
command IMP-001 scopes.

## Not in this item

- Order of work. `q-fix-order` is open and the operator's call: silent loss
  (correctness, minutes), time (smallest change, widest reach), or legibility
  (three cosmetic fixes that each repair an argument). Nothing here is expensive;
  the sequence is a judgement, not a dependency.
- The two-dialect argv complaint (`finding-smut-02`): a genuine papercut, but it
  is a UX nicety on a path the store's own `a-error-verdict` retracts as "not a
  surprise". Fold it in if the refusal path is being touched anyway.
- Anything needing new store semantics, a schema, or validation — excluded by
  `criterion-scope-smut`.

## Provenance

Product evidence, not audit evidence: RV-004's triage of the same store found no
0.1 conformance defect, and recorded that verdict in
`.doctrine/slice/001/notes.md` § Harvest → *External use evidence*.
