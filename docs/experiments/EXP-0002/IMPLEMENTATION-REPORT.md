# EXP-0002 implementation report

Implemented in `C:/dev/rusty_data_os-step2`; no commit, push, publish, PR, or remote
message was performed. N1-N8 separate replay/decode attribution, cache input
identity, repair Windows probes, extend text guards, and complete documentation
and traceability. The structural integrity profile remains unchanged.
Candidate: **D1 ordinary writes, no fsync, no crash-survival claim**.

## Scope delivered

Two standalone Rust 1.89 workspaces; external-dependency-free candidate path graph;
legacy pinned to reviewed/pushed Step 1 part A
`abda0a7e94a9727e410001e9724edc741f6e0d31` (not represented as merged);
CMT1 versioned traces/results and SHA-256 golden fixtures; independent per-field
model plus independent Python fixture arithmetic; all thirteen Memory fields;
small/1K/10K/100K generators; candidate RF1 D1 append/replay and independent row
and column reconstruction; native legacy single and protocol-22 dispatch;
manual six-trial drivers with raw outcomes, latency/stage samples, file sizes,
RSS, source/environment identity and retained failures; CI and synchronized docs.
Default retention now removes stores after validation and records trace
regeneration metadata without trace/source copies. Both CLIs accept trailing
`--retain-store` for forensic stores and the trace copy. Per-trial throughput
files reference operator-held raw results by absolute path and SHA-256.

The frozen work-order copy in this checkout matches supplied SHA-256
`24e257e7f8f404d00dd35278f9e9fe9baf3faf677920a8c0d1b30e69220495fc`.
The moved merge plan is byte-identical to its original tracked version.
No source path was resolved to or edited in `C:/dev/rusty_data_os`.

## N1-N8 proof output

The exact agreed proof chain ran from this checkout's root and exited **0**,
using `+1.89.0-x86_64-pc-windows-gnu`, every specified locked/offline flag, and
the agreed Linux-only harness exclusion on EXP-0001 clippy/test. Complete
[stdout/stderr](results/proof-windows-n1-n8.txt) is retained separately from prior logs.

| Gate | Result |
|---|---|
| Candidate fmt / clippy / tests | Passed; 15 tests, 0 failures |
| Legacy locked offline fetch / fmt / clippy / tests | Passed; 2 tests, 0 failures |
| EXP-0001 fmt / portable clippy / tests | Passed; 95 tests, 0 failures |
| Markdown links / git diff --check | Passed |

N1-N8 proof log SHA-256: `e299c826c2d71c8110fbdecc0b6f5409d2ec034ea7d0b5d7ff9048fd6dca88a9`.
The new tests cover RSS output shape, original-byte input identity and baseline
capture, plus all-artifact candidate labels and separate summary stage rows.
Small/1K tests cover the added text guards in both engines and all legacy modes.
Documentation links and whitespace were checked again after refreshing this report.

## N1-N8 implementation status

- N1: candidate `replay` times only `reopen_and_replay`; `decode` times CMM1
  payload parsing separately, before unchanged `rows`/`columns` rebuilds.
  `append` is unchanged. Series regression coverage asserts five measured samples
  for each stage in the summary. README and EXP-0002 declare Structural frames
  store/verify no CRC-32C; a CRC-32C profile cell remains a follow-on.
- N2: `LoadedTrace` hashes original input file bytes once, preserves immutable
  decoded input identity, and passes it to metadata and all trials. The decoder
  is unchanged. Post-load RSS baseline precedes the first trial and metadata
  probes. Whole-trace/history buffering and the inspector's unmeasured 10–15 GB
  per-trial 100K estimate are documented; a streaming loader/decoder is deferred.
- N3: Windows RSS and metadata probes try `pwsh`, then `powershell`; unavailable
  reasons remain explicit. RSS returns positive byte counts or `unavailable:`
  without panicking; Linux VmHWM is converted from KiB to bytes. This uses the
  requested shell fallback approach, with no new dependency or unsafe FFI.
