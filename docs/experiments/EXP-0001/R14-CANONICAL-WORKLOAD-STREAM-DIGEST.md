# R14 — Canonical Semantic Operation and Workload-Stream Digest Freeze

**Status:** BLK-008 resolved as experiment-local documentation design only
**Authority basis:** `d57945368051d75c604ca628da71c41d90d55061` (R13 merge)
**Profile:** `EXP-0001-SEMANTIC-OP-v1`
**Digest algorithm/profile:** `SHA-256/FIPS-180-4`
**Workload-stream domain:** `rusty-data-os/exp1/workload-stream/v1`
**Vector schema:** `EXP-0001-R14-vector-v1`

## 1. Decision and boundary

R14 completes only the R13-authorized BLK-008 documentation freeze. It preserves R7's SHA-256
selection, lowercase hexadecimal rendering, and `domain || 00 || bytes` separation. It consumes
R12's frozen semantic values without changing their generation. It does not define a physical
event, B1 record, result record, manifest serialization, validator, executable oracle, fixture,
generated workload, or evidence. BLK-009 and every implementation and execution gate remain open.

The words **MUST**, **MUST NOT**, **FAIL**, and **REJECT** are normative for a later, separately
authorized implementation.

## 2. Scalar rules and canonical semantic operation

`EXP-0001-SEMANTIC-OP-v1` uses R12's canonical record construction exactly:

```text
record = magic || field-count:u16be || field...
field  = tag:u8 || length:u32be || value:length-octets
```

Its magic is the eight ASCII octets `RDOS-SOP1` (`52 44 4f 53 2d 53 4f 50`), its field count is
exactly 13 (`000d`), and its fields occur once in strictly increasing tag order:

| Tag | Value |
|---:|---|
| `01` | Complete canonical R12 `OP1` bytes. |
| `02` | Applicable payload profile identifier: exact ASCII `EXP-0001-SHA256-CTR-v1`, `EXP-0001-SHA256-MOTIF-v1`, or `EXP-0001-ZERO-v1`. It MUST agree with OP1 tag `08`. |
| `03` | Exact payload bytes, including zero octets; its `u32be` field length MUST equal OP1 tag `07`. |
| `04` | Exact ASCII `EXP-0001-UUID4-SHA256-v1`. |
| `05` | RequestId, 16 RFC 9562 network-order octets. |
| `06` | EventId, 16 RFC 9562 network-order octets. |
| `07` | InformationId, 16 RFC 9562 network-order octets. |
| `08` | Exact ASCII `EXP-0001-ENVELOPE-INPUT-v1`. |
| `09` | Complete canonical R12 `ENV1` bytes. Its embedded OP1, identities, payload-independent envelope values, logical effective time, and ordered references MUST agree with fields `01`, `05`–`07`, and the operation tuple. |
| `0a` | Exact ASCII `EXP-0001-PRIOR-EVENTS-v1`. |
| `0b` | Exact ASCII `EXP-0001-LOGICAL-TIME-v1`. |
| `0c` | `base_ns`, signed `i64` encoded as its two's-complement eight-octet big-endian bit pattern. |
| `0d` | `unit_ns`, signed positive `i64` in the same representation. |

All text is exact case-sensitive US-ASCII with no terminator, BOM, whitespace, locale processing,
or Unicode normalization. No value is normalized; UUID text is decoded once to its 16 network-
order octets before this construction, and validated canonical generator inputs are encoded as
R12 specifies. Absent/present values remain inside ENV1 as R12's `00` or `01 || bytes`; absent,
empty, and zero are distinct. Signedness is limited to fields `0c`/`0d` and ENV1 effective time;
all lengths/counts and OP1 scalars retain their declared unsigned widths.

