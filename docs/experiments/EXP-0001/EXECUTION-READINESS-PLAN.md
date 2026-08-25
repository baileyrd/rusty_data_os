# EXP-0001 Execution-Readiness and Staged-Implementation Plan

**Status:** Proposed plan; no implementation or execution authorized
**Scope:** bridge from the completed EXP-0000 framework to an executable EXP-0001
**Evidence classification:** planning and traceability only; no implementation or benchmark evidence

## 1. Authority and current readiness

This plan is the authoritative ordering and gate map for preparing EXP-0001. The
[EXP-0000 contracts](../EXP-0000-measurement-and-semantics-readiness.md) supply its semantic and measurement inputs; the
[EXP-0001 definition](../EXP-0001-immutable-event-ingestion.md) remains the experiment authority. If either changes materially,
this plan must be reviewed and versioned before work continues.

EXP-0000 is complete **as measurement-readiness documentation**. No implementation, benchmark, correctness-run, recovery-run,
or performance evidence exists. [HYP-0001](../../hypotheses/HYP-0001-event-log-as-canonical-state.md) remains active and unproven,
and EXP-0001 remains proposed. Implementation authorization and confirmatory-execution authorization are separate gates.
A documentation-complete experiment framework is not an executable experiment.

| Readiness level | Present state | Exit condition |
|---|---|---|
| Framework readiness | Complete | EXP-0000's seven reviewed contracts remain linked and internally consistent. |
| Design-choice readiness | Incomplete | The physical and lifecycle choices required by the intended slice are resolved through R1–R8 as applicable. |
| Implementation readiness | Incomplete | The first-implementation authorization gate in section 6 passes for exactly one slice. |
| Correctness-validation readiness | Incomplete | Stable fixtures, oracle, recovery rules, fault mechanisms, and validation procedures exist for the applicable modes. |
| Descriptive execution readiness | Incomplete | A validated runnable cell, environment, stream, adapter, instrumentation, and result path pass the descriptive gate. |
| Confirmatory execution readiness | Incomplete | The stricter frozen-design gate in section 7 passes; descriptive readiness alone is insufficient. |

## 2. Remaining blocker registry

`Open` means no reviewed output resolves the blocker. Merely knowing the required fields does not resolve it. Dependencies name
other blocker IDs or the frozen semantic contracts (`SEM`). Unknowns refer to the repository [unknown registry](../../ASSUMPTIONS-AND-UNKNOWNS.md).

