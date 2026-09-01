# Research Roadmap

This roadmap defines a sequence of questions, not a fixed product plan. Each phase must earn the next with evidence.

## Phase 0 — Foundation and measurement design

**Goal:** establish the research discipline before performance code exists.

Deliverables:

- vision and principles;
- conceptual architecture;
- hypothesis and experiment lifecycle;
- benchmark methodology;
- baseline definitions;
- project continuity process;
- first experiment specification.
- Experiment 0 semantics, workload, baseline, fault-model, environment, raw-result, and interpretation prerequisites.

Exit criteria:

- `EXP-0000` is complete;
- `EXP-0001` has a reproducible benchmark plan and correctness criteria;
- baseline systems/primitives are identified;
- target test environments can be recorded consistently.

Phase 0 exited when EXP-0000 and these documentation criteria were completed. This is a readiness exit, not implementation or experimental evidence.

## Phase 1 — Canonical event ingestion

**Current substate:** planning/readiness. EXP-0001 remains proposed and non-executable. Slices A, B/B0, and A2 are merged and closed as bounded implementation/correctness-validation evidence; they supply no benchmark, durability, or performance evidence. [R12](experiments/EXP-0001/R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md) resolves BLK-006/007 as documentation design only. R16 resolves BLK-009 as documentation design; R18 subsequently authorizes only bounded raw D1 append/replay correctness. External-dependency-free generator and manifest conformance implementation exists with reviewed workspace path dependencies, but generated workload and benchmark execution remain blocked; descriptive and confirmatory execution, BLK-015, executable harness/capture, empirical equivalence, and durability claims remain blocked.

**Primary question:** what is the cost envelope of immutable event creation, sequencing, append, and different durability boundaries?

Candidate comparisons:

- memory-only append;
- buffered append;
- batched append;
- explicit sync per event;
- group commit;
- memory-mapped append where appropriate;
- alternative record encodings;
- checksummed versus non-checksummed records.

Measure:

- events/second;
- bytes/second;
- p50/p90/p95/p99/p99.9 latency;
- CPU time;
- allocations;
- memory footprint;
- storage bandwidth;
- write amplification where measurable;
- crash/recovery correctness.

Exit criteria should not simply be "fast." The experiment must identify which durability semantics and batching strategies produce useful trade spaces.

## Phase 2 — In-memory materialized state

**Primary question:** can canonical events feed an in-memory execution representation with competitive read/write behavior and acceptable replay cost?

Investigate separately:

- key/value baseline state;
- row-like state;
- column-like active state;
- indexing strategies;
- concurrency/sharding models;
- copy-on-write versus in-place updates;
- memory allocation strategies.

Critical measures:

- point-read latency;
- range/scan behavior;
- update latency;
- mixed read/write workloads;
- memory consumed per logical record;
- replay events/second;
- time-to-ready after restart.

## Phase 3 — Recovery and checkpointing

**Primary question:** how should the engine bound replay time without allowing checkpoints to become hidden canonical ownership?

Investigate:

- periodic snapshots;
- incremental checkpoints;
- checkpoint consistency;
- concurrent checkpointing overhead;
- verification against event history;
- recovery from torn/truncated records;
- checkpoint corruption behavior.

## Phase 4 — Secondary materializations

**Primary question:** can one canonical history efficiently support distinct physical representations without unacceptable write amplification or freshness lag?

Start with one materialization at a time.

Candidate sequence:

1. row-oriented persistent projection;
2. columnar analytic projection;
3. secondary index projection;
4. vector projection;
5. graph projection.

Each representation receives its own experiment and workload-specific baseline.

## Phase 5 — Query and access semantics

Only after execution and materialization performance are understood should the project select external access models.

Research topics:

- direct typed API;
- relational/query algebra;
- SQL compatibility or subset;
- document access;
- graph traversal;
- vector similarity;
- cross-materialization planning;
- optimizer behavior.

The important research question is whether the engine can select or combine representations without exposing physical ownership to applications.

## Phase 6 — Embedded engine hardening

Potential focus:

- stable APIs;
- versioned formats;
- fuzz/property testing;
- fault injection;
- observability;
- backup/restore;
- schema evolution;
- compatibility guarantees.

## Phase 7 — Server adapter

Only after the embedded core is sufficiently characterized:

- network protocol;
- sessions;
- authentication/authorization;
- connection management;
- remote transactions;
- admission control;
- client libraries;
- server-level benchmarks separated from core benchmarks.

## Phase 8 — Distributed research, if justified

Potential later topics:

- replication;
- partitioning;
- consensus;
- deterministic distributed logs;
- cross-node materialization;
- failure-domain durability.

This phase is explicitly non-committed. The project should not assume distribution is necessary until use cases and evidence justify the complexity.

### Slice A2 conformance checkpoint

R18 closes R17's external-dependency-free executable conformance, which contains reviewed workspace path dependencies, after preserving PR #64 as historical implementation authority and completing corrective PR #68 exact-head review and CI, for the frozen R12/R14/R16 contracts as
bounded correctness evidence. It is not workload execution or benchmark, persistence, durability,
or performance evidence. R17 itself did not advance Slice C/B1 or authorize a subsequent increment;
R18 supplies the separate authorization below.

### R18 Slice C/B1 readiness checkpoint

R18 closes A2 as bounded correctness evidence and prospectively authorizes only one experiment-local,
external-dependency-free raw D1 append plus reopen/replay correctness crate with exactly one reviewed workspace path dependency on `exp1-record-format`. The tranche validates R1/R5
framing, integrity, accepted physical prefixes, terminal truncation, and fail-closed scanning; it
never labels D1 bytes canonical. BLK-015, D2/D3, execution, benchmarks, physical faults, adapters,
production promotion, and every later tranche remain gated.


## R19 Phase 1 checkpoint

[R19](experiments/EXP-0001/R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md) closes
the merged raw D1 append/physical replay tranche only as bounded correctness evidence. The smallest
useful generated-workload descriptive D1 harness is not implementation-ready: the exact R7 Linux
semantic-operation-to-physical-record mapping and direct Linux capture implementation are not selected within the external-dependency-free workspace, which contains reviewed path dependencies and forbids unsafe code. Owner-reviewed mapping and capture-interface/dependency-or-bounded-unsafe decisions are the next smallest gate; caller/authority identity assignment is selected. Phase 1 execution, performance evidence, D2/D3,
faults, adapters, production, and later work remain unauthorized.

### R20 Phase 1 semantic mapping checkpoint

R20 resolves R19's semantic-to-physical ambiguity as documentation design by mapping one validated SOP1 to one structural RF1 type-3 provisional record. It prospectively authorizes only bounded mapping implementation in the existing three crates. The separate live Linux capture decision remains open, and no descriptive execution or evidence is authorized.
