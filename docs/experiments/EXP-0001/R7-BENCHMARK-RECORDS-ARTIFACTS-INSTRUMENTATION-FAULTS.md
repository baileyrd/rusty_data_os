# R7 Benchmark Records, Artifacts, Instrumentation, and Fault Apparatus

**Status:** Complete documentation design; no implementation or execution authorized
**Scope:** EXP-0001 readiness increment R7; BLK-010, BLK-021, BLK-022, and BLK-025
**Evidence classification:** normative experiment apparatus contract, not generated evidence, a benchmark result, or product architecture

## 1. Authority, boundary, and invariants

This contract physically realizes, without changing the meaning of, the logical
[`benchmark-environment/v1`](../../benchmarks/ENVIRONMENT-TEMPLATE.md) and
[`benchmark-raw-result/v1`](../../benchmarks/RAW-RESULT-TEMPLATE.md) contracts. It is governed by the
[methodology](../../benchmarks/METHODOLOGY.md), [interpretation contract](../../benchmarks/INTERPRETATION-CRITERIA.md),
[EXP-0000 lifecycle](../EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md) and
[recovery](../EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) contracts, R1–R6, and the
[execution-readiness plan](EXECUTION-READINESS-PLAN.md). Where this document is silent, those authorities control.

The following invariants are not negotiable:

1. Canonical history is the only product authority. Controller ledgers, records, artifacts, validators, recovery images, and analyses are benchmark evidence only.
2. D0/D1 remain provisional. D2/D3 are canonical only within the exact recorded platform contract. Sync success, process-kill survival, simulation, or a fault test cannot establish unsupported power-loss durability.
3. Published records and artifacts are immutable. A correction links to and retains what it supersedes; it never overwrites it.
4. Missing, failed, negative, invalid, diagnostic, and inconclusive observations remain distinguishable and retained.
5. Hostnames, users, network addresses, and absolute paths are excluded or consistently pseudonymized. Every redaction states its reproducibility impact.
6. R7 chooses no benchmark thresholds, matrix cells, sample sizes, stopping rules, Cargo layout, crates, CLI, executable, workflow, or CI architecture. Those remain R8 or R9 work.

## 2. BLK-010 — physical record serialization

### 2.1 Version and byte profile

`benchmark-record-json/1` is the physical profile for both existing v1 logical schemas.

| Property | Normative rule |
|---|---|
| Media type | `application/vnd.rusty-data-os.benchmark-record+json;profile=1` |
| Text | UTF-8, with no BOM; JSON per RFC 8259; duplicate object member names are invalid |
| Physical root | Exactly one JSON object; the logical record is that object, not an outer wrapper |
| Canonical bytes | RFC 8785 JSON Canonicalization Scheme (JCS), followed by no newline |
| Integers | JSON integer tokens only; validators must preserve and range-check exact integers rather than round through binary floating point |
| Decimal values | JSON numbers are forbidden for measured non-integer quantities; use an integer coefficient and integer base-10 scale |
| Null | Forbidden. Absence uses the explicit state object in section 2.3 |
| Extensions | Unknown fields are invalid. A new field requires a new logical or physical profile version |
| Canonical names | The logical table names are exact. A dotted name denotes nested objects: for example `schema.name` serializes as `{"schema":{"name":...}}` |
| Collections | Arrays retain declared order. Set-like arrays are sorted by their specified stable identity; the producer must not rely on object-member order |

The root `schema` object contains exactly `name` and integer `version`. It is
`{"name":"benchmark-environment","version":1}` or
`{"name":"benchmark-raw-result","version":1}`. This profile adds two reserved root members to each logical record:

```json
"physical_profile":"benchmark-record-json/1",
"record_integrity":{"algorithm":"sha-256","value":"<64 lowercase hexadecimal characters>"}
```