| ID | Blocker | Status | Dependencies | Affected unknowns | Required output | Controls |
|---|---|---|---|---|---|---|
| BLK-001 | Physical event encoding and record framing | Open | SEM, BLK-002, BLK-013 | UNK-001, UNK-012 | Versioned physical contract and test vectors | Slice A/C authorization |
| BLK-002 | Minimum integrity policy and supported integrity modes | Open | SEM | UNK-012 | Required coverage, detection, finalization, failure behavior, and mode contract | R1; BLK-003 |
| BLK-003 | Integrity/checksum algorithm | Open | BLK-002, BLK-001 | UNK-012 | Reviewed algorithm/profile freeze and vectors | Integrity-enabled implementation |
| BLK-004 | Request/event identity algorithms and assignment lifecycle | Open | SEM, BLK-012 | UNK-002, UNK-009, UNK-016 | Separate algorithms, authorities, collision/error rules, and capture points | Slice A; comparable streams |
| BLK-005 | Timestamp representation, clocks, and capture points | Open | SEM, BLK-012 | UNK-003, UNK-010 | Representation, precision, clock and lifecycle rules | Slice A; records |
| BLK-006 | Payload-content generator | Open | SEM | UNK-018 | Platform-independent algorithm and test vectors | Comparable stream |
| BLK-007 | Identity/envelope/reference generator | Open | BLK-004, BLK-005, BLK-012 | UNK-013, UNK-018 | Deterministic generation specification and vectors | Slice A; comparable stream |
| BLK-008 | Stream and artifact digest algorithm | Open | BLK-006, BLK-007 | UNK-018, UNK-022 | Algorithm, canonical digest inputs, and vectors | Stream/artifact validation |
| BLK-009 | Workload-manifest physical serialization | Open | BLK-006–BLK-008 | UNK-019 | Versioned serialization and validator rules | Descriptive execution |
| BLK-010 | Benchmark-record physical serialization and validation | Open | BLK-008, BLK-025 | UNK-022 | Versioned serialization, validation and correction rules | Result admissibility |
| BLK-011 | Sequencing-gap treatment | Open | SEM, BLK-013 | UNK-017 | Rules for failed/abandoned candidates and replay checking | Slice C correctness |
| BLK-012 | Retry and uncertain-outcome behavior | Open | SEM, BLK-013 | UNK-016 | Request lifecycle, duplicate handling, retry and uncertain classification | Correctness gate |
| BLK-013 | Minimum append/replay recovery rules | Open | SEM, BLK-002 | UNK-001, UNK-015–UNK-017 | Physical invariants for boundaries, terminal damage, scanning and fail-closed replay | R1; Slice C |
| BLK-014 | Target OS and hardware/environment | Open | — | UNK-014, UNK-020 | Reviewed target and completed environment record | Physical API research; series freeze |
| BLK-015 | Filesystem/storage stack and platform durability contract | Open | BLK-014 | UNK-014, UNK-021 | Stack-specific D2/D3 promise and limits | Canonical D2/D3 claims |
| BLK-016 | B0 implementation profile | Open | BLK-001, BLK-006–BLK-007 | UNK-020 | Exact lower-bound operations and accounting | Slice B/baseline validation |
| BLK-017 | B1 append and synchronization APIs | Open | BLK-013–BLK-015 | UNK-014, UNK-020 | API/error/sync/grouping profile and mapping | Slices C–E |
| BLK-018 | SQLite/RocksDB versions, builds, bindings and configurations | Open | BLK-014–BLK-015 | UNK-020, UNK-021 | Exact reproducible profiles with verified effective settings | Slices F/G; series freeze |
| BLK-019 | Adapter mappings for every baseline | Open | BLK-001, BLK-004–BLK-005, BLK-011–BLK-012, BLK-016–BLK-018 | UNK-020, UNK-021 | Versioned semantic-to-physical mapping and equivalence classification per baseline | Baseline equivalence |
| BLK-020 | Benchmark harness architecture | Open | BLK-009–BLK-010, BLK-019 | UNK-020, UNK-022 | Experimental component and dependency-boundary design | Code authorization |
| BLK-021 | Instrumentation and overhead method | Open | BLK-014, BLK-020 | UNK-022 | Named instruments, scope, calibration/bounding and loss rules | Descriptive/confirmatory execution |
| BLK-022 | Fault-injection mechanisms | Open | BLK-013–BLK-015, BLK-017–BLK-019 | UNK-015, UNK-021 | Injection-point mapping, apparatus validation and coverage | Recovery readiness |
| BLK-023 | Numeric threshold registry and rationale | Open | Primary-cell design, baseline profiles | UNK-008 | Reviewed versioned per-cell values with evidence/rationale | Confirmatory execution only |
| BLK-024 | Estimator, interval, repetitions, stopping and run order | Open | BLK-014, primary-cell design | UNK-008 | Frozen analysis specification | Confirmatory execution only |
| BLK-025 | Artifact storage and retention | Open | BLK-008, repository constraints | UNK-019, UNK-022 | Layout, durable references, retention/redaction and supersession rules | Execution/result admissibility |
| BLK-026 | Reproducible toolchain/build configuration | Open | BLK-014, BLK-018 | UNK-020, UNK-022 | Toolchain, target, flags, lock/build identity and reproduction procedure | Code/series authorization |
| BLK-027 | Cargo/workspace and CI bootstrap authorization | Open | BLK-020, BLK-026, section 6 | UNK-020, UNK-022 | Reviewed layout, dependency allowlist, CI plan, and approval record | First code creation |

## 3. Dependency ordering

The work is a directed sequence of independently reviewable freezes, not one resolve-everything gate.