A tag, value, identifier, embedded record, or profile mismatch; unknown, missing, repeated, or
out-of-order tag; noncanonical text/input; invalid UUID shape; length disagreement; trailing byte;
overflow; truncation; or value outside R12's P0–P5 and scalar limits MUST fail closed with the
applicable R12 classification or `E-SEMANTIC-OP-ENCODING`. No partial operation is emitted. The
maximum payload is P5 (1,048,576 bytes); consequently each SOP1 field remains within `u32::MAX`.
P0 is represented by tag `03` with length `00000000`, never by omission.

SOP1 is only a digest preimage for one semantic workload operation. It is not a canonical storage
record and makes no statement about adapter or physical encoding.

## 3. Ordered workload-stream bytes

`EXP-0001-WORKLOAD-STREAM-v1` is the following direct byte sequence (not the generic record):

```text
ASCII "RDOS-WS1"                                      # 8 octets
ASCII "EXP-0001-SEMANTIC-OP-v1"                       # 23 octets
operation_count:u64be                                  # 8 octets
warm_up_operation_count:u64be                          # 8 octets
measured_operation_count:u64be                         # 8 octets
repeat operation_count times:
    operation_length:u64be || complete SOP1 bytes
```

The profile identifier has fixed length 23 and no terminator. The 55-octet header is therefore
unambiguous. Counts MUST fit `u64`, their sum MUST equal `operation_count` without overflow, each
length MUST equal the following complete SOP1 length, and the last frame MUST end at end of input.
Truncation, trailing bytes, count overflow, length overflow, or declared length beyond an
implementation's separately declared safe resource limit fails closed; no prefix digest is
accepted as the stream digest.

Operations are in frozen semantic stream order: ascending segment order (all warm-up, then all
measured) and ascending `segment_operation_ordinal` within each segment. Each SOP1's OP1 segment,
ordinal, producer, producer-local order, and optional controlled schedule remain binding. A stream
with uncontrolled cross-producer interleaving does not insert observed assigned sequence or
runtime interleaving; those are later result data. This profile has one warm-up/measured boundary,
fully represented by header counts and each operation's OP1 segment. It adds no run boundary; a
run binds this one stream later through BLK-009 metadata.

An empty stream is the header with all three counts zero and no frames. Warm-up operations are
included, not discarded; measured operations follow them. A failed or rejected candidate emits no
SOP1, frame, or count because R2/R12 define failure rather than a semantic operation. A successfully
generated operation cannot be omitted because of later adapter rejection: the digest identifies
the intended shared semantic workload, not runtime acceptance.

Reordering changes framed byte order; substitution changes a SOP1; omission and duplication change
both counts and frames. Each therefore changes the preimage and is expected to change SHA-256;
parsing/count rules also prevent an unchanged header from concealing omission or duplication.
No filesystem, clock, process, locale, native integer layout, alignment, scheduler, adapter,
runtime hash iteration, assigned sequence, or execution result participates.

The workload digest is exactly:

```text
SHA-256(ASCII "rusty-data-os/exp1/workload-stream/v1" || 00 || workload_stream_bytes)
```

The domain prefix is 38 octets including the final `00`. Output is the 32 digest octets, rendered
externally as exactly 64 lowercase hexadecimal characters. Uppercase, prefixes, separators, or
alternate algorithms/domains are noncanonical metadata representations.

## 4. Documentation vectors

These vectors derive from the first two operations of R12 A01 (`A01-S2`). Both are warm-up,
seed 0, P1/high; operation 0 is minimal/monotonic and operation 1 is provenance/equal-burst.
Every other semantic input, identity, payload, and envelope value follows R12 A01 literally.
Hex below is an unbroken, case-insensitive display of literal octets; whitespace surrounding a
Markdown code block is not part of the bytes.

### 4.1 Per-operation canonical preimages

`S01` (A01 operation 0) is 755 octets:

```hex
52444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e000000010002000000164558502d303030312d5348413235362d4354522d763103000000209a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f04000000184558502d303030312d55554944342d5348413235362d76310500000010cf79754651a34f76b1718244bf8053db0600000010330f201aea7c4335a8ece6fe23266a1c07000000103d3c52813d4347db8825664f324e091d080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000013952444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee0500000001310600000001000700000001000800000010cf79754651a34f76b1718244bf8053db0900000010330f201aea7c4335a8ece6fe23266a1c0a000000103d3c52813d4347db8825664f324e091d0b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a
```

