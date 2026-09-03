# R30 — R29 Preflight Closure and Perf Counter ABI Decision

**Status:** Complete documentation/governance decision; the R29 implementation is closed and exactly one later counter-only implementation PR is prospectively authorized
**Scope:** EXP-0001 Fedora 44 Linux/x86_64 preflight and the smallest counter-only `perf_event_open` boundary
**Evidence classification:** bounded implementation/correctness-validation evidence plus prospective ABI design; no live capture, target validation, execution, benchmark, performance, durability, or recovery evidence
**Authority date:** 2026-09-03

## 1. R29 implementation closure

PR #103 was reviewed at exact head `eda5005c3a3e6e6ec76e90f882bc320e7da1bce3` and merged as
`1d8466d1ce8c7c99e0fbd572c1cb77b2e357ba11`. The Documentation validation and EXP-0001 Slice A
workflows both succeeded for that exact reviewed head. This closes the one implementation PR
authorized by R29.

The merged fourth crate is evidence only that the reviewed Linux/x86_64 layouts and constants can
be represented, the five authorized glibc calls can be isolated behind safe wrappers, typed
fail-closed outcomes and the three exact procfs parsers can be implemented, and those boundaries can
be exercised with deterministic synthetic tests. It is not evidence of live capture, behavior on
the target, a valid benchmark record, performance, durability, recovery, or execution. CI made no
live observation and its success is not target validation.

## 2. Decision and boundary

R30 freezes the smallest useful next instrumentation boundary: four counting events opened through
`perf_event_open`, read as counters, and nothing else. There is no mmap or perf ring buffer, sampling
record, overflow signal delivery, tracefs access, syscall tracepoint, scheduler-event tracepoint,
block-I/O tracepoint, live capture, host probe, or execution. This design resolves only the local
counter ABI portion of BLK-021/UNK-022. Effective instrumentation, overhead, capture, publication,
and execution remain open.

The sources are the R7-pinned Linux man-pages 6.15 `perf_event_open(2)` contract and Linux UAPI
`linux/perf_event.h`, interpreted for the R4 Fedora 44 x86_64 target. The values below are normative;
an implementation must not substitute host headers, infer a different version, or delegate a
choice to the running machine. A mismatch fails closed and requires a later authority.

## 3. Normative Linux/Fedora 44 x86_64 perf ABI

### 3.1 Invocation, subject, and event attributes

| Item | Frozen value |
| --- | --- |
| Invocation route | One dedicated `perf_event_open` wrapper invokes the glibc variadic `syscall` symbol as `syscall(298, attr_ptr, pid, cpu, group_fd, flags)`. x86_64 `__NR_perf_event_open = 298`; C `long` and each promoted integer argument are signed 64-bit. No generic syscall helper or other raw syscall route is permitted. |
| Scope | Current calling thread only: `pid = 0`, `cpu = -1`; the task may run on any CPU. `inherit = 0` prohibits child inheritance. User and kernel execution of that thread are both counted; no exclude bit is set. This is not process-wide aggregation. |
| Grouping | One independent FD per event; `group_fd = -1`. Grouping and group reads are prohibited. |
| Open flags | Exactly `PERF_FLAG_FD_CLOEXEC = 1 << 3 = 0x8`; all other open flags zero. Atomic CLOEXEC is mandatory; a later `fcntl` substitute is prohibited. |
| Attribute version | `PERF_ATTR_SIZE_VER0 = 64`; `perf_event_attr` size 64, alignment 8. Every byte is zero before named fields are assigned. Reserved bytes/bits remain zero and are never read. |
| Attribute fields | Set/readable-by-implementation inputs are only `type` (`u32`, offset 0), `size` (`u32`, offset 4), `config` (`u64`, offset 8), `read_format` (`u64`, offset 32), and the 64-bit flags word at offset 40. In that word only `disabled`, bit 0, is one; `inherit`, bit 1, and every other bit are zero. The union at offset 16, `sample_type` at 24, `wakeup_events` at 48, `bp_type` at 52, and `config1` at 56 remain zero. No attribute output is consumed, including after failure. |
| Read format | `PERF_FORMAT_TOTAL_TIME_ENABLED = 1 << 0 = 0x1` and `PERF_FORMAT_TOTAL_TIME_RUNNING = 1 << 1 = 0x2`; exact `read_format = 0x3`. IDs, groups, lost counts, and all other formats are prohibited. |

