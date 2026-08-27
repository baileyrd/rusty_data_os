# R5 Physical Record, Integrity, and Recoverable-Commit Contract

**Profile identity:** `EXP1-B1-RF1` / encoding version 1
**Integrity profiles:** `STRUCTURAL-0` and `CRC32C-1`
**Status:** frozen documentation design; not implemented, executed, or empirically validated

## 1. Decision and evidence boundary

This experiment-local contract resolves BLK-001 and BLK-003 together and completes the B1 D2/controlled-D3 design mapping required by R1 and R3. It is a design decision supported by the reviewed R1/R3 invariants and the standard, independently specified CRC-32C algorithm; it is not correctness, durability, power-loss-survival, or performance evidence. Canonical history remains the sole authority. Binding, reservation, provisional, and group records are lifecycle evidence only; only a valid final event selected by one valid commit record is canonical.

Alternatives rejected were in-place finalization (violates immutable finalized records and makes torn updates ambiguous), file-presence/rename as commit (adds namespace state and synchronization), one pre-finalization `fsync` (cannot preserve the later durability sample), structural-only canonical records (violates R1), and CRC-32/ISO-HDLC or a cryptographic digest (respectively weaker fit for common storage-error detection, and unnecessary authenticity/cost semantics). CRC-32C was selected because its public Castagnoli parameters are stable, broadly independently implementable, and appropriate only as accidental-error detection. It provides no authenticity, malicious-tampering resistance, encryption, authorization, stable-media, PLP, atomic-sector, or power-loss-survival guarantee.

## 2. Common frame (`EXP1-B1-RF1`)

Every integer is unsigned little-endian unless explicitly signed. Records are concatenated with no alignment, padding, segment header, or implicit bytes. Offsets are from the record start.

| Offset | Width | Field | Rule |
|---:|---:|---|---|
| 0 | 4 | magic | ASCII `RDE1` (`52 44 45 31`) |
| 4 | 2 | encoding version | exactly `1` |
| 6 | 1 | record type | `1` binding, `2` reservation, `3` provisional event, `4` D3 membership, `5` final event, `6` commit |
| 7 | 1 | integrity profile | `0` structural or `1` CRC-32C |
| 8 | 4 | total length | header plus body; `[32, 16,777,216]` |
| 12 | 4 | body length | exactly `total_length - 32` |
| 16 | 8 | physical ordinal | starts at 1 and strictly increases by one for every record, canonical or not |
| 24 | 4 | reserved | zero; nonzero is malformed |
| 28 | 4 | integrity | little-endian CRC-32C for profile 1; zero for profile 0 |
| 32 | variable | body | exactly `body_length` octets |

A scanner reads only the 32-byte header before validating lengths. Per scan it accepts at most 1,000,000 records, 16 MiB per record, 1 GiB cumulative bytes, and 64 MiB cumulative retained diagnostic state. Reaching a limit stops explicitly. Checked 64-bit addition must prove `offset + total_length` is within the artifact. A valid extent advances exactly by `total_length`. Unknown magic, version, type, or profile fails closed; there is no resynchronization search or downgrade. EOF at a record boundary is clean. Fewer than 32 terminal bytes, or a valid header whose declared extent crosses EOF, is terminal truncation after a valid prefix. The same condition before known later bytes, any impossible length, or any ambiguity is interior damage and stops canonical replay.

Profile 0 is eligible only for types 1, 3, and 4 in provisional/diagnostic D0/D1 use and its integrity field must be zero. Types 2, 5, and 6 always use profile 1. D2/D3 recovery rejects profile-0 evidence as proof of canonicality.

## 3. Bodies and semantic preservation

All UUID fields are the 16 RFC-ordered octets of their R3 nominal type. Length-prefixed byte strings preserve their bytes exactly; scanners neither normalize nor interpret opaque content.

