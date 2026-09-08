# Work order: merge plan Step 1 (part A) — batch Link/Delete repair, Clippy, CI

Source plan: `C:/dev/rusty_data_os/data-os-multimodal-merge-plan-2026-09-08.md`, section
"1. Repair immediate legacy defects". This work order covers the batch defects, the Clippy
failures and CI. The recovery/isolation probes and the snapshot contract are a separate,
later work order (part B); do not attempt them here.

Target repository: `C:/dev/rusty_multimodal_db` (baseline commit `478eeda4544aa98c4948aebc66b5ff8fad609645`,
PR #229). Work only inside that checkout. Follow its `AGENTS.md` change rules (`Result` + `?`, no
`unwrap` outside tests, docstrings on public items, flat control flow) and `WORKFLOW.md`.
Do not bump dependencies, the toolchain, `rust-version`, or the protocol version. Do not run
`git commit`; leave the working tree with your changes for inspection.

## Goal

Single requests and both batch modes (pipelined and atomic) agree on relationship correctness;
an atomic rejection leaves no partial cross-table effects; the repaired validation pipeline
(fmt, clippy on all three CI feature sets, tests, MSRV) passes.

## Confirmed defects (reproduced by the reviewer at the baseline)

In `src/server/serve.rs`, single-shot `Request::Link` and `Request::Delete` route through
`link_across` (TBL-FR-007, ~line 2964) and `delete_across` (DEL-FR-007, ~line 2931) in
`handle_connection` (~line 2800). The same ops inside `Request::WriteBatch` go to
`dispatch(store, ..)` (~line 2075) -> `ConnectionStore::write_batch` -> the table-local
`apply_write_op` / `link_records` / `delete_record` (lines ~320-390), so:

1. **Batch Link accepts a missing foreign endpoint.** `MemoryConnectionStore::link_records`
   (`src/server/memory.rs` ~689) documents that the server checks the far endpoint against the
   `entity` table before calling it; the batch path never does. The atomic override
   (`memory.rs` ~629-660, `entity.rs` ~531-560) checks only the own-table `left` endpoint.
   The equivalent single request answers `RecordNotFound` (or `Unsupported` when the server has
   no such table).
2. **Batch Delete leaves cross-table edges behind.** `delete_across` calls every other table's
   `detach_record` for relations whose `target_table` is the deleting table; the batch path
   deletes the record but leaves e.g. `memory` `mentions` edges pointing at a deleted entity,
   so a `NeighborsByRelation`/`Join`/`CountEdges` on the other table still reports the UUID.

The only existing functional batch test, `tests/server_memory_integration.rs`
`write_batch_pipelined_applies_each_and_atomic_is_all_or_nothing` (~line 1351), runs on a
single-table server and asserts `Linked` for a `mentions` link to an entity that no table
holds. That assertion encodes defect 1 and must change to the corrected outcome.

## Required changes

### R1. Batch relationship validation is table-aware (both modes)

- A `WriteOp::Link` inside a batch must validate its far endpoint exactly as `link_across`
  does for the single request: relation with a `target_table` -> the target table must be
  registered (`Unsupported` otherwise) and hold `right` (`RecordNotFound` otherwise). A
  relation without `target_table` behaves as today.
- Pipelined mode: the failing op's result is `WriteResult::Failed(code)` with the same code the
  single request would return; other ops stand on their own, as ADR-0060 specifies.
- Atomic mode: the batch is refused with `TransactionFailed { index, code }` naming the first
  failing op, and **nothing is applied to any table**.
- A batch on a single-table server (`serve`) must behave as the single request does on that
  server (a foreign-target relation is `Unsupported` there, per `link_across`).

### R2. Batch Delete detaches cross-table edges (both modes)

- A `WriteOp::Delete` inside a batch must, on `Deleted`, drop the same cross-table edges that
  `delete_across` drops for the single request (every other table's `detach_record` for each
  relation whose `target_table` is this table). `NotFound` deletes detach nothing.
- The detach failure semantics must match `delete_across`: `Unsupported`/`Malformed` from a
  detach is skipped; a `Storage` failure is reported for that op. Document any partial state
  exactly as `delete_across` documents its own.

### R3. Intra-batch dependencies

- Validation must account for effects of earlier ops in the same batch, for both modes:
  an `Insert` earlier in the batch makes a later `Link` to it valid; a `Delete` earlier in the
  batch makes a later `Link` to that id invalid. This applies to own-table endpoints and to
  foreign endpoints when the batch's own table is the relation's target (note: a batch is
  always table-targeted to the connection's selected table, so a foreign endpoint's existence
  only changes within a batch if the far table is the batch's table; state clearly what your
  implementation covers and test what it claims).

### R4. Atomic mode: whole-batch precondition atomicity across tables, with safe lock ordering

