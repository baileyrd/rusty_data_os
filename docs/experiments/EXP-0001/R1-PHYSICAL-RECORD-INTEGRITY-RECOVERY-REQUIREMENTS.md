# R1 — Physical Record, Integrity, and Recovery Requirements

**Status:** Reviewed requirements/research record; R1 complete
**Evidence classification:** Approved experiment-local correctness constraints derived from existing semantics; no implementation, benchmark, or correctness evidence
**Resolves:** BLK-002 and BLK-013
**Constrains without resolving:** BLK-001 and BLK-003
**Applies to:** EXP-0001 physical event records, validation, scanning, replay, and recovery

## 1. Question and authority

R1 answers: **What must a physical event record and a scan of physical records guarantee before a concrete framing or integrity algorithm may be selected?** It turns the semantic event envelope, acknowledgement/durability contract, and crash/recovery correctness contract into minimum physical requirements. It does not demonstrate that an implementation meets them.

The governing inputs are [REQ-001 through REQ-014](../../REQUIREMENTS.md), [ADR-0002](../../adr/ADR-0002-foundational-canonical-history-constraints.md), the [semantic event envelope](../EXP-0000/SEMANTIC-EVENT-ENVELOPE.md), [acknowledgement, visibility, fault, and durability contract](../EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), [crash/recovery correctness contract](../EXP-0000/CRASH-RECOVERY-CORRECTNESS.md), [EXP-0001](../EXP-0001-immutable-event-ingestion.md), and its [execution-readiness plan](EXECUTION-READINESS-PLAN.md). If this record conflicts with those authorities, the earlier authority governs and R1 must be corrected.

This experiment-local record is the correct authority because it constrains pre-implementation EXP-0001 profiles without promoting a durable architecture, public interface, or graduated format. An ADR is therefore neither required nor justified. Promotion beyond the experiment requires the evidence and decision process specified by repository governance.

## 2. Classification of statements

| Class | What R1 records |
|---|---|
| Existing fact | The repository contains semantics/readiness documentation but no Cargo project, physical format, executable validator, fixture, result, or benchmark evidence. |
| Requirement | A normative obligation below that every later eligible physical profile, validator, and recovery procedure must satisfy. |
| Assumption | A bounded premise retained for planning: one preserved artifact can be scanned under one declared profile and configuration, and later work can define finite resource limits. |
| Unresolved choice | A mechanism or policy deliberately left to BLK-001, BLK-003, BLK-011, BLK-012, or later readiness increments. |

Normative terms **MUST**, **MUST NOT**, **REQUIRED**, and **MAY** apply only to eligibility for EXP-0001 claims. They are requirements, not claims of observed behavior.

## 3. Alternatives considered

| Alternative | Disposition and reason |
|---|---|
| Require one integrity capability for every measurement | Rejected. Structural-only records remain useful for explicitly provisional or diagnostic D0/D1 measurements, provided they make no canonical or corruption-detection claim. |
| Permit structural parsing as evidence of canonicality | Rejected. Physical validity cannot prove that a durability boundary completed. |
| Choose framing and an integrity algorithm in R1 | Deferred to BLK-001 and BLK-003. Requirements must precede mechanism selection and stable vectors. |
| Recover by searching past damage for plausible later records | Rejected for canonical replay. It can invent boundaries and silently promote bytes after an undecidable region. |
| Always discard every damaged suffix | Rejected. A conclusively terminal incomplete suffix is distinguishable from ambiguous bytes only after the framing contract proves it. |
| Mutate or repair during validation | Rejected. It destroys repeatability and original fault evidence. A separately recorded transformation may create a new artifact. |
| Make integrity cryptographic | Out of scope. R1 requires accidental-error detection for applicable claims, not authenticity or adversarial protection. |

## 4. Integrity capability policy — BLK-002

### 4.1 Required capability classes

Every versioned physical profile MUST declare exactly which supported integrity mode it uses and MUST carry sufficient version/profile identity to select its complete validation contract. An unknown or unsupported encoding, version, profile, or mode MUST fail closed; a reader MUST NOT guess or downgrade it.

R1 defines these algorithm-neutral capability classes:

1. **Structural-only mode.** This mode provides framing/structural validation but no content-integrity capability. It MAY be used only for explicitly provisional or diagnostic D0/D1 measurements. It makes no corruption-detection, canonical-recovery, D2, or D3 correctness claim. Physical presence and successful parsing never promote a D0/D1 record into canonical history.
2. **Error-detecting mode.** This mode is REQUIRED for every EXP-0001 D2/D3 canonical-history, recovery-correctness, or corruption-detection claim. Its eventual declared algorithm and parameters detect accidental corruption only within their stated capability. It supplies no authenticity, malicious-tampering, encryption, access-control, or authorization guarantee. Algorithm, parameters, representation, and test vectors remain BLK-003 work; physical encoding remains BLK-001 work.

