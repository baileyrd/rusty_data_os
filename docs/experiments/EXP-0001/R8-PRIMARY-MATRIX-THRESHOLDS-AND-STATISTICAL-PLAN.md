# R8 primary matrix, thresholds, and statistical plan

**Identity:** `EXP-0001-R8/v1`

**Freeze date:** 2026-08-28

**Status:** complete documentation design; implementation and execution remain unauthorized
**Repository basis:** `f9d9876cf6599345a2e2244223a530ada2b9a828`

## 1. Scope and honest outcome

This record freezes the smallest candidate primary matrix, the analysis method, and the owner-approved practical-value registry before any EXP-0001 observations exist. The numeric margins are accountable product and engineering value judgments, not empirical facts or claims about expected performance.

Therefore BLK-023, BLK-024, and UNK-008 are resolved for this documentation decision and R8 is complete as documentation design. R9 is the next readiness increment, but neither this record nor R8 completion authorizes R9 content, implementation, Cargo, apparatus, execution, benchmarks, or durability claims. Those remain subject to the execution-readiness plan's separate gates.

### 1.1 Exact owner approval

The accountable owner approved `EXP-0001-R8/thresholds-v1` on **2026-08-28**, effective only for future EXP-0001 confirmatory series. The owner selected 10% as the minimum meaningful performance/resource improvement and ±5% as practical equivalence; for resource and recovery guardrails, the owner selected a wider 10% unacceptable-regression boundary so a 5–10% cost can be reviewed as constrained rather than automatically rejected. These are explicit owner value judgments about worthwhile benefits and tolerable costs, not measurements, statistical facts, or evidence that any system will attain them. Complexity is reviewed without a numeric composite because the owner judges independently visible counts and named findings more decision-useful than arbitrary weighted compensation. The shared values apply to the exact closed cell sets in section 4 because consistent practical meaning across payloads, eligible baselines, and D-modes is an owner policy choice; applicability and evidence gates still differ by cell. The approval is prospective and cannot classify or reclassify prior evidence.

## 2. Frozen claim and cell registry

### 2.1 Claims

| Claim ID | Narrow question | Primary metrics | Gate |
|---|---|---|---|
| C-INGEST | At an equivalent D-mode, what is the Data OS ingestion cost relative to the eligible baseline? | events/s; logical and encoded MiB/s; acknowledgement lifecycle p50/p90/p95/p99/p99.9 and maximum | All section 4 gates |
| C-RESOURCE | Does the same comparison stay within approved resource and physical-work limits? | CPU time/event; allocations and bytes/event; peak RSS; requested/synchronized/physical bytes and write amplification | All section 4 gates |
| C-RECOVERY | For D2/D3 only, does valid canonical history recover within approved limits after every claimed fault? | scan/replay events/s; scan/replay and time-to-ready durations | Correctness, platform, apparatus, and complete fault coverage first |
| C-COMPLEXITY | Is ingestion-path complexity within approved bounds? | separately defined counts and predeclared qualitative rules; never a composite score | Complete scoped evidence and owner-approved rule |

Correctness is absolute, not a metric that speed can compensate. C-RECOVERY is unavailable for D0/D1. Complexity remains separate from performance and cannot resolve architecture-wide HYP-0001.

### 2.2 Coordinates shared by every registered cell

All cells use the reference case: one producer, queue depth one, no batching except controlled D3 grouping, `deterministic-high-variation` content, `envelope-minimal`, `time-monotonic-effective`, identical immutable stream identity, and the R5/R6 mapping for the named baseline. Setup and preconditioning are outside the lifecycle interval; cache state and initial/new/reused storage state are identical within a pair and recorded. The measured lifecycle is caller entry immediately before validation/construction through that event's acknowledgement return; D3 includes group-formation wait. Throughput uses successfully completed eligible measured events divided by that same measured wall interval. Logical bytes are payload bytes; encoded and physical bytes remain separate.

The manifest freezes seed `0x455850303030312d5238` (the ASCII domain `EXP0001-R8`), distinct warm-up and measured stream identities, exact cache/preconditioning state, and exact series/environment identity. A future series must use 100,000 warm-up operations and 1,000,000 measured operations per repetition. These fixed counts satisfy the frozen p99.9 support floor for every fixed profile and both mixed profiles; mixed extents are integral cycles. Warm-up is never measured or consumed from the measured segment. A validated implementation may require a larger predeclared count in a new analysis version, never a smaller one.

