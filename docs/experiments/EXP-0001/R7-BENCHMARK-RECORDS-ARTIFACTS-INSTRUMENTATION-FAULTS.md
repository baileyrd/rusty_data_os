# EXP-0001 R7 — Benchmark records, artifacts, instrumentation, and faults

**Status:** Complete as documentation design, with the owner-dependent apparatus
limits in section 9 retained; no implementation, execution, or evidence exists
**Record profile:** `EXP1-R7-JSON-JCS-1`
**Decision date:** 2026-08-27
**Evidence class:** experiment-local design derived from R1–R6 and official
interface documentation; it makes no performance, durability, or apparatus claim

## 1. Scope, authority, and invariants

This R7 authority freezes the non-executable record, artifact, instrumentation,
and fault design needed to constrain R9. It resolves BLK-010, BLK-021, and
BLK-025 as documentation design. It resolves the process-termination portion of
BLK-022 and freezes a fail-closed matrix for the remainder; concrete kernel
crash, machine-power/reset, and storage-error apparatus remain blocked by
BLK-015 and the owner approvals named in section 9. BLK-020 is constrained, not
implemented. R8 retains all numeric thresholds, primary cells, estimators,
repetitions, stopping, multiplicity, and run order.

These records are evidence about an experiment, never project truth or recovery
input. Canonical history alone is authoritative. A recorded command, attempt,
candidate, native database item, recovered byte, or observation never becomes a
canonical accepted fact by being recorded. Effective, system, durability,
observation, sequence/replay, monotonic lifecycle, and wall-clock time remain
distinct. D0/D1 remain provisional; D2/D3 require their declared platform
contract and later fault evidence. Materializations and database/WAL state are
derived and rebuildable. Missing, substituted, corrupt, lossy, contaminated, or
untraceable evidence fails closed as invalid or inconclusive.

## 2. Selected physical record profile

### 2.1 Serialization and canonical bytes

Every environment, raw-result, manifest/provenance, fault-plan/outcome, and
validation-report record is one UTF-8 JSON value conforming to I-JSON and
canonicalized with JSON Canonicalization Scheme (JCS), RFC 8785. The media type
is `application/vnd.rusty-data-os.exp1-r7+jcs`; a `record_kind` parameter is not
used. Stored record bytes are exactly the JCS bytes with **no BOM, leading or
trailing whitespace, or newline**. JSON text sequences, JSON Lines, comments,
NaN, infinities, duplicate member names, and lone Unicode surrogates are
rejected. Member order is JCS lexicographic order; strings use JCS escaping and
numbers use its ECMAScript finite-number serialization. Integers in this profile
must be JSON numbers in `0..9_007_199_254_740_991`; larger signed or unsigned
values, including signed 64-bit nanosecond timestamps, are canonical decimal
strings matching `0|-[1-9][0-9]*|[1-9][0-9]*` with no `+`, leading zero, decimal,
or exponent.

All objects are closed: an unknown member, unknown `record_kind`, or unsupported
`schema_version` is `unsupported-version`, never ignored. Required nullable
fields are present and use a missing-state object; absence is permitted only
where the field table says `optional`. Duplicate identities, paths, artifact
roles, manifest members, lifecycle observations, or conflicting facts are
`duplicate-or-conflict`.

The common envelope is:

| Name | Type and rule |
|---|---|
| `schema_version` | Required string, exactly `EXP1-R7-JSON-JCS-1`. |
| `record_kind` | Required enum: `environment`, `raw_result`, `artifact_manifest`, `fault_plan`, `fault_outcome`, or `validation_report`. |
| `record_id` | Required lowercase UUID text in canonical `8-4-4-4-12` form; identity authority and assignment evidence are recorded, never inferred from content. |
| `series_id` / `run_id` | Required lowercase UUID; `run_id` is `not_applicable` only for a series manifest or series validation report. |
| `created_at_utc_ns` | Required signed-64 decimal string, Unix-epoch nanoseconds from an identified realtime source; observation metadata, never event system/effective/durability time. |
| `supersedes_record_id` | Required state: `{\"state\":\"not_applicable\"}`, `{\"state\":\"missing\",\"reason\":string}`, or `{\"state\":\"present\",\"value\":uuid}`. Self/cycle references are rejected. |
| `body` | Required closed object defined below. |

