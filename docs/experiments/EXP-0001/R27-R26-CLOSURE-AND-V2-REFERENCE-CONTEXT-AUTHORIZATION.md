# EXP-0001 R27 — R26 closure and v2 reference-context authorization

**Contract:** `EXP-0001-R27-V2-REFERENCE-CONTEXT-v1`
**Status:** R26 implementation closure and prospective bounded implementation authorization
**Evidence classification:** reviewed R26 conformance/correctness evidence plus governance and
implementation authorization; not workload, benchmark, durability, recovery, capture, or
performance evidence
**Decision date:** 2026-09-02

## 1. R26 implementation closure

R27 closes the implementation authorized by R26 as bounded conformance/correctness evidence. PR
#95's reviewed exact head is `35f9a0f245ac488828df4f639263edb3fb50be86`; its merge commit is
`f4ed0c310fa46c6de209ea0f776c4749e31cdd34`; both existing workflows succeeded for that exact
reviewed head. The merged change adds the side-by-side v2 profiles, ENV2/REF2/SOP2/WS2 codecs,
closed v2 manifest validation, domain-separated digests, and independently checked-in literal
fixtures and tests while preserving v1 behavior.

This evidence establishes only that the merged external-dependency-free
`exp1-workload-conformance` subset conforms to the frozen R26 contract and literal oracle. It is not
workload generation or execution, a benchmark result, reference-context closure, append/reopen
integration, recovery or durability evidence, or architecture promotion. The complete R20 gate
remains open until the separately authorized implementation below is merged, reviewed, and passes
exact-head CI.

## 2. Decision and sole follow-on authorization

R21's immutable catalog and caller-owned accepted prefix, R22's segment-local rule, R23's closed
scope, R25's valid zero-target bootstrap semantics, and the now-implemented R26 v2 oracle provide
the complete authority needed for one pure v2 contextual mapper. R27 therefore authorizes exactly
one follow-on implementation PR in the existing `exp1-raw-append-replay` crate. It shall implement
the v2 closed-scope constructor, immutable typed identity catalog, unforgeable transactional
accepted-prefix state, and contextual SOP2-to-RF1 mapping. It shall accept a zero-target operation
at ordinal zero of each segment and later prior ordinary EventIds only from the same stream and
segment.

That implementation, if its completion gate passes, closes the remaining R20 reference-context
correctness gate only as bounded implementation/correctness-validation evidence. It does not make
the old context-free mapper a full validator and does not authorize any later tranche.

## 3. Minimum R23 v2 closed-scope extension

R23 v1 behavior and bytes remain exactly unchanged. The v2 extension is a distinct, non-negotiated
profile pair:

* descriptor profile: `EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v2`;
* digest profile: `EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v2`;
* digest domain: `rusty-data-os/exp1/closed-stream-scope/v2`;
* digest preimage: `SHA-256(domain ASCII || 00 || exact descriptor bytes)`.

The v2 descriptor has exactly R23's top-level fields and member fields, ordering, JCS rules,
identity rules, artifact/reference rules, and limits, except `schema_version` is the v2 descriptor
profile. Its external digest descriptor retains R23's exact closed shape and uses the v2 digest
profile/domain. A v2 member resolves only through an exact accepted
`EXP-0001-WORKLOAD-MANIFEST-DIGEST-v2` descriptor to exact canonical
`EXP-0001-WORKLOAD-MANIFEST-JCS-v2` bytes, one exact `EXP-0001-WORKLOAD-STREAM-v2` (WS2) artifact,
the manifest's `EXP-0001-WORKLOAD-STREAM-DIGEST-v2` value, its exact byte length, and its
exact-artifact SHA-256 and immutable R7 metadata/provenance. All seven R23 member fields must equal
those resolved bindings, and the namespace must equal both the v2 manifest namespace and sole WS2
namespace.

