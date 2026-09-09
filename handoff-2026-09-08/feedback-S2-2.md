# Host dispositions after Step 2 inspection 1 (fresh Claude CLI) of snapshot 41dbfd5b…52b7

Host proof on that snapshot: candidate fmt/clippy/test PASS (11 tests); legacy networked fetch,
fmt/clippy/test PASS (2 tests); exp-0001 PASS (95, harness excluded); link validator PASS;
`git diff --check` PASS. Host final series on the fixed binaries: candidate 1K and legacy 1K
single valid with the minimal retention layout; candidate 10K valid earlier (18.6k ops/s,
replay 7.9 s); legacy 10K single trials run much longer than the 1K extrapolation (a warm-up
trial exceeded 15 minutes) and the series is still running.

The inspector's verdict was REVISE with six findings; the host verified each against the source
and accepts all six. This is the last fix round of the budget: implement every item completely,
rerun the proof chain, refresh the implementation report, keep LF endings, do not commit.

## M1 (medium) — per-engine durability line in CMT1-results

`Results::encode` writes `durability\t<hex(D1)>` unconditionally and `decode` rejects anything
else, so every legacy `observations.cmt` declares "D1 ordinary writes, no fsync" for an engine
whose insert/replace/delete paths call `sync_data`. R5 requires each results file to name its own
durability setting.

Disposition: add `durability: String` to `Results`; add `fn durability(&self) -> String` to
`Engine` (candidate returns `D1`, legacy returns its `DURABILITY` constant); write the line from
the engine and let `decode` accept any hex-encoded UTF-8 string (non-empty). Keep the
`candidate_durability=D1…` reference line in `results.cmt`, `environment.txt`, `throughput.txt`
and `summary.txt`, and add an `engine_durability=…` line next to it in each. Update
`reference.py`, regenerate `small.results.cmt` and `SHA256SUMS`, and assert in the candidate
test that the decoded durability equals `D1` and in the legacy test that it equals `DURABILITY`.
Bump nothing in the trace format; note in the README's CMT1 section that the results section's
durability line is engine-declared. No CMT1-results version bump is needed if you treat the
previous fixture as superseded in this same uncommitted change; say so in the report.

## M2 (low) — second git source in the legacy lockfile

`experiments/convergence-memory-legacy/Cargo.lock` also pins
`git+https://github.com/Rusty-Mill/rusty_mill?rev=cf284712…` (platform, platform-windows,
rusty_tls, winargv). Extend the workflow comment and the legacy README so the reachability and
token assumption names both git sources.

## M3 (low) — series label mismatch

`cm-legacy-runner/src/main.rs` builds the series label with `dispatch={mode:?}` while
`Legacy::label` uses `mode={:?}`. Expose `pub fn label_for(mode: Mode) -> String` in the library
and use it in both places so every file of a series carries the identical engine string.

## M4 (low) — results footprint at 10K/100K

Each `equal` query signature lists every matching record's digest twice (sorted digests and the
ordered id list) in `results.cmt` and again hex-encoded in `observations.cmt`; the host's 10K
candidate trial produced multi-hundred-megabyte raw files, and 100K would be tens of gigabytes
per series with the whole `operations` vector buffered in memory. Disposition, bounded:
- Document the per-size raw footprint next to the trace-memory note in the workspace README and
  in EXP-0002 §10/§15, with the observed 1K and 10K figures the host recorded (observed by the host,
  candidate 10K measured trial: `results.cmt` 20 MB, `observations.cmt` 15 MB, store 265 MB;
  candidate 1K trial: 1.3 MB, 0.8 MB, 27 MB).
- Write `observations.cmt` incrementally (stream each op line as it is produced) instead of
  buffering `Results.operations`; keep the canonical encoding identical, and keep the
  round-trip test passing.
- Do not change the query-signature representation in this increment; record the digest-collapse
  idea as a §18 follow-on question.

## M5 (low) — link-validator behaviour

Keep the tracked-plus-untracked scan (staging the move is not available to the builder), but
document it: extend the script's module docstring to say untracked, non-ignored Markdown is
validated and that a CI checkout is clean so this only affects local runs, and add one sentence
to `AGENTS.md`'s validation-sequence text (or the README section that names the tool) so the
behaviour change is recorded outside the implementation report.

## M6 (low) — repository conventions

- `docs/GLOSSARY.md`: convert the `Memory` and `B0` `##` sections into `- **term** — definition`
  bullets in the existing list.
- Rename `docs/hypotheses/H-CM-001-memory-convergence.md` to
  `docs/hypotheses/HYP-0002-memory-convergence.md` (identifier `HYP-0002`) and update every
  link and mention (EXP-0002 §3, PROJECT-STATUS, environment metadata `hypothesis=` line, README).
- Add one line to the workspace README's CMT1 section showing how to decode a hex field for hand
  reading (for example a one-line `python -c` using `bytes.fromhex`).

## Report

Refresh the proof table for the new snapshot; add an M1–M6 section in the style of the L
section. The host will re-run the 1K and 10K series on the fixed binaries and fill the host
entries in the results index itself; leave those entries empty.