- N4: workflow added; not yet executed remotely; the legacy leg assumes anonymous
  fetch of two GitHub git sources (`baileyrd/rusty_multimodal_db` and
  `Rusty-Mill/rusty_mill`). The workflow itself is unchanged in this round.
- N5: small trace has 59 operations, including failed and successful text status
  guards with subsequent gets; seeded traces guard status for every identity in
  both rounds. Golden CMT1/results and SHA-256s are regenerated, with hand-checked
  indices/status assertions updated. `update` remains access_count-only because
  the pinned legacy `update_field` exposes no other field through that path.
- N6: EXP-0002 section 15 includes exact Windows generation and candidate/legacy
  single/pipelined/atomic run commands for 1K/10K and the Linux toolchain note.
- N7: TRACEABILITY registers HYP-0002/EXP-0002 under merge-plan step 2; research
  questions link them to RQ-001 and RQ-004. Host-retained series are distinguished
  from repository entries; no performance conclusion is asserted.
- N8: candidate label identifies RF1 full after-images, Structural integrity and
  the process-cached git revision. D1 remains the separate durability declaration.
  Tests check the prefix and identical labels across every series artifact.

Before merge (owner action, not performed here): push the branch and record the
Actions run URL for both matrix legs.

The host's four definitive candidate/legacy-single 1K/10K series were produced
on snapshot `850eb388…9e6f` binaries. N1's stage split and N3's RSS fix post-date
them; N5 also changes trace bytes/operation mix. Those older series cannot be
relabeled with the new stage or RSS semantics or new input hashes. The host
filled the index entries after this round, identifying each source snapshot, input hash
and the later 1K re-run pair. This report makes no performance interpretation.

N2 interpretation qualification / proposed deviation: subtracting the post-load
process high water gives growth above that baseline, not an isolated trace/oracle
or engine allocation. The oracle does not exist at load time, and high-water
marks are not additive. The requested baseline is implemented; documentation
states this measurement limit instead of claiming an impossible decomposition.
No requested implementation action was denied or blocked. No network fetch,
commit, push or publication was attempted. **100K was not executed.**

## M1-M6 proof output

The complete agreed proof chain ran from this checkout's root and exited **0**,
using `+1.89.0-x86_64-pc-windows-gnu`, every specified locked/offline flag, and
`--exclude exp1-descriptive-d1-harness` on EXP-0001 clippy/test. Complete
[stdout/stderr](results/proof-windows-m1-m6.txt) is retained with LF endings.

| Gate | Result |
|---|---|
| Candidate fmt / clippy / tests | Passed; 12 tests, 0 failures |
| Legacy locked offline fetch / fmt / clippy / tests | Passed; 2 tests, 0 failures |
| EXP-0001 fmt / portable clippy / tests | Passed; 95 tests, 0 failures |
| Markdown links / git diff --check | Passed |

M1-M6 proof log SHA-256: `a5e877048249f8f70cd370277dd9619e9c3b69c236d07aeb0722e7d7b5f54f53`.
All files edited in this round are UTF-8 with LF endings. Prior proof logs are
preserved byte-for-byte, including the initial log's original line endings.
Both protected EXP-0001 trees remain unchanged. No dependency lockfile changed.


## M1-M6 implementation status

- M1: `Results.durability` and `Engine::durability` declare each engine's own
  setting. Canonical results decode non-empty hex-encoded UTF-8 declarations;
  diagnostics, environment, throughput and summary retain adjacent candidate
  reference and engine durability lines. Candidate and legacy tests assert the
  decoded declarations. The independent Python fixture now declares model-only
  execution, and its results/SHA-256 manifest were regenerated. The previous
  fixture is superseded within this same uncommitted change; CMT1-results stays
  revision 1, with no trace format or query-signature change.
- M2: CI comments and the legacy README name both locked Git sources and their
  reachability/token assumption. No remote workflow execution is claimed.
- M3: public `label_for(mode)` supplies both `Legacy::label` and the CLI's series
  label; dispatch modes are tested against the canonical observations header.
