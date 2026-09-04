# Research Roadmap

This roadmap defines a sequence of questions, not a fixed product plan. Each phase must earn the next with evidence.

## Phase 0 — Foundation and measurement design

**Goal:** establish the research discipline before performance code exists.

Deliverables:

- vision and principles;
- conceptual architecture;
- hypothesis and experiment lifecycle;
- benchmark methodology;
- baseline definitions;
- project continuity process;
- first experiment specification.
- Experiment 0 semantics, workload, baseline, fault-model, environment, raw-result, and interpretation prerequisites.

Exit criteria:

- `EXP-0000` is complete;
- `EXP-0001` has a reproducible benchmark plan and correctness criteria;
- baseline systems/primitives are identified;
- target test environments can be recorded consistently.

Phase 0 exited when EXP-0000 and these documentation criteria were completed. This is a readiness exit, not implementation or experimental evidence.

## Phase 1 — Canonical event ingestion

**Current substate:** planning/readiness. EXP-0001 remains proposed and non-executable. Slices A, B/B0, A2, raw D1 append/replay, and the complete R20 reference-context correctness gate are merged and closed as bounded implementation/correctness-validation evidence; they supply no benchmark, durability, or performance evidence. R29 closes the single R28-authorized integration of the existing literal SOP2, mapper, RF1, D1 append, and physical replay components as bounded deterministic correctness evidence. [R12](experiments/EXP-0001/R12-DETERMINISTIC-GENERATOR-SPECIFICATION-AND-VECTORS.md) resolves BLK-006/007 as documentation design only. R16 resolves BLK-009 as documentation design; R18 subsequently authorizes only bounded raw D1 append/replay correctness. External-dependency-free generator and manifest conformance implementation exists with reviewed workspace path dependencies, but generated workload and benchmark execution remain blocked; descriptive and confirmatory execution, BLK-015, executable harness/capture, empirical equivalence, and durability claims remain blocked.

**Primary question:** what is the cost envelope of immutable event creation, sequencing, append, and different durability boundaries?

Candidate comparisons:

- memory-only append;
- buffered append;
- batched append;
- explicit sync per event;
- group commit;
- memory-mapped append where appropriate;
- alternative record encodings;
- checksummed versus non-checksummed records.

Measure:

- events/second;
- bytes/second;
- p50/p90/p95/p99/p99.9 latency;
- CPU time;
- allocations;
- memory footprint;
- storage bandwidth;
- write amplification where measurable;
- crash/recovery correctness.

Exit criteria should not simply be "fast." The experiment must identify which durability semantics and batching strategies produce useful trade spaces.

## Phase 2 — In-memory materialized state

**Primary question:** can canonical events feed an in-memory execution representation with competitive read/write behavior and acceptable replay cost?

Investigate separately:

- key/value baseline state;
- row-like state;
- column-like active state;
- indexing strategies;
- concurrency/sharding models;
- copy-on-write versus in-place updates;
- memory allocation strategies.

Critical measures:

- point-read latency;
- range/scan behavior;
- update latency;
- mixed read/write workloads;
- memory consumed per logical record;
- replay events/second;
- time-to-ready after restart.

## Phase 3 — Recovery and checkpointing

**Primary question:** how should the engine bound replay time without allowing checkpoints to become hidden canonical ownership?

Investigate:

- periodic snapshots;
- incremental checkpoints;
- checkpoint consistency;
- concurrent checkpointing overhead;
- verification against event history;
- recovery from torn/truncated records;
- checkpoint corruption behavior.

## Phase 4 — Secondary materializations

**Primary question:** can one canonical history efficiently support distinct physical representations without unacceptable write amplification or freshness lag?

Start with one materialization at a time.

Candidate sequence:

1. row-oriented persistent projection;
2. columnar analytic projection;
3. secondary index projection;
4. vector projection;
5. graph projection.

Each representation receives its own experiment and workload-specific baseline.

## Phase 5 — Query and access semantics

Only after execution and materialization performance are understood should the project select external access models.

Research topics:

- direct typed API;
- relational/query algebra;
- SQL compatibility or subset;
- document access;
- graph traversal;
- vector similarity;
- cross-materialization planning;
- optimizer behavior.

The important research question is whether the engine can select or combine representations without exposing physical ownership to applications.

## Phase 6 — Embedded engine hardening

Potential focus:

