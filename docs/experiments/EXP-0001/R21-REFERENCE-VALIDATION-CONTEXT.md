# R21 Reference-Validation Context

**Profile:** `EXP-0001-R21-REFERENCE-CONTEXT-v1`  
**Status:** frozen documentation/governance decision  
**Evidence classification:** documentation design; not implementation, execution, or experimental evidence

## 1. Decision and disposition

R21 freezes the minimum pure context needed to classify every R12 section 5.3 reference outcome. It
separates an immutable **validated reference catalog**, which says which typed identities exist in
which complete workload streams, from mutable **accepted-prefix state**, which says how much of one
selected stream has successfully passed mapping. Neither object validates semantic meaning or
regenerates workload values.

This decision fully resolves the reference-context governance blocker and prospectively authorizes
one later bounded extension of `exp1_raw_append_replay::mapping`. It does **not** close the complete
R20 mapper gate. Closure requires the later implementation at an exactly reviewed head, all R20 and
R21 deterministic tests, and CI. The independent live-Linux-capture blocker remains open; no
descriptive D1 harness or execution is authorized.

## 2. Catalog authority and construction

### 2.1 Sole construction input

`ReferenceCatalog::from_validated_streams` takes a nonempty ordered slice of complete WS1
`EXP-0001-SEMANTIC-OP-v1` byte strings and one selected stream namespace. It calls
`exp1_workload_conformance::validate_stream` on every complete byte string before extracting any
field. Complete individual SOP1 operations, manifests, digests, or generator inputs are not catalog
authorities: they cannot independently prove complete ordering and warm-up/measured boundaries.
The manifest remains provenance for later execution, not an identity-membership index.

After validation, the constructor parses only the already-validated WS1 framing and these OP1,
SOP1, and ENV1 fields: stream namespace; segment; segment ordinal; producer identity and
producer-local ordinal; RequestId, EventId, and InformationId; fact type; and reference semantics.
It must not copy semantic validation, calculate identities, select references, generate payloads,
or infer a fact type. Post-validation extraction disagreement fails closed as
`ContextBuildError::ValidatedFieldExtraction`.

Each source stream must contain exactly one stream namespace, and namespaces must be unique across
the inputs. The selected namespace must occur exactly once. A stream's catalog position is its WS1
zero-based operation index. Warm-up positions precede measured positions exactly as the WS1 header
declares; segment ordinals restart at zero at that boundary. No reference rule treats measured zero
as earlier than warm-up. The total position, not a segment-local ordinal, decides prior/future.
`validate_stream` owns contiguous segment/ordinal validation; any discontinuity is
`ContextBuildError::SemanticValidation` before a catalog exists.

Producer identity and producer-local ordinal are retained as exact bindings for the current
operation. They neither establish global order nor replace WS1 position. R21 adds no producer
sequence rule: single-producer A01 equality is an anchor property, while controlled multi-producer
ordering remains whatever the validated semantic stream says.

### 2.2 Entries, identity domains, and collisions

For every operation the catalog has three typed identity bindings:

```text
IdentityBinding { bytes[16], role: Request | Event | Information,
                  stream_namespace[16], stream_position:u64,
                  segment, segment_ordinal:u64, producer[16], producer_ordinal:u64,
                  fact_class (Event only) }
```

`fact_class` is exactly `Ordinary`, `Correction`, or `Retraction`. Correction and retraction are
distinguished by the validated ENV1 reference-semantics value and its validated matching fact
type/profile; R21 does not maintain a second list of allowed fact-type strings. Request and
Information bindings carry no fact class.

Lookup is by the raw 16 identity octets in an ordered map; iteration order is irrelevant. Identity
bytes must be globally unique across every role and supplied stream. Exact reuse is not collapsed,
and any second binding—whether equal or unequal in role/content—is
`ContextBuildError::IdentityCollision`. This stricter catalog invariant makes lookup single-valued
and preserves R12's fail-without-remap rule. There is no retry, salt, replacement, or last-writer
wins behavior.

Including complete validated foreign streams is the only recognition mechanism for
`E-REFERENCE-CROSS-STREAM`. A target from a stream not supplied to the catalog is absent and is
therefore `E-REFERENCE-MISSING`; the mapper must not guess provenance from UUID bits. Callers that
need cross-stream classification must supply that complete validated stream at construction.

## 3. Bounds and resource accounting

The v1 constructor enforces, in this order, checked arithmetic and these inclusive limits:

