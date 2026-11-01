# Thread 2 — code map (raw)

Ran: grep -n over src/, counts over src/ tests/ docs/ README.md, reads of
src/entity.rs, src/declare.rs, src/store/{mod,index,journal}.rs, src/learn.rs,
docs/store-format.md; scratch-store probes with the built binary.

- ENTITY_ID at src/entity.rs:17, 27 references in src/.
- reference_id :31; scan_refs :71 (panics at :85-106 on a bad nested identity);
  extract :159; hoist :195 (skips ENTITY_ID).
- declare.rs: normalize :181, declaration_roots :400, changes entry :110.
- journal.rs: entity_id :246; translate refuses a contradicting write set.
- store/mod.rs:226-238 validates root + index agreement only.
- store/index.rs:28 FORMAT_VERSION = 1; :111 refuses other versions.
- Sweep size: 974 literal "id" over 27 files; 223 in
  tests/fixtures/interview/entities.json (66 entities, PROVENANCE.md beside it).
- docs/store-format.md:104-111 (canonical bytes, id first), :70-72 (basename is an
  internal uid), :250-252 (diagnostics say `holds id "b"`).
- src/learn.rs:65-86 (Declaring/References).
- Scratch probes: nested {id:7}/null/"" → exit 2; nested {id:"abc",name:"Ext"} →
  silent entity + reference; {id:"zzz"} → reference, missing; where on an absent
  field → empty result, exit 0.
- src/order.rs:136 uses unreachable! for a validated invariant (convention note).