| Order | Predecessor → dependent decision | Reason |
|---|---|---|
| 1 | Frozen semantics → BLK-002/013 → BLK-001/003 | Required integrity and recovery behavior precede framing; integrity policy precedes an algorithm. |
| 2 | Lifecycle semantics → BLK-004/005/011/012 | Physical identity, time, gaps and retries must preserve the semantic distinctions. |
| 3 | BLK-004–006/011/012 → BLK-007 → BLK-008/009 | Byte generation, identities and references precede reproducible streams, manifests and digests. |
| 4 | BLK-014 → BLK-015 → BLK-017/018/022 | Target stack determines meaningful sync and fault mechanisms; its durability contract precedes canonical D2/D3 claims. |
| 5 | BLK-001/004/005/011/012/015–018 → BLK-019 | Adapters can claim equivalence only after both semantic and physical sides are known. |
| 6 | BLK-008–010/019/025 → BLK-020/021 | The harness must preserve validated inputs, mappings, measurements and artifacts. |
| 7 | BLK-014/018/026 plus all effective configuration → series freeze | Environment and executable versions are frozen before any benchmark series. |
| 8 | Primary cells and BLK-023/024 → confirmatory freeze | Thresholds and analysis are fixed before observations, never after them. |
| 9 | Fixtures/oracles/adapters/BLK-022 → correctness and recovery pass → performance interpretation | Correctness gates speed; fault coverage must match the claimed contract. |

## 4. Recommended readiness sequence

Each increment ends in its own review; none silently authorizes a later increment.

| Increment | Question and inputs | Required artifact | Deliberately retained | Gate / implementation authorization | Validation | Revisit condition |
|---|---|---|---|---|---|---|
| R1 — minimum physical record, integrity-policy, and replay/recovery requirements | What must one record guarantee? Inputs: semantic envelope, durability and crash contracts. | Requirements/research record resolving BLK-002 and BLK-013 and constraining BLK-001/003. | Encoding and checksum algorithm. | Review semantic coverage and fail-closed rules; **no code**. | Examples reviewed against every recovery invariant. | A requirement is unimplementable or misses a declared fault. |
| R2 — deterministic workload bytes, identity/reference inputs, and digests | How can an equivalent operation stream be regenerated and verified? Inputs: workload contract plus R1 boundaries. | Generator/digest requirements and reference-vector plan for BLK-006–009; algorithm selections require separate recorded rationale. | Runtime harness and event encoding beyond digest-input boundaries. | Byte-for-byte cross-implementation review; **no code**. | Hand/reference cases and platform-independence review. | Ambiguous canonical input or substitution cannot be detected. |
| R3 — identity, time, gaps, retry, and uncertain-outcome lifecycle | When and by whom are values assigned and failures classified? Inputs: semantic/lifecycle contracts and R1. | Lifecycle record resolving BLK-004/005/011/012 and constraining BLK-007. | Concrete storage APIs. | No semantic ambiguity for Slice A/C; **no code**. | State-transition and failure-scenario review. | Contradiction with REQ-003/004/006/009/013. |
| R4 — target environment and platform durability | What exact stack is studied and what can D2/D3 claim? Inputs: environment template and platform primary sources/research. | Completed target profile and platform contract resolving BLK-014/015. | Baseline binaries and performance claims. | D2/D3 vocabulary approved only for stated stack; **no code**. | Contract-to-fault matrix review. | Stack or relevant configuration changes. |
| R5 — B0/B1 physical profiles and adapter contracts | What lower bounds and OS operations preserve the declared cells? Inputs: R1–R4 and baseline contract. | BLK-016/017 and B0/B1 portions of BLK-019. | SQLite/RocksDB details. | Profiles reviewed for equivalence; **no code**. | Mapping and error-path table review. | An API cannot meet the intended D-mode. |
| R6 — SQLite/RocksDB execution-profile freeze | Which exact reproducible builds and mappings are eligible? Inputs: R3/R4 and official version semantics. | BLK-018 and B2/B3 portions of BLK-019. | Empirical equivalence until tested. | Conditional/diagnostic status preserved; **no code by itself**. | Configuration reproduction and mapping review. | Version, build, binding, default or stack changes. |
| R7 — benchmark records, artifacts, instrumentation and faults | How are runs captured, validated, retained and faulted without corrupting inference? Inputs: R1–R6 and record contracts. | Resolution of BLK-010/021/022/025 and constraints for BLK-020. | Numeric thresholds and executable harness. | Descriptive design review; **no code by itself**. | Schema examples, provenance graph and apparatus-validation plan review. | Missing state/provenance or overhead cannot be bounded. |
| R8 — primary matrix, thresholds and statistical plan | What observations can decide predeclared claims? Inputs: interpretation contract, frozen profiles/environment. | Resolution of BLK-023/024 and exact primary-cell matrix. | Exploratory expansions. | Confirmatory-design approval; **no code by itself**. | Completeness and independent statistical review. | Unsupported rationale, excessive uncertainty, or material design change. |
| R9 — workspace, harness, CI and first-slice authorization | Can exactly one approved slice be implemented reproducibly? Inputs: applicable R1–R8 outputs. | BLK-020/026/027, section 6 checklist, and approval record naming the slice. | Every later slice and all execution not expressly approved. | **May authorize one slice only.** | Documentation/link checks plus planned format, unit, property and CI checks. | Dependency/layout/toolchain change or newly exposed semantic ambiguity. |

