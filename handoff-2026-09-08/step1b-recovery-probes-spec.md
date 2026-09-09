# Work order: merge plan Step 1 (part B) — recovery/isolation probes, repair-or-prevent, snapshot contract

Source plan: `C:/dev/rusty_data_os/data-os-multimodal-merge-plan-2026-09-08.md`, section
"1. Repair immediate legacy defects", paragraphs on the recovery/isolation probes and the
snapshot contract. Part A (batch Link/Delete, Clippy, CI) is a separate work order and must be
landed and inspected first; this work order builds on it.

Target repository: `C:/dev/rusty_multimodal_db`. Same rules as part A: follow `AGENTS.md` and
`WORKFLOW.md`; no dependency, toolchain or `rust-version` bumps; no protocol-version bump
unless a wire shape changes (none is expected here); do not `git commit`.
Keep the repository's LF line endings: edit files in place rather than rewriting whole files
through tools that emit CRLF (a prior attempt did, and every touched file became a whole-file diff).

## Goal

The four reproduced recovery/isolation probes run in-repo, and each either passes because the
legacy engine was repaired, or is prevented (the unsafe combination is refused with a clear
error and documented), with the honest contract written down. The legacy engine remains in
use by consumers, so this is a bounded repair, not a transaction-engine rewrite.

## The four probes (reviewer evidence, reproduced at 46817b9 and unchanged at 478eeda)

Verbatim source: the review harness `review_harness/src/lib.rs` from
`rusty_multimodal_db-review-evidence-2026-09-08.zip` (a copy is in this directory as
`probes-lib.rs`). Land them as integration tests in `tests/` (e.g.
`tests/server_recovery_probes.rs`), adapted to the crate's test conventions, keeping each
probe's assertions intact:

| Probe | Baseline failure | Mechanism (from `src/server/entity.rs` ~641-671 and ~87-107; `memory.rs`/`dog.rs` are identical by their own comments) |
|---|---|---|
| `rejected_conflicting_transaction_must_stay_rejected_after_reopen` | A `Conflict`-rejected field transaction reappears after reopen (count 99 instead of 1) | The journaled path appends the batch in `journal.commit` **before** the read-set check runs inside the apply closure; on `Conflict` the appended batch stays in the journal and is replayed by `with_journal` |
| `acknowledged_delete_after_transaction_must_allow_reopen` | Reopen fails with `Replay { batch: 0, index: 0, code: RecordNotFound }` | `delete_record` does not go through the journal/checkpoint sequence; the journal still holds a field update for the deleted record and replay treats the miss as a hard error |
| `acknowledged_replacement_must_not_be_rewound_by_older_journal` | After `replace_record` + `compact` + reopen, the older journaled value (10) overwrites the replacement (20) | `compact`/`replace_record` flush the slot image but do not truncate or watermark the redo journal; replay reapplies the older batch over the newer image |
| `snapshot_session_must_repeat_its_read` | A `BeginWith { SESSION_SNAPSHOT_ISOLATION }` session's second `GetById` observes a commit from another connection | `SESSION_SNAPSHOT_ISOLATION` (protocol.rs ~129-138) is a read-set validated at `Commit`, not a stable read view; the name and design doc title over-claim |

## Required changes

### R1. Journal outcome integrity (probe 1)

A transaction that is rejected (validation, `Conflict`, or apply error) must never be replayed
after reopen. Acceptable mechanisms: (a) run the read-set check and validation under the
exclusive section **before** the journal append, so only batches that will apply are
journaled; or (b) keep the append-first ordering but record an outcome marker and have replay
skip batches without a commit marker. Prefer (a) if it preserves the journal's crash-atomic
promise (`GRP-FR-001`–`005`, ADR-0025/0026); if you change the on-disk journal format, bump
`JOURNAL_FORMAT_VERSION` and refuse older files with a clear error, and say so in the notes.

### R2. Journal vs. other mutation paths (probes 2 and 3)

Every durable mutation that bypasses the journal (`delete_record`, `replace_record`,
`replace_record_if`, `insert_record`, `link_records`, `detach_record`, `update_field`, `compact`,
and the batch paths) must leave the journal in a state replay can apply correctly. Choose one
consistent rule and apply it to `Memory`, `Entity` and `Dog` (and `Relation` if it gains a
journal):

- **Checkpoint-then-truncate:** after such a mutation's own durability point, flush the slot
  image (`checkpoint_flush`) and truncate the journal under the same exclusive section, so
  replay never reapplies a batch older than the image; or
- **Prevent:** refuse the unsafe combination (journaled adapter + non-journaled mutation) with
  a clear `ErrorCode` and document it as unsupported for journaled stores.

