# Data OS + Rusty Multimodal DB — updated merge plan

Revision 2 · September 8, 2026 · Incorporates the supplied review and eight reproduced regression probes.

**Build one general-purpose database that can replace SQLite, PostgreSQL, DuckDB and additional database workloads, with one authoritative history and multiple optimized representations.** Data OS becomes the eventual home of the combined engine. Carry forward useful multimodal features and migrate applications through a working compatibility path.

The merge has six concrete steps. The broader database capabilities continue alongside and after that work. Memory is the first proving workload; the product must support general data and queries. An application can migrate as soon as its requirements work correctly, without waiting for full PostgreSQL or DuckDB compatibility.

This is a complete execution roadmap for the merge, with deliverables, dependencies and observable completion conditions. Individual implementation changes will still resolve code-level interfaces and algorithms. The database implementation and competitive performance remain work to do.

## Working approach

Ship code, relevant tests, measurements and short notes together. Keep changes small enough to verify. Do not create a documentation-only approval ladder or reopen the owner's replacement-database goal. Briefly align active repository instructions with that goal in the first relevant implementation change while preserving existing research evidence and useful correctness requirements.

Use the existing engines as comparison targets and migration sources. They should not become hidden production engines underneath the replacement. Performance claims must identify the workload and match correctness, durability, concurrency and retention settings. Universal superiority is an ambition, not an established result.

## Reviewed starting point

The source baseline below was checked on September 8, 2026. Check for intervening source changes when implementation starts; these findings refer to the pinned commits.