The only missing-state object states are `present`, `missing`, `not_applicable`,
`not_collected`, `unsupported`, and `redacted`. `present` requires `value`;
`missing`, `not_collected`, `unsupported`, and `redacted` require nonempty
`reason`; `redacted` also requires `sanitized_artifact_id`. `not_applicable`
permits no other member. A consumer must not coerce one state into another.

### 2.2 Closed physical field ledger

The normative, complete [R7 physical field ledger](R7-PHYSICAL-FIELD-LEDGER.md)
defines every envelope and nested member for all six record kinds, including
types and unions, integer ranges, units, enum domains, conditionality, array
ordering, and missing-state behavior. The following table is only a navigation
summary; it does not relax or extend that ledger.

The closed `body` objects physically cover the logical contracts.
Every `*_ns`, `*_bytes`, count, sequence, ordinal, and counter is a canonical
decimal string unless explicitly bounded below; rates are not stored in raw
records and are derived later. Enumerations use the exact lower-snake-case text
listed here. Maps are forbidden where repeated ordered entries could hide a
duplicate; those values use arrays sorted by the stated key.

| Kind | Required physical fields and exact rules |
|---|---|
| `environment` | Closed body members are `artifact_manifest`, `authority_revisions`, `baseline`, `build`, `capture`, `clocks`, `configuration_refs`, `cpu`, `data_locations`, `deviations`, `durability_contract_ref`, `fault_apparatus`, `host`, `instrumentation`, `memory`, `os`, `preparation`, `record_producer`, `redactions`, `repository`, `scheduler_security`, and `storage`; the ledger closes every nested object and named-fact domain. |
| `raw_result` | `profile_id`, `subject_id`, `baseline_id` state, `workload_manifest_artifact_id`, `environment_record_id`, `d_mode`, `ack_boundary`, `canonical_status`, `visibility`, `fault_contract`, `configuration`, `phase`, `sample_population`, ordered `operations`, `throughput_window`, `resource_counters`, `background_work`, `errors`, `recovery`, `correctness`, `equivalence`, `deviations`, and `artifact_ids`. Operations are sorted by `workload_ordinal` and carry separate request/event IDs, assigned sequence state, effective/system/durability/observation state, monotonic lifecycle points, acknowledgement, D3 membership/shared outcome, byte accounts, and error state. Latency and throughput use the same named acknowledgement boundary. Every logical RAW-RESULT-TEMPLATE field is represented directly or by a typed missing state. |
| `artifact_manifest` | `scope` (`series` or `run`), `publication_state` (`staged`, `published`, `superseded`, `expired`, `deleted`), `series_freeze`, ordered `artifacts`, and ordered `provenance_edges`. Artifacts sort by `logical_path`; edges sort by `(from_artifact_id,relation,to_artifact_id)`. Section 5 defines both structures. |
| `fault_plan` | `plan_version`, `profile_id`, `d_mode`, `fault_class`, `mechanism_label`, `mechanism`, `control_plane`, `lifecycle_injection_point`, `trigger`, `preconditions`, `promised_layers`, `excluded_layers`, `self_tests`, `contamination_controls`, `restart_recovery`, `oracle_obligations`, and `authorization_state`. No plan implies authorization. |
| `fault_outcome` | `fault_plan_record_id`, `armed_at_monotonic_ns`, `trigger_evidence`, `placement_class`, `observed_condition`, `apparatus_self_test`, `contamination`, `oracle_artifact_id`, `recovery_artifact_ids`, `classification`, `classification_reasons`, and `not_tested_cells`. Classification is exactly `pass`, `fail`, `invalid`, or `inconclusive`; unsupported cells are `not_tested`, never pass. |
| `validation_report` | `validator_identity`, `validator_version`, `validation_started_at_utc_ns`, `validated_record_id` state, `validated_artifact_id`, `byte_length`, `sha256`, `profile_checks` (ordered by check ID), `errors` (ordered by byte offset then code), and `outcome` (`valid` or `invalid`). Validation software does not exist yet, so future reports cannot be manufactured by this increment. |