Grouping for D3 is the R5 observable controlled profile and must freeze membership/window/count before a series; because that policy is not yet executable or effectively validated, these cells remain blocked. No other blocking variable is permitted besides planned temporal block/pair number. Pairing requires the same machine, storage path, immutable stream, preparation state, and temporal block. Any change to code/build, effective settings, adapter, storage stack, grouping, workload contract, analysis, or threshold version starts a new series.

### 2.3 Closed candidate-primary registry

Profile codes are `F1=fixed-P1 (32 B)`, `F2=fixed-P2 (256 B)`, `F3=fixed-P3 (4,096 B)`, `ME=mixed-equal-P1-P4`, and `MW=mixed-weighted-P1-P4-v1`. For each row, the five stable cell IDs are `PC-<mode>-<baseline>-{F1,F2,F3,ME,MW}`. They share C-INGEST and C-RESOURCE; D2/D3 rows also share C-RECOVERY and all rows share separately reported C-COMPLEXITY.

| Mode | Eligible baseline/profile | Five exact cell IDs | Equivalence/platform contract | Frozen role |
|---|---|---|---|---|
| D0 | B0 R5 profile | `PC-D0-B0-F1`, `PC-D0-B0-F2`, `PC-D0-B0-F3`, `PC-D0-B0-ME`, `PC-D0-B0-MW` | provisional process-memory equivalence candidate; never canonical | confirmatory candidate after all separate gates pass |
| D1 | B1 R5 D1 | `PC-D1-B1-F1`, `PC-D1-B1-F2`, `PC-D1-B1-F3`, `PC-D1-B1-ME`, `PC-D1-B1-MW` | provisional OS-buffer equivalence candidate | confirmatory candidate after all separate gates pass |
| D1 | B2 SQLite 3.53.4 R6 D1 | `PC-D1-B2-F1`, `PC-D1-B2-F2`, `PC-D1-B2-F3`, `PC-D1-B2-ME`, `PC-D1-B2-MW` | externally classified provisional D1; effective settings required | confirmatory candidate after all separate gates pass |
| D1 | B3 RocksDB 11.8.1 R6 D1 | `PC-D1-B3-F1`, `PC-D1-B3-F2`, `PC-D1-B3-F3`, `PC-D1-B3-ME`, `PC-D1-B3-MW` | externally classified provisional D1; effective settings required | confirmatory candidate after all separate gates pass |
| D2 | B1 R5 D2 | `PC-D2-B1-F1`, `PC-D2-B1-F2`, `PC-D2-B1-F3`, `PC-D2-B1-ME`, `PC-D2-B1-MW` | conditional on the exact R4/R5 platform and fault contract | unsupported until BLK-015/022 and other gates pass |
| D2 | B2 SQLite 3.53.4 R6 D2 | `PC-D2-B2-F1`, `PC-D2-B2-F2`, `PC-D2-B2-F3`, `PC-D2-B2-ME`, `PC-D2-B2-MW` | conditional on VFS/platform equivalence and recovery evidence | unsupported until BLK-015/022 and other gates pass |
| D2 | B3 RocksDB 11.8.1 R6 D2 | `PC-D2-B3-F1`, `PC-D2-B3-F2`, `PC-D2-B3-F3`, `PC-D2-B3-ME`, `PC-D2-B3-MW` | conditional on sync/platform equivalence and recovery evidence | unsupported until BLK-015/022 and other gates pass |
| D3 | B1 R5 controlled D3 | `PC-D3-B1-F1`, `PC-D3-B1-F2`, `PC-D3-B1-F3`, `PC-D3-B1-ME`, `PC-D3-B1-MW` | conditional on observable exact membership, shared sync/outcome, individual acknowledgement, platform and fault contract | unsupported until grouping, BLK-015/022, and other gates pass |

This is exactly 40 unique cells: 5 D0, 15 D1, 15 D2, and 5 D3. Completeness means all five cells and all 12 valid independent pairs in every enabled row complete every applicable claim and gate; no partial row supports a row-level claim. A baseline may be blocked or not tested, but never silently omitted. A claim is only cell-specific unless every required row/cell meets the rule.

P0/P4 fixed boundary runs, P5, all-zero/compressible content, nonminimal envelopes, temporal diagnostics, producer/concurrency sweeps, queue-depth sweeps, cache experiments, and unplanned fault conditions are diagnostic/exploratory and outside this registry. SQLite atomic transactions, RocksDB `WriteBatch`, and opaque D3 grouping are non-equivalent diagnostic forms. Different D-modes are separate families and never winner/loser comparisons.

