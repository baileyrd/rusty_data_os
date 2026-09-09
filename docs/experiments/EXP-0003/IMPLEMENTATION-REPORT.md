# EXP-0003 implementation report

Advisory implementation report for independent provider review. No commit, push or publication.

Checkout: `C:/dev/rusty_data_os-step3`, branch `codex/merge-step3-unified-commitment`,
base `e62fa699d3b264c0717428f86c0d11936ac3688a`. The checkout was initially clean.
The supplied work-order file was read only to verify SHA-256
`4fa291fd472764c6746e841607a151e71e3943058e249f6a4378a92bb507c043`; it matched.
All source paths were resolved in this checkout. The original and Step 2 checkouts were not edited.

## Delivered behavior

- uc-core: one OS-locked writer, immutable state publication, RF1 five-frame transaction,
  exact D2 sync placements and persisted realtime sample, fail-stop I/O, byte-zero replay,
  binding retention/retry, envelope verification, gaps/residue, uc1 checkpoint and prefix validation.
- uc-memory: CMM2 operations/validation, incarnation/tombstones/edges, all shared CMT1
  operations, independent row/column rebuilds, checkpoint/open stages and amplification counts.
- uc-harness: D1/D2 shared series metadata, optional forensic retention, exact-point child aborts.
- R8b: old runner entry points/default identity retained; binary patches include untracked source.
  A fresh-worktree reconstruction test verifies every manifest entry, including a binary fixture.
- CI third matrix entry and synchronized Ready/Open/Proposed experiment, hypothesis and ADR.
  No EXP-0001 source or earlier EXP-0001/EXP-0002 retained evidence was changed.

## Fix round 1 — P1–P9

The host accepted UC-01–UC-09 on the prior snapshot. These changes implement the
specified dispositions; earlier resolved findings were not reopened without new evidence.

| Item | Change and regression coverage |
|---|---|
| P1 | New requests containing literal payload `RDE1` fail before validation/write with the specified Invalid reason. Normalized-request decoding rejects old bindings with these bytes as corrupt, using a valid-CRC fixture. Both levels are tested. CMM2 hex payloads cannot contain the sequence. A legitimate UUID containing the magic at frame offset 32 still makes a truncated Binding corrupt/undecidable; a second test proves this residual limitation. README, ADR and EXP §9/§14/§18 disclose it. |
| P2 | Checkpoint creation first synchronizes the history writer at either level. Writer::synchronize takes &self, matching File::sync_all and preserving Log::checkpoint(&self). The placement test now checks a history sync for D1 and D2 checkpoints. HistoryShorterThanCheckpoint still fails closed; documents name the manual newer-checkpoint removal remedy and its explicit loss of newer state. |
| P3 | Deviation 1 and EXP §14 disclose fail-closed series behavior for unavailable Git and output inside a source prefix, including EXP-0002 callers. |
| P4 | The harness constructs one revision-bound label per series and passes it to each engine. A two-mode, six-trial regression compares environment, throughput, summary, results and observations identities. |
| P5 | Removed the redundant lifecycle pass and its cloned record vector; the frozen scanner remains the lifecycle authority. Replay I/O failures now preserve offset and ErrorKind in LogError::Io, separately from Corrupt. A directory-as-history regression exercises the I/O path. Non-scan I/O errors use offset zero. |
| P6 | Adapter auto request IDs use domain 0x41 instead of explicit-ID convention 0x52. A regression inserts explicit ID 6 before automatic ordinal 6, then verifies both records after reopen. README/ADR state auto IDs are conveniences, not stable identities across a torn-tail reopen. |
| P7 | Real-checkout reconstruction skips with a printed reason when selected prefixes have uncommitted changes. The tracked-edit/untracked-binary fixture remains mandatory. The disposable repository sets core.autocrlf=false; the CI comment states the shallow-checkout object assumption. No source Git metadata is modified. |
| P8 | README and EXP §10/§11 disclose the lifetime-resident accepted prefix, retained normalized requests, two slot-map clones inside complete_operations, and the roughly 200,000-transaction scan cap. EXP §18 lists check-only validator, incremental prefix hash and persistent state structure. |
| P9 | Glossary additions follow the preamble alongside integrity vocabulary; project status follows its header metadata; RQ-005 follows RQ-004. |

The host-approved payload ban narrows the opaque-payload contract. It cannot fulfill an
unqualified guarantee that every valid-frame torn tail reopens: arbitrary UUID/envelope
bytes remain outside that ban. The proposed follow-on is a codec-level contract change
that distinguishes those cases; this revision keeps the frozen codec and fails closed.
The regression records that remaining limitation rather than silently restricting UUIDs.

