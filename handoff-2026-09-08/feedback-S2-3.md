# Host dispositions after Step 2 inspection 2 (fresh Claude CLI) of snapshot 850eb388…9e6f

Host proof on that snapshot: candidate fmt/clippy/test PASS (12 tests); legacy networked fetch,
fmt/clippy/test PASS (2 tests); exp-0001 PASS (95, harness excluded); links PASS; diff check
PASS. Host definitive series on the final release binaries, all six trials valid in every cell:

| Cell | Wall time | Measured trial 1 ops/s | Notes |
|---|---|---|---|
| candidate 1K | 38 s | 18,741 | replay p50 785 ms, rows 12.7 ms, columns 17.8 ms |
| legacy 1K single | 180 s | 514 | insert/replace/delete p50 ≈ 2.7 ms (per-op `sync_data`) |
| candidate 10K | 422 s | 17,871 | replay p50 8.05 s, rows 149 ms, columns 271 ms |
| legacy 10K single | 10,883 s (3 h 1 m) | 73 | insert/replace/delete p50 ≈ 14–21 ms, p99 ≈ 45 ms |

Peak RSS was NOT captured on this host: every `trials.csv` row reads
`"unavailable: powershell: program not found"` because `peak_rss()` and `metadata()` invoke
`powershell`, and this machine has only `pwsh` (PowerShell 7) on PATH.

The inspector's verdict was REVISE with eight findings. The owner extended the budget by exactly
one fix round and one inspection. Implement every item below completely, rerun the proof chain,
refresh the implementation report, keep LF endings, do not commit. Do not widen scope beyond it.

## N1 (medium, from F1) — candidate replay stage attribution and integrity profile

- Split the candidate `finish` timing so `replay` covers `reopen_and_replay` only (physical
  frame scan and structural validation), and add a separate `decode` stage for CMM1 payload
  parsing into `Change` values; `rows` and `columns` stay as they are. Update the stage list in
  EXP-0002 §10 and the README, and the `summary.txt` row set. Keep `append` as is.
- State explicitly, in EXP-0002 §10 and §12 and the workspace README, that candidate frames use
  `IntegrityProfile::Structural` (no CRC-32C stored or verified) and that the measured replay
  therefore excludes the CRC-32C cost the B1 design selected; record "CRC-32C profile cell" as a
  §18 follow-on. Do not change the profile in this round (it would invalidate the host series).

## N2 (medium, from F2) — memory footprint and RSS metric, bounded

- Compute the input SHA-256 once from the file bytes in `load()` (or `series`) and pass it to
  `trial`/`metadata` instead of re-encoding the trace per trial. Do not restructure the decoder.
- Record an RSS baseline immediately after the trace is loaded and before the first trial, write
  it to series `environment.txt` as `rss_baseline_after_load=…`, and keep the per-trial
  process-lifetime high water as is; document in §10 that the metric is harness-inclusive and
  that the baseline line lets a reader subtract the trace/oracle share.
- Document the projected 100K footprint honestly in the README's 100K paragraph and §11 (whole
  trace and whole history buffered; the inspector's estimate is 10–15 GB per trial) and list a
  streaming loader/decoder as a §18 follow-on.

## N3 (host-found, medium) — peak RSS probe fails on this machine

`peak_rss()` and `metadata()` call `powershell`; this host has only `pwsh`. Try `pwsh` first and
fall back to `powershell`; if neither runs, keep the current "unavailable: …" text. Better: on
Windows read the working-set high water without spawning a shell, using
`std::process::Command` only as the fallback. Add a unit test that the probe returns either a
positive integer string or a string starting with `unavailable:` (never panics). The host will
re-run the 1K series after this round to confirm RSS is captured.

## N4 (low, from F3) — CI status wording

Keep the workflow. In `docs/PROJECT-STATUS.md`, EXP-0002 §16 and the implementation report,
describe CI as "workflow added; not yet executed remotely; the legacy leg assumes anonymous
fetch of two GitHub git sources", and add a one-line "before merge" item in the report: push the
branch and record the Actions run URL for both matrix legs.

## N5 (low, from F4) — field updates and guards

The pinned legacy adapter exposes only `access_count` through `update_field`, so per-field
updates of other fields cannot be exercised through that path; say so in EXP-0002 §8 (one
sentence) and keep `update` as is. Add at least one `guard` on a non-integer field (for example
`status`, field 8) to the small trace and to the seeded generator so the type-discriminant path
is exercised on both engines; regenerate the golden fixtures with `reference.py`, update
`SHA256SUMS`, and update the hand-checked indices in `conformance.rs`.

## N6 (low, from F5) — exact commands in the experiment document

Copy the exact `generate`/`run` commands for candidate and for legacy single/pipelined/atomic
into EXP-0002 §15 (Windows spelling plus the Linux `+1.89.0` note), keeping the README as the
expanded guide.

## N7 (low, from F7) — traceability and research questions

Add rows for HYP-0002 and EXP-0002 to `docs/TRACEABILITY.md` (authority: merge plan step 2;
status: Ready, correctness-validated, host series retained, no performance conclusion) and map
HYP-0002 to the relevant research question in `docs/RESEARCH-QUESTIONS.md`.

## N8 (low, from F8) — candidate label

`CandidateEngine::label()` returns the durability constant. Return a descriptive label such as
`candidate RF1 full after-image; IntegrityProfile::Structural; rev=<git HEAD>` (compute the
revision once in `metadata` and pass it, or read it in `create`), keep `durability()` as `D1`,
and assert the label prefix in the candidate test as the legacy test does.

## Results index (host entries)

Leave the host entries empty; the host will add the four definitive series (and a re-run 1K pair
after N3) using the template, and will commit the defined evidence subset itself.

## Report

Refresh the proof table for the new snapshot and add an N1–N8 section. Note that the host series
above were produced on snapshot `850eb388…9e6f` binaries and that N1's stage split and N3's RSS
fix post-date them; the host will state which series come from which snapshot in its entries.
