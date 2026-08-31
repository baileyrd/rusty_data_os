# EXP-0001 R16 — Workload-manifest serialization contract

**Authority base:** live `main` at `f38b637b27cf6748c875077a86e5e0f318ba0483`
**Status:** complete as documentation design; BLK-009 is resolved only at that boundary
**Manifest profile:** `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`
**Manifest-digest profile:** `EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1`
**Decision date:** 2026-08-31

## 1. Scope and selected physical profile

This authority completes only the R15-authorized BLK-009 documentation freeze. A workload
manifest identifies immutable deterministic input; it is not an environment, configuration,
validation, lifecycle, fault, or result record and does not show that a run occurred.

One manifest is one UTF-8 I-JSON object serialized using RFC 8785 JCS. The applicable
canonicalization profile is R7 `EXP1-R7-JSON-JCS-1`, restricted here by the closed ledger below;
the manifest's `schema_version` remains the distinct
`EXP-0001-WORKLOAD-MANIFEST-JCS-v1`. Its media type is
`application/vnd.rusty-data-os.exp1-workload-manifest+jcs`. Stored bytes are exactly the JCS
bytes: no BOM, leading/trailing whitespace, or newline. JSON Lines/sequences, comments,
duplicate names, lone surrogates, NaN, and infinities fail. JCS lexicographic UTF-16 member
ordering, escaping, and number rules apply. This profile uses no JSON numbers: every count,
length, seed, ordinal, and signed logical-time parameter is a canonical decimal string.

Objects are closed at every depth. Unknown fields fail `unknown-field`; missing required fields
fail `missing-field`; duplicate JSON names fail `duplicate-member`. No field is nullable or
optional. An inapplicable value is represented only by the explicitly tagged state objects
below, never by JSON `null`, omission, empty text, or an invented default. Arrays preserve their
declared semantic order and cannot contain duplicate entries.

## 2. Closed physical field ledger

### 2.1 Scalar rules

- UUIDs are lowercase RFC 9562 `8-4-4-4-12` text, non-nil, with valid hexadecimal and variant;
  manifest IDs are assigned identities and MUST NOT be inferred from content.
- `u64` is text matching `0|[1-9][0-9]*` and in `0..18446744073709551615`; `i64` additionally
  permits `-[1-9][0-9]*` and is in `-9223372036854775808..9223372036854775807`.
- SHA-256 values are exactly 64 lowercase hexadecimal characters. Profile identifiers are the
  exact case-sensitive ASCII strings enumerated below, at most 128 octets. Other strings are
  nonempty valid Unicode scalar sequences; identifiers and URIs additionally obey their rules.
- An immutable stream reference is the closed object
  `{artifact_id,artifact_manifest_ref,byte_length,created_by_record_id,media_type,role,sha256,uri}`.
  `artifact_id` and `created_by_record_id` are UUIDs; `byte_length` is `u64`; `role` is exactly
  R7 `configuration`; `media_type` is exactly
  `application/vnd.rusty-data-os.exp1-workload-stream`; `sha256` is the R7 exact-artifact digest
  `SHA-256(ASCII "rusty-data-os/exp1/r7/artifact/v1" || 00 || bytes)`; and `uri` is an absolute
  normalized `file:` or `https:` URI without query or fragment. `artifact_manifest_ref` is an R7
  closed `ref` object `{artifact_id,byte_length,sha256,uri}` resolving to the immutable R7
  artifact-manifest record that contains the stream artifact entry. URI alone is never identity.
- A state object is exactly `{"state":"not_applicable"}` or
  `{"state":"present","value":...}`. No other member or state is legal. R16 deliberately
  has no `missing`, `unknown`, or `null` state: all reproduction inputs must be known.

### 2.2 Top-level ledger

Every field below is required and occurs once.

