# IMP-019: every_store_refusal() omits NoFreshBasename, so the inventory pin cannot notice a missing row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found while discharging RV-030 F-1 (`SL-008` PHASE-02).

`src/error.rs` hand-maintains three lists that notice a new refusal. Two are honest: the
`Display` arm is compiler-enforced by match exhaustiveness, and the boundary table
(`[(Error, u8); 27]`) is pinned by its own length. The third, `every_store_refusal()`, is
neither: `StoreError` holds **14** variants — `NoFreshBasename` among them — while the vec
holds **13**, and the test pins `len() == 13`.

The consequence is narrower than it looks, and worth stating precisely. The pin does fire
when a row is added without the count moving, which is the mistake people actually make.
What it cannot do is notice a variant that never got a row at all: `NoFreshBasename` has
been missing since it was introduced and the suite is green, and a fourteenth variant added
tomorrow with no row would leave the count at 13 and also be green. Verified in `SL-008`
PHASE-02 — `src/error.rs:508-556` (the vec) against `:160-257` (the enum), with the mismatch
admitted in a comment at `:506-507`.

Two ways to close it, either acceptable:

1. Add the `NoFreshBasename` row and move the pin to 14. Cheapest, and it makes the list an
   actual inventory of the enum for the first time.
2. Make the list self-checking — assert every `StoreError` variant appears, or move the
   inventory's key off a hand-typed count. More work; the only form that makes a *future*
   omission impossible rather than merely detectable at review.

Deliberately not absorbed into `SL-008` PHASE-02: the omission predates that phase and
fixing it is outside its six-file charter. The claim in that phase's own diff line
(`:763-768`) was corrected there — the message now asserts only what the pin enforces.

## Second site, same class (found in `SL-008` PHASE-02's corrective commit `8503b93`)

`src/error.rs:505-508`, the doc comment on `every_store_refusal()` itself, still says the
pinned count "moves when a variant is added". That is the same overclaim the assertion message
made before F-1 corrected it, one screen above the correction: the count does not move when a
variant is added without a row. Left in place because the disposition scoped F-1 to the
assertion message and a one-line doc comment was not worth a second worker round-trip — but it
belongs with this item, since both sites are wrong for the same reason and both are fixed by
whichever of the two approaches above is taken.

## Resolution (2026-11-10)

Option 2, without a dependency. In `src/error.rs` tests:

- `ordinal(&StoreError) -> usize` is an exhaustive match with no wildcard, and
  `const STORE_VARIANTS: usize = 14` sits beside it. A new variant fails to compile
  next to the inventory.
- `the_store_inventory_holds_each_variant_once` asserts that the sorted ordinals of
  `every_store_refusal()` equal `0..STORE_VARIANTS`, so a missing or repeated row is red.
  It was first run red with `NoFreshBasename` missing (ordinal 12).
- Added the `NoFreshBasename { attempts: 64 }` row. Removed the hand-pinned
  `len() == 13` and the overclaiming doc comment that went with it (both sites).

Remaining gap: `ordinal` and `STORE_VARIANTS` are still edited by hand. A variant given
an ordinal without a bumped count and without a row would pass. The compile error puts
all three edits on one screen, which is as far as std alone goes.
