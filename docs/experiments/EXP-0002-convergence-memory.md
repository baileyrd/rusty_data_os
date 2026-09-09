# EXP-0002 — Shared Memory convergence

## 1. Identifier and title

EXP-0002, shared `Memory@2` application trace experiment, CMT1 revision 1.

## 2. Status

Ready. Implementation and correctness validation are authorized by the owner’s
[merge plan, step 2](../plans/data-os-multimodal-merge-plan-2026-09-08.md).
Measurements are operator initiated; no performance conclusion is available yet.
Candidate: **D1 ordinary writes, no fsync, no crash-survival claim**.

## 3. Linked hypothesis

[HYP-0002](../hypotheses/HYP-0002-memory-convergence.md).

## 4. Research question

Can a bounded ordinary-write RF1 history reproduce the same Memory operations as
the pinned legacy adapter and independently rebuild every field in rows and columns?

## 5. Hypothesis under test

Every operation outcome and query result, and both rebuilt final states, equal
an independent per-field model for all four versioned trace sizes. One mismatch
falsifies correctness for that cell, regardless of speed.

## 6. Independent variables

Engine (candidate or pinned legacy), record count (small, 1K, 10K, 100K), and legacy
dispatch mode (single, protocol-22 pipelined, protocol-22 atomic). Modes are separate
cells. No concurrent writers, relationships, network traffic, or fault injection.

## 7. Controlled variables

Byte-identical CMT1 inputs, seed, operation order, all thirteen Memory fields,
single owner, queue depth one, fresh store per trial, one warm-up followed by five
measured trials. OS/device caches are uncontrolled; fresh files do not mean cold
caches. Generation, oracle evaluation, comparison and result serialization are
outside operation timings.

## 8. Workloads

Small deterministic trace covers duplicate insert, missing identities, field
update (`access_count` only, as supported by legacy), replacement of all fields,
guard success/failure on declared `access_count` and text `status` (field 8),
hard delete and same-ID reinsert. The pinned legacy `update_field` path exposes
only `access_count`, so per-field updates of other fields cannot be exercised
through that path. Small now has 59 operations; the seeded generator also guards
on text status. N5 changes trace hashes and operation mixes relative to prior
host inputs; compare only identical input SHA-256s.
Queries follow each mutation class. Equality is on `category`; aggregation is
count/sum/min/max of `access_count`; pages use ascending `(updated_at_unix_ms, UUID)`
with an exclusive cursor. Soft deletion is data, not an implicit filter.
Seeded SplitMix64 traces insert N distinct UUIDs and repeatedly update, replace,
guard, delete and reinsert each. Content is 200–2000 UTF-8 bytes, metadata JSON
50–500 bytes, and tags number 0–8. Header shape parameters, generator seed, regeneration command and input SHA-256
are retained; the operator keeps custom inputs externally. Strings and tag order are compared byte-for-byte.

## 9. Correctness invariants

Insert cannot overwrite; missing mutations cannot create; failed guards cannot
change state; deletion removes every field; reinsert starts a new full record.
Each operation ID is unique and sequential. Ordered page membership and order
must both agree, including ties. Query digests cover UUID and all fields. Final
record lists and all thirteen column digests must agree. Candidate row and column
builders separately consume replayed history and each compare to the oracle.
Malformed traces fail before store creation. Runtime errors and mismatches are
retained and invalidate the trial; execution continues when the backend permits.

## 10. Benchmark metrics

Predeclared before implementation: monotonic `Instant` nanoseconds around each
complete engine operation, with separately reported `append`, `replay` (only
`reopen_and_replay`: physical frame scan/structural validation), `decode` (CMM1
payload parsing into `Change` values), `rows`, `columns`, and query stages.
`summary.txt` includes measured distributions for each of these stage names. Append is nested inside candidate
operation time and must not be added to it. Result digest computation is outside
operation time. Raw per-op samples remain in the operator output directory and
are referenced by path and SHA-256 in each trial's `throughput.txt`. The committed
evidence subset retains metadata, per-trial totals and summaries, with count, minimum,
p50/p90/p95/p99, maximum and throughput (operations divided by summed operation
time). Five trial samples cannot support tail claims; per-op distributions mix
only the same operation kind. Report logical file lengths, dataset shape, and
process peak RSS (Linux VmHWM / Windows PeakWorkingSet64; process lifetime high
water, includes oracle and retained trace, not isolated engine allocation).

