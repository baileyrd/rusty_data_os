# R31 — Perf Implementation Closure and Minimum D1 Instrumentation Decision

**Status:** Complete documentation/governance decision; the R30 implementation is closed and exactly one later non-live harness-assembly PR is prospectively authorized
**Scope:** EXP-0001 first descriptive B1/D1 observation and its minimum deterministic orchestration boundary
**Evidence classification:** bounded deterministic ABI/lifecycle/scaling/cleanup correctness evidence plus prospective orchestration design; no live capture, target validation, workload or benchmark execution, performance evidence, durability evidence, or recovery evidence
**Authority date:** 2026-09-03

## 1. Exact R30 implementation closure

PR #105 was reviewed at exact head `547d0bba730d842e354a058773408c66b368f326` and merged as
`409d0737759d4e10fc39ad879ccc975858e89d9f`. The Documentation validation and EXP-0001 Slice A
workflows both succeeded for that exact reviewed head. This closes the single deterministic perf
implementation PR authorized by R30.

The result is bounded deterministic ABI/lifecycle/scaling/cleanup correctness evidence only. It
shows that the frozen four-counter ABI, unique ownership, lifecycle, typed outcomes, checked
scaling, failure propagation, and reverse-order cleanup can be represented and tested with injected
results. It is not live capture, target validation, a workload or benchmark run, a valid result
record, or performance evidence. CI did not invoke a live interface or observe the target.

## 2. Minimum instrumentation for the first descriptive B1/D1 cell

The first descriptive B1/D1 correctness/performance observation must collect or explicitly retain
the typed outcome of each of these independent sources:

1. `CLOCK_MONOTONIC_RAW` for lifecycle start, end, and measured elapsed time;
2. `CLOCK_REALTIME` for observation correlation only, never elapsed-time calculation;
3. both process and measured-thread `getrusage` snapshots;
4. `/proc/self/statm`;
5. `/proc/self/status`;
6. `/proc/self/io`;
7. the measured B1 file length through `statx`, using the frozen `fstat` fallback only when `statx`
   returns `ENOSYS`; and
8. CPU cycles, instructions, page faults, and context switches through the R30 perf boundary when
   each counter is available.

Every source retains its identity, scope, before/after value or terminal typed outcome, and the
applicable units. A required non-perf source that does not produce the frozen successful shape
makes the observation invalid; its failure is still preserved. Perf is availability-conditional:
`unavailable`, `permission`, `error`, and `overflow` remain typed per event and never become zero,
a fabricated delta, or another source's value. Perf page-fault and context-switch observations and
process/thread `getrusage` observations remain separate; they must not be merged, reconciled,
substituted, or presented as equivalent.

This is a minimum, not authority to produce an R7 record, publish evidence, interpret a comparison,
or execute anything. Allocation metrics remain `unsupported` under R7 unless a later authority
selects an allocator observer.

## 3. Narrow tracefs deferral and its authority basis

Tracefs is not required for this first descriptive B1/D1 correctness/performance observation. R31
supersedes R7 section 6 only for this one narrow descriptive cell: syscall, scheduler, and block-I/O
tracefs attribution fields must be represented as one of the following missing states, with no
other wording:

- `not_collected`, reason `R31 first descriptive B1/D1 cell deliberately did not invoke tracefs`;
  or
- `unsupported`, reason `R31 target preflight established that tracefs cannot supply this channel`,
  only when a separately retained target-preflight outcome actually establishes that fact.

No tracefs-derived value, loss statement, syscall/scheduler/block-I/O attribution, or other
tracefs-derived metric or claim may be made. Tracefs absence alone does not invalidate this
descriptive cell unless a later authority makes a tracefs metric primary. Tracefs remains blocked
for confirmatory execution and for every syscall, scheduler, block-I/O, trace-loss, or attribution
claim.

This limited deferral is consistent with, rather than a silent weakening of, the existing
authorities. R7 already requires explicit `not_collected` and `unsupported` states and prohibits
coercing missing instrumentation to zero; it also says diagnostic-channel loss invalidates a run
only when R8 makes the metric primary. R8's D1 C-INGEST primary metrics are monotonic
caller-entry-to-acknowledgement throughput and lifecycle latency, while C-RESOURCE names CPU per
event, allocation cost, peak RSS, physical bytes, and write amplification. It does not name
tracefs-derived syscall, scheduler, or block-I/O attribution as a primary metric. The frozen clock,
resource, procfs, file-length, and availability-conditional perf sources can describe the selected
non-tracefs metrics without claiming the deferred attribution. R7's full tracefs calibration,
sentinel, drain, and loss rules still control any later tracefs use and all confirmatory work.

## 4. Frozen non-live orchestration contract

### 4.1 Inputs and ownership

The later implementation must expose deterministic orchestration over injected boundaries, not a
live runnable harness. Its complete logical input is:

