# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; R1–R6 documentation is complete; BLK-018 and all baseline design mappings are resolved, while BLK-015, implementation, empirical equivalence, evidence, and execution remain open; R7 is next | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) records the open decisions and gates. R1–R3 freeze requirements and lifecycle semantics; R4 accepts reviewed non-sensitive findings for conditional planning; R5 freezes framing, CRC-32C, append-only commit, and B0/B1 mappings. [R6](experiments/EXP-0001/R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md) freezes B2/B3 identities, configuration and mappings while preserving conditional D2 and unsupported strict D3 classifications. R7 is next, while implementation, execution, empirical equivalence, and durability claims remain blocked.
