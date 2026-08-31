# EXP-0001 R16 — Workload-manifest serialization contract

**Authority base:** live `main` at `f38b637b27cf6748c875077a86e5e0f318ba0483`
**Status:** complete as documentation design; BLK-009 is resolved only at that boundary
**Manifest profile:** `EXP-0001-WORKLOAD-MANIFEST-JCS-v1`
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
- An immutable stream reference is the closed object `{artifact_id,byte_length,sha256,uri}`.
  `artifact_id` is a UUID, `byte_length` is `u64`, `sha256` is the R7 exact-artifact
  digest `SHA-256(ASCII "rusty-data-os/exp1/r7/artifact/v1" || 00 || bytes)`, and `uri` is an absolute normalized `file:` or `https:` URI without query or fragment. URI alone
  is never identity.
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
| `stream_ref` | Immutable stream-reference object. The referenced bytes are exactly the R14 stream bytes, not a physical event or adapter encoding. |
| `authority_revisions` | Array of closed `{authority,revision}` objects, sorted by `authority`; exactly one each for `EXP-0000-WORKLOADS`, `EXP-0001-R2`, `EXP-0001-R7`, `EXP-0001-R12`, `EXP-0001-R14`, and `EXP-0001-R16`. `revision` is a nonempty immutable Git commit SHA or reviewed authority identifier. |

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

## 3. Validation and record boundary

Validation proceeds bytes/UTF-8/JSON/duplicate/I-JSON/JCS first, then closed schema, scalar
ranges, profiles, cross-field rules, reference resolution, referenced byte length and exact
artifact SHA-256, R14 stream parsing/counts, and finally the domain-separated workload digest.
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

M01 binds R12/R14 `A01-S2`/W01: two P1/high warm-up operations, no measured operations, R14
stream length `1596`, and digest
`81dbc6b6e33ee775d4b36aeaa0aca45b9649c987f180e378b5d5fbcf1bc3b024`. The following single
line is the **exact canonical JCS text and UTF-8 bytes**, with no trailing newline:

```json
{"authority_revisions":[{"authority":"EXP-0000-WORKLOADS","revision":"reviewed-v1"},{"authority":"EXP-0001-R12","revision":"e39551e"},{"authority":"EXP-0001-R14","revision":"78b8b35"},{"authority":"EXP-0001-R16","revision":"documentation-vector-v1"},{"authority":"EXP-0001-R2","revision":"reviewed-v1"},{"authority":"EXP-0001-R7","revision":"reviewed-v1"}],"counts":{"by_envelope_profile":[{"count":"1","profile":"envelope-minimal"},{"count":"1","profile":"envelope-provenance"}],"by_segment":[{"count":"2","segment":"warm_up"},{"count":"0","segment":"measured"}],"by_size_class":[{"count":"2","profile":"P1"}],"by_temporal_profile":[{"count":"1","profile":"time-equal-burst-v1"},{"count":"1","profile":"time-monotonic-effective"}],"measured_operation_count":"0","operation_count":"2","warm_up_operation_count":"2"},"created_at_utc_ns":"1788134400000000000","generator_inputs":{"actor_provenance":{"state":"present","value":"actor-A"},"base_ns":"1000","controlled_schedule":{"state":"not_applicable"},"correction_fact_type":{"state":"not_applicable"},"envelope_semantic_version":"1","generator_version":"1","ordinary_fact_type":"fact-A","producer_count":"1","producer_id":"10213243-5465-4768-899a-abbccddeef00","reference_cardinality":"0","schema_id":"eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee","schema_version":"1","seed":"0","source_provenance":{"state":"present","value":"source-A"},"stream_namespace":"00112233-4455-4677-8899-aabbccddeeff","unit_ns":"10","workload_contract_version":"1"},"manifest_id":"16000000-0000-4000-8000-000000000001","profiles":{"digest":"SHA-256/FIPS-180-4","envelope":"envelope-provenance","envelope_generator":"EXP-0001-ENVELOPE-INPUT-v1","identity_generator":"EXP-0001-UUID4-SHA256-v1","logical_time_generator":"EXP-0001-LOGICAL-TIME-v1","manifest":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","payload_content":"deterministic-high-variation","payload_generator":"EXP-0001-SHA256-CTR-v1","payload_size":"fixed-P1","reference_generator":"EXP-0001-PRIOR-EVENTS-v1","semantic_operation":"EXP-0001-SEMANTIC-OP-v1","size_class_order":"EXP-0000-SIZE-CLASS-ORDER-v1","temporal":"time-monotonic-effective","workload_contract":"EXP-0000-WORKLOADS-v1","workload_stream":"EXP-0001-WORKLOAD-STREAM-v1"},"record_kind":"workload_manifest","schema_version":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","stream_digest":{"algorithm":"SHA-256/FIPS-180-4","domain":"rusty-data-os/exp1/workload-stream/v1","value":"81dbc6b6e33ee775d4b36aeaa0aca45b9649c987f180e378b5d5fbcf1bc3b024"},"stream_ref":{"artifact_id":"16000000-0000-4000-8000-000000000002","byte_length":"1596","sha256":"aa7945512fee86f05a899a88568d831d990c17b7821ae631114ce4c690b1602d","uri":"https://example.invalid/exp-0001/w01.stream"},"supersession":{"reason":{"state":"not_applicable"},"supersedes_manifest_ids":[]},"workload_id":"16000000-0000-4000-8000-000000000003"}
```

The example intentionally contains fictional publication identities/URI and no generated
artifact. Mixed envelope/temporal counts describe the two operations; the top-level workload
profiles name the selected workload cell and do not erase per-operation bindings.

### 4.2 Negative vectors

| Vector | Mutation of M01 | Required disposition |
|---|---|---|
| N01 | Add top-level `"extra":true`. | `unknown-field`. |
| N02 | Change manifest/schema version to `...-v2`. | `unsupported-version`. |
| N03 | Change either digest value to 64 zeroes or its domain to `/v2`. | `digest` or `profile-mismatch`; never replace it. |
| N04 | Change total to `3`, measured to `1`, or a segment/profile subtotal without changing W01. | `count-mismatch`. |
| N05 | Use uppercase/nil/malformed `manifest_id`. | `type` or `range`. |
| N06 | Pretty-print, append LF, exchange any two members, or escape a character differently from JCS. | `noncanonical`, even when the JSON value is otherwise equivalent. |
| N07 | Give an original a reason, give a correction no target/reason, self-reference, form a cycle, or correct another workload ID. | `duplicate-or-conflict`, `supersession-cycle`, or `immutable-state`. |
| N08 | Duplicate a JSON name, omit `seed`, use `null`, or use numeric `2` for a count. | `duplicate-member`, `missing-field`, or `type`. |

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
