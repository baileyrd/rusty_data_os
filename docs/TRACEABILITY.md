# Traceability Registry

This registry links intent to planned validation without inventing evidence.

| Source | Traces to | Status |
|---|---|---|
| Vision: represent once, materialize many | P-001, P-002, P-003, P-006, P-010; REQ-001; RQ-001 | Research direction |
| Vision: evidence-driven success | P-011, P-014, P-015; ADR-0001 | Accepted governance |
| REQ-001 through REQ-014 | ADR-0002; HYP-0001 where empirical feasibility is involved | Approved constraints; feasibility unproven |
| HYP-0001 | RQ-001; EXP-0000; EXP-0001; later representation experiments | Active / unproven |
| EXP-0000 | RQ-002; EXP-0001 prerequisites; [single-event semantic envelope](experiments/EXP-0000/SEMANTIC-EVENT-ENVELOPE.md); [reproducible workload contract](experiments/EXP-0000/WORKLOADS.md); [EXP-0001 baseline contract](benchmarks/BASELINES.md); [acknowledgement, visibility, fault, and durability contract](experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md); [crash/recovery correctness contract](experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) | Running; envelope, workload, baseline, lifecycle/durability, and crash/recovery outputs complete; remaining outputs incomplete |
| EXP-0000 semantic envelope fields and observation-side lifecycle metadata | REQ-002 through REQ-010, REQ-013, REQ-014; ADR-0002 | Approved-semantics elaboration; the original envelope is immutable after commit, provenance and integrity are conditional where applicable, and physical choices and listed semantic questions remain unresolved |
| EXP-0000 acknowledgement, visibility, fault, and durability contract | REQ-001, REQ-002, REQ-004, REQ-013, REQ-014; ADR-0002; EXP-0001 | Measurement-readiness semantics; D0/D1 remain provisional, D2/D3 require an explicit platform durability contract, and no experimental survival evidence is claimed |
| EXP-0000 crash/recovery correctness contract | REQ-001 through REQ-006, REQ-009, REQ-012 through REQ-014; ADR-0002; EXP-0001 correctness gate | Measurement-readiness procedure; defines oracle, fault placement/matrix, deterministic fail-closed recovery, D3 treatment, and result classifications without selecting physical mechanisms or claiming evidence |
| EXP-0000 reproducible workload contract | REQ-003 through REQ-010, REQ-013, REQ-014; RQ-002; EXP-0001 workload and baseline fairness | Measurement-readiness input; freezes deterministic sizes, distributions and semantic class order, content/envelope/temporal profiles, single-producer assigned-sequence order, concurrent producer-local order, matrix rules, and logical manifest fields; assignment is not canonical commit, and the contract does not select sequencing-gap policy, payload-byte generation, uncontrolled cross-producer interleaving, encoding, time, or serialization mechanisms |
| EXP-0000 EXP-0001 baseline contract | REQ-003 through REQ-010, REQ-013, REQ-014; RQ-002; EXP-0001 comparison and correctness gates | Measurement-readiness input; selects B0–B3 semantic profiles, classifies D0–D3 equivalence, freezes adapter/version/configuration fairness and exclusions, and cites official semantics without selecting executable versions, adapters, platform sync primitives, or claiming evidence |
| EXP-0001 | RQ-003; HYP-0001 (partial evidence only) | Proposed; blocked by EXP-0000 |
| ADR-0001 | Research lifecycle and benchmark methodology | Accepted governance decision |
| ADR-0002 | REQ-001 through REQ-014 and terminology | Accepted research constraints; no empirical validation claim |

No listed link represents completed experiment evidence.