A v1 descriptor continues to accept only the exact R16/R14 v1 manifest, WS1, artifact, and digest
bindings already frozen by R23. A v2 descriptor accepts only the exact R26 v2 bindings above.
Every member of one descriptor must have the descriptor's version. A mixed v1/v2 descriptor,
member, manifest tuple, stream, stream digest, manifest digest, scope digest, substitution, alias,
omitted version, fallback, upgrade, downgrade, or negotiation fails
`UnsupportedScopeProfile`. Reuse of the unchanged R26-approved v1 identity, payload, time, size,
temporal, and SHA-256 algorithm profile literals inside a valid v2 manifest is not mixed scope
membership.

## 4. Exact construction API and owned model

Only `exp1_raw_append_replay::reference_context` may expose this additive public surface (names and
signatures are exact):

```rust,ignore
pub struct ScopeDigestInput<'a> {
    pub descriptor: &'a [u8],
    pub artifact_metadata: &'a [u8],
}
pub struct ManifestBindingInput<'a> {
    pub manifest: &'a [u8],
    pub manifest_digest_descriptor: &'a [u8],
    pub manifest_artifact_metadata: &'a [u8],
    pub stream: &'a [u8],
    pub stream_artifact_metadata: &'a [u8],
}
pub struct ClosedScopeInputV2<'a> {
    pub scope: ScopeDigestInput<'a>,
    pub members: &'a [ManifestBindingInput<'a>],
}
pub fn construct_reference_context_v2(
    input: ClosedScopeInputV2<'_>,
    selected_stream_namespace: [u8; 16],
) -> Result<ReferenceContextV2, ContextConstructionError>;
```

`ReferenceContextV2`, `ReferenceCatalogV2`, and `AcceptedPrefixStateV2` are public but have private
fields. `ReferenceContextV2` exposes exactly `catalog(&self) -> &ReferenceCatalogV2` and
`initial_state(&self) -> &AcceptedPrefixStateV2`. The catalog exposes read-only
`scope_id() -> [u8; 16]`, `cell_id() -> &str`, `selected_stream_namespace() -> [u8; 16]`,
`stream_count() -> usize`, `source_bytes() -> usize`, `operation_count() -> usize`, and
`identity_entry_count() -> usize`. State exposes read-only `accepted_operations() -> u64`,
`previous_sequence() -> u64`, and `previous_physical_ordinal() -> u64`. None of these success types
implements `Default`, exposes mutable entry/state access, or permits arbitrary construction.
Cloning state is allowed only as an exact value copy for caller transaction management; catalog
mutation, global state, interior mutability, background work, and serialization are forbidden.

Construction retains compact owned fixed-width bindings and the selected stream's ordered complete
SOP2 bytes needed for byte-exact position matching. It must not retain payload-bearing source WS2,
manifest, descriptor, or metadata buffers after construction, fetch data, infer provenance, or
regenerate identities/references. `ReferenceContextV2` owns its catalog and initial state without
`Arc`; later mapping immutably borrows the catalog and caller state.

## 5. Exact construction errors, precedence, and bounds

The public exhaustive error is:

```rust,ignore
pub enum ContextConstructionError {
    InvalidScopeEncoding, UnsupportedScopeProfile, InvalidScopeDigest,
    ScopeReferenceFailure, InvalidCellAuthority, InvalidMemberBinding,
    OmittedStream, ExtraStream, DuplicateStreamNamespace, SubstitutedStream,
    ForeignWorkloadOrCell, SemanticValidation(exp1_workload_conformance::Error),
    IdentityCollision, ResourceLimit, Extraction, SelectedStreamMissing,
}
```

Construction is transactional and returns the first error in this exact order:

1. checked input accounting; descriptor UTF-8/JSON/I-JSON/JCS/closed-schema/scalars, v2 profile,
   member count/order/uniqueness; mixed versions fail `UnsupportedScopeProfile` here;
2. exact v2 scope-digest descriptor, immutable R7 descriptor artifact/provenance, and digest;
3. resolve each member in canonical namespace order to its exact v2 manifest-digest descriptor,
   manifest/artifact metadata, supersession, and reviewed cell authority;