R9 may implement the frozen ledger but may not choose, rename, omit, widen, or
collapse fields. Any needed change requires a reviewed profile revision.

### 2.3 Identity, digest, and byte domains

R7 selects SHA-256 as the experiment-level digest, emitted as exactly 64
lowercase hexadecimal characters. SHA-256 is selected because it is a stable,
widely implemented FIPS 180-4 algorithm; this is substitution detection, **not
authenticity, authorization, or secrecy**. It is distinct from R5 CRC-32C,
which detects corruption inside B1 event records.

The digest input is `domain || 0x00 || bytes`, with these ASCII domains:

| Use | Domain | Exact `bytes` |
|---|---|---|
| Record identity check | `rusty-data-os/exp1/r7/record/v1` | Stored JCS record bytes. `record_id` remains the assigned identity; the digest is its validation attribute. |
| Exact artifact | `rusty-data-os/exp1/r7/artifact/v1` | Entire artifact byte stream from offset zero through declared length, with no normalization. |
| Workload stream | `rusty-data-os/exp1/workload-stream/v1` | The R2 length-delimited immutable stream bytes once BLK-006/007 freeze them. This selects the algorithm/profile but does not resolve those generator inputs. |

The SHA-256 hand vector for an empty exact artifact is the digest of the 34-byte
ASCII prefix `rusty-data-os/exp1/r7/artifact/v1` followed by `00`:
`3f9f15cdbbecaf81c03fe5a8e6370d55ecaf74f055ef7a33cf323861059fcc2b`.
The prefix and byte count must be independently checked before R9 adopts the
vector. Exact artifact digests cover compressed/encrypted containers as stored;
their decoded content, when needed, is a separate artifact and provenance edge.

The complete fictional examples and independently recomputable record-domain
vector are in [R7 documentation examples](R7-PHYSICAL-RECORD-EXAMPLES.md).

BLK-008 is therefore **partially resolved**: the algorithm, output, domains, and
artifact/record coverage are frozen. It remains open on the workload-stream
bytes until BLK-006/007 select generator algorithms and stable vectors. BLK-010
is not ambiguous because its record and artifact domains are complete. The
workload manifest (BLK-009) may reuse this JCS profile only after its complete
field contract is separately reviewed; R7 does not silently resolve BLK-009.

### 2.4 Correction, validation, and failures

Published bytes are immutable. A correction is a new record and artifact with a
new identity, `supersedes_record_id=present`, a `corrects` provenance edge, and a
reason. The old object remains retained and discoverable. Conflicting concurrent
corrections are both retained and make the chain invalid until a later record
explicitly supersedes both through provenance edges. Deletion never permits ID
reuse.

Parsing stops before semantic validation. Failure codes are: `io`, `length`,
`utf8`, `json-syntax`, `duplicate-member`, `non-ijson`, `noncanonical`,
`unsupported-version`, `unknown-field`, `missing-field`, `type`, `range`,
`enum`, `ordering`, `duplicate-or-conflict`, `reference`, `digest`,
`supersession-cycle`, and `policy`. A validation report preserves every safely
discoverable error and byte offset, but the validator must not repair input.
Parser crash, resource exhaustion, or incomplete validation is an invalid
validation attempt and cannot yield `valid`.

The examples authority supplies complete conforming environment and raw-result
records, explicit missing and correction/supersession cases, and focused invalid
cases. They are documentation, not executable fixtures or generated evidence.

## 3. Validation evidence and publication gate