| Project | Reviewed revision | Useful foundation | Immediate limitation |
|---|---|---|---|
| Data OS | [79d51e9, PR #124](https://github.com/baileyrd/rusty_data_os/commit/79d51e9404c8b35dee2662aa657b602e9061b44b) | Ordinary-write history, physical replay and independent row/column reconstruction in a bounded experiment | No complete durable, transactional database backend yet |
| Rusty Multimodal DB | [478eeda, PR #229](https://github.com/baileyrd/rusty_multimodal_db/commit/478eeda4544aa98c4948aebc66b5ff8fad609645) | Domains, relationships, indexes, structured queries, clients, server and protocol 22 write batches | Recovery inconsistencies and incorrect cross-table batch effects |

At the last inspection, Data OS's two head-specific checks passed. Multimodal's MSRV check passed; stable Clippy failed at two existing sites, preventing later checks in that job. Protocol 22 batches are table-targeted and do not provide whole-batch crash recovery. [Batch semantics](https://github.com/baileyrd/rusty_multimodal_db/blob/478eeda4544aa98c4948aebc66b5ff8fad609645/docs/decisions/ADR-0060-write-batch.md), [multimodal CI](https://github.com/baileyrd/rusty_multimodal_db/actions/runs/34261113652).

Eight targeted regression tests failed against the unmodified multimodal baseline: four recovery/isolation probes and four batch probes. They are executable evidence, not a full test-suite result. The reviewed evidence package retains the harness, lockfile, captured output and rerun instructions.

Data OS's 10K-event physical replay result had a median of 2.502494 ms and a range of 2.275321–17.027527 ms over five trials. Those events updated only 17 entities. This measures one replay stage, without an fsync durability claim; it does not establish database throughput or tail latency. [Retained measurements](https://github.com/baileyrd/rusty_data_os/blob/79d51e9404c8b35dee2662aa657b602e9061b44b/docs/experiments/exploratory-history-views/INCREMENTAL-LIFECYCLE-VALIDATION-COMPARISON.md).

## The six merge steps

| Step | Main location | Delivered result | Completion evidence |
|---|---|---|---|
| 1. Repair immediate legacy defects | Multimodal | Correct batch relationships and restored CI | New regressions pass for single requests and both batch modes; supported checks pass |
| 2. Run the shared Memory experiment | Data OS | Candidate and legacy engines execute identical traces | Independent expected results match; repeatable measurements cover realistic live records |
| 3. Unify commitment and recovery | Data OS | One authoritative transaction history | Recovery, retry and atomic-publication tests pass under the declared durability contract |
| 4. Integrate existing features | Primarily Data OS | Existing clients and selected domains use the shared core | End-to-end application traces and compatibility fixtures pass |
| 5. Migrate one application | Engine and consumer | A real consumer runs on verified imported data | Data comparison, reopen, restore and rollback paths are exercised |
| 6. Consolidate the projects | Both repositories | One maintained runtime and a clear compatibility path | Migrated consumers depend on the shared engine; retained legacy support is explicit |

The steps express dependencies, not six oversized PRs. Split each into working code increments where useful. Feature inventory and benchmark preparation can begin during the experiment; durable production migration depends on steps 3 and 4.

## 1. Repair immediate legacy defects

Fix two confirmed batch defects in both pipelined and atomic modes:

- **Batch Link accepts a missing foreign endpoint.** The equivalent single request rejects it. Make batch relationship validation table-aware.
- **Batch Delete leaves cross-table edges behind.** The record disappears but another table can still return its UUID as a neighbor. Make deletion detach the same relationships as the equivalent single request.

Include the four reproduced batch regressions with the repairs. Cover positive links, valid deletion, missing endpoints, and operations whose validity changes because of earlier operations in the batch. Atomic behavior must account for all affected tables, cross-table effects and lock ordering. Calling existing helpers in a loop does not by itself preserve atomicity. [Routing and cross-table helpers](https://github.com/baileyrd/rusty_multimodal_db/blob/478eeda4544aa98c4948aebc66b5ff8fad609645/src/server/serve.rs).

Repair the current Clippy failures in `src/durability/mmap_store.rs` and `src/server/pem.rs` alongside this substantive change, while preserving Rust 1.88 compatibility. Check both the MSRV and supported stable toolchains.

Keep the existing recovery/isolation probes runnable and link them from the implementation notes. Add passing regressions with the corresponding fixes. While the legacy engine remains in use, repair or prevent unsafe combinations used by its consumers; do not require a second complete transaction-engine rewrite before building the shared core. The snapshot-labeled behavior also needs an honest contract: either implement a stable snapshot or expose the weaker behavior accurately.

**Done when:** single requests and both batch modes agree on relationship correctness, atomic rejection leaves no partial cross-table effects, and the repaired validation pipeline passes.

## 2. Run the shared Memory experiment

Create two standalone workspaces in Data OS:

| Proposed path | Purpose |
|---|---|
| `experiments/convergence-memory/` | Candidate backend, independent expected results and measurement harness |
| `experiments/convergence-memory-legacy/` | Comparison runner pinned to the reviewed multimodal source |

Give each its own manifest and lockfile. Select Rust 1.89 explicitly; the existing nested EXP-0001 toolchain does not configure sibling directories. Add both manifests to CI in the same change. Fetch the locked legacy dependencies before any offline validation. Exchange a small, versioned trace/results format so the runners remain comparable without merging their dependency graphs.

Preserve the existing EXP-0001 workspace, fixtures, evidence and validation. Reuse suitable codec, replay and independent-view code. In these experiments, `Memory` means the existing application record type; it does not restrict the database to that domain.

Execute the same operations against both engines and an independent expected-state model:

- Insert, get, field update, whole-record replacement, guarded update, delete and reinsert, covering all current Memory fields.
- Equality lookup, column aggregation and ordered pagination, including duplicate ordering keys and explicit tie-breaking.
- Repeated mutations over 1K, 10K and 100K distinct records, plus a small deterministic trace that is easy to debug.
- Independent row and column reconstruction from history, compared against expected results rather than only against each other.

Measure complete operation costs as well as append, replay, rebuild and query stages. Record memory use, disk size, dataset shape, compiler, dependency versions, hardware, warm/cold conditions and enough repetitions to characterize variability. Preserve reusable drivers from the existing external-database benchmarks where they fit; the retained Dog benchmark and separate Memory Page measurements need equivalent cases before reuse as direct comparisons.

The first candidate may use the existing ordinary-write D1 path. Label it clearly and compare equivalent durability settings. Durable acknowledgement is step 3; this experiment must not imply that ordinary writes survive a host or power failure.

**Done when:** both runners execute the shared traces, independent expected results agree with candidate row and column views, CI exercises both workspaces, and retained measurements show where the candidate improves or regresses. A measured regression drives the next optimization; it does not invalidate useful integration work.

## 3. Unify commitment and recovery

Implement a bounded local transaction core, starting with one writer and explicit transaction size limits. General multi-client concurrency follows once local commitment and recovery are correct.

The proposed write path is: validate the complete transaction against a coherent state, including effects of its earlier operations; assign an ordered commit position; append a complete recoverable transaction; satisfy the requested durability level; publish the committed state atomically; return an outcome tied to the request identity. An error before commitment must not create a transaction that reappears after restart. Failures around the commitment boundary must distinguish rejection from an indeterminate outcome that can be resolved by request identity.

Provide:

- One ordered history for inserts, field changes, replacements, deletes, relationship changes and transaction boundaries.
- Explicit acknowledgement semantics. Durable success must wait for the required synchronization; ordinary buffered success remains a distinct contract.
- Integrity checks and clear handling of incomplete tails versus committed corruption, plus exclusive writer ownership of a storage directory.
- Retry resolution and deduplication with a stated retention scope, so losing the response does not duplicate a committed operation.
- Validated checkpoints tied to exact committed history positions. Rebuilds use a consistent checkpoint and subsequent committed history.
- Record incarnation or equivalent version semantics so delete/recreate of a UUID cannot attach old updates or relationships to a new record.

Use the reproduced failures to drive concrete recovery tests:

| Scenario | Required result after reopen or retry |
|---|---|
| A field transaction fails conflict validation | Its proposed value is absent |
| A committed field transaction is followed by deletion | Reopen succeeds and the record stays absent |
| An older field transaction precedes whole-record replacement and checkpointing | The replacement remains current |
| A UUID is deleted and recreated | Old updates and edges do not affect the new incarnation |
| Execution stops during a batch or checkpoint | No partial committed transaction becomes visible; recovery uses a valid checkpoint/history combination |
| Commitment succeeds but the response is lost | Retrying or resolving the request returns the existing outcome |

Exercise standalone field updates, field transactions, runtime mutations, and journal-enabled and disabled legacy adapters during integration. Fault injection and process termination test specific failure windows; they are not substitutes for host/power-loss validation. State the tested failure model and the durability claim together.

### What replaces the journal

The shared engine's authoritative commit history replaces the separate application field journals once every supported mutation goes through it. A durable recovery log still exists; it becomes part of the database core instead of an independently replayed source that can contradict inserts, replacements or deletes.

Correct the inventory from the supplied review: `RelationConnectionStore` already has a public `with_journal` constructor. The shipped `memory_server` uses `new` for Entity and Relation, and enables Memory's journal only when `SERVER_TXN_JOURNAL_PATH` is set. Test the shipped configuration as well as the public constructors. [Relation constructor](https://github.com/baileyrd/rusty_multimodal_db/blob/478eeda4544aa98c4948aebc66b5ff8fad609645/src/server/relation.rs#L64), [server setup](https://github.com/baileyrd/rusty_multimodal_db/blob/478eeda4544aa98c4948aebc66b5ff8fad609645/src/bin/memory_server.rs).

A checkpoint is an acceleration structure, not automatic permission to erase historical data. Preserve authoritative history for the promised time-travel range. Future retention or compaction policies must make any reduced history explicit.

**Done when:** acknowledged durable transactions recover under the declared failure model, rejected transactions remain absent, retries resolve correctly, and all supported mutations share the same ordering and recovery rules.

## 4. Integrate existing features

Port Memory, Entity and Relation first. Preserve the Reminder consumer path and inventory Dog, Order, Employee and any other adapters or consumers before claiming complete compatibility or removing code.

Reuse suitable indexes, relationship operations, structured query execution, Rust/Python clients, and protocol fixtures. Preserve the reviewed protocol 22 behavior and its 66-fixture baseline where applicable; extend fixtures for corrected behavior and explicitly version incompatible changes. Keep networking, TLS and application domain definitions outside the storage core. A `rusty_multimodal_db` compatibility facade can retain familiar imports and APIs while delegating to the shared engine.

The SQL parser currently lives on the client, while the server already executes structured queries. Broaden parsing, catalog, planning and execution where required. Reusing that executor does not require relocating every parser or rewriting the server first.

Run a complete application trace through individual and batch requests: insert records, link across tables, replace fields and records, query, delete, verify detachment, checkpoint, reopen and query again. Preserve specified soft outcomes such as Duplicate, GuardFailed and NotFound. Correct batch behavior must be tested with actual cross-table effects, not only table-local mocks.

Make read consistency executable. A read or session advertised as a snapshot must observe the claimed committed version across the trace. Derivable indexes and representations need a known applied history position; the query layer must wait, catch up or explicitly expose staleness rather than silently mix inconsistent versions.

**Done when:** selected existing clients run their real operations through the shared core, protocol and domain compatibility tests pass, and no supported mutation bypasses authoritative commitment.

## 5. Migrate one application

Use `rusty_remind_me` as the initial candidate consumer, subject to checking its actual current integration when the pilot begins. The retained spike alone does not establish what it currently runs.

1. Quiesce source writes and preserve a consistent copy of every source data file, journal and mutation log.
2. Export logical records and relationships using a compatible reader. Import into a new database directory, preserving IDs, timestamp units, fields and metadata.
3. Compare every supported field and relationship, record counts and sorted per-record digests. Exercise the application's important query results as well as raw data comparison.
4. Reopen and independently rebuild derived representations. Create and restore a backup into another directory and repeat the comparisons.
5. Point the consumer at the new engine and exercise its normal workflow and recovery path.

Start with verified `Memory@2` input. An older layout needs a compatible old reader, a verified consumer export, or an implemented and tested legacy decoder. The current decoder must not be treated as an automatic `Memory@1` converter. Import the history that is actually recoverable; do not invent old events from a present-state export. Record that an imported snapshot begins the available history when that is all the source supplies.

Before new writes, rollback can restore the preserved source and old runtime. After new writes, that source is stale: rollback needs a validated reverse export or recovery forward on the new engine. Establish that path before relying on a writable pilot. Do not operate two authoritative writers during cutover.

**Done when:** the consumer works on verified imported data, reopen and restore preserve results, and the rollback procedure accounts for writes made after migration.

## 6. Consolidate the projects

Make Data OS the maintained runtime home after the integration and pilot work. Import useful code selectively, retaining attribution, license notices and source revision references. Preserve the original repositories' research records and Git history.

Retain compatibility packages, legacy importers and documented supported formats as needed. Remove duplicate runtime paths only when the consumers that rely on them have migrated or have an explicitly maintained legacy release. Archive the old repository when this is true; repository tidying should follow the working merge.

**Done when:** active consumers use one maintained engine, compatibility obligations are covered, and there is no ambiguity about which runtime owns new database development.

## Database replacement capability roadmap

These are continuing engine deliverables, not prerequisites for every pilot migration. Build each as a usable capability with tests and comparison workloads. The shared Memory experiment supplies evidence and reusable infrastructure; hard-coded domains must give way to a runtime catalog and general schema support.

| Target | Capabilities to deliver | Evidence required for a replacement claim |
|---|---|---|
| SQLite workloads | Embedded API; runtime catalog; general tables, types, NULL behavior and constraints; SQL DDL/DML; schema migration; durable local transactions; dependable backup and reopen | Real embedded consumers migrate; SQL and data results match the supported contract; transaction/recovery tests pass; operation latency, file size and maintenance costs are measured |
| PostgreSQL workloads | Multi-table transactions; tested isolation; concurrent clients; indexes, joins and planner statistics; permissions and operational tooling; replication/failover when required by the deployment | Concurrent workload correctness, anomaly tests, contention behavior and latency under load; supported driver/protocol compatibility; restore and any claimed failover behavior |
| DuckDB workloads | Column execution; efficient joins, aggregation and sorting; statistics; parallel execution; compression; bounded memory and spill; CSV/Parquet access | Analytical results and performance across data sizes, including data and intermediate results larger than RAM, with peak memory and temporary disk use recorded |
| Additional database needs | Temporal reconstruction; document, graph, full-text and vector operators/indexes derived from the same history | Representative consumer workloads; explicit consistency/freshness behavior; reproducible rebuild and mutation correctness |

These workload distinctions align with [SQLite's usage guide](https://www.sqlite.org/whentouse.html), [PostgreSQL's concurrency documentation](https://www.postgresql.org/docs/current/mvcc.html) and [DuckDB's memory-management design](https://duckdb.org/2024/07/09/memory-management).

Support is declared by implemented, tested behavior. SQL syntax alone does not provide SQLite API, PostgreSQL wire-protocol or DuckDB feature compatibility. Prioritize the statements, drivers and operations used by migrating applications, then expand the executable compatibility corpus.

Benchmark equivalent results through equivalent API boundaries, with declared durability, concurrency, data shape, cache conditions, history retention and derived-view freshness. Include ingestion, query, recovery, rebuild, checkpoint, memory and storage costs. Collect enough observations before reporting tail percentiles. Reuse existing benchmark drivers without carrying forward unmatched semantics or presenting one fast stage as whole-database superiority.

The immediate sequence is **batch Link/Delete repair plus CI, then the shared Memory experiment, then durable unified commitment**. Further estimates should come from those working increments and the actual consumer requirements rather than an unsupported calendar promise.

## Supporting review artifacts

- `merge-plan-review-assessment-2026-09-08.md`: assessment of the supplied critique, factual corrections and reproduced findings.
- `merge-plan-review-evidence-2026-09-08.zip`: eight regression probes, locked dependencies, captured output and rerun instructions.
- `rusty_multimodal_db-review-2026-09-08.md` and its evidence ZIP: the earlier detailed repository review and reproductions.

This revision changes the plan only. It does not claim that repository repairs, the shared engine or an application migration have been implemented.
