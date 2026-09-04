# R34 — R33 Target-Preflight Governance Correction

**Status:** Complete documentation/governance correction; supersedes only the conflicting R33
target-mismatch and measured-file-ownership clauses and prospectively authorizes one corrected
synthetic-tested implementation PR
**Scope:** EXP-0001 preflight subset for a prospective Fedora 44 Linux/x86_64 target
**Evidence classification:** governance correction only; no implementation, Fedora-release
validation, live observation, execution, or performance evidence
**Authority date:** 2026-09-04

## 1. Conflict found at the R33 implementation gate

Closed, unmerged PR #114 was re-reviewed at exact head
`41d30f3484609b6b6ec03918bfd04b03ac56273a`; both head-specific workflows passed. The review found
that R33 simultaneously required classified artifacts for compile-time target mismatch and
valid-request file-open failure, required every artifact to state
`measured_file_ownership_release:"drop_completed"`, and supplied no truthful architecture value
for a non-x86_64 compilation target. No `File` is acquired in either pre-open case, so ordinary
drop cannot truthfully have completed for measured-file ownership. This is a governance defect,
not implementation or correctness evidence.

PR #114 also introduced public states and serialized values not authorized by R33:
`TargetArchitectureV1::Unsupported`, `PlatformDispositionV1::UnsupportedTargetOs`,
`TargetPreflightCallDispositionV1::GovernanceConflict`,
`NotAttemptedReasonV1::GovernanceConflict`, `OwnershipReleaseV1::PendingDrop`,
`unsupported_target_os`, and architecture value `unsupported`. The PR must not merge at that head.
R34 does not close it, adopt its code, or treat its passing workflows as evidence.

## 2. Minimal corrected artifact contract

R34 preserves the R33 schema name and every byte of its successful fictional vector. It changes
only the closed values needed to represent the already-required failure paths truthfully.

The `platform.disposition` value is exactly one of:

- `prospective_fedora_44_linux` when compile-time `target_os` is `linux`; or
- `unsupported_target_os` otherwise.

The `architecture.observed` value is exactly one of:

- `x86_64` when compile-time `target_arch` is `x86_64`; or
- `unsupported` otherwise.

These are compile-time target classifications, not host observations. Platform validation occurs
before architecture validation and both occur before file open. A non-Linux target produces the
first causal failure `platform_validation/platform`; Linux on a non-x86_64 architecture produces
`architecture_validation/architecture`. Either artifact is `invalid`, and no file operation or
later source check is attempted.

`lifecycle.measured_file_ownership_release` is exactly one of:

- `not_acquired` when no measured `File` owner was acquired; or
- `drop_completed` only after an acquired measured `File` has actually been dropped.

`not_acquired` is required for target-mismatch and measured-file-open-failure artifacts. It is not
a release claim, cleanup success, or OS-close observation. `drop_completed` retains exactly R33's
meaning. Construction may use an internal pre-drop staging state, but that state is not public,
not an artifact value, and cannot be serialized. In particular, `pending_drop` is not a permitted
serialized string or public contract state.

A valid-request open failure therefore remains an `invalid` artifact whose measured-file open
outcome and `measured_file_open/measured_file` first causal failure preserve the same normalized
I/O reason, whose dependent fields are not attempted using that failure ID, and whose ownership
fact is `not_acquired`. A metadata failure occurs after acquisition: it records open success,
records a `measured_file_regular_file/measured_file` causal error, drops the file, and records
`drop_completed` before serialization.

## 3. Call and retention dispositions remain closed

R34 does **not** extend R33's four `TargetPreflightCallDispositionV1` values or its
`NotAttemptedReasonV1` values. They remain exactly:

- call: `request_invalid`, `completed`, `serialization_failed`, or `retention_failed`; and
- retention not attempted: `RequestInvalid(RequestFailureReasonV1)` or
  `SerializationFailure`.

There is no `governance_conflict` call disposition or retention reason. Once this correction is
merged, every valid request can produce the R33-required classified artifact without asserting
unacquired ownership. Invalid request text remains the only pre-open case that returns no artifact.

All other R33 field order, wrapper mappings, failure IDs and precedence, classification rules,
serialization, retention behavior, byte oracle, exclusions, and closed values remain unchanged.
In particular, a causal failure is always `failure-0001` when present; cleanup failures receive
the following contiguous IDs in fixed cleanup order. Without a causal failure, the first cleanup
failure is `failure-0001`.

## 4. Corrected implementation authorization and gate

After R34 merges, exactly one later PR may revise or supersede the unmerged PR #114 implementation
within R33's existing file scope. It must remove the unauthorized governance-conflict states,
make any pre-drop staging state private and nonserializable, and implement only the corrected
closed values above. R33's prohibition on live invocation, host observation, parsers, dependencies,
manifest/lockfile/workflow/toolchain changes, and all wider execution remains in force.

The synthetic gate remains the full R33 section 8 gate, not merely regression tests for this
correction. Tests must exercise every closed wrapper tag and reason, every source metadata and
value field, every acquisition and failure point, target mismatch, open and metadata failure,
ownership sequencing, every causal/cleanup combination and ID rule, fail-closed construction,
exact serialization and escaping, path non-retention, and retention success, partial-write then
failure, immediate write failure, and flush failure. The frozen `IoErrorKindV1` mapping includes
`filesystem_loop` and `in_progress` where representable by the pinned toolchain; every future or
unlisted variant maps to `other`.

Passing synthetic tests would be bounded deterministic correctness evidence only after exact-head
review and CI. It would not establish live readiness, Fedora-release/effective-target validation,
host observation, capture, workload or benchmark execution, tracefs, durability, recovery, faults,
baselines, production readiness, or performance evidence.
