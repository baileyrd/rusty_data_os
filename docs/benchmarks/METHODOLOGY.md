# Benchmark Methodology

Rusty Data OS is explicitly fact-based and data-driven. Benchmark methodology is therefore part of the architecture process, not an afterthought.

## 1. General rule

A benchmark result is evidence only when another contributor can understand what was measured, under what semantics, and under what environment.

## 2. Equivalence before comparison

Never compare two systems as equivalent when they provide different correctness or durability guarantees.

For write tests, document at least:

- acknowledgement point;
- persistence primitive used;
- whether data survives process crash;
- whether data is intended to survive OS crash/power loss;
- whether batching/group commit is used;
- whether replication is involved.

## 3. Environment capture

Every published benchmark series should record:

- date/time of run;
- host identifier or anonymized test-machine label;
- CPU model, core/thread count, relevant topology;
- RAM size and speed if known;
- storage device model/interface;
- filesystem;
- operating system and kernel/build;
- relevant mount options;
- power/performance mode;
- virtualization/container state;
- Rust toolchain version;
- target triple;
- build profile and compiler flags;
- dependency lockfile/commit;
- repository commit SHA;
- benchmark configuration.

## 4. Repetition and distributions

Do not report only arithmetic mean.

Where sample counts allow, report:

- median/p50;
- p90;
- p95;
- p99;
- p99.9;
- throughput;
- sample count;
- dispersion measure such as standard deviation or median absolute deviation when useful.

Tail latency is especially important for storage systems.

## 5. Warm-up and steady state

Separate warm-up from measured runs.

Document whether a test measures:

- cold start;
- warm cache;
- steady state;
- recovery;
- compaction/materialization interference.

Do not silently mix these conditions.

## 6. Cache effects

Operating-system page cache and device caches can dominate storage results.

Each experiment must state whether caches are intentionally warm, cold, uncontrolled, or part of the workload being studied. Avoid claiming stable-media performance when the measured acknowledgement only reaches volatile caches.

## 7. Workload specification

A workload must declare:

- operation mix;
- key/value or payload sizes;
- distribution (uniform, sequential, Zipfian, etc.);
- concurrency;
- outstanding queue depth;
- data-set size relative to RAM;
- duration or operation count;
- batching parameters;
- read/write ratio;
- preconditioning steps.

For EXP-0001, the [reproducible workload contract](../experiments/EXP-0000/WORKLOADS.md) controls these declarations and the minimal matrix. Results must distinguish opaque payload bytes, encoded event bytes, and physical bytes written, and must report each applicable count/rate rather than using payload size as total record size. Comparable systems consume identical or byte-for-byte reproducible semantic operation streams; durability modes with different guarantees are not equivalent workload cells.

## 8. Isolation of variables

Prefer experiments that change one independent variable at a time.

When multiple factors are intentionally varied, record the complete matrix and use a design that makes interactions interpretable.

## 9. Correctness gating

Benchmark harnesses must run or reference correctness validation. A configuration that fails invariants is reported as invalid, not as a fast result.

## 10. Baseline fairness

Established systems should be configured by documented settings rather than deliberately weak defaults or unrealistic tuning.

If the project cannot make semantics equivalent, report the mismatch explicitly and avoid winner/loser claims.

EXP-0001 follows the [baseline contract](BASELINES.md): every series freezes product/source/build/binding, complete effective configuration, adapter, workload, durability contract, environment, and repository identity. Baseline-native transactions, rows, keys, WAL records, or batches are not presumed to equal canonical events. D0/D1 provisional results are not ranked as equivalent to D2/D3 canonical commit, and atomic or opaque engine grouping is diagnostic unless it satisfies the D3 contract. A material identity or semantic change starts a new series; samples from distinct series are not silently pooled.

## 11. Raw results

Prefer machine-readable raw results in addition to summarized tables/charts.

Raw result files should include enough metadata to map them to:

- repository commit;
- experiment version;
- test environment;
- configuration;
- timestamp.

Large generated data may be stored outside Git if necessary, but a manifest and durable reference should remain in the repository.

## 12. Regression policy

Once a benchmark becomes a recognized baseline, future changes should report both improvement and regression across relevant dimensions. Throughput gains do not automatically justify large tail-latency, memory, recovery, or durability regressions.

## 13. Decision threshold

Do not impose a universal rule such as "must be 10% faster." Different architectural decisions have different trade spaces.

Whenever practical, an experiment should predeclare what outcome would:

- support the hypothesis;
- refute the hypothesis;
- be practically equivalent/noise;
- require further investigation.

The rationale for those thresholds must be documented.
