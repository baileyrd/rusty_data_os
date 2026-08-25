# EXP-0000 Reproducible Workload Contract

**Scope:** deterministic operation streams for EXP-0001 and comparable baselines
**Status:** frozen for Experiment 0 measurement preparation
**Evidence classification:** controlled measurement input; not an implementation specification, production-workload claim, or experimental result

## 1. Purpose and boundaries

This contract fixes the workload dimensions needed to give Data OS candidates and baselines semantically equivalent input. It complements the [semantic event envelope](SEMANTIC-EVENT-ENVELOPE.md) and the [acknowledgement, visibility, fault, and durability contract](ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md); it does not change either contract or choose an encoding, framing, generator implementation, or clock.

The byte and rate boundaries are distinct:

- **payload bytes** are bytes inside the opaque domain-payload boundary only. They exclude the semantic envelope and physical record framing.
- **encoded event bytes** are payload bytes plus the encoded envelope and all event-encoding overhead. They exclude storage-path framing outside the encoded event.
- **physical bytes written** are all output emitted through the measured storage path, including framing, alignment, integrity, filesystem, or other attributable write amplification where measurable.
- **events/second**, **payload bytes/second**, **encoded event bytes/second**, and **physical bytes/second** are separate measurements. A byte-rate result must name its boundary.

Every result reports all applicable byte counts and rates. It must never present payload size as total record size or use an unqualified “MB/s.” If a layer cannot expose an applicable count, the result records it as unavailable and explains why rather than substituting another boundary.

## 2. Fixed payload-size classes

| Class | Exact payload bytes | Role |
|---|---:|---|
| P0 | 0 | Envelope/path-overhead control. |
| P1 | 32 | Tiny fact; primary comparison. |
| P2 | 256 | Small fact; primary comparison. |
| P3 | 4 KiB (4,096) | Larger data-bearing fact; primary comparison. |
| P4 | 64 KiB (65,536) | Boundary diagnostic. |
| P5 | 1 MiB (1,048,576) | Optional stress/diagnostic only. |

P1, P2, and P3 are the **primary fixed-size comparison set**. Together they expose envelope-dominated through larger-payload behavior without allowing the zero-length control or unusually large stress point to dominate the ordinary comparison. P0 and P4 are required boundary diagnostics when applicable. P5 is excluded from the primary comparison and is run only when explicitly predeclared and reported separately.

These sizes are controlled experimental points selected to isolate costs. They are not a claim that this set, or any mix below, represents all real systems.

## 3. Payload-size distributions and assignment

The following distributions are frozen. `N` is the operation count for one segment. Warm-up and measurement are separate segments, each with its own zero-based ordinal and count constraint; a manifest may select different named distributions for them, but may not silently continue one segment's ordinal into the other.

| Profile | Exact class allocation | Count constraint | Classification |
|---|---|---|---|
| `fixed-Pn` | Every operation is the named class. | Any positive `N`. | P1–P3 primary; P0/P4 diagnostic; P5 separate optional stress. |
| `mixed-equal-P1-P4` | P1 25%, P2 25%, P3 25%, P4 25%. | `N` is a multiple of 4. | Primary mixed profile. |
| `mixed-weighted-P1-P4-v1` | P1 60%, P2 25%, P3 10%, P4 5%. | `N` is a multiple of 20. | Primary mixed profile, but not claimed representative. |
| `stress-P5` | Every operation is P5. | Any positive `N`. | Optional diagnostic; never merged into normal summaries. |

Class order is fixed by the following platform-independent **semantic class-order algorithm v1**. It is part of this workload contract, not a future generator implementation:

1. Interpret `seed` as an unsigned 64-bit integer written as canonical decimal text (`0` through `18446744073709551615`, with no sign or leading zero except the value `0`). There is no byte-order interpretation.
2. Select the profile's base cycle: `mixed-equal-P1-P4` is `[P1, P2, P3, P4]`; `mixed-weighted-P1-P4-v1` is `[P1, P2, P1, P1, P3, P1, P2, P1, P1, P4, P1, P2, P1, P1, P3, P1, P2, P1, P1, P2]`. A fixed profile's cycle is its single named class.
3. Let `L` be the cycle length. Domain-separate the segments with `segment_offset = 0` for `warm-up` and `segment_offset = 1` for `measured`. For zero-based ordinal `i` within that segment, assign `cycle[(i + (seed mod L) + segment_offset) mod L]`.
4. Restart `i` at zero for each segment. Do not consume warm-up operations from the measured segment, even when the two segments use the same profile and seed.