A staged object becomes `published` only after: exact bytes and length are
stable; its artifact digest is independently recomputed; its record validates;
all references resolve to objects with matching identity/length/digest; the
closed provenance graph passes section 5; required validation reports are
published outside the object being validated; and atomic publication completes.
Validation reports never validate themselves. A series manifest pins validator
source/build identity before execution, but R9/BLK-026 must authorize it.

The manifest avoids self-reference by excluding its own artifact entry and
digest. A separate immutable **publication descriptor** is a closed JCS control
object defined by the ledger. It is the atomic discovery pointer. Generation
zero has no predecessor; generation `n>0` must equal its predecessor plus one,
name that predecessor's digest, and replace only a descriptor for the same
series and scope. A stale, skipped, forked, wrong-scope, invalid, unauthorized,
or digest-mismatched replacement fails `policy` or `reference` and leaves the
last valid generation authoritative. Referenced manifests remain immutable.
Filesystem realization is not selected here.

## 4. Deterministic artifact layout and references

Logical paths are relative ASCII paths with no empty, `.`, `..`, backslash,
control, percent-encoded, or case-fold-ambiguous segment:

```text
exp-0001/series/<series-uuid>/
  series-manifest.jcs
  records/environment/<record-uuid>.jcs
  inputs/<artifact-uuid>/<role-name>
  runs/<run-uuid>/
    run-manifest.jcs
    records/{raw-result,fault-plan,fault-outcome,validation}/<record-uuid>.jcs
    artifacts/<artifact-uuid>/<role-name>
```

An immutable reference is `{artifact_id,uri,byte_length,sha256}`. `uri` is an
absolute, normalized `file:` or `https:` URI chosen at publication; query and
fragment are forbidden. URI alone is never identity. Relocation uses a new
manifest/reference while retaining the same artifact ID only when exact bytes,
length, and digest match. Broken, unauthorized, wrong-length, or wrong-digest
references fail validation; a mirror may substitute only exact bytes.

Git may contain only reviewed small JCS metadata, sanitized summaries, stable
documentation vectors, and publication descriptors. Raw operation samples,
traces, storage images, logs, private environment capture, and large artifacts
remain external and are pinned by length/digest. R7 selects no service, bucket,
host path, capacity, or cloud provider.

## 5. Manifest, provenance, retention, and redaction

Each artifact entry is a closed object:
`{artifact_id,logical_path,role,media_type,byte_length,sha256,uri,sensitivity,
retention_state,created_by_record_id,validation_report_ids}`. IDs are lowercase
UUIDs; media types are lowercase registered/vendor types without parameters
unless the profile freezes them; length is decimal; sensitivity is `public`,
`sanitized`, or `access_sensitive`; retention is `staged`, `published`,
`superseded`, `expired`, or `deleted`. Roles include `workload_manifest`,
`environment_record`, `raw_result`, `fault_plan`, `fault_outcome`,
`lifecycle_ledger`, `apparatus_capture`, `recovery_capture`, `validation_report`,
`sanitized_derivative`, and `interpretation`.

Edges are `{from_artifact_id,relation,to_artifact_id}` where relation is
`generated_from`, `validated_by`, `corrects`, `supersedes`, `sanitizes`,
`decodes_to`, or `interprets`. The directed graph must be acyclic except that a
validation report points to its target through a record field rather than a
reverse provenance edge. Every raw result reaches exactly one frozen workload
manifest, environment record, subject/profile configuration, lifecycle ledger,
and applicable fault evidence. Every interpretation reaches its raw records.
Every derivative reaches source bytes. A graph walk recomputes every length and
digest and rejects missing or conflicting nodes. Thus every required object is
immutable, referenced, validated, and substitution-detectable.

Before execution, the series manifest pins repository authorities; series,
workload and environment identities; subjects/baselines and exact profile
revisions; intended D modes and fault contracts; R8 matrix/statistical authority;
validator/instrument identities; artifact policy; and every permitted deviation.
An unpinned input starts a new series or makes the run invalid.

