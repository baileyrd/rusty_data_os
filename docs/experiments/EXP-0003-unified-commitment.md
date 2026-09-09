# EXP-0003 — Unified commitment and recovery

## 1. Identifier and title

EXP-0003: one bounded RF1 transaction history for Memory.

## 2. Status

Ready. The owner’s [merge plan, step 3](../plans/data-os-multimodal-merge-plan-2026-09-08.md)
authorizes implementation, injected-fault correctness tests and manual descriptive measurements.
This method is recorded before implementation. No production graduation or performance conclusion.

## 3. Linked hypothesis

[HYP-0003](../hypotheses/HYP-0003-unified-commitment.md).

## 4. Research question

Can one recoverable transaction history resolve the six merge-plan recovery scenarios
and execute the shared Memory traces with explicitly requested D1 or D2 acknowledgement?

## 5. Hypothesis under test

A single RF1 history using R5 per-transaction sync recovers every scenario under injected
process termination and byte-level torn tails, and agrees with the independent CMT1 oracle.
One invalid promotion, lost required result, inconsistent retry or field mismatch falsifies the cell.

## 6. Independent variables

D1/D2, small/1K/10K traces, named injection point and byte-tail extent. Each is a separate cell.
D3, multiple writers, networking, compaction, legacy integration and physical faults are excluded.

## 7. Controlled variables

Byte-identical CMT1 inputs, seed 7, one writer, queue depth one, one transaction per event,
CRC-32C on all five frames, unchanged RF1 codec, fresh store per trial, uncontrolled caches.
One warm-up and five measured trials; generation, oracle comparison and output are outside timings.

## 8. Workloads

The shared small and repeated 1K/10K Memory traces cover all thirteen fields, insert,
field update, replacement, guarded update, deletion/recreation, equality, aggregation and pages.
Separate multi-operation transaction tests cover dependent operations and incarnation-bound edges.
An operations batch is one opaque event payload, never an atomic group of RF1 events.

## 9. Correctness invariants

Predeclared oracle for **injected process termination**, at both requested levels:

For Binding, Reservation and Provisional the named D2 hook fires after that record's
successful file sync; the corresponding D1 hook fires immediately after append. Final fires
after Final append and before Commit append; Commit fires after Commit append and before
D2's post-finalization sync. Synced fires after that sync (after Commit append at D1), before
publication. Checkpoint fires after checkpoint file sync and before its directory sync.


| Placement | Oracle and required state |
|---|---|
| After Binding / Reservation / Provisional | Must not be promoted; no new visible transaction; report available bindings/gaps |
| After Final before Commit | Must not be promoted; valid Final residue is reported, not an error |
| After Commit before post-finalization sync | May recover: uncertain outcome; a complete validated pair resolves to its original outcome under this process-only mechanism |
| After sync before publish | Must recover under this injected process-only test; old reader snapshot stays unchanged |
| Mid-checkpoint before directory sync | History commit must recover; use a valid checkpoint/history combination or full replay |
| Validation rejection | Rejected; append length and published state unchanged |
| Interior Final / Provisional corruption | Corrupt or undecidable; fail closed at damaged offset |

New payloads containing the literal byte sequence `RDE1` are rejected before append with
`Invalid("payload contains the RF1 magic")`; old normalized requests containing it fail
recovery as corrupt. The frozen scanner searches the incomplete frame body for this magic
and otherwise can misclassify a torn payload as interior damage. Literal payload magic is
unreachable for CMM2's hex-encoded values and fixed operation syntax.
This restriction does not guarantee recovery of every possible torn RF1 frame: a valid
caller request UUID can contain `RDE1` at frame offset 32 (body offset zero) (tested with a legitimately encoded
Binding truncated one byte before its end). That tail remains corrupt or undecidable.
Binary envelope fields can also contain the sequence. The scanner searches after its
32-byte header, so the header CRC alone does not trigger this heuristic. No additional
identity restrictions or frozen-codec changes are silently imposed.

Checkpoints first synchronize the history writer at either level, then create/sync the
checkpoint and synchronize its parent. `HistoryShorterThanCheckpoint` still fails closed.
For a remaining OS-crash case outside this tested model, the manual remedy is to preserve
the evidence and remove the newer checkpoint file (and any other checkpoint beyond the
surviving history) before reopening from history; this explicitly accepts the older state
and does not recover missing committed bytes.

