# R33 — R32 Adapter Closure and Target-Preflight Decision

**Status:** Complete documentation/governance decision; exactly one later synthetic-tested target-preflight implementation PR is prospectively authorized
**Scope:** EXP-0001 preflight subset for a prospective Fedora 44 Linux/x86_64 target
**Evidence classification:** bounded deterministic adapter correctness evidence plus prospective preflight design; no Fedora-release validation, live observation, execution, or performance evidence
**Authority date:** 2026-09-04

## 1. Exact closure and bounded next step

PR #111 was reviewed at exact head `71f58f65772fea2f0f58f5727d42e1405c7f09fb` and merged as
`05dd7cc0980df2914dff5814ab5f5fba5b8e09e0`; both exact-head workflows succeeded. This closes
R32's internal adapter only as bounded deterministic adapter, ownership, lifecycle,
wrapper-mapping, failure, and cleanup correctness evidence. It is not target validation or live
evidence.

The sole next implementation entry point is:

```rust,ignore
pub fn run_target_preflight(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> TargetPreflightExecutionV1
```

It may be added only in
`experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/target_preflight.rs` and re-exported
from `lib.rs`. There is no binary, CLI, build script, automatic hook, background task, or live test.
Invocation and retained host observations remain unauthorized, including in CI.

## 2. Request and measured-file ownership

`TargetPreflightRequest` contains exactly a validated lowercase 40-hex `repository_revision`, a
nonempty `build_identity` of at most 128 ASCII graphic characters excluding slash and backslash,
fixed expected `Fedora44Linux` and `X86_64` enum values, a transient `measured_file_path`, and a
stable `measured_file_identity` matching `^[a-z0-9][a-z0-9._-]{0,63}$`. The identity forbids `/`,
`\\`, whitespace, `..`, URI schemes, and equality with any rendered path component. Invalid text
is rejected before open. A host path is never placed in an artifact, outcome, error, or buffer.

The caller opens only the pre-existing path with `OpenOptions` read enabled and write, create,
create-new, truncate, and append disabled. It performs no read, seek, write, sync, mutation, lock,
rename, or deletion. `metadata().file_type().is_file()` must be true. The caller owns the `File`;
the adapter merely borrows its `AsRawFd` capability and never owns, duplicates, transfers, or
closes the descriptor.

After all checks, the caller releases its owned `File` by ordinary Rust drop. The lifecycle fact
is exactly `measured_file_ownership_release:"drop_completed"`: control completed the drop
operation. It is **not** an observed kernel `close(2)` result and makes no claim that an OS close
error was captured. No `measured_file_close` field or close-success promise exists.

## 3. Permitted checks and platform disposition

The live path may only: validate inputs and compile-time `target_os`/`target_arch`; open and validate
the file; call `clock_resolution` for realtime then monotonic-raw; use one merged
`LiveCaptureBoundary` for realtime, monotonic-raw, process/thread `getrusage`, statm, status,
process I/O, and file length in that order; then independently open, stop/read, and clean CPU
cycles, instructions, page faults, and context switches in that order. Acquired owners are cleaned
before proceeding. No measured action or delta exists.

The platform disposition is `prospective_fedora_44_linux`: it records only the request's expected
target together with a compile-time Linux match. It is not an operator assertion accepted as an
observed Fedora release. Fedora release and effective-target validation remain unresolved; no new
OS-release probe is authorized. Consequently the strongest successful artifact classification is
`preflight_subset_ready`, never `ready` or `target_validated`.

Tracefs, other host sources, workloads/actions, append/replay or R7 production, calibration,
benchmarks, publication, D2/D3, durability/recovery/faults, baselines, and production expansion are
excluded.

## 4. Two-stage result and disposition contract

The implementation first constructs and serializes an immutable
`EXP-0001-R33/target-preflight-artifact-v1` containing only facts known before retention. Its
`classification` is exactly `preflight_subset_ready`, `blocked`, or `invalid`. Only after those
bytes are fixed does it attempt retention. The returned, non-serialized
`TargetPreflightExecutionV1` contains `{ artifact: Option<TargetPreflightArtifactV1>,
serialized_bytes: Option<Vec<u8>>, retention: RetentionOutcomeV1, disposition:
TargetPreflightCallDispositionV1 }`.

`disposition` is exactly:

- `request_invalid` when request validation failed before any file open, artifact construction, or
  serialization;
- `completed` when an artifact was serialized and one `write_all` plus one `flush` succeeded;
- `serialization_failed` when no bytes were offered to the sink; or
- `retention_failed` when `write_all` or `flush` failed, including a partial write hidden inside
  `write_all`.

