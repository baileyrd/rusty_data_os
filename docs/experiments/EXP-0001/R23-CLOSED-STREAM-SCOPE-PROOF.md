# R23 Closed Stream Scope Proof

**Profile:** `EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v1`
**Digest profile:** `EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v1`
**Status:** frozen documentation/governance decision
**Evidence classification:** synthetic documentation design; not implementation, execution, or
experimental evidence

## 1. Decision

R23 selects one **closed-scope descriptor** as the minimum deterministic proof that a supplied set
contains every WS1 stream in one explicitly named workload/cell classification scope. Existing
R14 stream bytes and digests prove completeness of each individual stream. R16 manifests,
manifest digests, generator inputs, artifact bindings, workload IDs, and validated namespaces bind
one manifest to one stream. None enumerates the complete multi-stream set or says that no other
stream belongs to the cell. They are therefore necessary member evidence but are not a closure
proof.

The smallest additional authority is an immutable, closed, canonical descriptor which names the
cell, enumerates every member, and commits to that exact membership with a domain-separated digest.
R23 freezes that descriptor. A caller's assertion, vector length, iterator exhaustion, set of
successfully opened files, or absence from a partial catalog is never proof of completeness.

This decision **fully closes the remaining reference-context governance blocker**. It does not
authorize implementation. After R23 is reviewed and merged, the smallest prospective later
increment is a pure extension of `exp1-raw-append-replay::mapping` that validates this descriptor,
constructs the already-frozen R21 catalog from its bound members, and implements the R21/R22
reference dispositions. That increment requires separate authorization, exact-head review, and CI.

## 2. Exact descriptor and identity

The descriptor is one UTF-8 I-JSON object serialized as RFC 8785 JCS under R7
`EXP1-R7-JSON-JCS-1`. Stored bytes are exactly the JCS bytes, without BOM, whitespace outside the
object, or newline. Objects are closed at every depth; all fields are required and non-null.
Duplicate names, unknown or missing fields, duplicate array entries, non-I-JSON, or noncanonical
bytes fail closed. No JSON numbers occur.

The top-level object is exactly:

```text
{
  "schema_version": "EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v1",
  "record_kind": "closed_stream_scope",
  "scope_id": UUID,
  "cell_id": identifier,
  "members": [member, ...]
}
```

`scope_id` is a non-nil, lowercase RFC 9562 UUID with valid variant bits. It is an assigned
immutable identity, not derived from content and never reused for changed membership. `cell_id` is
the exact nonempty ASCII identifier assigned by the applicable reviewed EXP-0001 cell authority,
at most 128 octets. For the current R8 primary registry it is one exact `PC-<mode>-<baseline>-<profile>`
ID. A synthetic or pre-R8 correctness cell requires a separately reviewed immutable cell ID; a
caller cannot invent one while constructing context. The descriptor proves completeness only for
the named `(scope_id, cell_id)`, never globally or for a prefix/alias of either value.

Each `member` is the closed object:

```text
{
  "stream_namespace": UUID,
  "workload_id": UUID,
  "manifest_id": UUID,
  "manifest_digest": lowercase-sha256,
  "stream_digest": lowercase-sha256,
  "stream_byte_length": u64-decimal-string,
  "stream_artifact_sha256": lowercase-sha256
}
```

UUID and SHA-256 syntax follows R16. `stream_byte_length` matches `0|[1-9][0-9]*` and fits `u64`.
`members` contains 1 through 256 entries, sorted by the raw 16 namespace octets in ascending
unsigned lexicographic order. Namespace, workload ID, manifest ID, and manifest digest are each
unique across members. Reordering is noncanonical; any duplicate is a construction failure. The
workload IDs may differ because R16 requires changed stream bytes/semantics to receive a new
workload ID; the R23 cell identity, not equality of workload IDs, declares their common closed
classification scope.

The external scope digest is exactly:

```text
SHA-256(ASCII "rusty-data-os/exp1/closed-stream-scope/v1" || 00 || descriptor_bytes)
```

