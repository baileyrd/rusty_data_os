# Host dispositions after Step 3 inspection 2 (fresh Claude CLI) of snapshot 35b0f0a8…c3bf

Host proof on that snapshot: unified-commitment fmt/clippy/test PASS (29 tests);
convergence-memory PASS (17); exp-0001 PASS (95, harness excluded); links PASS; diff check
PASS. The inspector found no path that publishes an uncommitted transaction or silently drops a
committed one within the declared failure model. Verdict REVISE with four findings; the host
accepts all four. This is fix round 2 of 2 (the owner extended the inspection budget by one):
implement every item completely, rerun the proof chain, refresh the implementation report, keep
LF endings, do not commit, do not widen scope.

## Q1 (medium, from UC3-01) — patch generation must not depend on host Git configuration

Generate both diffs in `snapshot_at` with explicit configuration so the retained patch reproduces
working-tree bytes regardless of the host's settings:
`git -c core.autocrlf=false -c core.safecrlf=false -c diff.noprefix=false -c diff.mnemonicPrefix=false diff HEAD --binary --no-ext-diff --no-textconv`
and the same options for the `--no-index` hunks. In the reconstruction test's always-run fixture
pass, add a case that writes a CRLF-modified tracked text file in a worktree configured with
`core.autocrlf=true`, snapshots it, and verifies reconstruction in a restore worktree that also
has `core.autocrlf=true`. Record the requirement (explicit config, and that `source.sha256`
hashes working-tree bytes) in EXP-0003 §15 and the results index.

## Q2 (low, from UC3-02) — D2 injection points relative to the sync

For Binding, Reservation and Provisional at `D2`, fire the injection hook **after** that
record's sync (R5 order: append, then sync), keeping the `D1` placement as the append. If you
prefer to keep a pre-sync point as well, add distinct `…-appended` and `…-synced` names and
exercise both in `faults.rs`. State the exact placement relative to the sync in EXP-0003 §9 and
the workspace README.

## Q3 (low, from UC3-03) — opening must not create silently

Separate creation from opening: `Log::create(path)` (fails if the directory or history exists)
and `Log::open(path)` (fails with `LogError::NotFound` when the directory or `history.rf1` is
absent). `Directory::acquire` must not `create_dir_all` on the open path. Update the series
runner, the fault subcommand, the Memory adapter and all tests; add a test that opening a
nonexistent path fails and creates nothing.

## Q4 (low, from UC3-04) — provenance of the payload-magic constraint

Record the host authorization for the payload-magic rejection (disposition P1 of
`feedback-S3-1.md`, 2026-09-09, accepted by the host under the owner's merge-plan step 3
authority and the approved work order) in ADR-0003's authority section and in the AGENTS.md §10
sentence for EXP-0003, so the deviation from "payload-agnostic" is traceable to a recorded
decision. Keep the residual (magic inside UUID/envelope bytes) and the codec follow-on wording.

## Report

Refresh the proof table for the new snapshot and add a Q1–Q4 section. Host series entries stay
empty; the host records its series with their source snapshot and patch identity.
