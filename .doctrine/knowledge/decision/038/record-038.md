# DEC-038: The DSL emits a specification and lets `spec::parse` read it

"Adds no semantics" is a claim until something enforces it. Two mechanisms were available, and
they differ in more than taste.

**Build `Spec` directly** from `spec.rs`'s private helpers. Fast, no intermediate value — and the
invariant becomes a promise a reviewer keeps, while the helpers' visibility widens for a caller
that was never meant to see them.

**Emit the specification object and call `spec::parse`.** One allocation slower, and the invariant
becomes structural: the DSL can only say what the specification can hold, because the
specification is what reads it.

The second is worth its cost precisely because of what this slice must prove. The evidence for
"every DSL line equals a JSON specification" becomes a table of token lines against the JSON the
guide prints — the comparison `spec::parse` already performs — rather than two implementations kept
in step by hand and reviewed by hope.

The seam has a price, and it is named. `spec::parse` speaks JSON vocabulary ("`from` must be a
string or an array of strings"), and a human typed a token. So the DSL refuses its own cases first
— an ambiguous token, a JSON operand, a comma list a `!=` cannot carry, an unknown flag — and each
case where a `spec::parse` message can still reach a human is listed in the design rather than
discovered in a transcript.