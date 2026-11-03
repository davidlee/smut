# Conventions layer from a mirrored config home

A conventions set is direction; a store is data. That distinction decides where
sets may live, and it decided this shape.

## The layers

One relative suffix, three roots, local wins:

    <project-root>/.config/smut/<kind>/     the project's, travelling with the repo
    ~/.config/smut/<kind>/                  the user's
    the binary                              what ships

`<project-root>` is the walk-up terminus — the git root — so the current working
directory never enters the lookup and a subdirectory cannot see a different set
of advice than its repository. `<kind>` is the artefact kind (`conventions` today,
`schemas` if that day comes), which is why a second artefact kind adds no dot-entry
to a user's project.

## What makes this one mechanism rather than three

**Structural equivalence.** A project set and a user set are the same artefact
found earlier: same form, same loader, same listing. The operator's reason is the
right one — it reduces sharing friction, because "trade a set" and "override the
built-in" become the same operation at different scopes. The shipped set is not
special either; it is the lowest layer, not a privileged one.

**Total precedence, with origin always printed.** Project beats user beats binary.
Same-named sets do not merge: one wins and the other is reported as shadowed. Every
listing names the layer and the file each set came from, so a reader can always tell
where advice they are following originated — which is also the cheap cure for the
one real cost of this shape (below).

**An environment override**, so a monorepo or dotfiles user can point at one config
deliberately instead of fighting the default.

## The cost we accepted

`.config` in a repository already means "this directory is a home directory" —
dotfiles repositories, container home mounts, the thing that gets symlinked into
`~/.config`. XDG scopes the config home to the user, so the mirror borrows a name
whose established meaning is the opposite of project scope, and a reader meeting
`.config/smut/` in a fresh checkout may assume it is meant to be copied home. That
is a documentation problem with a one-line cure — origin printing plus a listing row
that names the file in force, which is what DEC-031 kept when it declined a header in
printed prose — rather than a structural one. The alternative was a permanent
second dot-directory to avoid writing that line.

## Not decided here

- **The name.** The verb and the leaf directory share it; `conventions` is the
  best-hedged candidate because it is already the word for "not enforced", while
  `recipe` collides with the guide's own "two-call recipe". IMP-013 owns this.
- **Seed identity** across layers, and what re-installing means when a same-named
  set exists at two scopes.
- **Malformed sets**: refusing by name and origin while keeping the other layers
  working.
- **The no-.git fallback** for the walk-up terminus.

## Why this survives a schema future

If user-definable typed schemas ever arrive they are a second artefact kind under
the same suffix, with their own verb, their own precedence and the enforcement —
loaded through the same ladder, which is why the loader is specified as generic over
declaration kind. Conventions keep never validating. User sets are also the cheapest
schema-discovery mechanism available: real sets show which constraints people
actually write before any enforcement model is committed to.

## Boundaries

Nothing appears inside `.smut/`: the store's layout stays owned by
`docs/store-format.md` under STD-001, and the sentence "whatever is stored there is
data" keeps no exception clause. A seeded entity in the store is a record of advice
given, never a source the binary reads back.