# Shared Memory experiment

[EXP-0002 methodology](../../docs/experiments/EXP-0002-convergence-memory.md) and
[HYP-0002](../../docs/hypotheses/HYP-0002-memory-convergence.md).
Candidate: **D1 ordinary writes, no fsync, no crash-survival claim**.
All code is experimental. Memory is the existing application record, not a
restriction on the eventual database. No Linux-only harness dependency is used.

Run from the repository root. On this Windows build box use
`+1.89.0-x86_64-pc-windows-gnu`; Linux CI uses `+1.89.0`. Generate each trace once
and pass that same file to both engines. Output directories must not exist, and
their parent must already exist. Keep operator output outside the checkout and
copy only the committed-evidence subset described in the
[results index](../../docs/experiments/EXP-0002/results/README.md). Commands for
the host's 1K and 10K checks (choose a new scratch parent):

```powershell
$cmOutputRoot = Join-Path $env:TEMP 'exp0002-host-series'
New-Item -ItemType Directory -Path $cmOutputRoot -ErrorAction Stop | Out-Null
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- generate 1k "$cmOutputRoot/1k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/candidate-1k"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/1k.cmt" "$cmOutputRoot/legacy-1k" single
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- generate 10k "$cmOutputRoot/10k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/candidate-10k"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/convergence-memory-legacy/Cargo.toml -p cm-legacy-runner --locked --offline -- run "$cmOutputRoot/10k.cmt" "$cmOutputRoot/legacy-10k" single
```

For optional 100K replace `10k` with `100k` in the last three commands and retain
whether it completed. CMT1 hex-encodes every text field: the host observed a
26 MB 1K trace before N5, projecting approximately 260 MB at 10K. Text guards
now add payloads; use the recorded input hash and shape when comparing sizes.
The 100K generator/loader buffers the encoded trace and decoded operations;
replay buffers the whole physical history and decoded `Change` values, alongside
engine/oracle/rebuilt states. The inspector estimates **10–15 GB per trial**
(unmeasured projection, not a guaranteed upper bound); allow additional headroom.
A streaming loader/decoder is a follow-on, not part of this increment. The format
is unchanged. No 100K run is part of CI or the proof gate.

The host observed the following candidate per-trial logical sizes before N5
(MB, rounded; new text guards change generated inputs):

| Size | results.cmt | observations.cmt | store before cleanup |
|---|---|---|---|
| 1K | 1.3 MB | 0.8 MB | 27 MB |
| 10K | 20 MB | 15 MB | 265 MB |

These are storage observations, not a performance comparison. Raw signatures
retain matching-record digests and ordered UUID lists in diagnostics and again
hex-encoded in observations. Large-query output can dominate retention; 100K
series may require tens of GB of raw output (projection, not measured here).
Operations are streamed to `observations.cmt` as produced. Its final record-count
header is completed with a 64 KiB compaction buffer, preserving canonical bytes.
The runner no longer buffers the complete observed-operation signature vector;
individual query signatures, the decoded input trace, timing samples, and raw-file
hashing (which still reads one complete file) remain memory costs. Streaming the
loader and raw-file hashing are follow-ons; query representation is unchanged.

For separate protocol-22 cells replace `single` with `pipelined` or `atomic`, and
choose a new output directory. Timings use batch width one to retain complete
per-operation latency, including batch construction/dispatch cost. Multi-operation
batches are exercised in correctness tests; batch-size scaling is unmeasured.

The legacy adapter is pinned to reviewed Step 1 part A commit
`abda0a7e94a9727e410001e9724edc741f6e0d31`. No journal is enabled. Runtime
insert/replace/delete log writes call `sync_data`; mmap field updates do not
flush per operation. This setting is **not equivalent** to candidate D1. Neither
engine has a power-loss claim here. Native baseline comparisons are descriptive
and cannot establish an equal-durability winner. Internal legacy append, replay
and rebuild timings are unavailable through its public adapter.

Candidate stages are `append`, `replay`, `decode`, `rows`, `columns` and query
operations. `replay` times only `reopen_and_replay`; `decode` separately parses
CMM1 payloads into `Change` values. Both appear in `summary.txt`; append remains
nested in complete operation time and must not be added to it.

Candidate frames use `IntegrityProfile::Structural`: no CRC-32C is stored or
verified. Measured replay excludes the CRC-32C cost selected by the B1 design;
this is a structural-profile cell, not the B1 CRC-32C profile cell. The profile
is unchanged in this round to preserve the meaning of prior host series.
Candidate identity is `candidate RF1 full after-image; IntegrityProfile::Structural;
rev=<git HEAD>`, with the revision cached once per process and D1 declared separately.

