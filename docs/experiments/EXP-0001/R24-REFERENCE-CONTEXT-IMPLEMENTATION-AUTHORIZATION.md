# R24 Reference Context Implementation Authorization

**Profile:** `EXP-0001-R24-REFERENCE-CONTEXT-v1`  
**Status:** frozen documentation closure and prospective implementation authorization  
**Evidence classification:** governance/implementation authorization only; not implementation,
execution, benchmark, durability, recovery, or experimental evidence

## 1. Closure and sole authorization

R21's immutable typed catalog and caller-owned accepted prefix, R22's strictly segment-local rule,
and R23's complete manifest-bound closed-stream-scope proof collectively close every governance
prerequisite for contextual reference classification. R24 therefore authorizes **exactly one** later
bounded pure-correctness increment: extend the existing `exp1-raw-append-replay::mapping` boundary to
construct that context and transactionally map one operation against it. Merging R24 prospectively
authorizes only that increment; it is not its implementation or closure.

The implementation remains experiment-local, external-dependency-free, and subject to the crate's
existing `#![forbid(unsafe_code)]`. It must reuse the unchanged `exp1-workload-conformance` and
`exp1-record-format` authority crates. It may not reproduce or weaken their validation or encoding.

## 2. Exact change boundary

The later implementation may add or modify only:

1. `experiments/exp-0001/crates/exp1-raw-append-replay/src/mapping.rs`;
2. `experiments/exp-0001/crates/exp1-raw-append-replay/src/reference_context.rs` (new);
3. `experiments/exp-0001/crates/exp1-raw-append-replay/src/lib.rs`, solely to publicly export the
   new module; and
4. `experiments/exp-0001/crates/exp1-raw-append-replay/tests/mapping.rs` and
   `experiments/exp-0001/crates/exp1-raw-append-replay/tests/reference_context.rs` (new).

No other source or test file is authorized. The workspace manifest, all crate manifests, and
`Cargo.lock` remain byte-for-byte unchanged: the existing mapper already has the two required path
dependencies, so no authority-backed reason for a manifest change exists. Both authority crates,
including their source and tests, remain byte-for-byte unchanged. No fourth crate or dependency is
authorized. Documentation synchronization at implementation closure is the only permitted change
outside the six implementation paths above.

## 3. Frozen public boundary

The new `reference_context` module owns these public, nonconstructible-success types:

- `ClosedScopeInput<'a>` borrows the exact R23 descriptor bytes, scope-digest descriptor and R7
  metadata/provenance bytes, the resolved R16 manifest/digest/R7 metadata for every member, and
  exactly one complete WS1 byte string per supplied member. Its fields may be public borrowed input
  records; it is not proof until validated.
- `ReferenceCatalog` is an immutable, opaque successful catalog. It exposes read-only counts and
  scope identity only; no public constructor, mutable entry access, or identity insertion exists.
- `AcceptedPrefixState` is opaque, caller-owned, and unforgeable. Only successful context
  construction creates its initial value and only successful contextual mapping creates its next
  value. It binds the scope, selected stream namespace, exact next WS1 position, segment and segment
  ordinal, and the R20 sequence/physical-ordinal state. Private fields and no public arbitrary
  constructor are mandatory.
- `ReferenceContext` groups the immutable catalog with its initial accepted-prefix state. Successful
  construction returns both; failure returns neither.
- `ContextConstructionError` is exhaustive and includes `InvalidScopeEncoding`,
  `UnsupportedScopeProfile`, `InvalidScopeDigest`, `ScopeReferenceFailure`,
  `InvalidCellAuthority`, `InvalidMemberBinding`, `OmittedStream`, `ExtraStream`,
  `DuplicateStreamNamespace`, `SubstitutedStream`, `ForeignWorkloadOrCell`,
  `SemanticValidation`, `IdentityCollision`, `ResourceLimit`, `Extraction`, and
  `SelectedStreamMissing`. Nested unchanged authority errors may be retained where applicable.
- `ReferenceError` is exhaustive and includes `Missing`, `Future`, `WrongKind`, `WrongFact`,
  `SelfReference`, `CrossStream`, and `CrossSegment`. The transactional entry point additionally
  reports accepted-position `Discontinuity` and accepted-prefix `Exhaustion` distinctly (either as
  variants of `ReferenceError` or a documented enclosing contextual-mapping error).

The constructor is one pure function equivalent to:

```rust,ignore
pub fn construct_reference_context(
    input: ClosedScopeInput<'_>,
    selected_stream_namespace: [u8; 16],
) -> Result<ReferenceContext, ContextConstructionError>;
```

The transactional mapper is one pure function equivalent to:

```rust,ignore
pub fn map_semantic_operation_with_context(
    semantic_operation: &[u8],
    assigned_sequence: u64,
    physical_ordinal: u64,
    catalog: &ReferenceCatalog,
    state: &AcceptedPrefixState,
) -> Result<ContextualMappedRecord, ContextualMappingError>;
```

`ContextualMappedRecord` contains the same complete frame and decoded record as `MappedRecord`, plus
an opaque owned `next_state()`. The input state is borrowed, never mutated. The exact Rust field
grouping/lifetimes and nesting of authority errors may vary only to satisfy ownership without
changing observable inputs, error distinctions, opacity, or semantics.

