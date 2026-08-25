# EXP-0001 — Immutable Event Ingestion

**Status:** Proposed; blocked by EXP-0000 and its documented prerequisites
**Linked hypothesis:** HYP-0001

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

Each variant must declare its integrity mode and any claimed corruption or truncation detection capability explicitly enough to measure. Per-event integrity metadata is conditional on that mode: the checksummed versus non-checksummed comparison remains a candidate, while Experiment 0 still must define the minimum required integrity policy before EXP-0001 is ready.

## 4. Durability modes

Benchmarks must follow the D0–D3 modes and result-declaration requirements in the [EXP-0000 acknowledgement, visibility, fault, and durability contract](EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md):

- **D0 — process-memory provisional acceptance:** never canonical commit.
- **D1 — OS-buffer provisional acceptance:** no explicit declared stable-storage synchronization and never canonical commit.
- **D2 — per-event declared stable-storage synchronization:** canonical only under the recorded platform durability contract.
- **D3 — grouped declared stable-storage synchronization:** canonical only under the recorded platform contract; a durability group is not an atomic multi-event transaction.

Each result must distinguish intended guarantees from fault behavior actually demonstrated. No synchronization call universally implies power-loss durability.

## 5. Workload dimensions

At minimum vary:

- event payload size;
- fixed versus variable payload size;
- single producer versus multiple producers;
- queue depth / outstanding operations;
- batch size where applicable;
- durability class.

Initial payload sizes should include small metadata-like events and larger data-bearing events; exact values must be declared before benchmark runs.

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

Required obligations, refined by the lifecycle contract and the later crash/recovery procedure:

1. every canonically acknowledged D2/D3 event is recoverable after every fault its recorded platform contract claims; D0/D1 acknowledgements remain provisional and are never recovered or exposed as committed merely because they were acknowledged;
2. recovered event order matches declared sequencing semantics;
3. partial/torn terminal records are detected and handled deterministically;
4. checksums, if enabled, detect intentional corruption within their documented capability;
5. duplicate or missing sequence identifiers are detectable;
6. replay never silently invents events.
7. explicit persistence or synchronization errors never produce successful acknowledgement; a failed D3 group acknowledges no member as committed;
8. canonical-reader and committed-history materializer visibility never precede canonical commit, while any earlier exposure is explicitly provisional;
9. every latency sample names its lifecycle interval; D3 per-event latency includes that event's group-formation wait through its own acknowledgement return.

## 8. Baselines

Baselines should include both simple primitives and established systems where semantics can be made comparable.

Potential baseline categories:

- direct Rust/OS file append implementation;
- a simple in-memory queue/vector lower bound;
- SQLite WAL modes with explicitly matched durability settings;
- RocksDB or another log-structured engine where configuration can be documented;
- other engines only when their semantics can be compared fairly.

The baseline set must be finalized before interpreting results.

## 9. Benchmark environment

Follow `docs/benchmarks/METHODOLOGY.md`. Record CPU, RAM, storage device, filesystem, OS/kernel, power settings, Rust version, compiler flags, dependency versions, and relevant storage/cache state.

## 10. Interpretation

EXP-0001 is not expected to "prove the database architecture." It should produce a trade-space map showing the real cost of event ingestion and durability under controlled conditions.

A useful result may be that some durability modes are competitive while others are not, or that batching changes the economics enough to justify a particular acknowledgement model.

## 11. Completion criteria

The experiment is complete when:

- correctness tests exist for each measured durability class;
- benchmark harness and configuration are reproducible;
- raw results are preserved;
- baselines are run under documented equivalent conditions;
- a written conclusion identifies supported, unsupported, and unresolved claims;
- HYP-0001 or follow-on hypotheses are updated without overstating evidence.
