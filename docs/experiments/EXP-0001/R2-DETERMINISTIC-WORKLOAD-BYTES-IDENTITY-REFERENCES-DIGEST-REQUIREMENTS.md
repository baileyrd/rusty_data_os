# R2 — Deterministic Workload Bytes, Identity/Reference Inputs, and Digest Requirements

**Status:** Complete requirements and reference-vector-plan increment; no algorithm or serialization selected
**Evidence classification:** Experiment-local requirements and planning only; not implementation, a fixture, a validator, correctness evidence, or benchmark evidence
**Scope:** Constraints on BLK-006 through BLK-009 for EXP-0001

## 1. Governing question and authority

> How can semantically equivalent EXP-0001 operation streams eventually be regenerated and verified byte-for-byte across Data OS and baseline implementations without confusing shared semantic input, adapter encoding, physical records, or observed execution order?

This record refines the [EXP-0000 workload contract](../EXP-0000/WORKLOADS.md), the [semantic envelope](../EXP-0000/SEMANTIC-EVENT-ENVELOPE.md), and the [EXP-0001 readiness plan](EXECUTION-READINESS-PLAN.md). It preserves R1's physical-boundary and fail-closed requirements but selects no physical representation. Approved semantics and the frozen semantic size-class-order algorithm are facts. Concrete generator, identity, digest, checksum, encoding, and manifest-serialization mechanisms are unresolved choices. Platform independence and reproducibility are requirements; their feasibility and cost remain assumptions to validate.

No ADR is required: this record makes no durable architecture selection. It is a bounded experiment-local requirements refinement. A later selection must follow repository governance and use a separately reviewable record; an ADR is warranted only if evidence supports promotion beyond the experiment.

## 2. Normative language and input model

`MUST`, `MUST NOT`, `SHOULD`, and `MAY` are normative. A future generator MUST accept the following typed, versioned, serialization-neutral tuple. Field names illustrate meanings, not an encoding or API.

| Typed input | Required meaning |
|---|---|
| `workload_contract_version`, `generator_profile_version` | Independently version the semantic contract and applicable generation rules. Unsupported versions fail. |
| `segment` | Closed enumeration distinguishing `warm-up` from `measured`; segment identity participates in generation even when all other values match. |
| `seed` | Canonical unsigned-decimal representation of an unsigned 64-bit value; no sign, whitespace, separators, alternate radix, or leading zeros except the single value `0`. |
| `segment_operation_ordinal` | Non-negative, segment-local ordinal with a declared legal bound; ordinal zero in each segment is distinct. |
| `payload_size_class`, `payload_length` | Assigned P0–P5 class and its exact byte length; both are checked against the frozen class definition. |
| `payload_content_profile` | Versioned `deterministic-high-variation`, `repeated-low-variation`, or `all-zero` profile. |
| `envelope_profile` | Versioned `envelope-minimal`, `envelope-provenance`, `envelope-causal-reference`, or `envelope-correction-retraction-reference` profile. |
| `envelope_semantic_version`, `event_fact_type`, `schema_identity`, `schema_version` | The envelope contract version and the exact event/fact and opaque-payload schema meanings required by the selected profile; none may be inferred from adapter defaults. |
| `source_provenance`, `actor_provenance` | Applicable source and actor provenance inputs required by the selected envelope profile, with explicit absence where they do not apply. |
| `temporal_profile`, `logical_effective_time_parameters` | Versioned logical effective-time relationship and all logical ordinal/offset parameters required to derive it. |
| `producer_identity`, `producer_local_ordinal` | Present where producer assignment applies; producer-local ordinal is independently bounded and must agree with the operation assignment. |
| `controlled_global_schedule_identity` | Present only when a frozen controlled global schedule exists; otherwise explicitly absent. |
| `request_identity_namespace`, `event_identity_namespace`, `information_identity_namespace`, `reference_generation_namespace` | Four distinct, versioned generation domains; equality or implicit reuse is invalid wherever the corresponding identities or references apply. |
| `reference_cardinality`, `target_selection_profile`, `reference_fact_semantics` | Exact cardinality, a versioned deterministic selection rule, and the applicable causal, correction, or retraction fact/reference meaning, including the ordinary-event prefix requirement where applicable. |

