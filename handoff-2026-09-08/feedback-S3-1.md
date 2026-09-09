# Host dispositions after Step 3 inspection 1 (fresh Claude CLI) of snapshot 5112e99e…965b

Host proof on that snapshot (independent, worktree, GNU 1.89.0): unified-commitment
fmt/clippy/test PASS (24 tests incl. the 14-placement abort matrix and lock release);
convergence-memory fmt/clippy/test PASS (17); exp-0001 PASS (95, harness excluded); links PASS;
`git diff --check` PASS; the injected-fault matrix re-run on the record passed (2 tests). Host 1K D1
series on release binaries: valid, 6,098 ops/s (insert p50 248 µs, update 51 µs, delete 47 µs;
replay 846 ms, decode 788 ms, checkpoint 547 ms, open_from_checkpoint 2.16 s; 87.6 MB physical
history for 27.2 MB payload, three copies). D2 series were interrupted and will be re-run on the
fixed binaries.

The inspector's verdict was REVISE with nine findings; the host verified UC-01 against the
frozen scanner and accepts all nine. This is fix round 1 of 2: implement every item completely,
rerun the proof chain, refresh the implementation report, keep LF endings, do not commit.

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

## Report

Refresh the proof table for the new snapshot; add a P1–P9 section. Host series entries stay
empty; the host re-runs 1K D1/D2 and 10K D2 on the fixed binaries.
