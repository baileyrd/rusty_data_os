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
| UNK-013 | Reopened/corrected by R25: R21–R23 freeze catalog, segment-local eligibility, and closed-scope proof, but closed unmerged PR #91 falsified the assumption that unchanged R12/R16 v1 can bootstrap a causal stream. R25 freezes prospective v2 semantics; R26 freezes the v2 contract and its PR #95 implementation is now closed by R27 as bounded conformance/correctness evidence. R27 authorizes the still-unimplemented pure v2 reference-context mapper; the complete correctness gate remains open pending that implementation. |
| UNK-014 | R4 selects bare-metal Bosgame M5/Fedora 44 and four intended paths. The owner accepts externally reviewed but unretained host, 1 ns clock-resolution, nearest-parent XFS/LVM/NVMe, write-back, FUA, and volatile-write-cache observations for conditional planning. BLK-014 is closed for R4 planning; final placement, complete execution provenance, exact PLP/controller protection, and empirical survival remain unresolved for execution and dependent claims. |
| UNK-015 | Concrete crash/fault-injection mechanisms and implementation of physical validity detection for partial, torn, truncated, or uncertain outcomes. EXP-0000 defines the semantic procedure and R1 defines minimum deterministic classifications and fail-closed scan/recovery policy; mechanisms remain open. |
| UNK-016 | Resolved by R3: durable binding, explicit reconciliation, exact-candidate retry, conflict handling, and commit-before-ack uncertainty rules. |
| UNK-017 | Resolved by R3: durable reservation, permanent no-reuse gaps, strict monotonic replay, reporting, and fail-closed conflicts. |
| UNK-018 | Platform-independent generator, permutation, identity, and stream-digest algorithms and reference vectors. R7 selects SHA-256 and the workload-stream domain; R12 freezes BLK-006/007 generator inputs and documentation vectors; R14 freezes canonical operation/stream bytes and digest vectors. Documentation design is resolved; implementation and executable conformance remain unresolved. |
| UNK-019 | Resolved as documentation design by R16: `EXP-0001-WORKLOAD-MANIFEST-JCS-v1` freezes the closed JCS schema, bindings, validation, immutability, and vectors. Dependency-free manifest construction and conformance validation exist; execution and publication capture remain absent. |
| UNK-020 | Narrowed by R5/R6/R9/R10: all B0–B3 documentation profiles/mappings are frozen; the Rust 1.89.0 workspace produced reviewed Slice A and Slice B correctness-validation evidence. R11 authorizes no executable reuse or expansion. Later-slice and benchmark-series toolchains, effective-configuration validation, and execution evidence remain unresolved. |
| UNK-021 | R5 maps B1 D2/controlled D3, and R6 classifies SQLite/RocksDB D2 as conditional and strict D3 as unsupported. Empirical survival/equivalence and BLK-015 platform protection remain unresolved. |
| UNK-022 | Narrowed by R29–R33: the semantic mapping, clocks/resource/file/procfs ABI, four-counter perf ABI, minimum first descriptive B1/D1 sources, non-live orchestration, and internal live-wrapper adapter are frozen and deterministically implemented through the R32 implementation closed by R33. R33 freezes, but does not implement or invoke, a preflight-subset boundary whose immutable artifact excludes retention facts, whose invalid request disposition has no artifact, whose returned call disposition reports mechanically normalized sink-I/O failure, whose artifact failures have a closed ordered schema, and whose ordinary file drop proves only ownership release. Its strongest classification is `preflight_subset_ready`; Fedora-release/effective-target validation remains unresolved. Live interface use, retained host observations, record schemas/producers/validators, calibration/overhead, capture, tracefs for confirmation or attribution/loss claims, execution, and evidence remain open. |

The [EXP-0001 execution-readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) maps unknowns to gates. R5 resolves UNK-001 and the B1 physical/integrity portion of UNK-012. R6 narrows UNK-020/021. [R7](experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) narrows UNK-015/018/019/022 without empirical proof. R7–R22 are complete documentation/governance inputs. Slices A, B/B0, A2, and bounded raw D1 append/replay are closed as correctness-validation evidence; R19 authorizes no harness or execution. Generated workloads, BLK-015, later slices, effective validation, execution, and benchmark evidence remain open.

### R17 / A2 disposition

The R12/R14/R16 contract subset now has external-dependency-free implementation/correctness evidence, with reviewed workspace path dependencies, under
issue #63. This does not resolve execution/capture unknowns, BLK-015, or any Slice C/B1,
persistence, fault, durability, benchmark, adapter, or production uncertainty.

### R18 disposition

A2 closure preserves PR #64 historical head and merge and is finalized by corrective PR #68 reviewed head and merge. M01 is the canonical positively valid R7-backed vector with matching caller-supplied fixture bytes and metadata. R18 narrows UNK-020 by authorizing one raw D1 append/reopen/replay correctness member with no external dependencies and exactly one reviewed workspace path dependency on `exp1-record-format`. It does not resolve UNK-014/015/021/022 or
BLK-015; platform survival, D2/D3, physical faults, execution, capture, and benchmarks remain open.


### R19 disposition

R19 closes the merged R18 implementation only as bounded correctness evidence and adds no
empirical fact. It narrows UNK-022 to the exact semantic-to-physical mapping and direct Linux capture implementation
decisions recorded in R19. It does not resolve UNK-014/015/021, BLK-015, or any execution unknown.


### R20 disposition

R20 resolves only UNK-022's semantic-to-physical mapping ambiguity as documentation design and prospectively authorizes a pure public mapper module in `exp1-raw-append-replay`, with direct path dependencies on the unchanged authority crates and no append integration. The direct Linux capture decision and every execution/evidence unknown remain open.