The exact version-0 layout is: `type` 0..4, `size` 4..8, `config` 8..16,
`sample_period_or_freq` 16..24, `sample_type` 24..32, `read_format` 32..40, flags 40..48,
`wakeup_events_or_watermark` 48..52, `bp_type` 52..56, and `config1_or_bp_addr` 56..64.
Compile-time assertions must cover size, alignment, every offset, every selected bit, and all-zero
reserved treatment.

### 3.2 Minimum event set

Exactly four FDs are permitted:

| Metric/source identity | `type` | `config` |
| --- | ---: | ---: |
| CPU cycles / perf hardware | `PERF_TYPE_HARDWARE = 0` | `PERF_COUNT_HW_CPU_CYCLES = 0` |
| Instructions / perf hardware | `PERF_TYPE_HARDWARE = 0` | `PERF_COUNT_HW_INSTRUCTIONS = 1` |
| Page faults / perf software | `PERF_TYPE_SOFTWARE = 1` | `PERF_COUNT_SW_PAGE_FAULTS = 2` |
| Context switches / perf software | `PERF_TYPE_SOFTWARE = 1` | `PERF_COUNT_SW_CONTEXT_SWITCHES = 3` |

Page faults and context switches are retained because R7 calls for explicit perf observations. They
are separate sources from R29 `getrusage` minor/major faults and voluntary/involuntary context
switches: their scopes and semantics are not interchangeable. Records must preserve each source
identity and must never add, merge, replace, or silently reconcile the perf and `getrusage` values.
No other perf counter is authorized.

### 3.3 Ownership and lifecycle

Each successful open returning `fd >= 0` immediately creates one uniquely owning, non-copyable FD
object. A negative syscall result reads errno once and creates no owner. The exact lifecycle for
each FD is:

1. open disabled;
2. `ioctl(fd, PERF_EVENT_IOC_RESET, 0)` where `PERF_EVENT_IOC_RESET = 0x2403`;
3. `ioctl(fd, PERF_EVENT_IOC_ENABLE, 0)` where `PERF_EVENT_IOC_ENABLE = 0x2400`;
4. the later, separately authorized measured interval (R30 authorizes no such interval);
5. `ioctl(fd, PERF_EVENT_IOC_DISABLE, 0)` where `PERF_EVENT_IOC_DISABLE = 0x2401`;
6. exactly one `read(fd, result_ptr, 24)`; and
7. exactly one `close(fd)`.

The `ioctl`, `read`, and `close` symbols are the glibc calls; their return type is signed 32-bit for
`ioctl`/`close` and signed 64-bit `ssize_t` for `read`. Each ioctl succeeds only on zero. Read
succeeds only on exactly 24 bytes. Close succeeds only on zero. No reset while enabled, repeated
enable/disable, accumulated interval, refresh, period, pause/resume, or cross-FD group operation is
permitted.

The read destination is size 24/alignment 8 with exactly three native-endian `u64` fields:
`raw_count` at offset 0, `time_enabled_ns` at offset 8, and `time_running_ns` at offset 16. All three
are always retained together, including when multiplexed or when scaling cannot produce a metric.
A 0-byte EOF, negative result, or any positive result other than 24 is not a metric.

All opened owners must be closed in reverse acquisition order on ordinary return, intermediate
open/lifecycle failure, and unwind. Explicit finalization reports close failure. Unwind cleanup uses
the same one-shot close boundary and records a sticky cleanup failure in caller-owned session state;
an observation cannot be released as valid unless every owner reports successful cleanup. Ownership
transfer, `dup`, leaked FDs, double close, and reliance on process exit are prohibited.

### 3.4 Multiplexing and checked scaling

`time_running_ns < time_enabled_ns` is explicit multiplexing; equality is not. Both times and the
raw count are preserved in either case. A valid unscaled counter requires
`time_enabled_ns > 0`, `time_running_ns > 0`, and `time_running_ns <= time_enabled_ns`.

The reported scaled count is the nearest integer with half values rounded upward:

```text
numerator = checked_u128(raw_count) * checked_u128(time_enabled_ns)
scaled = (numerator + floor(time_running_ns / 2)) / time_running_ns
```