R20's existing `MappingState`, `MappedRecord`, `MappingError`, and
`map_semantic_operation(bytes, sequence, ordinal, state)` remain source-compatible and unchanged in
behavior. The authorized signature evolution is additive: callers needing contextual validation use
the new constructor and contextual mapper. The old mapper is retained as the locally decidable R20
primitive and must not be presented as full reference validation.

## 4. Deterministic construction and bounds

Construction follows R23 section 4's eight stages exactly, returning the first error by stage and,
within a stage, canonical namespace order. It validates descriptor encoding/JCS/schema and member
order; scope digest and R7 provenance; R16 resolution, supersession, artifact, and cell authority;
member cross-bindings; complete R14/R16 WS1 bytes; exact supplied-set equality; then R21 bounds,
typed collisions, extraction, and selected-stream uniqueness. Only after all checks may it publish
the catalog and state. No catalog, accepted state, or partial result exists on any failure.

The inclusive R21 limits are unchanged and jointly enforced with checked arithmetic before
allocation: 256 streams; 16,777,216 total WS1 bytes; 65,536 operations across all streams; at most
`3 * operations`, hence 196,608, typed identity bindings; and 65,536 references in one operation.
R23 additionally bounds descriptor bytes at 262,144, members at 256, total accepted R16 manifest
bytes at 1,048,576, and resolved R7 metadata bytes at 4,194,304. R23 never relaxes an R21 limit.
Every count, sum, conversion, and allocation is checked; overflow and excess are `ResourceLimit`.

Typed collisions follow R21 globally and deterministically. Catalog entries retain identity role,
fact class, stream namespace, total stream position, segment, and segment ordinal. Construction
retains compact bindings, not unbounded source payloads or an authority graph.

## 5. Reference and transactional precedence

Semantic validation, including duplicate-target rejection and the 65,536-reference bound, precedes
context lookup. The operation must match the selected stream's exact next WS1 bytes and position;
otherwise `Discontinuity` wins before target classification. Exhausted accepted position is
`Exhaustion`. Targets are processed in encoded order, and the first invalid target wins:

1. current EventId: `SelfReference`;
2. a known non-Event identity: `WrongKind`;
3. a known correction/retraction EventId: `WrongFact`;
4. a known EventId in another stream: `CrossStream`;
5. a known same-stream EventId in the other segment: `CrossSegment`;
6. a known same-stream/same-segment greater ordinal: `Future`;
7. a defensive equal-position match: `SelfReference`;
8. a lower-ordinal ordinary EventId in the same stream and segment: valid; and
9. an identity absent from the proven-complete catalog: `Missing`.

Only after every reference and every unchanged R20 mapping/round-trip check succeeds may the result
be returned. Failure returns no frame and no next state; the borrowed caller state remains
bit-for-bit unchanged. Success returns exactly one frame and exactly one next state advancing the
accepted count/position and both R20 watermarks exactly once. The immutable catalog never changes.

## 6. Required deterministic test gate

Tests use synthetic implementation fixtures derived from the authorities. They are correctness
checks, not generated workload execution, benchmark observations, experimental evidence, or a
substitute for reviewed R7 artifacts. The later increment must deterministically cover:

- valid same-segment prior reference; missing; future; self; wrong kind; wrong fact; duplicate
  target semantic-validation precedence; and cross-stream;
- cross-segment in both warm-up-to-measured and measured-to-warm-up directions;
- R23 valid construction plus omitted, extra, duplicate, substituted, foreign-cell/workload, and
  digest-mismatched streams;
- every typed identity collision class;
- exact inclusive success and one-over/checked-overflow failure for every R21 and R23 resource
  limit, including 65,536 references in one operation;
- discontinuity and exhaustion;
- multi-target ordering where earlier valid targets do not hide the first invalid target, plus
  precedence cases that distinguish every adjacent error class; and
- transactional failure with no output and byte-for-byte identical input state, and success with
  one frame, one next state, exact single advancement, unchanged catalog, and unchanged legacy R20
  vector behavior.

Unreachable defensive branches must be identified as such rather than covered with a weakened or
forged authority input. Fixtures may be compactly generated in test code but may not modify an
authority crate or materialize/run an experimental workload.

## 7. Exact completion gate and exclusions

The authorization closes only when the bounded implementation exists at the exact paths in section
2; all section 6 tests pass; continuity, status, readiness, and traceability documentation is
synchronized; the exact implementation head and complete diff receive review; and both existing CI
workflows are green for that exact head. The unchanged R9 validation sequence plus
`git diff --check` must pass. Until then the full R20 correctness gate remains open.

Explicitly excluded are append/reopen integration; workload materialization or execution; Linux
capture or harness implementation; any fourth crate, dependency, manifest/lockfile change, unsafe
code, or authority-crate change; D2/D3, `fsync`, benchmark, durability, recovery, fault, adapter,
production, server, network, query, or distributed work. No performance, durability, recovery,
capture, workload, or experimental claim follows from R24 or its authorized correctness increment.
