# EXP-0000 — Measurement and Semantics Readiness (Experiment 0)

**Status:** Complete; readiness documentation only, with no experimental evidence
**Linked hypothesis:** HYP-0001 (prerequisite only)

## Purpose

Experiment 0 is a documentation and measurement-readiness increment. It does not implement or benchmark the engine. Its purpose is to make EXP-0001 falsifiable, reproducible, and correctness-gated before executable work begins.

## Required outputs

1. freeze the minimal single-event envelope fields at a semantic level without selecting binary encoding, identity algorithm, or clock source;
2. define workload payload sizes and distributions;
3. select and configure fair baseline primitives/systems;
4. specify acknowledgement, visibility, fault, and durability semantics;
5. define crash/recovery correctness procedures;
6. create benchmark environment and raw-result templates;
7. predeclare interpretation criteria, including how acceptable performance and complexity will be evaluated.

## Output checklist

- [x] semantic event envelope — completed by this increment in the [semantic event envelope contract](EXP-0000/SEMANTIC-EVENT-ENVELOPE.md);
- [x] workload payloads and distributions — completed by the [reproducible workload contract](EXP-0000/WORKLOADS.md);
- [x] baseline selection and configuration — completed by the [EXP-0001 baseline contract](../benchmarks/BASELINES.md);
- [x] acknowledgement, visibility, fault, and durability semantics — completed by the [lifecycle and durability contract](EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md);
- [x] crash/recovery procedures — completed by the [crash/recovery correctness contract](EXP-0000/CRASH-RECOVERY-CORRECTNESS.md);
- [x] environment and raw-result templates — completed by the [environment record contract](../benchmarks/ENVIRONMENT-TEMPLATE.md) and [raw-result record contract](../benchmarks/RAW-RESULT-TEMPLATE.md);
- [x] interpretation criteria — completed by the [EXP-0001 evidence interpretation and decision contract](../benchmarks/INTERPRETATION-CRITERIA.md).

All seven outputs are complete and cross-linked. They are bounded readiness outputs, not experimental evidence. EXP-0000 completion neither resolves its listed physical choices nor automatically authorizes EXP-0001 implementation or execution. The next bounded step is an explicit EXP-0001 readiness/implementation proposal, requiring repository review, that freezes the execution plan, threshold registry, and remaining measurement-critical choices before confirmatory execution.

## Completion criteria

Experiment 0 is complete only when each output is reviewable, links to applicable requirements and unknowns, and leaves no measurement-critical ambiguity that would make EXP-0001 results incomparable. Completion produces readiness documentation, not performance evidence. This criterion is satisfied by the seven linked contracts; unresolved EXP-0001 execution choices are explicit blockers to confirmatory execution rather than omissions from this framework.

## Out of scope

Implementation, Cargo setup, CI, benchmark execution, concrete encoding choices, schema execution, multi-event transactions, and distributed design.
