# R33 — R32 Adapter Closure and Target-Preflight Decision

**Status:** Complete documentation/governance decision; the R32 adapter implementation is closed and exactly one later synthetic-tested target-preflight implementation PR is prospectively authorized
**Scope:** EXP-0001 Fedora 44 Linux/x86_64 target preflight before any workload or benchmark execution
**Evidence classification:** bounded deterministic adapter, ownership, lifecycle, wrapper-mapping, failure, and cleanup correctness evidence plus prospective preflight design; no live target validation, retained host observation, execution, publication, or performance evidence
**Authority date:** 2026-09-04

## 1. Exact R32 implementation closure

PR #111 was reviewed at exact head `71f58f65772fea2f0f58f5727d42e1405c7f09fb` and merged as
`05dd7cc0980df2914dff5814ab5f5fba5b8e09e0`. The Documentation validation and EXP-0001 Slice A
workflows both succeeded for that exact reviewed head. This closes the single internal live-adapter
implementation PR authorized by R32.

The result is bounded deterministic adapter, ownership, lifecycle, wrapper-mapping, failure, and
cleanup correctness evidence only. Synthetic tests establish representation and mapping through an
uncalled boundary. They are not live target validation, benchmark execution, publication, or
performance evidence. No successful build, constructor, borrowed descriptor, wrapper result, or
merged code validates a target.

## 2. Smallest target-preflight caller

The next implementation has exactly one explicit entry point in
`experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/target_preflight.rs`:

```rust,ignore
pub fn run_target_preflight(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> TargetPreflightResultV1
```

`lib.rs` may re-export this function and its R33 types. There is no binary, CLI, build script,
fixture runner, test auto-discovery hook, global constructor, background task, or automatic probe.
An operator-controlled program must call the function deliberately after supplying all request
fields. The repository workflows and every test must use an injected synthetic internal boundary
and are prohibited from invoking this function's live path. A CI environment variable, target
match, available file, or test flag must never cause automatic invocation.

`TargetPreflightRequest<'a>` contains exactly:

1. `repository_revision: &'a str`: a lowercase 40-hex Git commit supplied by the operator;
2. `build_identity: &'a str`: the retained opaque build identifier supplied by the operator;
3. `expected_platform: Fedora44Linux` and `expected_architecture: X86_64`, fixed enum values;
4. `measured_file_path: &'a Path`, a transient open input that is never copied into a result,
   diagnostic, error, serialization buffer, or retained file; and
5. `measured_file_identity: &'a str`, an operator-assigned stable pseudonym matching
   `^[a-z0-9][a-z0-9._-]{0,63}$`, with `/`, `\\`, whitespace, `..`, URI schemes, and an equality
   match to any rendered path component prohibited.

Invalid identity/build syntax is decided before opening the measured file. The caller returns a
result on every ordinary path; retention failure is represented in that result and cannot be
reported as ready.

## 3. Measured-file open and ownership contract

The caller opens exactly the supplied, pre-existing measured file through `std::fs::OpenOptions`
with `read(true)`, `write(false)`, `create(false)`, `create_new(false)`, `truncate(false)`, and
`append(false)`. It follows no create-or-retry path and performs no read, seek, write, allocation,
synchronization, metadata mutation, lock, rename, or deletion. Open failure is retained with a
typed class and numeric OS error when present.

After open, `File::metadata().file_type().is_file()` must be true. A directory, symlink-resolved
non-regular object, device, FIFO, or socket makes the result `invalid`. The owned `File` stays in
the caller. `MeasuredFileReference::borrowed` lends its `AsRawFd` capability to
`LiveCaptureBoundary` only for the file check; no ownership transfer, `into_raw_fd`, descriptor
duplication, or close by the adapter is permitted. The caller closes its own file exactly once
after all permitted checks. Close/drop and retention are lifecycle outcomes. The result retains
only `measured_file_identity`; unsanitized host paths are forbidden even in failure text.

## 4. Exact permitted live checks

The live path may perform only this order, retaining every attempted outcome independently:

