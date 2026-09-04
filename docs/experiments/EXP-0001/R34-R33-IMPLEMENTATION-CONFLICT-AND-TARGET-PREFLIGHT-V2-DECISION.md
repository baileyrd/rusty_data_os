# R34 — R33 Implementation Conflict and Target-Preflight V2 Decision

**Status:** Complete documentation/governance decision; PR #114 is closed and unmerged; exactly one later
synthetic-tested v2 implementation PR is prospectively authorized
**Scope:** EXP-0001 target-preflight representation correction only
**Evidence classification:** exact-head implementation review finding and prospective schema design;
no implementation closure, live observation, execution, or performance evidence
**Authority date:** 2026-09-04

## 1. Review finding and disposition

Closed, unmerged PR #114 was re-reviewed at exact head
`41d30f3484609b6b6ec03918bfd04b03ac56273a`; both
head-specific workflows succeeded. The metadata-phase and failure-ID corrections are directionally
correct, but the head is not mergeable under R33. It adds public states and wire strings that R33's
closed v1 contract does not authorize, and its eight tests do not satisfy R33 section 8's exhaustive
synthetic gate. Green CI does not override the frozen authority.

The conflict is a governance defect rather than a code-only defect. R33 requires classified
artifacts for compile-target mismatch and valid-request file-open failure, while v1 can describe
only observed `x86_64` and `drop_completed`. It therefore cannot truthfully represent either an
unsupported compile target or the fact that measured-file ownership was never acquired. PR #114
was not merged and must not be treated as correctness evidence. Its unmerged Rust surface created
no public compatibility obligation.

R33's v1 schema, byte vector, enum sets, and strings remain unchanged historical authority. R34
supersedes only R33's implementation authorization and freezes the minimum v2 correction below.
There is no v1/v2 mixing, implicit fallback, or reinterpretation of a v1 artifact.

## 2. V2 identity and exact representation correction

The public schema constant and corrected artifact schema identifier are exactly:

```rust,ignore
pub const TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2: &str =
    "EXP-0001-R34/target-preflight-artifact-v2";
```

All R33 field order, mappings, source/perf order,
failure rules, retention rules, input rules, exclusions, and serialization rules carry forward
unchanged except where this document explicitly replaces them.

The sole later public entry point is exactly:

```rust,ignore
pub fn run_target_preflight(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> TargetPreflightExecutionV2
```

`TargetPreflightRequest<'_>` and `TargetPreflightRetention` are reused unchanged from R33. The
following R33 public model names are also reused unchanged because neither their closed values nor
their wire identity changes: `RequestFailureReasonV1`, `RetentionOutcomeV1`,
`RetentionIoErrorV1`, `IoErrorKindV1`, `UnavailableReasonV1`, `OverflowReasonV1`,
`ErrorReasonV1`, `UnitV1`, `FailureObjectV1`, and `TargetPreflightCallDispositionV1`.

Exactly these public models are v2: `TargetPreflightArtifactV2` (schema identity and nested
representation changed), `TargetPlatformV2` (its disposition set changed),
`PlatformDispositionV2`, `TargetArchitectureV2` (its observation set changed),
`ArchitectureObservationV2`, `TargetPreflightLifecycleV2` (its ownership set changed),
`MeasuredFileOwnershipReleaseV2`, and `TargetPreflightExecutionV2` (its artifact member is v2).
`TargetPreflightExecutionV2` contains exactly `{ artifact: Option<TargetPreflightArtifactV2>,
serialized_bytes: Option<Vec<u8>>, retention: RetentionOutcomeV1, disposition:
TargetPreflightCallDispositionV1 }`. Every other artifact member uses the unchanged R33 model and
wire mapping. No `TargetPreflightCallDispositionV2` exists: that closed set did not change.
Closed, unmerged PR #114 creates no reason to preserve an inert duplicate v1 API, so the later
implementation must not add or retain duplicate v1 Rust entry points or artifact/execution APIs
unless a later repository authority first identifies a concrete repository need.

The following v2 sets are closed:

- request `expected_platform` remains exactly `fedora-44-linux`, and request
  `expected_architecture` remains exactly `x86_64`;
- `platform.disposition` is exactly `prospective_fedora_44_linux` or
  `unsupported_target_os`;
- `architecture.observed` is exactly `x86_64` or `unsupported`;
- `lifecycle.measured_file_ownership_release` is exactly `drop_completed` or `not_acquired`.

`unsupported_target_os` means only that `cfg!(target_os = "linux")` is false. `unsupported` means
only that `cfg!(target_arch = "x86_64")` is false. Neither value identifies another OS or
architecture, performs host observation, or establishes Fedora/effective-target validation.

`not_acquired` is permitted only when no measured-file `File` owner was obtained because target
validation failed or the read-only open failed. `drop_completed` is permitted only after an
acquired `File` has actually been dropped. A pending construction state may exist privately, but
`pending_drop` is not a public enum value or serialized string and a pending artifact cannot be
serialized or retained.

V2 does **not** add a governance escape hatch. Reused `TargetPreflightCallDispositionV1` remains
exactly R33's four values: `request_invalid`, `completed`, `serialization_failed`, and
`retention_failed`. Retention not-attempted reasons remain exactly
`RequestInvalid(RequestFailureReasonV1)` or `SerializationFailure`. `governance_conflict` is not a
call disposition, not a retention reason, and not serialized. The R33 v1 strings
`unsupported_target_os`, architecture `unsupported`, and ownership `not_acquired` remain invalid;
they are valid only in an artifact bearing the exact v2 schema identifier.

## 3. Required mismatch and acquisition semantics

