# R15 — R14 Closure and BLK-009 Next Gate

**Authority base:** live `main` at `2519be01db5360d4ccf7338d287270239645d246` (PR #58)
**Exact reviewed R14 head:** `78b8b35e4efda44a8097db05f396679a1265a239`
**Status:** complete documentation/governance decision; the BLK-009 documentation increment is prospective and effective only when this record is reviewed and merged

## 1. R14 closure evidence

PR #58 merged R14 into `main` as `2519be01db5360d4ccf7338d287270239645d246`.
Its exact reviewed head was `78b8b35e4efda44a8097db05f396679a1265a239`, and the
exact-head Documentation validation and EXP-0001 Slice A validation checks passed.

R14 froze canonical `EXP-0001-SEMANTIC-OP-v1` operation bytes, ordered workload-stream
bytes, `SHA-256/FIPS-180-4` workload-stream preimages and literal vectors, and the logical
metadata obligations that a later BLK-009 manifest must bind. It therefore resolves BLK-008
as documentation design only. It created no executable generator or conformance
implementation, generated workload, manifest implementation or artifact, validator,
benchmark evidence, persistence, durability or fault work, or later-slice implementation.
The continuation gate passed only at this reviewed documentation/governance evidence boundary.

## 2. BLK-009 prerequisite analysis

The readiness-plan dependency order is authoritative: R2 defines logical manifest obligations,
R7 defines artifact/provenance rules and a reusable JCS profile, R12 freezes generator inputs,
and R14 freezes the stream bytes and digest that BLK-009 must bind. Those reviewed inputs are
now sufficient to freeze BLK-009 independently as documentation design.

The focused freeze must define one versioned, closed-world workload-manifest field contract and
canonical serialization that binds:

- workload identity, workload-contract version, content/envelope/temporal/permutation profiles,
  and every generator profile identifier selected by R12;
- semantic-operation profile `EXP-0001-SEMANTIC-OP-v1`, workload-stream profile and version,
  byte length, and immutable stream reference when bytes are external;
- digest algorithm/profile `SHA-256/FIPS-180-4`, domain
  `rusty-data-os/exp1/workload-stream/v1`, and lowercase digest value;
- total, warm-up, measured, and per-segment/per-semantic-class operation counts, including rules
  that reject disagreement with the stream header or operation segments;
- the exact authority revisions needed to interpret the workload and profiles; and
- immutable manifest identity plus correction/supersession and provenance links.

R7's `EXP1-R7-JSON-JCS-1` I-JSON/JCS rules are sufficient reusable serialization input, but do
not themselves settle the workload-manifest schema. The BLK-009 freeze must explicitly adopt or
version the applicable JCS rules, field types, optional/absent representation, canonical bytes,
manifest digest treatment, and independently checkable documentation vectors. All objects and
nested objects must be closed: an unsupported schema/profile/version, unknown or duplicate
field, missing required field, invalid enum/type/range/order, noncanonical bytes, unresolved or
mismatched reference, inconsistent identity/profile/count/digest, or supersession cycle is
rejected rather than ignored or repaired.

Published manifest bytes are immutable. A correction receives a new identity, preserves the old
object, states its reason, and links the superseded identity through R7 provenance; forks remain
invalid until an explicit later correction resolves them. URI alone is never identity, and an
external stream reference must bind artifact identity, normalized URI, byte length, and digest.

The workload manifest identifies intended deterministic input. Machine, OS, filesystem, build,
subject/baseline effective configuration, runtime deviations, lifecycle observations, faults,
and results belong in the later R7 environment, series/run, configuration, validation, and raw-
result records. The manifest may bind immutable references to those separately governed records
where required, but must not duplicate observations or imply that a run occurred.

No executable validator, generated manifest, generated workload, or generator implementation is
needed to review a physical field/serialization freeze and literal documentation vectors. If an
executable oracle becomes necessary, or the authorities cannot yield one unambiguous canonical
encoding, the increment fails closed and reports the exact blocker.

## 3. Boundary distinctions

1. **BLK-009 documentation design** freezes the schema, canonical bytes, vectors, validation
   dispositions, identities, references, and correction rules. This is the only prospective work.
2. **Manifest or validator implementation** creates executable serialization or validation and
   requires separate build, dependency, workspace, test, and CI authorization.
3. **Deterministic generator implementation** realizes R12 algorithms in code and remains a
   separate unresolved executable increment.
4. **Generated workload artifacts** are outputs of later authorized tools, not documentation
   vectors, and are not authorized.
5. **Descriptive execution readiness** additionally requires a validated runnable cell,
   environment, stream, adapter, instrumentation, and result path; it remains incomplete.
6. **Confirmatory execution readiness** additionally requires all correctness, platform,
   apparatus, matrix, statistical, and owner gates; it remains incomplete.

## 4. Prospective authorization

After R15 is reviewed and merged, exactly one next increment may create a focused BLK-009
workload-manifest serialization documentation authority implementing section 2. It may reuse R7
JCS rules only explicitly, freeze literal positive and negative documentation vectors, and
synchronize governance registries. It must not create executable schemas, code, fixtures,
generated artifacts, or observations.

This authorization does **not** authorize BLK-009 implementation or a validator, generator
implementation, workload/manifest generation, Slice C/B1, append/write/fsync/storage,
persistence/recovery/durability, adapters, capture, instrumentation implementation, fault work,
descriptive or confirmatory execution, benchmarks, measurements, performance claims, production
crates, server/network/query/distributed behavior, dependencies, Cargo, scripts, workflows, or
workspace/toolchain changes.

## 5. Retained blockers and revisit conditions

BLK-009 remains open until that separate documentation freeze is reviewed. Generator and
validator implementation remain absent. BLK-010/020/025/026/027 remain constrained by their
existing staged authorities, and BLK-015 remains open for platform durability claims and
execution. No implementation, generated artifact, persistence, benchmark, fault, execution, or
later-slice gate is advanced by R15.

Revisit and stop if the proposed schema conflicts with R2, R7, R12, or R14; cannot bind all R14
metadata without importing run observations; lacks independently checkable canonical vectors;
needs an executable oracle; or if live `main` changes the dependency order. Such a finding does
not broaden the increment.