1. validate request syntax and compare compile-time `target_os`/`target_arch` with Fedora 44
   Linux/x86_64 expectations; Fedora release is an explicit operator assertion in v1 because no
   additional OS-release reader is authorized;
2. open and validate the measured file as section 3 requires;
3. through the merged R29 wrappers, call `clock_resolution` once each for `Realtime` then
   `MonotonicRaw`;
4. through one `LiveCaptureBoundary`, call `realtime`, `monotonic_raw`, process `getrusage`, thread
   `getrusage`, `statm`, `status`, `process_io`, and measured-file `file_length`, in that order;
5. through that boundary, independently open CPU cycles, instructions, page faults, and context
   switches in that order; for every successful owner, immediately stop it (disable then read) and
   explicitly clean it before proceeding to the next event; and
6. close/drop caller-owned resources, serialize, and retain the result.

No delta is computed and no observation action occurs between before/after points. Counter values,
clock samples, procfs/resource values, and file length are capability diagnostics only. They must
not be interpreted as workload measurements. The caller may not access tracefs, inspect another
host source, materialize a workload, invoke a measured action, append or replay, repeat for a
benchmark, calibrate overhead, or produce an R7 record.

## 5. Closed `TargetPreflightResult` v1 contract

The type and serialized schema identifier are exactly
`EXP-0001-R33/target-preflight-result-v1`. The result is closed: unknown fields, absent required
fields, duplicate keys, unknown enum values, reordered arrays, non-decimal integers, and schema
version substitution are invalid.

The logical fields, in serialization order, are:

| Field | Required v1 value |
|---|---|
| `schema` | Exact identifier above. |
| `repository_revision` | Validated lowercase 40-hex input. |
| `build_identity` | Nonempty sanitized opaque input; maximum 128 ASCII graphic characters, excluding slash, backslash, control characters, and whitespace. |
| `platform` | `{expected:"fedora-44-linux", observed:"operator_asserted_fedora_44"|"mismatch"|"invalid", outcome:...}`. The assertion is not an independently observed release. |
| `architecture` | `{expected:"x86_64", observed:"x86_64"|"mismatch", outcome:...}` from the compile-time target. |
| `measured_file` | `{identity, open_outcome, regular_file_outcome, length_outcome}`; never a path. |
| `sources` | Ordered outcomes for `clock_resolution_realtime`, `clock_resolution_monotonic_raw`, `clock_realtime`, `clock_monotonic_raw`, `rusage_process`, `rusage_thread`, `proc_self_statm`, `proc_self_status`, and `proc_self_io`. |
| `perf_events` | Exactly four ordered entries: `cpu_cycles`, `instructions`, `page_faults`, `context_switches`; each has independent `open`, `stop_read`, and `cleanup` outcomes, using `not_attempted_due_to_prior_phase_failure` when applicable. |
| `lifecycle` | Ordered phase ledger plus independent `measured_file_close` and `result_retention` outcomes. |
| `first_causal_failure` | `null` or one typed failure containing phase, source, class, and numeric detail where applicable. |
| `cleanup_failures` | All cleanup failures in occurrence order; never replaces the causal failure. |
| `tracefs` | Exactly `{state:"not_collected",reason:"R33 target preflight deliberately did not invoke tracefs"}`. R33 cannot establish R31's `unsupported` alternative. |
| `classification` | Exactly `ready`, `blocked`, or `invalid`. |
| `reasons` | Nonempty for `blocked`/`invalid`, empty for `ready`; stable reason codes in discovery order. |

Every source outcome is one tagged value: `success(value)`, `unavailable(errno_or_reason)`,
`unsupported(reason)`, `permission(errno)`, `error(reason)`, `overflow(reason)`, or
`not_attempted_due_to_prior_phase_failure(failure_id)`. Existing wrapper outcomes map without
coercion. No unavailable, unsupported, permission, error, overflow, missing, or unattempted state
may become success, zero, an empty value, or another source's outcome.

### 5.1 Fail-closed classification

- `ready` requires valid identities, exact platform/architecture disposition, a successfully
  opened regular file, success for both clock resolutions and every required non-perf source,
  success for each perf open/stop-read/cleanup, successful file close, successful serialization
  and retention, no causal or cleanup failure, and only the frozen tracefs `not_collected` state.