## 3. Frozen statistical analysis specification (`EXP-0001-R8/analysis-v1`)

### 3.1 Unit, pairing, estimands, and estimators

The independent unit is one independently prepared repetition, never an operation. There are exactly **12 complete paired repetitions** per cell with a fixed stopping rule: attempt the predeclared 12 pair identities and stop; do not add pairs because of observed effects or precision. Within each pair, subject and baseline share the stream, preparation, environment, and temporal block.

For each repetition, event/byte throughput is total eligible events/bytes divided by measured elapsed seconds. Lifecycle quantiles use the empirical nearest-rank order statistic and maximum uses the observed maximum. CPU/event, allocation/event, bytes/event, amplification, recovery rates, and recovery durations use their declared totals and denominators. The paired effect is `log(subject/baseline)` for strictly positive higher-is-better throughput/rate metrics and `log(baseline/subject)` for strictly positive lower-is-better latency, resource, amplification, and recovery-duration metrics, so positive always favors the subject. Absolute subject/baseline values, ratios, and arithmetic differences are mandatory. Zero or undefined denominators make the repetition incomplete, not an invented continuity correction. Complexity is reported as raw counts and qualitative findings; it receives no statistical composite.

The reporting target is the population Hodges–Lehmann paired-shift pseudomedian of the paired log effects under the predeclared continuous, symmetric location-shift model. The estimator is the median of the 78 Walsh averages `(x_i + x_j) / 2` for `1 <= i <= j <= 12`. Its confidence set is the exact two-sided 95% interval obtained by inverting the one-sample Wilcoxon signed-rank statistic. The analysis must verify and record whether the paired-effect distribution is plausibly symmetric and whether pairing and repetitions are independent; if the location-shift target or assumptions are not justified, inference fails closed as `inconclusive` and only absolute values and descriptive effects may be reported. It may not silently be described as the ordinary population median or as assumption-free for arbitrary skew.

For an asserted shift `theta`, form `d_i = x_i - theta`; discard exact mathematical zeros, rank nonzero `abs(d_i)` from smallest to largest, assign midranks to exact ties, and sum positive ranks. Enumerate all `2^m` sign assignments of the `m` nonzero ranked values, retaining the fixed midranks, to obtain exact attainable tail probabilities. Report `m`, zeros, tie groups, attainable coverage, and interval endpoint inclusion. A zero or tie caused only by finite measurement representation must remain exact under the frozen representation; no jitter is allowed. If the exact tie-aware enumeration or interval inversion cannot be reproduced, the result is `inconclusive`. Recovery uses the same method only when all repetitions share one identical validated fault cell; fault types are never pooled. Maxima are descriptive and receive no population-maximum claim. Qualitative complexity has no interval.

### 3.2 Tail support, margin hypotheses, multiplicity, and decisions

Each repetition/lifecycle must retain at least 10,000 eligible observations for p99 and 100,000 for p99.9. The 1,000,000-operation extent exceeds both floors; omissions can still make a quantile unsupported. Operations are never pooled across repetitions to restore eligibility.

All thresholds are transformed into the positive-favors-subject log-effect orientation before testing. At a boundary `b`, improvement uses `H0: theta <= b` against `H1: theta > b`; avoiding an upper regression boundary uses `H0: theta <= b` against `H1: theta > b` after orientation; and demonstrating effect at or beyond an unfavorable boundary uses `H0: theta >= b` against `H1: theta < b`. Boundary equality belongs to the null. Practical equivalence `[L,U]` uses two one-sided tests, `H0L: theta <= L` versus `theta > L` and `H0U: theta >= U` versus `theta < U`, and requires both. Each p-value is the exact shifted signed-rank tail from section 3.1; its null, alternative, boundary, tail, raw value, and adjusted value are recorded. This specification tests practical margins, never merely zero effect.