4. enforce all member-to-manifest-to-artifact bindings, including exact v2 profiles and digests;
5. call `validate_manifest_v2` and `validate_stream_v2` for every complete WS2, then extract only
   validated fields; an authority failure is `SemanticValidation`, extraction disagreement is
   `Extraction`;
6. canonicalize supplied members by namespace and require exact equality: duplicate, omitted,
   extra, substituted, then foreign workload/cell are distinguished in that order;
7. enforce selected namespace exactly once, then typed global identity collisions; and
8. publish the complete immutable catalog and initial state.

No error returns a partial catalog or state. Within a stage members are examined in canonical raw
namespace-byte order; fields use document order. Earlier stages always win over later ones.

R21/R23 inclusive limits are unchanged: 262,144 descriptor bytes; 256 members/streams; 1,048,576
total manifest bytes; 4,194,304 resolved R7 metadata bytes; 16,777,216 total WS2 bytes; 65,536
operations; 196,608 typed identity bindings; and 65,536 references in one operation. Every count,
sum, product, conversion, buffer, and allocation uses checked arithmetic and is rejected as
`ResourceLimit` before allocation where knowable. Zero members is `InvalidScopeEncoding`; a missing
selected namespace is `SelectedStreamMissing`.

## 6. Catalog identity rules

Each validated SOP2 contributes Request, Event, and Information bindings with the exact R21 fields:
raw UUID bytes, role, namespace, total WS2 position, segment, segment ordinal, producer and producer
ordinal; Event additionally records ordinary/correction/retraction class. Classification comes only
from the validated R26 semantics. Identity bytes are globally unique across every role and member;
any second binding is `IdentityCollision`. The catalog never derives identity provenance from UUID
bits and never treats total WS2 position as reference eligibility.

The R26 literal WS2/manifest/operation fixtures under
`exp1-workload-conformance/tests/data/r26-v2` are the independent positive oracle. The
implementation must consume those checked-in literals through the existing public validator; it
must not copy their values into production source, generate replacements, or change the authority
crate.

## 7. Exact contextual mapping API and errors

The additive entry point is exactly:

```rust,ignore
pub fn map_semantic_operation_v2_with_context(
    semantic_operation: &[u8],
    assigned_sequence: u64,
    physical_ordinal: u64,
    catalog: &ReferenceCatalogV2,
    state: &AcceptedPrefixStateV2,
) -> Result<ContextualMappedRecordV2, ContextualMappingError>;
```

`ContextualMappedRecordV2` has private fields and exposes exactly `frame(&self) -> &[u8]`,
`record(&self) -> &exp1_record_format::Record`, and
`next_state(&self) -> &AcceptedPrefixStateV2`. The input state is borrowed and never mutated.
The exhaustive errors are:

```rust,ignore
pub enum ReferenceError {
    Missing, Future, WrongKind, WrongFact, SelfReference, CrossStream, CrossSegment,
}
pub enum ContextualMappingError {
    SemanticValidation(exp1_workload_conformance::Error),
    Discontinuity, Exhaustion, Reference(ReferenceError),
    Mapping(crate::mapping::MappingError), ResourceLimit, Extraction,
}
```

The legacy `MappingState`, `MappedRecord`, `MappingError`, and
`map_semantic_operation` remain source- and behavior-compatible. The new mapper creates the same
RF1 type-3 provisional record with the complete SOP2 as stable core and uses the unchanged R20
sequence/physical-ordinal and encode/decode/round-trip checks. It performs no append or I/O.

## 8. Exact mapping precedence and state transition

Mapping is transactional in this order:

1. enforce the 65,536-reference bound and call `validate_semantic_operation_v2`; malformed bytes,
   profiles, cardinality, or duplicate targets return `SemanticValidation` before lookup;
2. if accepted count equals selected stream length return `Exhaustion`; otherwise require the
   offered bytes and extracted namespace, total position, segment/ordinal, producer/ordinal, and
   three identities to equal the exact next catalog operation, else `Discontinuity`;
