# Host dispositions on Codex review 2 of the Step 4b-i work order (plan sha256 4b477545…f9a359)

All three findings accepted; the work order was revised as follows (new hash 66a53b9f…57d80).
Please re-review the revised plan for approval.

- P4B-008 (high, incomplete session-open guard set): confirmed against `serve.rs:2790-2849`'s full
  list — `Begin`, `BeginWith`, `Transaction`, `Insert`, `Link`, `Replace`, `Delete`, `Compact`,
  `ReplaceIf`, `WriteBatch`, `Use` are all refused `SessionOpen` while a session is open, checked
  before any table switch or mutation. R7 now enumerates the full set explicitly and requires a
  scenario proving a refused `Use` leaves the table unchanged and a refused immediate write
  (including `Delete`) leaves no effect after `Rollback`.
- P4B-009 (medium, missing `MAX_STAGED_OPS`/`MAX_BATCH_OPS`): both constants (4096 each, per
  `protocol.rs:141-155`) and their exact rejection semantics are now specified in the new
  "Session-open guards — the full request set, and the two operation-count caps" repository-facts
  section, and required in R7: session staging overflow is `SessionFull` with the session staying
  open and prior staged writes intact; an oversized `WriteBatch` is `Malformed` with zero effect.
  New required tests at the 4096/4097 boundary for both.
- P4B-010 (medium, `Malformed`/`Unsupported` conflated for Join): added `validate_join`'s exact
  4-way match verbatim as a new repository fact. R6's `Join` arm now implements that match
  directly (with `right_schema` always `None` in bare dispatch, which is what correctly produces
  `Malformed` for every `right_table: Some(_)` case reaching bare dispatch, not a blanket
  `Unsupported`). R7's `join_across` interception now re-runs the same match with the resolved
  `right_schema`, distinguishing: successful cross-schema join; `Malformed` for a named-but-not-
  registered target; `Malformed` for a named target that doesn't match the relation's declared
  target; `Unsupported` only for an *absent* `right_table` against a relation that declares one
  (reached through generic dispatch, not this interception). New required tests assert each of
  the four exact codes.