One confirmatory family is one mode × baseline row × claim × the five workload cells × its decision metrics; recovery fault types are separate families. Apply Holm's step-down procedure at family-wise alpha 0.05 to the complete set of one-sided margin p-values needed by the proposed classifications in that family, ordering raw p-values ascending with stable tie order `(cell ID, metric ID, boundary ID)`. Report every ordered hypothesis and Holm adjusted p-value `max_{j<=i}((M-j+1)p_(j))`, capped at one. A classification requires both (a) rejection of every necessary margin null after Holm adjustment and (b) the ordinary exact two-sided 95% Wilcoxon interval lying wholly inside that decision region. This deliberately conservative dual rule supplies an unambiguous interval and Holm family-wise control; no unspecified “Holm simultaneous interval” is used. Any interval endpoint equal to a region boundary is inside only where section 4 declares that boundary inclusive. An interval crossing or touching an excluded boundary, any failed necessary test, or incompatible attainable precision is `inconclusive`; a point estimate never breaks the tie.

### 3.3 Deterministic order and failure handling

Pair IDs are 1–12. For odd pairs baseline runs first; for even pairs subject runs first. Within each pair member, start from the section 2.3 table's 40 cell IDs in displayed row order and F1, F2, F3, ME, MW order. Let zero-based pair index be `q = pair_id - 1`: rotate the list left by `(7*q) mod 40`, then reverse the entire rotated list when `q` is odd. Both members use this identical cell order. The manifest records `schedule_version=EXP-0001-R8/order-v1`, pair ID, member order, rotation, reversal flag, and the materialized 40 IDs. This counterbalances member order and deterministically spreads starting positions without depending on a future generator or BLK-006/007.

Warm-up is excluded by role. A setup-validation failure, instrumentation loss/truncation, identity mismatch, interruption before fixed extent, or declared apparatus failure is a procedural failure. Preserve it, mark it invalid/incomplete, and run at most one replacement with a new linked identity using the same planned position; replacement is allowed without inspecting direction. If replacement also fails or a pair member is missing, the cell is incomplete and inconclusive. Correctness failure is retained and invalidates performance; it is never replaced to seek a favorable result. Uncertain outcomes remain uncertain and inconclusive. Valid extreme values remain included. No silent deletion, winsorization, optional stopping, favorable rerun, post-hoc threshold, or observed-result-driven expansion is allowed.

Reports enumerate every included, excluded, failed, replacement, and missing identity; absolute estimates, paired effects, intervals, raw and Holm-adjusted margin tests, dispersion (median and MAD of repetition values), maxima, counts, gates, and threshold outcomes are mandatory.

### 3.4 Primary statistical sources and rejected alternatives

Selections were made on 2026-08-27 from primary/original sources: Wilcoxon's signed-rank paper (1945), Hodges and Lehmann's estimator paper (1963), Holm's sequentially rejective procedure (1979), and Fisher's randomized-design text (1935). The repository's tail-support rule remains controlling. These sources justify methods, not practical-value thresholds.