It is published as the closed object
`{algorithm,domain,profile,scope_ref,value}`. Values are respectively
`SHA-256/FIPS-180-4`, `rusty-data-os/exp1/closed-stream-scope/v1`,
`EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v1`, an R7 immutable `ref` to the exact descriptor
artifact, and the 64-character lowercase digest. `scope_ref` and its R7 artifact entry must agree
on artifact identity, byte length, exact-artifact digest, URI, role `configuration`, media type
`application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs`, and creating record. The digest is
external and no self-digest placeholder or omitted-field transform exists.

## 3. Member proof and supplied-input equality

Every member resolves through its `manifest_digest` to exactly one accepted R16 external
manifest-digest descriptor and immutable manifest bytes. Its seven fields must equal the resolved
R16 bindings:

1. namespace equals `generator_inputs.stream_namespace` and the sole namespace decoded from WS1;
2. workload and manifest IDs equal the R16 fields;
3. manifest digest equals the R16 domain-separated digest of the exact canonical manifest bytes;
4. stream digest equals the R14 domain-separated digest in the manifest;
5. byte length and artifact SHA-256 equal `stream_ref` and the resolved R7 artifact entry; and
6. the referenced bytes pass complete R14/R16 WS1 validation and reproduce both stream digests.

The constructor receives the descriptor bytes, its digest descriptor and provenance, all resolved
R16 manifest/digest/provenance records, and exactly one complete WS1 byte string per member. After
validation it compares the canonical set of supplied namespaces and bindings for exact equality
with `members`. Every descriptor member must be supplied once and every supplied stream must have
one descriptor member. Thus every supplied stream and namespace is bound to the proof, while an
omitted, extra, substituted, foreign-cell/workload, duplicated, or digest-mismatched stream fails
before any R21 catalog exists.

An R16 workload ID does not prove cell membership by itself. Membership is authorized only by its
exact entry in a reviewed R23 descriptor for the named cell. A stream from another workload or cell
cannot be admitted merely because its bytes validate, its UUID has the right shape, or its profile
fields resemble the selected cell. Corrections follow R16 immutable supersession: a descriptor
names one exact accepted manifest revision. Changing to a corrected manifest, stream, membership,
cell, or binding requires a new `scope_id`, descriptor bytes, artifact, and scope digest; published
descriptors are never mutated.

## 4. Validation order, failures, and bounds

Context construction is transactional and validates in this exact order:

1. descriptor bytes, UTF-8, JSON, duplicate names, I-JSON, exact JCS, closed schema, scalar syntax,
   supported profiles, member count/order, and uniqueness;
2. scope digest descriptor, immutable R7 reference/provenance, and digest over unmodified bytes;
3. resolve every exact R16 manifest digest/manifest/artifact binding in member order, including
   supersession validity and the declared cell authority;
4. enforce every member cross-binding in section 3;
5. validate every complete WS1 and its R14/R16 counts, namespace, lengths, and digests;
6. canonicalize supplied inputs by namespace and require exact one-for-one equality with members;
7. enforce R21 aggregate bounds, typed global identity collision rules, selected-namespace
   uniqueness, and validated-field extraction; and only then
8. publish the immutable R21 catalog and initial accepted-prefix state.

Failures are context-construction failures, not reference dispositions: `InvalidScopeEncoding`,
`UnsupportedScopeProfile`, `InvalidScopeDigest`, `ScopeReferenceFailure`, `InvalidCellAuthority`,
`InvalidMemberBinding`, `OmittedStream`, `ExtraStream`, `DuplicateStreamNamespace`,
`SubstitutedStream`, `ForeignWorkloadOrCell`, `SemanticValidation`, `IdentityCollision`, or the
existing R21 bound/extraction/selection failures. The first failure in the ordered stages is
returned deterministically; within a stage, members are examined in canonical namespace order.
No catalog, frame, accepted state, or partial result is returned.

R21's inclusive limits remain authoritative: at most 256 streams, 16,777,216 bytes (16 MiB) of
total WS1 input, 65,536 operations across all streams, and `3 * operations` identity bindings, at
most 196,608. R23 additionally
limits descriptor bytes to 262,144, members to 256, the sum of accepted R16 manifest bytes to
1,048,576, and all resolved R7 metadata bytes to 4,194,304. All counts, additions, conversions,
digest buffers, and allocations are checked before use; overflow or a limit breach is
`ResourceLimit`. Implementations may stream digests and retain compact fixed-width bindings, but
may not fetch an unbounded graph, cache source payloads after construction, or weaken R21's
accounting.

