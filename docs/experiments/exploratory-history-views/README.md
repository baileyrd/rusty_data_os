# Exploratory History-to-Views Experiment

**Classification:** bounded exploratory observation; not confirmatory EXP-0001 evidence and not an architecture decision.

## Authority and question

The owner authorization, “Let’s stop the BS and just test the theory,” authorizes this one code PR to implement and execute the bounded workload despite the earlier workload-execution restrictions. It does not authorize other workloads, durability claims, production code, or a numbered planning cycle. The question is whether one reopened append-only history can independently reconstruct correct row-oriented and column-oriented final and midpoint states, and what the measured component costs are.

`EXPLORATORY-HV1/entity-time-value-v1` is a separate experimental payload containing entity ID, logical event time, and integer value. It is retained in valid structural RF1 provisional records, but it is not SOP1, SOP2, or an approved benchmark cell. Seventeen entities receive deterministic, repeated updates; logical time intentionally does not always increase with replay sequence.

## Predetermined method

For 100, 1,000, and 10,000 events, input generation occurs outside all timed regions. One warm-up precedes five measured trials. `std::time::Instant` separately measures RF1 encoding, ordinary-write append, physical reopen/replay, row construction, and column construction. Direct construction of each view from the identical in-memory events is measured separately as a computational baseline; it has no file-storage or replay semantics and is not an equivalent database comparison.

Each run requires a newly created output directory. A trial is valid only when every independently computed expected entity/value/time/sequence agrees at midpoint and final cutoffs, both views agree with those independent expectations, and physical replay has the exact record count, ordinal order, retained bytes, offsets, extents, byte count, and clean EOF. Focused tests also include a fixed hand-written oracle, malformed payload rejection, and output-directory collision rejection.

## Observed result

All 15 measured trials and all three warm-ups passed every correctness gate. Thus, in this bounded payload and implementation, both independently rebuilt views and the midpoint historical state matched. This supports feasibility only; it cannot establish that HYP-0001 has acceptable general performance or complexity.

Times below are median milliseconds and corresponding throughput from five samples. Parentheses show the observed time range. These samples do not justify tail-latency claims.

| Events | Encode | Append | Physical replay | Row rebuild | Column rebuild | Direct row | Direct column |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 0.032 (0.032–0.036), 3.10M ev/s | 0.098 (0.088–0.218), 1.02M ev/s | 0.093 (0.065–0.139), 1.08M ev/s | 0.0035 (0.0032–0.0055), 28.9M ev/s | 0.0036 (0.0035–0.0054), 27.8M ev/s | 0.0015 (0.0014–0.0025), 68.5M ev/s | 0.0018 (0.0017–0.0030), 55.6M ev/s |
| 1,000 | 0.285 (0.283–0.330), 3.51M ev/s | 0.761 (0.714–0.843), 1.31M ev/s | 5.526 (4.988–5.948), 181K ev/s | 0.027 (0.026–0.051), 37.4M ev/s | 0.026 (0.026–0.027), 38.4M ev/s | 0.010 (0.010–0.011), 95.5M ev/s | 0.010 (0.010–0.011), 96.2M ev/s |
| 10,000 | 3.161 (2.985–4.102), 3.16M ev/s | 6.678 (6.512–7.656), 1.50M ev/s | 459.493 (442.620–497.127), 21.8K ev/s | 0.311 (0.291–0.493), 32.1M ev/s | 0.260 (0.257–0.305), 38.4M ev/s | 0.125 (0.101–0.166), 80.2M ev/s | 0.097 (0.097–0.117), 103M ev/s |

Physical replay was the largest observed cost and degraded sharply with event count in this run; profiling its existing validation path is a follow-up question, not a conclusion from these samples. The run used Rust 1.89.0 release builds in a Linux x86_64 container-like environment on an Intel Xeon Platinum 8370C with an overlay filesystem. Cache state, co-tenancy, CPU frequency, mount details, storage hardware, and performance mode were uncontrolled or unavailable. Append is D1 ordinary-write submission only: there is no `fsync`, stable-storage, crash-survival, recovery, or durable-canonical-commit claim. Page-cache effects are uncontrolled. Perf counters were optional and not collected.

Machine-readable warm-up and trial observations are retained in [`results/container-20260908/trials.csv`](results/container-20260908/trials.csv); captured facts and explicit unavailable states are in [`results/container-20260908/environment.txt`](results/container-20260908/environment.txt). The exclusive run directory, including RF1 files, was retained outside the repository; no private path is published here.

## Reproduce on David's Linux machine

From the repository root, choose a path that does not already exist:

```bash
cargo run --release --manifest-path experiments/exp-0001/Cargo.toml --locked --offline --package exp1-descriptive-d1-harness --example history_views -- "$HOME/rusty-data-os-history-views-$(date -u +%Y%m%dT%H%M%SZ)"
```
