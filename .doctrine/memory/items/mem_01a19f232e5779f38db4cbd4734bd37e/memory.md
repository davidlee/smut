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
