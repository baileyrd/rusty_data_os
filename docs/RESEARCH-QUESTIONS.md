# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; R1–R5 documentation is complete; BLK-001/003/016/017 and B0/B1 BLK-019 are resolved as design, while BLK-015, B2/B3, implementation, evidence, and execution remain open; R6 is next | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) records the open decisions and gates. The [R1 requirements record](experiments/EXP-0001/R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolves minimum integrity and recovery policy without evidence or implementation authorization. [R2](experiments/EXP-0001/R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) constrains deterministic inputs, digest domains, manifest serialization, and later vectors without selecting or resolving BLK-006–009. [R3](experiments/EXP-0001/R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) resolves lifecycle semantics without implementation or evidence. The owner-approved R4 evidence boundary accepts reviewed non-sensitive findings for conditional planning. [R5](experiments/EXP-0001/R5-B0-B1-PHYSICAL-PROFILES-AND-ADAPTER-CONTRACTS.md) and its [focused physical contract](experiments/EXP-0001/R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) freeze framing, CRC-32C, append-only commit, and B0/B1 mappings. R5 is complete; R6 is next, while implementation, execution, and durability claims remain blocked.
