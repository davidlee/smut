# Thread 1 — governance applicability (raw)

Ran: doctrine adr list, doctrine spec list, doctrine policy list, doctrine standard
list, doctrine show DEC-00x/01x, grep smut-implementation-brief.org.

Findings:
- ADR-001 accepted — binding, governs this slice.
- DEC-005 (entity node shape + "embedded object carrying id cannot be inline
  metadata"), DEC-007 (id-first serialisation; select: [] yields id only),
  DEC-010 (documentation repeats the nested-id limit), DEC-016 (learn recipe spells
  select:["id"], where:{"id":...}), DEC-002 and DEC-012 (examples) all carry `"id"`.
- DEC-001 (id-keyed snapshot), DEC-008 (`--` for opaque ids), DEC-009 (rendering)
  unaffected.
- No specs, policies or standards exist — checked, not applicable.
- smut-implementation-brief.org:33 fixes "an entity is a JSON object with a nonempty
  string =id=" and :47 fixes DROP and identity immutability. The brief is a repo
  file, not a doctrine entity; :33 contradicts ADR-001 and nothing will flag it.
