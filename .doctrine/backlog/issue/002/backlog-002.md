# ISS-002: `id` is reserved in every nested object, so a payload's own id cannot be stored

`id` is not a field name in a declaration: anywhere in the tree, an object with a
string `id` member *is* an entity node. Three different behaviours hang off that
one key, and only one of them refuses:

```
# id plus siblings: silently declares an entity, and the occurrence becomes a reference
❯ smut declare '{"id":"x","payload":{"id":"abc","name":"Ext"}}'
{"changes":[{"id":"x","previous":null,"status":"created"},
            {"id":"abc","previous":null,"status":"created"}],"ids":["x"],"missing":[]}
❯ smut get x
{"objects":[{"id":"x","payload":{"id":"abc"}}],…}          # "Ext" moved to entity abc

# id alone: a reference, reported missing when unstored, and nothing created
❯ smut declare '{"id":"y","payload":{"id":"zzz"}}'
{"changes":[{"id":"y","previous":null,"status":"created"}],"ids":["y"],"missing":["zzz"]}

# id of any other type: the whole batch is refused
❯ smut declare '{"id":"n1","payload":{"id":null,"name":"E"}}'
smut: invalid entity id: null, and an id must be a nonempty string      # exit 2
```

That is a likely collision for a store whose whole premise is *permissive*: any
payload copied from an external system — an API response, a JSON export, a
document fragment — is likely to carry a domain `id` of its own, and a store that
cannot hold `{"external":{"id":7}}` or `{"external":{"id":null}}` is refusing
ordinary data. Today's workaround is a convention: rename the key (`external_id`,
`sid`) before declaring, and accept that the store cannot faithfully hold what the
source said.

The other half of the cost is silence. Cases 1 and 2 do not fail — they restructure
the graph, and the response reports a created entity the caller never wrote as one.
A caller storing an external payload gets a new entity per nested `id` it brought
along, all of them empty of intent.

## Options, none chosen

1. **Namespace identity: `smut_id`.** Costs a breaking change to every stored
   entity and every reference, plus a migration for existing stores, and an
   awkward read of every entity written before the change. Buys back `id` for
   data, which is what a permissive notebook arguably should never have taken.
2. **Keep `id`, namespace *references*.** An entity node is `{"id": ...}`;
   a reference could be a distinct shape (`{"$ref": "..."}`), leaving a nested
   `{"id": 7}` as data. Smaller break, but now two shapes mean identity and the
   language has a reserved key with a `$` prefix — the same limit the predicate
   language already documents, and therefore at least consistent.
3. **Keep both, disambiguate by type.** A nested `id` that is *not* a nonempty
   string becomes ordinary data instead of a refusal. Removes the exit-2 case and
   leaves the silent-hoist case untouched — half a fix, and it makes the parse
   rule type-dependent, which is harder to explain than "a string id is identity".
4. **Accept and document.** No behaviour change; `smut learn` already states that a
   nested object without an id is ordinary and that an embedded object therefore
   cannot carry `id` as inline metadata. The cost is a documented papercut on the
   most ordinary key in JSON.

## What decides it

Whether smut's job is to hold *entities a caller authored* (ids are the caller's
vocabulary, `id` is fair to take) or *arbitrary JSON the caller is keeping* (ids
are one more field, and taking `id` is a hostile default). The brief calls it a
permissive notebook, which leans toward the second — but the field name is in the
store format, the envelope, `select`, the traversal edges and every example, so
the change is format-wide rather than cosmetic.

## Neighbours

IDE-002 (renaming with referential stability) shares the root cause: identity is
expressed by a bare key in a value tree. If either is ever taken, they should be
decided together — one format migration, not two.