Compile-time target checks occur after textual request validation and before file open. A platform
mismatch is the first causal failure at `platform_validation/platform`, has `invalid_state` detail
`platform_mismatch`, records platform `unsupported_target_os`, records architecture from the
compile-time architecture check, performs no file open, and records ownership `not_acquired`. When
platform matches but architecture does not, the corresponding first causal failure is
`architecture_validation/architecture` with detail `architecture_mismatch`; file open is not
attempted and ownership is `not_acquired`. Platform failure has precedence if both mismatch.

After a valid request and matching compile target, read-only open failure produces an `invalid`
artifact whose first causal failure is `measured_file_open/measured_file`, whose open outcome and
failure detail losslessly encode that error, whose dependent regular-file and file-length checks
are `not_attempted` using that same failure ID, and whose ownership release is `not_acquired`.

After open succeeds, metadata failure preserves open success. It is the first causal failure at
`measured_file_regular_file/measured_file`; the regular-file outcome and failure detail losslessly
encode the metadata error, dependent file-length/capture work is not attempted using that failure
ID, and ownership is `drop_completed` only after drop. A non-regular file uses the same phase/source
with `invalid_state/not_regular_file` and the same post-drop rule.

For target mismatch and open failure, lifecycle phases contain only phases actually completed and
must not contain `file_opened` or `ownership_released`. `ownership_released` appears only after an
acquired owner has been dropped and the release field is `drop_completed`. The
`measured_file_ownership_release` field records the terminal ownership fact; it does not by itself
assert that an `ownership_released` phase occurred.

Failure IDs retain R33 ordering: when a causal failure exists it is `failure-0001`, followed by
cleanup failures in fixed cleanup occurrence order; without a causal failure the first cleanup
failure is `failure-0001`. IDs are assigned only after the causal and cleanup sets are known.

## 4. V2 construction and serialization invariants

The typed artifact is the sole source of every serialized platform, architecture, lifecycle,
outcome, and reason value. The serializer must not hardcode a successful target independently of
the artifact. Every public construction path must validate the complete artifact and fail closed;
it cannot return `TargetPreflightArtifactV2` for any contradictory combination. Any private staged
or partially constructed representation remains internal. Serialization independently revalidates
the complete artifact and rejects every internally staged, pending, or contradictory value before
it returns a buffer or offers any bytes to `TargetPreflightRetention`. Construction and
serialization reject every cross-field contradiction, including:

- v1 schema identity with any v2-only value;
- `not_acquired` after open success or `drop_completed` without acquisition and completed drop;
- target/open failure with `file_opened` or `ownership_released` phases;
- pending ownership presented for serialization;
- a skipped operation referring to any ID other than its causal dependency;
- an open-success/metadata-error artifact reported as `measured_file_open`; or
- noncontiguous or cleanup-before-causal failure IDs.

All R33 normalized `IoErrorKindV1` strings remain frozen, including `filesystem_loop` and
`in_progress` where representable on Rust 1.89.0; future or unlisted non-exhaustive variants map to
`other`. V2 introduces no dependency, parser, round-trip API, CLI, live invocation, or host test.

The successful v2 JSON-lines byte oracle is unambiguous: take the complete successful R33 section
7 oracle, including its final LF, and replace only the schema identifier value
`EXP-0001-R33/target-preflight-artifact-v1` with
`EXP-0001-R34/target-preflight-artifact-v2`. Every other byte remains identical. No other
successful v2 oracle is authoritative.

## 5. Exhaustive synthetic implementation gate

One later PR may implement only the v2 boundary above in the existing
`exp1-descriptive-d1-harness` target-preflight module and re-export. It must not emit a newly
constructed v1 artifact or accept mixed v1/v2 values. Because closed, unmerged PR #114 created no
compatibility surface, this authority does not authorize inert duplicate v1 definitions absent a
separately documented concrete repository need. Cargo manifests, lockfiles, dependencies,
toolchain, workflows, fixtures, and the
frozen `linux_capture`, `orchestration`, and `live_adapter` modules remain unchanged.

Before closure, synthetic tests must exercise, by table-driven coverage where practical:

1. every closed wrapper outcome tag and every nested unavailable, overflow, error, parse, and I/O
   reason, including explicit pinned-toolchain checks for `filesystem_loop`, `in_progress`, and
   fallback `other`;
2. every source and perf identity, scope, unit member, success-value field, acquisition point,
   causal failure point, cleanup failure point, and dependent `not_attempted` mapping;
3. platform mismatch, architecture mismatch, simultaneous mismatch precedence, open failure,
   metadata failure, non-regular file, success, causal-only, cleanup-only, and combined
   causal/cleanup paths;
4. ownership sequencing and rejection of every contradictory `not_acquired`, `drop_completed`,
   lifecycle-phase, schema-version, and pending-construction combination;
5. exact JSON-lines bytes, escaping, field/array order, non-ASCII rejection, path non-retention,
   deterministic repeatability, and typed-artifact-derived target values; and
6. request-invalid, serialization-not-attempted, successful retention, a partial write followed by
   failure inside `write_all`, immediate write failure, and flush failure, proving retention never
   mutates the already-fixed artifact bytes.

Test count is not the gate: the assertions must cover the complete closed cross-product surface
where combinations affect causality, cleanup, classification, lifecycle, or serialization. Tests
and CI invoke no live host boundary and retain no host observation.

## 6. Authorization boundary

The later implementation PR must run the unchanged R9 validation sequence and `git diff --check`.
Its success would be bounded deterministic conformance/correctness evidence only. It would not
establish live readiness, Fedora/effective-target validation, host observation, workload or
measured action, append/R7 production, tracefs, calibration, capture, benchmark execution or
publication, performance, D2/D3, `fsync`, durability/recovery/fault behavior, baseline equivalence,
or production readiness. All such work remains blocked or unauthorized.
