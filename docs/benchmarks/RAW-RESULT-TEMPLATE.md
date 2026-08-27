# Raw Benchmark Result Record Contract

**Logical schema:** `benchmark-raw-result/v1`
**Status:** EXP-0000 measurement contract; no measurements are recorded

**EXP-0001 physical profile:** The fields below are serialized, validated,
retained, and linked through provenance according to the documentation-only
[R7 authority](../experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md).
This remains the logical contract; R7 creates no schema, validator, result, or evidence.

## 1. Conformance and preservation

This logical record preserves one observation or a bundle whose members remain individually identifiable. **R** is required, **C** conditionally required, and **M** recommended. Raw observations are immutable: corrections create a new `result_id`, cite `supersedes`, and retain the old record. Failed, negative, invalid, diagnostic, non-equivalent, and inconclusive runs remain preserved. Summaries reference raw identities and never replace raw evidence.

The logical missing concepts `unknown`, `unavailable`, `unmeasured`, and
`inapplicable` map respectively to the selected EXP-0001 physical states
`missing`, `not_collected`, `not_collected`, and `not_applicable`;
`unsupported` remains `unsupported`. These states, zero, and empty collections
are distinct. Every number has a machine-processable unit and method. For
EXP-0001, `EXP1-R7-JSON-JCS-1` selects serialization, timestamps, UUID record
identities, SHA-256 artifact digests, raw-sample representation, and validation
rules. Validator implementation remains unauthorized and absent; the canonical
event-integrity algorithm remains the separate R5 CRC-32C decision.

## 2. Normative fields

### Identity and exact inputs

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `schema.name`, `schema.version`, `result_id` | Logical version and immutable result identity with declared algorithm. |
| C | `supersedes`, `correction_reason` | Prior identity and reason for correction/reclassification. |
| R | `experiment_ref`, `hypothesis_refs`, `requirement_refs` | Exact experiment/version and applicable hypotheses/requirements. |
| R | `subject_id`, `profile_id`, `baseline_id`, `series_id` | Subject/profile, baseline or reasoned inapplicable, and frozen series. |
| R | `repository` | Exact HEAD, dirty declaration and patch artifact if dirty. |
| R | `environment_ref`, `workload_ref` | Immutable environment and exact workload manifest/stream identities/digests. |
| R | `adapter_ref`, `configuration_refs` | Exact adapter and requested/effective subject, baseline and orchestration configuration. |
| R | `platform_contract_ref` | Exact acknowledgement/durability/promised-fault contract. |
| R | `producer_record` | Harness/producer identity, version/build, host role and capture mechanism. |

### Coordinates, lifecycle, and time

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `durability_mode`, `commit_status` | D0–D3 and `provisional` or `canonical-committed`; D0/D1 are always provisional. |
| R | `run_id`, `repetition_id`, `phase`, `sample_id` | Run coordinates; phase names setup, warm-up, measured, fault, recovery, cleanup, or another declared phase. |
| R | `observation_role` | Warm-up, measured, or non-performance phase; never silently pooled. |
| C | `operation_id`, `producer_id`, `thread_id` | Operation, workload producer, and thread where meaningful. |
| C | `durability_group_id`, `group_membership` | For D3: eligible/exact member identities, join/cut reason, shared sync and each acknowledgement. |
| R | `interval.start`, `interval.end`, `interval.elapsed` | Representations and elapsed value/unit/method with clock/source/precision reference. |
| R | `time_meanings` | Applicable system-acceptance, effective, durability and observation times, separately named and sourced. |
| R | `lifecycle_interval` | Exact endpoints for latency, including D3 group wait where applicable. |

### Counts and bytes

All fields are R; an explicit missing state and reason is valid. Definitions and partition/double-count relationships are declared, never assumed.

| Group | Values |
|---|---|
| `operation_counts` | Attempted, accepted, rejected, acknowledged, provisional, committed, failed, uncertain, recovered, corrupt, and missing counts, each with unit and method. |
| `logical_bytes` | Payload and applicable key/value/envelope logical bytes. |
| `encoded_bytes` | Complete event, framing and integrity bytes presented to the adapter. |
| `physical_bytes` | Written, read and synchronized bytes; separately attributable WAL, database/SST, manifest, temporary, checkpoint, compaction and other bytes where measurable. |

One transaction, row, key, WAL/database record, SST entry, or `WriteBatch` is not presumed to equal one canonical event. Any mapping requires an explicit adapter definition and evidence. Logical, encoded, requested-I/O, OS and device byte domains remain separate.