Later profiles MAY introduce additional explicitly named capabilities, but they cannot weaken these eligibility rules or silently reinterpret either class.

### 4.2 Semantic coverage and finalization

For an error-detecting profile, integrity coverage MUST semantically bind every physically represented value whose undetected alteration could change:

- record beginnings, endings, boundaries, or length interpretation;
- envelope version or any semantic-envelope content;
- opaque payload content;
- event, request, and information identities and references;
- assigned sequence, replay, acceptance, or ordering meaning;
- declared encoding, version, integrity mode, or profile identity; or
- any other value used to accept, order, validate, or reconstruct the event.

The eventual physical contract MUST enumerate covered values and every excluded or normalized field, including how the integrity value itself is excluded or normalized. That declaration cannot depend on incidental in-memory representation. R1 does not specify its byte encoding.

Integrity metadata MUST be finalized only after every covered value is final. A finalized physical record is immutable. Any later change requires construction of a new record and recomputation of its integrity metadata; in-place reinterpretation or patching is ineligible.

## 5. Minimum physical-record contract — constraints on BLK-001

A later frozen framing contract MUST provide:

1. deterministic discovery of each record's beginning, ending, and declared extent;
2. an unambiguous determination that a record is complete and finalized;
3. unambiguous version and integrity-profile identification;
4. validation of all controlling values before allocation, slicing, or indexed access uses them;
5. rejection of arithmetic overflow, impossible lengths, excessive declared extents, configured resource-limit violations, and any scan step that cannot advance;
6. deterministic parsing independent of machine endianness, alignment, pointer width, locale, and incidental memory layout;
7. lossless preservation of the complete semantic envelope and opaque payload; and
8. an explicit separation between physical completeness/validity and canonical commit status.

The scanner MUST be bounded by declared finite configuration limits and MUST either advance by a conclusively validated extent or stop with an explicit classification. A complete physical record MUST NOT be inferred canonically committed merely because its bytes exist.

BLK-001 remains open for concrete field order, integer representation, endianness, alignment, padding, magic values, commit markers, segment layout, and encoding. R1 neither prefers nor rejects any mechanism that meets these requirements.

## 6. Deterministic scan classifications

One scan MAY report multiple non-conflicting observations (for example, a valid prefix followed by terminal truncation), but every examined region MUST receive a deterministic disposition. At minimum the frozen scanner contract MUST distinguish:

| Classification | Required disposition |
|---|---|
| Complete and valid record | Include only if its physical and integrity checks pass; canonical replay additionally requires canonical eligibility. |
| Terminal incomplete/truncated record | Exclude and report; accept this classification only when frozen framing conclusively proves an incomplete record at the terminal suffix. |
| Malformed boundary or length | Stop and fail closed; never allocate or index from the malformed value. |
| Integrity failure | Exclude the affected record, stop canonical scanning, and report the declared profile and safely known location. |
| Unsupported encoding/version/integrity mode | Stop and fail closed without guessing, downgrade, or reinterpretation. |
| Interior corruption or incomplete interior record | Stop and fail closed; never skip it to promote later bytes. |
| Trailing garbage | Report and fail closed unless the framing contract conclusively classifies it as a terminal incomplete record; arbitrary bytes are not truncation. |
| Duplicate event identity | Detect, report both safely known occurrences, and never silently accept either ambiguity into replay. |
| Duplicate sequence position | Detect, report, and fail closed for canonical replay. |
| Sequence gap | Report; do not automatically classify as corruption while BLK-011 remains open. |
| Out-of-order sequence | Detect, report, and never reorder or silently accept it. |
| Undecidable canonical status | Exclude from canonical replay and fail closed; physical validity alone cannot decide it. |
| I/O error or resource-limit failure | Stop with an explicit error and incomplete scan result; never present an unexamined suffix as valid. |

Exact subcategories and report schemas may be refined later, but no refinement may merge a fail-closed condition into successful canonical replay.

## 7. Scan, replay, recovery, and repair invariants — BLK-013