Fix-round changes are confined to the existing README, ADR, experiment/report, glossary,
project-status and research-question documents, workflow comment, shared runner test,
uc-core lib/envelope/recovery/checkpoint and recovery tests, uc-memory lib/scenario tests,
and uc-harness main/test. A new proof transcript is retained below. Host series entries
remain empty; prior host observations are not catalogued as evidence for these fixed binaries.

## Fix round 2 — Q1–Q4

The host accepted UC3-01–UC3-04 after independently passing the prior 29/17/95-test
snapshot. This round implements those dispositions without reopening resolved findings.

| Item | Change and regression coverage |
|---|---|
| Q1 | Both snapshot diffs explicitly disable autocrlf/safecrlf and force standard prefixes, retaining working-tree bytes. The always-run fixture adds CRLF-modified tracked text with source and restore configured autocrlf=true, safecrlf=true and alternate diff prefixes, and verifies every manifest hash after patch application. EXP §15 and the results index record the requirement. |
| Q2 | Binding/Reservation/Provisional hooks follow successful sync at D2, and append at D1. The placement-log test asserts this order; the existing 14-abort matrix exercises all named points. README and EXP §9 state every hook's exact placement. |
| Q3 | Log::create exclusively creates a missing storage directory/history; Log::open never creates either and reports NotFound for absence. Directory::acquire no longer creates directories. Memory and fault setup use explicit create; recovery uses open. A core regression checks missing directory/history, no side effects, exclusive creation, and successful reopen. The runner's default creates_store_directory=false preserves existing callers; unified opts in to creating its own store. Cleanup tolerates a factory that failed before creating its store. |
| Q4 | ADR authority and the AGENTS §10 EXP-0003 sentence record host acceptance of disposition P1 on 2026-09-09 under owner step 3/work-order authority; the host later committed the disposition record in-repo as [HOST-DISPOSITIONS.md](HOST-DISPOSITIONS.md). The residual UUID/envelope ambiguity and codec follow-on remain recorded. |

This round changes AGENTS.md, ADR-0003, EXP-0003 and its results-index/report documentation,
the unified README, cm-trace/src/run.rs, uc-core/src/lib.rs and tests/recovery.rs,
uc-memory/src/lib.rs and tests/scenarios.rs, and uc-harness/src/main.rs and tests/faults.rs.
The new proof transcript is PROOF-fix-round2.txt. No other source scope is widened.
The explicit create/open split is the host-authorized Q3 API change from the original
create-on-open design; the apply/state-decoder arguments remain necessary and unchanged.
No additional deviations or tool-action denials occurred in this round. Host series entries
remain empty for the host to bind to its source snapshot and patch identity.

## Proof

Fix-round 2 agreed proof chain: **PASS, exit 0, 2026-09-09**. All 142 tests passed;
no failures, ignored tests or warnings in the final chain. Documentation was finalized
after that run, then the Markdown-link and whitespace checks were repeated successfully.

The exact eleven-command `&&` chain was run from the checkout root, using
`+1.89.0-x86_64-pc-windows-gnu`, the supplied manifests/flags, and the requested
`--exclude exp1-descriptive-d1-harness` for portable EXP-0001 checks.
Full fix-round 2 output: [PROOF-fix-round2.txt](PROOF-fix-round2.txt).
Fix-round 1 output remains in [PROOF-fix-round1.txt](PROOF-fix-round1.txt).
The prior delivered snapshot's passing output remains in [PROOF.txt](PROOF.txt).
All retained proof transcripts now use LF; their historical output is otherwise unchanged.

Before this fix round, the [first attempt](PROOF-attempt1.txt) passed unified correctness but stopped at
EXP-0002 Clippy's `nonminimal_bool` warning in the Git helper. The expression was fixed
without suppressing the lint. The [intermediate chain](PROOF-before-retry-regression.txt)
passed all gates before a final retry-precedence regression was added: an existing bound
request ID with an oversized different payload returns RequestIdReuse, not the new-request
size error. Its targeted core suite passed; the final complete chain includes that change.

| Suite | Tests / coverage | Final result |
|---|---|---|
| Unified commitment | 19 core + 3 harness + 8 Memory tests; 30 total | Pass |
| EXP-0002 candidate/shared workspace | 17 tests, including source patch reconstruction | Pass |
| Portable EXP-0001 | 95 existing tests | Pass |
| Formatting, Clippy, Markdown links, diff whitespace | All requested gates | Pass |

