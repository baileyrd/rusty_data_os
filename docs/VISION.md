# Vision

## 1. Problem statement

Current database systems tend to lock users into one primary physical and operational model: row-oriented, column-oriented, document-oriented, graph-oriented, vector-oriented, key/value, log-structured, or another specialized representation. Each model has legitimate strengths and trade-offs, but the choice often becomes deeply embedded in the architecture and application interface.

At the lowest level, however, it is all data. Rows, columns, graphs, vectors, documents, indexes, and other structures are different ways of viewing, interpreting, organizing, and optimizing the same underlying information.

The problem Rusty Data OS investigates is therefore:

> **Can the logical truth and durable history of data be separated from any single physical representation, allowing the same canonical information to be materialized into multiple optimized forms without making one form the permanent owner of the data?**

## 2. Systems-engineering perspective

Rusty Data OS approaches database design through a systems-engineering and model-based-systems-engineering lens.

The system should distinguish:

- the underlying information;
- the events that change that information;
- the current state derived from those events;
- the physical representations used to optimize particular interactions;
- the views through which users and applications interpret the information.

A representation is therefore treated as a view or projection with measurable qualities, not as an unquestioned definition of the data itself.

## 3. North star

> **Represent once. Materialize many. Optimize always.**

"Represent once" means preserve a canonical history of change without forcing every access pattern into one physical layout.

"Materialize many" means derive representations appropriate to workloads: in-memory state, rows, columns, vectors, graphs, indexes, snapshots, archives, replication streams, or representations not yet anticipated.

"Optimize always" means physical representations may evolve according to evidence and workload requirements without changing the fundamental identity of the underlying information.

## 4. Initial thesis

The primary research claim is:

> **A single canonical information history can support multiple independently optimized representations with acceptable performance and complexity.**

The initial research thesis is that a system built around:

1. immutable canonical events;
2. memory as the primary execution substrate;
3. rebuildable materializations;
4. explicit durability boundaries; and
5. evidence-driven optimization

may reduce the architectural lock-in created by traditional storage-first database design while remaining competitive in latency, throughput, recovery, and durability.

This thesis is intentionally not assumed true. It must be tested.

## 5. What success means

Success is not simply creating another database.

Success means demonstrating, with reproducible evidence, that the architecture can provide meaningful benefits in one or more dimensions without unacceptable regressions in others. Relevant dimensions include:

- ingest throughput;
- read latency;
- write latency;
- tail latency;
- memory efficiency;
- CPU efficiency;
- storage bandwidth and amplification;
- recovery time;
- rebuild time;
- durability guarantees;
- representation flexibility;
- operational complexity;
- correctness under concurrency and failure.

The project should be willing to conclude that a proposed idea does not work, works only under limited conditions, or should be abandoned.

## 6. Scope progression

The intended progression is:

```text
research harness
    -> event ingestion substrate
    -> in-memory execution/materialization
    -> durable history and recovery
    -> additional materialization strategies
    -> query/access interfaces
    -> embedded reusable engine
    -> server adapter
    -> distributed capabilities only if justified
```

The server is intentionally not the starting point. The core execution and persistence ideas must first earn confidence independently of network and service-layer complexity.

## 7. Initial non-goals

The following are not initial goals:

- full SQL compatibility;
- distributed consensus;
- multi-node replication;
- cloud service deployment;
- generalized plugin marketplaces;
- replacing every existing database workload;
- immediately supporting every data model;
- committing to a permanent event encoding before measurement;
- hiding durability semantics behind vague "successful write" behavior.

These may become future research topics, but they must not dilute early experiments.

## 8. Research culture

The project is fact-based and data-driven.

Feelings, familiarity, elegance, and intuition are useful for generating candidate ideas. They are not sufficient evidence for selecting an architecture.

The governing principle is:

> **Intuition proposes. Measurement challenges. Evidence decides.**

## 9. A parallel, disclosed initiative (2026-09-09)

The owner-authorized [2026-09-08 merge plan](plans/data-os-multimodal-merge-plan-2026-09-08.md)
is a separate initiative to build a general-purpose database intended to replace SQLite,
PostgreSQL and DuckDB workloads, with this repository as "the eventual home of the combined
engine." New database-engine development for that goal happens under
`experiments/unified-commitment/`, in parallel with, not instead of, the independent EXP-0001
exploratory-history research otherwise described here. `rusty_multimodal_db` remains the active
source repository; its consumers have not migrated, and it is neither archived nor deprecated.

**Superseded only for this initiative, by the named authorizations and to the stated extent:**

- §7's "full SQL compatibility" and "replacing every existing database workload" non-goals yield
  to the merge plan's stated replacement-database end goal. This changes direction, not the
  completion claim: Steps 1–5 have ported basic operations for three domains, not full coverage.
- §6's "server adapter" ordering yields only to the existing EXP-0005 exception in
  [Research Roadmap, "Phase 7 — Server adapter"](RESEARCH-ROADMAP.md#phase-7--server-adapter).
  That note defines the authorization's extent; this section does not broaden it.

**Retained unchanged and fully in force for the merge initiative:**

- "cloud service deployment";
- "generalized plugin marketplaces";
- "distributed consensus";
- "multi-node replication";
- "immediately supporting every data model";
- "committing to a permanent event encoding before measurement";
- "hiding durability semantics behind vague 'successful write' behavior."

No prior merge step or this work order lifts those retained exclusions. The encoding and
durability items remain engineering-rigor principles: Step 4c explicitly versioned CMM2→CMM3,
and Step 5 labels its `Durability::D1` choice "synthetic D1 fidelity only." See
[EXP-0005 §§22–23](experiments/EXP-0005-protocol-facade.md#22-step-4c-frozen-method-and-local-implementation)
and the [Step 5 test](../experiments/unified-commitment/crates/uc-facade/tests/remind_me_migration.rs).
§5's success criteria remain unmet and are not claimed met. Sections 1–8 retain their content,
research culture and evidence requirements; only the scoped exceptions listed above apply to
the parallel initiative. No real consumer cutover, production graduation or full consolidation
is claimed.
