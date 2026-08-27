# Assumptions and Unknowns

## Research assumptions

| ID | Assumption | Validation path |
|---|---|---|
| ASM-001 | Canonical history can be replayed into derived state. | Experiment 0 defines semantics; later recovery experiments measure it. |
| ASM-002 | Independent materializations may provide useful workload specialization. | HYP-0001 and later materialization experiments. |
| ASM-003 | Explicit durability boundaries can be implemented and compared fairly. | Experiment 0 definitions, then EXP-0001. |

These assumptions are neither benchmark evidence nor accepted empirical conclusions.

## Open unknowns

| ID | Unknown |
|---|---|
| UNK-001 | Resolved for EXP-0001 B1 by R5: versioned `EXP1-B1-RF1` framing, bounded scanning, append-only lifecycle/final/commit records, and stable documentation vectors. |
| UNK-002 | Resolved by R3 for EXP-0001: typed UUIDv4 identities with separate authority, validation, collision, and failure rules. |
| UNK-003 | Resolved by R3 for EXP-0001: signed 64-bit Unix-epoch nanoseconds for canonical times and run-relative monotonic nanoseconds for lifecycle measurement, with explicit source/capture rules. |
| UNK-004 | Executable schema semantics and evolution mechanism. |
| UNK-005 | Concurrency, batching, and multi-event transaction design. |
| UNK-006 | Checkpoint format and validation mechanism. |
| UNK-007 | Query, materialization, retention, replication, and distributed designs. |
| UNK-008 | Quantitative thresholds for acceptable performance and ingestion-path complexity across selected workloads. The interpretation contract defines their registry fields, but justified per-cell values remain unresolved and block confirmatory interpretation until frozen by the EXP-0001 execution plan. |
| UNK-009 | Resolved by R3: the event constructor assigns after validation/system-time capture and before durable sequence reservation. |
| UNK-010 | Resolved by R3: capture once after semantic validation and immediately before event construction, then reserve sequence after construction. |
| UNK-011 | Observation-side metadata retention, observer/context identity and multiplicity rules, and criteria for appending an observation as a separate canonical event. |
| UNK-012 | Resolved for EXP-0001 B1 by R5: structural-only provisional use and the versioned CRC-32C error-detecting profile, coverage, encoding, limits, and vectors are frozen. |
| UNK-013 | Validation and locality rules for causal-event and correction/retraction references. |
| UNK-014 | R4 selects bare-metal Bosgame M5/Fedora 44 and four intended paths. The owner accepts externally reviewed but unretained host, 1 ns clock-resolution, nearest-parent XFS/LVM/NVMe, write-back, FUA, and volatile-write-cache observations for conditional planning. BLK-014 is closed for R4 planning; final placement, complete execution provenance, exact PLP/controller protection, and empirical survival remain unresolved for execution and dependent claims. |
| UNK-015 | Concrete crash/fault-injection mechanisms and implementation of physical validity detection for partial, torn, truncated, or uncertain outcomes. EXP-0000 defines the semantic procedure and R1 defines minimum deterministic classifications and fail-closed scan/recovery policy; mechanisms remain open. |
| UNK-016 | Resolved by R3: durable binding, explicit reconciliation, exact-candidate retry, conflict handling, and commit-before-ack uncertainty rules. |
| UNK-017 | Resolved by R3: durable reservation, permanent no-reuse gaps, strict monotonic replay, reporting, and fail-closed conflicts. |
| UNK-018 | Platform-independent generator, permutation, identity, and stream-digest algorithms and their reference test vectors. R2 fixes typed-input, domain-separation, digest-domain, failure, selection, and vector-plan requirements, but concrete algorithms, rationale, and stable vectors remain unresolved. |
| UNK-019 | Physical workload-manifest serialization and storage/reference mechanism. R2 constrains canonicalization, parsing, field-state, platform-independence, supersession, external-artifact, and byte-domain behavior but selects no serialization or stable vectors. |
| UNK-020 | Narrowed by complete R5 B0/B1 design mappings; implementations, validation, exact B2/B3 releases/builds/bindings/mappings, and execution inputs remain unfrozen. |
| UNK-021 | B1 D2/controlled-D3 design is mapped by R5 without empirical proof. Whether SQLite/RocksDB paths satisfy D2 or strict D3 remains open for R6 and later evidence. |
| UNK-022 | Physical serialization, timestamp representation, identity/digest algorithms, artifact retention mechanism, and automated validation for benchmark records. Logical schemas and provenance/freeze invariants are defined. |

The [EXP-0001 execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) maps unknowns to gates. R5's [focused physical contract](experiments/EXP-0001/R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) resolves UNK-001 and the B1 physical/integrity portion of UNK-012, narrows UNK-020/021, and leaves UNK-015 executable fault mechanisms open. R5 is complete; R6 is the next documentation increment. BLK-015, implementation, execution, and evidence remain open.