RSS is harness-inclusive process-lifetime high water, reported in bytes (Linux
VmHWM converted from KiB; Windows PeakWorkingSet64). Windows probes try `pwsh`
first, then `powershell`, retaining an explicit unavailable reason if both fail.
`rss_baseline_after_load` in series `environment.txt` is captured immediately
after load, before metadata probes and the first trial. Subtracting it from a
later high water shows growth above the post-load baseline, which includes trace
loading and prior harness allocations. It cannot isolate engine memory or the
oracle share: the oracle is created later, and high-water marks are not additive.
Per-trial high waters still include oracle, history, result hashing and earlier
trials. The input SHA-256 is computed once from original file bytes during load
and reused by metadata and all trial results, without re-encoding each trial.

Candidate frames use `IntegrityProfile::Structural`: no CRC-32C is stored or
verified. Measured replay excludes the CRC-32C cost selected by the B1 design;
this is a structural-profile cell, not the B1 CRC-32C profile cell. The profile
is unchanged in this round to preserve the meaning of prior host series.

Legacy internal append/rebuild stages are unavailable through the public API and
must carry a reason, never a fabricated zero. Store sizes are observed before
engine handles are dropped and trial stores are deleted. The default retains no
trace or source copy; `--retain-store` keeps stores and a trace copy for forensic
runs. Cleanup and retention are outside the measured operation intervals.

Host-observed candidate per-trial sizes before N5 changed generated inputs: at 1K, `results.cmt` 1.3 MB,
`observations.cmt` 0.8 MB and store 27 MB; at 10K, 20 MB, 15 MB and 265 MB,
respectively (rounded). Stores are removed by default; raw files remain external.
At 100K, raw series output may reach tens of GB (unmeasured projection). The
26 MB observed 1K hex-encoded trace projects to about 260 MB at 10K; whole-trace
generation/loading at 100K requires multiple GB. Observations now stream per op;
final header compaction uses 64 KiB, with canonical encoding unchanged. Query
signatures, trace operations, timing samples and whole-file hashing still allocate
memory. A streaming trace loader remains a follow-on.


## 11. Environment requirements

Rust 1.89.0, edition 2024; release profile for manual measurements. Retain compiler
and Cargo versions, flags, target OS/architecture, hardware and filesystem probes,
git revision and porcelain status, source/lockfile SHA-256 identities, invocation,
timestamp and cache policy. Unavailable observations include a reason. Retain
`source.sha256` and `source.patch` and keep the identified checkout externally;
no source tree is copied into each series. No privileged probes.

At 100K the whole decoded trace and whole physical/decoded history are buffered,
alongside engine, oracle and rebuilt states. The inspector estimates 10–15 GB
per trial; this is an unmeasured capacity projection, not a guaranteed upper
bound. Text guards add payloads, so allow additional headroom. A streaming
loader/decoder is deferred; this round only removes repeated input encodings.

## 12. Baselines

Candidate uses `RawAppender` and `reopen_and_replay` via unchanged EXP-0001 path
dependencies: **D1 ordinary writes, no fsync, no crash-survival claim**.
Candidate frames use `IntegrityProfile::Structural`: no CRC-32C is stored or
verified. Measured replay excludes the CRC-32C cost selected by the B1 design;
this is a structural-profile cell, not the B1 CRC-32C profile cell. The profile
is unchanged in this round to preserve the meaning of prior host series.
Legacy is `rusty_multimodal_db` at
`abda0a7e94a9727e410001e9724edc741f6e0d31`, Step 1 part A reviewed and pushed branch
head, not an asserted merged commit. `MemoryConnectionStore::new`, journal off.
Its mmap field updates do not flush per operation, but insert/replace/delete log
paths call `sync_data`. Inspection of that pinned `generic/mmap_store.rs` makes
equal durability impossible through the public adapter. Proposed deviation from
the work order: retain native settings as explicitly non-equivalent diagnostics;
do not modify the pinned engine or rank these as equal-durability throughput.
Neither baseline is claimed to survive power loss. Protocol-22 measurement cells use batches of width one to preserve complete
per-operation latency; field updates and reads remain single calls. Actual
multi-write batch equivalence is tested separately, with dependent writes.
Batch-size scaling is not measured in this increment.

## 13. Predeclared interpretation criteria

Any correctness mismatch invalidates performance interpretation. Keep negative
results. Describe improvements/regressions by operation and stage, alongside RSS,
disk size, raw range and cache/durability qualifications. No universal superiority,
confirmatory significance, or architecture promotion follows from this experiment.
The native durability mismatch precludes an equivalent-durability winner.

## 14. Implementation notes

Two independent workspaces: [candidate](../../experiments/convergence-memory/Cargo.toml)
and [legacy](../../experiments/convergence-memory-legacy/Cargo.toml). The candidate
has only local path dependencies. CMT1 and its independent model are shared by
path; neither engine calls model mutation logic. RF1 payloads are experiment-local
full after-images or tombstones. They are provisional, not canonical commits.
No EXP-0001 source, fixture, or evidence is changed.