Evidence moves only `staged -> published -> superseded -> expired -> deleted`;
`published -> expired` is allowed when no correction exists. Failed, negative,
invalid, and inconclusive evidence follows the same policy and is never deleted
because it is unfavorable. `expired` makes bytes inaccessible only after the
predeclared retention obligation and dependency check. `deleted` requires a
separate immutable closed JCS **deletion-evidence control record** defined by the
ledger. It binds the authorizer, authorization artifact, approved scope, target
identity/length/prior digest, method, completion observation, and verification
result. Missing, invalid, mismatched, unauthorized, premature, or failed
evidence prohibits the transition; partial deletion is `inconclusive`, retains
the prior state, and requires a new attempt/evidence identity. Tombstones and
provenance remain. No transition rewrites an old manifest.

Redaction is never in-place. Capture forbids secrets, credentials, personal
hostnames/addresses, sensitive paths, and unsanitized machine/device IDs in Git.
A pseudonym is series-scoped, random, and non-reversible; its private mapping is
access-sensitive and external. A sanitized derivative gets new bytes/ID/digest,
uses `sanitizes`, enumerates transformations and removed fields, and cannot
replace raw evidence for a claim needing those fields. Secret discovery blocks
publication and invokes repository incident handling; it is not “fixed” by a
normal supersession that leaves the secret accessible.

## 6. Instrumentation and overhead design

### 6.1 Intended Fedora 44 sources and capture points

Instrumentation uses direct Linux interfaces, not a permanently selected Rust
crate: `clock_gettime(CLOCK_MONOTONIC_RAW)` for run-relative lifecycle points;
`clock_gettime(CLOCK_REALTIME)` only for UTC observation correlation;
`getrusage(RUSAGE_THREAD/RUSAGE_SELF)` for user/system CPU; `/proc/<pid>/statm`
and `/proc/<pid>/status` for memory; `/proc/<pid>/io` for process I/O accounting;
`statx`/`fstat` for file lengths; and tracefs events plus Linux `perf_event_open`
software/hardware counters for syscall, scheduler, block-I/O, page-fault, context
switch, CPU-cycle/instruction, and loss diagnostics. Allocations are unavailable
from a stable kernel API: an R9-approved allocator observer may supply allocation
counts/bytes, otherwise they are `unsupported`, never inferred from RSS.
SQLite statement/status APIs and RocksDB statistics/listeners expose
baseline-native checkpoint, WAL, compaction, flush, stall, and background work;
they are diagnostic and never replace common lifecycle measurements.

Capture points are immediately before/after validation, construction, sequence
reservation, persistence submission, synchronization, canonical commit,
acknowledgement delivery, visibility probe, group join/cut/shared outcome,
fault arm/trigger, restart, scan, classification, replay, and ready. Effective,
system, and durability values are semantic event fields from R3; observation UTC
is recorder metadata; sequence is order; monotonic points alone form lifecycle
durations. Wall time never substitutes for monotonic duration.

Throughput is accepted operations divided by the monotonic interval from the
first applicable submission to the last applicable acknowledgement at the same
boundary used for latency. CPU is user/system nanoseconds. Memory records
current and peak bytes without treating sampling as allocation. I/O records
logical/request, process-read/write, filesystem file-length change, block-layer
bytes/operations and sync calls separately; unavailable attribution is explicit.
Errors and background work are event records, not silently excluded samples.

### 6.2 Scope, clocks, loss, and lifecycle

Every source records process/thread/cgroup/CPU scope, unit, event or sampling
model, sampling period, counter width, initial/final raw values, enabled/running
time, multiplexing, and privilege state. Counters are read/reset before warm-up,
after warm-up, before measurement, and after measurement. Decrease is wrap only
when width and modulo prove it; otherwise it is reset/invalid. Per-CPU events
carry CPU and monotonic timestamp; total order is asserted only where the common
clock and capture point justify it.