3. classify targets in encoded order, first failure wins: current EventId `SelfReference`; known
   non-Event `WrongKind`; known correction/retraction EventId `WrongFact`; known foreign-stream
   EventId `CrossStream`; known same-stream other-segment EventId `CrossSegment`; known same-domain
   greater ordinal `Future`; defensive equal ordinal `SelfReference`; lower-ordinal ordinary EventId
   valid; absent identity `Missing`;
4. call the unchanged R20 physical mapper checks with the caller's watermarks; and
5. only after successful RF1 encode/decode/round trip return one record/frame and a next state.

A valid causal bootstrap is exactly segment ordinal zero with zero targets and passes step 3
without lookup. Each segment bootstraps independently. Every non-bootstrap must contain the exact
positive R26 policy cardinality and may reference only prior ordinary EventIds in the same stream
and segment. A target-bearing bootstrap is classified rather than discarded. Success increments
accepted operations and physical ordinal exactly once, adopts the supplied increasing nonzero
sequence, and preserves scope/namespace bindings. Every failure returns no frame and no next state;
the input state and catalog remain byte-for-byte/logically unchanged.

## 9. Exact files and dependency boundary

The one implementation PR may change only:

1. `experiments/exp-0001/crates/exp1-raw-append-replay/src/reference_context.rs` (new);
2. `experiments/exp-0001/crates/exp1-raw-append-replay/src/mapping.rs`;
3. `experiments/exp-0001/crates/exp1-raw-append-replay/src/lib.rs` only to export the module/API;
4. `experiments/exp-0001/crates/exp1-raw-append-replay/tests/reference_context.rs` (new);
5. `experiments/exp-0001/crates/exp1-raw-append-replay/tests/mapping.rs`; and
6. synchronized authority/status/readiness/traceability documentation at closure.

Only the two existing reviewed workspace path dependencies, `exp1-record-format` and
`exp1-workload-conformance`, plus `core`/`std`, are permitted. No new dependency is authorized.
`experiments/exp-0001/Cargo.toml`, the two authority crates, toolchain, workflows, and workspace
membership remain unchanged. The raw-append crate manifest and matching `Cargo.lock` entry may
change only if strictly necessary to expose an already-authorized existing workspace path
dependency; current main already declares both, so the expected change is none. Any such necessity
must be documented and may not alter versions, features, sources, or dependency graph beyond that
existing edge.

## 10. Completion tests and exclusions

The implementation must test, using the merged R26 literals as independent oracle: both valid
zero-target bootstraps; both valid ordinal-one same-stream/same-segment references; exact manifest,
WS2, artifact, manifest-digest and scope-digest binding; every mixed-version position; unchanged v1
scope behavior; omitted/extra/duplicate/substituted/foreign/digest-disagreeing/noncanonical scope;
every construction-error adjacency and precedence; all typed collision classes; every inclusive
bound and one-over/checked-overflow failure; missing, future, self, wrong-kind, wrong-fact,
cross-stream, cross-segment both directions, duplicate-before-lookup, target-bearing-bootstrap,
first-invalid-target ordering, discontinuity, and exhaustion. Tests must prove failure returns no
output/next state and leaves state/catalog unchanged, while each success returns byte-exact RF1,
advances all three watermarks once, and preserves legacy R20 tests. Defensive unreachable paths are
identified rather than reached with forged authority input.

Completion requires the authorized paths only, all tests above, synchronized documentation, review
of the complete exact implementation head, both existing CI workflows green for that head, the
unchanged R9 validation sequence, and `git diff --check`. Only then may documentation close the
remaining R20 gate as bounded correctness evidence.

Excluded are Rust in this R27 documentation PR; changes to R26 fixtures or authority crates;
append/reopen integration; a fourth crate; Linux capture/harness work; workload materialization or
execution; benchmarks/results; D2/D3, `fsync`, durability, recovery, faults, adapters, production
code, server/network/query/distributed work, unsafe code, and performance or architecture claims.
The descriptive D1 harness remains blocked on the independently required live Linux capture freeze.
