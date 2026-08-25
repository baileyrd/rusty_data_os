# Experiment Registry

Experiments are bounded investigations designed to test one or more falsifiable claims.

## Required experiment structure

Each experiment document should contain:

1. identifier and title;
2. status;
3. linked hypothesis;
4. research question;
5. hypothesis under test;
6. independent variables;
7. controlled variables;
8. workloads;
9. correctness invariants;
10. benchmark metrics;
11. environment requirements;
12. baselines;
13. predeclared interpretation criteria where practical;
14. implementation notes;
15. raw result locations;
16. results;
17. conclusion;
18. follow-on questions.

## Status values

- **Proposed** — design exists, implementation not started.
- **Ready** — methodology is sufficient to implement.
- **Running** — evidence collection is in progress.
- **Complete** — results and conclusion are recorded.
- **Inconclusive** — evidence does not support a sound decision.
- **Superseded** — replaced by a later experiment.

## Rule

Do not rewrite an experiment's original hypothesis after observing results. Add clarifications or follow-up experiments so the history of reasoning remains inspectable.

## Registry

| Experiment | Status | Purpose |
|---|---|---|
| [EXP-0000](EXP-0000-measurement-and-semantics-readiness.md) | Complete as readiness documentation; no evidence | Define semantics and measurement prerequisites without implementation. All seven contracted outputs are complete. |
| [EXP-0001](EXP-0001-immutable-event-ingestion.md) | Proposed; planning/readiness only | Measure single-event ingestion after the [execution-readiness plan](EXP-0001/EXECUTION-READINESS-PLAN.md) authorizes one earned implementation slice and the applicable execution gate passes. |
