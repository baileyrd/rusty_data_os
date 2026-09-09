# Host dispositions after inspection 1 (fresh Claude CLI) of snapshot ee3392c7…4409

Host proof on that snapshot (independent): fmt PASS; clippy ×3 PASS; server,research
`--no-fail-fast` 740 passed / 2 failed (exactly the two host-excluded Windows tests); client 238
passed; Python 5/5; `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features
server,research` PASS. The inspector's verdict was REVISE with four findings; the host verified
each against the source and accepts all four. This is the final fix round of the budget:
implement every item completely, rerun the proof chain, refresh the report, keep LF endings, and
do not commit.

## I1 (medium) — Reminder ships journaled and still has the probe-1/2/3 defects

`src/bin/reminder_server.rs:89-96` builds `ReminderConnectionStore::with_journal` whenever
`SERVER_TXN_JOURNAL_PATH` is set, so Reminder is a shipped journaled configuration (and the
merge plan's pilot consumer). `src/server/reminder.rs` still validates outside any lock, appends
through `CommitGroup::commit`, and runs `check_read_set` only inside the apply closure (probe 1);
`insert_record`/`replace_record`/`replace_record_if`/`delete_record`/`compact` and the batch paths
go straight to `self.store` with no gate or checkpoint (probes 2 and 3).

Disposition: repair, not prevent. Port the Memory pattern to `ReminderConnectionStore`:
`with_mutation` (gate + pre-mutation `checkpoint`) around every bypass mutation including
`update_field`, `write_batch_checked` and the per-op pipelined methods; journaled
`apply_transaction` = gate, exclusive preflight (validate + read-set check), then
`commit_serialized`. Add a `"reminder"` configuration to `exercise_configuration` in
`tests/server_recovery_probes.rs` (journal-enabled, matching the shipped constructor; Reminder has
no links, so assert its `link_records` refusal as Relation's is asserted), and include Reminder in
`concurrent_read_set_commits_have_one_durable_winner`. Update ADR-0061, the ADR-0025/0026
amendments, SERVER-001's amendment paragraph and 0.50.2 entry, and the report so they name five
repaired adapters (Memory, Entity, Relation, Dog, Reminder) and say that Order/Employee
(research-gated, not shipped) keep the old path. If the port hits a structural difference you
cannot resolve, stop and report it instead of silently choosing "prevent".

## I2 (medium) — `Err(Journal)`/`Err(Storage)` now reach old clients undowngraded

`UpdateField` is a protocol-1 request. With the checkpoint in `update_field`, `dispatch` can
answer `Response::Err { code: Journal }` (protocol 4) or `Response::Err { code: Storage }`
(protocol 13), but `downgrade_for_version` (src/server/serve.rs:1260-1277) only rewrites
`TransactionFailed`, so a connection negotiated below 4 or 13 receives a variant index it cannot
decode (compatibility rule 3). The amended protocol.rs version-4 row already claims `Err` is
downgraded, which the code does not do.

Disposition: extend `downgrade_for_version` with `Response::Err { code: Journal, .. }` below 4 →
`Unsupported`, and `Response::Err { code: Storage, .. }` below 13 → `Unsupported` (the nearest
older shape, with `error_message(Unsupported)`); add unit tests beside
`journal_error_code_is_downgraded_below_version_4` for both, plus one that a protocol-3
connection's `UpdateField` answered `Journal` by a failing adapter reaches the client as
`Err(Unsupported)` if a test adapter can inject it cheaply (otherwise the unit tests suffice; say
which). Make the protocol.rs version table rows for 4 and 13 and the `ErrorCode::Journal`/`Storage`
doc comments describe exactly what is rewritten.

## I3 (low) — design docs and registry not amended

Add a short "amended by ADR-0061 (2026-09-08)" note to
`docs/design/SERVER-JOURNAL-GROUP-COMMIT-DESIGN.md` (GRP-FR-001 single-writer progress and
GRP-FR-002 grouping superseded on the repaired adapters; name them) and to
`docs/design/SERVER-TRANSACTION-SESSION-DESIGN.md` (JRN-FR-007: a standalone `UpdateField` is
still not journaled but now checkpoints pending redo first on a journaled adapter). Add
`tests/server_recovery_probes.rs` to the SERVER-001 verification column in
`docs/specifications/SPEC-REGISTRY.md`.

## I4 (low) — fallible generic Dog apply after the first slot write

`commit_serialized` truncates the appended entry on any `CommitError::Apply`. For the concrete
stacks, apply after gated preflight can only fail with NotFound (excluded), but
`DogConnectionStore<S>` is generic and its journaled `update_field` maps a non-NotFound
`update_age` error to `Storage` while the unjournaled path maps it to `Malformed`.

Disposition: documentation plus consistency, no behavior change to the cleanup: state in ADR-0061
and in the `DogConnectionStore::with_journal` docstring that the crash-atomic promise requires
`S`'s `update_age` to fail only with NotFound after validation (a later failure leaves a partially
applied batch whose redo entry has been removed), and make the journaled and unjournaled Dog
`update_field` map the same non-NotFound error to the same code (`Storage`; update the
unjournaled arm and its comment).

## Report

Refresh the proof table for the new snapshot. Keep the two Windows-only exclusions worded as
before. Remove the sentence saying the host's acceptance applies only to the prior snapshot; the
host will proof this snapshot independently and record it in its own log.
