# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; Slices A, B/B0, and A2 are closed as bounded correctness evidence, and R18 prospectively authorizes only bounded Slice C/B1 raw D1 append and deterministic reopen/replay correctness; cost, performance, durability, generated-workload execution, benchmark evidence, BLK-015, D2/D3, `fsync`, faults, adapters, production, and later increments remain open or unauthorized | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) records the open decisions and gates. R1–R3 freeze requirements and lifecycle semantics; R4 accepts reviewed non-sensitive findings for conditional planning; R5 freezes framing, CRC-32C, append-only commit, and B0/B1 mappings. R5/R6 freeze all baseline profiles and mappings. [R7](experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) freezes evidence records, artifact provenance, instrumentation/overhead design, and the available fault boundary. R8 has frozen its bounded matrix, analysis design, and prospective owner-approved thresholds and is complete as documentation design; R9 froze the Slice A workspace/toolchain/test/CI boundary; the merged implementation passed its bounded correctness gate. R11 closes process-local, noncanonical D0 Slice B code; [R16](experiments/EXP-0001/R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md) resolves BLK-009 only as documentation design. Generator and manifest implementation still block workload observations, and execution, empirical equivalence, benchmarks, and durability claims remain blocked.

### R17 evidence note

Slice A2 answers only whether the covered frozen R12/R14/R16 vectors can be implemented together
without dependencies: its correctness suite says yes. It does not answer performance, execution,
storage, persistence, fault, durability, adapter, or production research questions.

The corrected M01/R7 fixture supports only the RQ-003 sub-question that the frozen contracts are mutually executable; it is not ingestion execution or performance evidence.
