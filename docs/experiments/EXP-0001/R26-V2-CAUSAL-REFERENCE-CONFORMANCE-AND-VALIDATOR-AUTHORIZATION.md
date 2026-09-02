# EXP-0001 R26 — v2 causal-reference conformance and validator authorization

**Contract:** `EXP-0001-CAUSAL-REFERENCE-CONFORMANCE-v2`  
**Status:** frozen documentation design and prospective bounded implementation authorization  
**Evidence classification:** governance/conformance design only; not implementation, correctness,
workload, benchmark, durability, recovery, or performance evidence  
**Decision date:** 2026-09-02

## 1. Question, decision, and boundary

R26 asks whether the R25 bootstrap correction can be made completely byte-decidable without
changing any v1 authority. The answer is yes. This authority freezes the v2 profile family,
relationships, binary encodings, digest domains, manifest ledger, validation precedence, and
literal-vector oracle requirements below. It then authorizes one later, documentation-derived
conformance implementation in the existing `exp1-workload-conformance` crate.

R26 does not contain or claim that implementation. It does not authorize the separately gated
R21–R23 reference-context implementation, close R20, or authorize workload materialization or
execution.

## 2. Closed v2 profile family and compatibility

The complete new family is:

| Layer | Exact identifier |
|---|---|
| workload contract | `EXP-0000-WORKLOADS-v2-causal-reference` |
| envelope semantic profile | `envelope-causal-reference-v2` |
| envelope generator/encoding | `EXP-0001-ENVELOPE-INPUT-v2` |
| reference generator/encoding | `EXP-0001-PRIOR-EVENTS-v2` |
| semantic operation | `EXP-0001-SEMANTIC-OP-v2` |
| workload stream | `EXP-0001-WORKLOAD-STREAM-v2` |
| workload-stream digest | `EXP-0001-WORKLOAD-STREAM-DIGEST-v2` |
| workload manifest | `EXP-0001-WORKLOAD-MANIFEST-JCS-v2` |
| workload-manifest digest | `EXP-0001-WORKLOAD-MANIFEST-DIGEST-v2` |

A v2 manifest uses all nine values together. The v1 identity, payload, logical-time,
size-class-order, temporal, and SHA-256 algorithm profile identifiers may remain literal inputs
because their algorithms and bytes are unchanged; this is reuse, not v1/v2 structural mixing.
Every version-sensitive value in the table MUST be v2. Any v1 value at one of those positions in a
v2 manifest or stream, any v2 value in a v1 object, an unknown value, alias, omitted version,
fallback, or attempted negotiation is `profile-mismatch` before count, reference, or digest
validation. A validator never upgrades, downgrades, or reinterprets bytes.

## 3. Exact semantic and containment relationships

One v2 semantic operation (`SOP2`) contains exactly one R12 `OP1`, its unchanged payload bytes and
profiles, one v2 envelope input (`ENV2`), and the unchanged logical-time outputs. `ENV2` embeds that
same OP1 byte-for-byte and contains exactly one v2 prior-events value (`REF2`). `REF2` is an ordered
list of ordinary EventId target bytes; target order is semantic. The OP1 identity triple and the
event/request/information identities embedded in ENV2 MUST equal the SOP2 values exactly.

For each `(stream_namespace, segment)`, segment ordinal zero has `REF2` count zero. Every ordinal
greater than zero has exactly the policy's positive `subsequent` count and each target is a distinct
prior ordinary EventId from `[0, ordinal)` in that same namespace and segment. The unchanged R12
selection algorithm is applied with that effective count. Warm-up and measured each start at zero;
total WS2 position never expands eligibility.

One `WS2` is an ordered sequence of complete SOP2 frames for exactly one namespace. It preserves
R14 ordering and segment/count invariants, except that its magic/profile bindings are v2. One v2
manifest binds exactly one WS2 through namespace, counts, all profiles, stream byte length,
exact-artifact SHA-256, and the v2 domain-separated stream digest. An R23 closed-scope member may
bind a v2 manifest and WS2 only after a later authority versions R23's accepted manifest/stream
profile set; R26 deliberately does not amend the R23 descriptor or authorize that implementation.

## 4. Canonical binary encoding and digest domains

All integers retain R12/R14's checked, unsigned big-endian fixed-width representation; strings are
UTF-8 preceded by a big-endian `u32` octet length, lists by a big-endian `u32` count, and UUIDs are
raw 16 octets. No native layout, padding, terminator, normalization, locale, or alternate encoding
is permitted. Existing OP1, payload, identity, and logical-time sub-encodings are copied unchanged.

