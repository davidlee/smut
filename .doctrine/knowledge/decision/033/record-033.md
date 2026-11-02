# Exit 2 is for what the caller typed

The exit table is a documented contract, printed by `smut learn` and pinned by tests:

    0  success, including reported missing ids and a truncated result
    1  no store, or the store could not be read or written
    2  invalid input or usage

A layered loader puts a third kind of failure into a command that had none: a set file
that exists and cannot be loaded. The caller typed a valid command, so exit 2 would blame
them for someone else's file; and the store is not involved, so exit 1's existing sentence
does not cover it either.

## The rule

**The line runs between the caller and the filesystem.** Exit 2 stays for what the caller
supplied — an unknown set name, an unknown `--format`, `--format json` without a name —
because those are retryable by reading the diagnostic. Exit 1 covers what the environment
supplied and the caller cannot fix by typing differently: an unreadable layer directory, a
set whose frontmatter is unparseable, a frontmatter key the format derives elsewhere. The
diagnostic names the layer, the path and — where there is one — the line.

The sentence `smut learn` prints for 1 is widened to say so:

    1  no store, or the store could not be read or written, or a conventions set could not be loaded

**A listing that reports a broken set exits 0.** This is not an exception: the table's zero
row already means "success, including reported missing ids", so a report whose content is a
problem is still a successful report. The listing's job is to show what exists at every
layer, failures included; making it exit 1 would mean a caller cannot list their way to the
diagnosis of the file that is breaking it.

Asking for the broken set **by name** exits 1, with the reason. The caller asks for advice
and receives none: that is a failed command, not a report.

## Consequences

- `Error` gains a third variant rather than pretending a set lives in a store. Nothing in
  the diagnostic may say "store" about a file that is only advice.
- Three behaviours need tests that can go red: the listing of a broken set at 0, the
  name-refusal of that same set at 1 with its path, and the unchanged exit 2 for a name
  that resolves to nothing at all.
- DEC-004's ordering survives: reading advice is not reading a store, and the step-1
  filesystem read does not create, lock or recover anything.
