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

Note that the id *is* the entity's basename in `.smut/` (the store refuses an
unsafe basename rather than hashing it), so the length also decides how the
directory listing reads. Shorter is in the direction of IMP-003 cluster 3
(opaque filenames), though that item's hash-fallback premise is now stale.

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

Decisions the design must settle, in order:

- **Where the bound lives** — in `Occupied::generate` (the core's collision
  contract, where the loop is) rather than in `Generated` (the shell, which
  supplies candidates and knows nothing of the set). Core is the coherent side.
- **What exhaustion is** — a new refusal rejection, or reuse of an existing
  store/write error. Exit 1 (the store could not be written) reads truer than 2
  (invalid input): nothing the caller supplied was invalid.
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
