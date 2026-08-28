# EXP-0001 Evidence Interpretation and Decision Contract

**Contract identity:** `EXP-0001-interpretation/v1`
**Status:** frozen EXP-0000 readiness contract; no evidence or numeric acceptance thresholds recorded

## 1. Purpose and freeze rule

This contract prevents an observed EXP-0001 result from determining how that result is judged. Before confirmatory execution, an execution plan must freeze this contract version, the applicable threshold-registry version, the analysis specification, and every primary cell. A later change creates a new version, gives a reason, and applies only to a new benchmark series; it cannot retroactively improve an interpretation.

This document defines the interpretation framework, not executable choices or evidence. In particular, it does not select hardware, binaries, adapters, encoding, framing, integrity, identity, serialization, timestamps, clocks, harnesses, or quantitative thresholds.

It operationalizes the evidence and correctness governance in [REQ-001 through REQ-014](../REQUIREMENTS.md) for the narrow EXP-0001 contribution to [HYP-0001](../hypotheses/HYP-0001-event-log-as-canonical-state.md). It cannot validate those requirements or resolve that hypothesis without admissible experiment evidence.

## 2. Evidence-admissibility gates

A result may support a performance conclusion only when **all** applicable gates pass:

1. **Identity:** exact environment, workload/stream, requested and effective configuration, adapter, code/build, subject, baseline, and series identities are recorded and immutable.
2. **Record and provenance:** every required field in the [raw-result contract](RAW-RESULT-TEMPLATE.md) is present or has an explicit missing state and reason; artifacts have identities, digests, producers, retention states, and provenance edges.
3. **Correctness:** the applicable invariant and recovery oracle reports `pass`. There is no unexplained loss, duplication, invention, corruption, ordering violation, premature canonical visibility, or false acknowledgement.
4. **Durability:** D0–D3, acknowledgement point, commit status, platform durability contract, and promised fault contract are declared. Every claimed D2/D3 fault class has been tested successfully under that contract.
5. **Equivalence:** the baseline equivalence class and satisfied conditions are recorded. Only equivalent candidates may support like-for-like claims; conditional evidence retains its condition.
6. **Completeness:** all predeclared required primary cells and repetitions are complete. Missing cells remain visible with status and reason.
7. **Population separation:** setup and warm-up are separate from measured observations; phases, roles, cells, and series are not silently pooled.
8. **Measurement state:** instrumentation identity, scope, overhead assessment, loss, truncation, clock limitations, and every missing-data state are recorded.

Gate failures are classified before effects are inspected:

| Condition | Required evidence classification |
|---|---|
| Corrupt/substituted identity or artifacts, undeclared material change, failed correctness, or unusable measurement | `invalid`; performance is diagnostic only |
| Semantic mismatch, including a durability or D3 mismatch | `non-equivalent`; may be diagnostic, never a like-for-like win |
| Useful fault, boundary, stress, or procedural observation outside the confirmatory design | `diagnostic only` |
| Missing required repetitions/cells, uncertain correctness, incomplete fault coverage, or uncertainty too wide for the frozen decision rule | `inconclusive` |

No failed gate can be relabeled as a performance win. Correctness failure overrides speed.

## 3. Analysis populations and observation handling

The execution plan labels every cell before execution:

- **primary confirmatory:** predeclared cells eligible for threshold-based conclusions;
- **boundary** and **stress:** limits and failure onset, reported separately;
- **diagnostic:** mechanism, fault-apparatus, uncontrolled-interleaving, or invalid-run investigation;
- **exploratory:** hypothesis-generating comparisons not covered by the frozen confirmatory family.

A complete independently prepared repetition is the independent comparison unit. Operations within it estimate its latency distribution and counters; millions of operations are not millions of independent experiments. Warm-up observations are excluded from measured estimates by role, while remaining preserved.

Before execution, the analysis specification must define handling of successful, failed, interrupted, partial, uncertain, and outlier repetitions. Procedural failures may trigger a rerun only under a frozen rule (for example, verified instrumentation loss or failed setup validation), irrespective of the observed direction. The original record, identity, failure, and replacement link remain. A valid unfavorable result or statistically unusual value is not a rerun or deletion reason. Robust estimators may limit an outlier's influence, but may not erase its record.

Every exclusion retains the raw record identity, reason, decision time, rule/version, and replacement identity if any. Summaries enumerate incomplete and excluded cells rather than silently dropping them. Concurrent runs with uncontrolled cross-producer interleavings remain diagnostic unless the frozen comparison design establishes a reproducibly equivalent operation set, producer-local order, and valid comparison estimand.

## 4. Required cell reporting

For each interpretable workload/profile/D-mode cell, report where applicable:

- event and byte throughput, with numerator, denominator, interval, and separate logical and encoded byte rates;
- p50, p90, p95, p99, and eligible p99.9 latency for each declared lifecycle interval;
- maximum observed latency, labeled an observation rather than an estimator of a population maximum;
- operation/sample counts, independent repetition counts, dispersion, effect uncertainty, and missing/excluded counts;
- CPU user/system/wall time and utilization, and cycles per operation when supported;
- allocation count/bytes, resident/virtual/peak memory, and measurement scope;
- logical, encoded, requested/synchronized, physical-read, physical-write, and storage bytes;
- read, write, and storage amplification with explicit numerators and denominators;
- synchronization requested/completed/failed counts and synchronization latency;
- recovery scan/replay throughput, scan/replay time, and time-to-ready with endpoints;
- errors, retries, uncertain outcomes, correctness and recovery classifications;
- instrumentation configuration, overhead estimate or bounded assessment, losses, and unavailable fields;
- baseline and subject absolute values, absolute differences, and ratios with uncertainty.

Averages alone are insufficient. Ratios without both absolute values are prohibited.

### 4.1 Tail-quantile eligibility

For a quantile `q`, a repetition is eligible only when its measured population contains at least **100 expected observations in the upper tail**, that is `n × (1 - q) >= 100`, after predeclared omissions. Thus p99 requires at least 10,000 observations and p99.9 at least 100,000; the rule applies independently to each repetition and lifecycle interval. This minimum keeps a reported tail from resting on only a handful of operations, though it does not by itself establish independence or precision. If any repetition used by a cell-level quantile is ineligible, report the lower eligible quantiles and the sample maximum, and mark that quantile `unsupported`; do not synthesize it by pooling operations across repetitions. The frozen analysis plan may require more support, never less.

## 5. Statistical and uncertainty contract

Primary comparisons use identical immutable streams or reproducibly equivalent streams under the workload contract. They use repeated, independently prepared runs. Pair subject and baseline repetitions when the same environment, stream, preparation, and planned temporal block make a pair defensible; otherwise use a predeclared unpaired design. Run order, randomization or counterbalancing, reset policy, and blocking variables are frozen in advance.

The execution plan must freeze, per metric or metric family:

1. estimand and exact estimator;
2. interval method and confidence level (or another named, justified uncertainty interval);
3. independent repetition count and fixed stopping rule;
4. paired/unpaired unit and treatment of incomplete pairs;
5. run-order/randomization policy;
6. transformation and robust summary, if any;
7. multiplicity procedure for the confirmatory comparison family.

Effect estimates and their uncertainty are primary; a p-value alone cannot support a conclusion. Skewed throughput, latency, amplification, and recovery observations require a justified robust summary (for example, a median or a predeclared transformed estimator) rather than an automatically chosen arithmetic mean. Per-operation samples describe within-run distributions and cannot inflate degrees of freedom across repetitions. No significance claim may rely only on enormous operation counts.

Multiplicity is controlled by the frozen family/procedure, or the affected comparisons are explicitly exploratory without confirmatory language. Materially different environments and benchmark series are reported separately and never pooled; a cross-environment synthesis requires its own predeclared estimand and design. There is no universal estimator appropriate to every metric. Reruns follow procedural-failure rules, never result desirability.

## 6. Versioned practical-threshold registry

The EXP-0001 execution plan must freeze a versioned registry before any confirmatory benchmark execution. It contains one resolved entry for every primary metric and workload/profile cell (or an explicit linked entry where the rationale truly applies identically). Each entry records:

| Required field | Meaning |
|---|---|
| Metric and lifecycle interval | Exact measured quantity and endpoints |
| Subject and baseline | Compared implementation/profile and appropriate baseline |
| Cell and semantics | Workload/profile, concurrency/configuration coordinates, and D-mode |
| Direction | Which direction is beneficial |
| Meaningful region | Practical-equivalence bounds or minimum meaningful effect |
| Regression guardrail | Maximum acceptable regression |
| Uncertainty rule | How the interval must relate to bounds for each classification |
| Rationale/evidence | Requirement, operational need, prior evidence, or other justified source |
| Role | Confirmatory or exploratory |
| Compensation | Whether another named benefit may compensate, and its predeclared rule |
| Governance | Owner, freeze time, immutable version/identity, and supersession link |

There is no universal percentage threshold. An unresolved, unsupported, or post-observation entry blocks confirmatory interpretation for that cell; its results may be descriptive or exploratory only. [UNK-008](../ASSUMPTIONS-AND-UNKNOWNS.md) remains open until justified numeric values are frozen; the later R8 owner approval resolves it for `EXP-0001-R8/thresholds-v1`, without changing this framework or classifying prior evidence.

## 7. Outcome classifications

Every conclusion names the exact workload/cell, environment and series, subject, baseline, D-mode, platform and fault contract, metric/lifecycle interval, criteria and threshold versions, analysis version, and included/excluded raw-result sets.

| Classification | Meaning |
|---|---|
| `supported within tested conditions` | Admissible confirmatory evidence meets the frozen support and regression rules for the stated narrow claim. |
| `constrained/conditionally supported` | Support holds only under named equivalence conditions, subset, platform contract, or limitation. |
| `practically equivalent` | Admissible evidence and its uncertainty satisfy the frozen equivalence rule; lack of detected difference is insufficient. |
| `refuted within tested conditions` | Admissible evidence meets the frozen refutation or unacceptable-regression rule. |
| `mixed trade-off` | Admissible dimensions improve and regress without satisfying a predeclared dominance or compensation rule. |
| `diagnostic only` | The observation explains behavior but is outside or fails eligibility for confirmatory inference. |
| `invalid` | Identity, provenance, procedure, measurement, or correctness makes the result unusable for performance inference. |
| `inconclusive` | Admissible information cannot decide the frozen rule, including insufficient precision or incomplete required evidence. |
| `not tested` | No qualifying observation exists for the declared cell or claim. |

