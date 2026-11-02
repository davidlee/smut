# Thread 1 — governance applicability (raw)

## Commands
```
doctrine show DEC-026 / DEC-028 / DEC-029 / DEC-017 / DEC-004 / DEC-008
sed -n 45,60p docs/store-format.md
doctrine show IMP-013 / IMP-011 / IMP-008 / ADR-001 / STD-001
```

## docs/store-format.md:49-56 (STD-001 surface)
```
is corruption (see [Corruption](#corruption)).

### Conventions live outside the store

The store layout reserves nothing for conventions: nothing under `.smut/` is read
back as a convention set. `.config/smut/conventions/` in the project root, the user
config home and the shipped binary is DEC-026/028's decided home for the layered
loader tracked by IMP-013; the binary shipped today embeds its one set. A set
declared into a store is an ordinary stored entity, a record of advice given.

## The index

```
