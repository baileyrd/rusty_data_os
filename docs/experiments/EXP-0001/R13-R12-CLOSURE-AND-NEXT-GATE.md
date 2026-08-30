# R13 — R12 Closure and Next Gate

**Record:** `EXP-0001-R13/stream-digest-freeze-authorization-v1`
**Decision date:** 2026-08-30
**Authority base:** reviewed `main` at `1a891f34b56e6a7ce053d4f0c6dbd50c45bf1ac2` (PR #54)
**Exact reviewed R12 head:** `e39551e64d9a799a3d15bf75aa70a323c8e40ca8`
**Status:** complete documentation/governance decision; the BLK-008 documentation increment is prospective and effective only when this record is reviewed and merged

## 1. R12 closure evidence

PR #54 merged as `1a891f34b56e6a7ce053d4f0c6dbd50c45bf1ac2`; its exact reviewed head was
`e39551e64d9a799a3d15bf75aa70a323c8e40ca8`. The exact-head **Documentation
validation** workflow and the unchanged **EXP-0001 Slice A** validation workflow succeeded.
The reviewed diff selected:

- payload profiles `EXP-0001-SHA256-CTR-v1`, `EXP-0001-SHA256-MOTIF-v1`, and
  `EXP-0001-ZERO-v1`;
- typed, distinctly namespaced, deterministic UUIDv4-shaped identities under
  `EXP-0001-UUID4-SHA256-v1`;
- canonical typed input encoding and domain separation, explicit failure classifications,
  reference-selection rules, complete envelope inputs, logical-effective-time derivation,
  collision handling, compatibility/correction rules, and documentation vectors; and
- BLK-006 and BLK-007 resolution **as documentation design/specification only**.

The R12 diff contained no Rust, test, Cargo, lockfile, script, executable fixture, generated
workload, dependency, workflow, toolchain, workspace, benchmark, Slice C, persistence, or
execution artifact. This closure is reviewed documentation/governance evidence only. It is not
generator implementation, executable conformance, comparable-stream generation, BLK-008/009
completion, benchmark evidence, persistence, durability, recovery, or performance evidence.

## 2. Dependency and candidate audit

The readiness-plan order is authoritative: BLK-006/007 precede BLK-008, BLK-009 depends on
BLK-006–008, and executable harness work depends on BLK-008–010/019/025. Mechanism ordering in
the staged table does not override those explicit gates.

| Candidate | Prerequisite analysis | Disposition |
|---|---|---|
| Generator implementation / executable conformance | R12 supplies a complete documentation algorithm and independently inspectable vectors, so correctness-only implementation could later be separated from workload generation and could test exact OP1/PAY1/ID1/ENV1 encodings, payload boundary/invalid cases, UUID pre/post-mask values, references, collision failure, and logical-time vectors. It is not a prerequisite to freezing BLK-008's canonical bytes: R12 A01 already specifies those semantic inputs. R9/R10 authorized the existing workspace, dependency-free build, and CI only for Slice A and unchanged Slice B reuse; R11 explicitly authorizes no executable reuse. SHA-256 is absent from the dependency allowlist and standard library, so implementation requires either a reviewed dependency (including version, license, build, provenance, offline availability, alternatives, and oracle independence) or separately reviewed additional implementation surface. BLK-020/026/027 therefore require a new prospective executable/build authorization first. | Not authorized; no code, fixture, dependency, or generator is added. |
| BLK-008 documentation completion | R7 already freezes SHA-256 and the workload-stream digest domain. R12 now supplies the upstream semantic values and A01 formula. What remains is one versioned canonical semantic-operation/stream byte serialization, boundaries and ordering, inclusion/count rules, warm-up/measured separation, failure rules, and independently established expected digest vectors. This work can be reviewed from literal documentation bytes with independent SHA-256 calculations; it needs neither executable generator conformance nor BLK-009 physical manifest serialization. | Smallest prerequisite-first increment; documentation freeze only is prospectively authorized. |
| BLK-009 documentation completion | R2 supplies the logical field obligations, and R7 supplies artifact provenance plus a JCS profile that may be reused only after separate review. A future record must freeze the physical manifest field contract, canonical serialization, schema/version identity, digest linkage, validation/failure rules, correction/supersession, and vectors. Because the manifest must bind the completed BLK-008 stream digest and BLK-009 explicitly depends on BLK-006–008, it cannot precede the BLK-008 freeze. It does not require a generated manifest or validator once its prerequisite is complete. | Remains open; not coupled into R13's prospective increment. |
| Slice C / B1 mechanism | R1/R3/R5 specify boundaries, replay, gaps, lifecycle, B1 framing, CRC-32C, append/sync, and finalization as design. That is not executable authorization. BLK-015 still blocks applicable platform durability claims; BLK-017 is design-only and remains dependent on that boundary; BLK-020 has no later-slice adapter/capture/fault/result scope; BLK-026/027 do not authorize later workspace/build/CI reuse; and deterministic input, digest/manifest, correctness-oracle, and fault/result gates remain incomplete. Mechanism implementation is distinct from comparable workload generation, descriptive execution, and confirmatory execution; none is unlocked by the staged Slice C label. | Not implementation-ready and not authorized. |

There is no owner or architecture ambiguity that requires a discretionary choice. The explicit
dependency edge makes the independently reviewable BLK-008 documentation freeze narrower than
BLK-009, any executable generator/build authorization, or Slice C.

## 3. Prospective authorization: BLK-008 documentation freeze only

After this record is reviewed and merged, the sole next increment may create one focused
documentation/research authority that completes BLK-008. It may:

1. preserve R7's selected SHA-256 algorithm and workload-stream domain rather than reselecting
   them;
2. freeze a versioned canonical byte construction for each R12 semantic operation and the
   ordered workload stream, including unambiguous field tags, lengths, counts, ordering,
   segment boundaries, empty/boundary cases, and rejection classifications;
3. supply independently checkable preimage and expected-digest vectors, including a compact
   R12 A01-derived anchor and negative/substitution cases; and
4. state the exact digest values and metadata that later BLK-009 manifest work must bind, without
   selecting that manifest's physical serialization.

The increment must fail closed on conflict or on any need for an executable oracle. Its vectors
must be reproducible with a general independent SHA-256 implementation from literal documented
bytes; they must not be produced by adding a repository script, fixture, generator, validator,
dependency, or executable artifact.

This authority does **not** authorize generator implementation or conformance code, BLK-009
completion, a workload or manifest, BLK-008/009 validators, workspace/toolchain/dependency/CI
changes, Slice C/B1, append/write/sync, persistence, adapters, capture, faults, benchmarks,
descriptive execution, confirmatory execution, or any observation or claim.

## 4. Retained blockers and revisit conditions

BLK-009 remains open behind BLK-008. BLK-015 remains open for platform durability claims and
execution. BLK-020/026/027 remain unresolved outside their reviewed Slice A and unchanged Slice B
boundaries; no existing executable authority may be inferred for a generator or later slice.
Generator implementation and executable conformance remain absent. Slice C/B1 and all
filesystem/storage, recovery, durability, adapter, instrumentation, result-capture, fault,
benchmark, production-crate, server, query, networking, distributed, and architecture-promotion
work remain unauthorized.

Revisit the prospective decision if the semantic-stream serialization cannot be specified
without BLK-009, if independent documentation vectors disagree, if R12 A01 lacks an input needed
by the R7 digest domain, if a repository executable becomes necessary, or if live authority
changes the dependency order. Such a finding stops the increment; it does not authorize coupling,
implementation, workload generation, or execution.

## 5. Validation and exclusion audit

R13 changes governance Markdown only. Its required closure validation is the unchanged R9
sequence (`cargo fmt`, `cargo clippy --locked --offline`, and `cargo test --locked --offline` for
the EXP-0001 workspace), Markdown-link validation, `git diff --check`, repository-wide terminology
and blocker consistency searches, dependency-order review against the readiness plan, and a diff
name/type audit proving that no executable/runtime artifact entered the change. Exact-head
Documentation validation and the unchanged EXP-0001 Rust workflow remain mandatory before merge.
