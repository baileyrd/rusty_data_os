# R12 — Deterministic Generator Specification and Documentation Vectors

**Status:** frozen experiment-local documentation design for BLK-006 and BLK-007; no implementation or execution authorization

**Vector schema:** `EXP-0001-R12-vector-v1`

**Selection date:** 2026-08-30
**Scope:** byte-for-byte payload, workload identity, envelope-input, reference, and logical-effective-time generation only

## 1. Decision and boundary

R12 selects `EXP-0001-SHA256-CTR-v1` for deterministic high-variation payloads,
`EXP-0001-SHA256-MOTIF-v1` for repeated low-variation payloads,
`EXP-0001-ZERO-v1` for zero payloads, and
`EXP-0001-UUID4-SHA256-v1` for deterministic `RequestId`, `EventId`, and workload-supplied
`InformationId` values. All use the canonical input records below. References use
`EXP-0001-PRIOR-EVENTS-v1`; envelope inputs use `EXP-0001-ENVELOPE-INPUT-v1`; logical
effective time uses the four already-frozen EXP-0000 profiles, now identified collectively as
`EXP-0001-LOGICAL-TIME-v1`.

This is a documentation freeze. It does not add a generator, executable fixture, workload,
manifest serialization, validator, digest completion, physical event encoding, or benchmark.
In particular, it does not resolve BLK-008 or BLK-009 and does not authorize Slice C/B1,
persistence, faults, descriptive execution, or confirmatory execution. The byte records in this
document are generator inputs, not the future semantic-stream or manifest serialization.

The words **MUST**, **MUST NOT**, and **FAIL** are normative for a later implementation.

## 2. Selection research

### 2.1 Selected specification sources

SHA-256 means SHA-256 exactly as specified in NIST FIPS 180-4 (August 2015), section 6.2,
including its big-endian message interpretation and 32-octet digest. NIST announced on
2023-03-07 that FIPS 180-4 will be revised after two rounds of public comment; therefore this
freeze names the edition rather than a moving library API. RFC 6234 (May 2011) is the stable
public, cross-language reference for the same SHA-224/SHA-256 operations and includes
implementation and test material. UUID layout, variant, version bits, octet order, and canonical
text follow RFC 9562 (May 2024), sections 4, 5.4, and 6.5. Sources were checked 2026-08-30:

- [NIST FIPS 180-4 landing page](https://csrc.nist.gov/pubs/fips/180-4/upd1/final) and
  [FIPS 180-4 PDF](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf);
- [RFC 6234](https://www.rfc-editor.org/rfc/rfc6234.html);
- [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562.html).

SHA-256 was selected because those stable specifications completely define its octets, padding,
and integer interpretation and implementations are broadly expected in Rust, SQLite-adjacent C,
and RocksDB-adjacent C++ environments. R12 adds no dependency and makes availability an
implementation-gate obligation, not an observed fact. FIPS and RFC text are publicly usable;
an implementation must separately record the license and provenance of any chosen library.

Considered alternatives were SHAKE256/cSHAKE (excellent native expansion and customization,
but a larger less-ubiquitous implementation surface), HMAC/HKDF (unneeded key semantics and
extra construction), ChaCha20/AES-CTR (key/nonce and primitive-profile choices with no security
need), BLAKE3 (well specified and fast but an additional non-FIPS family/dependency), and
language PRNGs such as `StdRng`/Mersenne Twister/xoshiro (library-version or family details can
drift and do not provide the same ubiquitous stable digest reference). Pre-generated artifacts
were rejected because they would be large executable inputs rather than independently
regenerable documentation. Direct truncation of one hash was rejected for payloads longer than
32 octets; UUID versions 5/7 and ULID were rejected because R3 requires UUIDv4-shaped values and
does not permit time/order inference. Performance remains future evidence; no speed advantage is
claimed.

SHA-256 collision and preimage resistance are not needed to call these values random, secret,
authentic, unpredictable, or unique. They provide none of those claims here. Deterministic
UUIDs merely have RFC-compatible version/variant shape. Any collision is a named failure.

## 3. Scalar and canonical-record rules

All text is the exact listed US-ASCII octets (a subset of UTF-8), without normalization,
terminator, BOM, surrounding whitespace, or locale processing. Identifiers are case-sensitive.
Unsigned integers are fixed-width big-endian octets; `u8`, `u16`, `u32`, and `u64` have widths
1, 2, 4, and 8. UUIDs are the 16 octets in RFC 9562 network order, not host-language integer
layout. No native-width, signed coercion, floating point, filesystem, clock, scheduler, or
library-default serialization participates.

The seed at a text boundary MUST be canonical decimal in `[0, 18446744073709551615]`: `0` is
valid; otherwise the first digit is `1`–`9`; only ASCII digits follow. `+0`, `-0`, `00`, `01`,
spaces, separators, non-ASCII digits, empty text, and overflow FAIL `E-SEED-SYNTAX` or
`E-SEED-RANGE`. After validation it is encoded as `u64be`.

Every hash input is a **record**:

```text
record = magic || field-count:u16be || field...
field  = tag:u8 || length:u32be || value:length-octets
```

Fields MUST occur once in strictly increasing tag order. Missing, repeated, unknown, or
out-of-order fields FAIL `E-TUPLE`. Length disagreement or trailing bytes FAIL `E-ENCODING`.
The field count includes every field. Optional values are never omitted: value `00` means absent;
`01 || bytes` means present. Empty and absent therefore differ. Strings longer than `u32::MAX`
or integers outside their declared type FAIL `E-RANGE` before hashing.

### 3.1 Common operation record `OP1`

`magic = 52 44 4f 53 2d 4f 50 31` (`RDOS-OP1`). It contains:

| Tag | Typed value |
|---:|---|
| `01` | workload-contract version, `u16be`, exactly `0001` |
| `02` | generator version, `u16be`, exactly `0001` |
| `03` | segment: warm-up=`00`, measured=`01` |
| `04` | seed, `u64be` |
| `05` | segment operation ordinal, `u64be` |
| `06` | size class P0..P5=`00`..`05` |
| `07` | payload length, `u32be` |
| `08` | content profile: high=`01`, low=`02`, zero=`03` |
| `09` | envelope profile: minimal=`01`, provenance=`02`, causal=`03`, correction/retraction=`04` |
| `0a` | temporal profile: monotonic=`01`, equal-burst=`02`, late=`03`, out-of-order=`04` |
| `0b` | stream namespace, 16 UUID octets; vectors use `00112233-4455-4677-8899-aabbccddeeff` |
| `0c` | producer identity, 16 UUID octets; vectors use `10213243-5465-4768-899a-abbccddeef00` |
| `0d` | producer-local ordinal, `u64be`; MUST equal `05` in the single-producer anchor |
| `0e` | controlled schedule: absent=`00`, or `01 ||` 16 UUID octets |

P0..P5 lengths MUST respectively be 0, 32, 256, 4096, 65536, and 1048576; mismatch FAILS
`E-SIZE-CLASS`. All are legal, P5 is the maximum declared output, and allocation/conversion
must be checked before output. Any requested larger output FAILS `E-RESOURCE-LIMIT`; partial
output is forbidden. The maximum ordinal is `u64::MAX`. Incrementing it FAILS `E-ORDINAL-OVERFLOW`.
Unsupported enum/version values FAIL `E-UNSUPPORTED`.

## 4. Payload profiles (BLK-006)

The payload domain record `PAY1` has magic `RDOS-PAY1`, fields `01 = OP1 bytes` and
`02 = block-index:u32be`. For high variation, block `j` is `SHA-256(PAY1(OP1,j))`, beginning at
`j=0`; concatenate blocks and retain exactly the first payload-length octets. The maximum P5
uses 32,768 complete blocks, indexes 0..32767. No digest state, counter, or unused suffix carries
between events. P0 validates the entire OP1 tuple and returns empty bytes without hashing a
block. Counter exhaustion before the requested length FAILS `E-COUNTER-OVERFLOW`.

For low variation, compute exactly one motif as `SHA-256(PAY1(OP1,0))[0..8]`, eight octets.
Repeat that motif from its first octet and truncate only the final repetition to the requested
length. Thus events vary deterministically, while each nonempty event has a precisely repeated
eight-byte pattern. P0 again returns empty. Substituting a constant motif or the high-profile
stream FAILS `E-PROFILE-SUBSTITUTION`.

For all-zero, after full OP1 validation return exactly payload-length `00` octets. It performs no
hash operation and is diagnostic only. Any nonzero byte or output-length truncation FAILS
`E-PROFILE-SUBSTITUTION` or `E-OUTPUT-LENGTH`.

OP1 includes segment, profile, stream/producer domains, and ordinal: warm-up/measured, adjacent
events, streams, producers, and profiles cannot share a hash preimage. A later implementation
must compare actual length to declared length before use and must never silently truncate because
of allocation, adapter, or baseline limits.

## 5. Identity, envelope, and references (BLK-007)

### 5.1 Deterministic identities

`ID1` is a record with magic `RDOS-ID1` and fields: `01=OP1 bytes`; `02=kind` where request=`01`,
event=`02`, information=`03`; `03=namespace version u16be (0001)`; and `04=namespace UUID`.
The frozen namespace UUIDs are respectively
`a1111111-1111-4111-8111-111111111111`,
`b2222222-2222-4222-8222-222222222222`, and
`c3333333-3333-4333-8333-333333333333`. Equal namespace UUIDs or a kind/namespace mismatch
FAILS `E-NAMESPACE`.

Compute `d = SHA-256(ID1)`, take `d[0..16]`, replace octet 6 with
`(octet6 & 0x0f) | 0x40`, and replace octet 8 with `(octet8 & 0x3f) | 0x80` (zero-based octets).
The result is stored as the nominal requested type and rendered as lowercase `8-4-4-4-12`
hexadecimal. No other bits change. Nil, bad variant/version, collision, or collision with unequal
binding FAILS `E-IDENTITY-SHAPE` or `E-IDENTITY-COLLISION`. Collision detection is over all
generated values in the applicable typed namespace and stream before execution. There is no
retry, salt, remap, alternate namespace, replacement, or silent reconciliation. Expected reuse
of one `InformationId` is possible only when the explicit OP1 identity-binding input is identical;
the workload must supply that binding rather than infer it from payload bytes.

Attempt IDs are deliberately absent: R3 assigns a fresh live attempt identity at the lifecycle
boundary, not from the shared workload stream. References contain an earlier `EventId`; they do
not generate a separate reference identity.

### 5.2 Explicit envelope-input record

`ENV1` (magic `RDOS-ENV1`) contains, in order: `01=OP1`; `02=envelope semantic version ASCII`;
`03=fact/event type ASCII`; `04=schema identity UUID`; `05=schema version ASCII`;
`06=source provenance option`; `07=actor provenance option`; `08=RequestId`; `09=EventId`;
`0a=InformationId`; `0b=logical effective-time i64 encoded as its two's-complement 8-octet
big-endian bit pattern`; `0c=reference semantics enum (none=00, causal=01, correction=02,
retraction=03)`; and `0d=ordered EventId list encoded as count:u32be || 16-octet items`.

The profile determines only applicability: minimal requires absent provenance/no references;
provenance requires both explicit nonempty provenance values; causal requires causal semantics
and at least one target; correction/retraction requires the matching explicit fact type and
semantics. Inconsistent combinations FAIL `E-ENVELOPE-PROFILE`. No default fact type, schema,
version, provenance, producer, reference meaning, or time parameter may be inferred. ENV1 does
not contain sequence, commit, system-acceptance, durability, observation, retry outcome, or gap
policy; a generator that invents one FAILS `E-FABRICATED-LIFECYCLE`.

### 5.3 Reference selection

The reference namespace is
`d4444444-4444-4444-8444-444444444444`, version 1. It is distinct even though the selected
algorithm returns existing EventIds. `EXP-0001-PRIOR-EVENTS-v1` takes explicit stream namespace,
current ordinal `i`, semantics, and cardinality `k`. The candidate prefix is the validated,
ordinary-event list at ordinals `[0,i)`, excluding correction/retraction events. Require `k>0`
and `k<=i`; select ordinals `i-k, ..., i-1` and emit their already-generated EventIds in ascending
ordinal order. Thus the first valid target is ordinary event 0 for `i=1,k=1`; `i=4,k=3` selects
1,2,3. Cardinality/order are not adapter choices.

Before acceptance every target MUST exist in the same stream, have lower operation ordinal, be
an `EventId`, be ordinary where required, occur only once, and not equal the current EventId.
Violations respectively FAIL `E-REFERENCE-MISSING`, `E-REFERENCE-FUTURE`,
`E-REFERENCE-WRONG-KIND`, `E-REFERENCE-WRONG-FACT`, `E-REFERENCE-DUPLICATE`,
`E-REFERENCE-SELF`, or `E-REFERENCE-CROSS-STREAM`. No filtering or replacement is allowed.

## 6. Logical effective time

`EXP-0001-LOGICAL-TIME-v1` accepts OP1 plus `base_ns:i64` and `unit_ns:i64`; `unit_ns` MUST be
positive. Arithmetic is checked in mathematical integers then required to fit `i64`:

- monotonic: `E(i)=base_ns + i*unit_ns`;
- equal-burst: `E(i)=base_ns + floor(i/100)*unit_ns`;
- late-arriving: normally `base_ns+i*unit_ns`; where `i mod 10=9`, subtract `100*unit_ns`;
- out-of-order: let `q=floor(i/4)`, `r=i mod 4`, `p=[0,2,1,3]`; then
  `E(i)=base_ns+(4*q+p[r])*unit_ns`.

Overflow FAILS `E-LOGICAL-TIME-OVERFLOW`; a nonpositive unit FAILS `E-LOGICAL-TIME-PARAMETER`.
These signed nanoseconds are caller/domain effective-time inputs consistent with R3. They do not
predict actual system time. A late-arriving execution is valid only if lifecycle evidence also
shows the selected effective time precedes actual system acceptance. Replay remains assigned
sequence order: for out-of-order ordinals 0..3, replay is `0,1,2,3` while effective ranks are
`0,2,1,3`.

## 7. Documentation vectors

All successful vectors use vector schema `EXP-0001-R12-vector-v1`, algorithms above, stream and
producer UUIDs from section 3.1, seed 0, warm-up, ordinal 0, P1/high/minimal/monotonic unless a
cell overrides it. Provenance is this R12 selection and the cited standards. A correction creates
a new vector schema/version naming the superseded vector; published values are never edited in
place.

The independent check procedure is: encode each length literally in big-endian, concatenate the
displayed fields, inspect field order/count and OP1 length, run any FIPS 180-4 SHA-256
implementation, apply only the stated UUID masks, and compare octets before formatting. A second
implementation can stream P3–P5 and compare the listed construction/digest without retaining a
fixture.

### 7.1 Normalization, limits, and payload coverage

| Vector | Exact input/change | Expected result |
|---|---|---|
| N01 | seed text `0` | `0000000000000000` |
| N02 | seed text `18446744073709551615` | `ffffffffffffffff` |
| N03 | `+0`, `-0`, `00`, `01`, space, empty | `E-SEED-SYNTAX` |
| N04 | `18446744073709551616` | `E-SEED-RANGE` |
| N05 | ordinal 0 then 1 | `0000000000000000`, `0000000000000001`; distinct OP1 |
| N06 | version 2 or enum 255 | `E-UNSUPPORTED` |
| N07 | missing field, P1/31, producer-local mismatch | `E-TUPLE`, `E-SIZE-CLASS`, `E-TUPLE` |
| N08 | ordinal max then increment | max encodes `ffffffffffffffff`; increment is `E-ORDINAL-OVERFLOW` |
| N09 | requested length 1048577 | `E-RESOURCE-LIMIT`, no partial bytes |
| P00 | P0 under each content profile | exactly empty (zero octets), after tuple validation |
| P01 | P1 | exactly 32 octets: high block 0; low motif repeated 4 times; or 32 zeroes |
| P02 | P2 | exactly 256 octets: 8 high blocks; low motif repeated 32 times; or zeroes |
| P03 | P3 | exactly 4096 octets: 128 high blocks; 512 motifs; or zeroes |
| P04 | P4 | exactly 65536 octets: 2048 high blocks; 8192 motifs; or zeroes |
| P05 | P5 maximum | exactly 1048576 octets: 32768 high blocks; 131072 motifs; or zeroes |
| P06 | same tuple measured rather than warm-up | segment byte changes `00` to `01`; first hash preimage differs |
| P07 | ordinal 1 rather than 0 | ordinal field differs; first hash preimage differs |
| P08 | change high/low/zero profile | profile byte differs; substitution is `E-PROFILE-SUBSTITUTION` |
| P09 | adapter returns fewer than declared octets | `E-OUTPUT-LENGTH`; never accepted as truncation |

P00–P05 are exact byte specifications because every octet is the concatenation/repetition/zero
rule in section 4, not a sampled expectation. The P5 row is independently documentable without
allocating or storing an artifact.

The exact P1 anchor outputs (one unbroken hexadecimal octet string each) are:

| Vector | OP1 SHA-256 cross-check | Exact 32 payload octets |
|---|---|---|
| P10 high | `b1566ba60808c4bab28f3a471eaed299b5cde2a7784e99dd075c7f272fc95dec` | `9a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f` |
| P11 low | `64041e3c55a066c4e19332de8325dfd48ead70dcd939fbbba7852d53eebd3712` | `3802fbfd223852b13802fbfd223852b13802fbfd223852b13802fbfd223852b1` |
| P12 zero | `9280324e264ca1b3b5cc8ac5948e022590e2139f978d8197b005882434907ae7` | `0000000000000000000000000000000000000000000000000000000000000000` |

Each OP1 is exactly 150 octets. P10's payload is SHA-256 of the 173-octet PAY1 record
(`RDOS-PAY1`, field count 2, the 150-octet OP1, and counter `00000000`). P11 uses its own OP1
and repeats the first eight digest octets `3802fbfd223852b1` four times. This gives a direct
length/count/hash hand-check rather than relying on prose alone.

### 7.2 Identity, envelope, reference, and collision vectors

| Vector | Typed input | Exact expected output |
|---|---|---|
| I01 | request/event/information kinds for one OP1 | request=`cf797546-51a3-4f76-b171-8244bf8053db`; event=`330f201a-ea7c-4335-a8ec-e6fe23266a1c`; information=`3d3c5281-3d43-47db-8825-664f324e091d` |
| I02 | reuse request namespace for event | `E-NAMESPACE` |
| I03 | force a registry containing the computed EventId bound to unequal OP1 | `E-IDENTITY-COLLISION`; no retry/remap |
| E01 | minimal, explicit versions/types/schema, absent provenance, no refs | valid ENV1 |
| E02 | provenance with source=`source-A`, actor=`actor-A` | valid ENV1; either absent is `E-ENVELOPE-PROFILE` |
| E03 | causal, ordinary prefix `[event0]`, `i=1,k=1` | target list exactly `[EventId(0)]` |
| E04 | correction then retraction fact types, `i=1,k=1` | matching semantics and exactly `[EventId(0)]` |
| R01 | ordinary prefix 0..3, `i=4,k=3` | targets `[EventId(1), EventId(2), EventId(3)]` in that order |
| R02 | self / future / missing target | `E-REFERENCE-SELF` / `E-REFERENCE-FUTURE` / `E-REFERENCE-MISSING` |
| R03 | target from another stream / RequestId / correction event | `E-REFERENCE-CROSS-STREAM` / `E-REFERENCE-WRONG-KIND` / `E-REFERENCE-WRONG-FACT` |
| R04 | duplicate target | `E-REFERENCE-DUPLICATE` |

The four envelope rows cover every frozen envelope profile. Information identity is workload
supplied; source/actor values and schema/fact/version values enter ENV1 literally. Producer UUID
and local ordinal enter OP1 and therefore every derived identity. The reference namespace never
substitutes for the EventId namespace or changes a target.

For I01 each ID1 is exactly 199 octets. Its pre-mask SHA-256 value is respectively
`cf79754651a38f7671718244bf8053dbd4360155c6b552d2c4ee0ff3cac7a322`,
`330f201aea7c833568ece6fe23266a1c3c222f9fddd51c1d20e9061c09994571`, and
`3d3c52813d43b7db8825664f324e091d78fea6e87c5318f0e4ad14fb6593e1c1`.
The exact post-mask octets are `cf79754651a34f76b1718244bf8053db`,
`330f201aea7c4335a8ece6fe23266a1c`, and `3d3c52813d4347db8825664f324e091d`.

### 7.3 Logical-time vectors

With `base_ns=1000`, `unit_ns=10`:

| Vector | Profile and ordinals | Expected `E(i)`; replay order |
|---|---|---|
| T01 | monotonic, 0..3 | `1000,1010,1020,1030`; `0,1,2,3` |
| T02 | equal-burst, 98..101 | `1000,1000,1010,1010`; ordinal/replay remains `98,99,100,101` |
| T03 | late, 8..10 | `1080,90,1100`; ordinal/replay remains `8,9,10` |
| T04 | out-of-order, 0..3 | `1000,1020,1010,1030`; ordinal/replay remains `0,1,2,3` |
| T05 | `base=i64::MAX`, ordinal 1 | `E-LOGICAL-TIME-OVERFLOW` |

T03 specifies the logical input only; it does not assert late arrival until actual lifecycle
evidence establishes the required comparison. T04 is the explicit proof that effective time does
not alter replay order.

### 7.4 Cross-implementation semantic anchor

Anchor `A01` is the compact sequence of four warm-up operations with seed 0, stream/producer
UUIDs above, producer-local ordinals equal to operation ordinals, no controlled schedule,
P1/high, envelope profiles respectively minimal, provenance, causal, correction, temporal
profiles respectively monotonic, equal-burst, out-of-order, late, and `base_ns=1000/unit_ns=10`.
All use envelope semantic version ASCII `1`, schema UUID
`eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee`, schema version `1`, ordinary fact type `fact-A` except
ordinal 3 uses `correction-A`; ordinal 1 uses source `source-A` and actor `actor-A`; ordinal 2
selects `[EventId(1)]`; ordinal 3 selects `[EventId(2)]`. Each operation's exact downstream
semantic input is:

```text
OP1 || payload bytes || RequestId octets || EventId octets || InformationId octets ||
effective-time:i64be || reference-count:u32be || ordered reference EventId octets || ENV1
```

Payload and identity octets are exactly derived by sections 4 and 5, and effective times are
`1000,1000,1020,-870` (`-870` is two's-complement `fffffffffffffc9a`). This immutable formula,
field order, and all typed values are the documentation-level bytes later BLK-008/009 work may
consume. It does **not** select a stream-record framing, digest the sequence, finalize manifest
serialization, or claim that a semantic-stream digest is complete.

## 8. Compatibility, correction, and disposition

An implementation MUST match version 1 byte-for-byte or reject it. Any change to magic, field
set/order/type, enum, namespace, expansion, mask, selection, or time rule requires a new version
and new vectors; it cannot be called version 1. Readers may support several explicit versions but
must not negotiate by guessing. Errata add a successor record that names this document and the
superseded vector; historical results retain the original authority and bytes.

On merge, BLK-006 and BLK-007 are resolved **as documentation design only**, and UNK-018 is
narrowed accordingly. Generator implementation, exhaustive executable conformance, dependency
selection, cost, and all observations remain absent. BLK-008 still awaits its exact semantic
stream byte input over these values; BLK-009 still awaits canonical manifest serialization and a
validator. UNK-019 and the executable/capture portions of UNK-022 remain open. Nothing in R12
authorizes workload generation, Slice C/B1, persistence, durability, fault work, benchmarking,
or execution.
