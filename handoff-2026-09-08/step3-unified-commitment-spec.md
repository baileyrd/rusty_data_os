# Work order: merge plan Step 3 — unify commitment and recovery (Data OS)

Source plan: `docs/plans/data-os-multimodal-merge-plan-2026-09-08.md`, section
"3. Unify commitment and recovery" and its subsection "What replaces the journal". Depends on
Step 2 (EXP-0002, committed at `772c720`/`e62fa69`).

Target repository: the Data OS build worktree at `C:/dev/rusty_data_os-step3`, branch
`codex/merge-step3-unified-commitment` created from `e62fa69` (Step 2 head). Resolve every path
against that worktree; never edit `C:/dev/rusty_data_os` or `C:/dev/rusty_data_os-step2`.
Follow `AGENTS.md` (§2 measurement method before implementation, §4, §5 correctness invariants
independent of performance, §6, §7 same-change documentation, §10 authorization ledger) and
`docs/experiments/README.md` (18-section document, status vocabulary). Do not `git commit`.

## Goal

A bounded local transaction core in Data OS with one authoritative, recoverable commit history:
validate a complete transaction against a coherent state, assign an ordered commit position,
append a complete recoverable transaction, satisfy the requested durability level, publish the
committed state atomically, and return an outcome tied to the request identity. The reproduced
legacy failures become concrete recovery tests that pass under a declared, tested failure model.
The Memory traces of EXP-0002 run on this core so the candidate can be measured at a declared
per-transaction sync durability level.

## Repository facts the design must respect (verified 2026-09-09)

- `experiments/exp-0001/crates/exp1-record-format` is the frozen `EXP1-B1-RF1` codec. `Body`
  kinds: `Binding`(1), `Reservation`(2), `Provisional`(3), `Membership`(4), `Final`(5),
  `Commit`(6). `Reservation`, `Final` and `Commit` must use `IntegrityProfile::Crc32c`;
  `Membership` must be `Structural`; `Binding` and `Provisional` accept either.
  `validate_lifecycle(&[Record])` enforces: strictly consecutive `physical_ordinal`, a
  `Reservation` needs its `Binding`, a `Final` needs `Binding` + `Reservation` at the same
  sequence, a `Commit` must be immediately adjacent to its `Final` with matching
  `final_ordinal`, event id, sequence and `final_crc32c`, and commit sequences strictly
  increase. Constants `MAX_RECORDS = 1_000_000`, `ScanLimits` caller-supplied.
- `docs/experiments/EXP-0001/R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md`
  freezes the recoverable-commit contract: only a valid `Final` selected by one valid `Commit`
  is canonical; the append/sync sequence is type 1 + fsync, type 2 + fsync, type 3 (+ one type 4
  per D3 group) + one pre-finalization fsync, one realtime clock sample per event, type 5
  immediately followed by type 6, then one post-finalization `fsync(data_fd)` = "the
  recoverable-commit establishment boundary"; file creation requires `fsync(data_fd)` then
  `fsync(parent_dir_fd)`; retry reuses the same binding, event and reserved sequence; a lost
  acknowledgement after the boundary returns the recovered committed event without appending
  another; restart sets the allocator above the greatest valid reservation high-water even
  with gaps; recovery scans from byte zero; checksum validity alone never establishes commit.
- `exp1-raw-append-replay::RawAppender` deliberately offers no synchronization and poisons on
  a terminal write failure; `reopen_and_replay(path, ScanLimits) -> ReplayReport` returns
  `accepted_prefix`, `records`, `scanned_bytes` and `ReplayTermination::{CleanEof,
  TerminalTruncation, Failure, IoFailure}`.
- Durability vocabulary (`docs/GLOSSARY.md`, `docs/experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md`,
  `docs/experiments/EXP-0001/R4-…-PLATFORM-DURABILITY-CONTRACT.md` §7): D0 process memory, D1 OS
  buffer, D2 per-event sync, D3 grouped sync. D2/D3 are canonical only when the recorded
  platform contract supports the boundary; "documented semantics alone are not empirical
  survival evidence"; forbidden phrases: "on stable media", "power-loss safe", "no torn
  writes", "atomic event append", "exactly once". `docs/experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md`
  gives the recovery oracle classes (Must recover / May recover: uncertain outcome / Must not be
  promoted / Rejected / Corrupt or undecidable), the fault taxonomy (process termination, OS
  crash, power loss, torn/truncated write, explicit I/O error) and the rule "a process kill while
  the OS continues running is never evidence of OS-crash or power-loss durability"; mechanism
  labels are injected / simulated / virtualized / physical.
