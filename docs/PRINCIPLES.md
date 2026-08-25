# Core Principles

These principles define the current conceptual foundation. Some are research assumptions and may be revised if evidence contradicts them.

## P-001 — Data is more fundamental than representation

Rows, columns, vectors, graphs, documents, and indexes are representations of information optimized for different access patterns. The system should avoid allowing one representation to become the permanent conceptual owner of the data unless evidence shows that such ownership is necessary.

## P-002 — Events are the candidate canonical history

Mutations should be representable as immutable events. Events describe what happened; derived state describes the result.

This is a hypothesis-bearing principle, not yet a performance claim. The cost, encoding, ordering, storage, and recovery implications must be measured.

## P-003 — State is derived

Current state should be reproducible from an authoritative history plus explicitly identified checkpoints or snapshots.

A materialized state that cannot be rebuilt or validated against canonical history creates hidden ownership and weakens the architecture.

## P-004 — Memory is the primary execution substrate

Computational work ultimately occurs in memory. The project will investigate architectures that acknowledge this directly: keep the active working state in memory and make persistence, recovery, and secondary layouts deliberate operations around that execution substrate.

Memory is not assumed durable. Durability semantics must remain explicit.

## P-005 — Persistence and execution are separate concerns

The format best suited for durable history need not be the format best suited for active reads, analytics, vector operations, graph traversal, or archival storage.

The architecture should permit different components to optimize these concerns independently.

## P-006 — Materializations are rebuildable

A row view, column view, vector view, graph view, index, snapshot, or cache should be treated as a derived materialization unless deliberately promoted by evidence-backed decision.

Rebuildability creates freedom to change physical layout as workloads evolve.

## P-007 — Time is first-class

If history is preserved canonically, replay, rewind, point-in-time reconstruction, audit, temporal debugging, and alternative materialization become natural capabilities rather than unrelated bolt-ons.

Time semantics must be explicit: event order, logical time, wall-clock time, commit/durability time, and observation time are not automatically equivalent.

## P-008 — Durability is a contract, not a feeling

A write acknowledgement must have a precisely stated durability meaning.

Potential levels may include:

- accepted into process memory;
- accepted into a protected in-memory structure;
- written to OS buffers;
- persisted according to filesystem semantics;
- explicitly synchronized to stable media;
- replicated to another failure domain.

Benchmarks must never compare writes with different durability guarantees as though they were equivalent.

## P-009 — Asynchrony is a mechanism, not a durability guarantee

Background persistence, batching, materialization, compaction, and indexing may improve throughput. They also introduce ordering, backpressure, visibility, and failure semantics that must be explicit and tested.

## P-010 — Optimize representations independently

The same canonical data may have several physical forms simultaneously. Each form should be optimized for a declared workload and measured independently.

## P-011 — Evidence controls architectural promotion

No experimental design becomes a core architecture commitment solely because it is elegant or familiar.

The promotion path is:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core implementation
```

Not every stage is mandatory for a trivial change, but foundational decisions require evidence.

## P-012 — Failure is information

A disproven hypothesis is progress if its experiment is sound and reproducible. Negative results should remain searchable and linked to subsequent decisions.

## P-013 — Build the engine before the server

Networking, authentication, protocol design, distributed coordination, and remote-client behavior add major sources of complexity. The core execution and durability model should be tested first as an embedded/local engine.

## P-014 — Benchmark against credible alternatives

Rusty Data OS should not be benchmarked only against itself. Where appropriate, experiments should compare to simple lower bounds, standard operating-system primitives, and established database/storage engines configured with equivalent semantics.

## P-015 — Correctness gates performance

A result that is fast but unrecoverable, unordered, inconsistent, or less durable than claimed is invalid.
