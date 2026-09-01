# R22 Cross-Segment Reference Rule

**Profile:** `EXP-0001-R22-CROSS-SEGMENT-REFERENCE-v1`
**Status:** frozen documentation/governance decision
**Evidence classification:** synthetic documentation design; not implementation, execution, or
experimental evidence

## 1. Decision

R22 selects **strictly segment-local references** for WS1. Segment is part of the reference domain.
For an operation in segment `s` at segment ordinal `i`, R12 section 5.3's candidate prefix `[0,i)`
means exactly the ordinary EventIds at segment ordinals `0 <= j < i` in **the same stream and the
same segment `s`**. Ordinal comparison is segment-local, never a comparison of total WS1 positions.
Total WS1 position continues to order bytes and accepted-prefix processing only.

Consequently:

| Current operation | Target operation | Disposition |
|---|---|---|
| warm-up | earlier warm-up ordinary event | eligible |
| measured | earlier measured ordinary event | eligible |
| warm-up | measured | prohibited: `E-REFERENCE-CROSS-SEGMENT` |
| measured | warm-up | prohibited: `E-REFERENCE-CROSS-SEGMENT` |

At warm-up ordinal zero, the same-segment candidate prefix is empty. At measured ordinal zero, the
same-segment candidate prefix is independently empty; preceding warm-up operations do not enter it.
Any known target in the other segment is cross-segment at either boundary and at every later
ordinal. Equal numeric ordinals in the two segments are distinct `(segment, ordinal)` positions and
never alias.

This is the smallest authority-consistent rule. It preserves R12's deterministic construction from
the operation's segment-local ordinal, respects the ordinal restart, prevents a changed warm-up
count/content from silently changing measured reference semantics, leaves R14's warm-up-then-measured
byte ordering untouched, and permits fail-closed validation without inventing runtime or adapter
behavior. The alternatives were rejected as follows: allowing measured-to-warm-up references makes
measured semantics depend on warm-up; total-position eligibility contradicts the segment-local
ordinal input and creates a second ordinal meaning; and asymmetric eligibility still introduces
that dependency while not resolving the duplicate numeric ordinal domain as simply.

## 2. Exact error and precedence

No R12 error accurately names a known target in the same stream but the other segment. It is not
`CrossStream`, `Future`, or `Missing`. R22 therefore defines the experiment-local disposition:

```text
E-REFERENCE-CROSS-SEGMENT
```

It means: **the target bytes resolve to an EventId binding in the current operation's stream, but
the target binding's WS1 segment differs from the current operation's segment**. It is a prohibited
reference regardless of either operation's total position or numeric segment ordinal. It did not
exist in R12. A later authorized implementation must add a distinct context-specific variant such
as `ReferenceError::CrossSegment`; it must not reuse an existing variant.

After semantic-operation validation and current-position/accepted-prefix matching, each encoded
target is classified in order with this precedence:

1. current EventId -> `E-REFERENCE-SELF`;
2. known non-Event identity -> `E-REFERENCE-WRONG-KIND`;
3. known correction or retraction EventId -> `E-REFERENCE-WRONG-FACT`;
4. known EventId in another stream -> `E-REFERENCE-CROSS-STREAM`;
5. known EventId in the same stream but other segment -> `E-REFERENCE-CROSS-SEGMENT`;
6. known same-stream, same-segment EventId with greater ordinal -> `E-REFERENCE-FUTURE`;
7. known same-stream, same-segment EventId at the current ordinal -> `E-REFERENCE-SELF`
   defensively;
8. known same-stream, same-segment, lower-ordinal ordinary EventId -> valid;
9. identity absent from a proven-complete catalog -> `E-REFERENCE-MISSING`.

Duplicate target bytes remain a semantic-validation failure before lookup. The first invalid member
of a multi-target list determines the result. Wrong-kind and wrong-fact therefore describe the
target before locality, and cross-stream precedes cross-segment because only a same-stream binding
has a meaningful WS1 segment relationship. Missing remains unavailable until the separate closed
scope proof exists. Every failure is transactional: no frame or next state is returned and catalog,
accepted count, sequence watermark, and physical-ordinal watermark remain unchanged.

## 3. R21 catalog and accepted-prefix effect

R21's `IdentityBinding` already retains `stream_namespace`, `stream_position`, `segment`, and
`segment_ordinal`; no new catalog field or wire representation is needed. A later implementation,
only after closed-scope governance also resolves and a separate increment authorizes work, must:

1. treat `(stream_namespace, segment)` as the eligibility domain;
2. use `segment_ordinal` only within that domain and never substitute `stream_position`;
3. keep `stream_position` solely for exact accepted-prefix/current-operation matching and state
   advance;
