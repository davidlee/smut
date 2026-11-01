# Journal protocol, state validation, and the durability guarantee

Settled with the operator on 2026-11-01, amended in six particulars. The
machinery is small; the guarantee is what has to be exact.

## State is a directory name

Under the lock, at most one of three sibling directories exists in `.smut/`.
Each rename is a state transition:

```
                mkdir + write staged bytes + manifest
   (nothing) ────────────────────────────────────────▶ journal.prepare/
                                                              │
                                          rename  ← THE COMMIT DECISION
                                                              ▼
                                                         journal.commit/
                                                              │ after apply + fsync(.smut)
                                                              ▼
                                                          journal.done/
                                                              │ unlink tree
                                                              ▼
                                                           (nothing)
```

`journal.prepare/` holds the new bytes for every touched file, staged under its
final basename, plus `manifest.json`:

```json
{"format_version":1,"puts":["index.json","<uid>.json"],"removes":["<uid>.json"]}
```

The manifest **is** the write set; the staged directory is its materialisation.

## The commit decision is not the same as its durability

| point | logical status | recovery on next command |
| --- | --- | --- |
| before the `prepare → commit` rename | nothing published | discard `prepare` — the store is exactly as it was |
| rename succeeded, parent fsync **not** yet successful | **uncertain after power loss** | may find `prepare` or `commit`; both outcomes are coherent |
| parent fsync succeeded | durable commitment | must roll forward |

**Applying never begins before that parent fsync succeeds.** The fourth step is
the only one that licenses publishing bytes.

Symmetrically: **fsync `.smut/` after `commit → done` before deleting any journal
contents.** Without it, power loss may restore the `commit` name after its replay
material has been deleted — file durability and directory-entry durability are
separate obligations.

## Ordering within a commit

Puts → index → removals. The index is published before removed files are
unlinked, so the committed index never names a file that does not exist.

**This buys nothing for non-cooperating observers** and is not claimed to. An
unlocked reader can read the old index, pause, and then fail to open a deleted
file; it can also observe changed entities before the index is replaced.
Coherence is guaranteed to cooperating commands holding the lock, and to no one
else.

## State-dependent validation and replay temporaries

| state | treatment |
| --- | --- |
| `prepare` | may be incomplete, including an absent or malformed manifest — discard it safely |
| `commit` | validate the **complete** manifest and staged file set *before replaying anything* |
| `done` | may be partially deleted — finish cleanup without requiring complete contents |

Validation of `commit` checks: manifest format version; every basename safe (no
separators, not `.`/`..`, no traversal); entries unique; `puts` and `removes`
disjoint; reserved names excluded (`lock`, the three journal names, `manifest.json`,
any `.tmp`); every listed put present in the staging directory; every staged file
listed. Any failure is a refusal, never a repair.

Cleanup and replay never follow symlinks — names are unlinked directly, never
resolved.

More than one state directory present is an error, not a tie-break.

**Replay temporaries are protocol-owned.** Publishing a file writes
`<basename>.tmp` in `.smut/`, fsyncs it, then renames over the target. A crash
between those two steps leaves a `.tmp` outside the journal. Recovery removes any
`*.tmp` in `.smut/` **before ordinary store validation**, so the strict
"unexpected file" rule cannot turn a recoverable crash into an unrecoverable
one. The `.tmp` suffix is reserved inside `.smut/` and is never a legal entity
basename.

## Recovery ordering is a prerequisite of validation

Recovery runs under the lock and **strictly before** store validation. Validating
first would refuse a crashed-but-recoverable store as corrupt — the failure mode
that bricks a store after its first interrupted command. The ordering, and the
lock that makes it meaningful, are inq-7's subject; this record states the
obligation.

Initialisation is part of the same accounting: creating `.smut/` requires syncing
its parent directory to claim survival of power loss, and the first transaction
needs a defined path before any valid index exists. Detail in inq-7.

## Post-commit failure is roll-forward-only

If apply fails (ENOSPC, EIO), the commit decision stands. Report exit 1, leave
`journal.commit/` in place, and let the next command complete it. Undoing a
committed batch because the disk filled would be a worse lie than failing
forward.

## The guarantee, in the words the format doc will use

> Under the documented filesystem and fsync assumptions, cooperating commands
> observe whole batches. Before application begins, the commit decision is made
> durable. Interrupted committed batches are replayed before any command serves
> store state. Failure before acknowledgement may leave the outcome uncertain;
> successfully acknowledged batches are durable.

That is **atomic visibility through locking and recovery**, on top of
non-atomic physical publication. It is deliberately *not* claimed that a group of
renames is atomic on any real filesystem.

## Test obligations

- Crash injection lives behind a **test-only cargo feature**, not an
  environment-triggered abort compiled into ordinary builds.
- Process interruption (`abort()` at a named boundary) tests replay and cleanup.
  It does **not** establish write-ordering durability: the kernel survives and
  may retain or flush unsynced writes. Power-loss ordering needs simulated
  filesystem failures, which is the same distinction SQLite draws.
- The suite must inject: short writes, failed fsync, failed rename, failed
  unlink, and interruption *during recovery itself* (including during a repeated
  recovery of an already-interrupted recovery).

## Alternatives considered

- Plain rename-out-of-staging — rejected: makes "the staged file is absent, so it
  was applied" an inference.
- Plain copy during replay — rejected: can publish a torn file.
- A `COMMITTED` marker file in one journal directory — viable, but the presence
  of a name cannot then represent "ambiguously committed", and the uncertainty
  window exists either way; a directory rename makes the state structurally
  unambiguous.
- Claiming physical batch atomicity — rejected: false.
