# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; R1 resolves BLK-002/013, R2 constrains open BLK-006–009, and R3 resolves BLK-004/005/011/012 while further constraining BLK-007; R4 documentation selects the target but BLK-014/015 remain evidence-pending; implementation and execution gates remain closed | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) now records the open decisions and gates. The [R1 requirements record](experiments/EXP-0001/R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolves minimum integrity and recovery policy without evidence or implementation authorization. [R2](experiments/EXP-0001/R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) constrains deterministic inputs, digest domains, manifest serialization, and later vectors without selecting or resolving BLK-006–009. [R3](experiments/EXP-0001/R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) resolves lifecycle semantics without implementation or evidence. R4 is documentation-complete but exact target evidence remains blocked; owner capture/review is next and confirmatory execution remains blocked until numeric thresholds, analysis, and all applicable physical inputs are frozen and validated.