`record_integrity.value` is SHA-256 over the JCS bytes of the complete root after removing only the
`record_integrity` member. This digest detects substitution and is not the `environment_id` or `result_id`.
Those immutable identities use the R3 typed UUIDv4 representation: lowercase canonical UUID text. Identity is assigned once by
the producer; changing any record content requires a new identity. The digest algorithm is fixed by this profile rather than
relying on an unvalidated free-text declaration.

Every reference object uses exactly:

```json
{"id":"<typed lowercase UUIDv4 when the target has one>","uri":"<immutable URI>","digest":{"algorithm":"sha-256","value":"<64 lowercase hex>"}}
```

`id` is omitted only for artifacts without a logical identity. At least `uri` and `digest` are required. Repository-relative URIs
use `repo:artifacts/...`; external URIs use `ext:<provider>/<immutable-object-key>`. Mutable URLs, branch names, `latest`, and
absolute host paths are invalid references.

### 2.2 Time, duration, quantities, and enumerations

Canonical instants use an object with signed 64-bit Unix-epoch nanoseconds and the clock evidence required by R3/R4:

```json
{"unix_epoch_ns":1770000000000000000,"clock_ref":"clock:os-realtime","capture_point":"record-finalized"}
```

Run-relative instants and elapsed durations use signed 64-bit and non-negative 64-bit integer nanoseconds respectively:

```json
{"run_relative_ns":1250,"clock_ref":"clock:monotonic"}
{"value":1250,"unit":"ns","method":"monotonic-end-minus-start"}
```

There are no floating timestamp strings, local times, implicit time zones, or bare numeric durations. Effective, system-acceptance,
durability/commit, acknowledgement, observation, and materializer times remain separately named; absent meanings use an explicit
state rather than being copied or inferred.

All measured quantities use one of:

```json
{"value":42,"unit":"event","method":"harness-counter"}
{"coefficient":12345,"scale10":-3,"unit":"event/s","method":"accepted-over-elapsed"}
```

Units are exact profile tokens: `ns`, `byte`, `event`, `operation`, `count`, `event/s`, `byte/s`, `ratio`, `percent`, or a later
versioned token. Zero is a numeric value, never a missing state. Enumerations are the exact lowercase/hyphenated terms in the logical
contracts; D-modes retain uppercase `D0`–`D3`.

### 2.3 Required, conditional, recommended, and missing states

The R/C/M/N levels in the logical contracts remain authoritative:

- **R:** the member must exist. If the logical contract permits missing evidence, its value may be an explicit state object.
- **C:** the member must exist when its stated condition is true. When false, it must exist as `inapplicable` so the condition was evaluated.
- **M:** the member must exist with a value or an allowed explicit state; omission is invalid.
- **N/inapplicable:** only an explicit `inapplicable` state with a reason is valid.

An explicit state object has exactly these members:

```json
{"state":"unmeasured","reason":"counter was outside this diagnostic run's declared scope","impact":"CPU comparison unavailable"}
```

Allowed states are `unknown`, `unavailable`, `unsupported`, `unmeasured`, and `inapplicable`. `reason` and `impact` are non-empty
UTF-8 strings. A state object cannot also contain `value`, `coefficient`, or measurement fields. Empty arrays mean a known empty set;
they do not mean missing. The environment contract does not allow `unmeasured`; the raw-result contract does. A logically required
identity, schema/version, series/run coordinate, classification, or immutable reference may not be replaced by a missing state.

### 2.4 Deterministic validation

A conforming validator performs these stages in order and emits all findings sorted by `(JSON Pointer, rule code)`:

1. Decode UTF-8 and JSON strictly; reject BOM, malformed input, duplicate names, non-integer numbers where prohibited, and trailing bytes.
2. Select the physical profile and exact logical schema/version; reject unknown profile, schema, version, fields, units, states, and enums.
3. Validate types, integer ranges, UUID spelling/version, lowercase hex digests, URI scheme, required members, conditional predicates, and missing-state legality.
4. Evaluate cross-field rules from the logical contracts: correction pairing; D0/D1 provisional status; D3 membership; warm-up separation; correctness/performance interpretation; count partitions; time endpoint order; reference completeness; and baseline/fault conditionality.
5. Recompute JCS bytes and `record_integrity`; validate each referenced artifact's size and digest when available. An unavailable target produces `inconclusive`, never `pass`.
6. Validate graph closure and series-freeze rules in section 3. A missing required node/edge is failure; an inaccessible external byte object is inconclusive and makes the record inadmissible until resolved.