- Frank Wilcoxon, “Individual Comparisons by Ranking Methods,” *Biometrics Bulletin* 1(6), 1945, DOI [10.2307/3001968](https://doi.org/10.2307/3001968).
- J. L. Hodges Jr. and E. L. Lehmann, “Estimates of Location Based on Rank Tests,” *Annals of Mathematical Statistics* 34(2), 1963, DOI [10.1214/aoms/1177704172](https://doi.org/10.1214/aoms/1177704172).
- Sture Holm, “A Simple Sequentially Rejective Multiple Test Procedure,” *Scandinavian Journal of Statistics* 6(2), 1979, [JSTOR 4615733](https://www.jstor.org/stable/4615733).
- R. A. Fisher, *The Design of Experiments*, 1935, [Internet Archive record](https://archive.org/details/designofexperime00fish).

Bootstrap intervals were rejected because an exact paired rank procedure is available at the fixed small repetition count; an arithmetic-mean t interval was rejected as unnecessarily distribution-sensitive; Bonferroni was rejected because Holm controls the same family-wise error without being uniformly less powerful; random stopping and sequential looks were rejected because they permit result-driven sampling.

## 4. Frozen threshold registry (`EXP-0001-R8/thresholds-v1`)

All ratios are `subject/baseline`. The following exact linkage makes applicability explicit. `T-THR`, `T-LAT`, `T-RES`, and `T-CPLX` apply separately to every one of the five named cells in every row. `T-REC` is N/A for D0/D1 and applies only to each exact D2/D3 cell after that cell's fault profile and apparatus are validated; until then its recovery conclusion is `unsupported`, not inherited silently.

| Exact set | Exact linked cells | T-THR/LAT/RES/CPLX | T-REC |
|---|---|---|---|
| D0/B0 | `PC-D0-B0-F1`, `PC-D0-B0-F2`, `PC-D0-B0-F3`, `PC-D0-B0-ME`, `PC-D0-B0-MW` | applicable separately | N/A: no recovery claim |
| D1/B1 | `PC-D1-B1-F1`, `PC-D1-B1-F2`, `PC-D1-B1-F3`, `PC-D1-B1-ME`, `PC-D1-B1-MW` | applicable separately | N/A: no recovery claim |
| D1/B2 | `PC-D1-B2-F1`, `PC-D1-B2-F2`, `PC-D1-B2-F3`, `PC-D1-B2-ME`, `PC-D1-B2-MW` | applicable separately | N/A: no recovery claim |
| D1/B3 | `PC-D1-B3-F1`, `PC-D1-B3-F2`, `PC-D1-B3-F3`, `PC-D1-B3-ME`, `PC-D1-B3-MW` | applicable separately | N/A: no recovery claim |
| D2/B1 | `PC-D2-B1-F1`, `PC-D2-B1-F2`, `PC-D2-B1-F3`, `PC-D2-B1-ME`, `PC-D2-B1-MW` | applicable separately | applicable separately only after exact fault validation; currently apparatus-unsupported |
| D2/B2 | `PC-D2-B2-F1`, `PC-D2-B2-F2`, `PC-D2-B2-F3`, `PC-D2-B2-ME`, `PC-D2-B2-MW` | applicable separately | applicable separately only after exact fault validation; currently apparatus-unsupported |
| D2/B3 | `PC-D2-B3-F1`, `PC-D2-B3-F2`, `PC-D2-B3-F3`, `PC-D2-B3-ME`, `PC-D2-B3-MW` | applicable separately | applicable separately only after exact fault validation; currently apparatus-unsupported |
| D3/B1 | `PC-D3-B1-F1`, `PC-D3-B1-F2`, `PC-D3-B1-F3`, `PC-D3-B1-ME`, `PC-D3-B1-MW` | applicable separately | applicable separately only after exact grouping/fault validation; currently apparatus-unsupported |

| Entry | Decision metric and boundary | Meaningful improvement | Equivalence | Regression classification |
|---|---|---|---|---|
| T-THR | successful events/s over measured caller-entry→ack interval; higher is better | ratio `>= 1.10` | `0.95 <= ratio <= 1.05` | regression when ratio `< 0.95` |
| T-LAT | p50/p90/p95/p99/eligible p99.9 for every declared caller-entry→own-ack lifecycle interval; lower is better | ratio `<= 0.90` | `0.95 <= ratio <= 1.05` | regression when ratio `> 1.05`; maximum is descriptive only |
| T-RES | separately: CPU/event, allocations/event, allocated bytes/event, peak RSS, physical bytes/event, and write amplification; lower is better | ratio `<= 0.90` | `0.95 <= ratio <= 1.05` | unacceptable when ratio `> 1.10`; `(1.05, 1.10]` is constrained/non-equivalent, not automatically unacceptable |
| T-REC-rate | scan/replay or other declared recovery rate; higher is better | ratio `>= 1.10` | `0.95 <= ratio <= 1.05` | unacceptable when ratio `< 0.90` |
| T-REC-duration | scan, replay, and time-to-ready durations at exact recovery endpoints; lower is better | ratio `<= 0.90` | `0.95 <= ratio <= 1.05` | unacceptable when ratio `> 1.10` |
| T-CPLX | scoped counts and named qualitative findings, each independent | no numeric boundary | no numeric boundary | no composite score or weighted compensation; every material increase requires explicit review |

Logical payload MiB/s and encoded MiB/s are mandatory absolute reporting boundaries coupled to event throughput, not independent higher-is-better decision metrics. Encoded bytes/event, physical bytes/event, and write amplification express encoding/storage cost and are reported separately under T-RES where applicable. Requested and synchronized bytes/calls are diagnostic unless the predeclared metric definition makes them a physical-cost decision metric. Instrumentation overhead is also metric-dependent: instrumented/minimal throughput uses the T-THR direction, while instrumented/minimal latency and resource cost use T-LAT/T-RES directions. R7 loss, calibration, provenance, and overhead admissibility remain gates; a failed gate cannot be repaired by a favorable ratio.

All interval boundary semantics above are inclusive exactly where `<=` or `>=` appears and exclusive exactly where `<` or `>` appears. Ratios in gaps between improvement, equivalence, and regression regions are constrained/non-equivalent and must be reported, not forced into another class. Every resource metric is reported independently; no universal resource composite exists.

Performance dimensions may trade against one another only when every affected dimension is reported and this frozen table permits the resulting classification. Correctness, durability semantics, deterministic replay, recovery correctness, evidence integrity, and benchmark admissibility are absolute gates and can never be compensated by performance. Correct recovery is an absolute prerequisite to any T-REC classification. Complexity cannot be hidden by performance gains: any material increase receives explicit review, with scoped counts and named findings retained independently.

This registry is prospective only. Any revision creates a new immutable threshold version and a new series; it never retroactively classifies prior evidence.

## 5. Decision table

Evaluate top to bottom; the first controlling row applies.

| Condition | Outcome |
|---|---|
| Corrupt/substituted/untraceable identity or artifact; failed correctness; unusable measurement | `invalid`; performance diagnostic only |
| Unequal D-mode, failed equivalence, atomic/opaque D3 substitution | `non-equivalent` / `diagnostic only`; never a win |
| Cell is outside the closed registry, a threshold is inapplicable, or an apparatus/evidence gate is not satisfied | `unsupported`; descriptive/exploratory only |
| Required cell, repetition, pair, fault, or eligible tail is incomplete | `inconclusive` for the cell and containing row/claim |
| Exact interval is not wholly within one frozen region, or a required Holm-adjusted margin test fails | `inconclusive` due to excessive uncertainty |
| Interval wholly satisfies improvement and all regression guardrails | `supported within tested conditions` (or `constrained/conditionally supported` when platform/equivalence conditions remain) |
| Interval wholly satisfies a frozen equivalence region | `practically equivalent` |
| Interval wholly crosses an unacceptable-regression/reject boundary | `refuted within tested conditions` |
| Dimensions fall in different allowed and unacceptable regions without an approved compensation rule | `mixed trade-off` |
| No qualifying observation exists | `not tested` |

A material design, platform, effective-setting, stream, grouping, threshold, or analysis change requires a new series. New versions cite their superseded versions and apply prospectively; old evidence retains its original classification.

## 6. Traceability, dispositions, and R9 boundary

This record traces to RQ-002/RQ-003, HYP-0001, REQ-001–010 and REQ-012–014, ADR-0002, all EXP-0000 workload/durability/recovery/interpretation contracts, benchmark methodology/baselines, EXP-0001, and R1–R7.

| Item | Disposition after this record |
|---|---|
| BLK-023 / UNK-008 | Resolved for the threshold decision by prospective owner-approved `thresholds-v1`; no empirical claim |
| BLK-024 | Resolved as documentation design by `analysis-v1`; implementation/tool validation remains open |
| BLK-015/020/022/026/027 | Open and unchanged; owner apparatus, harness, toolchain, code/CI authorization remain absent |
| UNK-014/020/021/022 | Open or narrowed exactly as R4–R7 record; no physical/effective evidence added |
| R8 / R9 | R8 complete as documentation design; R9 is next but not begun or authorized by this record |

Canonical history remains the only authority; evidence records are not project truth. Events are accepted facts, not commands or attempts. Effective, system, durability, observation, sequence/replay, lifecycle, and wall-clock meanings remain distinct. D0/D1 remain provisional; D2/D3 require exact platform and later fault/recovery evidence. Derived state remains rebuildable. Missing, corrupted, lossy, contaminated, incomplete, or untraceable evidence fails closed. This pre-observation plan proves no performance, durability, equivalence, or readiness claim.

## 7. Completion report

- **Bounded matrix:** the 40 stable IDs in section 2.3; no diagnostic cross-product expansion.
- **Thresholds:** prospective `thresholds-v1` freezes the exact owner-approved value judgments, inclusive boundaries, applicability, compensation limits, and non-retroactivity rule; it supplies no empirical claim.
- **Statistics:** 12 fixed paired repetitions, independently prepared runs, paired log effects, Hodges–Lehmann/Wilcoxon 95% intervals, Holm families, deterministic counterbalancing, fixed extents, and fail-closed missing/rerun/outlier rules.
- **Decision behavior:** section 5 covers pass, fail, invalid, unsupported, incomplete, non-equivalent, and excessive uncertainty.
- **Result:** R8 is complete as documentation design; BLK-023/UNK-008 and BLK-024 are resolved for this decision, R9 is the next readiness increment, and implementation remains unauthorized until its separate gate passes.
