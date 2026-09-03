# EXP-0001 R29 — R28 integration closure and live Linux capture decision

**Contract:** `EXP-0001-R29-LINUX-CAPTURE-BOUNDARY-v1`
**Status:** R28 integration closed; bounded clocks/resource/file/procfs preflight ABI frozen; perf and tracefs remain blocked; one narrower implementation PR prospectively authorized
**Evidence classification:** governance decision plus bounded deterministic integration/correctness evidence only; not experiment execution, capture evidence, durability evidence, benchmark evidence, or a performance conclusion
**Decision date:** 2026-09-03

## 1. R28 integration closure

R29 closes the test-only integration authorized by R28. PR #101's reviewed exact head is
`b88908cb9cbba39774437e582308bab25a88482b`; its merge commit is
`2168839a70baebdea1773fc56e7b8aa0dc9a89e4`. The **Documentation validation** and **EXP-0001
Slice A** workflows both succeeded for that exact reviewed head.

The merged test connects the four reviewed literal SOP2 operations to the contextual mapper,
byte-exact RF1 frames, one `RawAppender`, and deterministic physical reopen/replay. It checks the
pre-append failure boundary, caller-owned state, receipts, accepted prefix, records, sequence,
ordinal, bytes, and order. This is only bounded deterministic integration/correctness evidence. It
is not workload materialization or execution, capture, stable-storage or recovery evidence,
durability evidence, a benchmark, or a performance result.

The complete semantic-operation-to-physical-record mapping blocker identified by R19 is therefore
closed. The semantic mapping prerequisite is closed, but a complete descriptive D1 harness remains blocked.
Sections 2–6 freeze only the bounded preflight subset that repository authority can specify exactly;
perf, tracefs, effective capture, harness assembly, and execution remain separately gated.

## 2. Selected target and implementation boundary

The target is only the R4 primary target: Fedora Linux 44 on bare-metal Bosgame M5, `x86_64`.
The future implementation has no external Rust dependencies; every existing crate remains unchanged
and retains `#![forbid(unsafe_code)]`. Unsafe code is permitted only in one isolated `linux_capture`
module in the fourth crate, only for the five glibc calls frozen below, behind typed safe wrappers.
Parsing, policy, arithmetic, aggregation, record construction, and orchestration are forbidden in
unsafe blocks. Generic unsafe or syscall helpers are forbidden.

The authorized boundary is smaller than R7's complete instrumentation design. It uses the exact
glibc symbols `clock_gettime`, `clock_getres`, `getrusage`, `statx`, and `fstat`, plus safe `std::fs`
reads of three procfs files. `perf_event_open`, perf ring buffers, every tracefs file/event, and all
syscall/scheduler/block-I/O counter capture remain blocked: R7 does not freeze a sufficiently narrow
event set, ring-buffer ABI, tracepoint format, or loss protocol. No direct `syscall(2)` invocation is
authorized. Shell substitution, privilege elevation, permission/configuration changes, and fallback
to another OS or architecture are prohibited.

This closes only the local ABI ambiguity for the listed clocks/resource/file/procfs preflight
subset. BLK-020/026/027 extend only far enough to create and test that fourth-crate subset;
BLK-021 and UNK-022 remain open for effective instrumentation, perf, tracefs, overhead/loss, and
publishable capture. BLK-015 remains open. Capture, execution, and target validation remain
unauthorized.

## 3. Fail-closed typed outcomes

The wrappers expose exactly `success(value)`, `unavailable(reason)`, `permission(errno)`,
`overflow(reason)`, or `error(errno-or-parse-reason)`. They preserve the numeric errno on every
failed C call. Outputs are never read after a failed call. No unavailable field is encoded as zero,
no partial parse is success, and no errno or parse failure triggers a substitute interface except
the one `statx` fallback expressly frozen below.

Preflight records interface identity/profile, scope, units, permission, availability, and width.
Permission denial is retained and never triggers elevation. A missing required R7 channel keeps its
metric unavailable and capture/execution blocked; loss, read error, truncation, invalid denominator,
or overflow is never zero-filled. Captured values, when capture is later authorized, will remain
observations rather than clock-accuracy, durability, performance, or causal-attribution evidence.

## 4. Normative Linux/x86_64 ABI and parsing contract

