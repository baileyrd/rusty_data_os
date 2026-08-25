# Project Status

**Project:** Rusty Data OS  
**Status:** Phase 0 — Foundation and Measurement Design  
**North star:** Represent once. Materialize many. Optimize always.

## 1. Established direction

The project began from the observation that database systems commonly force early commitment to a physical representation—row, column, graph, vector, document, key/value, or similar—despite the deeper commonality that these are alternative views of data.

The working direction is to investigate a system where:

- immutable events are the candidate canonical history;
- active work occurs against in-memory state;
- persistent and specialized representations are derived materializations;
- materializations are rebuildable;
- durability boundaries are explicit;
- architecture decisions are evidence-driven;
- server behavior is deferred until the core engine has been characterized.

## 2. Agreed principles

The following are currently accepted as the project foundation, subject to revision if evidence contradicts them:

1. Events are canonical candidates; state is derived.
2. Memory is the primary execution substrate.
3. Persistence and execution should be separable concerns.
4. Materializations should be rebuildable.
5. Time/history should be first-class.
6. Storage models are projections, not assumed ownership models.
7. Every major performance/architecture claim must be benchmarkable.
8. Failed experiments are preserved.
9. Correctness and durability semantics gate performance claims.
10. The embedded/core engine comes before a server layer.

## 3. Current architecture maturity

The architecture in `ARCHITECTURE.md` is conceptual and exploratory. No event encoding, concurrency model, log format, checkpoint mechanism, transaction model, or query language has been selected as final.

## 4. Active hypothesis

`HYP-0001` asks whether an immutable canonical event history plus derived in-memory state can achieve competitive or superior behavior while decoupling logical data from physical storage representation.

This hypothesis is broad. It will be decomposed into smaller experiments rather than tested as one monolithic implementation.

## 5. Next experiment

The next planned executable work is:

**EXP-0001 — Immutable Event Ingestion**

Scope is intentionally narrow:

```text
caller -> event construction -> sequencing/append -> durability boundary
```

Excluded from EXP-0001:

- SQL;
- indexes;
- secondary materializers;
- generalized plugin framework;
- networking/server;
- transactions beyond what is required to define single-event correctness;
- distributed behavior.

## 6. Work required before EXP-0001 implementation

Before performance code is treated as benchmark evidence:

- finalize the initial benchmark environment reporting template;
- define initial event payload sizes/workload distributions;
- choose baseline primitives/systems for comparison;
- specify exact durability modes to be measured;
- define crash/recovery correctness tests;
- define raw-result storage conventions.

## 7. Decision policy

The project does not adopt architecture through consensus by intuition alone.

For major choices:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core code
```

A result may support, refute, narrow, or leave a hypothesis inconclusive.

## 8. Repository continuity

A new contributor or AI agent should read, in order:

1. `/AGENTS.md`
2. `/docs/PROJECT-STATUS.md`
3. `/docs/VISION.md`
4. `/docs/PRINCIPLES.md`
5. `/docs/ARCHITECTURE.md`
6. `/docs/RESEARCH-ROADMAP.md`
7. Applicable hypothesis, experiment, benchmark, ADR, and specification files.

The latest `main` branch is authoritative.
