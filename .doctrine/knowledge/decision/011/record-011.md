# The 0.1 query-spec surface

Settled 2026-11-01.

## Accepted keys

`from`, `select`, `order`, `limit`. Types as in the brief:

- `from`: string or array of strings; `[]` selects nothing; absent selects all.
- `select`: array of strings; duplicates behave as a set.
- `order`: array of `{field, direction?}`; `direction ∈ {asc, desc}`, default
  `asc`; unknown keys inside an entry are rejected; `[]` equals absent.
- `limit`: positive integer; 0 rejected.

`query {}` returns **all entities subject to the default limit** — not a list of
ids. A non-object spec is exit 2.

## Rejecting what arrives later

Recognised future keys are rejected by name, with a message naming the release
that will accept them (`where`, `follow`, `backlinks`, `depth`, `budget` → 0.2).

**Arbitrary unknown keys are reported plainly as unknown** — no release is named
for a key nobody planned. Naming a release for an unrecognised key would be a
guess presented as a promise.

## What rejection does and does not guarantee

Rejecting future keys keeps the *syntax* additive: a 0.1 call that works must not
become invalid when the key set widens. It does **not** guarantee unchanged
*behaviour* for every future grammar. Compatibility is protected by keeping
representative 0.1 invocations as tests, not by the act of rejecting today.