This is the complete ABI allowlist. On the selected glibc x86_64 ABI, C `int`, `long`, and `time_t`
are signed 32-, signed 64-, and signed 64-bit values; `size_t`, `dev_t`, and `ino_t` are unsigned
64-bit values and `off_t` is signed 64-bit. Declarations must preserve every listed size, alignment,
and offset with explicit padding; the implementation may read only listed fields. The errno
identifiers used below have x86_64 values `EPERM = 1`, `ENOENT = 2`, `EBADF = 9`, `EACCES = 13`,
`ENODEV = 19`, `ENOTDIR = 20`, `EINVAL = 22`, `ENOSYS = 38`, and `EOVERFLOW = 75`. A wrapper makes
one call and does not retry any errno, including `EINTR`; an errno not explicitly classified below
is `error` with its numeric value retained.

| Boundary | Invocation and constants | Representation and readable fields | Return and typed outcome |
| --- | --- | --- | --- |
| lifecycle/UTC clocks | glibc `clock_gettime(clockid_t, timespec *)` and `clock_getres(clockid_t, timespec *)`; signed 32-bit `clockid_t`; only `CLOCK_REALTIME = 0` and `CLOCK_MONOTONIC_RAW = 4` | `timespec`: size 16/alignment 8; signed 64-bit `tv_sec` offset 0, signed 64-bit `tv_nsec` offset 8; both readable. Require `0 <= tv_nsec < 1_000_000_000`; convert with checked `tv_sec * 1_000_000_000 + tv_nsec` into signed 128-bit nanoseconds before narrowing. | `0` success; `-1` reads thread-local `errno`; no partial result. `EINVAL`/`ENODEV`/`ENOSYS` => `unavailable`; `EACCES`/`EPERM` => `permission`; otherwise `error`. Invalid fields => `error`; arithmetic failure => `overflow`. |
| CPU resource totals | glibc `getrusage(int, rusage *)`; only `RUSAGE_SELF = 0`, `RUSAGE_THREAD = 1` | `timeval`: size 16/alignment 8, signed 64-bit seconds offset 0 and microseconds offset 8. `rusage`: size 144/alignment 8. Read only `ru_utime` offset 0, `ru_stime` 16, and signed 64-bit `ru_maxrss` 32 (KiB on Linux), `ru_minflt` 64, `ru_majflt` 72, `ru_inblock` 88, `ru_oublock` 96, `ru_nvcsw` 128, `ru_nivcsw` 136. Require `0 <= tv_usec < 1_000_000` and nonnegative counters; use checked conversions. | `0` success; `-1` errno; no partial result. `EINVAL`/`ENOSYS` => `unavailable`; `EACCES`/`EPERM` => `permission`; otherwise `error`. Invalid fields => `error`; arithmetic failure => `overflow`. |
| primary open-file length | glibc `statx(fd, "", AT_EMPTY_PATH | AT_STATX_SYNC_AS_STAT, STATX_SIZE, statx *)`; `AT_EMPTY_PATH = 0x1000`, `AT_STATX_SYNC_AS_STAT = 0x0000`, `STATX_SIZE = 0x00000200` | Linux UAPI `statx` version 0: size 256/alignment 8. Zero-initialize all bytes. Read only unsigned 32-bit `stx_mask` offset 0 and unsigned 64-bit `stx_size` offset 40. Success requires the size mask; size must fit signed 64-bit for the existing D1 file API. | `0` complete success; `-1` errno; no partial result. **Only `ENOSYS` permits fallback.** `EACCES`/`EPERM` => `permission`; `EOVERFLOW` => `overflow`; any other errno => `error`. Missing size mask => `unavailable` without fallback. |
| sole file-length fallback | glibc `fstat(fd, stat *)` on the same open fd, only after `statx` returned `ENOSYS`; no reopen | glibc x86_64 `stat`: size 144/alignment 8. Read only signed 64-bit `st_size` offset 48; negative is `error`. It supplies only file length. Every statx-only field is typed `unavailable`, never zero. | `0` success; `-1` errno; no partial result. `EACCES`/`EPERM` => `permission`; `EOVERFLOW` => `overflow`; `ENOSYS` => `unavailable`; otherwise `error`. |

The only safe-filesystem inputs are `/proc/self/statm`, `/proc/self/status`, and `/proc/self/io`,
read to EOF by `std::fs::read_to_string`. `PermissionDenied` maps to `permission`, `NotFound` or
`Unsupported` to `unavailable`, and other I/O kinds to `error`; invalid UTF-8 is `error`. Numeric PID
paths, retry after partial read, symlink-derived identity, and truncated/best-effort parses are
prohibited.

* `statm` is exactly one nonempty ASCII line with seven whitespace-separated unsigned base-10
  64-bit page counts: `size resident shared text lib data dt`. Extra/missing tokens, signs,
  non-ASCII, or overflow are errors. This tranche retains page counts only and does not invent a
  page-size ABI or convert them to bytes.
