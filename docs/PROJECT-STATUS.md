# Project Status

**Project:** Rusty Data OS
**Status:** Phase 1 planning/readiness — Slice A correctness gate passed; Slice B implementation candidate under review
**North star:** Represent once. Materialize many. Optimize always.
**Verified starting `main` checkpoint:** `68fe3fa` (PR #46 / corrective Slice A/A1)

## 1. Current facts

The repository contains the reviewed Slice A implementation: one experiment-local Rust workspace/package, authority-derived physical-record fixtures, deterministic record codec and bounded multi-record artifact scanner, executable V1–V10 dispositions, deterministic tests, and a least-privilege CI workflow. [R10](experiments/EXP-0001/R10-SLICE-A-CLOSURE-AND-SLICE-B-AUTHORIZATION.md) records that Slice A passed its continuation gate as implementation/correctness-validation evidence only. No engine, persistence, benchmark implementation/execution, benchmark evidence, or production Cargo baseline exists. R3 selects typed UUIDv4 live identities, signed 64-bit Unix-epoch-nanosecond canonical times (including durability time), OS-realtime clock classes for engine-assigned canonical times, and run-relative monotonic nanoseconds for lifecycle measurements. R4 records a 1 ns implementation-resolution observation for the relevant clocks while distinguishing resolution from accuracy. R5 selects B1 framing, CRC-32C, immutable final/commit records, and exact append/finalization mechanics as documentation design. R6 selects exact SQLite/RocksDB sources, build/API profiles, mappings, effective-setting obligations, and D-mode classifications as documentation design. Deterministic identity generation/serialization/vectors, concrete normalized-request equality, final event encoding, exact target clock API selection and retained API-specific evidence, clock synchronization/accuracy evidence, concurrency model, checkpoint format, generalized transaction model, query language, and distributed design remain unselected; benchmark implementation and physical execution evidence for the selected designs remain absent.

The current change adds a Slice B implementation candidate for review: a bounded, single-owner in-memory vector mechanism with process-local sequence and correctness accounting. It remains D0-only, provisional, noncanonical, and unexecuted as a workload or benchmark. The conceptual architecture is a research direction, not a benchmark-validated design.

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

The baseline checkpoint selects B0 in-memory (D0 only), B1 raw OS append (primary D1/D2/controlled D3), SQLite WAL, and RocksDB WAL. R5 freezes B0/B1 design profiles. The [R6 execution-profile authority](experiments/EXP-0001/R6-SQLITE-ROCKSDB-EXECUTION-PROFILES.md) now freezes SQLite 3.53.4 and RocksDB 11.8.1 source/build/API profiles, mappings, effective-setting evidence, equivalence classifications, and exclusions without installing, building, implementing, or claiming evidence. SQLite/RocksDB D2 remains conditional; strict D3 is unsupported and atomic multi-event transactions or opaque group commit remain diagnostic. The prospective owner-approved `EXP-0001-R8/thresholds-v1` values resolve UNK-008 for the R8 threshold decision; confirmatory execution remains blocked until the readiness plan freezes every remaining input and passes its separate gates.

[EXP-0001 — Immutable Event Ingestion](experiments/EXP-0001-immutable-event-ingestion.md) remains proposed and planned. Its [execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) gates every implementation slice. R1–R10 are complete documentation/governance inputs. The R8 record freezes a 40-cell candidate-primary matrix, statistical analysis design, and prospective owner-approved practical thresholds. BLK-023/UNK-008 are resolved for that threshold decision and R8 is complete as documentation design. The [R9 authority](experiments/EXP-0001/R9-WORKSPACE-HARNESS-CI-AND-SLICE-A-AUTHORIZATION.md) now freezes the Slice A-only workspace, harness boundary, exact Rust 1.89.0 toolchain, dependency-free build, tests, and CI plan. It resolves BLK-020/026 for Slice A and prospectively resolves BLK-027 when R9 is reviewed and merged. That merge authorized only Slice A implementation. The reviewed Slice A implementation and its exact-head CI are bounded correctness-validation evidence. R10 prospectively authorizes only a minimum, process-local, noncanonical, D0-only Slice B implementation after R10 merges. BLK-006/007 remain open and block workload observations; descriptive and confirmatory execution remain unauthorized. Kernel-crash, physical reset/power-loss, storage-error apparatus, BLK-015, later-slice harness/toolchains, effective validation, empirical equivalence, evidence, and execution remain open. Adapters, capture, fault execution, benchmarks, and durability claims remain unauthorized.

Phase 0's documented exit criteria are satisfied by the completed EXP-0000 framework: the benchmark plan and correctness criteria exist, baseline families are identified, and environments can be recorded consistently. This records entry into **Phase 1 planning/readiness**, not experimental execution. Slice A supplies bounded implementation/correctness-validation evidence only. R10 prospectively authorizes only the Slice B mechanism after merge; no execution, benchmark, persistence, durability, or performance evidence exists.

## 5. Decision policy

Foundational empirical claims follow:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core code
```

Governance and approved research constraints may be decided before empirical validation when their evidence classification is explicit. They must not be presented as proven performance claims.

## 6. Continuity and navigation

Read [AGENTS.md](../AGENTS.md) and [CHATGPT_WORKFLOW.md](../CHATGPT_WORKFLOW.md) first, then the authorities in the order they prescribe. Supporting registries are the [glossary](GLOSSARY.md), [assumptions and unknowns](ASSUMPTIONS-AND-UNKNOWNS.md), [research questions](RESEARCH-QUESTIONS.md), [requirements](REQUIREMENTS.md), and [traceability registry](TRACEABILITY.md).

The latest `main` branch is authoritative over conversation memory. The checkpoint above records the verified repository starting point for this continuity increment; it is not experiment evidence.