Absence of evidence is `not tested` or `inconclusive`, never practical equivalence.

## 8. Multi-metric trade space

Interpret throughput, median latency, each eligible tail, CPU, allocations, memory, read/write/storage amplification, recovery, correctness/durability, and implementation/operational complexity separately. Do not combine them into an unvalidated score.

Report dominance and the Pareto/trade-space frontier. When one material dimension improves and another crosses its frozen regression guardrail, classify the result `mixed trade-off`; a throughput gain cannot hide a material p99, memory, amplification, recovery, correctness, or durability regression. Different durability modes occupy different semantic trade spaces and are not ranked like-for-like.

Any compensating-benefit decision names the dimensions, thresholds, evidence, and accountable decision. It is predeclared where possible and later recorded through the experiment interpretation and, if it affects architecture, an ADR. A benchmark author may not silently infer compensation after seeing results.

## 9. Durability and baseline constraints

- D0 and D1 are provisional and cannot support canonical-durability claims.
- D2 and D3 are canonical only under their recorded platform durability and successfully tested fault contracts.
- Different D-modes are not like-for-like ranks.
- B0 applies only to D0. B1 is the primary primitive D1, D2, and controlled-D3 comparison.
- SQLite and RocksDB D1 are provisional; their D2 comparisons remain conditional on demonstrated platform semantics.
- Atomic multi-event transactions and RocksDB `WriteBatch` are not strict D3 equivalents.
- Opaque group commit remains diagnostic unless observable membership, shared synchronization, acknowledgement, and outcome satisfy D3.
- Any applicable correctness failure overrides speed.

These rules refine, and do not replace, the [baseline](BASELINES.md), [lifecycle/durability](../experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), and [crash/recovery](../experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) contracts.

## 10. Ingestion-path complexity evidence

EXP-0001 records raw counts and separately records qualitative evidence for:

- implementation size (with counting method) and component count;
- direct/transitive dependencies and native-build/toolchain burden;
- unsafe-code locations, size, purpose, and audit status;
- platform-specific code and configuration;
- adapter transformations and semantic impedance;
- runtime modes and user-visible configuration parameters;
- operational preparation, fault response, and recovery steps;
- instrumentation setup and diagnosis burden;
- failure states, retries, and uncertain-outcome handling;
- reproduction steps, maintenance obligations, and known fragility.

Counts retain definitions and scope; qualitative findings retain concrete examples, procedure, observer, and limitations. They are not added together, normalized into an arbitrary weighted score, or used to disguise a correctness deficit. The threshold registry must state any decision-relevant complexity bounds or qualitative acceptance rule before confirmatory use.

EXP-0001 evaluates ingestion-path complexity only. It cannot establish the complexity of future materializations, queries, checkpoints, servers, replication, or distributed operation, and it cannot by itself resolve the architecture-wide complexity clause in HYP-0001.

## 11. Experiment and research decision gate

EXP-0001 may characterize the ingestion trade space; support, constrain, or refute narrower ingestion hypotheses; identify viable or nonviable tested durability modes; justify the next smallest experiment; or require redesign and more evidence.

It may not prove HYP-0001 as a whole, claim that Data OS is generally faster than databases, automatically authorize a production architecture, erase negative evidence, or move experimental code into a core engine without the evidence → ADR → specification path.

Continuation requires admissible evidence and an explicit reviewed decision record. A mixed or negative result may justify a narrower follow-up only when the unresolved falsifiable question and why the smallest follow-up can answer it are recorded. Completion of EXP-0000 is readiness documentation, not permission to implement EXP-0001; the next bounded step is a reviewed EXP-0001 readiness/implementation proposal that resolves its execution blockers.

## 12. Immutable interpretation record

The future experiment conclusion creates an immutable interpretation containing:

- interpretation identity/version, creation time, and exact experiment identity;
- frozen criteria, threshold-registry, execution-plan, and analysis versions;
- every included and excluded raw-result identity, with exclusion reasons;
- analysis method, code/tool identity, configuration, and derived-artifact provenance;
- correctness, durability, fault-coverage, and equivalence status;
- absolute estimates, differences, ratios, dispersion, and uncertainty;
- threshold comparisons and outcome classification per claim/cell;
- Pareto/trade-space and separate complexity findings;
- supported, refuted, constrained, and unresolved claims;
- limitations, validity threats, missing data, and generalization boundary;
- deviations, their timing, cause, impact, and whether they force exploratory status;
- recommended decision and required hypothesis, research-registry, ADR, or specification updates;
- supersession/correction identity and reason where applicable.

Interpretations are never overwritten. A correction or reinterpretation creates a new identity, cites the record it supersedes, preserves both records and raw evidence, and explains the change, consistent with raw-result provenance.