* `status` is ASCII `Key: value` lines. Exactly one `VmRSS` and one `VmHWM` are required, each an
  unsigned base-10 integer followed by case-sensitive `kB` and no other token. Duplicate/missing or
  malformed selected keys are errors; conversion to bytes uses checked multiplication by 1024.
* `io` is ASCII `key: value` lines. Exactly one each of `rchar`, `wchar`, `syscr`, `syscw`,
  `read_bytes`, `write_bytes`, and `cancelled_write_bytes` is required. Each value is an unsigned
  base-10 64-bit integer with no unit/trailing token. Duplicate/missing/malformed/signed/overflowing
  selected values are errors. These are process-accounting observations, not physical-device I/O.

`perf_event_open` has **no authorized ABI in v1**: no syscall number, `perf_event_attr` size/version,
event identifier, pid/cpu scope, flags, read format, enabled/running-time handling, multiplex
scaling, ring buffer, or loss policy may be implemented. Tracefs likewise has **no authorized file
or parsing contract**. Both remain blocked pending a later authority; their absence makes dependent
R7 metrics unavailable and cannot be hidden by zero-fill or substitution.

These values freeze the subset of the Linux man-pages 6.15 and Linux UAPI/glibc authorities already
pinned by R7; they are not delegated to host-header discovery. A mismatch with the selected Fedora
44 headers or glibc during implementation is a fail-closed authority conflict requiring a new
documentation decision, not permission to alter a constant, layout, route, or fallback.

Compile-time assertions and deterministic tests must verify every constant, size, alignment, offset,
integer width, selected field, parser rule, error mapping, and overflow edge above. CI requires no
host interface or privilege and produces no experimental evidence.

## 5. Exactly one prospectively authorized implementation PR

R29 prospectively authorizes exactly one next implementation PR. It may create the fourth
experiment workspace member, provisionally named `exp1-descriptive-d1-harness`, and implement only
the Linux capture/preflight boundary above plus deterministic tests. The package remains
experiment-local and external-dependency-free.

Allowed Cargo changes are exactly:

1. add `crates/exp1-descriptive-d1-harness` to the member list in
   `experiments/exp-0001/Cargo.toml`;
2. add that crate's `Cargo.toml`, with direct path dependencies only on
   `exp1-record-format`, `exp1-workload-conformance`, and `exp1-raw-append-replay`; and
3. update only the corresponding workspace/package/path-dependency entries in
   `experiments/exp-0001/Cargo.lock`.

Dependency direction is one-way from the new harness crate to those three existing crates. No
existing crate may depend on the harness or gain/change any dependency, feature, manifest entry, or
source. No crates.io, git, build, dev, target-specific external, transitive external, or system
library Rust dependency is permitted. The unchanged toolchain and both existing workflows remain
the validation boundary. Synchronized closure/status documentation may accompany that one PR.

The implementation may define typed preflight data, safe wrappers, the three procfs parsers, checked
conversions, and deterministic synthetic tests for section 4 only. It may not open perf or tracefs,
invoke `syscall`, perform a live measurement/capture, materialize M01, append or replay data, execute
an R7 record producer, run a workload, benchmark, publish artifacts, or draw performance claims.

## 6. Retained exclusions and next gate

R29 authorizes no experiment execution, generated workload run, benchmark, or performance evidence.
It authorizes no D2/D3, `fsync`, durability, recovery, fault injection, SQLite or RocksDB execution,
adapter, production code, networking, server, query language, or distributed behavior. It
authorizes no destructive apparatus, privilege escalation, package installation, kernel/sysctl,
tracefs/perf permission, mount, filesystem, storage, firmware, power, service, or other machine
configuration change.

After the one authorized implementation is reviewed, merged, and closed by a separate authority,
a further governance increment must assess the capture boundary and decide whether any descriptive
D1 harness assembly or execution can be authorized. R29 itself authorizes neither.

## 7. Alternatives, assumptions, and revisit conditions

External Rust dependencies were rejected to preserve the reviewed offline workspace boundary.
Shell utilities were rejected for required measurements because they obscure ABI, scope, units,
permissions, and loss handling. Crate-wide unsafe permission and generic syscall utilities were
rejected because they expand the auditable boundary. Privileged CI and automatic elevation were
rejected because availability is data, not permission to alter the target.

This decision assumes the future implementation can express the required Fedora 44 Linux/x86_64
ABIs accurately without an external crate. Revisit before implementation if the target OS or
architecture changes, an exact required interface cannot be represented and tested within the
isolated boundary, the kernel/UAPI contract changes, or an R7/R8 required source cannot return the
typed provenance and loss state required here. Revisit does not authorize a workaround or broaden
the evidence claim.
