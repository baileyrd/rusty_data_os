# Rusty Data OS

**North star:** **Represent once. Materialize many. Optimize always.**

Rusty Data OS is an evidence-driven research and engineering project exploring a different foundation for database systems. The central idea is to separate the meaning and history of data from any single physical representation of that data.

Rather than treating a row store, column store, vector store, graph store, or file format as the authoritative owner of data, Rusty Data OS investigates whether a canonical immutable event history can serve as the durable source from which optimized in-memory and persistent materializations are derived.

This repository intentionally begins as a research program before it becomes a software product. Architectural ideas are hypotheses until experiments and benchmarks provide evidence.

## Core idea

Current database systems commonly require an early commitment to a storage and access model. Those choices are useful, but they also couple the logical meaning of data to physical representations optimized for particular workloads.

Rusty Data OS explores the alternative proposition that the underlying truth can be represented once and projected into many optimized forms.

```text
Command / mutation intent
        |
        v
Canonical immutable event history
        |
        +--> In-memory materialization
        +--> Row materialization
        +--> Column materialization
        +--> Vector materialization
        +--> Graph materialization
        +--> Index materialization
        +--> Archive / replication materialization
        +--> Future / unknown representations
```

No materialization owns the data. Materializations are rebuildable views of canonical history.

## Engineering rule

**Intuition may create a hypothesis. Evidence determines whether the hypothesis survives.**

The project lifecycle is:

```text
Idea -> Hypothesis -> Experiment -> Measurement -> Evidence -> Decision -> Architecture
```

Failed experiments are preserved as first-class engineering artifacts so the project does not repeatedly rediscover rejected approaches.

## Repository map

- `docs/VISION.md` — problem statement, north star, goals, and boundaries.
- `docs/PRINCIPLES.md` — foundational engineering principles.
- `docs/ARCHITECTURE.md` — current conceptual architecture; explicitly non-final.
- `docs/RESEARCH-ROADMAP.md` — staged research program.
- `docs/PROJECT-STATUS.md` — authoritative continuity point for current state.
- `docs/hypotheses/` — falsifiable technical hypotheses.
- `docs/experiments/` — experiment definitions and conclusions.
- `docs/benchmarks/` — benchmark methodology and baseline definitions.
- `docs/adr/` — architecture decision records.
- `docs/specifications/` — specifications that have earned stability through evidence.
- `experiments/` — executable experimental implementations.
- `crates/` — code that has graduated from research into reusable engine components.

## Current phase

The project is in **Phase 0: foundation and measurement design**. No production architecture is considered proven yet.

The first planned technical experiment is `EXP-0001`: immutable event ingestion, measuring the path from caller to event construction, append, and explicitly defined durability boundaries before introducing SQL, indexes, networking, generalized plugin frameworks, or persistent secondary representations.

See `docs/PROJECT-STATUS.md` for the authoritative current state.
