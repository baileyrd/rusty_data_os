# Host dispositions on Codex review 1 of the Step 3 work order (plan sha256 43a66609…f260)

All eight findings accepted; the work order was revised as follows. Please re-review the
revised plan (new hash) for approval.

- UCR-001 (high, lock file survives abort): ownership is now an OS-released advisory lock
  (`std::fs::File::try_lock()` on `<dir>/owner.lock`, stable since Rust 1.89) held for the
  `Log`'s lifetime; presence of the file means nothing; a second opener is refused with
  `WouldBlock`; a dead owner's directory reopens without any stale-file protocol. R7 adds a
  lock-ownership test (child holds → parent refused; child aborts → parent opens).
- UCR-002 (high, timestamp placement): R2 step 4 now spells out R5 §5 exactly: pre-finalization
  sync, one realtime sample, construct `Final` (carrying the sample) + adjacent `Commit`,
  post-finalization sync, publish, return the persisted sample; post-commit observation time is
  separate and never written into records.
- UCR-003 (high, undecidable truncation): R3 now classifies only what the bytes decide:
  `CleanEof`/`TerminalTruncation` = torn tail dropped and reported (same outcome for an
  interrupted append or a later truncation; stated explicitly); `Failure`/`IoFailure` with
  bytes remaining = fail closed with offset; the only detectable loss of a committed suffix is
  a checkpoint recording a position beyond the accepted prefix, which fails closed. R7's tests
  now separate torn tails, interior damage with the suffix preserved, and lost-committed-suffix
  below a checkpoint.
- UCR-004 (high, retry ignores durability): `Binding.normalized_request` = SHA-256(payload,
  requested durability); a retry with a different level is `Rejected { RequestIdReuse }`;
  `Outcome::Committed` carries `achieved: Durability` and duplicates return the original's.
  No upgrade protocol; new request id required.
- UCR-005 (high, early I/O errors): a fail-stop rule now covers every potentially mutating
  write or sync failure at any step (poisoned until `Log::open` re-establishes the safe append
  position); outcome reporting is separated from writability; unit tests must inject short
  writes, zero progress, write errors and sync errors at every step through a writer/syncer
  trait.
- UCR-006 (high, retry expiry vs lifecycle validator): retention scope is every binding in the
  history, always; the checkpoint's resolved table is a cache that must agree with the scan;
  no expiry (deferred to an explicit retention design, REQ-011). `RETRY_RETENTION` removed.
- UCR-007 (medium, paths): all path dependencies now use `../../../` from the crate
  directories.
- UCR-008 (medium, shared runner metadata and untracked sources): new R8b authorizes one
  backward-compatible change to `cm-trace`: `series_with_meta(SeriesMeta { experiment,
  hypothesis, source_prefixes })`, existing functions unchanged and output byte-identical
  (tested); the source manifest covers tracked and untracked non-ignored files under the
  prefixes; `uc-harness` passes EXP-0003/HYP-0003 and the four relevant prefixes; the
  EXP-0002 proof chain is rerun as part of the proof.