## 5. Reference classification after proof

R21 semantic validation, current-position matching, typed collision rules, immutable catalog,
caller-owned accepted prefix, ordered-target processing, and transactional failure remain in
force. R22's strictly segment-local eligibility and `E-REFERENCE-CROSS-SEGMENT` precedence remain
in force. After successful construction from one proven-complete scope, each target follows the
R21/R22 order:

1. self;
2. known non-Event role (`E-REFERENCE-WRONG-KIND`);
3. known correction/retraction EventId (`E-REFERENCE-WRONG-FACT`);
4. **known EventId in another bound stream (`E-REFERENCE-CROSS-STREAM`);**
5. known same-stream EventId in another segment (`E-REFERENCE-CROSS-SEGMENT`);
6. known same-stream/same-segment future or defensive same-position self;
7. known eligible prior ordinary EventId; then
8. **identity absent from the complete catalog (`E-REFERENCE-MISSING`).**

An incomplete or unproven scope fails context construction and can never produce `Missing`.
Duplicate encoded targets still fail semantic validation before lookup. The first invalid target
wins, and every failure returns no frame or next state and leaves the catalog, accepted count,
sequence watermark, and physical-ordinal watermark unchanged.

## 6. Synthetic documentation vectors

These vectors are design cases, not fixtures, tests, execution, or evidence. `A` and `B` are
complete R16-valid WS1 streams with distinct namespaces and exact member bindings; `C` is a valid
stream not listed in scope `S`.

| Vector | Descriptor and supplied inputs | Expected result |
|---|---|---|
| SYN23-01 | `S.members=[A,B]` in namespace order; exactly A and B with all bindings | valid complete scope; immutable catalog is published |
| SYN23-02 | `S.members=[A,B]`; only A supplied | `OmittedStream`; no catalog and never `Missing` |
| SYN23-03 | `S.members=[A,B]`; A, B, and C supplied | `ExtraStream`; no catalog |
| SYN23-04 | two supplied streams or members use A's namespace | `DuplicateStreamNamespace`; no catalog |
| SYN23-05 | B's position supplies different valid bytes/manifest under the claimed binding | `SubstitutedStream` or `InvalidMemberBinding`; no catalog |
| SYN23-06 | a member resolves to a workload/cell not authorized by `S.cell_id` | `ForeignWorkloadOrCell`; no catalog |
| SYN23-07 | A stream, manifest, artifact, manifest digest, or scope digest differs by one octet | corresponding digest/binding failure; no catalog |
| SYN23-08 | current A operation targets a known ordinary EventId in bound B | `E-REFERENCE-CROSS-STREAM`; state unchanged |
| SYN23-09 | current A operation targets UUIDv4-shaped EventId bytes absent from A and B | `E-REFERENCE-MISSING` only after SYN23-01 construction; state unchanged |

Iterator exhaustion, the two-element vector length, or failure to locate SYN23-09 in a catalog
built without validating `S` would not change that case into `Missing`.

## 7. Compatibility, closure, and exclusions

R23 supersedes only R21/R22 statements that complete closed-scope proof is unresolved. It does not
alter R12 generation or existing error meanings, R14 WS1 bytes/digests, R16 manifests, R20 mapping,
R21 ownership/bounds/collisions/accepted-prefix behavior, or R22 segment locality and precedence.
No existing R12/R14/R16 vector bytes change.

R23 closes the governance question, not the complete R20 correctness gate. That gate still needs a
separately authorized implementation, deterministic tests, exact-head review, and CI. The live
Linux capture freeze independently remains open and continues to block a descriptive D1 harness.

This change adds or authorizes no Rust source, Cargo/lockfile or authority-crate change,
reference-context implementation, append/reopen integration, workload materialization or
execution, Linux capture implementation, fourth crate, dependency, unsafe code, benchmark,
durability, recovery, fault, adapter, production, server, network, query, or distributed work;
no D2/D3, `fsync`, correctness-implementation, performance, or durability evidence follows.