Validation status is `pass`, `fail`, or `inconclusive`; `not-validated` is a recordable pre-validation state, never admissible evidence.
Given identical record bytes and the same immutable artifact set, validators must return the same status, rule codes, pointers, and
digest results. Tool crashes or unsupported profile versions are apparatus failures, classified `invalid`, not record failures.

Minimum rule codes are `JSON_SYNTAX`, `UNKNOWN_FIELD`, `MISSING_REQUIRED`, `CONDITION`, `TYPE`, `RANGE`, `ENUM`, `UNIT`,
`IDENTITY`, `TIME`, `MISSING_STATE`, `DIGEST`, `REFERENCE`, `GRAPH`, `CORRECTION`, `DURABILITY`, and `INTERPRETATION`.
Human prose may supplement but cannot replace them.

### 2.5 Correction and supersession

A correction is a complete new record with a new typed UUIDv4 identity and digest. Both `supersedes` and non-empty
`correction_reason` are present together; `supersedes` references the immediate prior record. The graph must be acyclic, retain every
node, and have at most one accepted successor per record within a publication set. Competing successors remain preserved and make the
set inconclusive until an explicit later correction resolves the fork. A corrected raw observation cannot change facts that would
require a new run (workload, environment, execution, or samples); such a change is a new result/series, not a correction.

Retraction, redaction, deletion, and reclassification are correction records, not mutation. Analyses declare whether they use the
latest non-tombstoned successor while still enumerating excluded identities and reasons.

### 2.6 Serialization examples

These fragments are exact JCS-compatible JSON but use shortened digest placeholders and therefore are documentation examples, not
published evidence. The first demonstrates a **valid structural pattern**:

```json
{"environment_id":"018f47d2-455a-4f7b-8dd9-97ad5d9c8a11","physical_profile":"benchmark-record-json/1","record_integrity":{"algorithm":"sha-256","value":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"schema":{"name":"benchmark-environment","version":1}}
```

In a complete environment record, all logical R/C/M fields follow section 2.3; this abridgement is not itself a complete environment
record. The following is an **invalid value** because zero has been conflated with missing evidence and the unit/method are absent:

```json
{"latency":{"value":0}}
```

The valid explicit-missing form is:

```json
{"latency":{"state":"unmeasured","reason":"latency collection was disabled before this diagnostic run","impact":"no latency or tail comparison is permitted"}}
```

A valid correction relationship is:

```json
{"correction_reason":"classification corrected after validator defect review; samples unchanged","result_id":"87cb3fe5-b51d-47d8-a944-76fef10ad4c2","supersedes":{"id":"b299ab72-c001-48f7-bc46-c277aa702ab1","uri":"repo:artifacts/records/b2/b299ab72-c001-48f7-bc46-c277aa702ab1.json","digest":{"algorithm":"sha-256","value":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}
```

A full conformance example would duplicate every field in the logical tables and could falsely resemble generated evidence. R7
instead freezes exact forms above; executable fixtures and validators are expressly deferred to R9. Before such fixtures are accepted,
they must cover one complete valid environment, one complete valid raw result, every invalid rule code, a correction chain/fork, and
every explicit missing state.

## 3. BLK-025 — artifacts, provenance, and retention

### 3.1 Layout and immutable manifests

Small reviewable evidence uses this repository-relative layout only after an authorized run:

```text
artifacts/EXP-0001/
  records/<first-two-id-chars>/<record-id>.json
  manifests/<first-two-id-chars>/<manifest-id>.json
  objects/sha256/<first-two-digest-chars>/<full-digest>
  tombstones/<first-two-id-chars>/<record-id>.json
```