## 15. Raw result locations

The [results index](EXP-0002/results/README.md) defines the committed subset:
series `environment.txt`, `source.sha256`, `source.patch`, `trials.csv`,
`summary.txt`, and each trial's `throughput.txt`. Full `results.cmt` and
`observations.cmt` stay in the operator output directory, referenced by absolute
path and SHA-256 in `throughput.txt`; do not copy entire series into the repo.
This is an accepted deviation from work-order R7, recorded in the results index:
raw per-operation samples are referenced by path and SHA-256, not committed.
Default output also retains per-trial `environment.txt` but no store directories,
trace copy or source tree. `--retain-store` retains stores and the trace copy
only in the operator's forensic output. Failures remain recorded in raw results
and any `failure.txt`. Operator commands and format details are in the workspace
README. Exclusive output directories and `create_new` prevent overwrite.

Host-observed candidate per-trial sizes before N5 changed generated inputs: at 1K, `results.cmt` 1.3 MB,
`observations.cmt` 0.8 MB and store 27 MB; at 10K, 20 MB, 15 MB and 265 MB,
respectively (rounded). Stores are removed by default; raw files remain external.
At 100K, raw series output may reach tens of GB (unmeasured projection). The
26 MB observed 1K hex-encoded trace projects to about 260 MB at 10K; whole-trace
generation/loading at 100K requires multiple GB. Observations now stream per op;
final header compaction uses 64 KiB, with canonical encoding unchanged. Query
signatures, trace operations, timing samples and whole-file hashing still allocate
memory. A streaming trace loader remains a follow-on.


Run from the repository root with a new scratch parent. These exact Windows
commands use GNU 1.89.0; on Linux replace the toolchain selector with `+1.89.0`
and use equivalent shell path variables. All runs create six trials. The README
remains the expanded guide. Legacy generation below uses the same shared generator;
all runs deliberately consume the same candidate-generated file.

```powershell
$cmOutputRoot = Join-Path $env:TEMP 'exp0002-n-host-series'
New-Item -ItemType Directory -Path $cmOutputRoot -ErrorAction Stop | Out-Null
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- generate 1k "$cmOutputRoot/1k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- generate 1k "$cmOutputRoot/legacy-generated-1k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/candidate-1k"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/legacy-1k-single" single
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/legacy-1k-pipelined" pipelined
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/legacy-1k-atomic" atomic
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- generate 10k "$cmOutputRoot/10k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- generate 10k "$cmOutputRoot/legacy-generated-10k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/candidate-10k"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/legacy-10k-single" single
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/legacy-10k-pipelined" pipelined
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/legacy-10k-atomic" atomic
```

Append `--retain-store` only for forensic output. For optional 100K use `100k`
in place of `10k` with new destinations; report executed/not-executed. The host
fills the result-index entries, including snapshot, input identity and validity.

## 16. Results

The [Windows proof](EXP-0002/IMPLEMENTATION-REPORT.md) passed after N1-N8: fifteen candidate/shared
tests, two legacy tests (all dispatch modes), and 95 portable EXP-0001 tests;
formatting, clippy, offline fetch, links and whitespace gates passed. Six host
series are committed as evidence subsets with filled entries in the
[results index](EXP-0002/results/README.md): 1K and 10K candidate and
legacy-single series from snapshot `850eb388` binaries (before the replay/decode
stage split, the RSS probe fix and the trace change of the final round), and a
1K candidate/legacy-single pair from snapshot `bd99987b` binaries, the revision
committed with this change. All 36 trials are valid. No performance conclusion is
drawn; the durability settings are not equivalent. Proof-run tests are validation,
not benchmark evidence.

Workflow added; not yet executed remotely; the legacy leg assumes anonymous
fetch of two GitHub git sources (`baileyrd/rusty_multimodal_db` and
`Rusty-Mill/rusty_mill`).
Further host series are recorded in the results index using its template; 100K
was not executed and is not run by CI.

## 17. Conclusion

Small/1K correctness validation passed and six host 1K/10K series (36 trials)
are valid against the oracle on both engines. The implementation includes
minimal retention and forensic opt-in; host series are cataloged in the results
index. No performance conclusion and no production graduation.

## 18. Follow-on questions

Which stages regress at realistic live-record counts? What equivalent durability
cell should step 3 enable? Does full after-image amplification justify a separately
measured field-delta experiment? Could a versioned digest-collapse representation
replace repeated query digest/UUID lists while preserving independent membership
and pagination-order checks? Query signatures are unchanged in this increment.
A CRC-32C profile cell should measure the B1-selected integrity cost separately.
Can a streaming loader/decoder bound trace/history memory without weakening
independent validation?
