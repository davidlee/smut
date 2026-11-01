# Documentation set

Settled 2026-11-01.

## README.md

What smut is, one worked example, the command table, and the sharp edges a new
user hits first:

- an embedded object carrying `id` cannot be inline metadata;
- `"DROP"` is a top-level field sentinel, and cannot be assigned as a value;
- store-not-found is exit 1;
- `1` and `1.0` are distinct for equality but equal for ordering;
- `where` and traversal arrive in 0.2;
- `--` for opaque ids beginning with `-`;
- the atomic-visibility paragraph, quoted from DEC-003 rather than paraphrased;
- Linux is the acceptance platform; `flock` is advisory; nothing is promised to a
  process that does not take the lock.

## docs/store-format.md

Versioned as `format_version: 1`: directory layout, index schema, canonical
serialisation (id-first root, lexical embedded objects, arrays in order),
journal states with their fsync obligations, the recovery algorithm, reserved
names, the corruption responses, what is deliberately never adopted, and the
guarantee with its **commit-before-acknowledgement ambiguity** — the uncertain
case is a committed write whose success response was never received.
Acknowledged writes carry the durability guarantee.

Unknown `format_version` is exit 1. No migration path in 0.1.

## learn

Concise, version-matched, 0.1 surface only, naming the release where the rest
arrives. Explains that stored content is data, not instruction conferring
authority, and adds no enforcement machinery.