The mixed-profile count constraints make each segment an integral number of complete cycles, so rotation preserves the exact counts. Fixed profiles are unaffected by rotation. Class assignment uses no generator ID, workload ID, byte interpretation, rejection, rounding, runtime random choice, benchmark timing, producer assignment/scheduling, or wall clock. These inputs may domain-separate other future generation rules, but cannot change semantic class order v1.

Normative test vectors (the shown values are the first operations of the named segment) are:

| Profile / seed / segment | First assigned classes | Exact counts for the stated segment |
|---|---|---|
| `mixed-equal-P1-P4` / `0` / warm-up, `N=8` | `P1, P2, P3, P4, P1, P2, P3, P4` | P1=2, P2=2, P3=2, P4=2 |
| `mixed-equal-P1-P4` / `0` / measured, `N=8` | `P2, P3, P4, P1, P2, P3, P4, P1` | P1=2, P2=2, P3=2, P4=2 |
| `mixed-equal-P1-P4` / `7` / warm-up, `N=8` | `P4, P1, P2, P3, P4, P1, P2, P3` | P1=2, P2=2, P3=2, P4=2 |
| `mixed-weighted-P1-P4-v1` / `0` / warm-up, `N=20` | `P1, P2, P1, P1, P3, P1, P2, P1, ...` | P1=12, P2=5, P3=2, P4=1 |
| `mixed-weighted-P1-P4-v1` / `0` / measured, `N=20` | `P2, P1, P1, P3, P1, P2, P1, P1, ...` | P1=12, P2=5, P3=2, P4=1 |
| `mixed-weighted-P1-P4-v1` / `7` / warm-up, `N=20` | `P1, P1, P4, P1, P2, P1, P1, P3, ...` | P1=12, P2=5, P3=2, P4=1 |

These vectors establish semantic size-class order only. Payload-content bytes, semantic identities, and serialized envelopes are **not yet byte-for-byte regenerable** unless an immutable pre-generated stream is supplied: their future generator specifications must still declare platform-independent algorithms, domain separation, integer/byte interpretation where applicable, and test vectors. Pre-generated streams remain valid when their manifest and byte-level identity are preserved.

## 4. Payload-content profiles

Payload length and payload contents are independent dimensions:

- **`deterministic-high-variation` (primary neutral profile):** deterministic, nonuniform bytes with high byte-to-byte and event-to-event variation. It is intended to avoid accidentally rewarding zero filling, repetition, compression, deduplication, or a small cache footprint; it is not a randomness or entropy claim.
- **`repeated-low-variation` (diagnostic):** deterministic repeated or low-variation patterns. Use it separately to expose compression, deduplication, checksumming, copying, or cache sensitivity.
- **`all-zero` (explicit diagnostic only):** every payload byte is zero. It is never an implicit default or included in a primary summary.

Generator identity, generator version, seed, segment, and event ordinal are recorded or derivable for every operation. A generator version must specify byte-for-byte platform-independent expansion, domain separation among content profiles and ordinals, and reference test vectors. Data OS and baselines receive the resulting bytes; neither may replace them with an internally convenient pattern. No generator implementation or dependency is selected here.

## 5. Semantic-envelope profiles

The approved envelope remains authoritative. Profiles control only which already-conditional metadata is applicable; required fields remain present and payload meaning remains opaque.

| Profile | Controlled relationship |
|---|---|
| `envelope-minimal` | Required envelope fields plus originating request identity for current command-originated scope; no optional/conditional relationship is invented. |
| `envelope-provenance` | Minimal profile plus a declared source/actor provenance value. |
| `envelope-causal-reference` | Minimal profile plus reference(s) to valid earlier generated event identities. |
| `envelope-correction-retraction-reference` | Minimal profile plus a correction or retraction fact type and valid earlier generated target reference(s). |

`envelope-minimal` is the primary ingest profile. The other profiles are diagnostics varied separately, one relationship profile at a time, so their metadata cost is attributable. A manifest declares reference cardinality and deterministic target-selection rule. Profiles neither make conditional metadata universally mandatory nor settle its encoding, validation policy, locality rules, or physical size.

## 6. First-class temporal profiles

Temporal generation uses logical ordinals and offsets until timestamp representation and clock sources are selected. Let the zero-based ordinal within the applicable stream segment be `i`, logical system-acceptance reference be `S(i)`, and effective-time value be `E(i)`. The generator provides only the effective-time input/relationship; lifecycle instrumentation supplies actual system, durability, and observation times at their declared points.

