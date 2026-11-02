# DEC-041: `--ids` and `--select` refuse each other; `--brief` stays out

Two small surface questions, decided together because both are about not growing this slice
sideways.

**`--ids` is not a display flag, it is `select:["smut_id"]`** — the exact shape the two-call recipe
extracts. So `--ids --select title` is two different selections in one request, and the options are
last-wins (silent), `--ids` wins (silent, one layer quieter), or merge into `["smut_id","title"]`
(a request nobody typed). Refusing names both flags and costs one sentence, and a silent precedence
rule is precisely the failure class this front-end exists to remove.

**`--brief` is a different seam.** One line per hit is the human reader's other half of the pain,
but it changes the output contract, and the envelope's identity between `q` and `query` is the test
that keeps this one engine. Output shape is IMP-002's presentation work; adding it here would make
that test meaningless and the slice twice its size.