| Member | Exact type and binding |
|---|---|
| `schema_version` | String, exactly `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`. |
| `record_kind` | String, exactly `workload_manifest`. |
| `manifest_id` | Manifest identity UUID. |
| `workload_id` | Stable semantic workload identity UUID. A correction retains it only when it corrects metadata for the same intended operation stream; changed stream bytes/semantics require a new workload ID. |
| `created_at_utc_ns` | `i64` text; publication metadata only, never effective/system/durability time. |
| `supersession` | Closed object in section 2.5. |
| `profiles` | Closed object in section 2.3. |
| `generator_inputs` | Closed object in section 2.4. |
| `counts` | Closed object in section 2.4. |
| `stream_digest` | Closed object `{algorithm,domain,value}`: exact values `SHA-256/FIPS-180-4`, `rusty-data-os/exp1/workload-stream/v1`, and SHA-256 text. |
| `stream_ref` | Immutable stream-reference object. The referenced bytes are exactly the R14 stream bytes, not a physical event or adapter encoding. Its R7 artifact entry MUST exactly match all duplicated identity, length, digest, URI, role, media-type, and creating-record values, and the resolved R7 provenance graph MUST bind that entry to its creating record and the workload-manifest artifact. Missing, conflicting, cyclic, or unreachable provenance fails `reference`. |
| `authority_revisions` | Array of closed `{authority,revision}` objects, sorted by `authority`; exactly one each for `EXP-0000-WORKLOADS`, `EXP-0001-R2`, `EXP-0001-R7`, `EXP-0001-R12`, `EXP-0001-R14`, and `EXP-0001-R16`. `revision` is exactly one tagged closed object: `{"kind":"git_sha","value":git-sha}` or `{"kind":"reviewed_authority_id","value":identifier}`. A `git-sha` is exactly 40 lowercase hexadecimal characters; a reviewed identifier is nonempty, is not 40 lowercase hexadecimal characters, and names an immutable reviewed authority revision. |

### 2.3 Profile ledger and compatibility

`profiles` has exactly these members:

| Member | Supported value/domain |
|---|---|
| `workload_contract` | `EXP-0000-WORKLOADS-v1` |
| `manifest` | `EXP-0001-WORKLOAD-MANIFEST-JCS-v1` |
| `payload_generator` | One of `EXP-0001-SHA256-CTR-v1`, `EXP-0001-SHA256-MOTIF-v1`, `EXP-0001-ZERO-v1`. |
| `identity_generator` | `EXP-0001-UUID4-SHA256-v1` |
| `envelope_generator` | `EXP-0001-ENVELOPE-INPUT-v1` |
| `reference_generator` | `EXP-0001-PRIOR-EVENTS-v1` |
| `logical_time_generator` | `EXP-0001-LOGICAL-TIME-v1` |
| `semantic_operation` | `EXP-0001-SEMANTIC-OP-v1` |
| `workload_stream` | `EXP-0001-WORKLOAD-STREAM-v1` |
| `digest` | `SHA-256/FIPS-180-4` |
| `size_class_order` | `EXP-0000-SIZE-CLASS-ORDER-v1` |
| `payload_size` | `fixed-P0`..`fixed-P5`, `mixed-equal-P1-P4`, or `mixed-weighted-P1-P4-v1`. |
| `payload_content` | Respectively `deterministic-high-variation`, `repeated-low-variation`, or `all-zero`, and MUST agree with `payload_generator`. |
| `envelope` | `envelope-minimal`, `envelope-provenance`, `envelope-causal-reference`, or `envelope-correction-retraction-reference`. |
| `temporal` | `time-monotonic-effective`, `time-equal-burst-v1`, `time-late-arriving-v1`, or `time-out-of-effective-order-v1`. |

There is no version negotiation, fallback, prefix match, alias, or best-effort reading. A reader
either explicitly supports this complete combination or returns `unsupported-version` or
`profile-mismatch`. A changed field, type, enum, canonicalization rule, generator algorithm,
digest construction, or compatibility combination requires a new manifest profile and vectors.

### 2.4 Reproduction inputs and counts

