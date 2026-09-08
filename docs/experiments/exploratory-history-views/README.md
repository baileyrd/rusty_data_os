# Exploratory History-to-Views Experiment

**Classification:** bounded exploratory observation; not confirmatory EXP-0001 evidence and not an architecture decision.

## Authority and question

The owner authorization, “Let’s stop the BS and just test the theory,” authorizes this one code PR to implement and execute the bounded workload despite the earlier workload-execution restrictions. It does not authorize other workloads, durability claims, production code, or a numbered planning cycle. The question is whether one reopened append-only history can independently reconstruct correct row-oriented and column-oriented final and midpoint states, and what the measured component costs are.

`EXPLORATORY-HV1/entity-time-value-v1` is a separate experimental payload containing entity ID, logical event time, and integer value. It is retained in valid structural RF1 provisional records, but it is not SOP1, SOP2, or an approved benchmark cell. Seventeen entities receive deterministic, repeated updates; logical time intentionally does not always increase with replay sequence.

## Predetermined method

For 100, 1,000, and 10,000 events, input generation occurs outside all timed regions. One warm-up precedes five measured trials. `std::time::Instant` separately measures RF1 encoding, ordinary-write append, physical reopen/replay, row construction, and column construction. Direct construction of each view from the same prepared `(sequence, event)` input is measured separately as a computational baseline; it has no file-storage or replay semantics and is not an equivalent database comparison. The row and column builders each apply events directly and neither materializes or calls the other view. The untimed expected-state oracle remains separate from both timed baselines.

Each run requires a newly created output directory. Metadata, trial, failure, and initially-incomplete summary files are exclusively created before trials; every completed observation is flushed immediately, and a failed trial is recorded with its identity and reason before a nonzero exit. A trial is valid only when every independently computed expected entity/value/time/sequence agrees at midpoint and final cutoffs, both views agree with those independent expectations, and physical replay has the exact record count, ordinal order, retained bytes, offsets, extents, byte count, and clean EOF. Focused tests also include a fixed hand-written oracle, malformed payload rejection, and output-directory collision rejection.

## Corrected observation

The corrected run independently applied decoded events to separate row and column state. All 15 measured trials and all three warm-ups passed every correctness gate. Thus, in this bounded payload and implementation, both independently rebuilt views and the midpoint historical state matched. This supports feasibility only; it cannot establish that HYP-0001 has acceptable general performance or complexity.

Times below are median milliseconds and corresponding throughput from five samples. Parentheses show the observed time range. These samples do not justify tail-latency claims.

| Events | Encode | Append | Physical replay | Row rebuild | Column rebuild | Direct row | Direct column |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 0.035 (0.034–0.062), 2.88M ev/s | 0.117 (0.095–0.197), 858K ev/s | 0.130 (0.108–1.341), 770K ev/s | 0.0043 (0.0037–0.0053), 23.2M ev/s | 0.0071 (0.0070–0.0084), 14.1M ev/s | 0.0018 (0.0017–0.0020), 54.2M ev/s | 0.0037 (0.0035–0.0043), 26.9M ev/s |
| 1,000 | 0.319 (0.317–1.661), 3.14M ev/s | 0.752 (0.732–0.843), 1.33M ev/s | 6.953 (6.249–8.390), 144K ev/s | 0.029 (0.027–0.065), 34.8M ev/s | 0.029 (0.028–0.038), 34.1M ev/s | 0.012 (0.012–0.012), 83.5M ev/s | 0.013 (0.013–0.041), 75.5M ev/s |
| 10,000 | 3.262 (3.217–3.508), 3.07M ev/s | 6.933 (6.877–11.876), 1.44M ev/s | 547.478 (529.186–589.000), 18.3K ev/s | 0.372 (0.286–0.478), 26.9M ev/s | 0.280 (0.254–0.315), 35.8M ev/s | 0.111 (0.110–0.133), 90.2M ev/s | 0.102 (0.101–0.121), 98.4M ev/s |

Physical replay remained the largest observed cost and degraded sharply with event count in the corrected run; profiling its existing validation path is a follow-up question, not a conclusion from these samples. The run used runtime-observed Rust 1.89.0 with an optimized (`debug_assertions=false`) Linux x86_64 binary in a container-like environment on an Intel Xeon Platinum 8272CL with an overlay filesystem. Cargo did not embed the profile name, target triple, overflow-check, locked, or offline settings, so the metadata marks those build facts unknown rather than inferring them from the invocation. Cache state, co-tenancy, CPU frequency, mount details, storage hardware, and performance mode were uncontrolled or unavailable. Append is D1 ordinary-write submission only: there is no `fsync`, stable-storage, crash-survival, recovery, or durable-canonical-commit claim. Page-cache effects are uncontrolled. Perf counters were optional and not collected.

Machine-readable corrected warm-up, trial, failure, and environment records are retained in [`results/container-20260908-corrected-independent-columns/`](results/container-20260908-corrected-independent-columns/). The original result set is preserved at [`results/container-20260908-superseded-dependent-columns/`](results/container-20260908-superseded-dependent-columns/) and is **superseded for independent-column comparison** because its column builder transposed row materialization rather than applying history independently. Its useful observation that physical replay dominated at 10,000 events is preserved and was observed again in the corrected run. The exclusive run directory, including RF1 files, was retained outside the repository; no private path is published here.

## Reproduce on David's Linux machine

From the repository root, choose a path that does not already exist:

```bash
cargo +1.89.0 run --release --manifest-path experiments/exp-0001/Cargo.toml --locked --offline --package exp1-descriptive-d1-harness --example history_views -- "$HOME/rusty-data-os-history-views-$(date -u +%Y%m%dT%H%M%SZ)"
```
