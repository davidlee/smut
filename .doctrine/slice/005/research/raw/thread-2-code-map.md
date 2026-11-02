# Thread 2 — code map (SL-005)

Producer: orchestrating agent, degraded mode (no project research agents defined).

## Hotspots — files this slice is expected to change

| file | site | fact |
| --- | --- | --- |
| `src/predicate.rs` | `parse_field`, `src/predicate.rs:124` | The only place a field's `Condition` list is built. Line 152 `.collect::<Result<Vec<Condition>, Reject>>()?` is the last moment the whole list exists as one value — the contradiction check's natural home. |
| `src/predicate.rs` | top-level `$` arm, `src/predicate.rs:71` | ISS-004's defect: any `$` key at predicate level that is not a clause returns `Reject::UnknownPredicateOperator`. The operator list is complete; only the message is short. |
| `src/error.rs` | `enum Reject`, `src/error.rs:18`; `MixedPredicateObject` at `:69` | One variant, one `Display` arm (`src/error.rs:305-315`) is the shape every neighbour already uses. |
| `tests/predicates.rs` | `a_malformed_predicate_rejects_the_whole_request_before_any_store_is_opened`, `tests/predicates.rs:549` | Refusals asserted through the binary: exit 2, **empty stdout**, **stderr contains the offending name**, and **no store created**. A new refusal test belongs in this table. |
| `src/error.rs` | `the_boundary_maps_each_failure_class_to_its_exit_code`, `src/error.rs:495` | A **fixed-size** `[(Error, u8); 24]` table — adding a variant obliges a bump to 25 plus a row, and nothing forces it (the array is not exhaustive over variants). |
| `src/predicate.rs`, `src/error.rs` | unit refusal tests `src/predicate.rs:304,354,396`; message tests `src/error.rs:757-773` | The pattern for a new variant: one unit test of the returned variant, one of the rendered text. |

## Cited facts the design can lean on

- **One call site.** `predicate::parse` is reached only from `src/spec.rs:132`
  (`"where" => spec.r#where = Some(predicate::parse(value)?)`), inside the spec
  parse that DEC-004 pins as complete before the store is opened. So the check
  costs no new plumbing and cannot accidentally run after store access.
- **No-store property is a module invariant.** The module doc
  (`src/predicate.rs:1-16`) claims "this module and everything it reaches touch
  neither the store, nor the clock, nor the filesystem (VA-1)". A check over a
  `Vec<Condition>` preserves it; anything reaching a store would break it — the
  reason DEC-027 refuses the "no scanned root carries that field" case.
- **Structural equality is `serde_json`'s.** `Predicate::holds` uses `==`
  (`src/predicate.rs`, `Condition::holds`), and the unit test
  `structural_equality_is_serde_json_equality` pins that `1 != 1.0` and that object
  key order is irrelevant. A contradiction test must use the same `==`, or it will
  disagree with evaluation.
- **Presence is per-operator.** `Condition::holds`: `Eq`, `Ne`, `In`, `Contains`
  all require `Some(_)` (`is_some_and` / match on `Some`); only `Exists(false)`
  holds of an absent field. This is what makes "`$exists:false` with any other
  condition" sound as a *rule* rather than a list.
- **`$ne` needs presence.** Pinned in `each_operator_holds_exactly_where_the_table_says`:
  "`$ne` needs the field present: absent is not 'not equal'".
- **The `$`-reserved disambiguation.** `parse_field` counts `$`-prefixed keys
  (`src/predicate.rs:132`); all-`$` is an operator object, none is a literal, mixed
  is `MixedPredicateObject` naming the field (`:144`).
- **Guide already covers the class.** `src/learn.rs:116` says an unknown key, an
  unknown operator, a malformed predicate and the rest "are all exit 2 and write
  nothing"; `src/learn.rs:122-127` describes the disambiguation rule and the field
  operators. No guide sentence is added by this slice, and the guide pin
  (`src/learn.rs:377`) is unaffected.
- **stderr is one line.** `src/main.rs:114-116` renders `smut: {error}\n`. A
  multi-sentence refusal is one long line; there is no second-line channel.
- **Existing pins the new message must not break.** `src/error.rs:764-767` asserts
  the `UnknownPredicateOperator` message *contains the operator name*;
  `tests/predicates.rs:549` asserts `{"where":{"$foo":1}}` produces stderr
  containing `$foo`. The name stays (ISS-004's own constraint).

## Naming precedents for a new variant and message

- Variant names are noun phrases for the defect: `MixedPredicateObject`,
  `InvalidPredicateValue`, `UnsortableValue`, `ConflictingDeclaration`.
- Every message names the offending noun first and the fix second:
  "field {field} mixes a literal with operator keys; compare the whole object with
  `$eq`" (`src/error.rs:305-309`), "{operator} must be {expected}" (`:313`).
- Lower-case, no trailing full stop, `smut:` added by the renderer.