Its independent raw-preimage SHA-256 cross-check (not the R7 workload digest) is
`efa80d1b021e590b8ac02b49a9bb0e68277cf39f32f3849aceabb33e2ec9b83c`.

`S02` (A01 operation 1) is 770 octets:

```hex
52444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e000000010002000000164558502d303030312d5348413235362d4354522d76310300000020701866aac4b5cfd4db8974593a0e4b9db5e5879a12cdda727f27885badb7696404000000184558502d303030312d55554944342d5348413235362d76310500000010fcec18c95b8e4655a5ce5463a7872f850600000010c57a25cf26e64dbaad56ea7cec2a4865070000001080f644cff28f432ab64174dfcc5cf873080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000014852444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee050000000131060000000901736f757263652d410700000008016163746f722d410800000010fcec18c95b8e4655a5ce5463a7872f850900000010c57a25cf26e64dbaad56ea7cec2a48650a0000001080f644cff28f432ab64174dfcc5cf8730b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a
```

Its independent raw-preimage SHA-256 cross-check is
`85a917fe5d4ef24e1904cb6b8ac2554fa60f99ae6f0c69db5e72cf6d81628ddf`.

### 4.2 Empty and two-operation streams

`W00`, the empty stream, is 55 octets:

```hex
52444f532d5753314558502d303030312d53454d414e5449432d4f502d7631000000000000000000000000000000000000000000000000
```

Prefixing the literal 38-octet digest domain and hashing gives
`6ed7e39756dab1b00e5860365288a35b7b8d40f92bc8d219de50eb633144d387`.

`W01`, ordered `[S01,S02]`, is 1596 octets:

```hex
52444f532d5753314558502d303030312d53454d414e5449432d4f502d763100000000000000020000000000000002000000000000000000000000000002f352444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e000000010002000000164558502d303030312d5348413235362d4354522d763103000000209a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f04000000184558502d303030312d55554944342d5348413235362d76310500000010cf79754651a34f76b1718244bf8053db0600000010330f201aea7c4335a8ece6fe23266a1c07000000103d3c52813d4347db8825664f324e091d080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000013952444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee0500000001310600000001000700000001000800000010cf79754651a34f76b1718244bf8053db0900000010330f201aea7c4335a8ece6fe23266a1c0a000000103d3c52813d4347db8825664f324e091d0b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a000000000000030252444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e000000010002000000164558502d303030312d5348413235362d4354522d76310300000020701866aac4b5cfd4db8974593a0e4b9db5e5879a12cdda727f27885badb7696404000000184558502d303030312d55554944342d5348413235362d76310500000010fcec18c95b8e4655a5ce5463a7872f850600000010c57a25cf26e64dbaad56ea7cec2a4865070000001080f644cff28f432ab64174dfcc5cf873080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000014852444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee050000000131060000000901736f757263652d410700000008016163746f722d410800000010fcec18c95b8e4655a5ce5463a7872f850900000010c57a25cf26e64dbaad56ea7cec2a48650a0000001080f644cff28f432ab64174dfcc5cf8730b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a
```

Prefixing the same domain gives the canonical workload-stream digest
`81dbc6b6e33ee775d4b36aeaa0aca45b9649c987f180e378b5d5fbcf1bc3b024`.

### 4.3 Negative and substitution cases