The injected writer matrix covers 40 write cases (short/zero/error/error after submission,
five frames, two modes) plus four D2 sync failures. Every failure checks both outcome and
poisoning, no subsequent writes, reopen and safe continuation. Successful call tracing
checks D1 `[1,2,3,5,6]` and D2 `[1,sync,2,sync,3,sync,5,6,sync]` and three payload copies.
Envelope mutations have freshly valid RF1 CRCs and cover selected and unselected Finals.
The process matrix covers seven placements at both modes (14 aborts), plus independent
owner-abort lock release. Parent-captured markers prove the requested placement occurred.
Tests assert EOF/Commit-tail handling, interior Final/Provisional failures with offsets,
checkpoint fallback/cache disagreement/history shortening, and deterministic full replay.
Small and 1K traces compare every operation and both reconstructions with the independent oracle.

These are correctness checks. No manual benchmark series, remote CI, OS-crash or power-loss
test was run by this agent. The host reported a valid D1 series on the prior snapshot;
its 1K D1/D2 and 10K D2 reruns on these fixed binaries remain pending in the
[results index](results/README.md); the commands are in EXP-0003 §15 and the workspace README.

## Disabled-mechanism controls

Controls operate on the same test payloads/history; no faulty production mode was introduced.

| Merge-plan scenario | How disabling/bypassing the mechanism is checked |
|---|---|
| Conflict validation | Direct in-place application of the two-operation payload leaks its first field update when the second rejects; staged Log publication leaves the original state unchanged. |
| Field update then delete | Replaying actual committed history while filtering out Delete resurrects the updated row; normal replay stays absent. |
| Older field change then replacement/checkpoint | Reapplying the actual older field event after rebuilding the replacement changes the replacement incorrectly; checkpoint and full replay keep it current. |
| Delete/recreate UUID | Stale incarnation 1 update/edge requests reject at generation 2; deliberately rebinding the update to generation 2 accepts and changes the new record, demonstrating why the original tag must be retained. |
| Stop during batch/checkpoint | Applying the unselected batch payload exposes both uncommitted changes; normal truncated-Commit replay exposes none. Removing independent checkpoint evidence makes a lost committed suffix undetectable, whereas the valid checkpoint forces failure. Partial newest checkpoints fall back to the prior validated position plus history. |
| Response lost | Reusing the original request returns its existing outcome without bytes; bypassing identity resolution with another ID revalidates the same insert and rejects Duplicate rather than returning the recorded outcome. |

## Deviations and precise interpretations

1. **R8b artifact identity conflict.** Literal byte identity for all old outputs conflicts with
   adding missing untracked-file patch hunks and preserving patch newlines. The implemented
   deviation keeps API signatures, CMT1 operation/results behavior, EXP-0002 default metadata
   and source prefixes, while correcting source.patch/source.sha256 retention. The default
   identity header is tested byte-for-byte; existing runner conformance/retention tests remain.
   The old identifying-only patch behavior is not preserved. This was reported before implementation.
   Series now fail closed when `git` is unavailable or output lies inside a source prefix
   (required by the evidence rule); this also applies to EXP-0002 callers.
2. **Source-output self-reference.** Series output inside a selected source prefix cannot
   supply a reconstructable manifest of itself. Such destinations fail explicitly before
   trials; operator commands use an external temporary output directory. Source names with
   newlines/non-UTF-8 fail explicitly rather than being silently omitted.
3. **Process-test placement.** Cargo supplies `CARGO_BIN_EXE_uc-harness` to integration tests
   of the uc-harness package. The child-process matrix therefore lives there; required
   uc-core/tests/recovery.rs and uc-memory/tests/scenarios.rs retain their core/Memory cases.
4. **R10 exclusion versus R8b.** The specific R8b authorization is the sole exception under
   convergence-memory: only cm-trace/src/run.rs changed. The general unchanged-path wording
   is preserved everywhere else. The legacy workspace and all EXP-0001 code remain untouched.
5. **EOF and recovery labels.** Exact record-boundary EOF is clean, with no nonexistent suffix
   reported as dropped. Incomplete Commit tails report/truncate/sync the accepted offset.
   D1 Committed denotes its ordinary-write protocol, not durable canonical evidence. Recovered
   achieved mode comes from the validated binding; bytes do not prove a prior sync returned.
   Setup, tail repair and explicit checkpoints synchronize at both levels; D1 appends do not.
