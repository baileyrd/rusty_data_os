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
| UNK-008 | Resolved for the R8 threshold decision by prospective owner approval of `EXP-0001-R8/thresholds-v1` on 2026-08-28. The values are explicit product/engineering judgments, not empirical facts; prior evidence is never retroactively classified, and separate admissibility/execution gates remain. |
| UNK-009 | Resolved by R3: the event constructor assigns after validation/system-time capture and before durable sequence reservation. |
| UNK-010 | Resolved by R3: capture once after semantic validation and immediately before event construction, then reserve sequence after construction. |
| UNK-011 | Observation-side metadata retention, observer/context identity and multiplicity rules, and criteria for appending an observation as a separate canonical event. |
| UNK-012 | Resolved for EXP-0001 B1 by R5: structural-only provisional use and the versioned CRC-32C error-detecting profile, coverage, encoding, limits, and vectors are frozen. |
| UNK-013 | Validation and locality rules for causal-event and correction/retraction references. |
| UNK-014 | R4 selects bare-metal Bosgame M5/Fedora 44 and four intended paths. The owner accepts externally reviewed but unretained host, 1 ns clock-resolution, nearest-parent XFS/LVM/NVMe, write-back, FUA, and volatile-write-cache observations for conditional planning. BLK-014 is closed for R4 planning; final placement, complete execution provenance, exact PLP/controller protection, and empirical survival remain unresolved for execution and dependent claims. |
| UNK-015 | Concrete crash/fault-injection mechanisms and implementation of physical validity detection for partial, torn, truncated, or uncertain outcomes. EXP-0000 defines the semantic procedure and R1 defines minimum deterministic classifications and fail-closed scan/recovery policy; mechanisms remain open. |
| UNK-016 | Resolved by R3: durable binding, explicit reconciliation, exact-candidate retry, conflict handling, and commit-before-ack uncertainty rules. |
| UNK-017 | Resolved by R3: durable reservation, permanent no-reuse gaps, strict monotonic replay, reporting, and fail-closed conflicts. |
| UNK-018 | Platform-independent generator, permutation, identity, and stream-digest algorithms and reference vectors. R7 selects SHA-256 and the workload-stream domain, but BLK-006/007 generator inputs and their stable vectors remain unresolved. |
| UNK-019 | Physical workload-manifest serialization remains open under BLK-009. R7 freezes artifact layout/reference/retention and permits reuse of its JCS profile only after the workload-manifest field contract is separately reviewed. |
| UNK-020 | Narrowed by R5/R6 and R9: all B0–B3 documentation profiles/mappings are frozen, and the Slice A Rust 1.89.0 toolchain/build is frozen. The corrective Slice A/A1 scanner is under review and becomes implemented only after its exact head is reviewed and merged; later-slice and benchmark-series toolchains, effective-configuration validation, and execution evidence remain unresolved. |
| UNK-021 | R5 maps B1 D2/controlled D3, and R6 classifies SQLite/RocksDB D2 as conditional and strict D3 as unsupported. Empirical survival/equivalence and BLK-015 platform protection remain unresolved. |
| UNK-022 | R7 resolves benchmark-record serialization/digests/artifact provenance/instrumentation as documentation design; R9 authorizes only Slice A physical-record fixtures and validators after merge. Executable benchmark-record schemas/validators, effective validation, capture, and evidence remain open. |

The [EXP-0001 execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) maps unknowns to gates. R5 resolves UNK-001 and the B1 physical/integrity portion of UNK-012. R6 narrows UNK-020/021. [R7](experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) narrows UNK-015/018/019/022 without empirical proof. R7 and R8 are complete documentation inputs; R9 is complete as a prospective Slice A-only authorization. BLK-015, corrected A1 exact-head review/merge, later slices, effective validation, execution, and evidence remain open.
