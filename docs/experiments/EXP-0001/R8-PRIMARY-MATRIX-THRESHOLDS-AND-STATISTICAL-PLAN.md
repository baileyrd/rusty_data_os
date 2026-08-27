# R8 primary matrix, thresholds, and statistical plan

**Identity:** `EXP-0001-R8/v1`

**Freeze date:** 2026-08-27

**Status:** incomplete at an explicit owner-decision boundary; documentation only
**Repository basis:** `f9d9876cf6599345a2e2244223a530ada2b9a828`

## 1. Scope and honest outcome

This record freezes the smallest candidate primary matrix and the analysis method before observations exist. It does **not** freeze a numeric practical-value registry: no repository authority or cited statistical source supplies the product/engineering value judgments needed to say how much throughput, latency, amplification, recovery, or ingestion complexity is materially better or unacceptably worse. Choosing familiar percentages would be invention.

Therefore BLK-024 is resolved as documentation design, but BLK-023 and UNK-008 remain open. Every cell below is **descriptive pending threshold approval**, not confirmatory. R8 is not complete, R9 is not next, and this record authorizes no implementation, apparatus, execution, benchmark, or durability claim.

### 1.1 Exact owner decision blocker

The accountable owner must approve, for each metric family in section 4, numeric practical-improvement and acceptable-regression bounds (and whether equivalence or non-inferiority is a desired claim), with units, inclusive comparison rules, and an operational/product rationale. The owner must also decide whether any named benefit can compensate for a regression and predeclare that rule. Approval must identify `EXP-0001-R8/thresholds-v1`, its effective future series, approver, and date. Until then every registry entry has approval `blocked`, threshold `unset`, and role `descriptive`; BLK-023 cannot honestly be resolved.

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
| D0 | B0 R5 profile | `PC-D0-B0-F1`, `PC-D0-B0-F2`, `PC-D0-B0-F3`, `PC-D0-B0-ME`, `PC-D0-B0-MW` | provisional process-memory equivalence candidate; never canonical | descriptive pending thresholds and validation |
| D1 | B1 R5 D1 | `PC-D1-B1-F1`, `PC-D1-B1-F2`, `PC-D1-B1-F3`, `PC-D1-B1-ME`, `PC-D1-B1-MW` | provisional OS-buffer equivalence candidate | descriptive pending thresholds and validation |
| D1 | B2 SQLite 3.53.4 R6 D1 | `PC-D1-B2-F1`, `PC-D1-B2-F2`, `PC-D1-B2-F3`, `PC-D1-B2-ME`, `PC-D1-B2-MW` | externally classified provisional D1; effective settings required | descriptive pending thresholds and validation |
| D1 | B3 RocksDB 11.8.1 R6 D1 | `PC-D1-B3-F1`, `PC-D1-B3-F2`, `PC-D1-B3-F3`, `PC-D1-B3-ME`, `PC-D1-B3-MW` | externally classified provisional D1; effective settings required | descriptive pending thresholds and validation |
| D2 | B1 R5 D2 | `PC-D2-B1-F1`, `PC-D2-B1-F2`, `PC-D2-B1-F3`, `PC-D2-B1-ME`, `PC-D2-B1-MW` | conditional on the exact R4/R5 platform and fault contract | descriptive/unsupported until BLK-015/022 and thresholds pass |
| D2 | B2 SQLite 3.53.4 R6 D2 | `PC-D2-B2-F1`, `PC-D2-B2-F2`, `PC-D2-B2-F3`, `PC-D2-B2-ME`, `PC-D2-B2-MW` | conditional on VFS/platform equivalence and recovery evidence | descriptive/unsupported until BLK-015/022 and thresholds pass |
| D2 | B3 RocksDB 11.8.1 R6 D2 | `PC-D2-B3-F1`, `PC-D2-B3-F2`, `PC-D2-B3-F3`, `PC-D2-B3-ME`, `PC-D2-B3-MW` | conditional on sync/platform equivalence and recovery evidence | descriptive/unsupported until BLK-015/022 and thresholds pass |
| D3 | B1 R5 controlled D3 | `PC-D3-B1-F1`, `PC-D3-B1-F2`, `PC-D3-B1-F3`, `PC-D3-B1-ME`, `PC-D3-B1-MW` | conditional on observable exact membership, shared sync/outcome, individual acknowledgement, platform and fault contract | descriptive/unsupported until grouping, BLK-015/022, and thresholds pass |

