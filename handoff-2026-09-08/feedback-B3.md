# Host dispositions after inspection 2 (fresh Claude CLI) of snapshot a9ba9d50…74c4

Host proof on that snapshot (independent): fmt PASS; clippy ×3 PASS; server,research
`--no-fail-fast` 744 passed / 2 failed (the two host-excluded Windows tests); client 238 passed;
Python 5/5; `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research`
PASS. The inspector's verdict was REVISE with four findings; the host verified F1 in the source
and accepts all four. The owner extended the budget by exactly one fix round and one inspection:
implement every item completely, rerun the proof chain, refresh the report, keep LF endings, do
not commit, and do not widen scope.

## J1 (medium) — a fully applied, durable batch is answered "Journal … nothing was applied"

In `commit_serialized`, when `apply` returns `Ok(true)` (size checkpoint due, `checkpoint_flush`
succeeded, the batch applied and durable in both image and journal) and the following
`self.truncate()` fails, the code marks the group `unusable` and returns `CommitError::Journal`.
Every adapter maps that to `(0, ErrorCode::Journal)`, so the client receives
`TransactionFailed { Journal, "…nothing was applied" }` for a commit that did apply and will
replay idempotently on reopen, and further writes are refused. The same unchanged message also
answers the indeterminate-cleanup path, contradicting ADR-0061.

Disposition:
- On a post-apply truncate failure after a successful flush, return `Ok(())`: the entry is valid
  redo over a flushed image, replay is idempotent, and the next size checkpoint retries the
  truncate. Do not set `unusable` on this path. Keep the pre-bypass `checkpoint` behavior
  (flush ok, truncate failed → refuse the bypass mutation with `Journal`, nothing applied,
  fail-stop) since proceeding would recreate the probe-2/3 hazard; say so in a comment.
- Change `error_message(ErrorCode::Journal)` so it no longer asserts "nothing was applied";
  something like "journal I/O failed; a transaction refused before apply applied nothing, a
  failed durable cleanup leaves the outcome indeterminate and writes are refused until reopen".
  If any wire fixture or golden vector pins that message text, update the fixture and state in
  the report that the message text is not part of the versioned wire shape (no protocol bump).
- Add a unit test in journal.rs: inject a truncate failure after a checkpoint-due apply, assert
  `Ok`, assert the group still accepts a following `commit_serialized`, and assert the retained
  entry replays idempotently on reopen (value unchanged).
- Update ADR-0061's failure-model text and the `ErrorCode::Journal` doc comment accordingly.

## J2 (low) — pure-request validation now runs after the checkpoint

`with_mutation` checkpoints (flush + truncate + fsync) before the request is validated:
`*_from_fields`, the field/kind match in `update_field`, and the label check in `link_records`
run inside the closure. A malformed or unknown-field write therefore performs store I/O, and a
checkpoint failure answers `Storage`/`Journal` instead of `Malformed`/`UnknownField`.

Disposition: on all five repaired adapters (Memory, Entity, Relation, Reminder, Dog's journaled
`update_field`), perform the pure-request validation (field parsing, value kind, label
validity, status discriminant) before taking the gate and checkpointing, and enter
`with_mutation` only for the store-dependent part. Preserve the old error precedence. Add one
assertion per adapter family (a malformed `update_field` on a journaled adapter with pending redo
leaves the journal length unchanged) to `tests/server_recovery_probes.rs` or the adapter unit
tests.

## J3 (low) — stale comments

Update to ADR-0061 wording or replace with a one-line pointer: `DogConnectionStore::validate_batch`
doc ("outside any lock at all … never deletes a record at runtime"); the `ConnectionStore`
`insert_record` trait doc in serve.rs (`Journal` "a transaction batch that was not journaled",
`Storage` "nothing applied"); `memory.rs` `apply_transaction` comment still citing
`GRP-FR-001`–`005`; the four generic adapters' `with_journal` docs that point at Dog's contract
without the ADR-0061 caveats Dog now carries.

## J4 (low) — SPEC-REGISTRY columns

Restore the SERVER-001 row's verification-status column to "Verified" and append
`tests/server_recovery_probes.rs` to the evidence column at the end of the row, alongside the
other integration tests.

## Report

Refresh the proof table for the new snapshot; keep the two Windows-only exclusions worded as
before; add a J1–J4 section in the style of the K and I sections.