The plan allows either "repair or prevent"; repair is preferred because `memory_server`
enables Memory's journal when `SERVER_TXN_JOURNAL_PATH` is set and consumers use deletes
and replacements on that table. Replay must also distinguish a torn tail (dropped, already
handled) from a committed batch that no longer applies; the latter must not silently corrupt
state — either it cannot occur under the chosen rule (prove it with a test) or it is reported.

### R3. Snapshot contract (probe 4)

Choose one and implement it honestly, without a wire change:

- **Recommended:** make a snapshot session's reads repeatable for the keys it has read (serve a
  repeated `GetById` for a tracked key from the read set's recorded value, overlaying
  read-your-writes as today) and document precisely that unread keys are not frozen and that
  `Commit` still validates the read set. Update the `SESSION_SNAPSHOT_ISOLATION` docs, the
  protocol table row for version 7, `docs/design/SERVER-SESSION-SNAPSHOT-ISOLATION-DESIGN.md`
  and ADR-0033's status/consequences to state the actual guarantee ("read-set validated,
  repeatable for read keys"), not full snapshot isolation.
- Or: keep behavior and rename the documented guarantee only (constant name stays for wire
  compatibility). Then probe 4 becomes a test of the documented weaker behavior, and the note
  must say a stable snapshot is deferred to the shared engine (merge plan step 3/4).

### R4. Shipped configuration coverage

Test the configurations that actually ship: `src/bin/memory_server.rs` builds Memory
`with_journal` only when `SERVER_TXN_JOURNAL_PATH` is set and Entity/Relation with `new`.
Exercise standalone field updates, field transactions, runtime mutations (insert/replace/
delete/link) and reopen for (a) journal-enabled Memory, (b) journal-disabled Entity/Relation,
and (c) the public `with_journal` constructors on Entity and Relation, including the
`RelationConnectionStore::with_journal` constructor the merge plan says exists (verify; if it
does not, say so and do not invent it).

### R5. Notes and traceability

Extend the part A report (`docs/reports/2026-09-08-batch-cross-table-repair.md`) or add a
sibling note listing each probe, its root cause, the chosen rule (repair or prevent), the
declared durability/failure model, and what remains for the shared engine. Update
`docs/PROJECT-STATUS.md`, traceability and the relevant ADRs per `AGENTS.md`.

### R6. Residuals carried from part A's final inspection (all documentation or small)

Part A landed as commits `c1d2640` and `abda0a7` on branch `codex/merge-step1-batch-cross-table`
(inspected APPROVED). Close these four low findings in this change:

- **SERVER-001 FR-060 requirement text** (around line 98) still describes the pre-repair
  WBT-FR-003 contract ("pre-validate every op with no lock … check each Link's own-table
  endpoint", Memory/Entity/Relation only). Amend it to the corrected contract (preflight under
  the exclusive section, foreign endpoint via the server's check, five atomic adapters,
  Dog/Order refusal) or add a one-line pointer to the 0.50.1 change-history entry.
- **Client docstrings**: `SchemaDrivenClient::write_batch` (src/server/client.rs ~1895-1906) and
  the Python client's `write_batch` docstring (clients/python/…/client.py ~383-386) do not say
  that in atomic mode a post-apply cross-table detach `Storage` failure is reported as
  `Failed(Storage)` in that Delete's slot with the batch otherwise applied. Add one sentence to
  each.
- **Spurious RecordNotFound**: an atomic batch reads far endpoints before the adapter's exclusive
  section; a concurrent single Insert on the far table (which does not take the relationship
  mutex) can land after that read, so the batch may be refused for a record that exists by
  apply time. Never a dangling edge. Document this direction in ADR-0060's atomic paragraph
  ("a far record inserted concurrently may be reported missing; retrying succeeds").
- **Throughput note**: pipelined batches hold the relationship mutex for their full duration on
  servers with a foreign relation (up to `MAX_BATCH_OPS` ops, each with a per-op fsync). ADR-0060
  records the trade-off; add a `benches/server.rs` row (or a documented follow-up item in
  `docs/roadmap/ROADMAP.md` if a bench row is out of proportion) so the cost is measured rather
  than unknown. Do not change the locking shape in this work order.

## Non-goals

- No new transaction engine, no multi-table transaction, no crash-atomic batches.
- No protocol bump unless a wire shape changes (none expected).

## Proof

Same commands as part A (run exactly, all must pass), plus the new probe tests must appear in
the `cargo test --features server,research` output as passing, and each probe must be shown to
fail when its fix is reverted (state how you checked):

```
cargo fmt --all -- --check
cargo clippy --all-targets --features server,research -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features client -- -D warnings
cargo test --features server,research
cargo test --features client
python -m unittest discover -s clients/python/tests -v
```

The host additionally runs `cargo +1.88.0 check --all-targets --features server,research`.
