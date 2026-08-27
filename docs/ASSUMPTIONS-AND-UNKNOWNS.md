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
| UNK-001 | Concrete event binary encoding and record framing. |
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
| UNK-012 | Physical event framing and integrity algorithm/profile details. R1 resolves the minimum integrity capability classes, semantic coverage, finalization, and failure policy; BLK-001 and BLK-003 mechanisms remain open. |
| UNK-013 | Validation and locality rules for causal-event and correction/retraction references. |
| UNK-014 | R4 selects bare-metal Bosgame M5/Fedora 44 and four intended paths. The owner accepts externally reviewed but unretained host, 1 ns clock-resolution, nearest-parent XFS/LVM/NVMe, write-back, FUA, and volatile-write-cache observations for conditional planning. BLK-014 is closed for R4 planning; final placement, complete execution provenance, exact PLP/controller protection, and empirical survival remain unresolved for execution and dependent claims. |
| UNK-015 | Concrete crash/fault-injection mechanisms and implementation of physical validity detection for partial, torn, truncated, or uncertain outcomes. EXP-0000 defines the semantic procedure and R1 defines minimum deterministic classifications and fail-closed scan/recovery policy; mechanisms remain open. |
| UNK-016 | Resolved by R3: durable binding, explicit reconciliation, exact-candidate retry, conflict handling, and commit-before-ack uncertainty rules. |
| UNK-017 | Resolved by R3: durable reservation, permanent no-reuse gaps, strict monotonic replay, reporting, and fail-closed conflicts. |
| UNK-018 | Platform-independent generator, permutation, identity, and stream-digest algorithms and their reference test vectors. R2 fixes typed-input, domain-separation, digest-domain, failure, selection, and vector-plan requirements, but concrete algorithms, rationale, and stable vectors remain unresolved. |
| UNK-019 | Physical workload-manifest serialization and storage/reference mechanism. R2 constrains canonicalization, parsing, field-state, platform-independence, supersession, external-artifact, and byte-domain behavior but selects no serialization or stable vectors. |
| UNK-020 | Exact EXP-0001 baseline releases/source identities, builds, bindings, physical adapters/mappings, verified defaults, and environment-specific configurations. The baseline families and semantic profiles are selected by the baseline contract, but these execution inputs remain unfrozen. |
| UNK-021 | Whether SQLite/RocksDB synchronization paths satisfy D2 on any selected platform and whether observable engine behavior can ever satisfy strict D3 rather than diagnostic group commit. |
| UNK-022 | Physical serialization, timestamp representation, identity/digest algorithms, artifact retention mechanism, and automated validation for benchmark records. Logical schemas and provenance/freeze invariants are defined. |

The [EXP-0001 execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) maps these unknowns to stable blocker IDs, dependencies, required outputs, and gates. The [R1 requirements record](experiments/EXP-0001/R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolves the policy portions of UNK-012 and UNK-015 through BLK-002/013, while UNK-001 mechanisms and BLK-001/003 remain open. R1 constrained UNK-016 and UNK-017; [R3](experiments/EXP-0001/R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) resolves them and UNK-002/003/009/010 for EXP-0001 while leaving physical realization open. [R2](experiments/EXP-0001/R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) constrains UNK-018/019 and the digest portion of UNK-022 without selecting mechanisms. R3 further constrains BLK-007. [R4](experiments/EXP-0001/R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md) closes BLK-014 for conditional R4 planning and permits R5 documentation design. BLK-015 remains open for dependent D2/D3 claims and execution pending final placement, exact PLP/controller protection, an API profile, and empirical survival evidence.
