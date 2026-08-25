# EXP-0000 — Measurement and Semantics Readiness (Experiment 0)

**Status:** Running; documentation and measurement-readiness work in progress
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
- [ ] baseline selection and configuration — incomplete;
- [x] acknowledgement, visibility, fault, and durability semantics — completed by the [lifecycle and durability contract](EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md);
- [x] crash/recovery procedures — completed by the [crash/recovery correctness contract](EXP-0000/CRASH-RECOVERY-CORRECTNESS.md);
- [ ] environment and raw-result templates — incomplete;
- [ ] interpretation criteria — incomplete.

The completed envelope, workload, lifecycle/durability, and crash/recovery contracts are bounded readiness outputs, not experimental evidence. Experiment 0 remains incomplete, EXP-0001 is not ready, and no implementation work is authorized. Baseline selection and configuration is the recommended next bounded output because fair baseline configuration depends on the frozen workload contract; this is a recommendation, not approval to implement or run a baseline.

## Completion criteria

Experiment 0 is complete only when each output is reviewable, links to applicable requirements and unknowns, and leaves no measurement-critical ambiguity that would make EXP-0001 results incomparable. Completion produces readiness documentation, not performance evidence.

## Out of scope

Implementation, Cargo setup, CI, benchmark execution, concrete encoding choices, schema execution, multi-event transactions, and distributed design.
