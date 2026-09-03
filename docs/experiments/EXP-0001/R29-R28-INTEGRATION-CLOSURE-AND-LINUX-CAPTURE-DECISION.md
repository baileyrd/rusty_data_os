# EXP-0001 R29 — R28 integration closure and live Linux capture decision

**Contract:** `EXP-0001-R29-LINUX-CAPTURE-BOUNDARY-v1`
**Status:** R28 integration closed; Linux capture boundary frozen; one bounded implementation PR prospectively authorized
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
closed. The sole remaining prerequisite decision for implementation of a descriptive D1 harness
was the live Linux capture implementation boundary; sections 2–6 freeze that decision. Harness
execution remains separately gated and unauthorized.

## 2. Selected target and implementation boundary

The capture target is only the R4 primary target: Fedora Linux 44 on the bare-metal Bosgame M5,
`x86_64`. The future implementation shall have no external Rust dependencies. Every existing crate
retains `#![forbid(unsafe_code)]` and remains unchanged.

Future unsafe code is permitted only in one narrowly isolated `linux_capture` module in the future
fourth experiment crate. Every unsafe operation shall be the minimum operation needed for a direct
foreign-function or system-call boundary and shall sit behind a reviewed safe wrapper. Application
logic, parsing, policy, aggregation, record construction, and lifecycle orchestration are forbidden
inside unsafe blocks. Generic unsafe helpers or utilities are forbidden.

The boundary shall use direct Linux/glibc interfaces for `clock_gettime`, `clock_getres`,
`getrusage`, `statx` with `fstat` fallback where required, and `perf_event_open`. Safe `std`
filesystem reads shall be used for procfs and tracefs where possible. It shall not use shell-command
substitution for required measurements, elevate privileges, attempt to change permissions, or
support or silently compile a capture implementation for architectures other than Linux/x86_64.

This selection resolves BLK-020/021/026/027 and UNK-022 only for the bounded capture/preflight
implementation boundary. It does not resolve BLK-015, authorize capture or execution, or validate
the target environment.

## 3. Fail-closed capture behavior

Before creating any publishable evidence, the future boundary must preflight every required source
for the selected observation. Preflight and capture results must record, as applicable:

* exact interface identity and version/profile, measurement scope, units, permissions,
  availability, and counter width;
* perf enabled time, running time, and multiplexing/scaling state; and
* trace/perf loss counters, recorder queue high-water/overflow state, sequence continuity, read
  status, sentinel status, and final drain status.

Permission denial is a typed observation that must be retained and must never trigger privilege
escalation. Safe wrappers must distinguish typed `success`, `unavailable`, `permission`, `loss`, and
`error` outcomes. An unavailable diagnostic field must carry an explicit typed disposition and must
never be encoded as numeric zero.

Absence or loss of a correctness or lifecycle channel invalidates the run. If a required R8-primary
metric is unavailable, the observation is invalid or inconclusive according to the existing R7/R8
authority. Trace or perf loss, queue overflow, sequence gaps, read errors, sentinel failure, or an
incomplete drain follows R7's existing failure and replacement rules without reinterpretation.
Diagnostic-channel loss makes the affected metric unavailable and invalidates the run when that
metric is primary; it is never zero-filled.

Captured values are observations only. Availability, a successful read, clock resolution, or a
counter value is not evidence of clock accuracy, durability, performance, causal attribution, or
fitness of an architecture.

## 4. ABI, arithmetic, and test safeguards

The future crate must compile-time gate its capture implementation to `target_os = "linux"` and
`target_arch = "x86_64"`; unsupported targets must fail explicitly rather than substitute another
interface. The isolated module must reproduce and review the exact required C/UAPI constants,
structure layouts, field offsets/alignment, signedness, and integer widths for the frozen target.
It must handle every return convention and partial result, map `errno` without collapsing distinct
permission/unavailable/error states, and check all size, timestamp, duration, counter, scaling, and
unit-conversion arithmetic for overflow or invalid denominators.

Compile-time assertions or deterministic tests shall verify layouts, sizes, alignments, integer
widths, and constants wherever possible. Safe wrappers must expose only typed results described in
section 3. Synthetic deterministic tests may exercise parsing, conversion, overflow, and error/loss
classification without requiring the host interfaces to be available.

CI must not require privileged perf or tracefs access. Host-dependent probes may not make ordinary
CI success depend on permission or availability, and no CI capture is EXP-0001 experimental
evidence. Deterministic tests, formatting, clippy, offline locked tests, and documentation validation
remain mandatory.

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

The implementation may define typed preflight/capture data, safe wrappers, parsers, conversions,
and deterministic synthetic tests. It may not materialize M01, append or replay data, execute an R7
record producer, run a workload, benchmark, publish capture artifacts, or draw performance claims.

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