| Profile | Deterministic semantic rule |
|---|---|
| `time-monotonic-effective` | `E(i)` advances by one logical unit with `i`. |
| `time-equal-burst-v1` | Consecutive groups of 100 operations share one effective-time ordinal; the ordinal advances once per group. A final partial group is permitted. |
| `time-late-arriving-v1` | Every tenth operation (`i mod 10 = 9`) is late by a logical offset of 100 relative units; other operations use the monotonic effective-time relation. The realized lifecycle evidence must preserve that these selected effective times precede their system times. |
| `time-out-of-effective-order-v1` | In each consecutive group of four operation ordinals, effective-time rank is assigned in order `[0, 2, 1, 3]`; the final partial group uses the corresponding prefix. Assigned local-sequence order remains `i` in the single-producer reference. |

`time-monotonic-effective` is the primary profile. The other profiles are separate temporal diagnostics. Logical units express ordering and controlled offsets only; they do not select timestamp width, epoch, precision, timezone, clock, or concrete mapping. If a later mapping cannot preserve a profile's required relationships, that run is invalid.

For the single-producer reference case, operation-ordinal order is the predeclared **assigned local-sequence order**. In a concurrent case without a controlled global submission schedule, each producer's local ordinal order is fixed but the cross-producer interleaving and resulting ordinal-to-assigned-sequence mapping are observed outputs. A controlled global submission schedule may instead predeclare that mapping. Assignment alone never makes a candidate canonical: D0/D1 candidates remain provisional, and in D2/D3 only events that canonically commit under the lifecycle and durability contract contribute their assigned sequence positions to canonical replay order. Whether uncommitted or failed candidates may leave gaps is deliberately unresolved. Effective-time ordering never replaces assigned sequence or the canonical replay order of committed events. System acceptance time, durability time, and observation time arise only at their approved lifecycle points and must not be fabricated as payload or generator inputs. Observation time remains outside the immutable original envelope. Every run records exactly one temporal profile and its version.

## 7. Execution dimensions

Before execution, every workload declares:

- measured operation count or measured duration, including the stopping rule;
- warm-up operation count or duration, separately from measurement;
- producer count;
- maximum outstanding queue depth;
- batch/group policy, including size/window/cut trigger and the distinction between transport batching and a D3 durability group;
- durability mode D0, D1, D2, or D3 and its complete platform contract;
- payload-size distribution;
- payload-content profile;
- envelope profile and any reference cardinality/target rule;
- temporal profile;
- seed and generator identity/version;
- cache and preconditioning state, including warm, cold, uncontrolled, or intentionally studied caches.

The reference execution case is one producer with maximum queue depth one and no batching except where the declared durability mechanism intrinsically requires a predeclared group policy. It is the attribution control, not a universal production recommendation. Hardware-dependent concurrency is not frozen. Every additional producer count and queue-depth point must be selected and recorded before execution; results may not add favorable points after inspection.

Duration-based runs must still record the exact completed operation count and per-class counts. Operation-count runs are preferred when exact cross-system stream identity is required; a duration stopping rule cannot be used to imply identical stream prefixes unless that prefix is explicitly fixed.

## 8. Matrix discipline

### 8.1 Minimal reference matrix

For each D0–D3 mode measured by EXP-0001, hold the reference execution case, `deterministic-high-variation`, `envelope-minimal`, `time-monotonic-effective`, seed, operation count, warm-up, and cache state constant, then run:

1. `fixed-P1`;
2. `fixed-P2`;
3. `fixed-P3`;
4. `mixed-equal-P1-P4`;
5. `mixed-weighted-P1-P4-v1`.

P0 and P4 fixed runs are boundary diagnostics for each applicable measured mode. P5, content diagnostics, nonminimal envelopes, temporal diagnostics, and concurrency sweeps are separate diagnostic series. A mode that a candidate cannot implement is explicitly “not supported/not tested,” never silently omitted. D0/D1 provisional modes and D2/D3 canonical modes are never ranked or compared as equivalent guarantees; comparisons occur within an equivalent declared semantic and platform contract.

### 8.2 Controlled expansion

- Change one independent variable from a named reference cell at a time.
- Predeclare every matrix cell and stopping rule. Preserve failed, unfavorable, invalid, inconclusive, and not-tested cells with their classifications.
- Use a full factorial subset only for a predeclared interaction hypothesis that identifies the factors, levels, analysis, and sample budget. Do not expand all dimensions opportunistically.
- Data OS and baselines consume the identical pre-generated stream or byte-for-byte reproducible stream from the same manifest. Adapter-specific envelope encoding may differ, but semantic fields, payload bytes, identities, references, and temporal relationships must be equivalent and attributable encoding/physical byte counts remain separate. The single-producer reference also preserves the identical global operation order. Concurrent comparisons preserve the identical operation set, producer-to-ordinal assignment, and each producer's local order; they preserve a global order only when an equivalent controlled global submission schedule was predeclared.

