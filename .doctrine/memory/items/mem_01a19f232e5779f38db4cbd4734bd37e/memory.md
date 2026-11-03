# The situation

SL-008 PHASE-03, attempt 1. Two criteria said the loader must distinguish an absent layer from an
unreadable one, and must sort each layer by name. The code did neither. The doc comments carried
the whole response to both criteria:

    /// This phase makes "absent" and "unreadable" the same thing, so nothing here errs yet.
    /// Nothing is sorted or deduplicated here.

The same contradiction in code — a bare `Vec<Entry>` return, a `continue` on every error — is what
the tests caught. In prose it survived an entire run and a green suite, because a contradiction
written as a doc comment reads like an implementation note.

# The rule

- When a criterion and the code disagree, **the doc comment is where the disagreement hides**.
  Read the doc comments covering the criterion before reading the code.
- A doc comment that *decides* a question ("nothing here errs yet", "we treat these the same") is a
  decision: it must be traceable to a criterion, a design section or an ADR. Untraceable, it is a
  finding.
- Reviewing a diff, treat persuasive prose next to load-bearing code as the highest-risk region,
  not the lowest — it is the part no test can redden.


# The second failure mode: the sentence written to discharge the finding

SL-008's audit (RV-035) found the same slice had produced **four false doc
sentences**, and **two of them were written by the fixes for earlier ones**:

- RV-032 F-3: `load`'s doc claimed a filesystem-read monopoly it did not have. The
  fix (`1ced7c8`) replaced it with "This module reaches a filesystem in three
  places, and this is the middle one" — `load` touches no filesystem, and there are
  two reaches. (RV-035 F-5.)
- RV-033 F-6/F-8: two sentences asserting where `.config/smut` is joined, both
  false; corrected in `e692735` into a third wording that is now accurate but was
  never falsified against `user_root`'s home fallback.

The rule this adds to the one above:

- A doc sentence **introduced to discharge a finding** needs the same falsification
  as the code: name the fact it asserts and check the fact, not the finding's
  closure. A fix's diff is the *highest*-risk prose in a review, not the safest,
  because the reviewer who raised the original finding reads the new sentence as
  the fix rather than as a new claim.
- A count ("in three places", "the middle one", "the only place") is a fact with a
  falsifier; a rule without a count usually is not. Prefer the rule.
- Raise it as its own finding even when it is one line and the behaviour is
  correct — three rounds of RV-032/RV-033 closed on "docs honest" while the next
  sentence was already wrong.