The versioned encodings are obtained from the corresponding v1 grammar in R12/R14 by exactly these
substitutions and no others:

* ENV2 magic is ASCII `RDOS-ENV2`, and its embedded reference-profile string and REF2 bytes name
  `EXP-0001-PRIOR-EVENTS-v2`;
* REF2 is `u32 target_count` followed by exactly that many 16-octet targets;
* SOP2 magic is ASCII `RDOS-SOP2`, its semantic-profile binding is
  `EXP-0001-SEMANTIC-OP-v2`, and its envelope-profile string/bytes are ENV2;
* WS2 magic is ASCII `RDOS-WS2`, followed directly by the fixed 23-octet ASCII semantic profile
  `EXP-0001-SEMANTIC-OP-v2`, then R14's unchanged three `u64` counts and ordered
  `u64 byte_length || SOP2_bytes` frames.

Lengths count octets, include no length field itself, and must be minimal/exact. Truncation,
trailing bytes, count/length disagreement, noncanonical embedded bytes, or an inconsistent repeated
binding is an encoding failure and yields no accepted digest.

The exact digests are:

```text
SHA-256(ASCII "rusty-data-os/exp1/workload-stream/v2" || 00 || complete_WS2_bytes)
SHA-256(ASCII "rusty-data-os/exp1/workload-manifest/v2" || 00 || complete_manifest_JCS_bytes)
```

Their metadata domains are respectively `rusty-data-os/exp1/workload-stream/v2` and
`rusty-data-os/exp1/workload-manifest/v2`; values are exactly 64 lowercase hexadecimal
characters. The NUL is one `00` octet. No v1 domain, raw digest, uppercase value, alternate
algorithm, self-digest placeholder, or transformed/partial preimage is accepted.

## 5. V2 manifest and JCS contract

The v2 manifest is UTF-8 I-JSON serialized with RFC 8785 JCS under R7
`EXP1-R7-JSON-JCS-1`: no BOM or trailing newline, no duplicate names, lone surrogates, NaN,
infinity, unknown fields, or JSON numbers. Objects are closed at every depth and all members are
required. R16 scalar syntax, sorting, bindings, artifact/provenance resolution, supersession, and
validation rules remain unchanged unless expressly replaced here.

The R16 top-level ledger is unchanged except `schema_version` and `profiles.manifest` are v2. The
profile ledger uses section 2's compatible family. In `generator_inputs`, remove
`reference_cardinality` and add exactly:

```json
"reference_cardinality_policy":{"kind":"segment_bootstrap_then_prior_v2","measured":{"bootstrap":"0","subsequent":"1"},"warm_up":{"bootstrap":"0","subsequent":"1"}}
```

This line demonstrates canonical member order, not a fixed subsequent value. The policy is a closed
object with exactly `kind`, `measured`, and `warm_up`; each segment is a closed object with exactly
`bootstrap` then `subsequent` in JCS order. `kind` is the literal shown. `bootstrap` is exactly
canonical u64 text `0`. `subsequent` is canonical u64 text in `1..18446744073709551615`.
Both segments are mandatory even at count zero. The scalar v1 member is forbidden.

The manifest validator derives segment ordinal from WS2 order and requires target count zero only
at ordinal zero and exactly the applicable `subsequent` value thereafter. Generation fails before
output if `subsequent > ordinal`; validation rejects an existing stream as `count-mismatch`.
Manifest counts, `profiles.envelope`, every OP1 envelope selector, ENV2 semantics, and the policy
must agree. The external digest descriptor is R16's same closed shape with the v2 manifest digest
profile/domain and an immutable exact-byte artifact reference.

## 6. Validator disposition and precedence

Validation is fail-closed and transactional. It returns no normalized object, digest, partial
stream, or repaired state on failure. Its stage precedence is:

1. bytes, lengths, UTF-8/JSON/duplicate/I-JSON/JCS and closed-schema encoding;
2. scalar ranges and the all-v2 compatibility tuple;
3. embedded/repeated operation, envelope, identity, namespace, segment, ordinal, and count bindings;
4. target syntax and duplicate target bytes (`E-REFERENCE-DUPLICATE`), before lookup;
5. bootstrap/subsequent policy cardinality;
6. ordered target classification; then
7. stream/artifact/manifest digest, immutable reference, supersession, and provenance bindings.

