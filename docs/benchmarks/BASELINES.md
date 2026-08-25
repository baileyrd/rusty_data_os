# EXP-0001 Baseline Contract

**Status:** EXP-0000 baseline selection and semantic configuration complete; no baseline has been implemented, installed, executed, or measured.

This contract selects the EXP-0001 controls that isolate useful costs under the frozen [workload](../experiments/EXP-0000/WORKLOADS.md), [acknowledgement/durability](../experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), and [crash/recovery](../experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) contracts. Selection is not a prediction that Data OS will outperform a baseline. No configuration below is benchmark evidence or authorization to begin EXP-0001.

## 1. Selected baselines and questions

| ID | Class and isolated behavior | Relevance and claim challenged | Modes and limits | Expected adapter cost/mismatch |
|---|---|---|---|---|
| **B0** | **Lower bound:** minimum process-memory copy/store and assigned-sequence work. | Separates unavoidable workload/sequence/memory costs from persistence and challenges any claim that Data OS ingestion overhead is near a minimal in-process path. | D0 only. D1–D3 are unsupported. | Smallest adapter, but still includes semantic-stream validation/mapping. It has no recovery, canonical commit, WAL, or database semantics. |
| **B1** | **Primitive comparison:** minimal raw operating-system append and declared synchronization path. | Exposes primitive write, synchronization, grouping, and framing costs below an engine; challenges claims that higher-level ingestion adds little cost beyond its required persistence boundary. | Primary baseline for D1, D2, and controlled D3. D0 is unsupported because B0 isolates it without file-path work. | Requires future framing/integrity and recovery choices; those costs cannot be hidden. Platform APIs and guarantees differ. |
| **B2** | **Established embedded transactional comparison:** SQLite in WAL mode. | Tests the same semantic operation stream through a mature transactional storage path and challenges whether the proposed path provides useful ingestion trade-offs over an established embedded engine. | D1 candidate; D2 conditionally equivalent; D3 diagnostic/non-equivalent initially; D0 prohibited like-for-like. | Row/schema, SQL/binding, B-tree/page, transaction, WAL, and checkpoint work differ from one canonical event. Adapter and byte expansion are reported. |
| **B3** | **Established write-optimized comparison:** RocksDB with WAL enabled. | Tests the stream through a mature LSM/WAL write path and challenges whether the proposed path offers useful costs or behavior beyond a write-optimized engine. | D1 candidate; D2 conditionally equivalent; opaque internal D3 diagnostic only; D0 and WAL-disabled-as-B0 are prohibited. | Key/value mapping, sequence/WAL/memtable, compaction, background work, and recovery semantics differ. Adapter and physical amplification are reported. |

These controls answer progressively different questions; they are not a single winner/loser ladder. D0/D1 results are provisional and must not be ranked as equivalent to canonical D2/D3 commit.

## 2. Semantic-equivalence matrix

| Baseline | D0: process memory | D1: OS-buffer acceptance | D2: per-event sync | D3: shared sync, individual events |
|---|---|---|---|---|
| **B0** | **Equivalent candidate** for provisional D0 only | **Unsupported** | **Unsupported** | **Unsupported** |
| **B1** | **Unsupported**; use B0 | **Equivalent candidate** after profile freeze and gates | **Conditionally equivalent** under the recorded platform durability contract | **Conditionally equivalent** only with observable membership, one shared sync outcome, and individual event acknowledgement; never an atomic batch |
| **B2 SQLite WAL** | **Prohibited as like-for-like** | **Equivalent candidate** only as externally classified provisional D1 | **Conditionally equivalent** under recorded VFS/platform semantics | **Diagnostic only** initially; an atomic multi-event transaction is prohibited as D3 equivalence |
| **B3 RocksDB WAL** | **Prohibited as like-for-like** | **Equivalent candidate** only as externally classified provisional D1 | **Conditionally equivalent** under recorded sync/platform semantics | **Diagnostic only** for opaque internal group commit; `WriteBatch` is prohibited as D3 equivalence |

