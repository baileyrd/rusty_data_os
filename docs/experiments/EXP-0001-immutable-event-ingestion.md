# EXP-0001 — Immutable Event Ingestion

**Status:** Proposed; EXP-0000 readiness framework complete, but implementation and confirmatory execution are not authorized
**Linked hypothesis:** HYP-0001

**Readiness authority:** The [execution-readiness and staged-implementation plan](EXP-0001/EXECUTION-READINESS-PLAN.md) is the authoritative bridge from EXP-0000 to eventual implementation. R1–R7 are complete documentation inputs, including the [R7 evidence and apparatus authority](EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md), but the first-implementation, descriptive-execution, and confirmatory-execution gates have not passed.

## 1. Research question

What are the throughput, latency, CPU, memory, storage, and correctness characteristics of the smallest useful immutable event ingestion path under explicitly different durability guarantees?

## 2. Scope

Measure only:

```text
caller
  -> event construction/encoding
  -> sequencing
  -> append
  -> declared acknowledgement/durability boundary
```

This experiment deliberately excludes query execution, indexes, secondary materializations, SQL, server networking, generalized plugin infrastructure, distributed behavior, and complex multi-record transactions.

Only single-event commits are in scope. Atomic multi-event batches are deferred. Payloads may be opaque bytes carrying schema identity/version; schema execution and concrete encoding are not selected here.

## 3. Candidate variants

Variants should be added incrementally so independent costs remain visible.

Potential candidates:

1. in-memory append only;
2. buffered file append;
3. buffered batched append;
4. per-event sync durability;
5. group commit with explicit batch/window policy;
6. memory-mapped append where platform semantics can be defined precisely;
7. checksummed versus non-checksummed records;
8. selected event encodings after the basic path is understood.

Not every candidate must appear in the first implementation.

Each variant must declare its integrity mode and any claimed corruption or truncation detection capability explicitly enough to measure. R1 requires an algorithm-neutral structural-only mode for provisional/diagnostic D0/D1 work and an error-detecting mode for any D2/D3 canonical-history, recovery-correctness, or corruption-detection claim. Concrete profiles and algorithms remain open.

## 4. Durability modes

Benchmarks must follow the D0–D3 modes and result-declaration requirements in the [EXP-0000 acknowledgement, visibility, fault, and durability contract](EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md):

- **D0 — process-memory provisional acceptance:** never canonical commit.
- **D1 — OS-buffer provisional acceptance:** no explicit declared stable-storage synchronization and never canonical commit.
- **D2 — per-event declared stable-storage synchronization:** canonical only under the recorded platform durability contract.
- **D3 — grouped declared stable-storage synchronization:** canonical only under the recorded platform contract; a durability group is not an atomic multi-event transaction.

Each result must distinguish intended guarantees from fault behavior actually demonstrated. No synchronization call universally implies power-loss durability.

## 5. Workload dimensions

All streams and matrix expansions must follow the frozen [EXP-0000 reproducible workload contract](EXP-0000/WORKLOADS.md). Its primary payload points are P1 (32 bytes), P2 (256 bytes), and P3 (4 KiB); P0 and P4 are boundary diagnostics, and P5 is a separately reported optional stress case. The primary mixed profiles have exact deterministic class counts and semantic class order. This does not claim byte-for-byte regeneration until the remaining content, identity, and envelope generation rules are frozen or an immutable digested stream is supplied.

Every run declares measured extent, warm-up, producers, outstanding queue depth, batching/group policy, D0–D3 mode, payload distribution and content, envelope profile, temporal profile, generator/seed, and cache/preconditioning state. The single-producer, queue-depth-one case is the reference, not a universal concurrency prescription. Primary comparisons use deterministic high-variation content, the minimal envelope, and monotonic effective time; content, optional envelope metadata, temporal behavior, and concurrency are changed separately unless a predeclared interaction hypothesis justifies a factorial subset.

Data OS and baselines must consume semantically equivalent deterministic operation streams. The single-producer reference preserves one predeclared global ordinal-to-assigned-sequence order. Concurrent cases preserve the operation set, producer assignment, and each producer's local order; absent a controlled global submission schedule, cross-producer interleaving and the ordinal-to-assigned-sequence mapping are observed and need not match between systems. Each mapping must be recorded and checked for unique monotonic assignment. D0/D1 checks make no canonical-history or replay claim. In D2/D3, assigned sequence determines canonical replay order only for canonically committed events; deterministic replay and no unexplained loss, duplication, or invention are checked against the declared successful/eligible operation set for that mode. Under R3, a reserved sequence is never reused: failed or uncommitted candidates need not appear as canonical events, and their permanent gaps are legal and reported. Duplicate, decreasing, conflicting, zero/invalid, or above-watermark canonical positions fail closed; canonical replay requires strict increase but not contiguity. Effective time never replaces the assigned sequence or canonical replay order of committed events. Payload, encoded-event, and physical byte counts and rates remain distinct, and durability modes are never treated as equivalent guarantees.

## 6. Metrics

Collect where practical:

- events/second;
- MiB/second;
- p50 latency;
- p90 latency;
- p95 latency;
- p99 latency;
- p99.9 latency for sufficiently large samples;
- maximum observed latency, interpreted carefully;
- CPU utilization/time;
- allocations per event;
- resident memory;
- bytes written;
- write amplification if measurable;
- recovery scan throughput;
- corrupted/torn/truncated record detection behavior.

## 7. Correctness invariants

Performance results are invalid unless applicable invariants pass.

Required obligations, refined by the lifecycle contract and the [crash/recovery correctness contract](EXP-0000/CRASH-RECOVERY-CORRECTNESS.md):

1. every canonically acknowledged D2/D3 event is recoverable after every fault its recorded platform contract claims; D0/D1 acknowledgements remain provisional and are never recovered or exposed as committed merely because they were acknowledged;
2. recovered event order matches declared sequencing semantics;
3. partial/torn terminal records are detected and handled deterministically;
4. checksums, if enabled, detect intentional corruption within their documented capability;
5. duplicate or missing sequence identifiers are detectable;
6. replay never silently invents events.
7. explicit persistence or synchronization errors never produce successful acknowledgement; a failed D3 group acknowledges no member as committed;
8. canonical-reader and committed-history materializer visibility never precede canonical commit, while any earlier exposure is explicitly provisional;
9. every latency sample names its lifecycle interval; D3 per-event latency includes that event's group-formation wait through its own acknowledgement return.
10. recovery uses the predeclared oracle, injection points, fault matrix, repeat procedure, invariants, D3 rules, and pass/fail/invalid/inconclusive classifications; uncertain commit-before-acknowledgement outcomes remain explicit and corrupt or undecidable history fails closed.

## 8. Baselines

The [EXP-0001 baseline contract](../benchmarks/BASELINES.md) selects B0 minimal in-memory, B1 raw OS append, B2 SQLite WAL, and B3 RocksDB WAL. Each answers a distinct cost/behavior question and must use the frozen semantic workload through a versioned adapter.

B0 is D0 only. B1 is the primary D1/D2/controlled-D3 primitive. SQLite and RocksDB D1 are provisional and their D2 profiles are only conditionally equivalent under recorded platform contracts. Their atomic multi-event transaction/`WriteBatch` forms are not D3 equivalents; opaque internal group commit is diagnostic unless it satisfies observable D3 membership, acknowledgement, and shared-outcome semantics. Analytic/columnar, vector, graph, server, distributed, and unrelated database baselines are deferred.

R5 freezes B0/B1 design profiles. [R6](EXP-0001/R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md) freezes exact SQLite 3.53.4 and RocksDB 11.8.1 source/build/API profiles and mappings. Effective configuration, final environment/toolchain, implementation, and correctness evidence must still be validated before execution.

## 9. Benchmark environment

Follow `docs/benchmarks/METHODOLOGY.md`. Record CPU, RAM, storage device, filesystem, OS/kernel, power settings, Rust version, compiler flags, dependency versions, and relevant storage/cache state.

## 10. Interpretation

All evidence and conclusions must follow the frozen [EXP-0001 interpretation and decision contract](../benchmarks/INTERPRETATION-CRITERIA.md). Before confirmatory execution, a reviewed execution plan must freeze its threshold registry and analysis choices; unresolved entries permit descriptive or exploratory evidence only.

EXP-0001 is not expected to "prove the database architecture." It should produce a trade-space map showing the real cost of event ingestion and durability under controlled conditions.

A useful result may be that some durability modes are competitive while others are not, or that grouping changes the economics enough to justify a particular acknowledgement model. EXP-0000 completion is readiness documentation, not permission to build or run this experiment. R5 resolves BLK-001/003/016/017 and B0/B1 BLK-019 as documentation design. [R6](EXP-0001/R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md) resolves BLK-018 and the B2/B3 design portion of BLK-019 while leaving D2 conditional and strict D3 unsupported. BLK-015, implementation, empirical equivalence, and execution remain open. R7 is complete as documentation design. [R8](EXP-0001/R8-PRIMARY-MATRIX-THRESHOLDS-AND-STATISTICAL-PLAN.md) freezes the bounded candidate matrix and statistical plan but remains incomplete at an explicit owner threshold-decision boundary; R9 is not next. Owner-dependent apparatus, capture work, code, Cargo files, fixtures, validators, adapters, workflows, benchmarks, fault actions, machine changes, implementation, and execution remain unauthorized.

## 11. Completion criteria

The experiment is complete when:

- correctness tests exist for each measured durability class;
- each claimed fault class has valid crash/recovery runs under the declared platform contract, with correctness passing before its performance is interpretable;
- benchmark harness and configuration are reproducible;
- raw results are preserved;
- B0–B3 applicable profiles are run under the baseline contract, with every conditionally equivalent or diagnostic classification preserved and no excluded semantic form presented like-for-like;
- an immutable interpretation record identifies supported, refuted, constrained, and unresolved claims under the frozen criteria and threshold versions;
- HYP-0001 or follow-on hypotheses are updated without overstating evidence.