Trace buffers are sized and pinned before a series. Per-CPU lost-event counters,
sequence gaps, perf `PERF_RECORD_LOST`, recorder queue high-water mark, write
errors, start/stop barriers, and expected sentinel events are retained. Any loss
on a correctness/lifecycle channel invalidates the run. Diagnostic-channel loss
makes that metric unavailable and may make the run invalid when R8 declares it
primary; it is never treated as zero. Start source -> verify sentinel -> warm-up
-> reset/snapshot -> barrier -> measure -> barrier -> snapshot -> sentinel ->
stop/drain -> validate. Shutdown timeout or undrained buffers is invalid.

Environment evidence records clock IDs, `clock_getres`, kernel clocksource,
realtime synchronization state/offset source if available, perf availability and
paranoid settings, tracefs mount/access, counter support, multiplexing, CPU
affinity/frequency policy, and observer versions. The R4 1 ns resolution is not
an accuracy claim. Cross-host UTC comparisons are unsupported without later
synchronization evidence.

### 6.3 Calibration and perturbation

Before subject runs, measure recorder-only empty-loop timestamp pairs, event
emission/drain, counter read, trace sentinel, and storage-free lifecycle traffic
using the frozen build/environment, with warm-up and raw distributions retained.
Then use paired randomized `instrumentation_off`/`instrumentation_on` repetitions
with identical frozen streams and configuration. “Off” retains only the minimal
external start/end and correctness ledger required to establish the run. Report
absolute on/off latency, throughput, CPU, memory, I/O, losses, and paired
differences/ratios. Never subtract calibration or perturbation from subject
results. R8 alone sets repetitions, ordering, estimators, intervals, and numeric
accept/reject thresholds. Pair mismatch, configuration drift, loss, counter
multiplexing beyond the future declared allowance, or observer-induced semantic
boundary change invalidates the pair.

## 7. Fault taxonomy and apparatus matrix

An injected mechanism and resulting condition are separate fields. `SIGKILL`
does not model kernel crash; a virtual reset does not prove physical volatile
cache loss; a directly corrupted/truncated copy does not prove any fault cause.
No destructive action may occur outside dedicated disposable experiment data,
and every future destructive run requires separate owner authorization.

| Fault / profiles | Intended mechanism and control plane | Lifecycle placement and evidence | Preconditions/self-test | Outcome/oracle |
|---|---|---|---|---|
| Process termination; B0 D0, B1 D1/D2/D3, SQLite D1/D2, RocksDB D1/D2 | External controller opens a pidfd for the exact subject PID and invokes `pidfd_send_signal(..., SIGKILL, ...)`; fallback `kill(2)` is prohibited because PID reuse weakens identity. | Every applicable R3/R5/R6 point from validation through acknowledgement; controller ledger records pidfd identity, armed point, trigger receipt, and absence of a later subject sentinel. | Disposable run, controller outside subject process, successful harmless signal-0 identity check, sacrificial-process kill/reap self-test, no shared service PID. | D0 loss permitted; D1 only its declared process-loss promise; D2/D3 exact-candidate reconciliation, must/may/must-not oracle and D3 per-member/shared outcome. |
| Kernel/OS crash; persistence profiles | Linux Magic SysRq `c` through a separately authorized dedicated target, or another owner-approved equivalent that demonstrably crashes the target kernel. **No mechanism is selected for execution yet.** | Boundary trigger from an out-of-fault-domain controller; pre-crash ledger and controller-side trigger evidence. | BLK-015/final path, dedicated host approval, privilege/security review, SysRq availability and a disposable non-data self-test plan. | Covered only if the platform contract promises it; otherwise inconclusive/not tested. Restart read-only preservation then deterministic recovery. |
| Machine reset / physical power loss; persistence profiles | **Owner-selected external physical power/reset apparatus is required.** Software reboot, SysRq, VM reset, and remote management without proven power-domain/cache effect are non-equivalent. | Same lifecycle matrix, with controller timestamp, commanded/observed power state, and affected volatile-layer evidence. | BLK-015, owner-named disposable machine/path, power controller and authority, storage/cache/PLP facts, recovery access and safety plan. | Only proven affected layers may support D2/D3. Otherwise inconclusive; never claim physical cache loss from virtual reset. |
| Partial/torn/truncated resulting conditions; B1 and recovered baseline artifacts | Offline, byte-exact mutation of a copy of dedicated experiment data according to R5 boundary cases; original preserved. This directly injects a condition, not a crash. Native SQLite/RocksDB file mutation is diagnostic unless their official recovery contract and exact mutation plan are separately reviewed. | Each R5 header/body/final/commit boundary and prefix/suffix/bit-change case; before recovery only. | Stable input digest, declared offsets/bytes, mutation digest, read-only original, validator self-test with known vectors. | Detect/classify complete, terminal partial, corrupt, duplicate, gap, conflict, or undecidable; fail closed and repeat recovery twice. |
| Explicit I/O/sync error; B1/SQLite/RocksDB | Kernel/device-mapper or filesystem fault facility must be selected after final storage topology and privilege review. Application mocks cannot establish kernel/device error behavior. **No concrete target mechanism is approved.** | Submit/write/sync/finalize/commit and database-native error-return points; record requested versus observed error and completed bytes. | BLK-015/final path, isolated disposable device, privilege/security approval, non-target isolation, reversible teardown, read/write/sync self-tests. | Error cannot yield canonical success; uncertain physical result uses exact-candidate reconciliation and fails closed. Unsupported injection is not tested. |