Every multiplication and addition is checked in `u128`, and the quotient must fit `u64`. When
enabled equals running the same formula must return the raw count. Zero running time, zero enabled
time, running greater than enabled, arithmetic overflow, or a non-`u64` quotient is fail-closed and
does not yield a valid metric. No floating-point scaling is permitted.

The selected lifecycle has one post-reset terminal read, so no subtraction is normally performed.
Any future injected/test parser that compares snapshots must reject a lower raw count or lower time
as an unexpected decrease/reset; wraparound, reset inference, and modular subtraction are
prohibited. A short read, overflow, unexpected decrease/reset, lifecycle failure, or cleanup failure
invalidates the affected observation and cannot be converted to a valid metric.

### 3.5 Outcomes, errno, retries, and fail-closed policy

All boundaries use R29's typed `success`, `unavailable`, `permission`, `overflow`, or `error`
outcomes and retain numeric errno. No interface call is retried, including `EINTR`; no substitute
event/interface, elevation, permission change, or zero-fill is allowed.

| Condition | Classification |
| --- | --- |
| `EPERM = 1` or `EACCES = 13` | `permission(errno)`; retain denial and never elevate. |
| `ENOENT = 2`, `ENXIO = 6`, `ENODEV = 19`, `ENOSYS = 38`, or `EOPNOTSUPP = 95` from open | `unavailable(errno)`; never zero. |
| `EOVERFLOW = 75` from any boundary, or checked numeric/scaling overflow | `overflow`; retain errno where present. |
| Any other errno, including `EINTR = 4`, `EBADF = 9`, `ENOMEM = 12`, `EBUSY = 16`, `EINVAL = 22`, `EMFILE = 24`, and `ENFILE = 23` | `error(errno)`. `EINVAL` is an authority/contract mismatch, not “unsupported.” |
| Open result `fd >= 0`; ioctl/close result zero; read result exactly 24 | Boundary success, subject to all structural, lifecycle, time, scaling, and cleanup checks. |
| Unexpected return, short/zero read, invalid denominator/time relation, reset/decrease, lifecycle error, or close error | `error` (or the more specific `overflow` above); never a valid metric. |

Unsupported or unavailable events stay typed unavailable. Multiplex status, enabled time, running
time, and raw count are retained even where scaling fails. Availability only says an interface
could return data; it is not performance evidence.

## 4. Tracefs remains blocked

Tracefs remains wholly unauthorized. Syscall, scheduler-event, and block-I/O attribution remain
unavailable until a separate authority freezes the exact tracepoints, tracefs paths and formats,
per-CPU ordering, sentinel protocol, buffer sizing/draining, loss detection, privilege policy, and
unavailable-field policy. Perf counters do not substitute for those attribution channels.

## 5. Exactly one prospective implementation PR

Exactly one later PR may modify only the existing fourth crate: extend
`experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/linux_capture.rs`, add deterministic
synthetic tests within that crate, and update synchronized status documentation. It may add no crate
or dependency and may not change Cargo manifests, `Cargo.lock`, fixtures, toolchains, workflows, or
any of the three existing implementation crates.

That implementation may add the dedicated callable glibc-syscall wrapper, exact structs/constants,
FD ownership, pure parsing/classification/scaling functions, cleanup state, and deterministic
injected-result tests. Tests must not invoke perf, probe the host, or perform a live observation in
CI or elsewhere. Compile-time layout checks and synthetic tests must cover every selected constant,
layout, result length, lifecycle transition, errno class, ownership/cleanup path, denominator,
multiplex case, rounding boundary, overflow, short read, reset/decrease, and prohibited-field rule.

## 6. Retained exclusions and revisit conditions

R30 authorizes no live capture or host probing; binary/CLI or harness execution; tracefs, mmap, ring
buffers, sampling, signals, or privilege changes; M01 materialization; append/replay orchestration;
R7 record producer; workload or benchmark execution; publication or performance conclusion; D2/D3,
`fsync`, durability, recovery, or faults; adapters or SQLite/RocksDB execution; production code;
networking, server, query, or distributed behavior.

Revisit before implementation if the target, architecture, glibc route, kernel UAPI, required
counter semantics, or exact ABI differs. Revisit also if deterministic ownership cannot guarantee
and report close on every path. Such a conflict does not authorize host-derived constants, retries,
a generic syscall helper, another event, tracefs, or a broader evidence claim.