- M4: operation observations are written directly to the exclusive output file
  as produced. The final actual record count completes the reserved header with
  a 64 KiB compaction buffer; no observed-operation vector is accumulated.
  Tests compare streamed bytes with the canonical encoder for the golden fixture,
  zero records/operations, 100 records and multi-buffer operation output, and
  reject incomplete/overfull output and invalid durability encodings. Interrupted
  output remains incomplete evidence. README and EXP-0002 sections 10/15 record
  host-observed raw/store sizes and remaining large-trace memory/retention limits;
  section 18 records digest collapse as a follow-on, without changing signatures.
- M5: the validator docstring and AGENTS validation instructions document existing
  tracked plus non-ignored untracked Markdown scanning and clean-CI behavior.
- M6: Memory/B0 use glossary bullets; HYP-0002 replaces the former experiment-local
  hypothesis identifier in the filename, links and environment metadata. The
  workspace README includes a one-line Python hex-decoding example.

No new deviation, dependency, denied or blocked action was required for M1-M6.
Previously accepted native-durability, internal-stage and batch-width limitations
remain. Host series entries were filled by the host after this round (four series on
snapshot 850eb388 binaries, a 1K re-run pair on bd99987b binaries). No measurement series or 100K run was executed in this fix round.

## L1-L4 proof output

The full agreed GNU 1.89.0/offline proof chain was rerun from this checkout's root
and exited **0**. [Full L1-L4 stdout/stderr](results/proof-windows-l1-l4.txt).

| Gate | Result |
|---|---|
| Candidate fmt / clippy / tests | Passed; 11 tests, 0 failures |
| Legacy locked offline fetch / fmt / clippy / tests | Passed; 2 tests, 0 failures |
| EXP-0001 fmt / portable clippy / tests | Passed; 95 tests, 0 failures; Linux-only harness excluded as agreed |
| Markdown links / git diff --check | Passed |

The retention suite now has four tests: existing all-mismatch retention and
overwrite refusal, default/forensic series layouts with source hashes and raw
path/SHA-256 references, invalid-series cleanup with retained diagnostics, and
trailing-flag parsing. Store bytes are asserted before cleanup. All source and
documentation files changed in this correction round, and this new proof log,
use UTF-8 and LF. Protected EXP-0001 trees remain unchanged. No measurement
series was run by this correction; no new dependency or format was introduced.

L1-L4 proof log SHA-256: `1230ba9e3ca8996a3a4699b43b9b83c939d3b4ab5fe834e4a8ed2d60b7b25a46`.

## Prior proof output

The user's complete chained proof command ran from the repository root with
`+1.89.0-x86_64-pc-windows-gnu`, all specified `--locked --offline` flags, and
`--exclude exp1-descriptive-d1-harness` on EXP-0001 clippy/test exactly as requested.
The chain exited **0**. [Full captured stdout/stderr](results/proof-windows.txt).

| Gate | Result |
|---|---|
| Candidate fmt / clippy | Passed; warnings denied |
| Candidate workspace tests | 8 passed, 0 failed |
| Legacy locked offline fetch | Passed from host-prefetched cache |
| Legacy fmt / clippy | Passed; warnings denied |
| Legacy workspace tests | 2 passed, 0 failed; small and 1K in single, pipelined and atomic modes, plus dependent multi-write batches |
| EXP-0001 fmt / portable clippy | Passed; warnings denied |
| EXP-0001 portable tests | 95 passed, 0 failed |
| Markdown link validation | All repository-relative links resolve |
| git diff --check | Passed |

Candidate tests include oracle and reconstruction equivalence, an independently
computed golden result, hand-checked aggregates/page ordering, malformed input,
physical truncation/corruption, and continued failure retention with overwrite
refusal. An initial development test exposed an undersized RF1 retained-frame
budget in the new harness; it was corrected to the actual history size before
this successful proof. This was a harness defect, not an engine performance result.

No files under `experiments/exp-0001/` or `docs/experiments/EXP-0001/` changed.
The unchanged full Linux EXP-0001 workflow remains the Linux-only harness gate.
The added workflow has not been remotely executed.

## L1-L4 implementation status

