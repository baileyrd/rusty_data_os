# EXP-0000 — Measurement and Semantics Readiness (Experiment 0)

**Status:** Proposed; next incomplete increment
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

## Completion criteria

Experiment 0 is complete only when each output is reviewable, links to applicable requirements and unknowns, and leaves no measurement-critical ambiguity that would make EXP-0001 results incomparable. Completion produces readiness documentation, not performance evidence.

## Out of scope

Implementation, Cargo setup, CI, benchmark execution, concrete encoding choices, schema execution, multi-event transactions, and distributed design.