For `request_invalid`, `artifact` and `serialized_bytes` are both `None`, retention is
`NotAttempted { reason: RequestInvalid(reason) }`, and `reason` is exactly one closed
`RequestFailureReasonV1`: `InvalidRepositoryRevision`, `InvalidBuildIdentity`, or
`InvalidMeasuredFileIdentity`.
These values serialize nowhere and contain no rejected text or path. Revision, build-identity, and
file-identity syntax therefore have the same pre-open outcome; there is no sanitized substitute in
an artifact. Artifact construction is permitted only after all request fields are valid.

Thus a successfully retained artifact can be `preflight_subset_ready`, `blocked`, or `invalid`,
while the call's final disposition independently reports post-serialization execution. A failed or
partial write/flush is observable in the returned outcome and never requires already-written bytes
to rewrite themselves. Retention failure does not retroactively alter artifact bytes or
classification.

`RetentionOutcomeV1` is exactly one of `NotAttempted { reason:
RequestInvalid(RequestFailureReasonV1)|SerializationFailure }`,
`Success { serialized_byte_length: u64 }`, or `IoFailure { operation: WriteAll|Flush,
error: RetentionIoErrorV1 }`. `RetentionIoErrorV1` is exactly `{ kind: IoErrorKindV1,
raw_os_error: Option<i32> }`: `kind` is the normalized `std::io::ErrorKind` string list frozen for
`ErrorReasonV1::Io` below (including `other`); every future or otherwise unlisted non-exhaustive
`ErrorKind` maps to `other`. `raw_os_error` is the unchanged result of
`io::Error::raw_os_error()` (`None` when absent). This mapping is exhaustive and mechanical for
every sink error; retention never maps a sink error to wrapper `Permission`, `UnavailableReasonV1`,
or `ErrorReasonV1`. A failed write reports `Flush` as not attempted inside the returned execution
bookkeeping; flush is attempted only after successful `write_all`.

## 5. Closed artifact schema and lossless merged-type mapping

The JSON object has the following required fields in order: `schema`, `repository_revision`,
`build_identity`, `platform`, `architecture`, `measured_file`, `sources`, `perf_events`,
`lifecycle`, `first_causal_failure`, `cleanup_failures`, `tracefs`, `classification`, and `reasons`.
Unknown, missing, or duplicate fields and unknown enum strings are invalid inputs to construction.
Arrays have the order stated below.

Every merged `Outcome<T>` maps losslessly to exactly one tagged object:
`{"success":T}`, `{"unavailable":UnavailableReasonV1}`, `{"permission":{"errno":i32}}`,
`{"overflow":OverflowReasonV1}`, or `{"error":ErrorReasonV1}`. `Unsupported` maps to
`{"unavailable":{"kind":"unsupported"}}`; it is not an outcome variant. A preflight-only skipped
operation uses `{"not_attempted":{"failure_id":string}}`, outside the merged wrapper mapping.

Reasons are closed tagged objects. `UnavailableReasonV1` is one of
`{"kind":"interface","errno":i32}`, `{"kind":"missing_statx_size"}`,
`{"kind":"not_found"}`, `{"kind":"unsupported"}`, or
`{"kind":"statx_only_after_fstat"}`. `OverflowReasonV1` is one of `arithmetic`, `file_size`,
`numeric_field`, `perf_scaling`, or `{"kind":"perf_errno","errno":i32}`. `ErrorReasonV1`
losslessly names every merged `ErrorReason`: `errno` with `errno`; `invalid_fraction`;
`negative_counter`; `negative_file_size`; `io` with `error_kind` equal to exactly one of
`not_found`, `permission_denied`, `connection_refused`, `connection_reset`, `host_unreachable`,
`network_unreachable`, `connection_aborted`, `not_connected`, `addr_in_use`,
`addr_not_available`, `network_down`, `broken_pipe`, `already_exists`, `would_block`,
`not_a_directory`, `is_a_directory`, `directory_not_empty`, `read_only_filesystem`,
`filesystem_loop`, `stale_network_file_handle`, `invalid_input`, `invalid_data`, `timed_out`,
`write_zero`, `storage_full`, `not_seekable`, `quota_exceeded`, `file_too_large`, `resource_busy`,
`executable_file_busy`, `deadlock`, `crosses_devices`, `too_many_links`, `invalid_filename`,
`argument_list_too_long`, `interrupted`, `unsupported`, `unexpected_eof`, `out_of_memory`,
`in_progress`, or `other`;
`invalid_utf8`; `parse` with one of `non_ascii`, `line_count`, `token_count`, `malformed_line`,
`missing_field`, `duplicate_field`, `signed_value`, `invalid_number`, `invalid_unit`, or
`trailing_token`; `perf_short_read` with signed `actual`; `perf_invalid_time`; `perf_decrease`;
`perf_lifecycle`; `perf_cleanup` with `errno`; `perf_unexpected_return` with signed `actual`;
`perf_cleanup_unexpected` with signed `actual`; `perf_event_mismatch` with `expected` and `actual`;
or `missing_file_capability`. Tagged reason objects always use `kind` first and the named detail
fields second.

