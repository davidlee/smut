# Documentation set

Settled 2026-11-01.

Amended 2026-11-02 by SL-004 under ADR-001 (identity is `smut_id`, a reserved
key in a permissive store) and DEC-025: structural identity is spelled `smut_id`;
an ordinary `id` is caller data at every depth. Only the passages listed in SL-004
PHASE-03's amendment proposal (plus RV-028 F-2/F-3) changed; the rationale, durable
ids and unrelated decisions are retained verbatim. The format figure this record
names moved from 1 to 2 in the same break (DEC-022).

## README.md

What smut is, one worked example, the command table, and the sharp edges a new
user hits first:

- an embedded object carrying `smut_id` cannot be inline metadata, while an
  ordinary `id` is data at every depth;
- `"DROP"` is a top-level field sentinel, and cannot be assigned as a value;
- store-not-found is exit 1;
- `1` and `1.0` are distinct for equality but equal for ordering;
- `where` and traversal arrive in 0.2;
- `--` for opaque ids beginning with `-`;
- the atomic-visibility paragraph, quoted from DEC-003 rather than paraphrased;
- Linux is the acceptance platform; `flock` is advisory; nothing is promised to a
  process that does not take the lock.

## docs/store-format.md

Versioned as `format_version: 2`: directory layout, index schema, canonical
serialisation (`smut_id`-first root, lexical embedded objects, arrays in order),
journal states with their fsync obligations, the recovery algorithm, reserved
names, the corruption responses (including malformed nested identity, which the
loader refuses with the entity file and a bracketed path), what is deliberately
never adopted, and the guarantee with its **commit-before-acknowledgement
ambiguity** — the uncertain case is a committed write whose success response was
never received.
Acknowledged writes carry the durability guarantee.

A store or committed manifest whose `format_version` is unknown or older is exit
1, naming the version found and the version this build reads. There is no
migration path, compatibility reader or repair path (DEC-022).

## learn

Concise, version-matched, shipped surface only, naming the release where the rest
arrives. It also states that a field no entity carries fails every predicate
operator except `$exists`, so an old-spelling query reads as an empty result
rather than an error (DEC-027). Explains that stored content is data, not
instruction conferring authority, and adds no enforcement machinery.