| Resource | Bound |
|---|---:|
| input streams | 256 |
| sum of complete WS1 input bytes | 16,777,216 bytes (16 MiB) |
| operations across all streams | 65,536 |
| identity bindings | `3 * operations`, at most 196,608 |
| references in one current operation | existing encoded `u32` count, additionally at most 65,536 |

Zero streams, a missing/duplicate selected namespace, an exceeded limit, failed conversion, or
addition/multiplication overflow returns respectively `EmptyCatalog`, `SelectedStreamMissing`,
`DuplicateStreamNamespace`, or `ResourceLimit`. Bounds are checked before allocation where the
count is available; failure returns no partial catalog.

The implementation must account and test `source_bytes`, `stream_count`, `operation_count`,
`identity_entry_count`, and the accepted-state footprint. It may retain compact owned fixed-width
entries and indexes, but not source WS1/SOP1/payload bytes after construction. Accounted logical
entry storage is the sum of all fixed fields above plus ordered-map/index allocation measured with
checked `usize`; implementation-specific allocator overhead is reported separately in tests and is
not serialized. Temporary validation/parser storage must be bounded by the 16 MiB input and one
operation. No unbounded cache is authorized.

## 4. Accepted-prefix state and lifecycle

`ReferenceContext` immutably borrows or owns an `Arc`-free `ReferenceCatalog` according to ordinary
Rust lifetimes and contains caller-owned `AcceptedPrefixState`:

```text
AcceptedPrefixState {
  selected_stream_namespace[16],
  accepted_operations:u64,
  mapping_state: MappingState
}
```

The catalog is immutable after successful construction and may be shared by immutable reference;
no global, lazy, background, or interior-mutable registry is allowed. State starts with the selected
namespace, accepted count zero, and `MappingState::initial()`. It cannot be publicly fabricated.
The operation offered next must be byte-for-byte the catalog operation identity binding at
`accepted_operations` and must have the catalog's stream, position, segment/ordinal, producer, and
three identities. A mismatch or skipped/repeated operation is
`ReferenceError::StreamOrOrdinalDiscontinuity`; end-of-stream is
`ReferenceError::StreamExhausted`.

On full mapping success only, the returned next state increments `accepted_operations` exactly once
with checked arithmetic and adopts the R20 returned `MappingState`. Catalog bytes never change. Any
semantic, reference, sequence/ordinal, resource, encode/decode, or round-trip failure returns no
frame and no next state; catalog and caller state remain bit-for-bit unchanged. Targets are never
filtered, replaced, reordered, deduplicated, or partially accepted.

State reconstruction from appended records, reopen/replay, a checkpoint, or a manifest is outside
this pure tranche. A caller may start only at the initial empty-prefix state. Prospective resume and
reconstruction require a later authority and do not follow from the catalog containing future
operations.

## 5. Deterministic validation and precedence

The mapper validates one operation transactionally in this exact order:

1. Call `validate_semantic_operation`; malformed SOP1, malformed reference encoding/cardinality,
   or duplicate reference bytes returns existing `MappingError::SemanticValidation` before any
   context lookup. Duplicate bytes therefore have effective precedence over every target property.
2. Match the current operation to the selected catalog position and accepted prefix; otherwise
   return `Reference(StreamOrOrdinalDiscontinuity | StreamExhausted)`.
3. For each target in encoded order, stop at the first failure and apply this target precedence:
   current EventId -> `SelfReference`; known non-Event role -> `WrongKind`; known EventId with
   correction/retraction fact class -> `WrongFact`; known EventId in another cataloged stream ->
   `CrossStream`; known same-stream EventId at a greater position -> `Future`; no binding ->
   `Missing`; known same-stream EventId at the current position -> `SelfReference` defensively;
   known same-stream prior ordinary EventId -> valid.
4. Apply the existing R20 sequence/physical-ordinal and RF1 construction checks.

Thus the public reference dispositions correspond exactly to R12:

| Variant | Disposition |
|---|---|
| `ReferenceError::Missing` | `E-REFERENCE-MISSING` |
| `ReferenceError::Future` | `E-REFERENCE-FUTURE` |
| `ReferenceError::WrongKind` | `E-REFERENCE-WRONG-KIND` |
| `ReferenceError::WrongFact` | `E-REFERENCE-WRONG-FACT` |
| duplicate rejected by `SemanticValidation` | `E-REFERENCE-DUPLICATE` |
| `ReferenceError::SelfReference` | `E-REFERENCE-SELF` |
| `ReferenceError::CrossStream` | `E-REFERENCE-CROSS-STREAM` |