`generator_inputs` has exactly: `workload_contract_version` (`u64`, exactly `1`),
`generator_version` (`u64`, exactly `1`), `seed` (`u64`), `stream_namespace` (UUID),
`producer_id` (UUID), `producer_count` (`u64`, exactly `1` for this frozen stream profile),
`controlled_schedule` (state whose present value is a UUID), `base_ns` (`i64`), `unit_ns`
(`i64`, strictly positive), `reference_cardinality` (`u64`), `schema_id` (UUID),
`schema_version` (nonempty string), `envelope_semantic_version` (nonempty string),
`ordinary_fact_type` (nonempty string), `correction_fact_type` (state whose present value is a
nonempty string), `source_provenance` and `actor_provenance` (states whose present values are
nonempty strings). Profile applicability is exact: minimal requires both provenance states and
correction fact type not applicable and reference cardinality zero; provenance requires both
provenance values; causal requires positive cardinality; correction/retraction requires positive
cardinality and a correction fact type. A controlled schedule must agree with every R12 OP1.

`counts` has exactly `operation_count`, `warm_up_operation_count`,
`measured_operation_count`, and arrays `by_segment`, `by_size_class`, `by_envelope_profile`, and
`by_temporal_profile`. Each count is `u64` text. `by_segment` contains exactly two closed
`{count,segment}` objects in order `warm_up`, `measured`. The other arrays contain one closed
`{count,profile}` entry for every value actually present, sorted by the profile strings; zero
entries are forbidden. Their sums each equal `operation_count`; segment counts equal their
top-level counterparts; and warm-up plus measured equals total. Counts must equal the R14 header
and decoded SOP1/OP1 segment and profile bindings. Empty total streams are permitted only with
both segment counts zero and empty profile arrays.

### 2.5 Supersession

`supersession` is either exactly
`{"reason":{"state":"not_applicable"},"supersedes_manifest_ids":[]}` for an original, or
`{"reason":{"state":"present","value":string},"supersedes_manifest_ids":[uuid,...]}` for a
correction. IDs are unique and lexicographically sorted; self-reference, cycles, missing targets,
or targets with a different `workload_id` fail. A correction always receives a new
`manifest_id`, new bytes, creation time, stream reference when bytes changed, and corresponding
R7 `corrects`/`supersedes` provenance edges. The prior identity, bytes, and evidence remain.
Concurrent corrections form an invalid fork until one later manifest explicitly names every
fork head. No ID is reused and no published manifest is mutated or erased.

### 2.6 External manifest-digest descriptor

The manifest digest profile is `EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1`. Its digest is exactly:

```text
SHA-256(ASCII "rusty-data-os/exp1/workload-manifest/v1" || 00 || complete_manifest_bytes)
```

`complete_manifest_bytes` is the complete canonical JCS byte sequence selected in section 1. The
domain prefix is 40 octets including the final `00`. The digest is external to the manifest and
therefore cannot refer to itself: no digest member, placeholder, omitted-member transform, or
second serialization participates. A publication descriptor carries the closed object
`{algorithm,domain,manifest_ref,profile,value}`, where `algorithm` is
`SHA-256/FIPS-180-4`, `domain` is `rusty-data-os/exp1/workload-manifest/v1`, `profile` is
`EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1`, `manifest_ref` is the R7 immutable `ref` for exactly the
manifest bytes, and `value` is the digest rendered as exactly 64 lowercase hexadecimal characters.
The descriptor and its R7 artifact entry MUST agree on manifest identity, byte length,
exact-artifact digest, and URI. Missing descriptors; unknown profiles; altered domains or algorithms; uppercase,
malformed, or unequal values; and any descriptor/manifest-reference mismatch fail `digest` or
`reference`. The manifest, stream, exact stream artifact, and manifest digest are four distinct
bindings and MUST NOT be substituted for one another.

## 3. Validation and record boundary

