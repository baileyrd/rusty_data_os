# Host dispositions on Codex review 1 of the Step 4b-i work order (plan sha256 021eab10…3a76)

All seven findings accepted; the work order was revised as follows (new hash 4b477545…f9a359).
Please re-review the revised plan for approval.

- P4B-001 (high, Transaction misrouted): confirmed against `serve.rs`'s real dispatch arm
  (`Request::Transaction { updates } => store.apply_transaction(&updates, &[])`, mapped to
  `Response::Ok`/`TransactionFailed`). Removed `Transaction` from the fallback-only set in both R6
  and the "Repository facts" section that previously (incorrectly) grouped it with
  `Authenticate`/`Hello`/`Begin`/`BeginWith`/`Commit`/`Rollback`. R6 now specifies the real arm;
  R7 specifies `SessionOpen` rejection only while a session is open. New required tests: a
  successful one-shot transaction and a failing one naming the right index with nothing applied.
- P4B-002 (high, missing `write_batch_checked`/atomic safety): R5 now requires
  `write_batch_checked` with the exact fail-closed default quoted from `serve.rs` (refuse
  nonempty atomic batches at index 0, apply nothing) and `apply_write_op` (the pipelined
  per-op dispatcher). R5's `write_batch` no longer permits falling back to pipelined execution
  for `atomic: true`. New required test: an atomic batch whose later op fails leaves the earlier
  op's write unapplied.
- P4B-003 (high, missing `detach_record`/cross-table Delete cleanup): R5 adds `detach_record`
  (default `Unsupported`). R7 adds the `delete_across`/`detach_across` cascade verbatim from
  `serve.rs`, run after a successful own-table delete, across every other registered table whose
  relation names this table as `target_table`. New required test: link across tables, delete the
  linked-to endpoint, verify the other table's adjacency and `CountEdges` reflect the detach.
- P4B-004 (high, no shared cross-connection synchronization): new Design decision D7 — a
  `Registry` type with one shared `Option<Arc<Mutex<()>>>` relationship lock (present only when
  some relation declares a `target_table`), acquired once per `Link`/`Delete`/`WriteBatch` request
  and held across that request's entire cross-table span, exactly matching `relationship_mutex`/
  `relationship_section`. R7 requires a deterministic two-connection interleaving test proving no
  dangling edge results from the race the finding described.
- P4B-005 (high, `BeginWith` flags under-specified): R7 now specifies all three flag behaviors
  (`SESSION_READ_YOUR_WRITES = 1`, `SESSION_VALIDATE_ON_STAGE = 2`, `SESSION_SNAPSHOT_ISOLATION = 4`,
  freely combinable, any other bit `Malformed`) verbatim from `protocol.rs`, including the tracked
  read-set/`MAX_TRACKED_READS`/`Conflict`-on-mismatch mechanics. New required tests: read-your-writes
  overlay, stage-time validation rejection, and both a clean and a conflicting snapshot-isolation
  commit.
- P4B-006 (medium, missing cross-table Join): R6's `Join` arm is now scoped explicitly to
  `right_table: None` only; R7 adds the `right_table: Some(name)` interception and `join_across`
  evaluation against the resolved right-hand schema, verbatim citation from `serve.rs:2863,3011`.
  New required tests: a successful cross-schema join and rejection of an unregistered/mismatched
  target table.
- P4B-007 (medium, round-trip proof insufficiency): new Design decision D6 supersedes the old one.
  The host independently derived and recorded the literal expected value of every field of all 66
  fixture lines in `handoff-2026-09-08/step4b-fixture-expected-values.txt` (a hand-written
  reference decoder implementing the same byte rules, run against the real fixture file, zero
  decode errors, all 66 lines produce sane values). R4 now requires both a literal-value
  assertion against that file and the original round-trip check, for every line.