- `docs/REQUIREMENTS.md`: REQ-001 canonical history is the single authority; REQ-009 stable
  request identities for duplicate detection and idempotent retry, distinct from event
  identities; REQ-012 a checkpoint identifies the exact canonical-history position and is
  validatable against it; REQ-013 canonical commit only after the declared durability boundary;
  REQ-014 single-event commits only (atomic multi-event batches are a future capability).
- `AGENTS.md` §10 currently authorizes only merge-plan step 2 and still lists "D2/D3, `fsync`
  durability, faults" as unauthorized. The owner-authored merge plan (step 3) authorizes this
  work; record that in §10 with one sentence, as Step 2 did, leaving the rest intact.
- There is no crash, kill or fault-injection harness in the repository; only byte-level
  truncation/corruption tests and in-memory scripted writer/reader injectors exist.
- Step 2: `experiments/convergence-memory/crates/cm-trace/src/run.rs` defines
  `trait Engine { fn label(&self) -> String; fn durability(&self) -> String; fn execute(&mut self, op: &Op) -> Result<Answer, String>; fn records(&self) -> Result<Vec<Memory>, String>; fn finish(&mut self, expected: &[Memory]) -> Result<Vec<(String, u128)>, String>; }`
  and the series driver; `cm-candidate` writes `Provisional` frames with `Structural` profile,
  CMM1 put/delete payloads, no incarnation field, no sync. Engine labels and per-engine
  durability lines are defined in `docs/experiments/EXP-0002/results/README.md`.
- Windows build box: spell `cargo +1.89.0-x86_64-pc-windows-gnu`; the Codex sandbox has no
  network (path dependencies only; no new external crates are available); `pwsh` exists,
  `powershell` does not; exp-0001 checks use `--workspace --exclude exp1-descriptive-d1-harness`.

## Design decisions settled by the host (do not relitigate)

- **One transaction = one RF1 event.** A transaction's operations are the opaque
  `stable_core` payload of a single `Provisional` record; the core interprets only the envelope
  and lifecycle (REQ-008, REQ-014). Multi-transaction atomicity (D3 groups) is out of scope.
- **Reuse the frozen R5 lifecycle and codec unchanged.** No new physical record kind, no codec
  change, no new wire format. `Binding` carries the caller's request identity; `Reservation`
  assigns the sequence; `Final` + adjacent `Commit` establish the commit; `Crc32c` on every
  record kind that permits it (use `Crc32c` for `Binding` and `Provisional` too).
- **Durability levels offered:** `D1` (ordinary writes, no sync) and `D2` (the R5 fsync
  placements per transaction). D3 grouping is a documented follow-on, not implemented.
- **Payload validation is a caller-supplied closure** run inside the commit critical section
  against the current published state (the Memory adapter supplies it). The core itself is
  payload-agnostic.
- **Location:** a new standalone workspace `experiments/unified-commitment/` (edition 2024,
  `rust-version = "1.89"`, same lints/profiles and `rust-toolchain.toml` shape as
  `experiments/convergence-memory/`, own `Cargo.lock` with workspace/path crates only). Nothing
  graduates into `/crates/`.
- **Tested failure model:** injected process termination at declared injection points (a child
  process run of the same binary that aborts at a chosen point) and torn/truncated tails
  (byte-level). No OS-crash or power-loss claim; the platform durability contract stays
  evidence-pending; the durability label must say so.

## Required changes

### R1. Workspace and crates