Large, sensitive, storage images, traces, or impractical binary evidence remains external at an approved durable provider. The
repository stores its immutable manifest under `artifacts/EXP-0001/manifests/`; no external object is referenced only by a mutable URL.
This layout is a record contract, not authorization to create generated evidence in R7.

Each artifact descriptor contains: immutable `artifact_id` (typed UUIDv4), immutable URI, SHA-256 digest, exact byte size, registered
media type, producer role/tool/version/build identity, semantic role, created instant, retention class, sensitivity/redaction state,
and optional supersession/tombstone reference. The digest is over stored bytes after any declared compression/encryption; a separate
descriptor and digest identify logical unpacked bytes when those bytes are interpreted. Directory evidence is a manifest of sorted
relative names, entry media types, sizes, and digests—never a digest of filesystem metadata.

Artifact roles are `environment-capture`, `source`, `build-input`, `build-output`, `workload-manifest`, `workload-stream`,
`configuration-requested`, `configuration-effective`, `adapter`, `platform-contract`, `raw-samples`, `counter-stream`, `trace`, `log`,
`fault-plan`, `fault-trigger`, `controller-ledger`, `storage-image`, `recovery-output`, `validator-input`, `validator-output`,
`raw-result`, `derived-analysis`, `redaction-map`, or a future versioned role.

### 3.2 Typed provenance graph

Nodes are immutable records or artifacts. Every edge has an `edge_id`, one allowed type, source ID, target ID, producer, capture time,
and optional reason. Direction reads “source relation target.” Allowed edge types retain the logical contract vocabulary:
`generated-by`, `captured-from`, `configured-by`, `derived-from`, `validated-by`, and `supersedes`.

```text
repository/source --generated-by--> build output --configured-by--> effective configuration
       workload manifest --generated-by--> workload stream
 environment record --captured-from--> host/platform capture
 run/raw result --configured-by--> environment + build + workload + adapter + platform contract
 run/raw result --captured-from--> raw samples + counters + logs
 run/raw result --configured-by--> fault plan --generated-by--> fault trigger/controller ledger
 recovery output --derived-from--> preserved storage image + controller ledger
 run/raw result --validated-by--> validator output
 derived analysis --derived-from--> enumerated raw-result identities
 correction --supersedes--> prior record/artifact
```

The graph must give each run paths to its environment; source/repository commit and build outputs; workload manifest and stream;
requested/effective subject, baseline, adapter, orchestration, and platform configurations; raw results/samples; validator inputs and
outputs; applicable fault plan, trigger, ledger, storage image, and recovery output; and every derived analysis. An inapplicable fault
path is an explicit state in the run, not a fabricated edge.

Substitution is detected when an ID resolves to a different descriptor, URI resolves to bytes of the wrong size/digest, manifest entry
is absent/extra, media type/role differs, or graph edge endpoint does not match the referenced identity/digest. Missing artifacts are
reported as `missing`, inaccessible as `unavailable`, digest mismatch as `substituted-or-corrupt`, and an undeclared replacement as
`invalid`. None can be silently repaired from another run.

### 3.3 Retention, redaction, deletion, and authority

Retention classes are `repository-permanent`, `external-permanent`, and `external-time-bounded`. Time-bounded evidence records the
reviewed expiry instant, provider, replication policy, owner role, and reproducibility impact. Raw records, manifests, validation
findings, negative/failing results, and correction/tombstone chains are permanent. Destructive retention of canonical product history
is outside this apparatus and remains prohibited absent future authority.

Redaction occurs before publication, creates a new artifact/record and digest, links `derived-from` and (when replacing a published
item) `supersedes`, and preserves the restricted original when policy permits. The redaction map records field/byte range, stable
pseudonym policy, reason, access class, and reproducibility impact; it must not expose the secret in the public graph.

