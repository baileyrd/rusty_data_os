# EXP-0003 results index

Three host series are catalogued below (1K D1, 1K D2, 10K D2 on the committed revision's
snapshot) plus one incomplete negative record. Proof tests are correctness validation, not
performance evidence; the series are descriptive measurements under the declared injected
failure model, with no platform durability or winner conclusion.

Retain environment.txt, source.sha256, source.patch, trials.csv, summary.txt and each trial's
throughput.txt and store.amplification.txt. Raw per-operation samples remain external, referenced by absolute path and
SHA-256 (the accepted EXP-0002/R7-style deviation). Do not copy stores or complete series here.
A series is evidence only when its porcelain is clean at revision or applying source.patch
to revision reconstructs every source.sha256 entry; the host verifies this before cataloguing.

`source.sha256` hashes exact working-tree file bytes, including line endings. Both tracked
and untracked patch diffs explicitly use `-c core.autocrlf=false -c core.safecrlf=false` and
`-c diff.noprefix=false -c diff.mnemonicPrefix=false` before `diff`, with `--binary --no-ext-diff --no-textconv`. Host Git conversion or prefix settings must not alter retained
patch bytes. The always-run fixture checks CRLF-modified tracked text and untracked binary
reconstruction with both source and restore configured `core.autocrlf=true`.

| Series | Revision/patch reconstruction | Input SHA-256 | Mode | Validity | Raw location |
|---|---|---|---|---|---|
| [2026-09-09-1a345528-1k-d1](2026-09-09-1a345528-1k-d1/) (1K, 2026-09-09 14:19:53 UTC) | `e62fa699` + `source.patch` (snapshot `1a345528`, the revision committed with this change); host applied the patch to a fresh worktree of `e62fa699` and all 52 `source.sha256` entries matched | `53201589e30fd48d4aeab077a6435bddc86902c25bd35bf70d1f39044cd070d0` | unified D1 | all six trials valid; measured trial 1: 4,417 ops/s; summary SHA-256 `ee46f69c23605c343b5587ca873121e87f8834842bd7116e861bb5322d95451e` | `C:\dev\rusty_data_os-evidence\EXP-0003\exp0003-final2\1k-d1\` (copied verbatim from the operator scratch path recorded in each `throughput.txt`) |
| [2026-09-09-1a345528-1k-d2](2026-09-09-1a345528-1k-d2/) (1K, 2026-09-09 14:21:01 UTC) | same snapshot and patch as above (verified) | `53201589e30fd48d4aeab077a6435bddc86902c25bd35bf70d1f39044cd070d0` | unified D2 | all six trials valid; measured trial 1: 351 ops/s; 304 s for six trials; summary SHA-256 `fb0b8572981e9e8136e607431110daf64178374992dfd1994f305430993b1072` | `C:\dev\rusty_data_os-evidence\EXP-0003\exp0003-final2\1k-d2\` |
| [2026-09-09-1a345528-10k-d2](2026-09-09-1a345528-10k-d2/) (10K, 2026-09-09 14:26:29 UTC) | same snapshot and patch as above (verified) | `0cc37e2c040166a6da0735812c843e969a249f0579a92ddb617a7fb4232a47d3` | unified D2 | all six trials valid; measured trial 1: 319 ops/s; 3,696 s (61.6 min) for six trials; 873,670,322 physical bytes for 270,856,774 payload bytes (three copies); summary SHA-256 `62571bdbb1ec5d163d5be4321f8fd53b378dd8e1add064c9f33ce38254c3cc58` | `C:\dev\rusty_data_os-evidence\EXP-0003\exp0003-final2\10k-d2\` |
| 2026-09-09-35b0f0a8-10k-d2 (10K, snapshot `35b0f0a8`, binaries before fix round 2) | not catalogued as evidence | `0cc37e2c…47d3`-equivalent regeneration (same generator and seed; input hash recorded in its `environment.txt`) | unified D2 | **incomplete**: warm-up and measured trials 1–4 valid, trial 5 lost when the coordinating session ended; no summary; kept as a negative record | `C:\dev\rusty_data_os-evidence\EXP-0003\exp0003-final-incomplete-35b0f0a8\10k-d2\` |

Host notes on the catalogued series (descriptive only, no winner claim): the 1K D1 cell ran
twice on earlier snapshots at 6,098 and 6,007 ops/s and here at 4,417 ops/s under concurrent
load on the build box; D2 throughput is bounded by four synchronizations per transaction
(1K: 351 ops/s, 10K: 319 ops/s, insert p50 about 3.5 ms at 10K). Stage times at 10K D2, measured
trial medians: replay 34.8 s, decode 7.5 s, rows 0.19 s, columns 0.30 s, checkpoint 5.0 s,
open_from_checkpoint 41.7 s (the full scan remains authoritative on reopen). Peak RSS is process
high water including harness and oracle allocations. The injected fault matrix passed on every
proof run (`uc-harness` `faults` tests, 14 placements at both levels, plus lock release).

Record failures and unavailable metadata; no winner or platform survival conclusion.