Validation proceeds bytes/UTF-8/JSON/duplicate/I-JSON/JCS first, then closed schema, scalar
ranges, profiles, cross-field rules, reference resolution including the complete R7 artifact/provenance binding, referenced byte
length and exact artifact SHA-256, R14 stream parsing/counts, the domain-separated workload digest, and
finally the external manifest-digest descriptor against the unmodified complete manifest bytes.
Any failure yields no accepted manifest. Validators must not reorder and accept, normalize text,
repair identifiers, drop unknowns, infer state, clamp counts, select profiles, fetch a substitute,
or recompute and replace declared authority values.

Failure codes are `io`, `length`, `utf8`, `json-syntax`, `duplicate-member`, `non-ijson`,
`noncanonical`, `unsupported-version`, `unknown-field`, `missing-field`, `type`, `range`, `enum`,
`ordering`, `duplicate-or-conflict`, `reference`, `digest`, `profile-mismatch`, `count-mismatch`,
`supersession-cycle`, and `immutable-state`. Digest algorithm/domain/value disagreement,
generator/profile incompatibility, impossible totals, malformed provenance, conflicting manifest
identity, and mutation of published state all fail closed under the corresponding code.

Machine, OS, filesystem, storage, clock observation, compiler/build, adapter/baseline effective
configuration, cache/preconditioning observation, queue/batch/durability setting, deviation,
fault, lifecycle timing, validation outcome, sample, and measurement belong in R7 environment,
series/run configuration, validation, fault, or raw-result records. They MUST NOT be copied into
this manifest. Those records reference this immutable manifest/stream identity and digest.

## 4. Literal documentation vectors

### 4.1 Canonical valid vector M01

M01 binds the uniform one-operation stream derived from R14 `S01`: one P1/high, minimal-envelope,
monotonic-time warm-up operation and no measured operations. The R14 stream is the 55-octet header
with counts `1/1/0`, followed by the eight-octet length `755` and the complete literal S01 bytes. It
is 818 octets; its workload-stream digest is
`0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09`; and its R7 exact-artifact
digest is `789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a`. Thus every operation
uses the manifest's single envelope and temporal profile and M01 can regenerate the stream without
possessing it. The following single line is the **exact canonical JCS text and UTF-8 bytes**, with
no trailing newline:

```json
{"authority_revisions":[{"authority":"EXP-0000-WORKLOADS","revision":{"kind":"git_sha","value":"70a29efd46dd3aee9ea9cb0831d0285b83cdd70a"}},{"authority":"EXP-0001-R12","revision":{"kind":"git_sha","value":"e39551e64d9a799a3d15bf75aa70a323c8e40ca8"}},{"authority":"EXP-0001-R14","revision":{"kind":"git_sha","value":"78b8b35e4efda44a8097db05f396679a1265a239"}},{"authority":"EXP-0001-R16","revision":{"kind":"reviewed_authority_id","value":"documentation-vector-v1"}},{"authority":"EXP-0001-R2","revision":{"kind":"git_sha","value":"2659fb34caf054a7742a854d69d17cdd59bd2040"}},{"authority":"EXP-0001-R7","revision":{"kind":"git_sha","value":"f9d9876cf6599345a2e2244223a530ada2b9a828"}}],"counts":{"by_envelope_profile":[{"count":"1","profile":"envelope-minimal"}],"by_segment":[{"count":"1","segment":"warm_up"},{"count":"0","segment":"measured"}],"by_size_class":[{"count":"1","profile":"P1"}],"by_temporal_profile":[{"count":"1","profile":"time-monotonic-effective"}],"measured_operation_count":"0","operation_count":"1","warm_up_operation_count":"1"},"created_at_utc_ns":"1788134400000000000","generator_inputs":{"actor_provenance":{"state":"not_applicable"},"base_ns":"1000","controlled_schedule":{"state":"not_applicable"},"correction_fact_type":{"state":"not_applicable"},"envelope_semantic_version":"1","generator_version":"1","ordinary_fact_type":"fact-A","producer_count":"1","producer_id":"10213243-5465-4768-899a-abbccddeef00","reference_cardinality":"0","schema_id":"eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee","schema_version":"1","seed":"0","source_provenance":{"state":"not_applicable"},"stream_namespace":"00112233-4455-4677-8899-aabbccddeeff","unit_ns":"10","workload_contract_version":"1"},"manifest_id":"16000000-0000-4000-8000-000000000001","profiles":{"digest":"SHA-256/FIPS-180-4","envelope":"envelope-minimal","envelope_generator":"EXP-0001-ENVELOPE-INPUT-v1","identity_generator":"EXP-0001-UUID4-SHA256-v1","logical_time_generator":"EXP-0001-LOGICAL-TIME-v1","manifest":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","payload_content":"deterministic-high-variation","payload_generator":"EXP-0001-SHA256-CTR-v1","payload_size":"fixed-P1","reference_generator":"EXP-0001-PRIOR-EVENTS-v1","semantic_operation":"EXP-0001-SEMANTIC-OP-v1","size_class_order":"EXP-0000-SIZE-CLASS-ORDER-v1","temporal":"time-monotonic-effective","workload_contract":"EXP-0000-WORKLOADS-v1","workload_stream":"EXP-0001-WORKLOAD-STREAM-v1"},"record_kind":"workload_manifest","schema_version":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","stream_digest":{"algorithm":"SHA-256/FIPS-180-4","domain":"rusty-data-os/exp1/workload-stream/v1","value":"0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09"},"stream_ref":{"artifact_id":"16000000-0000-4000-8000-000000000002","artifact_manifest_ref":{"artifact_id":"16000000-0000-4000-8000-000000000009","byte_length":"1274","sha256":"b65688eb056a71bacaff1178ef4d0693b1c5ef59c43bdbdaa7b360e562f4998c","uri":"https://example.invalid/exp-0001/artifact-manifest.jcs"},"byte_length":"818","created_by_record_id":"16000000-0000-4000-8000-000000000005","media_type":"application/vnd.rusty-data-os.exp1-workload-stream","role":"configuration","sha256":"789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a","uri":"https://example.invalid/exp-0001/s01.stream"},"supersession":{"reason":{"state":"not_applicable"},"supersedes_manifest_ids":[]},"workload_id":"16000000-0000-4000-8000-000000000003"}
```

M01 is 3423 octets. Applying section 2.6 to those literal bytes yields the independently
checkable manifest digest
`68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594`. Its external descriptor is therefore:

```json
{"algorithm":"SHA-256/FIPS-180-4","domain":"rusty-data-os/exp1/workload-manifest/v1","manifest_ref":{"artifact_id":"16000000-0000-4000-8000-000000000001","byte_length":"3423","sha256":"ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86","uri":"https://example.invalid/exp-0001/m01.manifest.jcs"},"profile":"EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1","value":"68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594"}
```

### 4.2 Realizable immutable R7 fixture correction

The owner selected M01 as the canonical positively valid vector. The prior synthetic 4096-octet,
all-`1` reference is superseded by these repository-contained literal R7 record bytes:

```json
{"body":{"artifacts":[{"artifact_id":"16000000-0000-4000-8000-000000000002","byte_length":"818","created_by_record_id":"16000000-0000-4000-8000-000000000005","logical_path":"exp-0001/series/16000000-0000-4000-8000-000000000007/runs/16000000-0000-4000-8000-000000000008/artifacts/16000000-0000-4000-8000-000000000002/configuration","media_type":"application/vnd.rusty-data-os.exp1-workload-stream","retention_state":"published","role":"configuration","sensitivity":"public","sha256":"789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a","uri":"https://example.invalid/exp-0001/s01.stream","validation_report_ids":[]}],"provenance_edges":[{"from_artifact_id":"16000000-0000-4000-8000-000000000002","relation":"generated_from","to_artifact_id":"16000000-0000-4000-8000-000000000001"}],"publication_state":"published","scope":"run","series_freeze":{"state":"not_applicable"}},"correction_reason":{"state":"not_applicable"},"created_at_utc_ns":"1788134400000000000","record_id":"16000000-0000-4000-8000-000000000004","record_kind":"artifact_manifest","run_id":{"state":"present","value":"16000000-0000-4000-8000-000000000008"},"schema_version":"EXP1-R7-JSON-JCS-1","series_id":"16000000-0000-4000-8000-000000000007","supersedes_record_id":{"state":"not_applicable"}}
```

