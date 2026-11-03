# CHR-006: src/conventions.rs residuals from SL-008's audit — load's filesystem arithmetic, and user_root's empty-home guard and its missing pin

Two residuals the SL-008 audit (RV-035 F-4, F-5) confirmed and deliberately did
not repair, because the audit stage does not write code: `src/conventions.rs`'s
last change is PHASE-05's concluded delta (`e54f7aa..c6cebe2`), and a source edit
after that would leave the phase registry describing code no phase concluded.

## 1. `load`'s doc counts three filesystem reaches where there are two

`src/conventions.rs:436-439` (added by `1ced7c8`, the commit that discharged
RV-032 F-3):

> This module reaches a filesystem in three places, and this is the middle one:
> `project_root` above walks the ancestors …, this function turns each root into
> that layer's one `<root>/<kind>` directory, and `read_layer` below reads the
> directory's entries.

`load` touches no filesystem — it joins two paths, delegates to `read_layer`,
sorts and extends. The reaches are two: `project_root` (`symlink_metadata`) and
`read_layer` (`read_dir`, `metadata`, `read`). `binary_layer` reads nothing (the
bytes are `include_str!`).

Smallest correction: "This module reaches a filesystem in two places:
`project_root` above … and `read_layer` below; `load` only joins and orders what
they return."

## 2. `user_root`'s empty-`home` ruling is unimplemented, and its stated pin does not exist

`notes.md` § Harvest → Open (and the SL-008 handover) records the seat's ruling
that "the non-emptiness EX-2 already applies to `config_home` reads onto `home`,
since an empty `home` resolves no config home either", **"pinned by one row in
PHASE-04's owned `src/conventions.rs`"**. Neither half holds:

- `user_root(None, Some(Path::new("")))` returns `Some(".config/smut")` — a
  relative layer root resolved against the cwd. The non-emptiness test is applied
  to `config_home` only (`src/conventions.rs:110-115`).
- No row exercises it. `an_empty_config_home_falls_through_to_home` passes the
  empty path as *`config_home`*, not as `home`.

Unreachable from the binary (`home_dir()` is absolute or `None`), reachable from
the crate's public API, and the same "relative root reads the wrong layer without
a symptom" class the function's own doc comment argues against.

Fix: `home.filter(|home| !home.as_os_str().is_empty())` on the fallback, plus one
row — `fn an_empty_home_is_an_absent_layer` — asserting `None`. If the ruling is
reversed instead, correct `notes.md`; the audit's harvest has already amended the
"pinned by one row" claim, which was false in either reading.

## Provenance

RV-035 F-4 and F-5 (`doctrine review show 35 -f 4,5`). Neither is a design or
governance item, so neither is reconcile's; both are owned code work.