Deletion is allowed only for legal/security/provider necessity. A signed-or-reviewed tombstone descriptor retains identity, prior
digest/size/media type/role, deletion time, reason category, authority role, affected analyses, and reproducibility impact. Deletion
never makes a result pass and never erases its existence. Loss before declared retention expiry is an apparatus failure.

Benchmark evidence informs later decisions only through the evidence/ADR process. No artifact, controller ledger, recovered image,
materialization, summary, or benchmark database becomes canonical product authority or repairs canonical history.

## 4. BLK-021 — instrumentation and overhead

### 4.1 Categories, scopes, clocks, and placement

| Category | Required scope and placement | Clock/source |
|---|---|---|
| Harness lifecycle | Per operation/group at system acceptance, durability-boundary completion, canonical commit, acknowledgement return, observation, and materializer availability | Run-relative monotonic clock selected by R3/R4; canonical system time remains separate |
| Subject/baseline counters | Process/thread/operation scope at adapter entry/exit and declared persistence calls | Same monotonic interval clock plus native counters |
| OS/resource | Process/cgroup/CPU/storage-device scope, sampled outside hot path where possible | Declared monotonic sampling clock; CPU clocks remain labeled CPU time |
| I/O/synchronization | Requested/completed/failed operations at adapter, syscall, filesystem, and observable block/device layers | Layer-native counters correlated to monotonic intervals; correlation is not assumed causation |
| Fault/recovery | Controller and subject clocks around arm, trigger, observed effect, restart, scan, replay, and ready | Both clocks recorded; synchronization/offset/uncertainty bounded |
| Correctness/oracle | Out-of-band ledger, recovered set/order, validator checks | Ordering evidence and recorded observation times; never inferred canonical time |
| Profiling/tracing | Named thread/process/kernel scope and declared sample/trace points | Instrument-native clock with documented conversion and uncertainty |
| Environment/thermal | Host/device scope before, during, and after run | Source-specific sampling time |

System acceptance, durability-boundary completion, canonical commit, acknowledgement observation, generic observation, and materializer
availability are distinct events. The primary latency endpoints are declared before execution. A timestamp at one event cannot stand in
for another. Wall/realtime clocks date records; monotonic clocks measure run intervals; CPU clocks measure consumed CPU; instrument
clocks require a documented mapping.

Every run records instrument category, name/version/build, scope, placement, configuration, privilege, sampling frequency or event
mode, enable/disable time, clock, resolution, calibration artifact, output artifact, and enabled state. `disabled`, `unsupported`, and
`unavailable` are not zero. Raw samples/counter streams are immutable artifacts; summaries and quantiles are `derived-analysis` with
algorithm/version and enumerated source identities.

### 4.2 Loss, failure, and interpretation

Each stream records expected, emitted, received, parsed, dropped, overwritten, late, and rejected samples/events where knowable, plus
buffer capacity and loss signals. Unknown loss is an explicit state. Loss affecting correctness placement makes correctness
`inconclusive` or the run `invalid`; loss affecting a required performance population makes performance uninterpretable. Noncritical
resource loss may preserve correctness while making only that metric unavailable. A zero drop counter is evidence only when the
instrument defines and successfully reads it.

Instrument failure classes are: `configuration-error`, `permission-denied`, `unsupported`, `clock-discontinuity`, `clock-mapping-uncertain`,
`buffer-overflow`, `sample-loss`, `parse-error`, `counter-wrap`, `counter-reset`, `subject-perturbation`, and `output-integrity-failure`.
They are retained in `execution_observations` and validation findings.

Correctness validity and performance interpretability are independent:

| Correctness | Instrumentation | Permitted interpretation |
|---|---|---|
| pass | required streams valid; overhead bounded | correctness-valid and performance-interpretable |
| pass | performance stream/overhead invalid or unavailable | correctness-valid; performance not interpretable |
| fail | any | correctness fail; performance diagnostic only |
| inconclusive/invalid | any | no correctness or durability claim; performance diagnostic only |

### 4.3 Calibration and overhead bound