- L1: minimal default series retention; scoped store cleanup after engine drop;
  forensic opt-in for stores and trace; source hashes/patch without source copy;
  raw-file paths/SHA-256s in throughput; committed subset and host-entry template.
- L2: workflow/legacy README state the GitHub reachability and repository-access
  assumption, conditional authenticated-token setup, and no remote workflow run.
  Host series entries are filled by the host when collected.
- L3: README documents hex-encoded trace sizes and the whole-trace memory
  cost at 100K. Streaming is a follow-on; no CMT1 format change.
- L4: status text describes the implemented state without a review-pending label.

## Deviations and unresolved measurement limits

1. **Equal native durability is impossible through the pinned public adapter.**
   Pinned `generic/mmap_store.rs` documents and implements insert/replace/delete
   log `sync_data`; access_count updates write mmap slots without per-op flush.
   No public setting disables those log syncs. The reported/proposed deviation is
   to keep the pin intact and retain separately labelled native diagnostic cells.
   No equal-durability winner or power-loss claim is permitted.
2. **Legacy internal stage timings are unavailable.** Its public adapter exposes
   complete operations and queries, not isolated append/replay/row/column timings.
   Those missing stages carry a reason; they are not fabricated as zero or
   substituted with candidate timings. Candidate stages are timed separately.
3. **Batch measurement width is one.** This preserves complete per-operation
   latency including dispatch. Actual multi-write batches with earlier-write
   dependencies are correctness-tested in both modes. Batch-size scaling and
   amortized batching throughput are not measured in this increment.
4. **The link validator required checkout support.** The original tracked-only
   scan crashed on the required unstaged plan move and missed added documents.
   It now checks existing tracked and untracked Markdown while still rejecting
   references to deleted destinations. No staging or commit was needed.
5. **N2 RSS baseline interpretation is qualified.** The requested baseline is
   present, but subtraction is growth above a process high water, not exact
   trace/oracle subtraction or isolated engine RSS. Oracle allocation starts
   after load. This limitation and the proposed interpretation were reported
   before implementation and are documented in EXP-0002 section 10.

L1 host disposition: host 1K series completed, both engines valid; retention changed per L1.
No new work-order deviation was required for L1-L4. The previously documented
native durability, unavailable legacy stage timings and batch-width limitations
remain unchanged.

## Denied or blocked actions

No requested L1-L4 or M1-M6 action was denied or blocked. No network fetch was attempted
in this sandbox; the full proof uses the agreed host-prefetched offline cache.
The host reports its independent networked locked fetch passed. Repository
visibility/access from GitHub Actions remains an explicit unverified assumption;
no remote workflow was run and no credentials were configured. No dependency,
commit, push, publication or edit in the original checkout was introduced.

Host measurement entries are maintained in the results index using its template.
The exact manual commands are in the [driver README](../../../experiments/convergence-memory/README.md).
**100K was not executed.** The experiment remains Ready, hypothesis Open, without
a performance conclusion. Automated correctness-test timings in proof logs are
not benchmark evidence.

## Files changed

