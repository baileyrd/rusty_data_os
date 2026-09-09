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
- `CHATGPT_WORKFLOW.md` — human-coordinated ChatGPT/Codex/PR workflow.
- `docs/GLOSSARY.md` — shared terminology.
- `docs/ASSUMPTIONS-AND-UNKNOWNS.md` — explicit assumptions and unresolved choices.
- `docs/RESEARCH-QUESTIONS.md` — stable research-question registry.
- `docs/REQUIREMENTS.md` — approved initial research constraints.
- `docs/TRACEABILITY.md` — links from intent through planned validation and decisions.
- `docs/hypotheses/` — falsifiable technical hypotheses.
- `docs/experiments/` — experiment definitions and conclusions.
- `docs/benchmarks/` — benchmark methodology and baseline definitions.
- `docs/adr/` — architecture decision records.
- `docs/specifications/` — specifications that have earned stability through evidence.
- `experiments/` — executable experimental implementations.
- `crates/` — code that has graduated from research into reusable engine components.

## Current phase

The owner’s merge plan step 3 now also authorizes
[EXP-0003 unified commitment](docs/experiments/EXP-0003-unified-commitment.md):
a bounded D1/D2 RF1 transaction core with Memory incarnation, retry resolution,
checkpoints and injected process-termination/torn-tail tests. Platform durability
evidence and manual descriptive measurements remain pending. No OS-crash or
power-loss claim, production graduation, or later feature integration follows.

The project has completed measurement-readiness documentation and contains bounded
experimental Rust implementations and CI. No production architecture is proven.
The existing EXP-0001 correctness and exploratory evidence remains intact.

The owner-authored [merge plan](docs/plans/data-os-multimodal-merge-plan-2026-09-08.md)
authorizes [EXP-0002](docs/experiments/EXP-0002-convergence-memory.md): shared
Memory@2 traces, an independent model, candidate/legacy runners and manual
measurements. Candidate: D1 ordinary writes, no fsync, no crash-survival claim.
Native legacy durability differs; operator measurement evidence is pending.

See [Project Status](docs/PROJECT-STATUS.md) for the authoritative current state.