Every algorithm MUST domain-separate purpose, profile, segment, the request-, event-, information-, and reference-generation namespaces wherever applicable, and every applicable version. Typed fields and sequences MUST have unambiguous boundaries and type identities; raw concatenation whose decomposition is not unique is forbidden. Required fields cannot be inferred from implementation defaults.

Generation MUST NOT depend on wall-clock or benchmark time; system, durability, or observation time; thread scheduling; uncontrolled producer interleaving; host endianness, pointer width, alignment, locale, or filesystem; Data OS or baseline behavior; or incidental random-number-library defaults. It MUST fail explicitly for incomplete or inconsistent tuples, unsupported versions/profiles, invalid lengths or ordinals, arithmetic overflow, and declared resource-limit violations.

## 3. Payload-content requirements (BLK-006)

For every valid tuple, output length MUST equal the exact P0–P5 length assigned by the frozen workload contract, including zero bytes for P0. Identical complete versioned inputs MUST produce identical bytes across implementations. Warm-up and measured domains MUST remain distinct even at equal seed and ordinal.

- `deterministic-high-variation` MUST provide deterministic byte-to-byte and event-to-event variation sufficient for its neutral-workload purpose where the assigned payload length permits variation. P0's required empty output cannot exhibit variation and is not a generation failure. The profile MUST NOT be described as cryptographic randomness or measured entropy.
- `repeated-low-variation` MUST use the later profile's exactly specified repeat/variation rule and MUST remain domain-separated from other profiles.
- `all-zero` MUST yield the requested count of zero bytes only because that profile explicitly requests it.

No adapter may substitute zeroes, compression-friendly patterns, reused implementation-native buffers, or any convenient local representation for generated bytes. Event-to-event variation is mandatory where the chosen profile specifies it. An inability to allocate or emit the exact requested length is a failure, not a truncated or substituted success.

R2 constrains BLK-006 but leaves it open: the expansion mechanism, parameters, dependency, rationale, and stable vectors are not selected.

## 4. Identity, envelope, reference, and temporal inputs (BLK-007)

R2 specifies generator inputs, not identity algorithms, assignment authority, or lifecycle. BLK-004 and R3 retain those decisions.

- Request, event, information, and reference-generation domains MUST remain distinct wherever applicable and MUST NOT be conflated. Information identity has its own versioned namespace and remains distinct from request and event identity.
- Generated identities MUST be stable across adapters and physical representations. Their complete input tuples MUST be deterministic and versioned.
- Envelope meaning MUST be reproducible from explicit typed inputs and versioned rule references: envelope semantic version, event/fact type, schema identity/version, and applicable source/actor provenance. `envelope-causal-reference` and `envelope-correction-retraction-reference` MUST also determine their exact fact/reference semantics rather than relying on adapter interpretation.
- A detected collision MUST fail generation with the conflicting inputs identified; silent retry, remapping, replacement, or namespace switching is forbidden.
- Causal and correction/retraction targets MUST be valid earlier generated event identities in the permitted stream. Cardinality and target selection MUST be deterministic, and a target-bearing stream MUST declare any required prefix of ordinary target events.
- Self, future, missing, cross-stream, wrong-kind, duplicate-when-forbidden, or otherwise invalid targets MUST fail explicitly.
- Generator inputs MUST NOT decide canonical commit, retry/idempotency behavior, uncertain outcomes, or sequencing-gap policy. Assigned sequence, system time, durability time, observation time, identity-assignment authority, and lifecycle capture points remain deferred to R3 and MUST NOT be fabricated as R2 generator inputs.

Time remains first-class. The generator produces only the workload's logical effective-time relationships. It MUST NOT fabricate or select representations for system-acceptance, durability, or observation time. Effective time MUST NOT be used as assigned sequence or replay order, including for late-arriving or out-of-effective-order profiles.

R2 therefore constrains BLK-007 while leaving BLK-004/005/011/012 and BLK-007 open.

## 5. Digest purposes and byte domains (BLK-008)

Digest records MUST declare purpose, algorithm identity, algorithm/profile version, exact byte-domain specification and version, output representation, and digest value. Domains MUST be separated so that a value for one purpose cannot be interpreted as another.

