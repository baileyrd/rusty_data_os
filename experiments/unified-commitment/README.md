# Unified commitment — EXP-0003

Bounded single-writer transaction history, frozen RF1 codec, CMM2 Memory adapter and
shared CMT1 runner. [Experiment/method](../../docs/experiments/EXP-0003-unified-commitment.md),
[hypothesis](../../docs/hypotheses/HYP-0003-unified-commitment.md),
[implementation report](../../docs/experiments/EXP-0003/IMPLEMENTATION-REPORT.md).

`Log::create(directory, apply, state_decoder)` exclusively creates a new directory and
history; the parent must exist, and an existing directory (even empty) or history is refused.
`Log::open(directory, apply, state_decoder)` requires both directory and history.rf1;
absence returns `LogError::NotFound` without creating either or an owner file. Both operations
hold the OS-released owner.lock advisory lock. MemoryEngine likewise exposes separate create
and open operations; series and fault setup explicitly create, while recovery explicitly opens.

`commit(Transaction, validator)` resolves retry before validation, stages the complete
payload against an immutable current state, appends Binding/Reservation/Provisional/
Final/Commit, and publishes a single Arc after the requested boundary. Readers keep
old immutable snapshots. The payload-agnostic apply/decoder functions must be deterministic.
The caller supplies explicit stable request IDs.
Auto request IDs use domain byte `0x41`, separate from the explicit test/caller convention
`0x52` and event domain `0x45`; callers must reserve `0x41` for the adapter. They are local
conveniences, not stable request identities across a torn-tail reopen: an ordinal removed
with a partial binding can be reused. Retry-sensitive callers must supply stable IDs.

Event IDs use a distinct deterministic local namespace with physical ordinal, not random generation.

Limits: 4 MiB payload, 1,024 CMM2 operations per transaction, 1,000,000 RF1 records,
1 GiB history. Recovery buffers the whole scan; diagnostics are bounded by file length.
Memory rows contain shared immutable records but clone the slot map to stage a transaction.
This deliberate cost is experimental; no streaming or performance conclusion is implied.

The accepted history prefix remains resident for the Log's lifetime, bounded by `MAX_HISTORY`
(1 GiB); every binding's complete normalized request is also retained. The adapter validator
clones the slot map once and the core clones it again per transaction: O(records) per commit,
inside `complete_operations`. Shared record values do not eliminate these map copies.
The frozen `MAX_RECORDS = 1_000_000` scan cap bounds a history to about 200,000 transactions
(five frames each), fewer with uncommitted residue; the byte limit may be reached earlier.


- `unified D1 ordinary writes, no sync, no crash-survival claim`
- `unified D2 per-transaction sync (R5 placements); tested failure model: injected process termination and torn tails; platform durability contract evidence-pending; no OS-crash or power-loss claim`

D1's no-sync label describes transaction appends. Namespace setup, tail repair and explicitly
requested checkpoints synchronize at both levels. D1 `Committed` denotes the completed
ordinary-write protocol, not a durable canonical claim. D2 synchronizes after Binding,
Reservation, Provisional, then the adjacent Final/Commit pair. The one persisted realtime
sample (Unix nanoseconds, i64) is taken after pre-finalization sync; no observation timestamp
is written into history. `File::sync_all` is used for files and directories; on Windows,
the directory handle requests GENERIC_WRITE and FILE_FLAG_BACKUP_SEMANTICS. These calls
are not evidence of OS-crash/power-loss survival.

The core restricts payload bytes as follows.

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

UCR1 bytes: magic including NUL, durability u8, payload length u64 LE, complete payload.
UCE1 bytes: magic including NUL, request UUID, event UUID, sequence u64, time i64,
durability u8, source count u16 = 0, reference count u16 = 0, complete UCR1. Both counts
explicitly mean empty: provenance/causal references are outside this bounded increment.
Every Final is checked against its binding even if unselected. A missing/truncated Commit
leaves Final residue. Three payload copies plus RF1/envelope framing are a measured cost.

After any write/sync failure the writer is poisoned until reopen. Before Final submission,
failure rejects; Final submission onward is conservatively indeterminate. Open resolves
all retained bindings from byte zero; no binding expiry or stronger-durability retry upgrade.
Same ID/different payload or durability rejects without append. A stronger request needs a new ID.
High-water gaps are retained; complete valid pairs alone produce replay effects. A lost
committed suffix is detectable only when independent valid checkpoint evidence extends beyond it.

