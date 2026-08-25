# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Active; semantic-envelope, workload, baseline, lifecycle/durability, crash/recovery, and environment/raw-result outputs complete; interpretation criteria incomplete | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; blocked by RQ-002 | EXP-0001 |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

The recommended next bounded RQ-002 output is predeclared interpretation criteria. This recommendation is not approval to implement, configure, or execute baselines.