### 5.1 Semantic operation-stream digest

This digest covers one unambiguous canonical byte domain for the exact ordered semantic stream and every distinguishing shared input. Coverage MUST include segment boundaries; workload and generator versions/profiles; exact payload bytes; request, event, and information identities as applicable; references; logical temporal inputs; producer assignment and producer-local order; and a controlled global schedule when declared.

An uncontrolled cross-producer interleaving and resulting assigned-sequence mapping MUST NOT enter the frozen shared-input digest: they are observed result data. Their separately recorded artifact or result may have its own digest.

### 5.2 Exact artifact digest

This digest covers exactly the stored bytes, from byte zero through the declared byte length, with no parsing or semantic normalization. Its record MUST pair the digest with byte length, media type and role, algorithm/profile identity, and provenance. Changing one byte or changing serialization MUST create a different exact-artifact identity.

### 5.3 Manifest digest

This digest covers the future canonical manifest serialization. The later contract MUST prevent self-reference using an explicitly declared excluded field/domain, an external digest descriptor, or an equivalent unambiguous construction. The exclusion and its canonical-byte effect MUST be versioned and testable.

A digest alone implies no authenticity, malicious-tampering resistance, authorization, or encryption. A future contract must explicitly select and validate any such capability. R2 selects neither SHA, BLAKE, CRC, nor any other digest/checksum mechanism, so BLK-008 remains open.

## 6. Canonical manifest-serialization requirements (BLK-009)

A later serialization MUST:

1. represent every logical EXP-0000 workload-manifest field and produce exactly one canonical byte representation per logical manifest;
2. parse unambiguously and reject duplicate or conflicting fields;
3. declare field ordering; numeric syntax/range; string encoding and normalization; collection ordering; optional/missing semantics; unknown-field behavior; and version-compatibility rules;
4. distinguish absent, empty, zero, unknown, unavailable, and inapplicable values;
5. preserve exact generator, stream, profile, segment, reference, temporal, execution, durability, and verification identities;
6. remain independent of platform endianness, width, locale, alignment, and filesystem behavior;
7. support immutable correction and supersession by linking a new manifest without overwriting its predecessor;
8. allow large stream artifacts to remain external through immutable references, exact lengths, digests, roles, media types, and provenance; and
9. keep manifest bytes, semantic-stream bytes, encoded-event bytes, and physical-record bytes as separate named domains.

Canonical JSON, CBOR, or any other serialization is not selected. BLK-009 remains open pending a versioned selection, rationale, validator rules, and stable vectors.

## 7. Layered reference-vector plan

These are documentation-level case requirements, not executable fixtures and not evidence.

| Layer | Required future cases |
|---|---|
| Input normalization | Canonical typed tuples; seed `0` and `18446744073709551615`; malformed decimal forms; missing, ambiguous, unsupported-version, overflowing, and resource-limit cases. |
| Payload generation | P0 through P5 exact lengths; warm-up/measured separation; ordinal zero, adjacent ordinals, profile boundaries, and large legal ordinals; every content profile; prohibited substitution cases. |
| Envelope and identity inputs | `envelope-minimal`, `envelope-provenance`, `envelope-causal-reference`, and `envelope-correction-retraction-reference`; envelope semantic version; event/fact type; schema identity/version; applicable source/actor provenance; distinct request, event, information, and reference-generation namespaces; collision failure; incomplete and cross-namespace inputs. |
| References | Causal and correction/retraction fact/reference semantics; first valid target; ordinary-event prefix; multiple targets; deterministic cardinality/order; and self/future/missing/cross-stream/wrong-kind invalid targets. |
| Logical time | Every temporal profile, including late-arriving and out-of-effective-order events; cases proving effective time does not reorder replay. |
| Producer order | Single producer, producer-local order, controlled global order, and an explicit uncontrolled-interleaving declaration whose observed mapping is excluded from shared input. |
| Serialization | Canonical manifest cases after selection, plus duplicate/conflicting, absent/empty/zero/missing-state, normalization, unknown-field, compatibility, correction, and external-artifact cases. |
| Digests | Semantic stream, manifest, and exact-artifact domains; one-byte, serialization, purpose, boundary, provenance, and substitution distinctions. |

