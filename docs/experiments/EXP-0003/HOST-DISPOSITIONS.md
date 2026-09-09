# EXP-0003 host dispositions (record of decisions cited by the ledger)

These are the host's dispositions on the independent inspections of the Step 3 work order
(`unify commitment and recovery`, owner merge plan step 3), copied verbatim from the coordinating
session's feedback files so that ADR-0003 and `AGENTS.md` §10 cite a document that lives in
this repository. Authority: the owner-authored
[merge plan, step 3](../../plans/data-os-multimodal-merge-plan-2026-09-08.md) and the work order
approved by an independent Codex review on 2026-09-09 (plan SHA-256
`4fa291fd472764c6746e841607a151e71e3943058e249f6a4378a92bb507c043`). The host is the Claude Code
coordinating session; the builder was Codex; inspectors were fresh Claude CLI sessions.

Two dispositions below change what the approved work order said:

- **P1** narrows the "payload-agnostic core" by rejecting payload bytes containing the RF1 magic
  (`RDE1`), to keep the frozen scanner's interior-damage heuristic from making a torn payload
  frame permanently unopenable. The residual (the magic inside UUID or envelope bytes) is tested
  and documented; a codec-level follow-on is recorded.
- **Q3** splits `Log::create` from `Log::open` and adds `Engine::creates_store_directory` to the
  shared runner (default `false`, existing callers unchanged), beyond the work order's single
  permitted runner change, so that opening a mistyped store path fails instead of creating an
  empty history.

The owner accepted closing Step 3 with the remaining inspection findings carried as residuals
(recorded in the coordinating session's build log; see the implementation report).

---

## Dispositions after inspection 1 (snapshot 5112e99e…965b), 2026-09-09

## P1 (medium, from UC-01) — RF1 magic inside a torn payload becomes permanent corruption

The frozen scanner classifies a last frame crossing EOF as `Failure { InteriorDamage }` instead
of `TerminalTruncation` when any later window of the remaining bytes equals `RDE1`. The payload
is embedded three times, so an arbitrary payload containing those four bytes, torn mid-frame,
makes `Log::open` fail closed with no repair path, contradicting §9 and the README.

Disposition (option b): reject at `Log::commit`, before any write, any transaction whose payload
bytes contain the sequence `RDE1` (`LogError::Invalid("payload contains the RF1 magic")`), and
reject such bytes in `envelope::request` decoding as corrupt so an old history cannot contain
them. Record the constraint in ADR-0003, the README and EXP-0003 §9/§14 (state that it is
unreachable for CMM2 hex payloads and is a codec-level follow-on for arbitrary payloads, §18).
Add a core test that a payload containing the magic is rejected with nothing appended, and a
test that a torn tail whose truncated bytes contain the magic in a *legitimately encoded*
frame cannot occur (or, if it still can through framing bytes, state precisely why not).

## P2 (low, from UC-02) — D1 checkpoint ahead of unsynced history

Call the history writer's synchronize (file sync) before writing a checkpoint at any level, so a
checkpoint never references history bytes that are less durable than itself. Test: a D1 log with
one commit and a checkpoint records the sync placement in the placements log. Keep the
README/§9 statement that `HistoryShorterThanCheckpoint` fails closed and name the manual
remedy (remove the newer checkpoint file) for the remaining OS-crash case outside the model.

## P3 (low, from UC-03) — undeclared runner control-flow change

Add one sentence to the implementation report deviation 1 and EXP-0003 §14: series now fail
closed when `git` is unavailable or the output directory lies inside a source prefix (required by
the evidence rule), and this applies to EXP-0002 callers too.

## P4 (low, from UC-04) — one engine label per series

Build the engine label once (with the revision) and pass it to `series_with_meta` so
`environment.txt`, `throughput.txt`, `summary.txt`, `results.cmt` and `observations.cmt` carry
the identical `engine=` string; add a `uc-harness` (or `uc-memory`) test mirroring cm-harness's
identity assertion across the evidence files.

## P5 (low, from UC-05) — redundant lifecycle pass and I/O mislabel

Remove the redundant `validate_lifecycle` call in `read_history` (or keep it under
`debug_assertions` with a comment), and map `ReplayTermination::IoFailure` to a distinct
`LogError::Io { offset, kind }` so an I/O condition is not reported as byte corruption. Tests
updated accordingly.

## P6 (low, from UC-06) — auto request ids collide with explicit ids

Derive adapter-generated request ids in a distinct identity domain (a domain byte not used by
explicit ids, e.g. `0x41`) and document that auto ids are conveniences that are not stable
request identities across a torn-tail reopen (README and ADR).

## P7 (low, from UC-07) — environment-dependent reconstruction test

Keep the fixture-based second pass as the CI assertion; make the first pass (real checkout)
skip with a printed reason when the checkout has uncommitted changes under the prefixes; set
`core.autocrlf=false` in the temporary clone; note the shallow-checkout assumption in the
workflow comment.

## P8 (low, from UC-08 and host observation) — undocumented resident costs and limits

Document in the README and EXP-0003 §10/§11: the accepted history prefix is retained in memory
for the log's lifetime (bounded by `MAX_HISTORY`); every binding's normalized request is
retained; the adapter validator clones the slot map once and the core clones it again per
transaction (O(records) per commit, inside `complete_operations`); the frozen `MAX_RECORDS =
1_000_000` scan cap bounds one history to about 200,000 transactions (five frames each). List
"check-only validator, incremental prefix hash, persistent state structure" as §18 follow-ons.
No code change required for this item beyond the documentation.

## P9 (low, from UC-09) — document placement

Move the two glossary bullets into the alphabetical list after the preamble; place the EXP-0003
section in PROJECT-STATUS below the header block; append RQ-005 after RQ-004.

---

## Dispositions after inspection 2 (snapshot 35b0f0a8…c3bf), 2026-09-09

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

---

## Residuals carried at close (inspection 3, snapshot 1a345528…1a89), owner decision 2026-09-09

- UC-R2 (low): a history synchronize failure inside `Log::checkpoint` returns an error but does
  not poison the log.
- UC-R3 (low): a surviving pre-commit `Binding` makes its request id permanently
  `Rejected { Uncommitted }`; R5 §5's resume rule (reuse the exact candidate when equality is
  provable) is not implemented. Callers must issue a new request id.
- UC-R4 (low): a checkpoint whose position matches an accepted event but whose prefix hash
  differs is rejected with fallback to full replay rather than failing closed.
- UC-R6 (low): `store.amplification.txt` is written beside the store directory via
  `with_extension`, outside the locked directory.
- UC-R7 (low): the recorded `regeneration_command` names the `cm-harness` binary for EXP-0003
  series; the generator is shared and the bytes are identical.
- UC-R5 was resolved by the host: the `UCE1` header is the five bytes `55 43 45 31 00`
  (`UCE1` followed by NUL), matching the code; the work order's rendering of `\0` was the
  ambiguity.
