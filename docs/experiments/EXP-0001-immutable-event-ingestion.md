# EXP-0001 — Immutable Event Ingestion

**Status:** Proposed  
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

## 4. Durability classes

Benchmarks must label the exact acknowledgement semantics. Initial classes to define may include:

- **D0 — process-memory accepted:** acknowledged after insertion into process memory; does not survive process failure.
- **D1 — OS-buffer accepted:** bytes handed to the operating system but not explicitly synchronized to stable storage.
- **D2 — stable-storage sync:** acknowledgement occurs only after the selected OS/filesystem sync operation returns successfully.
- **D3 — grouped stable-storage sync:** a group of events shares a sync operation; acknowledgement semantics and failure window must be explicit.

Names may change before implementation. Semantics must not be ambiguous.

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

Candidate invariants:

1. every acknowledged event that the selected durability class promises to retain is recoverable after the corresponding fault model;
2. recovered event order matches declared sequencing semantics;
3. partial/torn terminal records are detected and handled deterministically;
4. checksums, if enabled, detect intentional corruption within their documented capability;
5. duplicate or missing sequence identifiers are detectable;
6. replay never silently invents events.

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
