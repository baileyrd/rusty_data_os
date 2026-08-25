# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; R1 requirements resolve BLK-002/013, but implementation and execution gates remain closed; next is R2 deterministic workload bytes, identity/reference inputs, and digests | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) now records the open decisions and gates. The [R1 requirements record](experiments/EXP-0001/R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolves minimum integrity and recovery policy without evidence or implementation authorization. The next bounded step is R2; confirmatory execution remains blocked until numeric thresholds, analysis, and all applicable physical inputs are frozen and validated.
