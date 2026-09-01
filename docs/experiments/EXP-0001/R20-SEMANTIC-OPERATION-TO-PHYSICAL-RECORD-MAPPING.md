# R20 Semantic Operation to Physical Record Mapping

**Profile:** `EXP-0001-SOP1-TO-EXP1-B1-RF1-D1-v1`
**Status:** frozen documentation/governance decision
**Evidence classification:** documentation design; not implementation, execution, or experimental evidence

## 1. Decision and boundary

R20 resolves the semantic-to-physical blocker recorded by [R19](R19-SLICE-C-B1-CLOSURE-AND-DESCRIPTIVE-D1-HARNESS-READINESS.md). Every validated R14 `EXP-0001-SEMANTIC-OP-v1` operation maps to **exactly one** `EXP1-B1-RF1` type-3 provisional record. There is no binding, reservation, membership, final-event, or commit record in this D1 mapping. The record is noncanonical lifecycle evidence and remains provisional after a successful OS-buffer write.

This decision does not reinterpret SOP1 as a canonical envelope. It selects the complete canonically serialized SOP1 byte string as R5's D1 `stable_core`: SOP1 contains the complete ENV1 and opaque payload and is independently validated before construction. It does not make SOP1 a canonical storage record or canonical committed history, and it does not add a lifecycle field that SOP1 does not contain.

The mapping is deterministic for the tuple `(validated SOP1 bytes, assigned_sequence, physical_ordinal)`. The last two values are explicit later-ingestion inputs. They are not derived from a workload ordinal. This qualification is necessary: R14 expressly excludes runtime assigned sequence and interleaving from the semantic stream, while R3 makes sequence engine-assigned. Collapsing either ordinal into the other would change those earlier decisions.

## 2. Exact record construction

For semantic stream index `i` (zero-based, including warm-up before measured operations), validate and decode the complete SOP1 with `exp1-workload-conformance`, then construct this `exp1_record_format::Record`:

```text
Record {
  physical_ordinal: supplied_physical_ordinal,
  integrity: Structural,
  body: Provisional {
    event_id: SOP1 tag 06,
    sequence: supplied_assigned_sequence,
    group_id: 0,
    member_index: 0,
    member_count: 1,
    stable_core: complete SOP1 bytes, byte-for-byte,
  }
}
```

`supplied_physical_ordinal` is the next physical ordinal in the destination artifact: 1 for an empty artifact, otherwise the validated last ordinal plus one. It must be nonzero and consecutive. `supplied_assigned_sequence` is the nonzero sequence assigned by later ingestion execution. Within one produced artifact, operations are submitted in semantic stream index order and supplied sequences must be strictly increasing; gaps are permitted, but duplicate or decreasing values fail. Physical ordinal order therefore preserves submission order, while effective time never changes it. A controlled schedule may determine later submission order only when its separately validated schedule authority says so; R20 does not invent scheduling.

There is one physical record per accepted semantic operation, so operation `i` consumes exactly physical ordinal `start + i`. No fixed multi-record group, implicit record, padding, alignment, or lifecycle companion exists.

## 3. Exhaustive semantic-field placement

The `stable_core` is the complete SOP1 record, so the following placement is exact rather than a lossy projection.

| Semantic value | Physical placement |
|---|---|
| Workload version, generator version, segment, seed, segment operation ordinal, size/content/envelope/temporal profiles, stream namespace, producer identity, producer-local ordinal, controlled schedule | SOP1 tag `01`, complete OP1 bytes, inside `stable_core` |
| Payload profile and exact opaque payload bytes, including P0 empty payload | SOP1 tags `02` and `03` inside `stable_core`; payload is not copied or normalized |
| Request identity | SOP1 tag `05`, and ENV1 tag `08` within SOP1 tag `09`; both must agree |
| Event identity | Record body `event_id`, SOP1 tag `06`, and ENV1 tag `09` within SOP1 tag `09`; all three must agree |
| Information identity | SOP1 tag `07` and ENV1 tag `0a` within SOP1 tag `09`; both must agree |
| Envelope semantic version and fact/event type | ENV1 tags `02` and `03` within SOP1 tag `09` |
| Schema identity and version | ENV1 tags `04` and `05` within SOP1 tag `09` |
| Source and actor provenance | ENV1 tags `06` and `07`; exact option encoding and bytes are retained |
| Logical/effective time | ENV1 tag `0b`, signed `i64be`; SOP1 base/unit tags `0c`/`0d` retain its generation inputs |
| Reference meaning and ordered target EventIds | ENV1 tags `0c` and `0d`; causal, correction, and retraction remain distinct enum values |
| Assigned local sequence | Type-3 body `sequence` only; it is absent from and must not be inserted into SOP1/ENV1 |
| Physical position | RF1 header `physical_ordinal` only; it is not semantic operation ordinal or sequence |

