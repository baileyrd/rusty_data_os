# Host dispositions after inspection round 2 (fix round 3, owner-extended budget)

Inspector (fresh Claude CLI, session d5319f38-89bc-4c65-b8bf-8f13d31508e5) verdict: REVISE.
The owner extended the budget by one fix round and one inspection round. Host proof on the
current snapshot is green except the two excluded Windows-only tests. Apply exactly the items
below; do not commit; rerun the proof chain and report raw outputs.

## J1 (medium, accept: implement) — Employee atomic batches

Implement `write_batch_checked` on `EmployeeConnectionStore` with the same shape as
Memory/Entity: `self.store.with_exclusive(|inner| ...)`; per op run `check(i)`, then preflight
(`Link` with label `collaborates_with`, both endpoints via an existence overlay over
`GetById::<Employee>::get`, self-loop `Malformed`; any other `WriteOp` is `Unsupported` at
preflight, so a batch containing one is rejected with nothing applied), then apply via the
same link path `link_records` uses. Keep `write_batch` delegating for atomic. Replace the unit
test that pins the refusal with one that pins agreement: single `Link`, pipelined `Link` and
atomic `Link` all answer `Linked` on the same fresh fixture, an atomic batch with a
non-Link op is `TransactionFailed { index, Unsupported }` with no edge added, and a self-loop
or missing endpoint rejects atomically with nothing applied. Update the Employee doc line,
ADR-0060 and SERVER-001 v0.50.1 text so Dog/Order remain the only refusing adapters (verify
Order has no `link_records`; if it does, treat it exactly like Employee).

## J2 (low, accept) — SERVER-002 §20 and the FR-060 traceability row

Add one or two sentences to SERVER-002 §20 stating: batch `Link` is validated against the
relation's target table (`Unsupported` when unregistered, `RecordNotFound` when the far row is
missing); batch `Delete` cascades detaches to other registered tables; in atomic mode a
post-apply detach `Storage` failure returns `BatchResults` with `Failed(Storage)` in that
Delete's slot after the own-table apply succeeded. Semantics only, no wire change, no
SERVER-002 version bump. Update the `SERVER-001-FR-060` traceability row to name the atomic
adapters (Memory, Entity, Relation, Reminder, Employee) and the Dog/Order refusal.

## J3 (low, accept: implement the skip) — relationship mutex on servers with no foreign relation

Skip the relationship section when no registered table declares a relation with a
`target_table` (compute once in `serve_tables`, e.g. an `Option<Arc<Mutex<()>>>` or a bool
passed with the lock), so single-table `serve` servers and table sets with no cross-table
labels keep the baseline interleaving. When at least one foreign relation exists, keep the
single mutex for Link/Delete/WriteBatch as now (per-table sections in registration order are
NOT required in this round). Record the remaining cross-table serialization explicitly in
ADR-0060 (one sentence: pipelined batches of up to `MAX_BATCH_OPS` ops hold the section for
their duration on multi-table servers) and add a follow-up line to the report naming a
`benches/server.rs` measurement as future work. Add a unit test that the section is skipped
for a registry without foreign relations and taken for one with.

## J4 (low, accept) — pipelined `ReplaceIf` guard validation

In `write_batch_across`'s pipelined branch (or in the trait default `apply_write_op`, whichever
keeps `dispatch` parity), run `validate_predicate(&store.describe(), guard)` for
`WriteOp::ReplaceIf` and answer `WriteResult::Failed(code)` on rejection, matching
`Request::ReplaceIf`. Add one pipelined assertion to
`reminder_batch_rejection_is_atomic_and_pipelined_results_are_independent` (unknown guard
field => `Failed(UnknownField)` or the exact code the single request returns; check it).

## Proof (unchanged)

```
cargo fmt --all -- --check
cargo clippy --all-targets --features server,research -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features client -- -D warnings
cargo test --features server,research
cargo test --features client
python -m unittest discover -s clients/python/tests -v
```

Known, host-excluded Windows-only failures: `dog_server` certificate-path `:` split and the
live Python test's `python3` name. Everything else must pass.