Every future vector MUST record its vector schema/version and purpose; exact typed inputs; exact expected bytes or digest; expected success or named failure classification; algorithm/profile and serialization versions; provenance and rationale; an independent implementation or hand-check procedure; and a supersession relationship when corrected. Failure vectors record no invented successful bytes.

Stable generator, digest, and manifest vectors require their concrete selections and rationale first. R2 cannot create physical-record vectors. Those remain blocked by BLK-001 and every applicable BLK-003 algorithm/profile, each of which must be selected, versioned, justified, and backed by stable vectors before physical-record work is eligible.

## 8. Selection criteria and alternatives considered

Later generator, identity, digest, or serialization selection records MUST document:

- alternatives considered and selection/rejection rationale;
- a complete public specification and version-stability expectation;
- platform-independent behavior and independently reproducible derivation or reference vectors;
- availability in Rust and every relevant baseline environment;
- implementation/dependency complexity, license, maintenance, and supply-chain implications;
- only the collision, error-detection, or statistical properties actually required;
- performance cost as a quantity to measure rather than assume;
- security capabilities and explicit non-capabilities; and
- compatibility, migration, correction, and supersession policy.

The alternative categories retained are pre-generated immutable streams versus deterministic regeneration; counter/expansion, permutation, or PRNG-style generation; direct versus derived identity construction; cryptographic, non-cryptographic, and error-detection digest families; and text or binary canonical manifests. R2 rejects none categorically and selects none. Incidental library defaults, ambiguous concatenation, implementation-native serialization, and silent collision/substitution recovery are ineligible because they violate the requirements independent of algorithm family.

## 9. Consequences, risks, deferrals, and revisit conditions

R2 makes the eventual equivalence boundary reviewable and distinguishes three digest purposes. It constrains BLK-006 through BLK-009 but resolves none: exact algorithms/serializations, rationale, validators, and stable vectors remain absent. BLK-002 and BLK-013 remain resolved by R1; their status is unchanged.

Risks include a later candidate lacking cross-language implementations, canonicalization hiding logical distinctions, generators exceeding resource limits at legal extremes, collisions invalidating a stream, and digest labels being mistaken for security guarantees. Performance and collision/error behavior require later validation.

Explicitly deferred are BLK-001/003/004/005/011/012 and all other readiness blockers; identity assignment lifecycle; timestamp representations and clocks; retry/idempotency and uncertain outcomes; sequencing gaps; concurrency policy; transactions; checkpoints; platform durability; distributed behavior; event/record encoding; executable validation; and artifact-retention mechanisms. Cargo, Rust code, scripts, fixtures, validators, implementation, descriptive execution, and confirmatory execution remain unauthorized. No payload bytes, identities, digest values, serialized manifests, physical records, correctness results, or benchmark results are produced here.

Revisit R2 if a candidate cannot encode its typed inputs without ambiguity, equivalent implementations disagree, substitution cannot be detected, lifecycle work changes an input meaning, a new workload profile is introduced, or primary evidence invalidates an assumption. Changes require an explicit superseding record and synchronized continuity documents; earlier records and vectors remain immutable historical artifacts.

## 10. Traceability and supersession

| Output | Traceability |
|---|---|
| Typed generator input and payload requirements | BLK-006; UNK-018; EXP-0000 workload contract; REQ-003–REQ-010, REQ-013, REQ-014 |
| Identity/reference/temporal inputs | Constraints on BLK-007 and BLK-004/005/011/012; UNK-002/003/009/010/013/016–018; semantic envelope |
| Digest domains | BLK-008; UNK-018/022; benchmark methodology and raw-result artifact contract |
| Manifest canonicalization requirements | BLK-009; UNK-019; EXP-0000 workload manifest |
| Reference-vector plan | BLK-006–009 constraints; physical vectors remain gated by BLK-001/003 |

This record is authoritative for R2 until a later repository record explicitly identifies the superseded sections, rationale or evidence, compatibility effect, and replacement. R3—identity, time, gaps, retry, and uncertain-outcome lifecycle—is the next recommended bounded increment.
