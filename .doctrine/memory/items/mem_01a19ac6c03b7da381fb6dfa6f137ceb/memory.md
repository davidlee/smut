# What makes an assertion evidence

A test assertion earns its place only if some plausible change to the code turns it red.
An assertion that no input can falsify still occupies the *evidence slot*: it reads as proof
of a property in the plan, the design and the phase sheet, and it silently stops anyone
looking for the real proof.

SL-005's binary refusal test asserted `!scratch.has_store()` with the failure message "the
request is validated before the store is opened, so no store exists". It cannot fail. The
test drives `query`, `Query::creates_store()` is `false` (`src/cli.rs:121-123`), and
`Store::run` returns `StoreError::Absent` without touching the filesystem when the store is
absent (`src/store/mod.rs:277-284`). Under *every* ordering of `spec::parse` against
`store.run`, `.smut` is never created. The ordering that test claimed to prove was actually
proved by the neighbouring `assert_eq!(refused.code, 2)`: a request validated after the store
step exits 1 with `no store at …`.

The idiom was not wrong where it was born — for `declare`, which *can* create a store, the
same assertion discriminates.

## The check

For each negative assertion, name the change that would make it red. If the answer is "none
within the command's reachable behaviour", it is documentation wearing an assertion's
clothes. Then find the assertion in the same test that *does* discriminate, and make sure the
claim narrative (design table, plan criterion, phase sheet) is attached to that one.

Related: it is the same failure as `mem.fact.doctrine.vt-inert-keywords` one level up — a
check whose signal is nil, recorded in a narrative that assumes signal. Substring presence
versus behaviour, and unfalsifiable assertion versus behaviour, are the same defect class.

## Strength ordering hollows out the weaker assertions

A listing test that asserts the exact row first, then `starts_with(…)`, then `contains(…)`, reads
well and leaves two assertions with no input that could redden them: once the exact row is pinned,
the weaker ones are implied. Put the strongest assertion **last**, so each weaker row still has a
world where it fails and the strong one would not.

The same arithmetic across a test file: an assertion is only worth its line if some *other*
assertion in the same test would not already have caught the defect.