For every row the plan records setup/teardown hashes, apparatus version, control
and target fault domains, trigger latency/placement bounds, contamination of
timing/storage, and one no-fault control. Apparatus validation is repeated after
configuration change. A missed/unproved placement is invalid; a proved trigger
that cannot establish the promised layer is inconclusive. Recovery preserves an
untouched image where practical, restarts the selected native path, validates
R5/native records, reconciles exact candidates without inventing facts, checks
must/may/must-not classes and D3 membership/shared outcome, and repeats scan and
replay twice. Performance from correctness failure is inadmissible.

## 8. Documentation-only harness constraints

The dependency graph is acyclic:

```text
frozen series manifest -> deterministic producer -> subject/baseline adapter
                                              |                 |
                                              v                 v
out-of-band controller/oracle -> lifecycle recorder      native result
              |                       |                         |
              v                       v                         v
       fault controller ------> recovery oracle ------> record validators
                                                             |
                                                             v
                                                    artifact publisher
```

The producer owns immutable intended operations, not canonical outcomes. An
adapter owns only its mapping and exposes lifecycle transitions. The recorder is
append-only observation and cannot call commit. The out-of-band oracle consumes
pre-fault evidence and recovered output but cannot repair subject state. The
fault controller cannot publish a pass. Validators are pure consumers of frozen
bytes. Only the publisher may move validated staged objects to an immutable
publication generation; it cannot edit them. Subject/baseline-specific modules
depend inward on common contracts; common lifecycle, oracle, record, and artifact
contracts never import a database adapter. R9 must preserve these boundaries and
freeze concrete processes, IPC, queueing, ownership, backpressure, and failure
propagation before executable BLK-020 is resolved.

## 9. Exact residual blockers and disposition

R7 is complete at its honest documentation boundary. BLK-010/021/025 are
resolved as design. BLK-008 is closed only for record/exact-artifact algorithm
and domains; its workload-stream input awaits BLK-006/007. BLK-022 remains open
for kernel crash, physical reset/power loss, and explicit storage/I/O errors
because the repository has no owner-approved final path, PLP/cache fact, power
controller, disposable block device, privilege/security decision, or destructive
fault authority (BLK-015; UNK-014/015/021). Process termination and offline R5
condition injection are exact documentation selections but unimplemented.

BLK-020 remains constrained, not executable. BLK-009, BLK-015, BLK-023/024,
BLK-026/027, implementation, effective validation, execution, and evidence stay
open. UNK-022 is resolved as physical records, digest, artifact policy, and
instrumentation design, but automated validation remains open. UNK-018/019 retain
the generator/workload-manifest dependencies above. R8 is next and may freeze
only primary matrix/statistics/thresholds; it cannot authorize code or faults.