- stable APIs;
- versioned formats;
- fuzz/property testing;
- fault injection;
- observability;
- backup/restore;
- schema evolution;
- compatibility guarantees.

## Phase 7 — Server adapter

Only after the embedded core is sufficiently characterized:

- network protocol;
- sessions;
- authentication/authorization;
- connection management;
- remote transactions;
- admission control;
- client libraries;
- server-level benchmarks separated from core benchmarks.

## Phase 8 — Distributed research, if justified

Potential later topics:

- replication;
- partitioning;
- consensus;
- deterministic distributed logs;
- cross-node materialization;
- failure-domain durability.

This phase is explicitly non-committed. The project should not assume distribution is necessary until use cases and evidence justify the complexity.

### Slice A2 conformance checkpoint

R18 closes R17's external-dependency-free executable conformance, which contains reviewed workspace path dependencies, after preserving PR #64 as historical implementation authority and completing corrective PR #68 exact-head review and CI, for the frozen R12/R14/R16 contracts as
bounded correctness evidence. It is not workload execution or benchmark, persistence, durability,
or performance evidence. R17 itself did not advance Slice C/B1 or authorize a subsequent increment;
R18 supplies the separate authorization below.

### R18 Slice C/B1 readiness checkpoint

R18 closes A2 as bounded correctness evidence and prospectively authorizes only one experiment-local,
external-dependency-free raw D1 append plus reopen/replay correctness crate with exactly one reviewed workspace path dependency on `exp1-record-format`. The tranche validates R1/R5
framing, integrity, accepted physical prefixes, terminal truncation, and fail-closed scanning; it
never labels D1 bytes canonical. BLK-015, D2/D3, execution, benchmarks, physical faults, adapters,
production promotion, and every later tranche remain gated.


## R19 Phase 1 checkpoint

[R19](experiments/EXP-0001/R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md) closes
the merged raw D1 append/physical replay tranche only as bounded correctness evidence. The smallest
useful generated-workload descriptive D1 harness is not implementation-ready: the exact R7 Linux
semantic-operation-to-physical-record mapping and direct Linux capture implementation were not selected when R19 was decided within the external-dependency-free workspace, which contains reviewed path dependencies and forbids unsafe code. R20 now resolves the mapping as documentation design; the capture-interface/dependency-or-bounded-unsafe decision remains the next gate; caller/authority identity assignment is selected. Phase 1 execution, performance evidence, D2/D3,
faults, adapters, production, and later work remain unauthorized.

### R20 Phase 1 semantic mapping checkpoint

R20 resolves R19's semantic-to-physical ambiguity as documentation design by mapping one validated SOP1 to one structural RF1 type-3 provisional record. It prospectively authorizes only a pure public mapper module in `exp1-raw-append-replay`, depending directly on `exp1-record-format` and `exp1-workload-conformance`; append integration and changes to the other crates remain excluded. The separate live Linux capture decision remains open, and no descriptive execution or evidence is authorized.

### R21 Phase 1 reference-context checkpoint

R21 freezes the bounded reference-catalog/accepted-prefix split and locally decidable R12 outcomes without regenerating semantics. R22 classifies cross-segment targets, and R23 now freezes a canonical manifest-bound proof of complete cell scope. R25 subsequently records that the unchanged v1 bootstrap-to-reference premise is contradictory and supersedes R24 for implementation authorization only; the complete R20 gate remains open. Live Linux capture, the descriptive D1 harness, execution, and every later research phase remain gated.

### R22 Phase 1 cross-segment checkpoint

R22 freezes strictly segment-local reference eligibility and the experiment-local
`E-REFERENCE-CROSS-SEGMENT` disposition without changing R12/R14 bytes or ordering. This fully closes
only the cross-segment governance question. Proof of a complete closed stream scope, R21
implementation, the complete R20 gate, live Linux capture, a descriptive D1 harness, execution, and
later phases remain gated.


### R23 Phase 1 closed-scope checkpoint

R23 freezes the minimum deterministic closure proof: an immutable JCS descriptor names one reviewed
cell, canonically enumerates every namespace with exact R16 manifest and R14 stream/artifact
bindings, and commits to the bytes with a domain-separated digest. Exact equality with supplied
validated streams is required. This closes governance only; reference-context implementation,
live capture, harness construction, execution, and later phases remain separately gated.

