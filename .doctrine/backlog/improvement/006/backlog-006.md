# IMP-006: Generated ids: 8-character candidates and a bounded collision retry

Found by the operator on first use of 0.2: a generated id is 21 characters of
noise — `amKYdgvGhl0j2JYBa_ppN` for a one-line note. Two separable concerns share
one subject, so they are one item: the *shape* of a candidate, and what happens
when a candidate is taken.

## 1. Length — 21 down to 8

The 21 is not a decision, it is `nanoid::nanoid!()`'s default: `src/cli.rs`'s
`struct Generated` is the shell's only entropy and names no length or alphabet.
No accepted decision or requirement pins it. `smut learn`'s only claim about
generated ids is that an object without an id is given one, and that ids are
opaque — neither promises a length, so shortening is a change to the observable
surface but not a contradiction of it.

Arithmetic, so the tradeoff is on the record rather than felt: the default
alphabet is 64 symbols, so 8 characters is 64^8 ≈ 2.8 × 10^14. A birthday
collision becomes likely (~50%) near 2 × 10^7 ids in one store, which is far past
any plausible single notebook — and a collision is not a correctness risk anyway,
because the retry below draws again. Length is therefore a legibility choice, not
a safety one; 8 is the operator's number and nothing argues against it.

One consequence to settle with the length, not after it: **the default alphabet
contains `-`**, so a generated id can begin with one (measured: 2 of 60 generated
ids in a scratch store did — the expected 1/64). Such an id is legal but only
reachable at the shell through `--` (`smut get -- -E8hVNfI7`), which is exactly
the friction `--` exists to solve for *stored* ids and should not be manufactured
for *new* ones. Options: keep the default alphabet and accept the 1.6%, or draw
from a hyphen-free alphabet. Hyphen-free is the smaller surprise; `_` is
harmless. Whatever is chosen, it wants a test that generated ids avoid the
leading-hyphen case, or a documented statement that they do not.

The public id does **not** reach a path: `docs/store-format.md` makes an entity's
basename an internal uid, independent of the public id and stable until deletion,
and `index.json` is the only place the two meet. The length is therefore a pure
wording change on the human surface — no filename, no directory-listing and no
path-length consequence. The internal uid is a second, independent draw
(`fresh_basename` in `src/store/journal.rs`, also `nanoid::nanoid!()`), invisible
except in diagnostics and in `ls`: out of scope here, and IMP-003 cluster 3's
business rather than this item's.

## 2. Retry — the loop exists; its bound does not

Retry on collision is already implemented and already tested:
`Occupied::generate` (`src/id.rs`) loops until a candidate is unoccupied,
recording each accepted id, and `ScriptedSource` pins the forced-collision cases
— `generation_avoids_the_snapshots_occupied_ids` (2 draws),
`generation_avoids_an_explicit_id_in_the_request`,
`generation_avoids_an_earlier_generated_id` (3 draws), plus the same three at the
declaration level in `src/declare.rs`. The `Occupied` set deliberately covers the
snapshot, the request's explicit ids and this batch's earlier generations, so a
slipped collision can never become an accidental upsert (DEC-001 rule 3).

What is missing is the *bound*. The loop is unbounded by contract — `IdSource`'s
doc comment says as much: "a source that never yields a free candidate loops by
contract, not by accident". Against real entropy that is fine and unreachable;
against a degenerate source it is a hang with no diagnostic, which is a worse
failure than a refused declaration. A max-retries constant (16 draws is already
absurd) turns the spin into a refusal, and gives the test something to assert:
a source that always repeats one occupied candidate must *terminate*, not loop.

A second, unbounded redraw loop exists for internal uids: `fresh_basename_from`
in `src/store/journal.rs` redraws until a basename is free of the taken set and
the directory. Whatever bound the public-id loop gets, the same question is asked
of that one; the shareable thing is a constant and a failure shape, not a loop,
because the two draw different types and one of them probes the filesystem.

Decisions the design must settle, in order:

- **Where the bound lives, and what it can report.** `Occupied::generate` is the
  truthful site — the collision test is there, and `Generated` knows nothing of
  the set — but it returns `Id`, so bounding it means returning
  `Result<Id, Reject>`. DEC-001 rule 1 binds `Reject` to exit 2 *because the core
  cannot express anything but user error*, and an exhausted source is not user
  error: the bound collides with an accepted decision. Options: widen the core's
  error to admit an internal-invariant class (reopening DEC-001 rule 1), assert
  the invariant instead of reporting it (panic, exit 101), or leave the core
  unbounded by contract and place the bound in the shell's source. This wants the
  operator.
