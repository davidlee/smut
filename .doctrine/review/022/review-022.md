# Review RV-022 — code-review of ISS-002

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Second opinion on candidate v0.2: ADR-001, IDE-002, IMP-005, IMP-007,
ISS-002 and ISS-003. The locus is ISS-002; evidence spans identity extraction,
declaration normalization, response serialization, CLI parsing, store loading
and the format contract. No implementation or acceptance changes are in scope.

Probe whether the proposed spelling solves the actual collision, whether minting
and rename depend on that spelling, which contracts need freezing before release,
and whether the linked records agree. Validate representative behavior in scratch
stores against a fresh local build. Existing README edits belong to the user.

## Assessment

**Recommendation: proceed with ADR-001's key and envelope decision after correcting
the linked-record drift and narrowing its rationale.** The architecture does not
need a new identity abstraction for v0.2. The dangerous part is the current `id`
collision: ordinary imported data can silently become graph structure. Moving the
reservation to `smut_id` is a proportionate correction while the consumer surface
is small.

This is a completed second-opinion pass, not ADR acceptance or release sign-off.
Findings remain open for response. Read them with `doctrine review show RV-022
--full`: F-1 is the competing proposed/accepted and envelope contract, F-2 is the
overstated reservation/dependency argument, F-3 reconfirms existing IMP-005.

### ADR-001 — support, with precise boundaries

Use `smut_id` consistently in entity roots, nested references, returned entities,
`changes[].previous`, convention declarations and `changes[].smut_id`. Keeping
container names such as `ids`, `missing` and edge `source`/`target` is coherent:
they describe response structure, while `smut_id` names the field consumers also
use in predicates and projections. Changing the change-record field is a surface
choice worth making now; the current proposal makes that choice clearly enough.

The rename **reduces collision risk**. It does not abolish reserved spellings:
an imported object carrying `smut_id` will still be interpreted structurally.
Document that boundary without promising this name can never collide. Nor does
the rename make every JSON value literal: a top-level entity field equal to the
string `DROP` still means removal. A fresh probe confirmed that it disappears
while the same string inside an embedded ordinary object survives. The original
brief explicitly chooses that behavior, so it is not a new implementation bug.
For this release, explicitly retain that patch convention and describe the tool
as reserving `smut_id` and a top-level patch sentinel. A future requirement to
store every JSON value literally would need a separate design conversation.

The deliberate version break with no migration or compatibility reader is
reasonable within the operator's stated direction. Bump the actual on-disk
format version independently of the package version. The implementation must
update both index and journal format checks, not only entity extraction. Prove
that old clean stores and old committed journal manifests are refused before
their old-format content is interpreted or replayed. Existing recovery rules
still apply; this is not a request for a new migration or repair path.

Acceptance should cover the **contract**, not just replacing the constant:
ordinary `id` must survive at roots and at depth, with null, numeric and string
values; `smut_id` alone must continue to mean a reference; identity plus fields
must retain hoisting and upsert behavior. Cover predicate/projection/traversal
field spelling, response serialization, conventions, fixture data and the
version-matched guide. A stale client sending `id` after the rename is ordinary
data by design; there is no honest way to detect that intention while also
freeing `id` for arbitrary payloads.

### ISS-002 — do now

This is the material breaking change. Verified against a fresh build: declaring
`{"id":"x","payload":{"id":"external","name":"Ext"}}` creates two entities
and stores only the reference under `x.payload`; numeric nested identity is exit
2. Namespacing the key directly addresses both. Reconcile this item's obsolete
accepted-status claim and `changes[].id` recommendation before implementation
treats either as authority.

### ISS-003 — support minting, separate its contract

Pre-minting ids gives callers the names needed to author a cyclic graph in one
atomic declaration, without placeholders or marker syntax. It still requires
two invocations: mint, then declare. That is useful because the first call has no
store effect and the graph is published in one mutation. It does not change the
rule that an unmarked nested object is embedded data.