That pure mapper now has bounded implementation/correctness-validation evidence only for locally decidable rules. The frozen state has no prior-event membership or stream identity, so future, self, and cross-stream reference rejection cannot be established; duplicate bytes alone are rejected by the SOP1 validator. R21 freezes only the locally decidable catalog/context subset; R22 resolves cross-segment eligibility, while closed stream-scope proof still requires governance before implementation and reviewed correctness closure. This does not resolve the direct Linux capture decision, authorize a descriptive harness, or supply execution, durability, recovery, or performance evidence.

### R21 disposition

R21–R23 resolve the reference-context governance blocker and UNK-013 as documentation design: R21 freezes the catalog/context split, R22 resolves cross-segment eligibility, and R23 freezes complete closed-scope proof. R24 now prospectively authorizes exactly one bounded pure catalog/context implementation in the existing mapper; it does not implement or close that gate. Reconstruction, capture, harness construction, execution, durability, recovery, performance, and every R21 exclusion remain open or unauthorized.

### R22 disposition

R22 resolves the cross-segment portion of UNK-013 as documentation design with strictly
segment-local eligibility and `E-REFERENCE-CROSS-SEGMENT`. R23 subsequently resolves the complete closed stream classification scope as documentation design. R21 implementation, complete R20 closure, live Linux capture, the descriptive D1 harness, and execution remain blocked or unauthorized.

### R23 disposition

R23 resolves the remaining UNK-013 governance question with an immutable canonical descriptor,
manifest-bound canonical stream membership, and a domain-separated scope digest. Only a catalog
built after exact supplied-set equality may classify an absent identity as `E-REFERENCE-MISSING`;
an unproven scope fails construction. Implementation, full R20 closure, capture, and execution
remain unauthorized.

### R24 disposition

R24 converts the closed R21–R23 governance design into one prospective, bounded pure-implementation
authorization in `exp1-raw-append-replay`. It freezes no-manifest/no-dependency and unchanged-authority
boundaries, transactional APIs and errors, exact bounds and precedence, required synthetic tests,
and exact-head CI closure. Implementation and correctness evidence remain pending; capture, harness,
execution, durability, recovery, and performance remain open or unauthorized.


### R25 disposition

R25 records the R24 bootstrap-to-reference premise as a failed governance assumption and preserves
closed PR #91 as unmerged negative evidence. It does not change any v1 bytes. Prospective v2 gives
each segment ordinal 0 exactly zero causal targets and requires positive prior same-segment targets
after bootstrap, represented by explicit per-segment bootstrap/subsequent cardinalities. R24 is
superseded for implementation authorization only. R26 now supplies the v2 conformance/validator-and-vector
authorization; a new bounded reference-context implementation still requires separate authorization; UNK-013 and the
complete R20 gate remain open at those implementation/evidence boundaries.


### R26 disposition

R26 resolves the remaining v2 byte-level conformance-design portion of UNK-013 and UNK-019 by
freezing the complete profile tuple, binary/JCS encodings, digest domains, cardinality-policy ledger,
bindings, dispositions, precedence, and literal-vector requirements. Its prospective implementation
authorization is limited to the existing workload-conformance crate and supplies no implementation
evidence. The contextual R21–R23 implementation, complete R20 closure, workload generation or
execution, and all capture/durability/performance unknowns remain open.

## R27 update

R26 conformance implementation is no longer an unknown: PR #95 reviewed head
`35f9a0f245ac488828df4f639263edb3fb50be86`, merged as
`f4ed0c310fa46c6de209ea0f776c4749e31cdd34` with exact-head successful CI, supplies bounded v2
conformance/correctness evidence. The assumption that a closed scope may silently combine v1 and
v2 is rejected: R27 requires a distinct exact v2 scope profile and homogeneous membership while
preserving v1 unchanged. The remaining R20 unknown is whether the authorized pure v2 contextual
mapper satisfies the frozen construction, precedence, bounds, and transactional tests; no evidence
exists until its reviewed exact head passes CI. The live Linux capture/API decision remains open
and independently blocks a descriptive D1 harness.


### R31 disposition

R31 closes the R30-authorized perf implementation as bounded deterministic ABI/lifecycle/scaling/cleanup correctness evidence and resolves the minimum-source and orchestration-policy portions of UNK-022 for one first descriptive B1/D1 cell. It permits exact tracefs `not_collected` or evidenced `unsupported` states only for that cell because R8's primary metrics do not require tracefs attribution. It does not resolve live use, effective target validation, R7 record production/validation, calibration or overhead, capture, execution, evidence, or confirmatory tracefs and attribution/loss gates.


### R32 disposition

R32 closes the injected R31 orchestration implementation only as bounded deterministic correctness evidence. It resolves the internal interface design by freezing a borrowed measured-file identity plus `AsRawFd` capability, one dependency-free Linux/x86_64 `LiveCaptureBoundary`, and independently owned per-event perf sessions with retained aggregate compatibility. UNK-022 remains open for implementation closure, live use, target/effective validation, record production, calibration/overhead, capture, execution, publication, and evidence. Tracefs and every attribution/loss claim remain separately blocked.


### R33 disposition

R33 closes the authorized adapter implementation at PR #111 reviewed head `71f58f65772fea2f0f58f5727d42e1405c7f09fb`, merge `05dd7cc0980df2914dff5814ab5f5fba5b8e09e0`, after both exact-head workflows succeeded, only as bounded deterministic correctness evidence. It freezes one operator-invoked, non-CI preflight and retained diagnostic contract and authorizes one later synthetic-tested implementation. It closes no live/effective portion of UNK-022: no construction, borrowed file, successful wrapper result, preflight artifact, or compilation is target validation or R7/performance evidence.