For each instrumentation bundle, use paired, interleaved calibration runs with the identical subject, workload, environment, run
length, preparation, and measurement path: bundle disabled versus enabled. Preserve both as distinct configurations in one calibration
series. Perform warm-up separately. Measure wall/monotonic elapsed, throughput numerator, raw latency population, CPU, I/O, memory,
and drops for both states. Also run the instrument against a no-op/idle control where meaningful to measure fixed sampling and output
cost. Randomize or balance pair order under the later R8 plan.

Report paired absolute differences and ratios with the later predeclared estimator/interval; report clock-read cost separately by
repeated back-to-back reads and timer resolution; report buffer/serialization cost from emitted bytes/events and CPU; and state which
parts are directly measured versus conservatively bounded. No numeric acceptance threshold is selected here. Until R8 defines a
cell-specific limit and the observed interval satisfies it, performance results using that bundle are descriptive only. Correctness
runs may remain valid if placement and oracle behavior pass and perturbation does not alter the boundary under test.

Calibration is repeated for every materially different bundle, rate, scope, privilege, clock, platform, or subject configuration.
Instrumentation cannot be subtracted post hoc unless the predeclared analysis explicitly authorizes it; raw enabled measurements and
calibration evidence remain preserved.

## 5. BLK-022 — fault mechanisms

Mechanism labels state `injected`, `simulated`, `virtualized`, or `physical` exactly as the recovery contract requires. The apparatus
must be tested first against a sacrificial non-benchmark target with observable markers before it can produce admissible runs.

| Fault class | Proposed mechanism and lifecycle placement | Required evidence and recovery | Limits, contamination, and safety |
|---|---|---|---|
| Process termination | Out-of-process controller sends an uncatchable termination to the subject PID/cgroup at a named R3 lifecycle token; control path is isolated from subject | Arm/trigger/receipt timestamps, PID/start identity, lifecycle ledger, exit status, surviving storage image; preserve image then use declared restart/scan/replay oracle | PID reuse, graceful handlers, controller colocation, delayed scheduling, and surviving child processes invalidate placement; verify with sacrificial process; kill only scoped cgroup |
| OS/kernel crash | Dedicated control host invokes a documented kernel panic/watchdog or hard VM reset at the named token | Trigger console/control-host log, boot identity before/after, volatile-layer declaration, preserved disk image where possible, then declared recovery | VM reset is `virtualized`, not physical kernel/device evidence; remote storage/control survival can contaminate; dedicated host, console access, filesystem checks, and cleanup required |
| Power loss/reset | Remotely controlled power device or hardware reset cuts the complete subject power domain after controller token | Independent power-controller log, before/after boot IDs, device/cache/PLP topology, controller ledger outside domain, untouched medium image, then recovery | Reset or remote outlet success does not prove capacitor discharge or volatile-cache loss. R4 lacks verified PLP/controller evidence, so power-loss results remain limited/inconclusive for unsupported D2/D3 claims; prevent shared-host/data damage and verify emergency restore |
| Partial/torn/truncated storage | On a copy or disposable device, deterministic block/file mutation or short-write shim targets declared R5 record offsets/stages; direct mutation is `injected`/`simulated` | Before/after byte images, offset/length/mask, digests, framing/CRC expectation, mutation tool identity, then fail-closed scanner/recovery twice | Does not reproduce a physical power fault; copy-on-write, sparse allocation, checksumming, compression, cache, wrong offset, or post-fault repair can mask it; never mutate sole evidence or canonical data |
| Explicit I/O/sync error | Narrow adapter/syscall/device fault hook returns a named short write/error at submission, write, flush, or sync boundary | Armed rule, invocation count, requested/returned bytes, exact error, call trace, lifecycle/ack evidence, stored image, recovery | Hook scope/order may differ from real hardware; retry or library interception can mask it. Validate hit/no-hit controls and fail closed on unexpected calls; remove hook and verify clean configuration after run |