| Case | Exact mutation | Disposition |
|---|---|---|
| N01 | Exchange the two complete W01 frames and their 8-octet lengths, leaving valid counts. | A different preimage; cannot equal W01 byte-for-byte. |
| N02 | Remove S02 and set all three counts to one warm-up/zero measured. | A valid one-operation stream with a different header and body, not W01. |
| N03 | Append a second S02 frame and set operation/warm-up counts to three. | A valid three-operation stream with a different header and body, not W01. |
| N04 | Change S01's payload-profile identifier to `EXP-0001-ZERO-v1` without changing OP1/payload. | Reject `E-SEMANTIC-OP-ENCODING` for profile disagreement; no digest is accepted. |
| N05 | Change digest domain ASCII `.../v1` to `.../v2`, or substitute SHA-512. | Reject as unsupported metadata; it is not an R7 workload-stream digest. |
| N06 | Delete one byte without changing its enclosing length/count. | Reject truncation/length disagreement; no prefix digest is accepted. |

N01–N03 demonstrate that order, omission, and duplication are byte-level distinctions without
assigning those altered streams normative digest values. N04–N06 are rejection vectors and have
no invented successful digest.

### 4.4 Independent calculation provenance and audit

The values were independently calculated during documentation review with Python 3's standard
`hashlib.sha256`: literal hex was decoded to octets, the domain ASCII plus `00` was prepended for
W00/W01, and the resulting byte counts and digests were checked. This method is review provenance,
not a repository script, dependency, fixture, executable oracle, generated artifact, or execution
evidence. Any general FIPS 180-4 SHA-256 implementation can reproduce every expected digest from
the literal bytes above. A reviewer can also use `xxd -r -p | sha256sum` after supplying the
literal domain prefix; repository tooling is not required.

Vector coverage is deliberate: every expected digest above has literal complete preimage bytes;
S01/S02 check canonical operation construction, W00 checks the boundary, and W01 checks framing,
ordering, counts, and the selected R7 domain.

## 5. Metadata retained for BLK-009

A later BLK-009 manifest contract MUST bind, without reinterpreting this freeze:

- semantic operation profile `EXP-0001-SEMANTIC-OP-v1` and stream profile
  `EXP-0001-WORKLOAD-STREAM-v1`;
- digest algorithm/profile `SHA-256/FIPS-180-4`, workload domain
  `rusty-data-os/exp1/workload-stream/v1`, and exactly 32 digest octets / 64 lowercase hex;
- `operation_count`, `warm_up_operation_count`, and `measured_operation_count`;
- workload contract and generator versions from OP1;
- applicable payload profile identifiers, `EXP-0001-UUID4-SHA256-v1`,
  `EXP-0001-ENVELOPE-INPUT-v1`, `EXP-0001-PRIOR-EVENTS-v1`, and
  `EXP-0001-LOGICAL-TIME-v1` needed to interpret the stream;
- the digest value and an immutable reference to the exact semantic-stream bytes if external.

This is a logical metadata obligation only. R14 does not select manifest field names/order,
physical schema, JCS or another serialization, absent/optional representation, external artifact
layout, correction/supersession encoding, validator, or manifest digest construction.

## 6. Compatibility, disposition, and retained gates

A conforming future reader either accepts version 1 byte-for-byte or rejects it. Any change to
magic, profile identifier, field set/order/type, width, signedness, length, count, normalization,
framing, ordering, inclusion, domain, or digest rule requires an explicit successor profile and
new vectors. Published bytes remain immutable; correction requires a successor authority naming
what it supersedes.

BLK-008 is resolved **only as documentation design**: R7 supplies algorithm/domain, this record
uniquely supplies canonical semantic-operation and ordered stream preimages and expected vectors,
and section 5 retains later binding obligations. UNK-018 is resolved at the documentation-design
boundary; UNK-022 remains open for executable/capture validation. BLK-009 and UNK-019 remain open.

No generator implementation, workload generation, manifest serialization/artifact/validator,
Slice C/B1, append/write/fsync/storage, persistence/recovery/durability/fault work, benchmark or
execution, result capture, performance claim, production crate, architecture expansion, source,
test, Cargo, script, fixture, dependency, workflow, toolchain, or workspace change is authorized
or performed by R14. Any next increment requires separate reviewed authorization.