- `blocked` means the request and target identity are valid but capability cannot be established:
  any `unavailable`, `unsupported`, or `permission` source/perf outcome, or unavailable retention
  destination, is retained as a blocking reason. Nothing is silently optional in target preflight.
- `invalid` means malformed/contradictory identity, platform or architecture mismatch, open/error/
  overflow, non-regular file, lifecycle/order violation, wrapper error after acquisition, any
  cleanup failure, serialization failure, schema violation, or unsafe retention/path leakage.

`invalid` takes precedence over `blocked`, which takes precedence over `ready`. All independently
safe checks continue after a blocked outcome. A causal or lifecycle-invalidating failure stops new
acquisition; already acquired owners are cleaned, and later fields use the explicit unattempted
state. The first such failure remains primary and cleanup failures remain ordered separately.

## 6. Deterministic serialization and retention

The implementation must use a dependency-free, hand-written UTF-8 JSON-lines serializer. It emits
exactly one JSON object followed by one LF, fields and arrays in section 5 order, no insignificant
whitespace, integers in shortest decimal form, lowercase enum strings, JSON escaping for `"`, `\\`,
and U+0000–U+001F, and rejects every other non-ASCII input rather than introducing normalization.
It writes no timestamps or nondeterministic map order. A parse/validate-and-reserialize function
must reproduce byte-identical output for documentation vectors and synthetic results.

The operator supplies an already-open retention sink through `TargetPreflightRetention`; the live
caller may not derive or create an output path. The implementation writes the complete line to an
in-memory buffer first, then makes one `write_all` attempt and one `flush` attempt. It must not
truncate, synchronize, rename, publish, retry, or claim atomicity/durability. Retention outcomes
include serialized byte length and success, permission, unavailable, or error; partial write or
flush failure makes the result `invalid`. The caller returns the in-memory result even when sink
retention fails.

Retained files are staged diagnostics under operator control. Recommended repository-relative
staging, when the operator has separately created a sink, is an ignored
`artifacts/staged-target-preflight/` location named only from the pseudonym and schema version.
No host result is authorized in this PR or its implementation PR. A retained result is explicitly
**not** an R7 environment record, raw result, validation report, benchmark result, published
artifact, or performance evidence, and cannot be promoted merely by copying or renaming it.

## 7. Documentation vectors (fictional; not observations)

All values below are invented contract examples. They were not captured from a host.

### 7.1 Complete fictional valid vector

```json
{"schema":"EXP-0001-R33/target-preflight-result-v1","repository_revision":"1111111111111111111111111111111111111111","build_identity":"fictional-build-01","platform":{"expected":"fedora-44-linux","observed":"operator_asserted_fedora_44","outcome":"success"},"architecture":{"expected":"x86_64","observed":"x86_64","outcome":"success"},"measured_file":{"identity":"measured-file-alpha","open_outcome":"success","regular_file_outcome":"success","length_outcome":{"success":{"bytes":4096,"source":"statx"}}},"sources":[{"id":"clock_resolution_realtime","outcome":{"success":1}},{"id":"clock_resolution_monotonic_raw","outcome":{"success":1}},{"id":"clock_realtime","outcome":{"success":1000000000}},{"id":"clock_monotonic_raw","outcome":{"success":2000000000}},{"id":"rusage_process","outcome":{"success":{"user_nanoseconds":100,"system_nanoseconds":50,"maximum_resident_bytes":8192,"minor_faults":2,"major_faults":0,"input_blocks":0,"output_blocks":0,"voluntary_context_switches":1,"involuntary_context_switches":0}}},{"id":"rusage_thread","outcome":{"success":{"user_nanoseconds":80,"system_nanoseconds":40,"maximum_resident_bytes":8192,"minor_faults":1,"major_faults":0,"input_blocks":0,"output_blocks":0,"voluntary_context_switches":1,"involuntary_context_switches":0}}},{"id":"proc_self_statm","outcome":{"success":{"size":10,"resident":2,"shared":1,"text":1,"lib":0,"data":3,"dt":0}}},{"id":"proc_self_status","outcome":{"success":{"resident_bytes":8192,"high_water_bytes":12288}}},{"id":"proc_self_io","outcome":{"success":{"rchar":100,"wchar":0,"syscr":1,"syscw":0,"read_bytes":0,"write_bytes":0,"cancelled_write_bytes":0}}}],"perf_events":[{"id":"cpu_cycles","open":"success","stop_read":{"success":{"raw":10,"time_enabled":10,"time_running":10,"scaled":10}},"cleanup":"success"},{"id":"instructions","open":"success","stop_read":{"success":{"raw":8,"time_enabled":10,"time_running":10,"scaled":8}},"cleanup":"success"},{"id":"page_faults","open":"success","stop_read":{"success":{"raw":1,"time_enabled":10,"time_running":10,"scaled":1}},"cleanup":"success"},{"id":"context_switches","open":"success","stop_read":{"success":{"raw":2,"time_enabled":10,"time_running":10,"scaled":2}},"cleanup":"success"}],"lifecycle":{"phases":["request_validated","file_opened","sources_checked","perf_checked","resources_closed","serialized","retained"],"measured_file_close":"success","result_retention":"success"},"first_causal_failure":null,"cleanup_failures":[],"tracefs":{"state":"not_collected","reason":"R33 target preflight deliberately did not invoke tracefs"},"classification":"ready","reasons":[]}
```