Tested torn tails at every boundary and inside Commit permit only complete selected events. Complete
Final without Commit is residue. Boundary EOF needs no repair; incomplete suffix is truncated
to the accepted offset and synchronized before another append. The bytes cannot distinguish
interrupted append from later truncation. Only independent valid checkpoint evidence beyond
the accepted history detects loss of a committed suffix; then open fails closed.

Every potentially mutating write/sync failure poisons the writer. Pre-Final failures reject;
boundary uncertainty is indeterminate. Retry after reopen returns the original outcome without
append, or RequestIdReuse for different normalized bytes, including changed durability.
Two full replays must agree. Checkpoint cache disagreement fails closed; history remains authority.
UUID recreation increments incarnation and rejects predecessor updates/edges.

## 10. Benchmark metrics

Predeclared `Instant` nanoseconds: complete operation and query stages, append, replay
(physical scan), decode (payload parsing), rows, columns, checkpoint and open_from_checkpoint.
Append is nested in operation time. Record counts, logical store bytes, payload bytes and
physical history bytes expose amplification: three payload copies plus framing/envelope overhead.
The runner reports min/p50/p90/p95/p99/max/count and separate operations/second, excluding warm-up.
Five stage samples do not justify tail claims. RSS includes harness/oracle and uses process high water.

The accepted history prefix remains resident for the Log's lifetime, bounded by `MAX_HISTORY`
(1 GiB); every binding's complete normalized request is also retained. The adapter validator
clones the slot map once and the core clones it again per transaction: O(records) per commit,
inside `complete_operations`. Shared record values do not eliminate these map copies.
The frozen `MAX_RECORDS = 1_000_000` scan cap bounds a history to about 200,000 transactions
(five frames each), fewer with uncommitted residue; the byte limit may be reached earlier.

## 11. Environment requirements

Provision memory for the resident accepted prefix and full binding requests, plus two slot-map
copies during each commit; the 1 GiB history and 1,000,000-frame caps bound one store.

Rust 1.89, edition 2024, workspace/path dependencies only, release builds for measurements.
Record OS, target, filesystem, CPU/RAM/device, compiler/profile/flags, source revision and patch,
trace hash, cache state, missing probes and operator storage assumptions as in
[methodology](../benchmarks/METHODOLOGY.md). Safe `File::sync_all` supplies the local sync call;
Windows directory handles need writable access and `FILE_FLAG_BACKUP_SEMANTICS`.
Documented semantics and successful calls alone are not empirical survival evidence.
The platform durability contract remains evidence-pending. A child abort leaves the OS running;
this is neither OS-crash nor power-loss evidence. No such survival claim is made.

## 12. Baselines

Unified D1 and unified D2 are separate cells. EXP-0002’s structural ordinary-write candidate
and native legacy settings are diagnostics with different integrity, transaction and durability costs.
No equal-durability winner claim. Existing evidence and source authorities stay unchanged.

## 13. Predeclared interpretation criteria

Correctness failure invalidates performance interpretation. Preserve failures and unavailable
observations. Descriptive distributions cannot promote architecture or establish platform durability.
Each recovery scenario needs a disabled-mechanism counterexample or an explanation of unreachability.

## 14. Implementation notes

`Log::create(directory, apply, state_decoder)` exclusively creates a new directory and
history; the parent must exist, and an existing directory (even empty) or history is refused.
`Log::open(directory, apply, state_decoder)` requires both directory and history.rf1;
absence returns `LogError::NotFound` without creating either or an owner file. Both operations
hold the OS-released owner.lock advisory lock. MemoryEngine likewise exposes separate create
and open operations; series and fault setup explicitly create, while recovery explicitly opens.

[Standalone workspace](../../experiments/unified-commitment/README.md);
[proposed ADR](../adr/ADR-0003-unified-commitment.md). One payload-agnostic event per transaction,
unchanged R5 Binding/Reservation/Provisional/Final/Commit lifecycle, single immutable publication.
UCR1 retains requested durability and full payload; UCE1 repeats identities, sequence and realtime
sample, includes the normalized bytes, and explicitly writes zero provenance/reference counts.
Every Final envelope is validated even without a Commit. Achieved on recovery comes from the
validated bound request; it records the requested protocol, not independent proof of sync return.
No stronger retry upgrade exists. Every history binding remains retained without expiry.
The SHA-256 cache never substitutes for byte equality. uc1 checkpoints retain position, prefix hash,
encoded state, resolved cache and CRC-32C, use create_new/file sync/directory sync, and never erase history.