| Type | Body, in exact order | Constraints and role |
|---|---|---|
| 1 binding | `request_id[16], event_id[16], normalized_request_length:u32, normalized_request[...]` | Complete versioned normalized-request serialization is retained byte-for-byte. Empty is legal only in documentation vectors; executable use remains gated by BLK-007/009. Conflicting bytes for one request fail closed. |
| 2 reservation | `request_id[16], event_id[16], sequence:u64, high_water:u64` | Both nonzero; `sequence <= high_water`; establishes binding and consumed sequence. Later reservation records must not decrease high-water. |
| 3 provisional | `event_id[16], sequence:u64, group_id:u64, member_index:u16, member_count:u16, core_length:u32, stable_core[...]` | Stable pre-durability envelope core plus opaque payload bytes. D2 uses group 0, index 0, count 1. D3 uses nonzero group, zero-based unique index, fixed nonzero count. Never canonical. |
| 4 D3 membership | `group_id:u64, member_count:u16, reserved:u16, repeated(event_id[16], sequence:u64)` | Nonzero group/count, reserved zero, exact length `12 + 24*count`, unique events/sequences, increasing sequence order. Freezes membership before the shared boundary. |
| 5 final event | `event_id[16], request_id[16], sequence:u64, durability_time:i64, envelope_length:u32, complete_envelope[...]` | The byte string is the complete semantic-envelope serialization including the same event/request/sequence/durability values, all source descriptors, references, and opaque payload. An implementation must byte-validate equality under the later envelope serialization authority; until that authority exists these vectors freeze physical containment, not BLK-007/009. |
| 6 commit | `event_id[16], sequence:u64, final_ordinal:u64, final_crc32c:u32, group_id:u64, member_index:u16, member_count:u16` | Selects exactly one immediately preceding unmatched type-5 record with matching identity/sequence/CRC. D2 group fields are 0/0/1. D3 fields match type 4. A final event without this record is noncanonical residue. |

Body-length arithmetic is checked before access. Duplicate bindings, conflicting event/request relations, reused/decreasing sequences, sequence above recovered high-water, duplicate final or commit records, commit-before-final, nonadjacent/mismatched final references, D3 membership conflicts, and canonical commit order that is not strictly increasing all fail closed. A legal reservation without a matching commit is a reported gap. Lifecycle records never supply missing canonical-envelope fields and never override a committed event.

## 4. Integrity profile `CRC32C-1`

Parameters are CRC-32C/Castagnoli: width 32, polynomial `0x1EDC6F41` (reflected `0x82F63B78`), initial register `0xffffffff`, input and output reflected, final XOR `0xffffffff`, no augmentation, check value for ASCII `123456789` = `0xe3069283`. The stored integer is little-endian.

Coverage is every byte from offset 0 through `total_length - 1`, including framing, profile, ordinal, body, complete envelope, and payload, with bytes 28–31 normalized to four zero octets during calculation. Nothing else is excluded. Validation first performs bounded structural checks, then recomputes and compares. CRC failure excludes the record and stops canonical scanning. This profile detects all single-bit changes and the standard CRC-32C bounded burst/error classes, but collision probability and correlated faults prevent any absolute corruption-detection claim.

## 5. Exact append, synchronization, visibility, retry, and recovery sequence

The one B1 file is opened and appended as R5 already specifies; no finalized record is mutated. Creation requires `fsync(data_fd)` followed by `fsync(parent_dir_fd)` before any dependent boundary can succeed. With an existing, already namespace-durable file, phases below require only the stated data-file synchronization. No rename, link, rotation, replacement, or deletion is permitted during a run.

1. Append type 1, then `fsync(data_fd)` (and parent after first creation). This durably binds the request, event, and full normalized request before reservation.
2. Append type 2, then `fsync(data_fd)`. Only success exposes the reservation; uncertainty stops assignment for recovery. High-water never decreases and reserved sequences are never reused.
3. Append type 3. For D3, append exactly one type 4 after every member's type 3. Complete all writes, then call the declared pre-finalization `fsync(data_fd)` once (shared by the frozen D3 membership). Failure yields no durability time or canonical member.
4. Immediately after successful return, sample the OS realtime clock exactly once per event, serially in membership order. A sample is never copied between members. Sampling failure abandons that member; controlled D3 is not an atomic transaction.
5. For each successfully sampled event in increasing sequence order, construct an immutable type 5 then immediately append its type 6. After all selected pairs are complete, call `fsync(data_fd)` once. This post-finalization call is the recoverable-commit establishment boundary. On its success, each independently valid pair becomes canonical; on error/uncertainty, expose and acknowledge none until recovery decides each pair.
6. Only after step 5 success may canonical readers see an event and may its acknowledgement be attempted. D3 shares physical boundaries and retains membership/outcome evidence, but each pair is an individual commit, never an atomic multi-event transaction. Formation wait remains measured per the existing contract.

Retry reuses the same binding, event, and reserved sequence. It first scans: an existing valid commit returns the same event; binding/reservation without commit resumes the exact candidate only when equality is provable; otherwise it fails closed. A lost acknowledgement after step 5 returns the recovered committed event without appending another. Restart sets the allocator above the greatest valid reservation high-water even when gaps exist.