This is exactly 40 unique cells: 5 D0, 15 D1, 15 D2, and 5 D3. Completeness means all five cells and all 12 valid independent pairs in every enabled row complete every applicable claim and gate; no partial row supports a row-level claim. A baseline may be blocked or not tested, but never silently omitted. A claim is only cell-specific unless every required row/cell meets the rule.

P0/P4 fixed boundary runs, P5, all-zero/compressible content, nonminimal envelopes, temporal diagnostics, producer/concurrency sweeps, queue-depth sweeps, cache experiments, and unplanned fault conditions are diagnostic/exploratory and outside this registry. SQLite atomic transactions, RocksDB `WriteBatch`, and opaque D3 grouping are non-equivalent diagnostic forms. Different D-modes are separate families and never winner/loser comparisons.

## 3. Frozen statistical analysis specification (`EXP-0001-R8/analysis-v1`)

### 3.1 Unit, pairing, estimands, and estimators

The independent unit is one independently prepared repetition, never an operation. There are exactly **12 complete paired repetitions** per cell with a fixed stopping rule: attempt the predeclared 12 pair identities and stop; do not add pairs because of observed effects or precision. Within each pair, subject and baseline share the stream, preparation, environment, and temporal block.

For each repetition, event/byte throughput is total eligible events/bytes divided by measured elapsed seconds. Lifecycle quantiles use the empirical nearest-rank order statistic and maximum uses the observed maximum. CPU/event, allocation/event, bytes/event, amplification, recovery rates, and recovery durations use their declared totals and denominators. The paired effect is `log(subject/baseline)` for strictly positive throughput/rate metrics (positive favors subject) and `log(baseline/subject)` for strictly positive latency, resource, amplification, and recovery-duration metrics (positive favors subject). Absolute subject/baseline values and arithmetic differences are also mandatory. Zero or undefined denominators make the repetition incomplete, not an invented continuity correction. Complexity is reported as raw counts and qualitative findings; it receives no statistical composite.

The cell estimator is the Hodges–Lehmann one-sample location estimator of the 12 paired log effects (median of all Walsh averages). Its interval is the exact, distribution-free two-sided **95% confidence interval** obtained by inversion of the Wilcoxon signed-rank statistic; ties/zero differences require an exact tie-aware implementation identified in the future analysis record. This robust paired estimator was chosen before observations because run-level performance effects may be skewed. Arithmetic means, pooled-operation tests, unpaired tests when a valid pair exists, normal-theory selection after inspection, and p-value-only conclusions are rejected.

Recovery uses the same estimator only when all repetitions share an identical validated fault cell; fault types are never pooled. Maxima are descriptive and receive no population-maximum claim. Qualitative complexity has no interval.

### 3.2 Tail support, multiplicity, and decision precision

Each repetition/lifecycle must retain at least 10,000 eligible observations for p99 and 100,000 for p99.9. The 1,000,000-operation extent exceeds both floors; omissions can still make a quantile unsupported. Operations are never pooled across repetitions to restore eligibility.

One confirmatory family is one mode × baseline row × claim × the five workload cells × its metrics. Holm's step-down procedure controls family-wise error at 0.05 across the two-sided interval/test decisions in that family; report both unadjusted and Holm-adjusted results. Modes, baselines, claims, series, and fault types are not pooled. The interval used for a threshold decision is the simultaneous interval corresponding to the Holm decision; if the implementation cannot provide that inversion unambiguously, the result is inconclusive and a new pre-observation analysis version is required.

