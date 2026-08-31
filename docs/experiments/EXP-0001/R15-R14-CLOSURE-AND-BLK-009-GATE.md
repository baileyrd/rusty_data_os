# R15 — R14 Closure and BLK-009 Gate

**Record:** `EXP-0001-R15/workload-manifest-freeze-authorization-v1`
**Decision date:** 2026-08-30
**Authority base:** reviewed `main` at `2519be01db5360d4ccf7338d287270239645d246` (PR #58)
**Exact reviewed R14 head:** `78b8b35e4efda44a8097db05f396679a1265a239`
**Status:** complete documentation/governance decision; the BLK-009 documentation increment is
prospective and effective only when this record is reviewed and merged

## 1. Exact R14 closure evidence

PR #58 merged as `2519be01db5360d4ccf7338d287270239645d246`; its exact reviewed head was
`78b8b35e4efda44a8097db05f396679a1265a239`. Exact-head **Documentation validation** and
**EXP-0001 Slice A** validation passed. The reviewed R14 authority froze:

- canonical semantic-operation bytes under `EXP-0001-SEMANTIC-OP-v1`;
- ordered workload-stream bytes under `EXP-0001-WORKLOAD-STREAM-v1`, including framing,
  warm-up/measured ordering and counts, and fail-closed parsing rules;
- SHA-256 workload-stream preimages and independently checkable documentation vectors under
  domain `rusty-data-os/exp1/workload-stream/v1`; and
- the metadata obligations a later BLK-009 manifest must bind.

BLK-008 is therefore resolved as **documentation design only**, and UNK-018 is resolved only at
that boundary. The merged diff created no executable generator, conformance implementation,
workload generation, manifest implementation, validator, generated artifact, benchmark evidence,
persistence, durability, fault work, or later-slice implementation. R14 is reviewed
documentation/governance evidence, not implementation, execution, or empirical evidence.

The R14 continuation gate passed: the literal vectors, canonicalization rules, dependency order,
and exclusion audit supplied the documentation evidence R13 required. No ambiguity, vector
disagreement, executable-oracle requirement, or changed dependency edge prevents the next
prerequisite analysis.

## 2. BLK-009 prerequisite analysis

BLK-009 depends on BLK-006–008. R12 resolved BLK-006/007 as documentation design and R14 resolved
BLK-008 at the same boundary. R2 supplies logical manifest obligations, R7 supplies artifact and
provenance rules plus reusable canonical-JSON design input, and R14 supplies the exact stream
identity and digest bindings. None requires an executable generator, generated stream, validator,
or manifest artifact merely to review a physical serialization contract.

| Concern | Required BLK-009 documentation decision | Prerequisite finding |
|---|---|---|
| Identity and versions | Bind a manifest schema/profile identifier, workload contract/version and workload identity; bind R12 generator/input profile identifiers and R14 semantic-operation and workload-stream profile identifiers exactly. | R2/R12/R14 provide the semantic values. The next record selects only their manifest field names, types, and canonical placement. |
| Stream digest | Bind `SHA-256/FIPS-180-4`, domain `rusty-data-os/exp1/workload-stream/v1`, exactly 32 digest octets rendered as 64 lowercase hex, the digest value, and an immutable reference containing byte length and digest for external semantic-stream bytes. | R7/R14 already select the algorithm/domain and immutable-reference obligations; BLK-009 must not reinterpret them. |
| Counts and segments | Bind total, warm-up, and measured operation counts; require overflow-safe `warm_up + measured = total`; bind the declared segment profiles, ordinals, seeds, operation mix, payload/envelope/reference/logical-time profiles, producer assignment, and controlled-schedule state where applicable. | R2/R12/R14 freeze the meanings and stream boundary. Inconsistent duplicated values must fail closed. |
| Environment and configuration | Include only immutable workload-definition inputs and references needed to regenerate or locate the exact stream, including authority/profile revisions and workload/generator configuration. Actual host, OS, compiler/build, storage, cache/preconditioning observation, adapter effective settings, clocks, run identity, assigned sequence, outcomes, and deviations belong to later environment/raw-result/run records. | R2 separates the frozen definition from actual result metadata; R7 supplies typed immutable references and later record relationships. |
| Artifact/provenance | Bind the semantic-stream artifact reference when external and the authority/configuration references needed for interpretation. Preserve R7 identity, SHA-256, byte-length, URI, provenance-edge, retention, and publication obligations where those artifacts enter an R7 manifest. | R7's artifact ledger remains authoritative. BLK-009 must neither duplicate it inconsistently nor pretend the workload manifest is itself an R7 `artifact_manifest` record. |
| Canonicalization | Select one closed, versioned physical serialization, exact scalar representations, member/array order, duplicate handling, Unicode policy, and literal positive/negative vectors. | R7's `EXP1-R7-JSON-JCS-1` common envelope is not directly applicable: its closed `record_kind` union does not contain a workload manifest. RFC 8785/JCS rules and R7's I-JSON, duplicate-member, integer-as-string, lowercase-digest, and closed-object choices are sufficient reusable input, but reuse requires an explicit BLK-009 profile rather than silently extending R7. |
| Immutability and correction | Make a published manifest immutable. A correction is a new manifest identity/version that explicitly names the superseded manifest and reason; historical run/result records continue to reference the exact manifest they consumed. Reject cycles, broken chains, in-place mutation, and ambiguous concurrent successors. | R7 supplies the provenance/supersession model; BLK-009 must freeze the workload-specific identity and chain representation. |
| Closed world and failure | Reject unknown, missing, duplicate, out-of-order where order is normative, ill-typed, out-of-range, noncanonical, unsupported-version, dangling-reference, digest, count, identifier/profile, and supersession inconsistencies. A reader accepts the declared version byte-for-byte or rejects it; it never guesses or ignores unknown fields. | The required behavior is fully specifiable and reviewable with documentation vectors. Executable validation remains a separate authorization. |

The analysis finds no unresolved semantic or implementation prerequisite to a **documentation-only**
BLK-009 freeze. R7's JCS profile is reusable input, not an already-selected workload-manifest
serialization. The next increment must make that selection explicitly and may stop if doing so
reveals a conflict. UNK-019 remains open until that reviewed freeze is complete.

## 3. Boundary classification

These boundaries must not be collapsed:

1. **BLK-009 documentation design** freezes the closed logical/physical field contract,
   canonical serialization, version identity, linkage, validation/failure rules, supersession,
   and literal documentation vectors. This is the only prospectively authorized work.
2. **Manifest or validator implementation** parses, emits, validates, or stores the contract.
   It remains unauthorized and requires separate workspace, dependency, and CI review.
3. **Deterministic generator implementation** realizes R12/R14 algorithms. It remains absent and
   unauthorized; BLK-020/026/027 do not authorize it.
4. **Generated workload artifacts** are concrete streams or manifests. None may be created by the
   documentation freeze.
5. **Descriptive execution readiness** additionally requires executable generation/validation,
   harness/capture/adapters, applicable platform gates, and effective configuration evidence.
   BLK-009 documentation alone does not pass it.
6. **Confirmatory execution readiness** additionally requires every confirmatory gate, apparatus,
   equivalence, freeze, and exact-head review. It remains later and blocked.

## 4. Prospective authorization: BLK-009 documentation freeze only

After this record is reviewed and merged, the sole next increment may create one focused
documentation/research authority for BLK-009. It may:

1. freeze a dedicated, closed, versioned workload-manifest field contract and canonical physical
   serialization, with explicit rationale for reuse or specialization of R7's JCS rules;
2. bind every identity, profile, digest, count, segment, generator/workload configuration,
   immutable stream reference, and authority/provenance item identified in section 2;
3. define immutable publication, correction/supersession, compatibility, unknown-field,
   version-negotiation, reference-resolution, and fail-closed validation rules; and
4. provide small literal canonical-serialization/digest vectors plus negative cases that are
   independently reviewable without repository executable tooling.

It must distinguish workload-definition data from later environment/run/result observations and
must reject all inconsistent duplicated bindings. If the dedicated contract cannot be frozen
without a new semantic decision, executable oracle, dependency, generated artifact, or R7 change,
the increment stops and reports that exact blocker rather than expanding scope.

This authorization does **not** authorize manifest/validator or generator implementation,
workload/manifest generation, scripts, fixtures, schemas executable by tooling, dependencies,
Cargo/workspace/toolchain/CI changes, Slice C/B1, append/write/fsync/filesystem/storage,
persistence, replay/recovery, durability, faults, adapters, instrumentation/capture, descriptive
or confirmatory execution, benchmarks, measurements, results, or performance claims.

## 5. Retained blockers and completion report

BLK-009 remains open until the separately reviewed documentation freeze completes. BLK-015 and
later platform/execution gates remain open. BLK-020/026/027 remain bounded to their reviewed Slice
A and unchanged Slice B uses and grant no generator, validator, harness expansion, or later-slice
authority. UNK-019 remains open; UNK-018 retains implementation/executable-conformance work and
UNK-022 retains executable/capture validation. Generator and manifest implementation, generated
workloads, Slice C/B1, descriptive execution, confirmatory execution, persistence, benchmarks,
faults, durability evidence, and production work remain unauthorized.

R15 changes only governance Markdown: this authority plus synchronized `AGENTS.md`, project
status, roadmap, research-question/unknown/traceability registries, EXP-0001, and its readiness
plan. Required validation is `git diff --check`, Markdown-link validation, the unchanged R9
`cargo fmt`, `cargo clippy --locked --offline`, and `cargo test --locked --offline` sequence,
repository-wide terminology/blocker searches, dependency-order review, and a changed-path
exclusion audit. No new executable implementation, generated workload or manifest artifact,
execution, benchmark, persistence, fault, durability, or later-slice work occurs in R15.