“Equivalent candidate” means eligible for correctness validation, not already proven equivalent. “Conditionally equivalent” requires the applicable platform contract, adapter mapping, acknowledgement point, and recovery oracle to pass. Internal or opaque group commit is not strict D3 unless exact membership, join/cut behavior, acknowledgements, and the shared durability outcome satisfy the D3 contract. Physical coalescing alone is insufficient.

## 3. B0 — minimal in-memory lower bound

The future B0 consumes the same semantic operation stream and performs only the minimum event copy/store and assigned-local-sequence work required by its frozen profile. It performs no persistence, file I/O, background flush, replication, materialization, checkpoint, or hidden durability work and returns only D0 provisional acknowledgement. Nothing stored by B0 is canonical commit.

B0 must report allocations, bytes copied, sequence-assignment cost, resident/allocated memory, latency, and throughput. Before execution, its data structure, ownership/copy behavior, capacity/growth and allocation strategy, sequence mechanism, producer model, and instrumentation must be frozen. This document deliberately selects no Rust data structure. B0 is a lower-bound control, not a database comparison.

## 4. B1 — raw operating-system append

Three future profiles are selected:

- **B1-D1:** encode/frame as later declared, complete every partial write, and acknowledge after the OS accepts the append/write, with no declared stable-storage synchronization. The result is provisional.
- **B1-D2:** append one independently identified event, invoke the profile's declared synchronization primitive for that event, and acknowledge canonical commit only after successful completion under the platform durability contract.
- **B1-D3:** join independently identified single events to a predeclared group, perform one declared synchronization operation shared by its exact membership, then acknowledge each eligible event according to that shared outcome. This is not an atomic multi-event transaction.

Every executable B1 profile must freeze and record:

1. OS/platform API and exact open, encode/frame, write/append, synchronize, and acknowledge operation sequence;
2. user-space, runtime, and kernel buffering layers, including whether any buffer flush precedes synchronization;
3. file create/open flags, permissions, append/offset behavior, concurrency and serialization rules;
4. short/partial write, interruption, error, retry, and uncertain-outcome handling;
5. synchronization primitive and its data/metadata scope, plus directory synchronization for creation/rename where applicable;
6. group membership, maximum size/window, cut triggers, join cutoff, synchronization ownership, and per-event acknowledgement point;
7. OS/filesystem/mount/device/cache platform durability contract and promised fault classes;
8. integrity/framing and deterministic recovery behavior under the recovery oracle;
9. file initial state, rotation, allocation/preallocation, reuse, and cache/preconditioning state; and
10. instrumentation placement and measured overhead.

Rust documents `File::sync_all` as attempting to synchronize data and metadata and `File::sync_data` as potentially omitting metadata; both may map to platform-specific calls and errors ([official Rust `File` documentation](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all)). They are therefore candidate mechanisms, not universal power-loss guarantees. This increment does not universally select either one.

## 5. B2 — SQLite WAL