6. **Retained amplification counts.** Each trial adds store.amplification.txt beside its
   removable store, so physical/payload byte counts survive default cleanup. These counts
   are not mislabelled as nanosecond samples. Raw per-operation sample retention follows
   the work order's accepted EXP-0002/R7-style external-path/SHA-256 deviation.

The frozen RF1 codec, record kinds, UCR1/UCE1 serialization and R5 transaction placements
were not redesigned. Empty provenance/reference counts are explicit. No production
graduation, D3, compaction, expiry or legacy feature integration is included.

## Denied actions and platform findings

Fix round 1 had no denied tool actions or unresolved proof blockers. The findings below
are retained from the original implementation; no original checkout was edited this round.

- Initial PowerShell profile startup attempted external theme-cache/registry writes and was
  denied by the environment. Subsequent commands used pwsh without loading that profile.
- A disposable safe-Rust Windows probe found file sync succeeds; read-only directory sync
  fails with error 5; writable directory handles with BACKUP_SEMANTICS succeed. The required
  access/flag selection follows Microsoft's
  [FlushFileBuffers documentation](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)
  and [directory-handle documentation](https://learn.microsoft.com/en-us/windows/win32/fileio/obtaining-a-handle-to-a-directory).
  This establishes callable API behavior here, not namespace/power-loss survival evidence.
- Preliminary tail-repair tests found Windows append-only handles reject set_len (error 5).
  Open now repairs/synchronizes using a writable recovery handle, then opens the append handle;
  all tail/fail-stop cases pass. The contract was preserved.
- The local Git clone transport was blocked by the sandbox's CreateFileMapping error 5.
  The reconstruction test instead initializes a disposable repository with read-only object
  alternates, sets only its own reference to the existing revision, and adds fresh worktrees
  there. It creates no commits and modifies no original-checkout Git metadata.
- No escalation, commit, push, publication or network dependency fetch was requested.
  No unresolved environment blocker remains for the agreed portable proof. Remote CI and
  manual descriptive series remain unexecuted, as assigned to the host.

## Files changed

11 existing tracked files changed and 28 new files, all listed below. Final HEAD remains
`e62fa699d3b264c0717428f86c0d11936ac3688a`; no changes under the frozen excluded paths.

Existing tracked files:

```text
.github/workflows/convergence-memory.yml
AGENTS.md
README.md
docs/GLOSSARY.md
docs/PROJECT-STATUS.md
docs/RESEARCH-QUESTIONS.md
docs/TRACEABILITY.md
docs/adr/README.md
docs/experiments/README.md
experiments/README.md
experiments/convergence-memory/crates/cm-trace/src/run.rs
```

New files:

```text
docs/adr/ADR-0003-unified-commitment.md
docs/experiments/EXP-0003-unified-commitment.md
docs/experiments/EXP-0003/IMPLEMENTATION-REPORT.md
docs/experiments/EXP-0003/PROOF.txt
docs/experiments/EXP-0003/PROOF-fix-round1.txt
docs/experiments/EXP-0003/PROOF-fix-round2.txt
docs/experiments/EXP-0003/PROOF-attempt1.txt
docs/experiments/EXP-0003/PROOF-before-retry-regression.txt
docs/experiments/EXP-0003/results/README.md
docs/hypotheses/HYP-0003-unified-commitment.md
experiments/unified-commitment/.gitignore
experiments/unified-commitment/Cargo.lock
experiments/unified-commitment/Cargo.toml
experiments/unified-commitment/README.md
experiments/unified-commitment/rust-toolchain.toml
experiments/unified-commitment/crates/uc-core/Cargo.toml
experiments/unified-commitment/crates/uc-core/src/checkpoint.rs
experiments/unified-commitment/crates/uc-core/src/envelope.rs
experiments/unified-commitment/crates/uc-core/src/lib.rs
experiments/unified-commitment/crates/uc-core/src/recovery.rs
experiments/unified-commitment/crates/uc-core/src/sha.rs
experiments/unified-commitment/crates/uc-core/tests/recovery.rs
experiments/unified-commitment/crates/uc-memory/Cargo.toml
experiments/unified-commitment/crates/uc-memory/src/lib.rs
experiments/unified-commitment/crates/uc-memory/tests/scenarios.rs
experiments/unified-commitment/crates/uc-harness/Cargo.toml
experiments/unified-commitment/crates/uc-harness/src/main.rs
experiments/unified-commitment/crates/uc-harness/tests/faults.rs
```

Build output remains ignored under target/. No generated workload or benchmark series is
catalogued as evidence by this implementation report.