- Modified: [AGENTS.md](../../../AGENTS.md).
- Modified: [README.md](../../../README.md).
- Modified: [crates/README.md](../../../crates/README.md).
- Deleted old location: `data-os-multimodal-merge-plan-2026-09-08.md` (byte-identical plan moved below).
- Modified: [docs/GLOSSARY.md](../../GLOSSARY.md).
- Modified: [docs/PROJECT-STATUS.md](../../PROJECT-STATUS.md).
- Modified: [docs/experiments/README.md](../README.md).
- Modified: [experiments/README.md](../../../experiments/README.md).
- Modified: [tools/validate_markdown_links.py](../../../tools/validate_markdown_links.py).
- Added: [.github/workflows/convergence-memory.yml](../../../.github/workflows/convergence-memory.yml).
- Added: [docs/experiments/EXP-0002-convergence-memory.md](../EXP-0002-convergence-memory.md).
- Added: [docs/experiments/EXP-0002/IMPLEMENTATION-REPORT.md](IMPLEMENTATION-REPORT.md).
- Added: [docs/experiments/EXP-0002/results/README.md](results/README.md).
- Added: [docs/experiments/EXP-0002/results/proof-windows.txt](results/proof-windows.txt).
- Added: [docs/hypotheses/HYP-0002-memory-convergence.md](../../hypotheses/HYP-0002-memory-convergence.md).
- Added: [docs/plans/data-os-multimodal-merge-plan-2026-09-08.md](../../plans/data-os-multimodal-merge-plan-2026-09-08.md).
- Added: [experiments/convergence-memory-legacy/.gitignore](../../../experiments/convergence-memory-legacy/.gitignore).
- Added: [experiments/convergence-memory-legacy/Cargo.lock](../../../experiments/convergence-memory-legacy/Cargo.lock).
- Added: [experiments/convergence-memory-legacy/Cargo.toml](../../../experiments/convergence-memory-legacy/Cargo.toml).
- Added: [experiments/convergence-memory-legacy/README.md](../../../experiments/convergence-memory-legacy/README.md).
- Added: [experiments/convergence-memory-legacy/crates/cm-legacy-runner/Cargo.toml](../../../experiments/convergence-memory-legacy/crates/cm-legacy-runner/Cargo.toml).
- Added: [experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/lib.rs](../../../experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/lib.rs).
- Added: [experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/main.rs](../../../experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/main.rs).
- Added: [experiments/convergence-memory-legacy/rust-toolchain.toml](../../../experiments/convergence-memory-legacy/rust-toolchain.toml).
- Added: [experiments/convergence-memory/.gitignore](../../../experiments/convergence-memory/.gitignore).
- Added: [experiments/convergence-memory/Cargo.lock](../../../experiments/convergence-memory/Cargo.lock).
- Added: [experiments/convergence-memory/Cargo.toml](../../../experiments/convergence-memory/Cargo.toml).
- Added: [experiments/convergence-memory/README.md](../../../experiments/convergence-memory/README.md).
- Added: [experiments/convergence-memory/crates/cm-candidate/Cargo.toml](../../../experiments/convergence-memory/crates/cm-candidate/Cargo.toml).
- Added: [experiments/convergence-memory/crates/cm-candidate/src/lib.rs](../../../experiments/convergence-memory/crates/cm-candidate/src/lib.rs).
- Added: [experiments/convergence-memory/crates/cm-candidate/tests/replay.rs](../../../experiments/convergence-memory/crates/cm-candidate/tests/replay.rs).
- Added: [experiments/convergence-memory/crates/cm-harness/Cargo.toml](../../../experiments/convergence-memory/crates/cm-harness/Cargo.toml).
- Added: [experiments/convergence-memory/crates/cm-harness/src/lib.rs](../../../experiments/convergence-memory/crates/cm-harness/src/lib.rs).
- Added: [experiments/convergence-memory/crates/cm-harness/src/main.rs](../../../experiments/convergence-memory/crates/cm-harness/src/main.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/Cargo.toml](../../../experiments/convergence-memory/crates/cm-trace/Cargo.toml).
- Added: [experiments/convergence-memory/crates/cm-trace/src/format.rs](../../../experiments/convergence-memory/crates/cm-trace/src/format.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/src/generate.rs](../../../experiments/convergence-memory/crates/cm-trace/src/generate.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/src/lib.rs](../../../experiments/convergence-memory/crates/cm-trace/src/lib.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/src/results.rs](../../../experiments/convergence-memory/crates/cm-trace/src/results.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/src/run.rs](../../../experiments/convergence-memory/crates/cm-trace/src/run.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/src/sha.rs](../../../experiments/convergence-memory/crates/cm-trace/src/sha.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/conformance.rs](../../../experiments/convergence-memory/crates/cm-trace/tests/conformance.rs).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/fixtures/.gitattributes](../../../experiments/convergence-memory/crates/cm-trace/tests/fixtures/.gitattributes).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/fixtures/SHA256SUMS](../../../experiments/convergence-memory/crates/cm-trace/tests/fixtures/SHA256SUMS).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/fixtures/reference.py](../../../experiments/convergence-memory/crates/cm-trace/tests/fixtures/reference.py).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/fixtures/small.cmt](../../../experiments/convergence-memory/crates/cm-trace/tests/fixtures/small.cmt).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/fixtures/small.results.cmt](../../../experiments/convergence-memory/crates/cm-trace/tests/fixtures/small.results.cmt).
- Added: [experiments/convergence-memory/crates/cm-trace/tests/retention.rs](../../../experiments/convergence-memory/crates/cm-trace/tests/retention.rs).
- Added: [experiments/convergence-memory/rust-toolchain.toml](../../../experiments/convergence-memory/rust-toolchain.toml).