Precision is sufficient only when the adjusted interval lies wholly inside one frozen decision region. Any interval crossing a practical-improvement, equivalence/non-inferiority, zero, or regression boundary is `inconclusive`; a point estimate never breaks the tie. With thresholds unset, every threshold outcome is necessarily unsupported.

### 3.3 Deterministic order and failure handling

Pair IDs are 1–12. For odd pairs baseline runs first; for even pairs subject runs first. Pair order is the deterministic Fisher–Yates permutation produced from the section 2.2 seed by the future frozen cross-platform generator; BLK-006/007 currently prevents materializing that order, so execution is blocked. Preparation/reset is repeated before each member, while paired cache/storage-state class remains identical.

Warm-up is excluded by role. A setup-validation failure, instrumentation loss/truncation, identity mismatch, interruption before fixed extent, or declared apparatus failure is a procedural failure. Preserve it, mark it invalid/incomplete, and run at most one replacement with a new linked identity using the same planned position; replacement is allowed without inspecting direction. If replacement also fails or a pair member is missing, the cell is incomplete and inconclusive. Correctness failure is retained and invalidates performance; it is never replaced to seek a favorable result. Uncertain outcomes remain uncertain and inconclusive. Valid extreme values remain included. No silent deletion, winsorization, optional stopping, favorable rerun, post-hoc threshold, or observed-result-driven expansion is allowed.

Reports enumerate every included, excluded, failed, replacement, and missing identity; absolute estimates, paired effects, adjusted intervals, dispersion (median and MAD of repetition values), maxima, counts, gates, and threshold outcomes are mandatory.

### 3.4 Primary statistical sources and rejected alternatives

Selections were made on 2026-08-27 from primary/original sources: Wilcoxon's signed-rank paper (1945), Hodges and Lehmann's estimator paper (1963), Holm's sequentially rejective procedure (1979), and Fisher's randomized-design text (1935). The repository's tail-support rule remains controlling. These sources justify methods, not practical-value thresholds.