### R24 Phase 1 implementation-authorization checkpoint (superseded)

R24 prospectively authorizes the smallest pure reference-context extension in the existing mapper.
Only the frozen mapper/context source and test paths may change; manifests, lockfile, authority crates,
dependencies, append/reopen, capture, and execution remain unchanged or excluded. R25 supersedes this implementation authorization only after its required v1 bootstrap-to-reference gate proved impossible. R24 remains an incomplete historical record; R26 now supplies the separate v2 conformance/validator authorization, not a reference-context implementation.


### R25 Phase 1 bootstrap causal-reference correction

R25 preserves the failed R24/closed-PR-#91 history and every R12/R14/R16 v1 byte while freezing a
prospective v2 uniform causal profile: ordinal 0 bootstraps independently with zero targets in each
segment, and later operations require positive ordered prior same-stream, same-segment ordinary
EventIds. A v2 manifest must encode separate bootstrap and subsequent cardinality for warm-up and
measured segments rather than one scalar. R25 supersedes R24 implementation authorization only and
authorizes no code. R26 now supplies the first, v2 conformance/validator-and-vector authorization;
a new bounded reference-context implementation still requires a later separate authorization. The complete R20
gate and every capture, harness, execution, durability, recovery, and benchmark gate remain open.


### R26 Phase 1 v2 conformance checkpoint

R26 completes the documentation freeze requested by R25: all version-sensitive profile identifiers,
canonical binary and JCS encodings, digest domains, per-segment cardinality policy, immutable
bindings, validator precedence, and literal-vector coverage are fixed. It authorizes only a later
side-by-side v2 extension of the existing workload-conformance crate with unchanged dependencies and
R9 validation. Reference-context code, the complete R20 gate, capture, harnesses, execution, and all
later phases remain gated.

## R27 checkpoint: v2 reference-context implementation authorized

R27 closes the merged R26 v2 conformance implementation as bounded correctness evidence and freezes
the minimum versioned R23 scope extension. The authorized pure v2 catalog/accepted-prefix/contextual mapper in
`exp1-raw-append-replay` is merged and R28 closes it and the remaining R20 reference-context
correctness gate as bounded correctness evidence. The independent live Linux capture freeze still
precedes any descriptive D1 harness, and workload/benchmark execution and later roadmap slices
remain unauthorized.


## R28 checkpoint: R27 closure and test-only D1 integration authorized

[R28](experiments/EXP-0001/R28-R27-CLOSURE-AND-END-TO-END-D1-INTEGRATION-AUTHORIZATION.md)
closes PR #98 reviewed head `67715b3efc4732542152ea9d935d92ebdb2ca0d6`, merged as
`f5cde575cbd82bb788b9519c4efc56e4d1186131` after both exact-head workflows succeeded. This closes
only the complete R20 reference-context correctness gate. One follow-on may update only the existing
`reference_context.rs` integration test plus closure documentation to prove all four literal SOP2
operations map to byte-exact RF1 frames, append through `RawAppender`, and physically reopen/replay
without loss or reordering, with transactional pre-append failure and `std`-only cleanup. This is
deterministic correctness testing, not a harness, capture, execution, durability, benchmark, or
performance increment. The live Linux capture freeze and every later roadmap gate remain blocked.

R29 closes the authorized test-only integration at PR #101 reviewed head `b88908cb9cbba39774437e582308bab25a88482b`, merge `2168839a70baebdea1773fc56e7b8aa0dc9a89e4`, with both exact-head workflows successful. It is bounded deterministic integration/correctness evidence only.


## R29 checkpoint: R28 integration closed and bounded preflight ABI frozen

[R29](experiments/EXP-0001/R29-R28-INTEGRATION-CLOSURE-AND-LINUX-CAPTURE-DECISION.md) closes the merged PR #101 integration as bounded deterministic correctness evidence and closes R19's semantic-to-physical mapping blocker. It freezes only a Fedora 44 Linux/x86_64 external-dependency-free clocks/resource/file/procfs preflight ABI, with five glibc calls isolated behind typed safe wrappers and fail-closed parsing/outcomes. Perf and tracefs remain blocked because their event, ABI, scope, scaling, and loss contracts are not frozen.