1. A conclusively valid and canonically eligible prefix MAY be replayed when followed by a conclusively terminal incomplete suffix. The suffix MUST be excluded and reported.
2. A suffix is terminal incomplete only when the frozen framing contract proves that classification. Ambiguous bytes are undecidable and fail closed.
3. A scanner MUST NOT skip interior corruption, malformed framing, incomplete interior content, or undecidable bytes and continue as though later bytes were valid canonical history.
4. Validation and recovery MUST NOT invent, reorder, overwrite, silently repair, or promote records.
5. Sequence gaps MUST be reported but are not automatically corruption until BLK-011 resolves failed/abandoned-position policy.
6. Duplicate permanent event identities, duplicate sequence positions, and out-of-order positions MUST always be detected and MUST NOT be silently accepted.
7. Validation and recovery MUST be deterministic, idempotent, bounded in resource use, and guaranteed to make progress or stop with an explicit classification/error.
8. Repeated scans of the same preserved artifact with the same reader/profile and configuration MUST produce the same classifications and replay result.
9. Validation MUST NOT mutate its input. The original faulted artifact SHOULD be preserved before repair or truncation where practical.
10. A repair/truncation tool MUST be separate from validation, record its exact input identity and transformation, and produce a new artifact. Repaired output MUST NOT replace the original correctness evidence.
11. When later framing makes it safely possible, reports MUST include exact byte offsets/extents and any safely decoded event identity and sequence. Unsafe decoding MUST NOT be attempted merely to enrich a report.
12. A checkpoint remains derived. It MAY establish a scan starting point only after its exact canonical-history position is validated; checkpoint format remains unresolved.

These rules resolve the minimum append/replay recovery behavior, not concrete fault injection, filesystem operations, retry behavior, or how gaps affect future assignment.

## 8. Durability and canonicality invariants

- D0 and D1 are provisional and noncanonical even when their bytes survive a fault.
- D2 and D3 events MAY be canonical only after their declared durability boundary completes under a separately frozen platform contract.
- Physical completeness, structural validity, and integrity validity are necessary but not sufficient for canonical recovery.
- A persistence or synchronization error cannot produce canonical success or a successful canonical acknowledgement.
- Commit before acknowledgement can yield an explicit uncertain caller outcome while recovery contains exactly one valid canonical event; retry policy remains BLK-012.
- Observation and recovery metadata remain outside the immutable original event.
- Validation MUST combine physical findings with the authoritative lifecycle/durability evidence permitted by the frozen contract; if canonical status remains undecidable, it fails closed.

For D3 specifically:

1. exact group membership and shared-outcome evidence required by the existing correctness contract MUST be preserved and validated;
2. grouping MUST NOT be represented or interpreted as an atomic multi-event transaction;
3. a shared synchronization failure permits no successful canonical acknowledgement for any member;
4. physical residue MUST NOT be promoted to make a group appear complete; and
5. recoverability is assessed per event and checked against the recorded group outcome.

## 9. Documentation-only scenario matrix

These examples define required outcomes, not byte fixtures, executable tests, algorithm vectors, or evidence.

| Scenario | Required observation or outcome |
|---|---|
| Zero, minimum nonzero, and maximum configured legal payload | Deterministically accept if the complete profile is valid; preserve payload exactly; reject any extent beyond the configured maximum before allocation. |
| Large but legal payload | Validate with bounded resources and no overflow; size alone cannot change semantic content or canonicality. |
| Adjacent valid records | Discover both extents without overlap, gap, or dependence on memory alignment; replay only canonically eligible records in assigned order. |
| Truncation in each logical region (framing/profile, envelope, payload, integrity/finalization) | If and only if terminal framing proves incompleteness, replay a valid eligible prefix and exclude/report the suffix; the same damage in the interior fails closed. |
| Covered envelope or payload value altered | Error-detecting mode reports integrity failure within its capability; structural-only mode makes no detection/canonical claim. |
| Covered length/boundary information altered | Reject malformed/inconsistent structure or integrity; never use an unchecked value for allocation or access. |
| Covered version or integrity-profile information altered | Reject by integrity/profile validation or as unsupported; never select a guessed fallback profile. |
| Impossible, malformed, overflowing, or excessive length | Stop explicitly before allocation/indexed access; never wrap or make a non-progressing scan. |
| Unsupported encoding version or integrity mode | Fail closed and report the unsupported identity. |
| Terminal damage versus identical-looking interior damage | Only conclusively proven terminal incompleteness permits the valid prefix result; interior or ambiguous damage stops canonical scanning. |
| Valid prefix plus arbitrary trailing garbage | Report trailing garbage and fail closed unless framing conclusively proves terminal incompleteness. |
| Duplicate sequence position | Detect and fail closed; never choose a winner. |
| Reordered sequence positions | Detect and fail closed; never reorder to manufacture validity. |
| Missing sequence position | Report a gap without automatically declaring corruption; defer its lifecycle meaning to BLK-011. |
| Duplicate permanent event identity | Detect and fail closed; never treat the second physical occurrence as a second fact. |
| D0/D1 bytes survive a fault | They remain provisional and noncanonical regardless of parse or integrity success. |
| D2 record before its declared boundary | It cannot be canonically successful; physical residue is not promoted. |
| D2 record after successful declared boundary | It is eligible for canonical recovery only if physical/integrity and lifecycle checks also pass. |
| Commit completes but acknowledgement is lost | Report caller outcome as uncertain; recovery may contain exactly one valid canonical event, without inventing an acknowledgement. |
| D3 shared synchronization succeeds | Validate exact membership/outcome evidence and assess each member; successful members may be canonical under the platform contract. |
| D3 shared synchronization fails | No member receives successful canonical acknowledgement; residue is not promoted. |
| D3 has partial physical residue | Assess each event, preserve the failed shared outcome, and never fill, invent, or promote members to simulate group completion. |
| I/O stops the scan or a resource limit is reached | Report the explicit failure and validated prefix; do not classify the unexamined region as valid or canonically replayable. |

