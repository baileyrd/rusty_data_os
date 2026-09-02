# Research Question Registry

| ID | Question | Status | Primary path |
|---|---|---|---|
| RQ-001 | Can one canonical information history support multiple independently optimized representations with acceptable performance and complexity? | Active; unproven | HYP-0001 and staged experiments |
| RQ-002 | What minimal semantics and measurements are required before event-ingestion implementation? | Complete as a readiness framework; all EXP-0000 outputs complete, with no implementation or evidence claim | EXP-0000 |
| RQ-003 | What are the cost and correctness characteristics of single-event ingestion at explicit durability boundaries? | Planned; R25 records the R12/R16/R21–R24 bootstrap contradiction and supersedes R24 implementation authorization; separate v2 conformance and later reference-context authorizations are required; descriptive D1 capture remains blocked/gated, and cost, performance, durability, execution, benchmark evidence, BLK-015, D2/D3, `fsync`, faults, adapters, production, and later increments remain open or unauthorized | EXP-0001 and its execution-readiness plan |
| RQ-004 | Can derived representations be rebuilt and validated at acceptable cost? | Deferred | Later replay/checkpoint experiments |

“Acceptable” remains workload- and requirement-specific; no benchmark evidence exists yet.

RQ-002 completion records measurement readiness, not evidence. The [EXP-0001 readiness plan](experiments/EXP-0001/EXECUTION-READINESS-PLAN.md) records the open decisions and gates. R1–R3 freeze requirements and lifecycle semantics; R4 accepts reviewed non-sensitive findings for conditional planning; R5 freezes framing, CRC-32C, append-only commit, and B0/B1 mappings. R5/R6 freeze all baseline profiles and mappings. [R7](experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) freezes evidence records, artifact provenance, instrumentation/overhead design, and the available fault boundary. R8 has frozen its bounded matrix, analysis design, and prospective owner-approved thresholds and is complete as documentation design; R9 froze the Slice A workspace/toolchain/test/CI boundary; the merged implementation passed its bounded correctness gate. R11 closes process-local, noncanonical D0 Slice B code; [R16](experiments/EXP-0001/R16-WORKLOAD-MANIFEST-SERIALIZATION-CONTRACT.md) resolves BLK-009 only as documentation design. Generator and manifest implementation still block workload observations, and execution, empirical equivalence, benchmarks, and durability claims remain blocked.

### R17 evidence note

Slice A2 answers only whether the covered frozen R12/R14/R16 vectors can be implemented together
without dependencies: its correctness suite says yes. It does not answer performance, execution,
storage, persistence, fault, durability, adapter, or production research questions.

The corrected M01/R7 fixture supports only the RQ-003 sub-question that the frozen contracts are mutually executable; it is not ingestion execution or performance evidence.


### R19 evidence and blocker note

R19 closes PR #74's exact reviewed Slice C/B1 implementation only as raw D1 submission and
physical accepted-prefix replay correctness evidence. It identifies the smallest useful
generated-workload descriptive D1 harness boundary but does not authorize it: R7's required Linux
direct Linux capture interfaces have no uniquely selected implementation under the current empty
external-dependency allowlist and unsafe-code prohibition, and, when R19 was decided, the semantic-operation-to-physical-record mapping was not frozen. R20 now resolves the mapping only; the interface/dependency-or-bounded-unsafe decision remains required before a harness; caller/authority identity assignment is selected and
requires a complete validated manifest. A later
separate execution gate remains mandatory.


### R20 mapping note

R20 freezes exactly one validated SOP1 to one structural type-3 RF1 provisional record and permits only a later pure public mapper module in `exp1-raw-append-replay`, with direct path dependencies on the unchanged `exp1-record-format` and `exp1-workload-conformance` crates. Append integration is excluded. The direct Linux capture decision remains the separate blocker; R20 adds no execution, correctness result, durability, recovery, or performance evidence.

### R21 reference-context note

R21 freezes the bounded immutable catalog and accepted prefix; R22 selects strictly segment-local eligibility; R23 completes the governance answer with a canonical manifest-bound closed-scope descriptor and digest. R24 subsequently authorizes only their bounded pure implementation; no implementation or correctness evidence exists yet. Live capture and all experimental execution remain blocked or unauthorized.

### R22 cross-segment reference note

R22 answers the cross-segment sub-question as documentation design: warm-up and measured references
are each strictly segment-local, and a known same-stream target in the other segment receives
`E-REFERENCE-CROSS-SEGMENT`. R23 subsequently answers the independent closed stream-scope proof. Reference-context implementation, full R20 closure, capture, and execution remain separately unauthorized.

### R23 closed-scope note

R23 completes the reference-classification governance answer: a reviewed cell's exact canonical
member list binds every R16 manifest and R14 WS1 stream and is committed by a domain-separated
digest. Omitted, extra, substituted, duplicate, foreign, or mismatched inputs fail context
construction. This closes governance only; implementation and evidence remain absent.

### R24 implementation-authorization note (superseded)

R24 historically authorized only the pending pure R21–R23 reference-context extension of the existing R20 mapper; R25 supersedes that implementation authorization after the v1 bootstrap contradiction was verified.
Its closed-scope construction, immutable catalog, opaque accepted-prefix state, transactional mapping,
resource/precedence tests, documentation synchronization, exact-head review, and both existing CI
workflows must close before correctness evidence exists. It does not authorize capture or execution.


### R25 governance-correction note

Closed, unmerged PR #91 falsified the assumption that the uniform R16 v1 causal profile could both
map each segment's first operation and later carry a valid R12/R22 prior reference. R25 preserves all
v1 vectors and freezes only a prospective v2 zero-target bootstrap plus positive subsequent
same-segment-reference policy with an explicit per-segment manifest representation. It supersedes
R24 implementation authorization only and adds no implementation or correctness evidence. RQ-003's
complete R20, capture, and execution questions remain open pending separately authorized v2
conformance and a later bounded reference-context implementation.