At step 5, zero targets at segment ordinal zero is valid. Zero targets later is
`E-REFERENCE-CARDINALITY`. A target-bearing bootstrap is not a cardinality failure: after target
syntax/duplication, each target is classified in encoded order as self, wrong-kind, wrong-fact,
cross-stream, cross-segment, future/defensive same-position self, eligible prior, then missing,
using R21–R23's exact errors. Consequently V25-06a is `E-REFERENCE-CROSS-SEGMENT` and V25-06b is
`E-REFERENCE-CROSS-STREAM`. The first invalid target wins. A bad target is never discarded to
manufacture a valid bootstrap, and every failure leaves accepted-prefix and mapper watermarks
unchanged.

Context-free conformance can prove encoding, profiles, policy counts, duplicates, and literal known
relationships in its oracle corpus. `E-REFERENCE-MISSING` is valid only with R23-proven complete
scope; without it the validator must report context required, never missing. Context construction
failures retain R23's stage precedence and are never target dispositions.

## 7. Required independent literal vectors

The authorized implementation PR MUST add independent, checked-in literal ENV2, REF2, SOP2, WS2,
manifest-JCS, exact-artifact SHA-256, v2 stream digest, and v2 manifest digest constants. Expected
bytes and hashes must be transcribed from an independently reviewed calculation, never generated by
the code under test. Each fixture records this R26 section as provenance and identifies every input.

The corpus MUST realize V25-01 through V25-06b exactly, including separate warm-up and measured
bootstraps and valid ordinal-1 references. Each positive operation has literal component bytes;
each positive stream has literal counts, bytes, byte length, raw SHA-256 and domain digest; each
positive manifest has literal canonical bytes, length, raw SHA-256 and external digest descriptor.
Negative vectors state the single exact byte/field mutation and expected first disposition.

The same corpus MUST cover: malformed target encoding before lookup; duplicate targets before
lookup; self; future; wrong kind; wrong fact; cross-stream before cross-segment/future/missing;
cross-segment before future/missing; missing only after complete scope; first-invalid-target wins;
no transactional state advance; exact R23 scope success; omitted, extra, substituted, duplicate,
foreign, digest-disagreeing and noncanonical scope failures; R21 inclusive maximums and one-over
resource failures; and R23 construction precedence. V1 golden tests MUST remain byte-for-byte
unchanged and MUST reject every v2 substitution, while v2 tests reject every v1 substitution.

## 8. Immutability, supersession, and binding

R26 supersedes R25 only where R25 deferred exact v2 profiles, encodings, domains, manifest bytes,
and validator authorization. It does not supersede R25's failed-assumption record or R21–R23's
context rules. It supersedes R24's already-superseded implementation path with no new
reference-context authority.

Every v1 identifier, byte, vector, digest, fixture, manifest, and disposition remains unchanged.
A v2 correction creates a new manifest ID and workload ID, immutable bytes, artifact records and
digests; R16 supersession edges name the exact prior manifest and never mutate it. A changed stream
requires new exact bindings. A later R23-compatible v2 scope must name those exact new bindings and
use a new scope identity; neither v1 nor v2 membership proves the other.

## 9. Narrow follow-on implementation authorization

After R26 merges, exactly one PR may change the existing
`experiments/exp-0001/crates/exp1-workload-conformance` crate to implement this v2 conformance
family and validator alongside, not instead of, v1. It may change only that crate's existing Rust
source and tests and add literal test data under that crate. No Cargo manifest, `Cargo.lock`,
toolchain, workspace member, or workflow may change. Allowed dependencies are only its already
reviewed direct workspace path dependency/dependencies and `core`/`std`; no new normal, dev, build,
target, git, registry, or path dependency is authorized.

Tests must exercise every requirement in section 7, golden encode/decode equality, exact digest
preimages, manifest JCS and closed-ledger rejection, all mixed-version combinations, deterministic
error precedence, integer/length boundaries, and unchanged complete v1 regression vectors. The
unchanged R9 format, clippy, and full-workspace test sequence is the completion gate. Review and
exact-head CI establish only bounded conformance correctness evidence.

Excluded are changes to `exp1-record-format`, `exp1-raw-append-replay`, any authority document,
Cargo/workflows/toolchain, a fourth crate, R21–R23 context implementation, mapping or append/reopen
integration, fixture generation at runtime, generated workloads, harness/capture/execution,
benchmarks/results, D2/D3, `fsync`, faults, durability/recovery/performance claims, adapters,
production crates, server/network/query/distributed work, and unsafe code. A later reference-context
authorization remains mandatory after the v2 conformance implementation is merged and reviewed.
