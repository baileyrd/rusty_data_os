# HYP-0001 — Immutable Event History as Canonical Data Foundation

**Status:** Active / unproven  
**Created:** 2026-08-25

## Hypothesis

A single canonical information history can support multiple independently optimized representations with acceptable performance and complexity while decoupling logical information from any single persistent row, column, vector, graph, or document representation.

## Motivation

Traditional database choices tend to combine logical data ownership with a physical model optimized for specific workloads. If canonical history can be stored independently and materializations can be rebuilt, the system may be able to optimize different access patterns without forcing one representation to own the truth.

## Why this is not yet a conclusion

The approach may fail because of:

- event serialization overhead;
- write amplification;
- replay/recovery time;
- memory consumption;
- synchronization cost;
- ordering bottlenecks;
- checkpoint complexity;
- secondary materialization lag;
- transaction semantics;
- schema evolution cost;
- excessive operational complexity.

Any of these may outweigh the flexibility benefit.

## Decomposition

HYP-0001 will be tested through smaller hypotheses/experiments, beginning with:

- cost of event construction and append;
- cost of explicit durability levels;
- replay throughput;
- in-memory state update/read cost;
- checkpoint/recovery behavior;
- independent secondary materialization cost.

## Falsification direction

The hypothesis should be narrowed or rejected if, under equivalent correctness and durability semantics, the architecture consistently exhibits unacceptable regressions against credible baselines and no compensating flexibility/recovery/materialization benefit justifies those regressions.

No single synthetic benchmark can prove HYP-0001. The purpose of the research program is to progressively constrain the conditions under which the hypothesis is true or false.

## Related experiments

- `EXP-0000` — Measurement and Semantics Readiness (complete as documentation; no evidence)
- `EXP-0001` — Immutable Event Ingestion (proposed; execution-readiness plan exists, gates remain closed)

## Related decisions

- `ADR-0001` — Evidence-driven architecture process

## Evidence summary

The [bounded exploratory history-to-views run](../experiments/exploratory-history-views/README.md) independently reconstructed row and column final and midpoint states correctly from reopened RF1 history in every corrected measured trial. The original dependent-column results are retained but superseded for independent-view comparison. Physical replay dominated at 10,000 events and showed sharply declining throughput in both runs. This is feasibility and cost-direction evidence only, not confirmatory EXP-0001 evidence; HYP-0001 remains active and unproven.