Retained success values are exact: integer scalars remain signed/unsigned as in the merged type;
`ResourceUsage`, `Statm`, `StatusMemory`, and `ProcessIo` retain every same-named merged field;
`FileLength` is `{"bytes":i64,"source":"statx"|"fstat_fallback","statx_only_fields":
{"success":null}|{"unavailable":UnavailableReasonV1}}`; and `PerfCounter` is
`{"event":string,"raw_count":u64,"time_enabled_ns":u64,"time_running_ns":u64,
"multiplexed":bool,"scaled_count":Outcome<u64>}`.

Each source entry is `{"identity":string,"scope":string,"unit":UnitV1,"outcome":Outcome}`.
Source identity, scope, and unit use the merged R31/R32 values. The two resolution identities are
`clock_resolution_realtime` and `clock_resolution_monotonic_raw`, scope `observation`, unit
`nanoseconds`; remaining identities are `realtime`, `monotonic_raw`, `process_rusage`,
`thread_rusage`, `statm`, `status`, `process_io`, and `file_length`, with scopes respectively
`observation`, `observation`, `process`, `measured_thread`, `process`, `process`, `process`, and
`measured_file`. Units are `nanoseconds`, `statm_pages`, `bytes`, or closed objects
`resource_usage`, `process_io`, and `perf_counter` whose member units exactly mirror the merged
unit structs (`nanoseconds`, `bytes`, `events`, or `operations`). Perf entries retain `event`, scope
`measured_thread`, the complete perf-counter unit object, plus `open`, `stop_read`, and `cleanup`.

`lifecycle` contains ordered `phases` and
`measured_file_ownership_release:"drop_completed"`. It contains no serialization, retention, or
OS-close result. `first_causal_failure` is null or `{id,phase,source,class,detail}`;
`cleanup_failures` repeats that exact object shape. This is a closed `FailureObjectV1` encoding:

- `id` is `failure-` plus a four-digit, zero-padded positive decimal ordinal. Ordinals follow
  observation occurrence: the causal failure first when present, then cleanup failures in the
  fixed perf-event cleanup order below. With no causal failure, the first cleanup failure is
  `failure-0001`. IDs are contiguous and never reused or sorted by their text.
- `phase` is exactly one of `platform_validation`, `architecture_validation`,
  `measured_file_open`, `measured_file_regular_file`, `clock_resolution_realtime`,
  `clock_resolution_monotonic_raw`, `source_capture`, `perf_open`, `perf_stop_read`,
  `perf_cleanup`, or `ownership_release`. `source_capture` is used only for the eight non-resolution
  source identities in their frozen order. Cleanup failures are ordered by actual cleanup
  occurrence in the frozen independent-event order: CPU cycles, instructions, page faults, then
  context switches.
- `source` is exactly `platform`, `architecture`, `measured_file`, one of the ten source identities
  frozen above, one of the four perf event names, or `lifecycle`. The complete legal phase/source
  pairs are `platform_validation/platform`, `architecture_validation/architecture`,
  `measured_file_open/measured_file`, `measured_file_regular_file/measured_file`, each
  `clock_resolution_*` phase with its identically named source, `source_capture` with one of the
  eight non-resolution source identities, each of `perf_open`, `perf_stop_read`, and `perf_cleanup`
  with one of the four perf event names, and `ownership_release/lifecycle`. All other pairs are
  rejected at construction.
- `class` is exactly `unavailable`, `permission`, `overflow`, `error`, or `invalid_state`.
  For the first four, `detail` is respectively an `UnavailableReasonV1`, `{"errno":i32}`, an
  `OverflowReasonV1`, or an `ErrorReasonV1`, byte-for-byte the same reason object used by the
  corresponding frozen outcome mapping. `invalid_state` detail is exactly one of
  `{"kind":"platform_mismatch"}`, `{"kind":"architecture_mismatch"}`,
  `{"kind":"not_regular_file"}`, `{"kind":"lifecycle_violation"}`, or
  `{"kind":"ownership_release_incomplete"}`. No free-form detail is permitted.