The owner must provide, in one reviewed future authority: final non-sensitive
storage topology and placement; exact volatile-cache/PLP facts; named disposable
target and isolation boundary; approved privileges/security posture; exact
kernel-crash, power/reset, and I/O-error apparatus with controller identity; and
separate destructive-execution authorization. Until then those matrix cells are
`not_tested`, never complete or passed.

## 10. Decision rationale, rejected alternatives, and sources

JCS was selected over ad hoc JSON/JSON Lines (ambiguous whitespace/order and
record boundaries), YAML/TOML (no selected canonical byte profile), and CBOR
(unnecessary binary tooling at this documentation stage). SHA-256 was selected
over CRC-32C (already scoped to accidental event-record corruption), SHA-1
(obsolete security profile), and a new/unavailable digest. This does not turn an
unkeyed digest into authenticity. Direct kernel/process interfaces were selected
over a single profiler so scope, drops, clocks, and unsupported metrics remain
explicit. pidfds were selected over PID-only signaling to bind process identity.
No power or device-error apparatus was guessed.

Official primary sources consulted on 2026-08-27 (publication/update identity is
the date/version displayed by the source where supplied):

- [RFC 8785, *JSON Canonicalization Scheme*](https://www.rfc-editor.org/rfc/rfc8785),
  June 2020, and [RFC 7493, *I-JSON*](https://www.rfc-editor.org/rfc/rfc7493),
  March 2015: serialization, duplicate/number/Unicode constraints.
- [NIST FIPS PUB 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final), August
  2015: SHA-256 definition (NIST announced a future revision, which does not
  alter this pinned profile).
- Linux man-pages 6.15 pages for
  [`clock_gettime(2)`](https://man7.org/linux/man-pages/man2/clock_gettime.2.html),
  [`getrusage(2)`](https://man7.org/linux/man-pages/man2/getrusage.2.html),
  [`perf_event_open(2)`](https://man7.org/linux/man-pages/man2/perf_event_open.2.html),
  [`proc_pid_io(5)`](https://man7.org/linux/man-pages/man5/proc_pid_io.5.html),
  [`proc_pid_statm(5)`](https://man7.org/linux/man-pages/man5/proc_pid_statm.5.html),
  [`statx(2)`](https://man7.org/linux/man-pages/man2/statx.2.html), and
  [`pidfd_send_signal(2)`](https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html):
  clock, resource, counter, process-I/O/memory, file, and identity-bound signal
  interfaces.
- Linux kernel documentation for
  [tracefs/ftrace](https://docs.kernel.org/trace/ftrace.html),
  [perf ring buffers](https://docs.kernel.org/next/userspace-api/perf_ring_buffer.html),
  [procfs](https://docs.kernel.org/filesystems/proc.html),
  [Magic SysRq](https://docs.kernel.org/admin-guide/sysrq.html), and
  [device-mapper delay/flakey targets](https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html):
  capture/loss and candidate fault-interface semantics. Their availability does
  not grant privilege or select a safe target.
- SQLite 3.53.4 C API/status and WAL/PRAGMA documentation and RocksDB 11.8.1
  Statistics/Listeners/WAL documentation, as pinned by R6: native diagnostic
  counters and background-work interpretation.

These sources define interfaces, not measured availability, overhead, Fedora 44
behavior, storage survival, or apparatus correctness. Source identity and access
date must be copied into the future series manifest and revalidated against the
pinned Fedora kernel/userspace before execution.

## 11. Traceability and completion report

This authority traces to BLK-008/010/015/020/021/022/025 and retained
BLK-023/024/026/027; UNK-014/015/018/019/020/021/022; RQ-003; REQ-001–010 and
REQ-012–014; ADR-0002; EXP-0000 record, durability, recovery, and interpretation
contracts; benchmark methodology; and R1–R6. It changes project knowledge only
as documentation design. It creates no schema, validator, harness, workflow,
installation, machine/storage action, fault, benchmark, result, R8 threshold, or
R9 authorization.
