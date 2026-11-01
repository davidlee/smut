# IDE-002: Renaming an entity with referential stability

Today an id is permanent: `extract`/`hoist` in `src/entity.rs` consume the `id`
member as identity and skip it (`if key == ENTITY_ID { continue }`), `apply`
reinserts it, and no verb offers a rename. Even `{"id":"DROP","body":"b"}`
declares an id literally named `DROP` — the sentinel never reaches identity,
because the member is consumed before drop semantics apply.

The consequence, measured: `delete` leaves every incoming reference untouched, so
a delete-and-re-declare under a new id strands the referrers and they surface only
as `dangling` on a traversal that names the field (`{"field":"related",
"source":"a","target":"b"}`). `dangling` is a diagnostic, not a pointer you can
retarget: fixing it means re-declaring each referrer by hand.

The idea worth testing is whether smut should offer a *rename that carries the
references* — the one graph-wide edit that is cheap for a machine and expensive
for a person, and therefore exactly the kind of thing this store exists to do.

## Why the store is already shaped for it

- **Identity and placement are separate.** An entity's basename is an internal
  uid, stable until deletion, and `index.json` is the only place id and file
  meet. A rename could keep the uid, so the operation is an index-key change plus
  reference rewriting — no data file has to move, and the entity's bytes change
  only if the id lives inside them (it does: the file holds the complete entity,
  `id` first).
- **A batch is atomic.** One declaration is one write set and one journal commit,
  so a rename plus every rewritten referrer can land in a single commit: no reader
  observes a half-renamed graph, which is precisely the failure the current
  delete-and-redeclare forces on anyone doing it by hand.
- **The collision contract already exists.** A rename to an occupied id is a
  refused batch, not a merge — the same rule that keeps a generated collision from
  becoming an accidental upsert.

## The questions that decide whether it is worth having

1. **Surface.** A `smut rename FROM TO` verb, or a declaration form (`{"id":"old",
   "rename_to":"new"}`)? A verb says what it does and keeps the declaration
   language unchanged; a field keeps one input path and one batch. The declaration
   form raises an immediate question: what else may ride along in that batch.
2. **What counts as a reference to rewrite.** Every nested object with `id: old`,
   at any depth, in any entity — including inside the renamed entity's own fields.
   The rewrite must be a value-tree walk, not a top-level field sweep, or a
   reference two levels down is silently missed. (The traversal language cannot
   see nested references today, which makes a missed one invisible.)
3. **Dangling and missing targets.** Renaming an id that nothing references is
   trivial. Renaming an id whose own references are already dangling is fine. The
   awkward case is an entity that references a *missing* target by the old id:
   does the rewrite touch references that resolved to nothing? Argue yes — the
   text was meant for that entity — and say so.
4. **Response shape.** A rename touches N entities, so `changes` would report N
   updates. Does the response need to say which one was the rename and which were
   rewritten references (`previous` already carries the whole prior entity, so it
   is reconstructable, just noisy)?
5. **Repeated renames and stable ids.** If renames are cheap, an id stops being a
   commitment — which weakens the current advice to choose referentially stable
   ids. That is a real trade: ids as user-facing handles that can be tidied, or ids
   as permanent keys. Half-way (rename allowed, references must be rewritten by
   hand) is worse than either.

## What would make this a "no"

If the destination is a store a *human* reads (`ls`, `git log`), referential churn
is a historical signal, and permanent ids are the cheaper answer: the discipline
of naming things once is a feature, not a limitation. That argument is why this
sits in the backlog and not in a release.

## Neighbours

ISS-002 (`id` reserved in nested objects) shares the root cause — identity is
expressed by one bare key name in a value tree — and any redesign of how identity
is written should weigh both together.
