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

Exit criteria:

- `EXP-0001` has a reproducible benchmark plan and correctness criteria;
- baseline systems/primitives are identified;
- target test environments can be recorded consistently.

## Phase 1 — Canonical event ingestion

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