The causal object is the first non-cleanup failure in check order and later dependent operations
are not attempted. `cleanup_failures` contains only cleanup failures in the deterministic order
above, independently of the causal failure. Tracefs is exactly
`{"state":"not_collected","reason":"R33 target preflight deliberately did not invoke tracefs"}`.

`preflight_subset_ready` requires all pre-retention validations and outcomes to succeed, ownership
release/drop to complete, no causal or cleanup failure, and the frozen tracefs state. `blocked`
covers unavailable or permission capability outcomes. `invalid` covers only failures after a valid request is representable: target OS/architecture
mismatch, non-regular file, error/overflow, lifecycle violation, or cleanup failure. Invalid
precedes blocked. Request-invalid and retention outcomes never participate in artifact
classification.

## 6. Deterministic serialization and retention

Use a dependency-free handwritten UTF-8 JSON-lines serializer: one object plus LF, field order as
above, no insignificant whitespace, shortest decimal integers, lowercase enum strings, required
JSON escaping, and rejection of other non-ASCII input. No parser or parse/validate/reserialize API
is authorized or required. Construction validates inputs fail closed before serialization.

The operator supplies an already-open sink. The implementation builds one complete in-memory
buffer, then performs one `write_all` and, only if it succeeds, one `flush`. It does not retry,
truncate, sync, rename, publish, claim atomicity/durability, or derive a path. A staged diagnostic
is not an R7/environment/raw-result/validation/benchmark record or evidence.

## 7. Byte-exact fictional vectors

The following single line, including its final LF, is the exact successful artifact vector (shown
in a fenced block whose content ends immediately after that LF). Values are invented, not observed.

```json
{"schema":"EXP-0001-R33/target-preflight-artifact-v1","repository_revision":"1111111111111111111111111111111111111111","build_identity":"fictional-build-01","platform":{"expected":"fedora-44-linux","disposition":"prospective_fedora_44_linux"},"architecture":{"expected":"x86_64","observed":"x86_64"},"measured_file":{"identity":"measured-file-alpha","open":{"success":null},"regular_file":{"success":true},"length":{"identity":"file_length","scope":"measured_file","unit":"bytes","outcome":{"success":{"bytes":4096,"source":"statx","statx_only_fields":{"success":null}}}}},"sources":[{"identity":"clock_resolution_realtime","scope":"observation","unit":"nanoseconds","outcome":{"success":1}},{"identity":"clock_resolution_monotonic_raw","scope":"observation","unit":"nanoseconds","outcome":{"success":1}},{"identity":"realtime","scope":"observation","unit":"nanoseconds","outcome":{"success":1000000000}},{"identity":"monotonic_raw","scope":"observation","unit":"nanoseconds","outcome":{"success":2000000000}},{"identity":"process_rusage","scope":"process","unit":{"resource_usage":{"user":"nanoseconds","system":"nanoseconds","maximum_resident":"bytes","minor_faults":"events","major_faults":"events","input_blocks":"operations","output_blocks":"operations","voluntary_context_switches":"events","involuntary_context_switches":"events"}},"outcome":{"success":{"user_nanoseconds":100,"system_nanoseconds":50,"maximum_resident_bytes":8192,"minor_faults":2,"major_faults":0,"input_blocks":0,"output_blocks":0,"voluntary_context_switches":1,"involuntary_context_switches":0}}},{"identity":"thread_rusage","scope":"measured_thread","unit":{"resource_usage":{"user":"nanoseconds","system":"nanoseconds","maximum_resident":"bytes","minor_faults":"events","major_faults":"events","input_blocks":"operations","output_blocks":"operations","voluntary_context_switches":"events","involuntary_context_switches":"events"}},"outcome":{"success":{"user_nanoseconds":80,"system_nanoseconds":40,"maximum_resident_bytes":8192,"minor_faults":1,"major_faults":0,"input_blocks":0,"output_blocks":0,"voluntary_context_switches":1,"involuntary_context_switches":0}}},{"identity":"statm","scope":"process","unit":"statm_pages","outcome":{"success":{"size":10,"resident":2,"shared":1,"text":1,"lib":0,"data":3,"dt":0}}},{"identity":"status","scope":"process","unit":"bytes","outcome":{"success":{"resident_bytes":8192,"high_water_bytes":12288}}},{"identity":"process_io","scope":"process","unit":{"process_io":{"rchar":"bytes","wchar":"bytes","syscr":"operations","syscw":"operations","read":"bytes","write":"bytes","cancelled_write":"bytes"}},"outcome":{"success":{"rchar":100,"wchar":0,"syscr":1,"syscw":0,"read_bytes":0,"write_bytes":0,"cancelled_write_bytes":0}}}],"perf_events":[{"event":"cpu_cycles","scope":"measured_thread","unit":{"perf_counter":{"raw_count":"events","time_enabled":"nanoseconds","time_running":"nanoseconds","scaled_count":"events"}},"open":{"success":null},"stop_read":{"success":{"event":"cpu_cycles","raw_count":10,"time_enabled_ns":10,"time_running_ns":10,"multiplexed":false,"scaled_count":{"success":10}}},"cleanup":{"success":null}},{"event":"instructions","scope":"measured_thread","unit":{"perf_counter":{"raw_count":"events","time_enabled":"nanoseconds","time_running":"nanoseconds","scaled_count":"events"}},"open":{"success":null},"stop_read":{"success":{"event":"instructions","raw_count":8,"time_enabled_ns":10,"time_running_ns":10,"multiplexed":false,"scaled_count":{"success":8}}},"cleanup":{"success":null}},{"event":"page_faults","scope":"measured_thread","unit":{"perf_counter":{"raw_count":"events","time_enabled":"nanoseconds","time_running":"nanoseconds","scaled_count":"events"}},"open":{"success":null},"stop_read":{"success":{"event":"page_faults","raw_count":1,"time_enabled_ns":10,"time_running_ns":10,"multiplexed":false,"scaled_count":{"success":1}}},"cleanup":{"success":null}},{"event":"context_switches","scope":"measured_thread","unit":{"perf_counter":{"raw_count":"events","time_enabled":"nanoseconds","time_running":"nanoseconds","scaled_count":"events"}},"open":{"success":null},"stop_read":{"success":{"event":"context_switches","raw_count":2,"time_enabled_ns":10,"time_running_ns":10,"multiplexed":false,"scaled_count":{"success":2}}},"cleanup":{"success":null}}],"lifecycle":{"phases":["request_validated","file_opened","sources_checked","perf_checked","ownership_released"],"measured_file_ownership_release":"drop_completed"},"first_causal_failure":null,"cleanup_failures":[],"tracefs":{"state":"not_collected","reason":"R33 target preflight deliberately did not invoke tracefs"},"classification":"preflight_subset_ready","reasons":[]}
```

