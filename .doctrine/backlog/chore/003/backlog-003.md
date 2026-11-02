# CHR-003: SL-005's locked design.md still carries six sentences the implemented rule falsifies

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found while hardening SL-005's plan (2026-11-02). The slice's design is **locked**
(`dr-01a19a1e`, rev 43), so these were not repaired in place: editing the materialised
design would desync the run's fingerprints for prose that changes no behaviour. The plan
implements the correct behaviour in every case; each sentence below is a description
that has drifted from the rule it describes.

1. **§1's diagram label, `required-position walk: each Field is judged only when
   reached through And`, is false for a bare root field.** The root `Field` is judged
   without any `And`. This is the residue of the "crossed `And` only" error DEC-027
   already names in its provenance.
2. **§2's "Shape of the check" and §6's `src/predicate.rs` row say the walk is "called
   once at the end of `parse`".** `parse` re-enters itself through `parse_all` and
   `parse_not`, so a walk installed there runs at every clause operand and refuses a
   contradiction inside `$or` — the exact defect RV-026 F-1 raised. The correct shape,
   which §1's diagram already shows, is a private recursive builder plus a public
   `parse` that runs the walk once.
3. **§5's binary-level row does not say whether the pinned stderr line carries the
   `smut: ` prefix**, while the message-level pin beside it is the bare sentence.
   Both strings should be given.
4. **§6 says "including the four pins"; §5 defines three** (`a_contradiction_spanning_and_clauses_is_not_refused`,
   `a_single_disjunct_or_is_not_refused`, `an_unbounded_contains_is_not_refused`), and
   RV-026 F-17 enumerates the same three.
5. **§7's "`$or` contents are not analysed" over-claims.** An `$or` that must not hold
   hands its children `false`, so a `$not` beneath it restores demand and the field *is*
   judged. The single-disjunct pin and the `$or` example are still right; the summary
   sentence is not. Same over-claim in DEC-027 (amended this session).
6. **§3's second sentence, `$exists: false beside any other condition`, names one
   operator and no empty list**, while DEC-030's own criterion asks each sentence to
   name the two operators or the empty list it is about. One of the two should move.

Also recorded, and not a design defect: §1 says "required-position" where §2-§6 and the
plan say "demanded" — one word should win.

Sink: the slice's audit/reconcile seam is the designed place to reconcile design prose
against implemented behaviour; batch this with that pass rather than reopening the run.