Physical test vectors remain ineligible until BLK-001 and every applicable BLK-003 algorithm/profile are resolved, versioned, and accompanied by stable vectors. This matrix cannot authorize fixtures or validators.

## 10. Consequences, risks, and revisit conditions

### Consequences

- BLK-002 is resolved by the two minimum capability classes, profile declaration, semantic coverage, finalization, and fail-closed rules.
- BLK-013 is resolved by the minimum framing constraints, classification table, deterministic valid-prefix rule, fail-closed scan/replay invariants, and repair separation.
- BLK-001 and BLK-003 now have eligibility constraints but remain open; implementation cannot begin from this record alone.
- Error-detecting metadata creates physical and computational cost that later experiments must measure rather than assume acceptable.

### Risks

- A later framing candidate may be unable to prove terminal incompleteness without adding explicit structure.
- An eventual integrity capability may leave correlated or out-of-scope errors undetected; its declared capability and fault coverage must remain precise.
- Canonicality may remain undecidable until lifecycle and platform contracts are frozen.
- Resource limits can change which artifact is fully scannable; configuration is therefore part of repeatability identity.

### Revisit conditions

Revisit and supersede R1 if a candidate cannot meet a requirement, a declared fault lacks a deterministic classification, later primary evidence invalidates an assumption, a new durability mode changes canonicality, or promotion beyond EXP-0001 is proposed. A relaxation requires an explicit replacement record and updated traceability; an implementation cannot silently redefine these requirements.

## 11. Explicit deferrals and authorization state

R1 deliberately does **not** choose or resolve:

- BLK-001 encoding/framing details, including bytes, endianness, field layout/order, widths, alignment, padding, magic values, markers, or segments;
- BLK-003 algorithm, parameters, encoding, capability strength, or vectors;
- BLK-011 sequencing-gap lifecycle policy or BLK-012 retry/idempotency and uncertain-outcome lifecycle;
- identity/time algorithms, concurrency, transaction semantics, checkpoint format, distributed behavior, concrete fault mechanisms, filesystem APIs, synchronization primitives, or platform durability;
- cryptographic authenticity, malicious-tampering protection, encryption, authorization, or access control; or
- any performance or correctness-evidence claim.

Cargo bootstrap, Rust implementation, fixtures, validators, byte vectors, executable tests, descriptive execution, and confirmatory execution remain unauthorized. R2—deterministic workload bytes, identity/reference inputs, and digests—is the next recommended bounded documentation increment.

## 12. Traceability and supersession

| R1 output | Traces to |
|---|---|
| Integrity capability and coverage policy | BLK-002; UNK-012 policy portion; REQ-001 through REQ-010, REQ-013, REQ-014 |
| Physical-record constraints | BLK-001 constraint only; UNK-001 remains open; semantic envelope |
| Scan/replay/recovery invariants | BLK-013; UNK-015 policy portion; crash/recovery correctness contract; REQ-001–REQ-006, REQ-009, REQ-012–REQ-014 |
| Gap/duplicate/uncertain classifications | Constraints on BLK-011 and BLK-012; UNK-016 and UNK-017 remain unresolved |
| D0–D3 canonicality | Acknowledgement/durability and crash/recovery contracts; no platform contract selected |

This record is authoritative for R1 until a later repository record explicitly states that it supersedes this path, identifies the replaced sections, explains the evidence or requirement change, and synchronizes the readiness plan, project status, unknowns, research questions, experiment definition, and traceability registry. Older records remain historical; supersession never retroactively changes evidence classifications or makes repaired artifacts original evidence.