R8 need not block implementation of a fixture-only first slice when that slice produces no benchmark claim, but it must precede
confirmatory execution. R6 may occur after early B0/B1 implementation if those adapters are not enabled. This preserves independent
review rather than forcing unrelated database choices into the first code gate.

## 5. Conditional staged-implementation proposal

All eventual code remains under `/experiments/`; nothing graduates to `/crates/` without evidence, an ADR when warranted, and a
specification. The ordering is conditional: A is the candidate first slice; B–E earn continuation in order because each isolates a
new mechanism. F and G may follow C once common adapter validation is stable and may be ordered independently; they do not have to
wait for D3. H is introduced incrementally for the first persistence slice, not postponed as a monolith. A failed gate can stop the
sequence, and no slice is authorized by this plan.

| Slice | Research question / D-mode | Prerequisites; inputs → outputs | Correctness tests | Initially enabled cells | Exclusions | Evidence and continuation gate |
|---|---|---|---|---|---|---|
| A — deterministic fixtures and validators | Can semantic operations and physical records be generated and rejected deterministically? No benchmark D-mode. | R1–R3 physical contracts, stable vectors → fixtures/validators and oracle outputs. | Golden-vector, round-trip where applicable, malformed/truncated/corrupt, identity/reference/order checks. | None; validation only. | Append, sync, performance claims, production schema. | Reproducibility/correctness evidence; approve B only if vectors and oracle pass independently. |
| B — B0 in-memory lower bound | What is construction/sequencing/accounting overhead without persistence? D0 only. | A, B0 profile, harness subset → provisional in-memory observations and mappings. | Unique monotonic assignment, no invention/duplication, D0 never labeled canonical. | Reference single-producer P1–P3 minimal-envelope D0, descriptive first. | Recovery/durability and database adapters. | B0 mechanism evidence; approve C only if accounting and D0 semantics validate. |
| C — B1 raw append/replay | What does OS-buffer append and deterministic replay cost? D1 only. | A/B, R4/R5, fault/result subset → append artifact and replay/oracle record. | Boundaries, short writes/errors, terminal damage, gaps, replay order; no D1 canonical claim. | Reference P1–P3 minimal-envelope D1, descriptive first. | Stable-media claims and grouping. | D1/replay evidence; approve D only after B1 mapping, fault apparatus and recovery oracle pass. |
| D — B1 per-event sync | What is per-event declared synchronization behavior/cost? D2 only under the platform contract. | C plus R4 sync contract → synchronized records and fault results. | Ack/error ordering and every claimed D2 fault; recover every canonical ack. | D2 subset matching validated D1 cells; confirmatory only after section 7. | D3 and claims beyond the target stack. | D2 correctness/cost evidence; approve E only if D2 is valid and D3 remains a useful separate question. |
| E — controlled B1 grouping | What trade space follows from observable controlled grouping? D3 only. | D plus frozen membership/window/count policy → group outcomes and membership evidence. | Shared sync/outcome, no partial group ack, per-event latency includes formation wait, faults at group boundaries. | Small predeclared D3 subset; expand only after validation. | Atomic multi-event transactions and opaque grouping. | D3 trade-space evidence; continue only if group semantics and measurement are valid. |
| F — SQLite adapter | Which baseline profiles are semantically comparable and at what cost? D1; D2 only conditionally validated. | A/C, R4/R6 mapping → adapter/config/equivalence and results. | Stream mapping, effective settings, error/retry, replay and claimed-fault oracle. | Cells equivalent to validated subject cells. | Strict D3 via transactions; product conclusions. | Conditional baseline evidence; next adapter requires stable shared validation, not a win. |
| G — RocksDB adapter | Same question for the selected RocksDB WAL profile. D1; D2 only conditionally validated. | Same class as F with its independently frozen profile. | Same obligations plus verified WAL/write options and recovery mapping. | Equivalent validated cells only. | `WriteBatch` as strict D3 and opaque grouping claims. | Conditional baseline evidence; continuation requires valid equivalence and reproducibility. |
| H — fault/recovery and result-capture integration | Can each persistence slice produce admissible, retained correctness evidence? Applicable D1–D3, never a new mode. | Begins with C; R7 artifacts/faults/instrumentation → validated raw records and provenance. | Apparatus self-tests, full applicable fault matrix, schema/provenance/digest validation, missing-data behavior. | Diagnostic apparatus cells first; later the exact approved matrix. | New storage mechanisms or post-hoc analysis. | Admissibility evidence; performance interpretation stays blocked until correctness and the applicable execution gate pass. |

