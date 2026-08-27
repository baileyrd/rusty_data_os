# Project Status

**Project:** Rusty Data OS
**Status:** Phase 1 planning/readiness — implementation not authorized
**North star:** Represent once. Materialize many. Optimize always.
**Verified starting `main` checkpoint:** `3c08edc6c862350d367f7d1b7b4bffb97f53d4fe`

## 1. Current facts

This repository is documentation-only. An active documentation-validation CI workflow checks Markdown links and changed-line whitespace, but no engine implementation, Cargo baseline, Rust engine build/test/benchmark or implementation-validation CI baseline, or benchmark evidence exists. R3 selects typed UUIDv4 live identities, signed 64-bit Unix-epoch-nanosecond canonical times (including durability time), OS-realtime clock classes for engine-assigned canonical times, and run-relative monotonic nanoseconds for lifecycle measurements. Deterministic identity generation/serialization/vectors, concrete normalized-request equality, event encoding, target platform clock APIs and verified resolution, physical durability/finalization mechanics, concurrency model, checkpoint format, transaction model, query language, and distributed design remain unselected.

The conceptual architecture is a research direction, not a benchmark-validated design.

## 2. Approved foundation

The primary unproven research claim is that a single canonical information history can support multiple independently optimized representations with acceptable performance and complexity.

The approved semantic constraints are recorded in [ADR-0002](adr/ADR-0002-foundational-canonical-history-constraints.md) and [REQ-001 through REQ-014](REQUIREMENTS.md). In summary:

- canonical events are accepted facts, while commands are requested intent and rejected commands remain separate evidence;
- canonical history alone is authoritative; memory, checkpoints, indexes, and materializations are derived;
- local monotonic sequence provides deterministic replay without committing future distributed ordering;
- temporal, permanent identity, provenance, correction/retraction, schema-version, payload-boundary, compaction, checkpoint, and durability semantics are explicit;
- EXP-0001 is restricted to single-event commit and opaque payloads with schema identity/version.

These are constraints on research and correctness, not evidence that the architecture performs acceptably.

## 3. Active hypothesis

[HYP-0001](hypotheses/HYP-0001-event-log-as-canonical-state.md) asks whether one canonical information history can support multiple independently optimized representations with acceptable performance and complexity. It is active and unproven. No implementation or experimental result supports or refutes it yet.

## 4. Active and next incomplete increments

[EXP-0000 — Measurement and Semantics Readiness](experiments/EXP-0000-measurement-and-semantics-readiness.md), also called Experiment 0, is complete as a readiness-documentation experiment. Its [minimal single-event semantic envelope](experiments/EXP-0000/SEMANTIC-EVENT-ENVELOPE.md), [reproducible workload contract](experiments/EXP-0000/WORKLOADS.md), [EXP-0001 baseline contract](benchmarks/BASELINES.md), [acknowledgement, visibility, fault, and durability contract](experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), and [crash/recovery correctness contract](experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) are complete. The [environment](benchmarks/ENVIRONMENT-TEMPLATE.md) and [raw-result](benchmarks/RAW-RESULT-TEMPLATE.md) record contracts are also complete. The [interpretation and decision contract](benchmarks/INTERPRETATION-CRITERIA.md) completes its outputs by freezing admissibility, analysis, uncertainty, threshold-registry, outcome, trade-space, and ingestion-complexity rules. These are documentation and measurement-readiness outputs, not implementation or evidence.

The baseline checkpoint selects B0 in-memory (D0 only), B1 raw OS append (primary D1/D2/controlled D3), SQLite WAL, and RocksDB WAL. It freezes semantic profiles, equivalence classifications, adapter fairness, series identity, correctness gates, and exclusion/replacement policy without selecting binaries, implementations, schemas, encodings, platform sync mechanisms, or claiming evidence. SQLite/RocksDB D2 remains conditional; atomic multi-event transactions and opaque group commit are not strict D3 equivalence. Exact threshold values remain open under UNK-008, and confirmatory execution is blocked until a reviewed execution plan freezes them and all remaining physical inputs. EXP-0000 completion neither configures a product nor authorizes implementation or execution.

[EXP-0001 — Immutable Event Ingestion](experiments/EXP-0001-immutable-event-ingestion.md) remains proposed and planned. Its repository-recorded planning bridge is the [execution-readiness and staged-implementation plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md), which inventories open blockers, orders readiness increments, and gates every implementation slice. Its [R1 physical-record, integrity, and recovery requirements](experiments/EXP-0001/R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md) resolve BLK-002 and BLK-013 as requirements-only constraints while BLK-001 and BLK-003 remain open. The [R2 deterministic-workload requirements and reference-vector plan](experiments/EXP-0001/R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) is complete and constrains BLK-006 through BLK-009 without resolving their algorithm, serialization, rationale, or stable-vector requirements. The [R3 identity, time, sequencing-gap, retry, and uncertain-outcome lifecycle contract](experiments/EXP-0001/R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) resolves BLK-004/005/011/012 and further constrains open BLK-007 without implementation or evidence. R4 is complete for conditional planning at the owner-approved [target/platform evidence boundary](experiments/EXP-0001/R4-FEDORA-44-BOSGAME-M5-TARGET-AND-PLATFORM-DURABILITY-CONTRACT.md). The reviewed host, clock-resolution, nearest-parent XFS/LVM/NVMe, write-back, FUA, and volatile-write-cache observations are accepted for planning despite non-retained provenance. BLK-014 is closed for this R4 purpose, while BLK-015 remains open for dependent D2/D3 claims and execution because final placement, exact PLP/controller protection, the API profile, and empirical fault survival remain unverified. The next authorized increment is R5 B0/B1 physical-profile and adapter-contract **documentation design only**; implementation, Cargo bootstrap, fixtures, validators, adapters, descriptive or confirmatory execution, benchmarks, and durability claims remain unauthorized.

Phase 0's documented exit criteria are satisfied by the completed EXP-0000 framework: the benchmark plan and correctness criteria exist, baseline families are identified, and environments can be recorded consistently. This records entry into **Phase 1 planning/readiness**, not Phase 1 implementation. No Phase 1 evidence exists; Cargo or Rust work remains prohibited until the plan's explicit first-implementation authorization gate passes.

## 5. Decision policy

Foundational empirical claims follow:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core code
```

Governance and approved research constraints may be decided before empirical validation when their evidence classification is explicit. They must not be presented as proven performance claims.

## 6. Continuity and navigation

Read [AGENTS.md](../AGENTS.md) and [CHATGPT_WORKFLOW.md](../CHATGPT_WORKFLOW.md) first, then the authorities in the order they prescribe. Supporting registries are the [glossary](GLOSSARY.md), [assumptions and unknowns](ASSUMPTIONS-AND-UNKNOWNS.md), [research questions](RESEARCH-QUESTIONS.md), [requirements](REQUIREMENTS.md), and [traceability registry](TRACEABILITY.md).

The latest `main` branch is authoritative over conversation memory. The checkpoint above records the verified repository starting point for this continuity increment; it is not experiment evidence.
