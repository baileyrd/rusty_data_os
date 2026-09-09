# Host dispositions after Step 2 build round 1 (snapshot 19e25cf4…fcae4, base 54bd1a4)

Host proof on your snapshot (independent, worktree, GNU 1.89.0): candidate fmt/clippy/test PASS
(8 tests); legacy networked `cargo fetch --locked` PASS, fmt/clippy/test PASS (2 tests); exp-0001
fmt/clippy/test PASS with the harness crate excluded (95 tests); link validator PASS;
`git diff --check` PASS. The host ran the 1K series for both engines with the release binaries
(output directed to a scratch directory outside the checkout): candidate and legacy single both
`valid=true` on all six trials. Observed retention per 1K series: candidate 198 MB, legacy
104 MB, plus a 26 MB `trace.cmt` copy per series; per trial the `store/` directory is 27 MB
(candidate) or 11 MB (legacy), `results.cmt` 1.3 MB, `observations.cmt` 0.8 MB; the `source/`
copy is 193 KB per series. A 10K series would therefore retain roughly 2 GB (candidate) and 1 GB
(legacy) under `docs/experiments/EXP-0002/results/`, which cannot be committed.

Implement every item below in this checkout (no commit), rerun the full proof chain, refresh the
implementation report, keep LF endings.

## L1 (medium) — retention policy: stop retaining store files and trace copies by default

`run::series` retains, per trial, the complete engine store and, per series, a full copy of the
input trace and of every workspace source file. Change the default to minimal retention and make
the committed-evidence layout explicit:

- Default retention per series: `environment.txt`, `source.sha256`, `source.patch`, `trials.csv`,
  `summary.txt`, and per trial `environment.txt`, `throughput.txt`, `observations.cmt` and
  `results.cmt`. Do not copy the trace (its SHA-256 is already in every results file; record the
  exact regeneration command `generate <size> …` and the generator seed in `environment.txt`), do
  not copy the source tree (`source.sha256` + `source.patch` identify it), and delete each trial's
  `store/` after its digests are computed and compared. Add an explicit opt-in flag (for example a
  trailing `--retain-store` argument) that keeps the store directories and the trace copy for
  forensic runs; document it.
- Committed evidence: define in `docs/experiments/EXP-0002/results/README.md` exactly which files
  from a series are copied into the repository (series-level `environment.txt`, `source.sha256`,
  `source.patch`, `trials.csv`, `summary.txt`, and per-trial `throughput.txt`), and that
  `results.cmt`/`observations.cmt` raw per-operation samples stay with the operator's output
  directory and are referenced by path and SHA-256. Add a template subsection for a host series
  entry (engine, mode, size, date, output location, validity, summary sha256).
- Update `experiments/convergence-memory/README.md` (retention paragraph and the "Each invocation
  creates…" list) and EXP-0002 §10/§15 accordingly. Keep `retention.rs` passing; extend it or add
  a test that a series directory contains no `store/` or `trace.cmt` by default and does with the
  opt-in flag.

## L2 (low) — CI assumption and host-results recording

- The workflow's legacy `cargo fetch --locked` needs GitHub Actions to reach
  `github.com/baileyrd/rusty_multimodal_db`. The host could not verify that repository's
  visibility from this network. State the assumption in the workflow (a comment) and in the
  legacy README: if the repository is private, the job needs a token (e.g. a
  `CARGO_NET_GIT_FETCH_WITH_CLI=true` step with a `GITHUB_TOKEN`-authenticated git credential),
  and say the workflow has not been executed remotely.
- EXP-0002 §16 currently says "Host is expected to smoke-run 1K and 10K". Reword to state that
  host series are recorded in the results index when collected, and leave the index entry to be
  filled by the host (the host will add its 1K and 10K entries after this round using the
  template from L1).

## L3 (low) — size and memory notes for large traces

CMT1 hex-encodes every text field, so the 1K trace is 26 MB and 10K is about 260 MB; the 100K
generator and loader hold the whole encoded trace and decoded operations in memory (multiple
GB). Say this explicitly in the workspace README's 100K paragraph with the observed 1K size, and
note that a streaming loader is a follow-on, not part of this increment. No format change.

## L4 (low) — report and status text

In `docs/experiments/EXP-0002/IMPLEMENTATION-REPORT.md` and `docs/PROJECT-STATUS.md`, replace
"pending independent review"/"advisory" phrasing in status-bearing sentences with neutral
implementation status (the report title may keep "implementation report"). Record the host's 1K
observation in the report's deviations section only as "host 1K series completed, both engines
valid; retention changed per L1" — no performance interpretation.