uc1 checkpoint body: `UC1\0`, sequence/Commit ordinal/end offset (u64 LE each), 32-byte
SHA-256 of that exact history prefix, state blob (u64 length + bytes), resolved cache blob,
then CRC-32C of all preceding bytes. Cache entries are sorted by request UUID, with count
u64, UUID, event UUID, normalized SHA-256, normalized blob and outcome (0 uncommitted;
1 plus sequence/ordinal u64, time i64, achieved u8). The cache covers bindings before the
checkpoint position; open always rebuilds every binding, including later residue, from history.
Checkpoints first synchronize the history writer at either level, then create/sync the
checkpoint and synchronize its parent. `HistoryShorterThanCheckpoint` still fails closed.
For a remaining OS-crash case outside this tested model, the manual remedy is to preserve
the evidence and remove the newer checkpoint file (and any other checkpoint beyond the
surviving history) before reopening from history; this explicitly accepts the older state
and does not recover missing committed bytes.
Files use create_new, file sync then parent sync. Repeating a checkpoint at the same position
returns AlreadyExists. Newest valid matching checkpoint accelerates state replay; the complete
history scan and cache comparison remain mandatory. There is no history deletion/compaction.

CMM2 is UTF-8, `CMM2\n` then newline-terminated put/update/delete/link operations. Put and
delete retain record incarnation; updates and both edge endpoints name their incarnation.
Insert increments the retained tombstone generation; deletion detaches both incident edge directions.
The adapter validates all operations in order, including effects of earlier operations in one payload.
CMS2 checkpoints retain tombstones and edges as well as live Memory values.

Run from this checkout's root, using a new output parent:

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

On Linux use `+1.89.0` and equivalent shell paths. The host performs these manual series;
CI/proof executes small/1K correctness, not benchmark series. Append `--retain-store` for
forensic retention. Default retains metadata/results, removes validated stores, and does not
copy source trees. Each trial also retains `store.amplification.txt` with input payload/physical
history byte counts; timings retain append/replay/decode/rows/columns/checkpoint/open_from_checkpoint.

`fault d1|d2 DIR binding|reservation|provisional|final|commit|synced|checkpoint` aborts the
workspace's own binary at the selected point; the parent verifies its captured placement marker.
For Binding, Reservation and Provisional the named D2 hook fires after that record's
successful file sync; the corresponding D1 hook fires immediately after append. Final fires
after Final append and before Commit append; Commit fires after Commit append and before
D2's post-finalization sync. Synced fires after that sync (after Commit append at D1), before
publication. Checkpoint fires after checkpoint file sync and before its directory sync.

Both fault modes require a missing DIR beneath an existing parent.
`fault lock DIR hold` creates a history and holds owner.lock until stdin triggers abort. Process tests live in
uc-harness integration tests because Cargo supplies `CARGO_BIN_EXE_uc-harness` to that package.
Core byte/I/O tests and Memory scenarios remain in their respective required integration files.

`source.sha256` hashes exact working-tree file bytes, including line endings. Both tracked
and untracked patch diffs explicitly use `-c core.autocrlf=false -c core.safecrlf=false` and
`-c diff.noprefix=false -c diff.mnemonicPrefix=false` before `diff`, with `--binary --no-ext-diff --no-textconv`. Host Git conversion or prefix settings must not alter retained
patch bytes. The always-run fixture checks CRLF-modified tracked text and untracked binary
reconstruction with both source and restore configured `core.autocrlf=true`.

Source retention uses SeriesMeta EXP-0003/HYP-0003 and the four frozen prefixes. It includes
tracked and untracked non-ignored files and a reconstructable binary patch. R8b necessarily
corrects the old patch bytes for untracked changes while preserving APIs/EXP-0002 defaults.
The reconstruction test uses a temporary object-sharing repository and fresh Git worktrees,
never writes to the source repository's Git metadata and never commits. The real-checkout
pass skips dirty source prefixes with a reason; the fixture repository starts with
`core.autocrlf=false` and additionally tests reconstruction with host conversion enabled.
Put series output outside the selected source prefixes: self-referential manifests cannot
be reconstructed, so the runner rejects those destinations explicitly before trials.

Follow-ons: check-only validator, incremental prefix hash, persistent state structure,
and a codec-level contract change supporting arbitrary payloads and unambiguous torn tails.