Identity collision and stream discontinuity are catalog/context failures, not invented R12
reference dispositions. A known foreign correction EventId is `WrongFact` before `CrossStream`; a
foreign non-Event identity is `WrongKind`; self wins before role/fact/locality; and an uncataloged
identity is always `Missing`. Multiple ordered targets are all-or-nothing: the first invalid member
in encoded order determines the one error and no later member is examined observably.

## 6. Prospective public surface and ownership

Only `exp1-raw-append-replay::mapping` may later change. It may add public
`ReferenceCatalog`, `ReferenceContext`/`AcceptedPrefixState`, `ContextBuildError`, and
`ReferenceError`; add `MappingError::Reference(ReferenceError)` and
`MappingError::Context(ContextBuildError)` if construction shares that error surface; and replace or
overload `map_semantic_operation` with a pure function accepting `&ReferenceCatalog` plus the
unforgeable accepted-prefix state. `MappedRecord::next_state` may return the combined next state.
Names may vary only mechanically; fields, invariants, variants, precedence, and ownership may not.

The implementation must call the existing conformance validator and retain the existing direct
path dependencies on `exp1-record-format` and `exp1-workload-conformance`. It may add internal
validated-field extraction and deterministic tests inside `exp1-raw-append-replay` only. Neither
authority crate, the workspace manifest, any Cargo manifest or lockfile, nor another crate may
change. No new wire format or serialization is selected: catalog and state are ephemeral in-memory
values. Debug output is not a persistence contract.

## 7. Documentation vectors

These are independently reviewable design vectors, not execution or benchmark evidence. A01 and
R01–R04 refer to the literal R12 authorities. `SYN-*` identities are deliberately synthetic
non-evidence and are formed by changing the stated final UUID octet while retaining UUIDv4 shape.

| Vector | Catalog/current prefix and target | Expected result |
|---|---|---|
| V21-01 | A01 position 2; accepted positions 0–1; target A01 EventId(1) | valid prior ordinary; one frame and accepted count 3 |
| V21-02 | R01 ordinary positions 0–3; current 4 targets EventId(1), (2), (3) | valid in that exact order; one atomic success |
| V21-03 | target `SYN-MISSING=330f201a-ea7c-4335-a8ec-e6fe23266aff`, absent from all inputs | `Missing` |
| V21-04 | accepted position 0, current position 1 targets same-stream EventId(2) | `Future` |
| V21-05 | current A01 EventId(2) targets itself | `SelfReference` |
| V21-06 | current position 2 targets A01 RequestId(1) | `WrongKind` |
| V21-07 | current position 2 targets A01 InformationId(1) | `WrongKind` |
| V21-08 | supplemental validated correction/retraction operation is prior; its EventId is targeted | `WrongFact` for either fact class |
| V21-09 | R04 repeats the same target bytes | conformance `SemanticValidation` / `E-REFERENCE-DUPLICATE`, before lookup |
| V21-10 | second complete validated stream contains the target EventId; selected stream does not | `CrossStream`; omitting that stream instead produces `Missing` |
| V21-11 | two catalog bindings reuse bytes with unequal role or content | `ContextBuildError::IdentityCollision`; no catalog (synthetic non-evidence) |
| V21-12 | WS1 skips ordinal, repeats ordinal, starts measured at 1, or returns to warm-up | conformance semantic failure; no catalog |
| V21-13 | valid state before V21-04/V21-05/V21-06/V21-08/V21-10 | returned error has no frame/next state; original accepted count and R20 watermarks unchanged |
| V21-14 | V21-01 with valid sequence/physical ordinal | accepted count and both R20 watermarks each advance once; catalog counters and entries unchanged |

For a multi-target operation `[valid EventId(1), invalid RequestId(1), invalid missing]`, the second
member produces `WrongKind`; no target is removed, the missing third member cannot replace that
result, no frame is returned, and state is unchanged. These supplemental arrangements do not alter
the canonical A01 bytes and are not observations.

## 8. Explicit exclusions

R21 adds and authorizes no Rust implementation in this change; Cargo/lock changes; authority-crate
changes; external dependency; unsafe code; fourth crate; append/reopen integration; workload
materialization or execution; benchmark execution or evidence; Linux capture design or
implementation; D2/D3 or `fsync`; canonical commit/recovery; fault work; adapters; production code;
server, network, query, or distributed work; or durability/performance claim. The later authorized
work remains a pure correctness component and nothing more.
