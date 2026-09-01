# Project Status

**Project:** Rusty Data OS
**Status:** Phase 1 planning/readiness — R23 closes closed-stream-scope governance; reference-context implementation remains separately unauthorized; live capture remains blocked and execution unauthorized
**North star:** Represent once. Materialize many. Optimize always.
**Verified R18 authority base:** `79cbd64a436b104835a4279c07ba2777fb06cddb` (PR #68 merge); final corrective A2 head `fcaf7f14c94df5a6cda1aeeb283b6726551d1844`

## 1. Current facts

The repository contains the reviewed Slice A implementation: one experiment-local Rust workspace/package, authority-derived physical-record fixtures, deterministic record codec and bounded multi-record artifact scanner, executable V1–V10 dispositions, deterministic tests, and a least-privilege CI workflow. [R10](experiments/EXP-0001/R10-SLICE-A-CLOSURE-AND-SLICE-B-AUTHORIZATION.md) records that Slice A passed its continuation gate as implementation/correctness-validation evidence only. No engine, persistence, benchmark implementation/execution, benchmark evidence, or production Cargo baseline exists. R3 selects typed UUIDv4 live identities, signed 64-bit Unix-epoch-nanosecond canonical times (including durability time), OS-realtime clock classes for engine-assigned canonical times, and run-relative monotonic nanoseconds for lifecycle measurements. R4 records a 1 ns implementation-resolution observation for the relevant clocks while distinguishing resolution from accuracy. R5 selects B1 framing, CRC-32C, immutable final/commit records, and exact append/finalization mechanics as documentation design. R6 selects exact SQLite/RocksDB sources, build/API profiles, mappings, effective-setting obligations, and D-mode classifications as documentation design. R12 freezes experiment-local deterministic payload/identity/reference/logical-time generation and documentation vectors, and the external-dependency-free generator/manifest conformance implementation exists with reviewed workspace path dependencies. Generated workload and benchmark execution remain absent. Concrete normalized-request equality, final event encoding, exact target clock API selection and retained API-specific evidence, clock synchronization/accuracy evidence, concurrency model, checkpoint format, generalized transaction model, query language, and distributed design remain unselected; benchmark implementation and physical execution evidence for the selected designs remain absent.

The merged Slice B implementation is a bounded, single-owner in-memory vector mechanism with process-local sequence and correctness accounting. R11 closes it as implementation/correctness-validation evidence only. It remains D0-only, provisional, noncanonical, and unexecuted as a workload or benchmark. The conceptual architecture is a research direction, not a benchmark-validated design.

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

[EXP-0001 — Immutable Event Ingestion](experiments/EXP-0001-immutable-event-ingestion.md) remains proposed and planned. Its [execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) gates every implementation slice. R1–R16 are complete documentation/governance inputs. The R8 record freezes a 40-cell candidate-primary matrix, statistical analysis design, and prospective owner-approved practical thresholds. BLK-023/UNK-008 are resolved for that threshold decision and R8 is complete as documentation design. The [R9 authority](experiments/EXP-0001/R9-WORKSPACE-HARNESS-CI-AND-SLICE-A-AUTHORIZATION.md) now freezes the Slice A-only workspace, harness boundary, exact Rust 1.89.0 toolchain, external-dependency-free build with reviewed workspace path dependencies, tests, and CI plan. It resolves BLK-020/026 for Slice A and prospectively resolves BLK-027 when R9 is reviewed and merged. That merge authorized only Slice A implementation. The reviewed Slice A implementation and its exact-head CI are bounded correctness-validation evidence. [R11](experiments/EXP-0001/R11-SLICE-B-CLOSURE-AND-NEXT-GATE.md) closes the merged minimum, process-local, noncanonical, D0-only Slice B implementation and prospectively authorizes only a documentation freeze for BLK-006/007. BLK-006/007 are resolved as documentation design only by R12. [R14](experiments/EXP-0001/R14-CANONICAL-WORKLOAD-STREAM-DIGEST.md) resolves BLK-008 as documentation design only. [R16](experiments/EXP-0001/R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md) resolves BLK-009/UNK-019 only as documentation design; implementation, workload observations, and descriptive and confirmatory execution remain unauthorized. Kernel-crash, physical reset/power-loss, storage-error apparatus, BLK-015, later-slice harness/toolchains, effective validation, empirical equivalence, evidence, and execution remain open. Adapters, capture, fault execution, benchmarks, and durability claims remain unauthorized.

Phase 0's documented exit criteria are satisfied by the completed EXP-0000 framework: the benchmark plan and correctness criteria exist, baseline families are identified, and environments can be recorded consistently. This records entry into **Phase 1 planning/readiness**, not experimental execution. Slices A, B/B0, A2, and the bounded R18-authorized Slice C/B1 subset supply implementation/correctness-validation evidence only. No workload execution, benchmark, D2/D3 durability, or performance evidence exists.

The R18-authorized Slice C/B1 crate now supplies bounded correctness-validation evidence for
complete-frame validation, process-local raw D1 write submission, poison-on-terminal-write failure,
and deterministic read-only physical reopen/replay. This observation does not establish stable
storage, namespace durability, acknowledged-byte recovery, canonical recovery, D2/D3, workload or
benchmark execution, faults, adapters, production readiness, performance, or authority for a later
tranche.

## 5. Decision policy

Foundational empirical claims follow:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core code
```

Governance and approved research constraints may be decided before empirical validation when their evidence classification is explicit. They must not be presented as proven performance claims.

## 6. Continuity and navigation

Read [AGENTS.md](../AGENTS.md) and [CHATGPT_WORKFLOW.md](../CHATGPT_WORKFLOW.md) first, then the authorities in the order they prescribe. Supporting registries are the [glossary](GLOSSARY.md), [assumptions and unknowns](ASSUMPTIONS-AND-UNKNOWNS.md), [research questions](RESEARCH-QUESTIONS.md), [requirements](REQUIREMENTS.md), and [traceability registry](TRACEABILITY.md).

The latest `main` branch is authoritative over conversation memory. The checkpoint above records the verified repository starting point for this continuity increment; it is not experiment evidence.

## R17 / Slice A2 update

Issue #63 prospectively authorized the single external-dependency-free Slice A2 workload-conformance crate with reviewed workspace path dependencies.
R18 closes that bounded implementation/correctness-validation tranche after corrected exact-head review and CI for reviewed R12/R14/R16
vectors only. BLK-006–009 are executable only in this conformance subset; BLK-020/026/027 extend
only to its existing workspace/CI boundary. Under R17 alone, BLK-015, Slice C/B1, execution,
benchmarks, capture, persistence, faults, durability, adapters, production code, and architecture
promotion remained open or unauthorized; R18 supplies the separate bounded next authorization.

## R18 closure and next readiness boundary

[R18](experiments/EXP-0001/R18-A2-CLOSURE-AND-SLICE-C-B1-READINESS.md) preserves PR #64 historical implementation head
`d2ee72aa4ff047d4cfcaa1df82d83f13566568f2` and merge
`9b5d89a36ed71d38420e9ae19f59d441a9d927aa`, and closes A2 with corrective PR #68 reviewed head
`fcaf7f14c94df5a6cda1aeeb283b6726551d1844` and merge
`79cbd64a436b104835a4279c07ba2777fb06cddb`; both corrective exact-head workflows passed. M01 remains the canonical positively valid R7-backed vector with its real 1,274-byte stream
artifact-manifest fixture and independent 1,152-byte workload-manifest artifact fixture. R18 historically authorized the now-closed
third, external-dependency-free experiment workspace member with exactly one reviewed workspace path dependency on `exp1-record-format` for raw D1 append and deterministic physical
reopen/replay correctness. BLK-015 is not needed because no `fsync`, D2/D3, survival, canonical
recovery, execution, benchmark, fault, adapter, production, or performance claim is permitted.


## R19 closure and blocked descriptive D1 readiness

[R19](experiments/EXP-0001/R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md)
binds Slice C/B1 closure to PR #71 reviewed head `21143b716de006dd5ec639c0b76a1b031d359fc1`
and merge `e9c292cd614d97b2bf299fd8d2637de76dcdca54`, plus PR #74 reviewed head
`5c448695f4e460cab57eaadd7f7a83bfce1559ab` and merge
`ef29804347faa812502f855e5cc3ffee6f4901c2`; both PR #74 exact-head workflows passed. R19 found
the candidate generated-workload descriptive D1 harness blocked because authorities did not then
freeze the M01-semantic-operation-to-EXP1-B1-RF1-Record mapping or the exact implementation of
required direct Linux capture interfaces. R20 now resolves the former only. The existing
external-dependency-free workspace contains reviewed workspace path dependencies and forbids unsafe
code. A later decision must still select the direct-interface implementation, dependency/unsafe
policy, privilege/loss behavior, and unavailable-field policy. Caller/authority identity assignment
is selected and requires a complete validated identity manifest. No fourth crate, workload
materialization, capture, or execution is authorized.

## R20 semantic-to-physical mapping decision

[R20](experiments/EXP-0001/R20-SEMANTIC-OPERATION-TO-PHYSICAL-RECORD-MAPPING.md) freezes one validated SOP1 operation to exactly one structural type-3 `EXP1-B1-RF1` provisional record. The complete SOP1 is the stable core; its event ID is duplicated in the type-3 body, and later ingestion supplies distinct nonzero assigned sequence and consecutive physical ordinal values. This resolves R19's mapping blocker as documentation design and prospectively authorizes only a pure public mapper module in `exp1-raw-append-replay`, with direct path dependencies on `exp1-record-format` and `exp1-workload-conformance`; only that crate's manifest and the matching lock entry may change, and append integration is excluded. The independent live-Linux-capture freeze remains open; no harness, execution, D2/D3, `fsync`, canonicality, durability, or recovery claim follows.

The authorized pure mapper and deterministic tests provide bounded implementation/correctness-validation evidence only for locally decidable mapping rules; the complete R20 correctness gate is not closed. Duplicate reference bytes are rejected by SOP1 validation, but future, self, and cross-stream membership require prior-event and stream context absent from the frozen mapper state. R21 freezes the locally decidable catalog/context subset, R22 resolves cross-segment eligibility, and R23 freezes the complete closed-scope proof. Separate authorization and implementation are still required for full mapper closure. They do not integrate with append/reopen, materialize or execute a workload, or establish durability, recovery, performance, or capture evidence. The live-Linux-capture decision remains open and blocks a descriptive D1 harness.

## R21 reference-context decision

[R21](experiments/EXP-0001/R21-REFERENCE-VALIDATION-CONTEXT.md) freezes a catalog built only from complete validated semantic streams and a separate caller-owned accepted-prefix state. Its typed, stream-bound identity entries, bounds, collisions, and partial precedence are frozen. R22 supersedes the cross-segment ambiguity, and R23 now proves a complete closed stream set through a canonical manifest-bound scope descriptor. The governance blocker is closed, but the complete R20 gate remains open and no Rust implementation is authorized. No Rust/Cargo change, append integration, workload execution, capture, durability, recovery, benchmark, or performance evidence is part of R21. The independent live Linux capture blocker still prevents a descriptive D1 harness.

## R22 cross-segment reference decision

[R22](experiments/EXP-0001/R22-CROSS-SEGMENT-REFERENCE-RULE.md) selects strictly segment-local
reference eligibility: R12 `[0,i)` is the same-stream, same-segment prefix, and total WS1 position
remains only byte/accepted-prefix order. A known same-stream target in the other segment fails with
the new experiment-local `E-REFERENCE-CROSS-SEGMENT`, after wrong-kind, wrong-fact, and cross-stream
classification and before same-segment future/missing handling. R22 supersedes only R12 section
5.3's ambiguous cross-segment interpretation and R21's matching unresolved language; existing
R12/R14 vectors and bytes are unchanged. R23 now closes the separate complete closed stream-scope governance question. R21 implementation and full R20 closure still require separate authorization, implementation, review, and CI; live Linux capture, a descriptive D1 harness, and execution remain unauthorized.

## R23 closed stream-scope decision

[R23](experiments/EXP-0001/R23-CLOSED-STREAM-SCOPE-PROOF.md) freezes a canonical JCS closed-scope
descriptor for one reviewed cell, canonical namespace-ordered membership, exact R16 manifest and
R14 stream/artifact bindings, and a domain-separated scope digest. Exact equality between the
proven member set and supplied validated WS1 streams fails closed on omissions, additions,
substitutions, duplicates, foreign membership, or digest disagreement. This fully closes the
reference-context governance blocker and makes absence classifiable as `E-REFERENCE-MISSING` only
after successful complete-scope construction. It authorizes no implementation; the full R20 gate
still requires a separate increment, review, and CI. All execution and capture exclusions remain.