R30 closes the authorized fourth-member implementation at PR #103 reviewed head
`eda5005c3a3e6e6ec76e90f882bc320e7da1bce3`, merge
`1d8466d1ce8c7c99e0fbd572c1cb77b2e357ba11`, with both exact-head workflows successful, only as
bounded preflight implementation/correctness evidence. R30 freezes the next boundary as four
independent counter reads through `perf_event_open` and authorizes one non-live deterministic
extension only in the existing fourth crate. Tracefs, effective capture, harness execution, and all
later roadmap work remain blocked.

## R30 checkpoint: R29 preflight closed and counter-only perf ABI frozen

[R30](experiments/EXP-0001/R30-R29-PREFLIGHT-CLOSURE-AND-PERF-COUNTER-ABI-DECISION.md) records the exact PR #103 head, merge, and successful workflows as bounded correctness evidence only. Its prospective next increment is limited to callable wrappers, FD ownership, pure policy/scaling, and synthetic tests for CPU cycles, instructions, page faults, and context switches. CI and tests may not invoke perf or observe a host. Tracefs, mmap/ring buffers, sampling/signals, live capture, execution, publication, performance, durability, faults, adapters, and production remain outside roadmap authorization.


## R31 checkpoint: perf implementation closed and minimum descriptive D1 instrumentation frozen

[R31](experiments/EXP-0001/R31-PERF-IMPLEMENTATION-CLOSURE-AND-MINIMUM-D1-INSTRUMENTATION-DECISION.md) closes PR #105 reviewed head `547d0bba730d842e354a058773408c66b368f326`, merge `409d0737759d4e10fc39ad879ccc975858e89d9f`, with both exact-head workflows successful, only as bounded deterministic ABI/lifecycle/scaling/cleanup correctness evidence. It is not live capture, target validation, execution, or performance evidence.

The first descriptive B1/D1 minimum uses frozen clocks, both `getrusage` scopes, procfs/file observations, and four availability-conditional perf counters as separate sources. Tracefs is narrowly deferred with exact missing states because R8 does not make its attribution diagnostic primary; it remains blocked for confirmation and every attribution/loss claim. That deterministic injected-orchestration PR is now closed by R32 as bounded correctness evidence. Live use, records, workloads, benchmarks, publication, D2/D3, `fsync`, durability/recovery, faults, adapters/baselines, production, and later phases remain gated.


## R32 checkpoint: deterministic orchestration closed; internal adapter boundary frozen

[R32](experiments/EXP-0001/R32-R31-ORCHESTRATION-CLOSURE-AND-LIVE-ADAPTER-DECISION.md) closes PR #108 reviewed head `fb1200c31e4730404a1ee941cfb58fa77520f43b`, merge `0a5c2471a6f1d5d87a4d58a7b12ec921ec6bddcb`, with both exact-head workflows successful, only as bounded deterministic injected orchestration, lifecycle, source-metadata, failure, and cleanup correctness evidence. It freezes the borrowed measured-file capability, a single internal Linux/x86_64 live-boundary mapping, and independent per-event perf ownership. One later implementation PR may add only that adapter and synthetic tests in the existing fourth crate plus synchronized documentation. No test or CI host call, live caller, probe, target validation, tracefs, workload, record, capture, execution, publication, performance, durability, fault, external adapter/baseline, or production work is authorized.


### Closed R32-authorized adapter implementation

PR #111 reviewed head `71f58f65772fea2f0f58f5727d42e1405c7f09fb`, merged as `05dd7cc0980df2914dff5814ab5f5fba5b8e09e0` with both exact-head workflows successful, closes the single internal `LiveCaptureBoundary` implementation only as bounded deterministic adapter correctness evidence. It adds no caller or probe and preserves the distinction from live evidence.

## R33 checkpoint: adapter closed; target-preflight boundary frozen

[R33](experiments/EXP-0001/R33-R32-ADAPTER-CLOSURE-AND-TARGET-PREFLIGHT-DECISION.md) freezes the smallest explicit operator-invoked Fedora 44 Linux/x86_64 target-preflight caller, pre-existing read-only regular-file rules, exact wrapper checks, and a closed versioned retained diagnostic result. Exactly one later PR may implement that boundary in `exp1-descriptive-d1-harness`, dependency-free and with synthetic tests only. It may not be invoked in CI or retain host observations under that authorization. Workload materialization/actions, append/R7 integration, tracefs, calibration, benchmark execution/publication, durability/recovery/faults, baselines, and production remain gated.