| Path | Contents |
|---|---|
| `experiments/unified-commitment/Cargo.toml`, `rust-toolchain.toml`, `.gitignore` (`/target/`), `Cargo.lock`, `README.md` | Workspace as described above |
| `crates/uc-core` | The transaction core: `Directory` (exclusive writer ownership of a storage directory through an OS-released advisory lock: open `<dir>/owner.lock` and hold `std::fs::File::try_lock()` (stable since Rust 1.89) for the `Log`'s lifetime; a second opener that gets `WouldBlock` is refused; the lock file's mere presence means nothing, so a dead owner's directory reopens without any stale-file deletion protocol), `Log` (append-only RF1 history file), `Transaction` request type (request id `Uuid`, payload bytes, requested `Durability::{D1, D2}`), `Outcome::{Committed { sequence, physical_ordinal, durability_time, achieved: Durability }, Rejected { reason }, Indeterminate { request_id }}`, the write path, recovery, retry resolution, checkpoint |
| `crates/uc-memory` | The Memory adapter: implements `cm_trace::run::Engine` over `uc-core` with a CMM2 payload (put/delete plus **incarnation** number per record) and the payload validator; exposes settings `D1` and `D2` |
| `crates/uc-harness` | Binary: `run TRACE.cmt OUT_DIR d1|d2 [--retain-store]` through `cm_trace::run::series_with_retention`, plus a `fault` subcommand used by the injected-termination tests (see R7) |

`uc-memory` and `uc-harness` path-depend on `../../../convergence-memory/crates/cm-trace`;
`uc-core` path-depends on `../../../exp-0001/crates/exp1-record-format` and
`../../../exp-0001/crates/exp1-raw-append-replay` (for `reopen_and_replay` only; the
sync-capable appender is new in `uc-core`). Do not modify anything under
`experiments/exp-0001/`. The only permitted change under `experiments/convergence-memory/` is
the backward-compatible runner parameterization in R8b.

### R2. Write path (`uc-core`)

`Log::commit(&mut self, txn: Transaction, validate: impl FnOnce(&State) -> Result<(), Rejection>) -> Result<Outcome, LogError>`:

1. **Resolve retry first.** If `txn.request_id` is already bound (see R5), return the recorded
   outcome for that binding without appending anything.
2. **Validate** the complete transaction against the coherent published state under the single
   writer's exclusive section, including effects of the transaction's own earlier operations
   (the caller closure sees the state and the payload). A rejection appends nothing.
3. **Assign** the next sequence (allocator above the greatest valid reservation high-water at
   open, gaps legal) and the next physical ordinals.
4. **Append** the complete recoverable transaction exactly in the R5 order, with the exact
   R5 §5 sequence for `D2`: `Binding` (request → event) then sync; `Reservation` then sync;
   `Provisional` (payload) then the pre-finalization sync; **then sample the realtime clock
   once**; construct `Final` (carrying that sample as `durability_time`) immediately followed
   by `Commit` (whose `final_crc32c` covers the `Final` bytes); then the post-finalization
   sync. For `D1` the same record order with no sync calls. The returned `durability_time` is
   the persisted sample; any post-commit observation time is reported separately and never
   written into the records.
5. **Publish** the new state atomically (a single swap of an `Arc`/immutable snapshot, or an
   equivalent that readers observe all-or-nothing) only after the durability boundary of the
   requested level.
6. **Return** `Outcome::Committed { …, achieved }` tied to the request identity, `Rejected`
   for step 2, or `Indeterminate { request_id }` when an I/O error occurs after the
   `Final`/`Commit` bytes were submitted but before the post-finalization sync returned.

**Fail-stop rule (all levels).** After any potentially mutating write or synchronization
failure at any step of 4 (short write, zero progress, I/O error, sync error), including
failures before the `Final`, the log is poisoned: every later `commit` returns
`LogError::Poisoned` without touching the file, until `Log::open` re-establishes a safe append
position from the validated prefix. Report the error outcome (`Rejected` for pre-`Final`
failures whose bytes cannot be canonical, `Indeterminate` around the commit boundary)
separately from writability. Errors before step 4 leave no record in the file; a poisoned
pre-`Final` failure leaves lifecycle-evidence-only records that recovery classifies as a
reported gap, never as a commit. Unit tests must inject short writes, zero-progress writes,
write errors and sync errors through a writer/syncer trait (the pattern of
`exp1-raw-append-replay`'s scripted writer) at every step and assert both the outcome and the
poisoned state.

### R3. Recovery (`Log::open`)

Scan from byte zero with `reopen_and_replay`; run `validate_lifecycle` on the accepted prefix;
classify per `CRASH-RECOVERY-CORRECTNESS.md` §2 using only what the bytes can decide:

- `CleanEof`, or `TerminalTruncation` (the last frame's extent crosses end of file and no bytes
  follow it): a torn tail. The incomplete suffix is dropped and reported in the `OpenReport`
  with its offset. This is the same outcome whether the tail came from an interrupted append or
  from a later truncation of the file; the bytes cannot distinguish them, and the plan does not
  claim to.
- `Failure` (interior damage with bytes remaining after it: CRC mismatch, malformed frame,
  lifecycle violation) or `IoFailure`: "corrupt or undecidable"; `open` fails closed with the
  offset and error. Never drop interior damage to salvage a prefix.
- Independent evidence: if the newest valid checkpoint records a commit position beyond the
  file's accepted prefix, the history has lost committed bytes; `open` fails closed
  ("history shorter than checkpoint") rather than silently reverting. This is the only case in
  which removal of a committed suffix is detectable, and the document must say so.

Rebuild published state by replaying only committed events in sequence order through the
caller's `apply` closure. Report gaps (reservation without commit) and unresolved bindings
(binding without commit) in the `OpenReport`. Two independent replays of the same file must
yield identical state digests (test). Determine the safe append position as the end of the
accepted prefix after dropping a torn tail (the file is truncated to that position and synced
before the first new append, and this is recorded in the report).

### R4. Record incarnation

The CMM2 payload carries `incarnation: u64` per record id, assigned by the adapter from its
published state (first insert = 1; delete then reinsert = 2, …). Field updates and relationship
changes name the incarnation they apply to; an update naming an old incarnation is rejected at
validation. The `uc-memory` state is keyed by `(id)` with the current incarnation, so a
recreated UUID never receives updates or edges intended for its predecessor.

### R5. Retry resolution and deduplication

Request identity is the `Binding.request_id`. `Binding.normalized_request` stores the
complete, versioned normalized request byte-for-byte, as R5 requires: `b"UCR1\0"`, one byte
for the requested durability (`1` = D1, `2` = D2), the payload length as `u64` little-endian,
then the payload bytes. The SHA-256 of that serialization is an auxiliary lookup key held in
memory and in the checkpoint cache, never a substitute for the retained bytes. R5 also requires the
complete opaque payload inside `Final.complete_envelope` together with the same event,
request, sequence and durability values as the outer record and any applicable provenance.
`Final.complete_envelope` is therefore the versioned envelope `UCE1`, little-endian, in this
order: `b"UCE1 "`; `request_id` (16 bytes); `event_id` (16 bytes); `sequence` (u64);
`durability_time` (i64, the persisted realtime sample); requested durability (1 byte);
`source_descriptor_count` (u16) followed by that many `(u16 length, bytes)` descriptors;
`reference_count` (u16) followed by that many `event_id` references (16 bytes each); then the
`UCR1` normalized request bytes (which contain the payload). This bounded core has no
provenance or causal references, so both counts are written as `0` (the explicit empty
representation) and the document says so. On every recovery, `uc-core` decodes `UCE1` from
each accepted `Final` and fails closed (`EnvelopeMismatch { physical_ordinal }`) unless its
`request_id`, `event_id`, `sequence` and `durability_time` equal the outer `Final`'s, its
`UCR1` bytes equal the bound `Binding.normalized_request` for that request, and its
requested-durability byte is a valid value equal to the durability byte inside those `UCR1`
bytes. `Commit` agreement (`event_id`, `sequence`, `final_ordinal`, `final_crc32c`) is checked
only when a `Commit` selects that `Final`, and that check is the frozen `validate_lifecycle`
rule. A valid `Final` with no complete adjacent `Commit` (absent, or truncated as a torn tail)
is uncommitted residue exactly as R5 prescribes: excluded from published state, reported in
the `OpenReport`, never an error. The `achieved` durability reported for a recovered commit is
derived from the validated bound request, not from either envelope byte alone. Unit tests
must exercise each envelope mismatch with valid RF1 CRCs (re-encode the frame after altering
the envelope), and R7 covers a `Final` with an absent `Commit` and with a truncated `Commit`
at both `D1` and `D2`. The payload is thus written three times per transaction (binding,
provisional, final), each independently framed.
The document records this write amplification (three payload copies plus framing and envelope
overhead) as a measured cost and lists payload-by-reference as a §18 follow-on requiring a
contract change.
Retention scope: **every binding present in the history file**, always. `Log::open` rebuilds
the full binding table from the scan even when it starts from a checkpoint; the checkpoint's
`resolved` table is only a cache that must agree with the scan (disagreement fails closed).
No expiry exists in this increment; the frozen lifecycle validator rejects duplicate bindings,
so a request id can never be re-bound while its binding is in the file. Expiry is deferred to
an explicit retention design (REQ-011) and the document says so. Resolution rules: same request
id with byte-identical normalized request → the recorded outcome, including the `achieved`
durability of the original commit; same id with a different normalized request (different
payload **or** different requested durability) → `Rejected { reason: RequestIdReuse }` with
nothing appended; an `Indeterminate` request id is resolved at reopen to `Committed` (if its
`Commit` is valid) or `Rejected` (if not), and that resolution is returned on retry. A caller
that wants a stronger durability for an already committed request must issue a new request id;
there is no upgrade protocol in this increment.

### R6. Checkpoint

`Log::checkpoint(&self, state_encoder) -> CheckpointRef` writes a checkpoint file
`<dir>/checkpoint-<sequence>-<physical_ordinal>.uc1` (new file, `create_new`, synced, then
the parent directory synced) containing: the exact commit position (sequence, physical
ordinal, byte offset after the `Commit` record), the SHA-256 of the accepted history prefix up
to that offset, the encoded state, the retry `resolved` cache (see R5: not authoritative), and
a CRC-32C of the checkpoint body. `Log::open` prefers the newest valid checkpoint whose history prefix hash still matches
the file bytes and replays only later committed events; a mismatch rejects the checkpoint
(reported) and falls back to full replay. History is never truncated by a checkpoint in this
increment (retention/compaction is a documented follow-on; REQ-011).

### R7. Recovery tests (the six plan scenarios) and the fault harness

`crates/uc-core/tests/recovery.rs` and `crates/uc-memory/tests/scenarios.rs` must contain, at
minimum, one test per plan scenario with the plan's required result asserted after reopen or
retry:

| Scenario | Required result |
|---|---|
| A field transaction fails conflict validation | Its proposed value is absent after reopen; nothing appended |
| A committed field transaction is followed by deletion | Reopen succeeds and the record stays absent |
| An older field transaction precedes whole-record replacement and checkpointing | The replacement remains current after reopen from the checkpoint and from full replay |
| A UUID is deleted and recreated | Old updates and edges do not affect the new incarnation |
| Execution stops during a batch or checkpoint | No partial committed transaction becomes visible; recovery uses a valid checkpoint/history combination |
| Commitment succeeds but the response is lost | Retrying or resolving the request returns the existing outcome |

Fault mechanism (label every test with it): **injected process termination** — `uc-harness
fault <scenario> <dir> <point>` runs the scenario in a child process that calls
`std::process::abort()` at the named injection point (after `Binding`, after `Reservation`,
after `Provisional`, after `Final` before `Commit`, after `Commit` before the post-finalization
sync, after the sync before publish, mid-checkpoint before its directory sync); the parent test
then reopens the directory and asserts the oracle class (Must recover / Must not be promoted /
Rejected / Corrupt-undecidable) for each point at both `D1` and `D2`. **Torn tail** — byte-level
truncation at every record boundary of a committed history and one byte inside the last
`Commit` (all torn tails: reopen succeeds, the incomplete suffix is dropped and reported).
**Interior damage** — flip one byte inside an interior `Final` and inside an interior
`Provisional` while preserving every later byte (both must fail closed with the offset).
**Lost committed suffix** — truncate a history below the position recorded by a valid
checkpoint (must fail closed with "history shorter than checkpoint"). **Lock ownership** —
a child process holding the directory keeps the parent's `open` refused; after the child
aborts, the parent opens. State in the test names and in the EXP-0003 document that these are
injected faults, not OS-crash or power-loss evidence.

### R8. EXP-0002 integration

`uc-memory` implements `Engine` for the CMT1 traces (same operations and outcomes as
`cm-candidate`, plus incarnation-aware validation) with two settings and labels:
`unified D1 ordinary writes, no sync, no crash-survival claim` and `unified D2 per-transaction
sync (R5 placements); tested failure model: injected process termination and torn tails;
platform durability contract evidence-pending; no OS-crash or power-loss claim`. `finish`
reports `replay`, `decode`, `rows`, `columns` stages as the candidate does, plus `checkpoint`
(time to write one) and `open_from_checkpoint`. Small and 1K traces must pass in both settings
as tests; the host runs the 1K and 10K series manually (commands in the README and EXP-0003
§15, Windows spelling).

### R8b. Shared runner metadata (the one permitted change under `experiments/convergence-memory/`)

`cm_trace::run::series_with_retention` hard-codes `experiment=EXP-0002`/`hypothesis=HYP-0002`
and hashes only the two Step 2 workspaces into `source.sha256`. Add a backward-compatible
`SeriesMeta { experiment: &str, hypothesis: &str, source_prefixes: &[&str] }` parameter (a new
function `series_with_meta`; the existing `series`/`series_with_retention` keep their
signatures and pass the EXP-0002 defaults so every existing caller and output stays
byte-identical, proven by a test). The source manifest must cover **tracked and untracked,
non-ignored** files under the prefixes (`git ls-files --cached --others --exclude-standard`),
and `source.patch` must be recoverable, not only identifying: it is `git diff HEAD --binary`
followed by one `git diff --no-index --binary /dev/null <file>` hunk (use `NUL` on Windows)
for every untracked, non-ignored file under the prefixes, so `revision` plus `source.patch`
reconstructs every manifest entry. Add a test that applies the retained patch to a fresh
`git worktree` of `revision` in a temporary directory and checks every manifest hash.
Evidence acceptance rule for the results index: a series is evidence only if its `porcelain`
is clean at `revision`, or its `source.patch` reconstructs every manifest entry (the host
verifies this before cataloguing a series). `uc-harness` passes
`experiment=EXP-0003`, `hypothesis=HYP-0003` and the prefixes
`experiments/unified-commitment/`, `experiments/convergence-memory/crates/cm-trace/`,
`experiments/exp-0001/crates/exp1-record-format/`, `experiments/exp-0001/crates/exp1-raw-append-replay/`.
Rerun the EXP-0002 proof chain after this change (it is in the Proof list).

### R9. CI

Add `experiments/unified-commitment` as a third matrix entry of
`.github/workflows/convergence-memory.yml` (rename the workflow file/name to cover both
experiments, or add a sibling workflow with the identical toolchain/offline pattern): fmt,
clippy `-D warnings`, tests `--workspace --all-targets --locked --offline`, including the
fault-injection tests (they spawn the workspace's own binary; make the binary path resolvable
via `CARGO_BIN_EXE_uc-harness` in integration tests).

### R10. Documents (same change, AGENTS §7)

- `docs/experiments/EXP-0003-unified-commitment.md` (18 sections, status `Ready`, predeclared
  correctness invariants = the oracle classes per injection point, predeclared metrics = the
  EXP-0002 stage set plus checkpoint/open stages, environment requirements, raw result
  locations under `docs/experiments/EXP-0003/results/` with the same committed-evidence subset
  rule as EXP-0002 and the same accepted R7-style deviation for raw per-op samples).
- `docs/hypotheses/HYP-0003-unified-commitment.md` (Open): a single RF1 recoverable-commit
  history with per-transaction sync recovers every plan scenario under the injected failure
  model and serves the Memory traces at a declared D2 level.
- `docs/adr/`: one ADR, following the existing file naming there, recording the decisions in
  "Design decisions settled by the host" (transaction = one event, R5 lifecycle reuse, D2
  placements, payload-agnostic core, incarnation, retry retention scope, checkpoint format
  `uc1`), status Proposed with the merge plan as authority.
- `docs/GLOSSARY.md` bullets: **incarnation**, **request identity** (if absent),
  **indeterminate outcome**. `docs/TRACEABILITY.md` and `docs/RESEARCH-QUESTIONS.md` rows for
  HYP-0003/EXP-0003. `docs/experiments/README.md` registry row. `docs/PROJECT-STATUS.md`
  section. `experiments/README.md` and root `README.md` current-phase paragraph. `AGENTS.md`
  §10: one sentence authorizing EXP-0003 (bounded D1/D2 transaction core, injected-fault
  recovery tests, manual descriptive measurements) by reference to merge plan step 3, leaving
  the rest intact; keep the existing "power-loss / OS crash" exclusions.
- Everything under `experiments/exp-0001/`, `experiments/convergence-memory*/`,
  `docs/experiments/EXP-0001/` and `docs/experiments/EXP-0002/` stays unchanged.

## Non-goals

- No multi-client concurrency (one writer, one process), no D3 group commit, no network, no
  legacy adapter integration (merge plan step 4), no compaction/retention, no schema catalog.
- No OS-crash, power-loss, "stable media" or "exactly once" claims; no platform durability
  contract measurement.
- No changes to the RF1 codec, to exp-0001 crates, or to Step 2 workspaces.

## Proof

```
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/unified-commitment/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/convergence-memory/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/exp-0001/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline
python tools/validate_markdown_links.py
git diff --check
```

The legacy workspace is untouched by this work order and is not part of its proof. The host
additionally runs the 1K series in `d1` and `d2` on both engines' evidence rules, the 10K
series once in `d2`, and the fault-injection matrix, and records them in EXP-0003's results
index. Each recovery scenario must be shown to fail (or to be unreachable) when its mechanism is
disabled (state how you checked).
