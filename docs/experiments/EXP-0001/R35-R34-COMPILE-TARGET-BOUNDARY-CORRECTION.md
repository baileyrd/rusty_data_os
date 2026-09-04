# R35 — R34 Compile-Target Boundary Correction

**Status:** Complete documentation/governance correction; exactly one later synthetic-only v2 implementation PR is prospectively authorized after R35 merges
**Scope:** EXP-0001 target-preflight compile-target boundary only
**Evidence classification:** repository-boundary review and prospective governance design; no Rust implementation, live observation, target validation, execution, or performance evidence
**Authority date:** 2026-09-04

## 1. Conflict and decision

R34 correctly established that a valid-request measured-file open failure needs an ownership fact distinct from successful drop. It incorrectly required the public preflight to produce platform- and architecture-mismatch artifacts at run time. The existing fourth crate has an unconditional `compile_error!` outside Linux/x86_64; its `orchestration` module imports Linux capture types, and its capture and live-adapter modules are Linux/x86_64 boundaries. Consequently, no public function in this supported crate can execute on an unsupported Rust compilation target. Making those paths executable would require a broader portability refactor that no authority permits.

R35 preserves that crate boundary. Unsupported operating systems or architectures are build-time rejections, not preflight artifacts, call dispositions, retention outcomes, or observations. `run_target_preflight` is compiled and callable only when `all(target_os = "linux", target_arch = "x86_64")` is true. An implementation must not add runtime `cfg!` mismatch branches that are unreachable inside the supported crate.

R35 supersedes **only** R34's requirements for unsupported-target runtime artifact values, mismatch failure objects and lifecycle paths, and mismatch tests. R34 remains historical and is not edited. Its v2 identity, exact public entry point and type boundary, file-acquisition correction, failure ordering, construction and serialization rules, retention rules, exhaustive synthetic standard, and exclusions otherwise remain controlling.

## 2. Corrected closed v2 target sets

The v2 schema identifier remains exactly:

```rust,ignore
pub const TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2: &str =
    "EXP-0001-R34/target-preflight-artifact-v2";
```

The sole later public entry point remains exactly the R34 signature, under the crate's existing Linux/x86_64 compilation boundary:

```rust,ignore
pub fn run_target_preflight(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> TargetPreflightExecutionV2
```

The request values remain exactly `expected_platform = "fedora-44-linux"` and `expected_architecture = "x86_64"`. The corrected artifact value sets are closed as follows:

- `platform.disposition` has exactly one value: `prospective_fedora_44_linux`;
- `architecture.observed` has exactly one value: `x86_64`; and
- `lifecycle.measured_file_ownership_release` remains exactly `drop_completed` or `not_acquired`.

`unsupported_target_os` is not an authorized v2 platform value. Architecture `unsupported` is not an authorized v2 observation. Neither string may be represented by public v2 variants, constructed, or serialized. They do not become call dispositions, retention reasons, unavailable reasons, or failure details. The reused `TargetPreflightCallDispositionV1` and `RetentionOutcomeV1` sets remain unchanged; no governance-conflict or unsupported-target escape state exists.

The successful v2 byte oracle remains the R34 oracle: the complete successful R33 section 7 oracle with only its schema identifier replaced by `EXP-0001-R34/target-preflight-artifact-v2`. Its frozen platform and architecture values are therefore `prospective_fedora_44_linux` and `x86_64`.

## 3. Preserved file-acquisition and lifecycle correction

The R34 v2 schema remains necessary because valid textual input can be followed by read-only file open failure on the supported target. Such failure produces the R34 `invalid` artifact with first causal failure `measured_file_open/measured_file`, lossless open outcome and failure detail, dependent regular-file and length checks marked `not_attempted` using that failure ID, and `measured_file_ownership_release = "not_acquired"`. Its lifecycle does not claim `file_opened` or `ownership_released`.

After acquisition, metadata failure and non-regular-file handling retain R34's exact phase, dependency, drop, and lifecycle rules. `drop_completed` is valid only after an acquired `File` is dropped; it does not claim observable OS close success. `not_acquired` is valid only when no owner was obtained. Pending ownership remains private and unserializable. R34's causal-before-cleanup failure ordering, contiguous IDs, classification, source/perf order, request validation, typed-artifact-derived serialization, fail-closed cross-field validation, JSON-lines rules, path non-retention, and write/flush retention outcomes all remain unchanged.

## 4. Corrected exhaustive synthetic gate

The later implementation must satisfy R34 section 5 except that its target-mismatch cases are replaced by all of the following synthetic or compile-time checks:

1. prove that the compiled target constants yield only `platform.disposition = prospective_fedora_44_linux` and `architecture.observed = x86_64`;
2. prove that the public v2 target types contain no `unsupported_target_os` platform value and no architecture `unsupported` value, and that neither string can be serialized as such a value;
3. preserve deterministic coverage for open failure, metadata failure, non-regular file, success, causal-only, cleanup-only, and combined causal/cleanup paths;
4. preserve exhaustive `not_acquired` versus `drop_completed` acquisition, lifecycle, construction, and serialization contradiction tests; and
5. preserve every other R34 wrapper outcome, nested reason, source/perf metadata, failure, ordering, serializer, escaping, non-ASCII, deterministic-repeatability, request-invalid, and retention test obligation.

Tests and CI remain synthetic and must not invoke the live boundary, probe a host, or retain a host observation. Cross-compiling this crate to an unsupported target is not required: the existing compile rejection is the authority boundary, not a preflight test case.

## 5. What the compile gate does not prove

The compile gate proves only that Rust compiled the crate for a Linux/x86_64 target. It does not prove that the effective host is the intended machine, that the operating-system release is Fedora 44, or that the selected kernel, glibc, filesystem, storage, permissions, or instrumentation match the prospective profile. In particular, `prospective_fedora_44_linux` is a design disposition, not a Fedora-release observation. Fedora-release validation and effective-host validation remain unresolved under UNK-022 and cannot be inferred from successful compilation or synthetic tests.

## 6. Prospective authorization and exclusions

After R35 merges, exactly one later PR may implement the corrected v2 boundary in the existing `exp1-descriptive-d1-harness` crate. It may add only the target-preflight module, re-export, exact R34/R35 public v2 construction and serialization boundary, injected retention behavior, and synthetic tests. The existing `lib.rs`, `linux_capture.rs`, `orchestration.rs`, and `live_adapter.rs` target boundary and behavior remain unchanged except that the later PR may add the minimum `lib.rs` module/re-export declarations required for target preflight. Manifests, lockfile, dependencies, fixtures, workflows, and toolchain remain unchanged. The unchanged R9 validation sequence and `git diff --check` are the implementation gate.

R35 itself changes documentation only and supplies no implementation or correctness evidence. It authorizes no portability refactor, live invocation, host observation, target probing or validation, record or workload production, measured action, append integration, tracefs, calibration, capture, benchmark execution or publication, performance claim, D2/D3, `fsync`, durability, recovery, fault work, baseline execution, production code, or later increment.
