# ISS-001: crates.io README resolves no relative link: docs/store-format.md and LICENSE are dead on the published page

Found while closing SL-001's audit, after the owner published 0.1.0.

## What is wrong

`README.md` carries two relative links:

- `[docs/store-format.md](docs/store-format.md)` — under *What smut promises*, the
  link that carries the durability guarantee's full statement;
- `[LICENSE](LICENSE)` — under *License*, on the page of a GPL-licensed crate.

crates.io renders a crate's README against its own paths, so neither resolves;
both 404 from the published page. This is the same failure the owner already hit
with the brand image, which was absolutised to
`https://github.com/davidlee/smut/raw/main/smut.jpg` in `094005f` for exactly this
reason. The two prose links were left behind.

## Evidence

- Published: crates.io API for `smut` → 0.1.0, created `2026-11-01T08:30:46Z`,
  one version, not yanked. The repo's `README.md` at head is the published bytes.
- `docs/store-format.md` is not in the crate: `cargo package --list` → 37 files,
  and the `.doctrine/`/brief/Nix exclusions are deliberate. So an absolute link
  must point at the repository, not at a packaged file.
- `0.1.0` is immutable, so any cure lands in the next published version.

## Cures, cheapest first

1. **Absolutise both** to `https://github.com/davidlee/smut/blob/main/docs/store-format.md`
   and `.../blob/main/LICENSE`. One-line each, no packaging change, follows the
   precedent set for the image.
2. Ship `docs/` in the crate as well, so a reader of the tarball has the format
   document. Orthogonal to (1): the crates.io renderer would still not resolve a
   relative link.
3. Link the doc via `smut learn` or docs.rs instead. Rejected as the primary cure:
   the guarantee is quoted in the README, so its full statement should be one
   click away from where it is quoted.

## Why it is an issue and not a chore

The link is the only route from the published page to the normative statement of
the store's durability guarantee. A dead link there makes the guarantee look
unfootnoted, and 0.1's promise is precisely the thing a reader should be able to
check.

## Guard worth adding with the cure

Nothing in the repo detects this. A test in the shape of
`tests/readme_example.rs` — parse `README.md`, require every link target to be an
absolute URL or an in-page anchor — would catch the whole class, including the
next person who adds a relative link. `smut` has no excuse for missing it: the
check is a `select` over the README's own bytes.

## Related

- RV-004 F-11 / PHASE-08/VH-1: the README was read and confirmed as a newcomer
  *before* this became visible, because a newcomer reads the repo, not crates.io.
- DEC-010 (documentation set) is not violated — the documents exist and are
  correct; only the published *render* of the links is broken.