The fixture is exactly **1274 octets** and its R7 exact-artifact digest is
`b65688eb056a71bacaff1178ef4d0693b1c5ef59c43bdbdaa7b360e562f4998c`. It uses the complete
`EXP1-R7-JSON-JCS-1` envelope and closed run-scoped `artifact_manifest` body, including `scope`,
`publication_state`, `series_freeze`, the full stream artifact entry, and the digest-bound internal provenance-edge array. The parsed edge and caller-supplied resolved R7 graph agree on the
`generated_from` relation from stream artifact `...0002` to workload-manifest artifact `...0001`;
`created_by_record_id` binds the stream entry to record `...0005`. The unchanged R14 stream remains
818 octets with workload digest `0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09`
and exact-artifact digest `789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a`.
The resulting M01 values are 3423 octets, manifest digest
`68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594`, and R7 exact-artifact
digest `ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86`. These literals, not
implementation-generated expectations, are the corrected oracle.

The example retains non-production publication identities and URIs, but its manifest-reference
metadata now binds the literal repository-contained R7 fixture above. The manifest and fixture
lengths and digests, R14 stream length, workload-stream digest, and exact stream-artifact digest
are normative and independently reproducible.

### 4.3 Negative vectors

| Vector | Mutation of M01 | Required disposition |
|---|---|---|
| N01 | Add top-level `"extra":true`. | `unknown-field`. |
| N02 | Change manifest/schema or manifest-digest profile to `...-v2`. | `unsupported-version`. |
| N03 | Substitute the stream digest, exact-artifact digest, or manifest digest for another; change any digest to 64 zeroes; or change a domain to `/v2`. | `digest` or `profile-mismatch`; never replace it. |
| N04 | Change total to `2`, measured to `1`, or a segment/profile subtotal without changing the S01 stream. | `count-mismatch`. |
| N05 | Use uppercase/nil/malformed `manifest_id`, a short Git SHA, a 40-hex reviewed identifier, or an unknown/ambiguous revision tag. | `type`, `range`, or `enum`. |
| N06 | Pretty-print, append LF, exchange any two members, or escape a character differently from JCS. | `noncanonical`, even when the JSON value is otherwise equivalent. |
| N07 | Give an original a reason, give a correction no target/reason, self-reference, form a cycle, or correct another workload ID. | `duplicate-or-conflict`, `supersession-cycle`, or `immutable-state`. |
| N08 | Duplicate a JSON name, omit `seed`, use `null`, or use numeric `1` for a count. | `duplicate-member`, `missing-field`, or `type`. |
| N09 | Omit/mismatch the stream role, media type, creating record, artifact-manifest reference, artifact entry, or required provenance edge. | `reference`; no stream or manifest is accepted. |


## 5. Disposition and exclusions

BLK-009 and UNK-019 are resolved **as documentation design only** because this authority uniquely
freezes the profile/version, physical bytes, complete bindings, validation behavior, immutable
correction semantics, and independent vectors. No next implementation increment is automatically
authorized. Manifest/generator implementation, executable schema or validator, fixtures,
generated manifests/workloads/artifacts, Slice C/B1, persistence, replay/recovery, durability,
fault work, capture, descriptive or confirmatory execution, benchmark evidence, dependencies,
Cargo, scripts, workflows, toolchains, production crates, and performance claims remain absent
and unauthorized. UNK-022 remains open for executable validation/capture; BLK-015 and the existing
later implementation/execution gates remain unchanged.