All five operation cases use the same one-record shape: minimal ordinary, provenance ordinary, causal ordinary, correction, and retraction. Profile/meaning differences remain in ENV1. Correction and retraction append their own provisional records and preserve the ordered prior ordinary EventIds as targets; they never replace, mutate, or omit a target record.

## 4. Ownership and excluded lifecycle values

The deterministic workload supplies every SOP1 byte: payload; request, event, and information identities; schema identity/version; fact type; envelope version; provenance; references and their meaning; effective time; generator inputs; and semantic ordinals. Later ingestion alone supplies assigned sequence and next physical ordinal after validating the destination prefix.

System-acceptance time, durability time, acknowledgement, observation time, lifecycle monotonic samples, attempt identity, acceptance/rejection outcome, and persistence/capture evidence are not SOP1 inputs and have no field in this type-3 record. They must be retained only in the later R7 result/lifecycle evidence authorized for that execution. They may not be placed in `stable_core`, substituted for effective time, or synthesized during mapping. D1 has no durability time, canonical commit, recoverable acknowledgement, or durability claim.

This is the complete provisional D1 representation. It does not use or define R5 types 1, 2, 4, 5, or 6; CRC-backed finalization; D2/D3; `fsync`; group commit; recovery of canonical events; or stable-storage behavior. Structural framing detects structure, not corruption or durability.

## 5. Framing, validation, and deterministic failure

### 5.1 Implementation ownership and dependency direction

The later mapper implementation has exactly one owner: a new public `mapping` module in the existing
`exp1-raw-append-replay` crate. Its public API is a pure mapping boundary: it accepts complete SOP1
bytes plus the caller-supplied sequence/ordinal state and returns either one complete validated RF1
frame (and the checked next state) or an error. SOP1 field parsing and cross-field checks are internal
to that module. The API does not accept a file, appender, writer, or path, and calling `RawAppender`
or otherwise integrating append/reopen behavior is excluded from this correctness tranche.

The only permitted workspace path dependencies for `exp1-raw-append-replay` are its existing direct
dependency on `exp1-record-format` and one new direct dependency on
`exp1-workload-conformance`. The latter dependency is required so the physical-boundary mapper calls
the semantic authority's validator rather than reproducing it. This keeps both authority crates as
independent leaves and places their composition in the already-authorized raw D1 boundary; neither
semantic conformance nor RF1 framing depends upward on append/replay. The later implementation may
change only `exp1-raw-append-replay/Cargo.toml` and the corresponding dependency list in `Cargo.lock`;
those exact dependency-only manifest/lock changes are authorized. The workspace root manifest,
`exp1-record-format`, and `exp1-workload-conformance` must remain unchanged. The external-dependency
allowlist remains empty, and no fourth crate is authorized.

The record uses RF1 encoding version 1, type 3, and `STRUCTURAL-0`; integrity bytes are zero. Encoding and physical validation must call only `exp1-record-format`. No mapping implementation may copy, fork, or locally reproduce RF1 encoding, CRC, scanner, or validation logic.

Before constructing anything, the complete operation must pass `exp1-workload-conformance` SOP1 validation. The mapper then checks all duplicated identities/embedded values, RF1 UUID shape, nonzero sequence/ordinal, strict sequence and physical order, checked lengths, and RF1 resource limits. It calls `exp1-record-format` to encode the `Record`, decodes/validates the complete frame with that crate, and requires exact record equality. Only then may the complete frame be returned.

Missing, malformed, conflicting, unsupported, out-of-order, duplicate, oversized, overflowed, or unrepresentable input fails closed before append. A destination whose last valid ordinal cannot be established, a zero/exhausted ordinal or sequence, a sequence/order conflict, an event-ID disagreement, a SOP1/ENV1 disagreement, or any codec/round-trip disagreement likewise fails. Failure emits no partial record, substitutes no value, filters no reference, renumbers nothing, and does not advance caller-owned state. Existing destination bytes are unchanged.

## 6. Documentation vectors

These vectors are review calculations, not executable evidence. Hex integers in the RF1 header/body are little-endian; embedded SOP1 remains exactly R12/R14's byte order.

### V20-01 ordinary M01/S01

Use the literal 755-byte R14 S01, synthetic assigned sequence 1, and empty-artifact physical ordinal 1. Expected event ID is `330f201a-ea7c-4335-a8ec-e6fe23266a1c`; kind/profile are `03/00`; body values are sequence 1, group 0, index 0, count 1, and core length 755. The frame is 827 bytes. Its first 72 bytes are:

```hex
52444531010003003b0300001b03000001000000000000000000000000000000330f201aea7c4335a8ece6fe23266a1c0100000000000000000000000000000000000100f3020000
```

