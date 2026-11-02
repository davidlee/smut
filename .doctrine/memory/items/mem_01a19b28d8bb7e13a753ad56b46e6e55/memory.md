`doctrine knowledge edit decision <ID> --alternatives` and `--consequences` are **comma-separated array setters**. A sentence containing a comma is silently split into multiple array elements — across the whole value, not per item — so the stored array holds fragments of one sentence.

Observed: DEC-007's `consequences` has been stored as `["Two byte forms are specified separately (pretty files", " compact stdout); the select set never determines key order; …"]` — one sentence, split at its only comma. Slug-adjacent cosmetic damage in the short fields ("iterative fixpoint loop", " label as request path") is the same mechanism.

Rules:
- When writing those two fields, keep each element **comma-free** (rephrase with semicolons/em-dashes) or the text will be mangled. Scalars (`--context`, `--choice`, `--rationale`) are not split.
- Do not attempt to "repair" a pre-existing split by re-passing the joined sentence: it splits again at the same comma. Repair needs a comma-free rewrite.
- Audit reading of a facet array should suspect this before treating a fragment as an authoring error.