The object supplies every v1 field and complete fictional values for each existing typed wrapper
structure. It is a byte-exact documentation vector, not a host observation.

### 7.2 Focused fictional blocked vectors

- CPU cycles `open=permission(13)`, its stop/cleanup explicitly unattempted, all other checks
  successful: `classification=blocked`, reason `perf.cpu_cycles.permission`.
- `/proc/self/io=unavailable(not_found)` with all safe later checks retained: `blocked`, reason
  `source.proc_self_io.unavailable`.
- retention sink unavailable after an otherwise successful preflight: `blocked`, reason
  `retention.unavailable`; it is never `ready` and the returned in-memory result preserves this.

### 7.3 Focused fictional invalid vectors

- identity `../../real/host/path`: rejected before open as `invalid`, reason
  `measured_file.identity_unsanitized`; the path is absent from the result.
- measured object is a directory: `invalid`, reason `measured_file.not_regular`; no wrappers run.
- perf read scaling overflows and its close returns errno 5: the overflow is
  `first_causal_failure`, the close is the first `cleanup_failures` entry, and classification is
  `invalid`.
- architecture is `aarch64`: `invalid`, reason `architecture.mismatch`; no live check runs.

## 8. Exactly one prospective implementation PR and exit gate

Exactly one later PR may modify only the existing `exp1-descriptive-d1-harness` crate to add the
one caller and typed result/classification boundary above, dependency-free deterministic
serialization/retention, synthetic tests, and the necessary synchronized governance documents.
`src/target_preflight.rs` is the only new source module authorized. Existing Cargo manifests,
lockfile, dependencies, fixtures, workflows, and toolchains remain unchanged; no external
dependency is authorized or needed.

Synthetic tests must cover every classification and outcome tag, exact ordering, all request/file
rules without a real descriptor, every wrapper mapping through injected operations, four
independent perf outcomes and cleanup at every acquisition point, first-causal-versus-cleanup
ordering, path non-retention, exact JSON bytes/escaping/round trip, short/failed writes and flush,
and the complete and focused documentation vectors. A repository test must prove that CI has no
live invocation route; manual invocation is prohibited during CI. Exit requires exact-head review
and both unchanged workflows—Documentation validation and EXP-0001 Slice A—green for that exact
head.

Success closes only deterministic implementation correctness. It does not authorize actual target
execution or retention of host observations, workload generation/materialization, measured
actions, append integration, R7 environment/raw-result/validation-report production, calibration
or overhead experiments, benchmark execution or publication, tracefs access, D2/D3, `fsync`,
durability, recovery, faults, SQLite/RocksDB execution, production code, networking, servers,
queries, or distributed work. Every such step requires later evidence and separate authority.