## 6. First implementation authorization gate

Creating any Cargo file or Rust code requires one repository-recorded reviewer approval that names exactly one slice and confirms:

- the first slice is approved and bounded;
- every physical contract required by that slice is resolved, versioned, and linked;
- stable fixture/test-vector inputs and a defined independent correctness oracle exist;
- repository layout and dependency boundaries keep experimental code under `/experiments/`;
- the toolchain, target and build configuration are frozen reproducibly;
- the CI validation plan states format, static, unit, property/vector, and documentation checks applicable to the slice;
- an explicit allowed direct-dependency list records purpose, version policy, alternatives, license/build implications, and why standard facilities are insufficient;
- no unresolved semantic ambiguity could invalidate the slice;
- rollback/revisit conditions and prohibited later slices remain explicit; and
- approval identity, date, reviewer, addressed blocker IDs and exact repository state are recorded.

The gate fails closed. Passing this planning increment does **not** satisfy it, authorize Cargo bootstrap, or authorize all slices.

## 7. Execution gates

### Descriptive execution

Descriptive execution may record mechanism behavior only after the selected runnable cell has validated workload identity, subject or
adapter mapping, environment/series identity, applicable correctness oracle, instrumentation/overhead assessment, and validated raw
record/artifact path. It must be labeled descriptive, diagnostic, or exploratory. If numeric thresholds or analysis rules remain open,
it cannot support threshold-based claims.

### Confirmatory execution

Confirmatory execution additionally requires completed versioned threshold-registry entries; a frozen estimator, interval,
repetition, stopping, multiplicity and run-order method; a complete predeclared primary-cell matrix; exact environment and series
identity; a validated workload stream; validated subject and every applicable baseline adapter; passed correctness and recovery gates;
exact fault coverage matching each claim; and validated artifact retention and benchmark records. Deviations start a new series or
downgrade evidence as the interpretation contract requires.

A descriptive result can never be retroactively relabeled confirmatory. A later confirmatory design requires new observations gathered
under that already-frozen design.

## 8. Decision records and traceability

Every R increment must record: blocker IDs addressed; affected unknowns; linked requirements and experiment sections; decision or
research-record identity; alternatives considered; evidence actually available; assumptions; unresolved questions; revisit conditions;
status; and immutable supersession links. A freeze is not `resolved` until its required output is reviewed and repository-recorded.

An ADR is required when a choice materially constrains durable architecture, public interfaces, cross-experiment correctness semantics,
or a format proposed for graduation. Experiment-local encodings, algorithms, versions, bindings, flags, target environments, harness
mechanics, and statistical freezes belong in versioned experiment research/configuration records unless promoted. An ADR must not turn
an untested performance candidate into validated architecture; empirical promotion cites admissible evidence.

## 9. Phase reconciliation and next increment

Phase 0's documented exit criteria are now satisfied: EXP-0000 is complete, EXP-0001 has a reproducible semantic benchmark plan and
correctness criteria, baseline families are identified, and target environments can be recorded consistently. The project therefore
enters **Phase 1 planning/readiness**, not Phase 1 implementation. There is still no Phase 1 implementation or evidence, EXP-0001 is
not executable, and section 6 remains mandatory.

The **one next bounded increment** is **R1 — minimum physical event-record, framing, integrity-policy, and replay/recovery requirements**.
It must state what the first physical record guarantees before selecting a concrete encoding or checksum algorithm. This plan does not
perform R1.