Recovery scans from byte zero. It validates physical ordinals and all records until clean EOF, conclusive terminal truncation, or a fail-closed condition. It builds lifecycle evidence, then emits only type-5/type-6 pairs satisfying all binding, reservation, high-water, order, profile, CRC, D3, and immediate-adjacency rules. Valid provisional/final residue is reported and excluded. A valid prefix may be replayed before conclusive terminal truncation; interior/ambiguous damage, integrity failure, unsupported identity, duplicate/order conflict, or undecidable canonical status stops canonical replay at the last validated prefix and reports failure. Checksum validity alone never establishes commit or durability.

## 6. Stable documentation vectors

Hex is one uninterrupted record; spaces shown in explanatory mutations are not bytes. All CRCs were independently recomputed using the parameters in section 4.

| Vector | Bytes / mutation | Expected disposition |
|---|---|---|
| V1 minimum legal structural binding | `5244453101000100440000002400000001000000000000000000000000000000101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f00000000` | Valid provisional type 1, 68 bytes; never canonical. |
| V2 nontrivial provisional payload `DATA` | `52444531010003004c0000002c00000003000000000000000000000000000000000102030405460788090a0b0c0d0e0f07000000000000000000000000000000000001000400000044415441` | Valid 76-byte type 3; payload/core preserved; never canonical. |
| V3 reservation/high-water | `5244453101000201500000003000000002000000000000000000000041f0e427101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f07000000000000000700000000000000` | Valid 80-byte lifecycle record; CRC `27e4f041` displayed little-endian as `41f0e427`; consumes sequence 7 but is not a fact. |
| V4 final event with envelope `DATA` | `524445310100050158000000380000000500000000000000000000008a1267a2000102030405460788090a0b0c0d0e0f101112131415461798191a1b1c1d1e1f070000000000000015cd5b07000000000400000044415441` | Valid 88-byte type 5, durability time 123456789; CRC `a267128a`; noncanonical without a matching adjacent commit. |
| V5 terminal truncation | V4 with its final byte removed and artifact ending there | Report terminal truncation; replay only any earlier eligible prefix; exclude V4. |
| V6 malformed length | V4 with bytes 8–11 changed to `1f000000` | Total length below 32: malformed length, stop before allocation. |
| V7 unsupported version/profile | V4 with bytes 4–5 changed to `0200`, or byte 7 to `02` | Unsupported, fail closed without CRC fallback or downgrade. |
| V8 covered corruption | V4 with final byte `41` changed to `40` | CRC mismatch, exclude and stop. Independent recomputation over the mutation gives a value other than stored `a267128a`. |
| V9 duplicate/order conflict | V3 followed by an otherwise valid reservation for a different event at sequence 7, or by ordinal 2 again | Fail closed; no winner and no renumbering. |
| V10 ambiguous/interior damage | V3 + first 40 bytes of V4 + any later bytes/magic | Interior/ambiguous truncation; do not search for or replay later records. |
| V11 controlled-D3 representation | Type 4 fixes nonzero group, ordered members and count; each type-6 repeats group/index/count and points to its adjacent V4-shaped type 5 | Membership mismatch, missing member outcome, copied durability sample, or shared-boundary failure is reported; no atomic-group claim and only independently valid committed pairs are eligible. |

To reproduce a CRC: decode hex, replace byte offsets 28–31 with zero, run the stated reflected CRC-32C from `0xffffffff`, XOR the result with `0xffffffff`, and encode the 32-bit result little-endian. V3 and V4 deliberately cover different body shapes. These are documentation artifacts, not executable fixtures or validators.

## 7. Limits, traceability, and continuation

This contract resolves BLK-001 and BLK-003 as documentation design. It resolves BLK-016 for the B0 design profile and BLK-017 for the B1 design profile; implementation/evidence remain absent. It completes only the B0/B1 portion of BLK-019; B2/B3 remain R6 work. BLK-015 remains open: final paths, exact protection, evidence capture, and empirical survival are not established. UNK-001 and the physical/profile portion of UNK-012 are resolved for EXP-0001 B1; UNK-015 remains open for executable fault mechanisms; UNK-020/021 are narrowed only for B0/B1 design.

The decisions trace to REQ-001–010, REQ-013/014, RQ-003, ADR-0002, every EXP-0000 semantic/durability/recovery/baseline/workload/environment/result/interpretation/methodology contract, and R1–R5. The complete R1 scenario and R3 lifecycle mappings are represented in sections 2–6. R5 is complete as documentation design and R6 is the next documentation-only increment. This does not authorize R6 content, code, Cargo, workflows, executable fixtures, implementation, machine changes, capture, fault execution, benchmarks, or any durability/performance conclusion.