- **What exhaustion is** — contingent on the above. My earlier guess (exit 1, the
  store could not be written) is precisely what the layering currently cannot
  say.
- **Whether the constant is observable** — recommended no; it is an internal
  guard, and `smut learn` should not teach a number a caller can do nothing with.
  The guide's id paragraph gains at most a clause about the shape of a generated
  id, if the alphabet changes.

`ScriptedSource` panics when its script runs out, so the always-colliding test
either repeats one candidate n times or gains a small `RepeatSource`; the panic
is the right behaviour for the exhaustion-of-script case and should not be
loosened to make the new test easy.

## Touches

`src/id.rs` (length is not here — the bound and its tests are), `src/cli.rs`
(`Generated`'s alphabet and length), `src/declare.rs`'s generation tests (drawn
counts survive; anything asserting a 21-character shape does not — none found),
`smut learn`'s declaring paragraph if the alphabet changes, and the README's
declaration examples, which currently show a generated id inline.

## Ready to pick up when

Immediately — independent of 0.2's closure, no dependency, no store-format
change. Small enough to ride along with any phase that already touches
declaration, and cheap enough to be its own.

## Outcome — implemented 2026-11-01

Landed outside slice governance, by operator direction ("don't slice it,
implement it"). Both halves are in, plus the second loop.

**Length and alphabet** — `src/cli.rs`: `ID_LENGTH = 8` and `ALPHABET`, nanoid's
`SAFE` set with the hyphen removed (63 symbols, `_` kept). The literal is written
out so a draw allocates nothing, and a test pins it against
`nanoid::alphabet::SAFE` filtered of `-`, so the two cannot drift. Measured
against the built binary: 200 generated ids, all 8 characters, all unique, none
containing a hyphen. Explicit ids are untouched — `{"id":"-weird"}` still needs
`--` and `{"id":"a"}` is still accepted.

**The bound, and how it reports.** This is where the item's guess was wrong and
the decision went the other way. `MAX_CANDIDATES = 16` (`src/id.rs`) now bounds
`Occupied::generate`, and exhausting it **asserts** — it does not return an exit
code. The reasoning, since it reverses what this item proposed: the brief fixes
two failure classes and `error.rs` says "exactly these two classes, and no third"
(exit 1 storage, exit 2 input). An exhausted id source is neither — it is our own
broken contract, since `nanoid` draws fresh entropy every call and 63^8 ≈ 2.5 ×
10^14 — so reporting it as either class would be a lie, and a `Reject` variant
would have contradicted DEC-001 rule 1 as well. Asserting an unreachable
invariant is the codebase's existing convention (`unreachable!` in `src/order.rs`),
and IMP-005's "refuse rather than panic" is about malformed *user data*, which
this is not. Test: `generation_terminates_after_the_candidate_bound_rather_than_spinning`
catches the panic and pins both termination and the draw count.

**The second loop, also bounded.** `fresh_basename_from` (`src/store/journal.rs`)
redrew forever on a claimed basename. Here the failure *is* a store condition, so
it reports through a new `StoreError::NoFreshBasename { attempts }` — exit 1, no
governance tension. One constant governs both loops.

**Not touched, deliberately.** `smut learn` gains nothing: it promises no length,
and stating 8 would turn an opaque id's shape into a guarantee. The README needed
nothing — it shows only explicit ids, so this item's earlier claim that it carried
a generated one inline was wrong. `docs/store-format.md` is untouched: basenames
are internal uids, unaffected by public-id length.

Gate: `just check` clean (build, both clippy configurations at `-D warnings`,
`cargo fmt --check`, both feature test runs — 350 tests).

## Addendum — uid alphabet, by operator direction

The uid keeps its 21 symbols (nobody types it, nobody reads it, so the only thing
to buy is collision reach: 63^21 ≈ 10^37). What it drops is the hyphen. A leading
hyphen in a basename is harmless, so this is not the public id's reason — it is
consistency: two generated shapes that differ only in *length* are easier to
recognise than two that differ in alphabet as well.

The symbol set therefore moved out of the shell and into `src/id.rs` as
`HYPHEN_FREE`, next to `MAX_CANDIDATES` — both are shared shapes of generated
identity rather than one caller's choice. `src/cli.rs` keeps `ID_LENGTH = 8` and
its draw; `src/store/journal.rs` gains `UID_LENGTH = 21` and draws from the same
set. The pin against `nanoid::alphabet::SAFE` moved with the constant; the uid
draw gets its own test, drawing 200 names because one draw would not distinguish
the alphabets (a hyphen is a 1-in-64 chance per symbol). Measured on the built
binary: 40 entities, every basename 21 symbols plus `.json`, no hyphen anywhere.