For that artifact, successful retention returns `Success { serialized_byte_length }` and call
disposition `completed`. If `write_all` partially writes then fails with an `Other` error whose raw OS error is 5, the
artifact bytes remain identical, while the returned outcome is `IoFailure { operation: WriteAll,
error: { kind: Other, raw_os_error: Some(5) } }`, flush is not attempted, and disposition is
`retention_failed`. A flush permission-denied error similarly returns `IoFailure { operation:
Flush, error: { kind: PermissionDenied, raw_os_error: Some(13) } }` and `retention_failed`. A
serialization failure returns no artifact bytes, `NotAttempted { reason: SerializationFailure }`,
and `serialization_failed`.

An invalid repository revision, build identity, or measured-file identity returns no artifact or
bytes, `NotAttempted { reason: RequestInvalid(...) }`, and `request_invalid`; rejected values never
appear in an artifact. Focused artifact failures after a valid request use the same exact schema:
CPU-cycle permission is `blocked` with `{"permission":{"errno":13}}`; proc I/O unsupported is
`blocked` with `{"unavailable":{"kind":"unsupported"}}`; a non-regular file, architecture
mismatch, overflow, wrapper error, or cleanup error is `invalid`. None depends on retention.

The exact byte oracle for the CPU-cycle causal failure object (no trailing LF because this object is
embedded in the artifact) is:

```json
{"id":"failure-0001","phase":"perf_open","source":"cpu_cycles","class":"permission","detail":{"errno":13}}
```

In a separate artifact with no causal failure, a context-switch cleanup failure with errno 5 is
`{"id":"failure-0001","phase":"perf_cleanup","source":"context_switches","class":"error","detail":{"kind":"perf_cleanup","errno":5}}`; it is the sole element of `cleanup_failures`.

## 8. Implementation gate

Exactly one later synthetic-tested PR may add the module, re-export, typed construction,
classification, serializer, and injected retention tests in the existing fourth crate. Manifests,
lockfile, dependencies, fixtures, workflows, and toolchains remain unchanged. Tests cover every
closed tag/reason, source metadata and value field, ordering, fail-closed validation, path
non-retention, exact serializer bytes/escaping, ownership-release semantics, causal/cleanup order,
and short/failed write and flush outcomes. No parser or round-trip tests are included.

Success is bounded deterministic correctness evidence only. Live invocation/host observation,
Fedora-release/effective-target validation, workload/action, append/R7 production, calibration,
benchmark/tracefs, D2/D3, durability/recovery/fault, baseline, and production work remain blocked.