Each invocation creates one warm-up and five measured fresh-store trials. Default
series retention is `environment.txt`, `source.sha256`, `source.patch`,
`trials.csv`, and `summary.txt`. Each trial retains `environment.txt`,
`throughput.txt`, `results.cmt` diagnostics with raw operation nanoseconds and
expected/actual signatures, and canonical `observations.cmt` with final digests.
The environment records generator seed, shape, input SHA-256 and the exact
`generate <size> ...` regeneration command. Regenerate from the repository root
into the command's new destination and verify its SHA-256; custom or edited
traces must be preserved externally by the operator.

The default does not copy the input trace or source tree. Source identity uses
`source.sha256` and `source.patch`; keep the corresponding checkout available.
After digests are computed and compared, store size is recorded, engine handles
are dropped, and each trial's `store/` is deleted, including for invalid trials.
Raw results and failure diagnostics remain in the operator's output directory.
For forensic runs append `--retain-store` to either runner's `run` command
(after the legacy dispatch mode). This retains every trial's `store/` and a
series-level `trace.cmt`; it still does not copy sources. Cleanup, retention and
raw-file hashing occur outside engine operation timers.

Only the series-level metadata/summary files and each trial's `throughput.txt`
belong in committed evidence; the [results index](../../docs/experiments/EXP-0002/results/README.md)
defines the exact layout. `throughput.txt` includes absolute paths and SHA-256s
for the operator-retained `results.cmt` and `observations.cmt`. Preserve those raw
files, including negative or failed results, and update references if relocated.
Mismatches are printed and retained; remaining operations/trials run; any invalid
trial returns a nonzero process status. No warm-up samples enter the summary.
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

## CMT1 revision 1

UTF-8, LF, trailing LF required, no blank lines. Header tokens are tab-separated:
`CMT1`, `1`, name, seed (u64 decimal), distinct inserted UUID count, operation
count, content minimum/maximum bytes, metadata minimum/maximum bytes, maximum
tag count. IDs are 32 lowercase hexadecimal characters, compared lexicographically
as 16 bytes. Each operation line starts with its sequential one-based u64 ID.

Values are `s` plus UTF-8 hex, `i` plus canonical signed decimal, `b0`/`b1`, or
`l<count>:<comma-separated UTF-8 hex tags>`. Thus empty lists and one empty tag
are distinct. For hand reading, decode a text hex payload (omit its `s` prefix):
`python -c "print(bytes.fromhex('68656c6c6f').decode('utf-8'))"`.
A Memory is UUID then thirteen values in this exact order:
content, category, tags, source, metadata_json, created_at_unix_ms,
updated_at_unix_ms, memory_type, status, sensitive, access_count,
deleted_at_unix_ms, node_id. Count and deletion stamp are non-negative. JSON is
opaque text, tag order significant, and timestamps are signed i64 milliseconds.

| Operation | Remaining tokens | Semantics |
|---|---|---|
| insert | Memory | Inserted or Duplicate, no overwrite |
| get | UUID | Rows (one record) or NotFound |
| update | UUID, non-negative i64 | Set access_count, Updated or NotFound |
| replace | Memory | Replace every field, Replaced or NotFound |
| guard | field ordinal, expected Value, Memory | Equality compare-and-replace, Replaced/GuardFailed/NotFound |
| delete | UUID | Hard delete, Deleted or NotFound |
| equal | category UTF-8 hex | Full matching records in UUID order |
| aggregate | none | Count/sum/min/max of access_count; empty gives 0/0/None/None |
| page | positive limit, `none` or timestamp and UUID | Ascending timestamp/UUID, exclusive cursor |

`CMT1-results` revision 1 exchanges input SHA-256, hex-encoded engine declaration,
operation count and final record count, followed by the mandatory engine-declared durability line (non-empty UTF-8,
hex-encoded),
numbered operation signatures, UUID-sorted final record digests and thirteen
numbered column digests. Candidate observations declare D1; legacy observations
declare the legacy setting. Adjacent diagnostics and metadata retain both
`candidate_durability` as a reference and `engine_durability` for the engine run.
Signatures retain outcome, aggregate, sorted per-record
digests and query UUID order separately so sorting cannot hide a pagination error.
Record SHA-256 domain is `CMT1/record` plus tab and canonical Memory bytes; column
domain is `CMT1/column/<ordinal>` plus LF and UUID/value lines sorted by UUID.
All SHA-256s are full 32-byte lowercase hex. [Golden fixtures](crates/cm-trace/tests/fixtures/SHA256SUMS)
include an independent Python reference; Rust tests round-trip formats, reject
noncanonical inputs and hand-check outcomes, ordering and aggregates.

The standalone trace crate owns no engine dependencies. Its small SHA-256 routine
was copied from the repository's existing workload-conformance implementation;
fixture hashes are independently produced by Python hashlib. Candidate CMM1
payloads store a `put` after-image or a `delete` UUID inside structural RF1 type-3
provisional frames. Row and per-field column builders read those changes separately.