The complete-frame SHA-256 is `32b63591f5e20ac37e25478d3cdcaca5ad7310be07c32ccdbb3c28bad2c1c9b7`. This digest is a documentation cross-check, not RF1 integrity metadata.

### V20-02 provenance

Use literal R14 S02, synthetic sequence 2, physical ordinal 2. Expected event ID is `c57a25cf-26e6-4dba-ad56-ea7cec2a4865`; ENV1 retains source `source-A` and actor `actor-A`; core length is 770 and frame length 842. The first 72 bytes are:

```hex
52444531010003004a0300002a03000002000000000000000000000000000000c57a25cf26e64dbaad56ea7cec2a4865020000000000000000000000000000000000010002030000
```

The complete-frame SHA-256 is `df4f1358a51683aaf1c8bcd2663c4369d5ed08ae03ac173ae0d6860955e44ff3`.

### V20-03 causal and V20-04 correction/retraction

Generate R12 A01 ordinal 2 exactly, supply sequence 3/physical ordinal 3, and require its ENV1 causal enum `01`, count 1, and sole target equal A01 `EventId(1)`. Generate A01 ordinal 3 exactly, supply sequence 4/physical ordinal 4, and require fact type `correction-A`, correction enum `02`, count 1, and sole target equal A01 `EventId(2)`. In each case the outer event ID equals SOP1/ENV1, the entire SOP1 is the core, and exactly one type-3 structural frame results. A retraction vector changes only the authority-defined operation inputs to the retraction fact type and ENV1 enum `03`; treating it as correction, changing the target, or emitting a second record fails.

### V20-05 boundaries and rejection

P0 maps with a present zero-length SOP1 payload tag and a nonempty SOP1 core. P5 is accepted only when the resulting frame remains within RF1's 16 MiB limit. Sequence `0`, physical ordinal `0`, `u64::MAX + 1`, mismatched outer/inner event IDs, invalid UUID shape, duplicate target bytes, missing applicable provenance, unknown profile/version/kind, nonconsecutive physical ordinal, duplicate/decreasing sequence, truncated SOP1, or a frame over the RF1 limit rejects with no output. A valid SOP1 at sequence `u64::MAX` can be the final mapped operation; any later sequence allocation fails rather than wrapping.

## 7. Future correctness gate and disposition

R20 prospectively authorizes only the later bounded `exp1-raw-append-replay::mapping` implementation and its tests under section 5.1's exact dependency boundary. The gate tests the mapper as a pure operation and does not authorize append/reopen integration. It must reuse `exp1-workload-conformance` and `exp1-record-format`, add no fourth crate or external dependency, and run no workload or benchmark.

The pure mapper can decide reference encoding, semantics/cardinality, UUID shape, duplicated bytes, and all SOP1/ENV1 cross-field agreement from one operation. It cannot decide whether a target belongs to the same stream or to the previously accepted prefix, and therefore cannot distinguish a valid prior target from a future, self, or cross-stream target: `MappingState` deliberately contains only sequence and physical ordinal watermarks. The existing conformance validator likewise validates one SOP1 and rejects duplicate target bytes, but has no membership history. R20 did not freeze additional history or stream state, so this tranche must not invent it. The complete R20 correctness gate remains open; a separately reviewed governance increment must freeze the minimum reference-validation context before those rejection cases can be implemented and tested.

The frozen P5 maximum keeps every authoritative SOP1-derived frame below RF1's 16 MiB limit, so an oversized/resource failure cannot be constructed through the validated public input domain. Likewise, post-validation extraction failures and immediate encode/decode/round-trip disagreement are defensive composition checks, not directly triggerable test branches with the unchanged authority crates. Tests must not claim direct coverage of those unreachable branches; the explicit errors are retained so future authority-crate drift fails closed. Unknown SOP1 profiles, OP1 versions/kinds, ENV1 version/reference kind, malformed/truncated bytes, duplicate reference bytes, and caller sequence/ordinal errors are constructible and require direct negative tests.

The locally decidable independent gate must prove: all five operation cases and P0/P5; literal V20-01/V20-02 lengths, prefixes, digests, decoded equality, and byte-for-byte cores; A01 causal/correction targets; retraction distinction; ordinal/sequence separation including legal gaps; every constructible section 5/6 rejection except future/self/cross-stream membership; no partial output/state advance; and exclusive RF1 encode/validate reuse. Tests may use synthetic mapper inputs but may not describe them as observations.

Passing the locally decidable gate is bounded implementation/correctness-validation evidence only and does not close the complete R20 gate. The live-Linux-capture decision from R19 remains open, so no descriptive D1 harness or execution is authorized. R20 selects no capture crate, API, unsafe FFI policy, privilege/loss behavior, or reduced capture subset.