- an immutable `ObservationPlan` containing one B1/D1 cell identity, observation identity, subject
  identity, measured-thread identity, frozen source list, and the three exact tracefs missing-state
  channel entries from section 3;
- a borrowed measured-file handle identity for the injected `statx`/fallback boundary;
- one injected `CaptureBoundary` implementation that returns ordered clock, process/thread
  `getrusage`, procfs, file-length, and perf lifecycle outcomes without invoking the host; and
- one injected, exactly-once `MeasuredAction` that returns a typed success or failure and cannot be
  retried by the orchestrator.

The caller owns the immutable plan, borrowed file identity, and injected implementations. The
orchestrator owns all observation state and returned values. The capture boundary uniquely owns any
synthetic perf session it creates and must close it through the R30 rules; neither plan nor action
may acquire, borrow, transfer, duplicate, or close that ownership. Inputs are non-global, explicit,
and non-cloneable where cloning could repeat an action or owner.

### 4.2 Exact order and lifecycle states

The only successful state path is:

```text
created
-> before_captured
-> counters_armed
-> measuring
-> action_completed
-> counters_stopped
-> after_captured
-> cleaned
-> complete
```

`created -> failed -> cleaned` and a transition from any later nonterminal state to
`failed -> cleaned` are the only failure paths. `complete` and `cleaned_after_failure` are terminal;
no transition, second action, second finalization, or retry is permitted.

The exact call order is: validate the plan; capture pre-observation realtime correlation; capture
before process/thread `getrusage`, all three procfs sources, and file length; open/reset/enable each
available perf counter in R30 order; capture monotonic start; invoke the measured action exactly
once; capture monotonic end; disable/read each opened perf counter; capture after file length, the
three procfs sources, process/thread `getrusage`, and realtime correlation; then finalize all cleanup.
Only the two monotonic points bound the action lifecycle. Warm-up, repetition scheduling, workload
materialization, append orchestration, and publication are not part of this contract.

### 4.3 Output and validity

The sole output is an `ObservationOutcome`, either:

- `complete`, containing the immutable input identities, ordered transition ledger, monotonic
  start/end/checked elapsed value, realtime correlation observations, before/after outcomes for
  both `getrusage` scopes and every procfs/file source, four separately identified perf outcomes,
  the three exact tracefs missing states, measured-action outcome, and successful cleanup status;
  or
- `invalid`, containing the same safely acquired identities, transition ledger and partial typed
  observations plus one ordered primary failure, any later cleanup failures, and the terminal state.

No partial output may be promoted to `complete`. Monotonic reversal/overflow, action failure,
unexpected order/transition, duplicate call, a required non-perf source failure, perf lifecycle
failure after a successful open, or any cleanup failure yields `invalid`. A perf open classified as
typed unavailable or permission is retained per event and does not by itself invalidate this
availability-conditional descriptive observation. Errors are never replaced, retried, zero-filled,
or hidden by a later error. The first causal failure is primary; cleanup is nevertheless attempted
for every acquired owner in reverse acquisition order, and every cleanup failure is appended in
that deterministic order.

## 5. Exactly one prospective harness-assembly PR

Exactly one later non-live PR may add deterministic orchestration code and synthetic tests. It may
modify only:

- `experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/lib.rs`;
- `experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/linux_capture.rs`;
- one new harness/orchestration source module within that crate; and
- synchronized status, traceability, and readiness documentation.

It may add no crate, dependency, manifest, lockfile, fixture, toolchain, or workflow change. It may
not add a binary or CLI. Tests and CI must invoke only injected deterministic boundaries: no clock,
resource, procfs, file, perf, tracefs, or other live interface may be called, and no host may be
probed.

Fail-closed synthetic tests must cover the exact successful call order and output, each legal and
illegal state transition, exactly-once action behavior, each required-source failure at each phase,
every per-event perf missing/error/overflow class, monotonic reversal/overflow, primary-versus-cleanup
failure ordering, reverse cleanup after every acquisition point and unwind, no double cleanup, no
partial-valid release, and the exact tracefs missing-state reasons. Completion requires exact-head
review and both unchanged R9 workflows successful. Passing them would be bounded deterministic
orchestration correctness evidence only.

## 6. Retained exclusions and next gates

R31 authorizes no live interface invocation, target probe or validation, generated workload,
workload or benchmark execution, evidence publication, performance conclusion, R7 record producer,
binary/CLI, append orchestration, or baseline execution. D2/D3, `fsync`, durability, recovery,
faults, adapters, SQLite/RocksDB execution, production crates, networking, servers, queries, and
distributed behavior remain excluded.

Before any descriptive execution, a later authority must close live-use, effective-configuration,
record-production/validation, workload-materialization, calibration/overhead, and execution gates.
Before confirmatory execution or any trace attribution claim, a later authority must additionally
freeze and validate tracefs interfaces, sentinel/drain/loss behavior, scope, privilege, and
unavailable-field policy. R31 does not resolve those gates by deferring tracefs for one descriptive
cell.