Durability configurations with different guarantees may appear in the trade-space report, but not as like-for-like performance winners and losers.

## 9. Correctness and reproducibility gates

A stream or comparable run is valid only when:

1. repeated runs and comparable baselines use the same workload/operation-stream identity and operation set; operation-count comparisons use the same exact global order for the single-producer reference, while concurrent comparisons use the same producer assignment and per-producer local order unless an equivalent controlled global submission schedule also fixes global order;
2. measured and warm-up operation counts and exact counts per payload class/profile match the manifest;
3. every payload length matches its assigned class before ingestion and after recovery where applicable;
4. the recorded manifest deterministically regenerates every aspect whose algorithm has been frozen; until byte-content, identity, and envelope-generation rules are frozen, byte-for-byte validity instead requires an immutable pre-generated stream and its digest under a separately declared digest method;
5. request identities and permanent event identities are distinct and stable across adapters;
6. causal and correction/retraction references select valid, earlier generated targets under the manifest rule; target-bearing streams include a deterministic prefix of ordinary target events when needed;
7. late-arriving operations retain effective-time versus lifecycle system-time distinctions, and no timestamp field substitutes for sequence order;
8. stream generation and identities do not depend on benchmark timing, thread scheduling, producer interleaving, wall-clock coincidence, or baseline behavior; and
9. assigned local sequence identifiers are unique and strictly monotonic in assigned order, and the ordinal-to-assigned-sequence mapping is recorded. For the single-producer reference, that mapping follows predeclared ordinal order. A concurrent run follows a predeclared global assignment order only when it declares a controlled global submission schedule; otherwise cross-producer interleaving and the mapping are observed outputs, and different systems are not required to produce identical cross-producer assignment order. D0/D1 validate assignment behavior without claiming canonical history or replay. For D2/D3, replay deterministically reproduces the assigned-sequence order of the events that canonically committed. Within the declared successful/eligible operation set for the mode, verification detects and reports unexplained loss, duplication, or invention; failed or uncommitted candidates are not required to appear as canonical events, and any resulting sequence gaps are reported without treating the unresolved gap policy as decided.

Any adapter transformation is documented and verified against a semantic stream oracle before its performance is interpretable.

## 10. Documentation-level workload manifest

The manifest is a logical schema, not a choice of JSON, TOML, YAML, binary format, or other serialization. It contains at least:

| Field group | Required information |
|---|---|
| Workload identity | Workload ID and workload-contract/profile version; operation-stream identity; measured and warm-up segment identities. |
| Generator | Semantic class-order algorithm/version and its canonical unsigned-decimal seed; content/identity/envelope generator IDs/versions, algorithm specification references, domain separation, and test-vector versions when those rules exist. |
| Extent | Measured operation count or duration/stopping rule; warm-up count or duration; exact completed counts. |
| Payload | Size distribution/version, exact expected and completed class counts for each segment, content profile/version. |
| Semantics | Envelope profile/version, reference cardinality and target-selection rule where applicable, temporal profile/version and logical parameters. |
| Identity | Deterministic request-identity and event-identity derivation specification references and namespaces, without conflating them. |
| Execution | Producer count, outstanding queue depth, producer-to-ordinal assignment rule, each producer's local-order rule, and either a controlled global submission schedule or an explicit declaration that cross-producer interleaving is observed. |
| Grouping | Batch/group mode, size/window/cut trigger, membership rule, and whether it is a D3 durability group. |
| Durability | D0–D3 mode and reference to the complete declared platform durability contract. |
| State | Cache state, preconditioning procedure, dataset/storage initial state, and warm/cold/steady-state classification. |
| Stream verification | Pre-generated location if used, byte length, operation count, and digest method/value; otherwise deterministic regeneration procedure and expected test-vector results. Result metadata records the ordinal-to-assigned-sequence mapping, the declared successful/eligible operation set and mode, canonical-commit outcome for D2/D3 candidates, and any unexplained loss, duplication, invention, or sequence gaps. |

The manifest is frozen before execution and retained with raw results. Result metadata adds actual counts and environment data without rewriting the workload definition.

## 11. Deliberately unresolved

This contract selects only the semantic size-class order algorithm, not a generator implementation. It does not select payload-content expansion; event encoding or framing; identity or digest algorithms; timestamp representation, epoch, precision, or clock; physical manifest serialization; generator language/library/dependency; integrity and retry policies; sequencing-gap policy; transactions; checkpoints; baseline products/configurations; platform APIs; universal concurrency values; or a universal cross-producer ordering mechanism. It authorizes no implementation or benchmark execution and does not make EXP-0001 ready. The next recommended bounded output is baseline selection and configuration, because fair adapters and settings now depend on this frozen workload contract.
