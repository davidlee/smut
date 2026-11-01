# Lock, pipeline ordering, and store initialisation

Settled with the operator on 2026-11-01, amended in five particulars plus two
wording corrections.

## The pipeline, in order

```
1.  parse input                              (shell) — invalid input dies here, exit 2,
                                              before any filesystem access
2.  pre-compute against an EMPTY snapshot    (pure) — only when .smut is absent; this is a
                                              pre-check, never authoritative
3.      empty write set  ────────────────────▶ success, store never created
        non-empty        ────────────────────▶ continue
4.  mkdir .smut · fsync PARENT dir · open/create lock · flock(LOCK_EX)
5.  RECOVERY                                 (before any store validation)
6.  LOAD + VALIDATE store
7.  RECOMPUTE against the validated snapshot (authoritative — another process may have
                                              initialised and written meanwhile)
8.  COMMIT                                   (skipped when the write set is empty)
9.  PREPARE OUTPUT                            (serialise / render fully — still under the lock)
10. unlock
11. write output
```

Step 2 exists so that a request which cannot produce a write never creates a
store — and so that invalid input is refused before any filesystem access. Step 7
exists because the pre-check ran against an empty snapshot: the lock may have been
won by a process that created the store and committed a first batch.

For a read or delete command against an absent store the pipeline is shorter: no
store → store-not-found, exit 1, nothing created.

## Concurrent initialisation

- `mkdir` returning `AlreadyExists` is normal: join the locking path.
- `lock` is opened and created without truncation and never replaced or renamed.
  (Replacing or unlinking it forks the lock domain; truncating does not change its
  inode and serves no purpose, so it is avoided too.)
- **Any process using a possibly newly created `.smut/` must sync the parent
  directory entry itself before acknowledging a write.** It must not rely on the
  process that happened to win `mkdir` surviving long enough to do so.

## Recovery ordering and evidence preservation

Recovery runs first under the lock, in this order:

1. **Refuse without modifying evidence** if more than one journal state directory
   exists. The ambiguity is the finding; deleting anything first would destroy it.
2. Clean only the narrowly reserved temporary namespace (basenames the protocol
   itself generates), never arbitrary `*.tmp` files.
3. A committed journal is validated **completely** before replay *or* any
   associated cleanup.
4. Nothing in recovery resolves a symlink; names are unlinked directly.

## Validation includes unexplained files

Validation covers the index and mapped files, and equally the files the index does
not explain. After loading:

- mapped file missing · duplicate path mapping · id/content disagreement · unknown
  format version · malformed JSON → exit 1, never repair;
- **unexpected entry in `.smut/`** (not `lock`, not a journal state directory, not
  `index.json`, not a mapped entity file, not the reserved temporary namespace) →
  the same explicit inconsistency error, naming the entry.

An unexplained entity file must not silently disappear from the graph, and a
permissive rule here is how silent loss begins. The directory namespace is
reserved by the tool.

## Output preparation and the stdout ambiguity

Output is **serialised in full before unlocking** and written afterwards. That
keeps serialisation and rendering errors separate from pipe failures, and
guarantees the emitted response describes the snapshot actually processed.

A stdout failure after a commit cannot undo the commit. That ambiguity is
documented, not engineered around: the batch is durable whether or not anyone read
the acknowledgement.

## What the lock covers

One exclusive lock for every command that observes or mutates store state,
including reads (a read may recover, which mutates). `learn`, `--help` and
`--version` touch neither lock nor store. The lock is held through commit and
output preparation, released before the bytes are written.

Nothing is promised to a process that does not take the lock.

## Test obligations

- Two simultaneous first declarations against a store that does not yet exist:
  exactly one store, both batches present or one refused coherently, never a
  mixed or orphaned state.
- Termination before the first lock acquisition: no store, no partial store.
- A second process recovering the first transaction: replay completes, then the
  second process's own command runs.
- Interruption during *initialisation* specifically — after `mkdir`, after parent
  fsync, after lock creation, before the first index publication. There is one
  index-publication path, but the directory-creation and pre-index empty states
  have their own crash windows and their own tests.
- Interruption during recovery, including a second recovery of an already
  interrupted recovery.

## Alternatives considered

- Trusting the `mkdir` winner to fsync the parent — rejected: the winner can die
  before acknowledging anything, leaving a store that exists in memory and not on
  disk.
- Shared read locks with upgrade — rejected: the read path mutates on recovery,
  so the upgrade is the common case, and it adds a race class for no gain.
- Treating an index-less `.smut/` as an empty store — rejected: a lost index
  beside intact entity files would silently orphan every entity in it.
- Cleaning arbitrary `*.tmp` during recovery — rejected: it is not the protocol's
  namespace to delete, and doing so can destroy the evidence a refusal needs.
- Deleting temporaries before checking journal-state ambiguity — rejected: it
  destroys evidence before the refusal that needs it.