Prior proof log SHA-256: `dd8cbff615cbb4840ae47b9440fcb28f14fd82eb9339406f00aa74587f9803a2`.

## Files updated for L1-L4

The inventory above covers the whole uncommitted work order. This correction
round updates the following existing implementation files plus its proof log:

- [Shared series runner and retention policy](../../../experiments/convergence-memory/crates/cm-trace/src/run.rs).
- [Retention regression tests](../../../experiments/convergence-memory/crates/cm-trace/tests/retention.rs).
- [Candidate CLI](../../../experiments/convergence-memory/crates/cm-harness/src/main.rs).
- [Legacy CLI](../../../experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/main.rs).
- [Shared workspace README](../../../experiments/convergence-memory/README.md).
- [Legacy workspace README](../../../experiments/convergence-memory-legacy/README.md).
- [CI workflow comments](../../../.github/workflows/convergence-memory.yml).
- [EXP-0002 experiment](../EXP-0002-convergence-memory.md).
- [Results policy and host-entry template](results/README.md).
- [Project status](../../PROJECT-STATUS.md).
- [This implementation report](IMPLEMENTATION-REPORT.md).
- [L1-L4 proof output](results/proof-windows-l1-l4.txt).

## Files updated for M1-M6

The full uncommitted inventory above remains applicable. This round updates:

- Shared Rust: `cm-trace/src/results.rs`, `cm-trace/src/run.rs`,
  `cm-trace/tests/conformance.rs`, `cm-trace/tests/retention.rs`.
- Fixtures: `reference.py`, `small.results.cmt`, `SHA256SUMS`; `small.cmt` is unchanged.
- Adapters: candidate `cm-harness/src/lib.rs` and `main.rs`; legacy
  `cm-legacy-runner/src/lib.rs` and `main.rs`.
- Workflow and guides: `.github/workflows/convergence-memory.yml`, both workspace
  READMEs, `AGENTS.md`, `tools/validate_markdown_links.py`, `docs/GLOSSARY.md`,
  `docs/PROJECT-STATUS.md`, and renamed `docs/hypotheses/HYP-0002-memory-convergence.md`.
- Experiment records: `EXP-0002-convergence-memory.md`, this report, the results
  index, and new [M1-M6 proof log](results/proof-windows-m1-m6.txt).

## Files updated for N1-N8

- Candidate code: `cm-candidate/src/lib.rs`, `cm-harness/src/lib.rs`,
  `cm-harness/src/main.rs` (under `experiments/convergence-memory/crates/`).
- Shared code/tests: `cm-trace/src/run.rs`, `cm-trace/src/generate.rs`,
  `cm-trace/tests/conformance.rs`, `cm-trace/tests/retention.rs`.
- Fixtures: `small.cmt`, `small.results.cmt`, `reference.py`, `SHA256SUMS`.
- Legacy tests: `experiments/convergence-memory-legacy/crates/cm-legacy-runner/src/lib.rs`.
- Documentation: shared workspace README, EXP-0002 definition, PROJECT-STATUS,
  [TRACEABILITY](../../TRACEABILITY.md), [RESEARCH-QUESTIONS](../../RESEARCH-QUESTIONS.md),
  results index, this report, and the new N1-N8 proof log.

The prior inventory covers the rest of the uncommitted work order. Prior proof
logs and protected EXP-0001 trees remain unchanged; edited files use UTF-8/LF.
