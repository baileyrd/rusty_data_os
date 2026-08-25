# Benchmark Baselines

**Status:** initial registry; specific versions/configurations must be frozen per experiment.

Rusty Data OS should compare itself against both theoretical/simple lower bounds and credible existing implementations.

## Baseline classes

### B0 — In-memory lower bound

A minimal Rust in-memory structure with no durability. This helps isolate event construction, synchronization, and memory-management overhead.

### B1 — Raw operating-system append

A minimal file append implementation using the same language/toolchain and explicit sync behavior. This approximates the direct primitive cost beneath higher abstractions.

### B2 — SQLite

Useful for local transactional durability comparisons when journal/WAL and synchronous settings are declared precisely.

Configurations must record relevant pragmas and transaction batching.

### B3 — RocksDB or comparable log-structured engine

Potential baseline for write-optimized durable ingest. Configuration must declare WAL behavior, sync settings, compression, memtable, compaction state, and batching.

### B4 — Analytic/columnar engine

A column-oriented baseline such as DuckDB may become relevant when secondary column materialization and analytic query phases begin. It is not necessarily relevant to EXP-0001's narrow ingest path.

### B5 — Specialized representations

Vector and graph engines should be introduced only when the corresponding materialization experiments begin.

## Rules

1. A baseline is included because it tests a meaningful claim, not because it is easy to beat.
2. Semantics must be documented before performance comparisons.
3. Version numbers and configuration must be captured in experiment results.
4. Baselines can change as the research question changes; historical results must remain traceable.
