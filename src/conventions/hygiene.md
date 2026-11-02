---
summary: general structure and relations
---
# Conventions: general hygiene

Structure is what makes a store queryable later. These are the habits worth
forming before there is much to query. They are advice, not a schema: smut
prints them and never checks them.

1. Give every entity `type` and `title`. The discriminator is what makes `where`
   usable later, and a title is what makes a rendered result readable.
2. Give anything with a lifecycle a `status`, and record the values in use
   somewhere the store can show you.
3. Choose one direction per relation by the question you will ask. The inverse
   is what `backlinks` answers, so the direction you pick decides which question
   is cheap and which costs a scan.
4. Model containment explicitly with `part-of`, before writing, not at entity 40.
5. Keep the label vocabulary to a handful, define each in one line, and normalise
   when adding one.
6. State whose claim a claim is.
7. Distinguish a withdrawn record from a replaced one. `retracted` means the
   content is withdrawn and kept as a record of its own error; `superseded` means
   something newer replaced it, and `supersedes` is the edge that makes it so. A
   store that says `superseded` retired something "with its retraction intact"
   has collapsed the two, and a `where` on either status then lies — the store
   this release is built from does exactly that, and its own `vocab` is the
   evidence that it was needed.
8. Render in bounded chunks. Render is a human channel; tool output is not.

Seeding a set

  `smut conventions hygiene --format json` prints this set as one entity
  declaration, and piping it into `smut declare` stores it under the fixed id
  `conventions-hygiene`, so running the pipe again updates that entity instead of
  adding another. Re-installing replaces every field the seed supplies — `body`
  included — and leaves the fields you added alone. The seeded body is therefore
  advice, not a record: keep an amendment by editing under a different id or by
  keeping it outside the seeded fields, or expect the next install to replace it.
  The replacement is reported rather than warned about: the reply to that
  `declare` carries the entity as it stood before the batch, `body` included,
  under `previous`, and `--compact` drops that field.

The two-call recipe — traverse for ids, then filter those ids as roots — lives in
`smut learn`, together with the completeness guard that goes with it. This set
states principles and does not repeat the recipe.

`epistemic-records`, a domain-specific second set, is deferred rather than
shipped: its open questions are content decisions for one narrow domain, not
mechanism for this release. Until it exists, naming it is refused by name.
