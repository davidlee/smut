# Put the walk in a wrapper, never at the end of the recursive function

`predicate::parse` re-enters itself for every clause operand (`parse_all` at
`src/predicate.rs:101`, `parse_not` at `:116`). A whole-AST post-check placed at the
end of `parse` therefore runs once per operand, with the position flag the outermost
call had — so a check that must only fire where a field is *demanded* fires inside
`$or` and under a single `$not` too, refusing predicates that are satisfiable.

The shape that works: keep the recursive builder private, and make the public `parse`
build the AST once and then run the check exactly once on the root.

Why it matters beyond smut: "call the check once at the end of `parse`" is the natural
sentence to write in a design, and it is silently wrong for any recursive parser whose
check carries context. When reviewing such a design, ask whether the named call site is
the outermost one or every one.