- Frank Wilcoxon, “Individual Comparisons by Ranking Methods,” *Biometrics Bulletin* 1(6), 1945, DOI [10.2307/3001968](https://doi.org/10.2307/3001968).
- J. L. Hodges Jr. and E. L. Lehmann, “Estimates of Location Based on Rank Tests,” *Annals of Mathematical Statistics* 34(2), 1963, DOI [10.1214/aoms/1177704172](https://doi.org/10.1214/aoms/1177704172).
- Sture Holm, “A Simple Sequentially Rejective Multiple Test Procedure,” *Scandinavian Journal of Statistics* 6(2), 1979, [JSTOR 4615733](https://www.jstor.org/stable/4615733).
- R. A. Fisher, *The Design of Experiments*, 1935, [Internet Archive record](https://archive.org/details/designofexperime00fish).

Bootstrap intervals were rejected because an exact paired rank procedure is available at the fixed small repetition count; an arithmetic-mean t interval was rejected as unnecessarily distribution-sensitive; Bonferroni was rejected because Holm controls the same family-wise error without being uniformly less powerful; random stopping and sequential looks were rejected because they permit result-driven sampling.

## 4. Threshold registry skeleton (`EXP-0001-R8/thresholds-v1-blocked`)

Each of the 40 cells inherits the following entries; inheritance is permitted only because metric definitions and rationale needs are identical, while the eventual numeric values may differ by mode, baseline, and workload. No blank means zero.

| Entry | Metric/boundary; beneficial direction | Improvement / regression / equivalence rule | Gates and rationale | Approval/version |
|---|---|---|---|---|
| T-THR | events/s and logical/encoded MiB/s over measured caller-entry→ack interval; higher | **unset**; future bounds expressed as subject/baseline ratios with explicitly inclusive limits | correctness, durability, equivalence, completeness, evidence quality, instrumentation overhead | blocked; `thresholds-v1-blocked` |
| T-LAT | p50/p90/p95/p99/eligible p99.9 caller-entry→own-ack; lower | **unset**; future bounds as subject/baseline ratios; maximum descriptive | same gates; D3 includes formation wait | blocked; `thresholds-v1-blocked` |
| T-RES | CPU/event, allocations/bytes/event, peak RSS, physical bytes and amplification; lower | **unset** separately per metric; no universal percentage | same gates; exact numerator/denominator required | blocked; `thresholds-v1-blocked` |
| T-REC | scan/replay rate higher; scan/replay/time-to-ready lower; exact recovery endpoints | **unset** separately per validated fault cell | absolute correctness, platform survival, complete apparatus/fault coverage | blocked; `thresholds-v1-blocked` |
| T-CPLX | scoped counts lower and named qualitative rules | **unset**; no weighted score or performance compensation | complete provenance, counting method, observer and limitations | blocked; `thresholds-v1-blocked` |

Instrumentation loss or overhead outside the R7 predeclared acceptable state blocks the cell rather than adjusting its threshold. Any approved threshold revision creates a new immutable version and new series; it never retroactively reclassifies evidence. The owner approval described in section 1.1 is the only path from this skeleton to a confirmatory registry.

## 5. Decision table

Evaluate top to bottom; the first controlling row applies.

| Condition | Outcome |
|---|---|
| Corrupt/substituted/untraceable identity or artifact; failed correctness; unusable measurement | `invalid`; performance diagnostic only |
| Unequal D-mode, failed equivalence, atomic/opaque D3 substitution | `non-equivalent` / `diagnostic only`; never a win |
| Cell is outside the closed registry or threshold is unset/unapproved | `unsupported`; descriptive/exploratory only |
| Required cell, repetition, pair, fault, or eligible tail is incomplete | `inconclusive` for the cell and containing row/claim |
| Admissible adjusted interval crosses any frozen decision boundary | `inconclusive` due to excessive uncertainty |
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
| BLK-023 / UNK-008 | **Open:** exact owner decision in section 1.1; no values invented |
| BLK-024 | Resolved as documentation design by `analysis-v1`; implementation/tool validation remains open |
| BLK-015/020/022/026/027 | Open and unchanged; owner apparatus, harness, toolchain, code/CI authorization remain absent |
| UNK-014/020/021/022 | Open or narrowed exactly as R4–R7 record; no physical/effective evidence added |
| R8 / R9 | R8 incomplete; R9 not next and not begun |

Canonical history remains the only authority; evidence records are not project truth. Events are accepted facts, not commands or attempts. Effective, system, durability, observation, sequence/replay, lifecycle, and wall-clock meanings remain distinct. D0/D1 remain provisional; D2/D3 require exact platform and later fault/recovery evidence. Derived state remains rebuildable. Missing, corrupted, lossy, contaminated, incomplete, or untraceable evidence fails closed. This pre-observation plan proves no performance, durability, equivalence, or readiness claim.

## 7. Completion report

- **Bounded matrix:** the 40 stable IDs in section 2.3; no diagnostic cross-product expansion.
- **Thresholds:** no consequential value is supportable from repository authority or statistical sources; the exact owner approval is recorded rather than guessed.
- **Statistics:** 12 fixed paired repetitions, independently prepared runs, paired log effects, Hodges–Lehmann/Wilcoxon 95% intervals, Holm families, deterministic counterbalancing, fixed extents, and fail-closed missing/rerun/outlier rules.
- **Decision behavior:** section 5 covers pass, fail, invalid, unsupported, incomplete, non-equivalent, and excessive uncertainty.
- **Result:** R8 is **not complete**; BLK-023 and UNK-008 remain open, BLK-024 is resolved as design, and no implementation or R9 action is authorized.
