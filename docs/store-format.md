# The smut store format

This document describes the on-disk format of a `smut` store, version
`format_version` **1**. It is written against the 0.1 build and is the home of
the durability promise: the protocol below is what that promise rests on.

`format_version` appears in two documents — the store index and each journal
manifest — and neither accepts a version this build does not read. An unknown
`format_version` is exit 1. **There is no migration path in 0.1, and no repair
path at any point:** every inconsistency listed under [Corruption](#corruption)
is refused, never reconstructed.

Everything here is stated for a Linux filesystem with working `flock` and
`fsync`. See [Portability](#portability) for what is not promised.

## Layout

A store is one directory, `.smut/`, beside the working directory:

```
.smut/
  lock                  permanent lock inode; created once, never replaced
  index.json            committed index
  <uid>.json            one file per entity, holding the complete entity
  journal.prepare/      transient: staged bytes, commitment unknown
  journal.commit/       transient: committed, not yet applied
  journal.done/         transient: applied, being cleared
```

The store is never searched for in a parent directory, and every command that
reads or writes a store uses `.smut/` in the current directory. A read or a
delete against an absent store fails with exit 1 and creates nothing; only a
declaration with something to write creates the store.

`lock` is an empty file held under an exclusive `flock` for the whole of every
command that touches the store, reads included, because a read may recover. The
lock inode is created on demand, never truncated, never replaced by a rename,
and never unlinked — so locking it is locking the store, not locking one of its
names.

Reserved names inside `.smut/` are `lock`, `index.json`, `manifest.json`, the
three journal directory names, and anything ending in `.tmp`. None of them can
be an entity's basename. `lock`, `index.json`, the three journal states and the
protocol's temporaries are entries the store expects to find there; a
`manifest.json` belongs inside a journal state, so a top-level one is
corruption. Any other entry must be one the index maps, and an entry that is not
is corruption (see [Corruption](#corruption)).

## The index

`index.json` maps public ids to the basenames that hold them:

```json
{
  "format_version": 1,
  "objects": {
    "smut": "Ab3xKqMsmwQgzx6nfOhc2.json"
  }
}
```

- `format_version` is the integer `1`.
- `objects` maps each public id to one safe basename. An id appears once.
- Basenames are single path components: no separator, not `.` or `..`, not one
  of the reserved names, not in the `.tmp` namespace. The rule holds on read and
  on write, so no construction can serialise a basename the reader would refuse.
- Ids are lexical (byte-wise) within `objects`, so the index's bytes never come
  from directory iteration order.

An entity's basename is a generated internal uid plus `.json`. Internal uids are
filesystem-safe, independent of public ids, and stable until the entity is
deleted: an entity's public id never appears in a path.

## Entity files

An entity file holds the **complete** entity, including its public id. `select`
never narrows what is persisted, so a query's projection has no effect on what is
written.

## The journal manifest

`journal.prepare/manifest.json` names the write set:

```json
{
  "format_version": 1,
  "puts": [
    "Ab3xKqMsmwQgzx6nfOhc2.json",
    "index.json"
  ],
  "removes": [
    "Zq7pR4KsmwQgzx6nfOhc2.json"
  ]
}
```

The manifest **is** the write set, and the staged directory is its
materialisation. Disagreement between them is corruption, never something to
infer around. **Every commit stages a fresh `index.json`**, including a batch
that only updates existing entities and changes no id-to-file mapping, so the
write set always has the same shape and no member's absence ever means
"unchanged".

## Canonical bytes

One entity ordering, applied at the final serialisation boundary through a
shared wrapper: the entity root puts `id` first and its remaining keys lexical,
embedded ordinary objects are all-lexical, and arrays keep their order. Objects
without an entity root — the index, the manifest — are all-lexical. Serialising
and then re-parsing to arrange keys is never done, because a parsed value erases
the ordering a wrapper imposed.

Two byte forms, deliberately different:

| surface | form |
| --- | --- |
| entity files, `index.json`, journal manifest | pretty-printed, 2-space indent, trailing newline |
| command stdout | compact single line, trailing newline |

Array order is data and is preserved everywhere. Object key order is not data:
it is fixed by the rule above so that the same store always serialises to the
same bytes.

## The commit protocol

**The state is a directory name.** Under the lock at most one of the three
journal directories exists, and each rename is a state transition:

```
                            mkdir + staged bytes + manifest + fsync each
   (nothing) ──────────────────────────────────────────────▶ journal.prepare/
                                                                    │
                                              rename  ← THE COMMIT DECISION
                                                                    ▼
                                                               journal.commit/
                                                                    │ apply + fsync(.smut)
                                                                    ▼
                                                                 journal.done/
                                                                    │ unlink tree
                                                                    ▼
                                                                 (nothing)
```

### The replay material is durable before the decision to replay it

The `prepare → commit` rename publishes **staged bytes**, not bytes written
where they will be read: recovery replays *from the staging directory*. A
durable `commit` name over non-durable staging would be either a refusal of a
batch that was committed or a republication of torn bytes. So, before the
rename:

1. **every staged put file** — every entity file the write set puts, and
   `index.json`, which every commit stages — **and `manifest.json`**, is written
   to completion and fsynced **individually**;
2. `journal.prepare/` itself is fsynced, making its directory entries durable;
3. only then is the rename performed, and `.smut/` fsynced to make the decision
   durable.

A parent-directory fsync does not carry its children; file contents and
directory entries are separate obligations.

Any barrier failure **before** the rename is a **pre-commit** failure: nothing
has been published, so the command exits 1, the staging state is discarded, and
the store is exactly as it was. That is the opposite treatment to a failure
after the rename, which is roll-forward-only — and the asymmetry is the point.

### The commit decision is not the same as its durability

| point | logical status | recovery finds |
| --- | --- | --- |
| before the rename | nothing published | `prepare` — discarded; the store is exactly as it was |
| rename done, parent fsync not yet successful | uncertain after power loss | `prepare` or `commit`; both are coherent |
| parent fsync successful | durable commitment | `commit` — rolled forward |

The middle row is coherent **only because of the barrier above**: `commit` is a
findable state that can be replayed to completion, which requires every staged
byte to be durable already. Without that ordering the table would describe a
state recovery cannot honour.

Applying never begins before that parent fsync succeeds. Symmetrically,
`.smut/` is fsynced after the `commit → done` rename *before* any journal
content is deleted, or a power loss could restore the `commit` name after its
replay material is gone.

### Ordering within a commit

Puts → index → removals. Each published file is written to `<name>.tmp` in
`.smut/`, fsynced, and renamed over its target; each removal is an unlink. The
index is published before removed files are unlinked, so the committed index
never names a file that does not exist.

This is a convenience for a human reading the directory, not a guarantee to an
unlocked observer. An unlocked reader can read the old index, pause, and then
find a file gone; it can also see changed entities before the index is replaced.
See [Who the guarantee is for](#who-the-guarantee-is-for).

### When apply fails after the decision

If apply fails (a full disk, an I/O error), the commit decision stands: the
command exits 1, `journal.commit/` is left in place, and the next command
completes it. Rolling back a committed batch because the disk filled would be a
worse lie than failing forward.

## Recovery

Recovery runs under the lock and **strictly before** store validation:
validating first would refuse a crashed-but-recoverable store as corrupt, which
is how a store would be bricked by its first interrupted command.

1. Refuse, without modifying evidence, if more than one journal directory
   exists.
2. Remove only protocol-owned temporaries — the narrow namespace of single path
   components ending in `.tmp`. Nothing else is swept.
3. `done` → finish cleanup. `prepare` → discard. `commit` → validate the
   manifest completely, then replay, then rename to `done`, fsync, and clear.

Validation of a `commit` state checks, before a single byte is replayed:
manifest format version; every basename safe; entries unique; `puts` and
`removes` disjoint; reserved names excluded; every listed put present in the
staging directory and a regular file; every staged file listed. Any failure is
a refusal, never a repair.

Replay is a total re-execution: each staged file is written to `<name>.tmp`,
fsynced, and renamed over its target; each removal is unlinked again. Nothing is
inferred from what is missing, so there is no silent-repair path at all.
Recovery and cleanup never follow a symlink — names are unlinked directly, never
resolved.

A crash between writing `<name>.tmp` and renaming it leaves a temporary outside
the journal; recovery removes it before validation, so the strict
unexpected-entry rule cannot turn a recoverable crash into an unrecoverable one.

Recovery is itself interruptible. If it is interrupted, the next command's
recovery converges on the same state: a `commit` state is replayed from the
staged bytes it still holds, and a `done` state is cleared again. Recovery
publishes no index of its own.

## Corruption

A malformed index, a missing indexed file, a duplicated path mapping, an
id/content disagreement, an unsafe basename, a symlinked entry, an unknown
format version and an unexplained entry in `.smut/` are each an explicit
inconsistency error, reported on stderr, naming the entry where the entry is the
evidence, with exit 1. A few, as the build prints them:

```
smut: the store index has format_version 2, and this build reads only 1
smut: index.json is not a valid store entry: malformed JSON: expected ident at line 1 column 2
smut: O3f_kqMsmwQgzx6nfOhc2.json is not a valid store entry: the index maps it, and the file is not there
smut: the entity file O3f_kqMsmwQgzx6nfOhc2.json holds id "b", and the index maps it to a
smut: unexpected entry in the store: stray.json is not explained by the index, and is never adopted or repaired
smut: the store entry Zq7pR4KsmwQgzx6nfOhc2.json is a symlink, and the store never follows one
smut: more than one journal state is present (journal.commit, journal.prepare), and recovery never guesses which one to apply
```

### What is never adopted

An unindexed entity-looking file is never adopted, and an `.smut/` that holds
entity files but no `index.json` is refused rather than initialised over. There
is no document editing, no file watching, and no supported way to add, rename or
remove a store entry by hand: the index and the entity files are written by the
commit path and by nothing else.

## Portability

Linux is the acceptance platform. The format is plain files and directories, and
nothing in it is Linux-specific by intent, but nothing else is tested.

`flock` is advisory. It excludes cooperating `smut` commands and nothing else: a
process that does not take the lock — an editor, `cat >`, a stray script — is
not excluded, and the store's consistency does not survive edits made that way.
Nothing is promised to a process that does not take the lock.

`fsync` is assumed to do what it is documented to do on the filesystem the store
lives on. The interruption tests assert the *order* of the protocol's steps by
aborting the process at each named boundary; they do not, and cannot, establish
what a power loss leaves on a disk. Write-ordering durability is assumed from the
filesystem, not proven by this project.

## Who the guarantee is for

> Under the documented filesystem and fsync assumptions, cooperating commands
> observe whole batches. Before application begins, the commit decision is made
> durable, together with every staged byte that decision commits to.
> Interrupted committed batches are replayed before any command serves store
> state. Failure before acknowledgement may leave the outcome uncertain;
> successfully acknowledged batches are durable.

That is **atomic visibility through locking and recovery** on top of non-atomic
physical publication. It is not claimed that a group of renames is atomic on any
real filesystem.

The uncertainty is named precisely, and its direction matters:

- **A write may have committed with no success response received.** A process
  can die between the commit decision and the response reaching the caller, so
  an interrupted command's effect is *unknown*, not *undone*. Re-running the
  batch is the caller's decision, and there is no request-id or exactly-once
  retry in 0.1.
- **An acknowledged write is durable.** Once a command has printed its success
  response and exited 0, the batch is committed and made durable, and no later
  command will find it half-applied.

The ambiguity is *commit before acknowledgement*, never after it.