4. add the distinct `CrossSegment` reference variant and precedence from section 2; and
5. advance the accepted prefix and R20 mapping state exactly once only after every target and all
   later mapping checks succeed.

The catalog must retain both segments, including future measured entries, so a warm-up-to-measured
target can be classified as cross-segment rather than missing. Catalog presence does not make a
future operation accepted or eligible. R22 does not resolve how the constructor proves that its
multi-stream input is the complete closed classification scope.

## 4. Compatibility and supersession

R22 supersedes only the ambiguous cross-segment interpretation of R12 section 5.3. Wherever that
section says candidate prefix `[0,i)`, it is henceforth read as the same-stream, same-segment prefix
defined in section 1. All unaffected R12 generation, collision, ordering, reference-cardinality,
and existing error rules remain authoritative. R22 does not silently rewrite R12 or claim that
`E-REFERENCE-CROSS-SEGMENT` existed there.

R12's existing vectors and bytes remain unchanged because none specifies a cross-segment target.
R14's SOP1/WS1 encodings, digests, and warm-up-then-measured byte order remain unchanged. R16 is
unchanged. R21 section 2.1's and section 5's unresolved cross-segment text is superseded by R22;
R21's catalog design, accepted-prefix lifecycle, other precedence rules, vectors, exclusions, and
unresolved complete-closed-scope requirement remain in force.

## 5. Documentation vectors

All `SYN22-*` cases are **synthetic documentation design, not observations, implementation tests,
or evidence**. They use complete WS1-valid operations and ordinary EventId targets unless another
kind is stated. “State unchanged” means no frame/next state and bit-for-bit unchanged catalog,
accepted count, and both R20 watermarks.

| Vector | Arrangement | Expected result |
|---|---|---|
| SYN22-01 | warm-up ordinal 1 targets warm-up ordinary EventId at ordinal 0 | valid; state advances exactly once |
| SYN22-02 | measured ordinal 1 targets measured ordinary EventId at ordinal 0 | valid; state advances exactly once |
| SYN22-03 | measured ordinal 0 targets a warm-up EventId | `E-REFERENCE-CROSS-SEGMENT`; state unchanged |
| SYN22-04 | measured ordinal 7 targets a warm-up EventId | `E-REFERENCE-CROSS-SEGMENT`; state unchanged |
| SYN22-05 | warm-up operation targets a measured EventId present later in WS1 | `E-REFERENCE-CROSS-SEGMENT`, not `Future`; state unchanged |
| SYN22-06 | warm-up ordinal 0 and measured ordinal 0 exist; either targets the other's EventId | distinct domains; `E-REFERENCE-CROSS-SEGMENT` |
| SYN22-07 | warm-up ordinal 1 targets its own EventId | `E-REFERENCE-SELF`; state unchanged |
| SYN22-08 | measured ordinal 1 targets its own EventId | `E-REFERENCE-SELF`; state unchanged |
| SYN22-09 | warm-up ordinal 1 targets warm-up ordinal 2 | `E-REFERENCE-FUTURE`; state unchanged |
| SYN22-10 | measured ordinal 1 targets measured ordinal 2 | `E-REFERENCE-FUTURE`; state unchanged |
| SYN22-11 | multi-target measured operation has valid earlier measured target, then warm-up target, then another valid target | second target yields `E-REFERENCE-CROSS-SEGMENT`; no partial acceptance; state unchanged |
| SYN22-12 | either allowed case above with valid sequence and physical ordinal | one frame; accepted count and both R20 watermarks each advance exactly once; catalog unchanged |

R12 R01 continues to cover an ordered same-segment ordinary prefix; R02 continues to cover
same-segment self and future; R03 continues to cover truly foreign-stream, wrong-kind, and
wrong-fact targets; and R04 continues to give duplicate validation precedence. R21 V21-13/V21-14
continue to specify transactional failure/success. The supplemental vectors add only the missing
segment dimension.

## 6. Disposition, remaining blocker, and exclusions

R22 **fully resolves the cross-segment governance question**. It does not resolve the independent
proof of a complete closed stream set. Therefore R21's full reference-context implementation
remains unauthorized, the complete R20 correctness gate remains open, and a later authorization,
implementation, exact-head review, and CI are still required. The live Linux capture freeze remains
open, so no descriptive D1 harness or execution is authorized.

R22 adds or authorizes no Rust, Cargo, or lockfile change; authority-crate or reference-context
implementation; append/reopen integration; workload materialization or execution; benchmark
execution or evidence; Linux capture design or implementation; unsafe code or external dependency;
fourth crate; D2/D3 or `fsync`; canonical commit/recovery; durability/performance claim; fault,
adapter, production, server, network, query, or distributed work.