For every mechanism the fault plan fixes: target mode/profile, lifecycle point and before/during/after placement, trigger token and
timeout, controller topology, seed/repetition, expected observable effect, promised/excluded fault class, evidence list, recovery procedure,
contamination checks, apparatus self-test, abort path, access boundaries, backup/restore, cleanup, and responsible role. “During” requires
evidence that the operation began and had not completed; timing alone is insufficient.

Apparatus validation uses positive controls (the intended target is affected), negative controls (unrelated targets and unarmed runs are
not), boundary markers, trigger latency/uncertainty measurement, repeatability checks, evidence-integrity checks, and cleanup verification.
A changed filesystem, repaired image, reused database, lingering injector, lost controller ledger, or clock discontinuity is contamination
and starts a new run after cleanup.

Coverage is recorded per lifecycle point × mechanism × mode/profile as exactly one of:

- **pass:** declared mechanism and placement occurred, complete evidence exists, and every applicable oracle invariant passed;
- **fail:** mechanism/placement occurred and a promised correctness invariant failed;
- **invalid:** placement/fault did not occur as declared, evidence is missing, safety boundary was violated, or contamination occurred;
- **inconclusive:** attempt is established but the mechanism cannot establish the claimed physical boundary;
- **not-tested:** outside the approved plan, never a pass.

Explicit errors and partial-condition injection test handling paths. Process kills test process loss. Kernel crashes test the recorded OS
path. Virtual reset tests its virtual boundary. Only a physical method with the complete verified platform path can support a narrowly
worded physical power-loss claim, and even then only for that configuration.

## 6. Constraint on BLK-020 — future harness boundaries

R7 freezes roles and interfaces, not implementation architecture:

1. **Workload source** emits an immutable manifest/stream identity without knowing a baseline API.
2. **Lifecycle controller** assigns run coordinates and records acceptance through materializer events without deciding canonical state.
3. **Subject adapter** maps the common event/lifecycle contract to one frozen R5/R6 profile and exposes requested/effective configuration.
4. **Instrument collector** emits raw, scoped, clocked streams plus loss/accounting evidence; it does not summarize in place.
5. **Fault controller** runs outside the target fault domain where required and exchanges named arm/trigger/observed tokens.
6. **Oracle/recovery evaluator** consumes preserved system bytes and the independent ledger; it cannot modify or synthesize canonical history.
7. **Record assembler** creates complete immutable logical records and provenance edges without inventing missing values.
8. **Validator** deterministically validates bytes, references, graph closure, and cross-field invariants before publication.
9. **Artifact store** provides immutable put/resolve/digest/tombstone behavior and never becomes product authority.
10. **Analysis consumer** reads validated raw identities and emits derived artifacts without replacing raw samples.

Interfaces carry versioned identities, typed references, explicit states, lifecycle tokens, clock metadata, and error classifications.
No role may silently share mutable configuration, infer a stronger durability mode, overwrite evidence, or treat a materialization as
canonical. R9 may choose a workspace, crate, executable, CLI, and CI arrangement only after its own authorization gate.

## 7. Resolution and remaining limitations

R7 resolves BLK-010, BLK-021, BLK-022, and BLK-025 as documentation design and constrains BLK-020 only at role/interface boundaries.
No validator, fixture, store, instrument, injector, recovery tool, or harness exists; no apparatus has been calibrated or validated; no
artifact has been captured; and no fault or benchmark has run. SHA-256/JCS choices here concern benchmark evidence and do not select
canonical-event integrity or the still-open workload serialization/digest choices.

BLK-015 remains open: final placement, complete storage-path protection, PLP/controller evidence, and empirical survival are absent.
BLK-006–010 implementation dependencies, BLK-023/024 R8 matrix/statistics, BLK-026 toolchain, BLK-027 bootstrap, and R9 architecture and
authorization remain open. Exact external provider, retention durations for time-bounded artifacts, signing/approval mechanism, complete
executable schemas, and numeric overhead acceptability are deliberately deferred to their owning review. These limitations prohibit
implementation, descriptive/confirmatory execution, and all durability/performance conclusions.