SQLite is selected as the embedded transactional baseline. No moving “current” version is frozen here: the exact SQLite release, amalgamation/source identifier, source digest, build options, wrapper version, and VFS must be pinned in the future environment manifest before execution. SQLite release behavior changes over time, as shown by the [official release history](https://www.sqlite.org/changes.html); results from different versions are distinct benchmark series and may not be pooled.

Initial profiles:

- **SQLite-D1:** `journal_mode=WAL`, `synchronous=NORMAL`, exactly one semantic event per transaction, and acknowledgement after transaction completion. Disable automatic checkpoints during the measured interval where the pinned release supports it; record explicit/unavoidable checkpoints and the initial/new/reused WAL state. For Data OS comparison this is D1 provisional despite SQLite transaction terminology. SQLite documents that WAL mode with `NORMAL` omits a synchronization operation during most transactions and may lose durability after power loss ([official `PRAGMA synchronous` documentation](https://www.sqlite.org/pragma.html#pragma_synchronous)).
- **SQLite-D2:** `journal_mode=WAL`, `synchronous=FULL`, exactly one semantic event per transaction, and acknowledgement after transaction completion. This is only conditionally equivalent under the recorded SQLite VFS, OS, filesystem, mount, device/cache, and fault/durability contract. The same official documentation describes the additional WAL synchronization associated with `FULL`; the actual promised failure boundary remains platform-specific.
- **SQLite-D3:** no strict initial equivalent. Placing multiple events in one SQLite transaction adds atomic multi-event transaction semantics and must not be described as equivalent to Data OS single-event group durability. Any internal or measured checkpoint/commit coalescing is diagnostic unless D3 membership and outcomes are observable and matched.

The adapter must use a neutral minimal schema sufficient to preserve the semantic operation, document its rationale, and expose every transformation; it must not invent a schema designed to weaken SQLite. Before every series, set requested PRAGMAs and then query/record their effective values rather than assuming acceptance. Freeze or record page size, WAL autocheckpoint, journal mode, synchronous level, transaction boundaries, connection count, busy timeout/handler and locking behavior, database/WAL/SHM initial and reuse state, checkpoint activity, schema and indexes, compile options, VFS, cache/preconditioning, adapter mapping, and payload/encoded/physical byte accounting.

## 6. B3 — RocksDB with WAL

RocksDB is selected as the established write-optimized LSM/WAL baseline. As with SQLite, the exact release/tag or commit, source digest, build configuration, dependency and binding versions must be pinned in the future environment manifest. The [official RocksDB releases](https://github.com/facebook/rocksdb/releases) are the version source; no results may be mixed across releases.

Initial profiles:

- **RocksDB-D1:** WAL enabled, `WriteOptions.sync=false`, and normal WAL flushing to the OS rather than `manual_wal_flush` process-only retention. A successful write is externally D1 provisional. Official RocksDB documentation states that writes normally go to the WAL and memtable and describes `sync=false` as relying on OS flushing ([Basic Operations](https://github.com/facebook/rocksdb/wiki/Basic-Operations)).
- **RocksDB-D2:** WAL enabled, `WriteOptions.sync=true`, one logical event per write call, and acknowledgement after successful return. Conditional equivalence depends on the pinned version's actual synchronization path (`use_fsync` included) and recorded platform durability contract.
- **RocksDB-D3:** concurrent internal group commit may be measured only as diagnostic unless exact membership, join/cut and per-event acknowledgement/shared durability outcomes can be observed and matched. The official [WAL performance documentation](https://github.com/facebook/rocksdb/wiki/WAL-Performance) describes concurrent writes sharing WAL synchronization, but opaque coalescing does not establish strict D3 equivalence. `WriteBatch` must not claim D3 equivalence because it introduces atomic multi-key/event batch semantics; official documentation describes a write batch as atomic ([Basic Operations](https://github.com/facebook/rocksdb/wiki/Basic-Operations#atomic-updates)).
- **RocksDB without WAL:** excluded as a substitute for B0. If a later question measures it, it is a separate diagnostic profile because memtable and background persistence/flush behavior differ from a pure in-memory lower bound. The official [WAL behavior documentation](https://github.com/facebook/rocksdb/wiki/Write-Ahead-Log-%28WAL%29) supplies version-dependent behavior to verify when the profile is pinned.

Before execution, freeze and record exact version/source/build; WAL enablement; `WriteOptions.sync`; `manual_wal_flush`; `use_fsync`; WAL directory; WAL compression; column-family count; write batching; memtable configuration/state; compaction configuration/state; background threads, flushes, stalls, and other work; data compression; cache configuration/state; database new/reused/preconditioned state; recovery mode; keys and values; adapter mapping; logical, encoded, WAL/SST and other measurable physical bytes; and every condition preventing equivalence. Record every relied-upon default and verify it against the pinned source/version—defaults must never be silently inherited.

## 7. Adapter fairness contract

Every baseline consumes an equivalent operation set from the frozen workload contract. An adapter must:

1. preserve payload bytes exactly and preserve request identity versus event identity rather than collapsing them;
2. preserve effective-time relationships, applicable references, and the selected semantic envelope/content/temporal profiles;
3. record workload ordinal, assigned local sequence, baseline-native ordering/identity, and their mapping; a baseline-native order does not replace Data OS ordering semantics;
4. include event construction, validation, transformation, sequence assignment, database calls, and acknowledgement work in the applicable measured interval; precomputing or moving baseline-specific work outside it is prohibited unless the same declared phase applies to every subject;
5. report payload bytes, baseline encoded/key/row/value bytes, and measurable physical bytes separately, including amplification and unattributable bytes;
6. keep setup, database creation, preconditioning, checkpoint, compaction, cleanup, and recovery outside the ingestion interval unless the declared question intentionally includes them; unavoidable background work is recorded and measured rather than hidden;
7. report unsupported fields or semantics as mismatches—never fabricate, discard, or silently reinterpret them; and
8. version the mapping and retain enough detail to reconstruct it.

A baseline transaction, key, row, WAL record, page, SST entry, or write batch is not automatically one Data OS canonical event. Fairness is semantic, not merely equal API-call count or payload size.

## 8. Version and configuration freeze policy

Each benchmark series has an immutable identity containing: product/library version; tag/commit/amalgamation or other source identity and digest; build flags/features and compiler/toolchain; binding/wrapper version; configuration file or complete explicit option set including verified defaults; adapter version/commit; workload-contract and immutable-stream/manifest identity; D-mode and platform durability contract; environment identity; and repository commit.

A change creates a **new series**, not a repeat, when it can affect executed code, semantic equivalence, acknowledgement/durability, operation mapping, physical work, recovery, background behavior, or measured environment. This includes product/binding/adapter versions, build options, relied-upon defaults, schema/key mapping, synchronization or batching, workload-contract revision, storage stack, or fault contract. Pure run identifier/time and repetition number changes remain repetitions only when the frozen identity is otherwise byte-for-byte identical. Cross-series analysis may show separately labeled results but must never silently pool samples.

## 9. Correctness gates before performance interpretation

Every measured profile must first demonstrate that:

- its acknowledgement and visibility evidence matches the declared D-mode; D0/D1 never become canonical merely because bytes later survive;
- recovered data passes the applicable crash/recovery oracle and promised fault classes;
- rejected, failed, provisional, corrupt, or uncertain operations are never silently promoted;
- workload ordinal, assigned sequence, and native ordering/identity mappings are complete and valid;
- D2/D3 claims extend only to fault classes promised and demonstrated by the recorded platform contract;
- adapter transformations preserve the semantic operation set and byte identities;
- checkpoints, compactions, flushes, stalls, and other background work are controlled or measured; and
- every mismatch is labeled diagnostic, non-equivalent, invalid, or inconclusive as applicable.

A failing configuration has no interpretable performance result. These gates do not resolve the still-open physical integrity, framing, fault-injection, or platform durability choices.

## 10. Selection, exclusions, and replacement

B0–B3 cover the minimum in-process cost, raw persistence primitive, embedded transactional path, and write-optimized LSM/WAL path needed to challenge EXP-0001's narrow ingestion claims. B4 analytic/columnar engines including DuckDB, B5 vector/graph engines, servers, distributed systems, and unrelated databases are deferred until experiments study their representations or operational semantics; adding them now would answer different questions and introduce unrelated work.

A baseline may be removed or replaced only when it is unavailable under a reproducible supported source/build, its semantics cannot answer the declared question, a defect invalidates it, or a revised research question makes it irrelevant. The change requires documented rationale, a new contract/version, and preserved historical configurations/results. A difficult configuration, operational burden, unfavorable result, or inability of Data OS to outperform it is not grounds for replacement. A replacement starts a new series and never rewrites prior evidence.

## 11. Deliberately unresolved

This selection does not choose product binaries, exact versions, bindings, adapters, SQLite schema, RocksDB key/value mapping, B0 structure, B1 API/sync primitive, event encoding/framing, integrity mechanism, platform durability guarantee, checkpoint behavior, retry/idempotency, identity/timestamp/clock algorithms, sequencing-gap policy, transactions, or distributed design. Those choices require later bounded freezes and correctness validation. The recommended next bounded EXP-0000 output is benchmark environment and raw-result templates; that recommendation is not authorization to install, implement, execute, or benchmark anything.
