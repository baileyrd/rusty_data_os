# R32 — R31 Orchestration Closure and Live Adapter Decision

**Status:** Complete documentation/governance decision; the R31 orchestration implementation is closed and exactly one later non-live-tested internal live-adapter implementation PR is prospectively authorized
**Scope:** EXP-0001 internal boundary between deterministic R31 orchestration and the existing R29/R30 Linux wrappers
**Evidence classification:** bounded deterministic injected orchestration, lifecycle, source-metadata, failure, and cleanup correctness evidence plus prospective adapter design; no live readiness, target validation, capture, workload or benchmark execution, publication, performance evidence, durability evidence, or recovery evidence
**Authority date:** 2026-09-03

## 1. Exact R31 implementation closure

PR #108 was reviewed at exact head `fb1200c31e4730404a1ee941cfb58fa77520f43b` and merged as
`0a5c2471a6f1d5d87a4d58a7b12ec921ec6bddcb`. The Documentation validation and EXP-0001 Slice A
workflows both succeeded for that exact reviewed head. This closes the single deterministic
orchestration implementation PR authorized by R31.

The evidence is limited to deterministic injected orchestration, lifecycle, source metadata,
failures, and cleanup. Synthetic tests show that the frozen plan, exactly-once action, ordered
source outcomes, fail-closed partial result, first causal failure, and reverse cleanup can be
represented without invoking a host. They do not establish that the live wrappers can be composed
with that orchestrator, that a target is valid, or that any capture or execution has occurred.

## 2. Decided interface gap

The R31 `CaptureBoundary` is deliberately injectable, while the existing R29/R30 wrappers expose
live Linux operations with different ownership shapes. Two mismatches must be resolved before a
later caller can even be proposed:

1. the orchestration boundary carries only a string measured-file identity, but the R29 file
   wrapper requires a borrowed `AsRawFd` capability; and
2. the R30 live perf wrapper opens all four descriptors as one aggregate session, so one event's
   unavailable or permission result prevents retention of independently available event results,
   whereas R31 requires four per-event outcomes and owners.

R32 resolves only the internal interface design. It does not resolve live use, target validation,
capture, or execution.

## 3. Frozen measured-file reference

The later implementation must replace the string-only input boundary with an explicit borrowed
measured-file reference. That reference contains both:

- a stable measured-file identity retained in the observation output; and
- a borrowed `AsRawFd` capability used only to supply the existing file wrapper.

The reference transfers no ownership and grants no duplication or close authority. The adapter,
orchestrator, plan, and action may neither duplicate nor close the borrowed descriptor. Its lifetime
must prevent retention beyond the caller's borrow.

Deterministic injected tests must not require or synthesize a real descriptor. The boundary must
therefore preserve an identity-only injected/test representation, or an equivalent generic borrowed
capability abstraction, while making a real `AsRawFd` borrow explicit only for the live adapter.
Neither representation may imply that identity proves existence or validates the target.

## 4. Frozen `LiveCaptureBoundary`

Exactly one internal `LiveCaptureBoundary` implementation may map the orchestrator operations to
the existing R29 clock, resource, procfs, and file wrappers and the refined R30 perf ownership
boundary. It is Linux/x86_64-only and external-dependency-free. It must preserve the existing typed
outcomes, checked arithmetic, source identities, units, scopes, lifecycle order, first causal
failure, deterministic cleanup-failure order, and fail-closed result rules.

The adapter must keep process and thread `getrusage` observations distinct from one another and
from perf page-fault and context-switch observations. It may not merge, reconcile, substitute, or
claim equivalence among those sources.

The adapter must not access tracefs. It must carry only the exact R31 missing states and reasons:
`not_collected` with the deliberate-non-invocation reason, or `unsupported` only with separately
retained preflight evidence. It may not generate that evidence by probing tracefs.

No constructor, method, or successful wrapper result may validate a target merely because it
exists or a descriptor can be borrowed. Target validation requires a later explicitly authorized
caller and a retained validation result. R32 authorizes neither.

## 5. Independent per-event perf ownership

The aggregate four-counter live session must be refined into one independently owned session per
event. For each CPU-cycles, instructions, page-faults, and context-switches event:

1. its open produces its own typed outcome;
2. each successful open uniquely owns exactly one descriptor;
3. that owner performs reset then enable;
4. stopping performs disable then read with the existing R30 classification and scaling rules; and
5. cleanup closes that descriptor exactly once.

An unavailable, permission, error, or overflow outcome for one event must not erase another event's
result. Every acquired owner is cleaned in reverse acquisition order on success or failure. The
first causal error remains primary and every later cleanup failure is retained deterministically.

The existing aggregate public `PerfCounterSession` behavior must remain compatible. If retaining
it is mechanically impossible, the implementation PR must document an explicit compatibility
disposition in synchronized governance documentation before changing that behavior; silence or an
accidental semantic change is prohibited. No new aggregate policy may weaken per-event retention.

## 6. Exactly one prospective implementation PR

Exactly one later PR may implement only this internal adapter boundary. It may modify:

- `experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/lib.rs`;
- `experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/linux_capture.rs`;
- `experiments/exp-0001/crates/exp1-descriptive-d1-harness/src/orchestration.rs`;
- at most one new adapter module in that crate, and only if necessary; and
- synchronized project-status, research-roadmap, traceability, assumptions/unknowns, and execution-readiness documentation.

It must add no `Cargo.toml` or `Cargo.lock` change, dependency, crate, fixture, workflow, or
toolchain change. It must add no live caller, binary, CLI, test invocation, automatic probe, or
workload. Tests and CI must remain wholly synthetic and must never invoke clocks, `getrusage`,
procfs, `statx`, `fstat`, perf, tracefs, or any other host interface. A live implementation existing
behind an uncalled boundary is not live readiness or observation evidence.

The synthetic gate must cover borrowed-file identity/capability separation without a real FD,
mapping of every orchestration operation, each per-event perf open disposition, independent result
retention, reset/enable and disable/read order, reverse exactly-once cleanup after every acquisition
point, primary-versus-cleanup failure ordering, aggregate compatibility, distinct resource scopes,
and absence of tracefs and host calls. Completion requires exact-head review and both unchanged R9
workflows successful. Success would be bounded deterministic adapter correctness evidence only.

## 7. Retained exclusions and later gates

R32 does not authorize target probing or validation, workload materialization or execution, capture
publication, append integration or R7 record production, benchmark execution, publication,
performance conclusions, D2/D3, `fsync`, durability, recovery, faults, adapters or baselines outside
this internal capture adapter, production code, networking, servers, queries, or distributed
behavior.

A later authority must separately freeze an explicit caller, target-validation result and retention,
record production/validation, calibration and overhead, workload materialization, and execution
gate before any descriptive run. Confirmatory tracefs and every attribution or loss claim remain
blocked behind their separate interface, privilege, calibration, sentinel, drain, and validation
decisions.