- The atomic guarantee ADR-0060 claims (precondition- and isolation-atomic) must now hold for
  the cross-table checks too: a foreign-endpoint failure must be detected before any op is
  applied, and the check must not be silently invalidated by a concurrent write on the far
  table in a way that leaves an edge to a record that was already gone at the time the batch
  was applied. Choose the mechanism (e.g. a server-level routine that resolves cross-table
  preconditions and holds the far table's read/exclusive section, or a new
  `ConnectionStore` hook that receives a validated view of the other tables). Calling existing
  helpers in a loop is not by itself atomic.
- If you hold sections on more than one table, acquire them in one fixed global order (the
  server's registered table order from `serve_tables`) and release together; document the
  order and why it cannot deadlock against `delete_across`, `link_across`, `Commit` and
  another batch.
- Cross-table detaches for a batched Delete happen after the batch's own-table apply succeeds,
  in the same fixed order. State honestly in the docs whether they are inside or outside the
  batch's atomic section (crash-atomicity across the batch remains the named storage
  follow-on; do not claim it).
- Update ADR-0060's "Acceptance and implementation" section (or add ADR-0061 if you judge the
  change consequential enough) to state the corrected contract precisely: what atomic covers,
  what it does not, and the lock order.

### R5. Regressions (must fail on the baseline, pass after the fix)

Add integration tests over a real socket using the three-table server helper
(`start_three_table_server_at` in `tests/server_memory_integration.rs`, or an equivalent
helper) so cross-table effects are real, not table-local mocks. Cover, for the single
request, the pipelined batch and the atomic batch:

- positive link: memory -> existing entity under `mentions` is `Linked`, visible from the
  entity side via the `memory` table's `neighbors_by_relation`/`Join`, and `CountEdges`
  agrees;
- valid deletion: deleting a linked entity (on the `entity` table) via batch removes the
  `mentions` edge in the `memory` table — the memory's neighbors no longer include the UUID,
  `CountEdges` decrements, and the effect survives a reopen;
- missing endpoint: batched link to an entity id no table holds is `Failed(RecordNotFound)`
  pipelined and `TransactionFailed { index, RecordNotFound }` atomic with nothing applied
  (the batch's earlier inserts are absent afterwards);
- operations whose validity changes because of earlier ops in the batch (R3), one positive
  and one negative case;
- the single-table-server case: batched link under a foreign-target relation answers as the
  single request does.

Also fix the existing batch test's `Linked`-to-nonexistent-entity assertion. Keep the
existing `WriteBatch` wire vectors unchanged (no wire change is involved); if any Python
client conformance fixture encodes the wrong outcome, correct it and say so.

### R6. Clippy and CI

- Fix the two stable-Clippy failures (`manual_slice_split`/`chunks_exact` with a constant
  size): `src/durability/mmap_store.rs:699` and `src/server/pem.rs:110`. Use a form that
  compiles on Rust 1.88 (`slice::as_chunks` is stable only from 1.88, so it is acceptable;
  verify, or use an `allow` scoped to the site with a one-line reason if it is not).
- Keep `.github/workflows/ci.yml` green on both jobs. If unpinned stable is the recurring
  cause, do not pin the toolchain (a toolchain change is ask-first in `WORKFLOW.md`); note the
  option in the implementation notes instead.
- Also update `docs/PROJECT-STATUS.md`, `docs/traceability/TRACEABILITY.md` and the SERVER
  specification/registry entries per `AGENTS.md` if a unit's state changes; keep it brief.

### R7. Implementation notes

Add a short note (a new `docs/reports/2026-09-08-batch-cross-table-repair.md` or an ADR
section) that: lists the two defects and the tests that reproduce them; states the corrected
atomic contract; and links to the recovery/isolation probes as still-open (name the four
probe scenarios from the plan: rejected conflicting transaction reappears after reopen;
delete after a journaled transaction breaks reopen with `Replay { RecordNotFound }`; an older
journal rewinds a later replacement after compaction; a "snapshot" session observes a later
commit). Mark those as out of scope for this change and pending part B.

## Non-goals

- No journal-format change, no crash-atomicity across a batch, no protocol bump.
- No changes to the recovery/journal replay or snapshot semantics (part B).
- No dependency or toolchain bumps. No new crates.

## Proof

Run exactly; all must pass. Local Windows substitutes `server,research` for `--all-features`
because `perf-events` and `external-db-bench` do not build here.

```
cargo fmt --all -- --check
cargo clippy --all-targets --features server,research -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features client -- -D warnings
cargo test --features server,research
cargo test --features client
python -m unittest discover -s clients/python/tests -v
```

The host will additionally run `cargo +1.88.0 check --all-targets --features server,research`
(the MSRV job's local equivalent). Report actual outputs; a test you could not run is a stated
limitation, not a pass.