R8b’s unchanged-output wording has a necessary exception: recoverable untracked-file patches
correct the previous identifying-only artifact. APIs and default EXP-0002 metadata remain compatible.
The specific R8b runner exception overrides the work order’s general Step 2 unchanged-path exclusion.

The host-approved P1 constraint rejects `RDE1` in payloads before writes and in normalized
request decoding. CMM2 hex payloads cannot contain it; binary framing/envelope fields still
can, so §9 records the tested residual torn-frame ambiguity.
Series now fail closed when `git` is unavailable or the output directory lies inside a source
prefix (required by the evidence rule); this applies to EXP-0002 callers too.

## 15. Raw result locations

[Results index](EXP-0003/results/README.md). Retain the same committed evidence subset as EXP-0002:
environment.txt, source.sha256, source.patch, trials.csv, summary.txt and trial throughput.txt.
Raw per-operation results.cmt/observations.cmt remain external with absolute path and SHA-256;
this carries forward the accepted R7-style retention deviation. Default removes validated stores;
--retain-store keeps stores and trace copy. No source tree copy. A series is evidence only if
porcelain is clean at revision or source.patch reconstructs every source.sha256 manifest entry.
The host verifies reconstruction before cataloguing, runs 1K D1/D2 and 10K D2 manually, and
records the fault matrix. Commands are in the workspace README, using the Windows GNU selector.

From the checkout root, with a newly created operator directory:

```powershell
$ucOutput = Join-Path $env:TEMP 'exp0003-host-series'
New-Item -ItemType Directory -Path $ucOutput -ErrorAction Stop | Out-Null
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --locked --offline -- generate 1k "$ucOutput/1k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --locked --offline -- run "$ucOutput/1k.cmt" "$ucOutput/1k-d1" d1
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --locked --offline -- run "$ucOutput/1k.cmt" "$ucOutput/1k-d2" d2
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --locked --offline -- generate 10k "$ucOutput/10k.cmt"
cargo +1.89.0-x86_64-pc-windows-gnu run --release --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --locked --offline -- run "$ucOutput/10k.cmt" "$ucOutput/10k-d2" d2
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml -p uc-harness --test faults --locked --offline -- --nocapture
```

Retain each trial's `store.amplification.txt` with the evidence subset: payload byte
count, physical history bytes and three-copy representation. Source output directories
must be outside the selected source prefixes; a self-referential manifest cannot be
reconstructed. Such destinations fail explicitly before trials (an R8b retention constraint).

`source.sha256` hashes exact working-tree file bytes, including line endings. Both tracked
and untracked patch diffs explicitly use `-c core.autocrlf=false -c core.safecrlf=false` and
`-c diff.noprefix=false -c diff.mnemonicPrefix=false` before `diff`, with `--binary --no-ext-diff --no-textconv`. Host Git conversion or prefix settings must not alter retained
patch bytes. The always-run fixture checks CRLF-modified tracked text and untracked binary
reconstruction with both source and restore configured `core.autocrlf=true`.

## 16. Results

The local agreed proof passed: 30 unified, 17 shared/candidate EXP-0002 and 95 portable
EXP-0001 tests, formatting, Clippy, links and whitespace. Full output and deviations are
recorded in the [implementation report](EXP-0003/IMPLEMENTATION-REPORT.md).
The host reported a D1 series on the prior snapshot; manual series on these fixed binaries
and remote CI remain pending. Host series entries stay empty. No measurement conclusion.

## 17. Conclusion

Bounded implementation and correctness validation pass for the tested injected fixtures,
subject to the residual torn-frame ambiguity disclosed in §9.
Independent review and manual descriptive measurements remain pending.
Hypothesis remains Open and experiment Ready; no platform durability or performance conclusion.

## 18. Follow-on questions

Check-only validator, incremental prefix hash and persistent state structure are follow-ons.
Supporting arbitrary payloads and eliminating the remaining magic-based torn-frame ambiguity
require a codec-level contract change; the payload restriction alone is not a universal repair.
Payload-by-reference requires a contract change to remove triple payload copies. Explicit retention
(REQ-011), compaction, bounded/streaming replay, D3 grouping, multi-client concurrency, schema catalog,
legacy integration and platform OS-crash/power-loss validation remain separate work.