### Metrics and execution observations

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `throughput` | Operation/byte rate, numerator/denominator, interval, unit and method. |
| R | `latency` | Raw samples or histogram/distribution reference, sample count, interval, unit, method, bucket/quantile algorithm and loss/rounding metadata. Quantiles alone do not replace raw evidence. |
| R | `cpu`, `allocations`, `memory` | User/system/wall CPU and scope; allocation count/bytes; resident/virtual/peak/cache measures, each with unit/method or explicit missing state. |
| R | `io`, `synchronization` | Per-layer I/O counts/bytes/queues and requested/completed/failed syncs, waits, primitive and group scope. |
| R | `amplification` | Explicit numerator, denominator and scope for each write/read/space ratio. |
| M | `resource_measurements` | Context switches, faults, scheduler, energy, thermal, frequency, network or other predeclared counters. |
| R | `execution_observations` | Errors, retries, partial writes, checkpoints, compactions, flushes, stalls, backpressure and background work with count/time/source or explicit state. |

Derived metrics cite source counters and calculation version. Distributions state population and omissions; incompatible clock or measurement scopes are not conflated.

### Correctness, faults, recovery, and classification

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `correctness_gate` | Pass, fail, or inconclusive; invariant/oracle version, checks, failures and evidence. Performance interpretation requires pass. |
| C | `fault` | Injection point/trigger, requested and observed fault, apparatus, promised fault class and outcome. |
| C | `oracle_obligations` | Expected eligible/committed/provisional set, order, D3 membership, corruption and uncertainty obligations. |
| C | `recovery` | Classification; recovered/missing/duplicate/invented/corrupt/uncertain identities/counts; scan, replay and time-to-ready values/units/methods. |
| C | `d3_outcome` | Exact membership, shared synchronization result, and each independent event outcome; never an implied atomic multi-event transaction. |
| R | `equivalence_classification` | Equivalent-candidate, conditionally-equivalent, diagnostic, non-equivalent, or declared class and satisfied conditions. |
| R | `result_classification` | Applicable valid, invalid, failed, negative, inconclusive, and diagnostic labels with reasons. |

D0/D1 cannot be silently promoted to D2/D3 canonical outcomes. Correctness failure makes performance diagnostic only regardless of speed. Conditionally equivalent and diagnostic baseline records retain those classifications in all summaries.

### Artifact integrity and provenance

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `artifacts` | Each stream, histogram, log, trace, configuration, manifest, oracle, recovery image and diagnostic: immutable URI, role, media type, byte size, digest algorithm/value, producer/tool version/build, relation and retention state. |
| R | `provenance_edges` | Typed `generated-by`, `captured-from`, `configured-by`, `derived-from`, `validated-by`, or `supersedes` source/target relationships. |
| R | `validation.status` | Not-validated, pass, fail or inconclusive plus validator identity/version/configuration, time and findings. |
| R | `validation.integrity` | Record/artifact checksum results and algorithms. These do not select the unresolved canonical event-integrity algorithm. |

## 3. Cross-record provenance and freeze invariants

The raw result references its immutable environment parent rather than copying a mutable subset. Both reference a common artifact graph. Workload stream, configuration, adapter, executable, platform contract, fault plan and validation output have identities/digests, with edges showing production and checks. External large data require a durable repository manifest/reference sufficient to detect substitution.

Any semantics- or execution-affecting environment, code, configuration or artifact change starts a new series under the environment contract. Exact workload, environment, configuration, code and artifact identities remain traceable. Corrections link, never overwrite. Summaries enumerate included and excluded result identities and reasons. Safe redaction is recorded without breaking stable technical relationships.

## 4. Non-normative illustration (not a measurement)

```yaml
schema: {name: benchmark-raw-result, version: 1}
result_id: "illustrative-result-id"
series_id: "illustrative-series-id"
environment_ref: {id: "illustrative-environment", digest: "<algorithm>:<digest>"}
durability_mode: D1
commit_status: provisional
observation_role: measured
operation_counts:
  attempted: {value: 100, unit: event, method: harness-counter}
  committed: {value: 0, unit: event, method: lifecycle-classifier}
correctness_gate: {status: pass, note: "illustrative only"}
latency: {state: unmeasured, reason: "example intentionally contains no data"}
artifacts: []
```

All values are fictional placeholders. YAML remains a non-normative illustration
of the logical contract and is not the selected EXP-0001 serialization.
EXP-0001 selects `EXP1-R7-JSON-JCS-1`; its complete fictional JSON record is in
the [R7 examples](../experiments/EXP-0001/R7-PHYSICAL-RECORD-EXAMPLES.md).