This works under either key spelling. Do not bind its implementation or release
schedule to the format break merely because the items were discussed together.
If included now, my proposed minimal contract is `smut new --count N`, default
one, returning a stable JSON object `{"ids":[...]}` even for one result. Reject
invalid/nonpositive counts as input errors; specify an allocation bound. Keep
ids opaque and unique within the result, with no reservation. State explicitly
whether minting consults an existing store: I favor a store-independent generator
that also works before `.smut/` exists, with probabilistic uniqueness and the
already accepted collision/upsert exposure. That is a proposal, not a decision
recorded on behalf of the operator. Store-aware collision avoidance is possible,
but would introduce store access/recovery and still cannot reserve a returned id.

### IDE-002 — defer; do not declare resolved

Generated ids make semantic naming optional; they do not fix existing semantic
ids, imports, merging stores, or identity mistakes. A bad opaque id is still
permanent. Nevertheless a future explicit rename can be additive: preserve the
internal filename, replace the public key and rewrite every incoming structural
reference under one journaled batch. There is no demonstrated need to implement
it before the spelling change. If implemented later, destination collisions,
missing sources, cycles and receipt semantics need a contract. A reference to a
deleted entity remained dangling in the scratch probe, confirming that ordinary
delete/redeclare is not a rename substitute.

### IMP-005 — include with the format work

Confirmed independently: after manually inserting `bad:{"id":null}` into an
indexed entity, `get` succeeds and traversal exits 101. The panic even occurs
when the traversal asks for a different field, because reference scanning visits
the entire entity. This is a loader-precondition gap, not an argument against
assertions inside code that actually receives validated values.

The new loader should recursively validate the structural identity rules and
reject corruption with exit 1 and the offending entry named. After the rename,
validate `smut_id`; ordinary payload `id` must remain unrestricted. Include the
corruption regression and format-document rule already requested by IMP-005.
This is useful hardening while touching the boundary, not a second format break
or evidence that CLI-authored stores currently corrupt themselves.

### IMP-007 — optional, additive ergonomics

Short aliases can ship later without breaking the long forms, so they should not
delay locking identity. If adding them now, my preference is `-f` format, `-i`
show ids, `-s` show fields, `-H` heading field, `-L` heading level and `-c` compact.
Consistency means a letter retains the same meaning wherever that option exists;
it does not require an alias for every future flag. Keep long forms canonical in
examples, expose aliases in help, and preserve `--` for hyphen-leading ids. These
letters are a preference, not a correctness finding.

## Ranked next steps

1. Reconcile the proposal's records, then lock the exact identity/envelope
   contract and explicit remaining reservations with the operator.
2. Implement the format break together with IMP-005 and verify both old-format
   refusal and new-format identity semantics at the public CLI boundary.
3. Settle the minimal minting contract independently; leave reference-preserving
   rename deferred and treat short aliases as optional additive work.

## Verification and limits

`cargo build --locked --offline` passed on the current source. Fresh debug-binary
probes ran only in temporary stores: nested-id collision, invalid nested-id
refusal, a reciprocal graph declared in one batch, traversal, delete-induced
dangling references, malformed-store get versus traversal, and literal `DROP`
behavior. These observations support the current implementation claims above.
The proposed `smut_id`, minting and rename behavior is unimplemented and was not
runtime-tested. No full regression suite, crash/durability suite, release build
or publication check was run. No code, ADR acceptance, backlog status, or existing
README edits were changed.

## Harvest

Produced: RV-022 records the second opinion and three findings. Learned: current
behavior independently reconfirms the existing issue evidence; no new memory
record is needed. Open: operator acceptance remains on ADR-001; authoring and
rename choices already have ISS-003 and IDE-002 as durable homes. IMP-005 and
IMP-007 retain their existing ownership. No duplicate backlog items or governing
slice were created for this review.
