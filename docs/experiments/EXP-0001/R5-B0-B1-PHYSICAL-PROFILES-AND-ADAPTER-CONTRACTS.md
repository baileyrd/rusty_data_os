# R5 — B0/B1 Physical Profiles and Adapter Contracts

**Status:** incomplete documentation design; not implemented or empirically validated
**Scope:** EXP-0001 B0 and B1 only
**Authority:** R1–R4, ADR-0002, and the EXP-0000 semantic, baseline, workload, recovery, environment, raw-result, interpretation, and methodology contracts

## 1. Decision boundary

This record narrows BLK-016 and BLK-017 and documents the B0/B1 portions of BLK-019, but does **not** resolve them. Review exposed that an exact R3/R1-compliant B1 D2/D3 mapping cannot be completed until BLK-001 selects the physical encoding, durable request/event binding, reservation/high-water representation, and recoverable commit mechanism, and BLK-003 selects the integrity/finalization mechanism. The initial Linux append and synchronization API/error profile is fixed conditionally; the post-boundary finalization and recoverable-commit path is not. BLK-015 also remains open for final placement, protection, and empirical survival. R5 is therefore not complete and R6 is not authorized.

Canonical history remains the sole authority. The profiles do not select record framing, field layout, commit markers, side records, in-place updates, integrity/checksum algorithms, digest or identity serialization, or numeric grouping limits. In particular, the abstract operations below are obligations, not permission to assume that a second append, rewrite, sidecar, marker, or any other unreviewed mechanism satisfies them.

## 2. Required lifecycle and physical evidence

Every eligible adapter must map the following ordered lifecycle without collapsing an observation into a durable fact:

1. **Stable request/event binding:** establish the R3 retry-stable request identity, one event identity, and normalized request relation in recoverable evidence before canonical commit.
2. **Sequence reservation:** durably establish the assigned sequence and advance a recoverable reservation high-water mark. Failure may leave a permanent legal gap; restart must never reuse it.
3. **Submission:** submit the event's provisional physical representation and record byte extents/outcomes. Submitted bytes are not yet a finalized canonical record.
4. **Declared boundary:** complete every file-content and required namespace synchronization operation named by the selected platform contract for all evidence that must survive at this point.
5. **Durability-time capture:** after that boundary succeeds, take **one exact clock sample per event**. D3 may share the boundary observation context, but must still perform and retain one sample for each event; one observation copied or “derived” into several event records does not satisfy R3.
6. **Envelope/integrity finalization:** incorporate that event's sampled durability time and all other covered final values, then compute and freeze the BLK-003 integrity metadata. The finalized record is immutable under R1 §4.2.
7. **Recoverable commit establishment:** durably establish the finalized record, its request/event binding, its assigned sequence/reservation state, and its canonical commit state under the eventual BLK-001 mechanism. This step requires the content and namespace synchronization necessary for recovery to distinguish committed from provisional bytes. It is not satisfied by the earlier boundary, because the durability-time value and finalized integrity did not yet exist then.
8. **Canonical commit and visibility:** only after recoverable establishment succeeds may the adapter transition the event to canonical committed and publish it to canonical readers.
9. **Acknowledgement:** acknowledge only after canonical commit. A lost acknowledgement after commit produces R3's uncertain caller outcome; it does not undo the fact.
10. **Reconciliation:** lookup/retry and restart scanning must use the durable binding, reservation high-water mark, finalized integrity, and commit evidence to return the same event, preserve legal gaps, classify provisional residue, or fail closed without inventing or duplicating a fact.

Evidence must identify each operation and outcome, exact event/request/sequence, byte extents, boundary and namespace operations, the single per-event sample, finalization profile, recoverable-commit evidence, visibility/acknowledgement class, and reconciliation result. Because BLK-001/003 are open, R5 cannot name the exact write/synchronization sequence that realizes steps 1–7 and cannot claim D2/D3 equivalence.

## 3. B0 — provisional in-memory lower-bound candidate

B0 remains one process, one owning thread, and one preallocated growable contiguous vector. For a validated candidate it constructs an entry, reserves capacity, assigns the next **process-local** sequence, moves the entry once into the vector tail, publishes only to a provisional observer, and returns a provisional D0 acknowledgement. Allocation, capacity, construction, insertion, logical bytes, stage time, and outcome are accounted explicitly.

This process-local counter is intentionally **not** R3's durable binding/reservation/high-water mechanism. Process loss destroys entries, request/event reconciliation state, and the counter; no gap or no-reuse promise survives restart. B0 therefore measures a lower bound only and must not be described as retaining the complete R3 lifecycle. It has no physical finalization, declared durability boundary, durability-time sample, integrity finalization, recoverable commit establishment, canonical visibility, or recovery.

Validation, allocation, overflow, exhaustion, or resource failure rejects before publication. If insertion completion is uncertain, remove the candidate or fail the run closed without acknowledgement. Persistence, serialization solely for another baseline, background promotion, synchronization, recovery artifacts, indexes, materialization, and hidden durability work are excluded.

## 4. B1 — conditional Linux raw-append substrate

### 4.1 Initial API, placement, and submission profile

Open the selected regular file relative to an opened parent directory with `O_WRONLY|O_APPEND|O_CLOEXEC|O_CREAT`; do not use `O_DIRECT`, `O_SYNC`, or `O_DSYNC`. One owning writer holds the descriptor. Bind `fstat`, `/proc/self/fd`, available `statx`, mount, filesystem, and block-stack identity to the run; symlinks, unexpected identity/placement, replacement, remount, configuration drift, or concurrent writers invalidate it.

Submit each provisional representation using `write(fd, remaining, remaining_len)` until complete. Positive short writes advance only by returned bytes; `EINTR` before progress retries; zero progress is terminal. No later record interleaves with an incomplete one. Logical offsets, byte counts, call outcomes, and provisional residue are retained for R1 scanning.

This fixes only the initial append API. It does not decide whether the durable binding/reservation, post-boundary finalized envelope, integrity value, or commit state lives in the record, a marker, a side structure, or another representation. Those are BLK-001/003 choices and may change the required write and synchronization sequence.

### 4.2 D1

D1 stops after complete buffered submission. It has no declared synchronization boundary, durability-time sample, finalized canonical record, recoverable commit, canonical visibility, or recovery obligation. Its acknowledgement means only **complete OS-buffer submission, noncanonical**. Surviving bytes remain provisional.

### 4.3 D2 and controlled D3

A D2 candidate requires all ten steps in section 2, including one exact durability-time sample for that event after the declared pre-finalization boundary and a later recoverable establishment of its finalized envelope and commit state. Controlled D3 freezes observable membership and serial submission before a shared declared boundary, but then requires one exact post-boundary sample, finalization, recoverable commit establishment, canonical transition, visibility, and acknowledgement **for each member**. Formation wait is included. It remains a set of individual commits, not an atomic multi-event transaction.

One `fsync(data_fd)` immediately followed by sampling and an in-memory commit cannot meet these requirements: the sampled durability time, finalized integrity, durable binding/reservation state, and commit evidence were not recoverable at that boundary. Likewise, one shared clock observation copied into member records is not one exact sample per event. R5 does not silently add a second append, rewrite, commit marker, side record, or synchronization call to repair that gap. Consequently B1 D2 and controlled D3 are incomplete and unsupported for equivalence until BLK-001/003 select a compliant realization and its exact boundary sequence is reviewed; BLK-015 and evidence gates would still remain afterward.

### 4.4 File content and namespace synchronization

Every declared boundary must enumerate the file-content and namespace facts it promises. For a newly created file, synchronize required file content before synchronizing its parent directory; an event cannot cross the relevant boundary until both succeed. Link, rename, replace, rotation, and deletion require synchronization of affected content and every affected parent directory, including both old and new parents when distinct. Unsupported directory synchronization makes that profile unsupported.

The post-boundary finalization and recoverable-commit mechanism may itself create or alter file or namespace state. Its required content and directory synchronization belongs to step 7, after the per-event sample and finalization, and cannot be credited to step 4. Exact operations remain unresolved with BLK-001/003. Namespace operations during a measured run are prohibited unless a later reviewed profile explicitly maps and accounts for them.

### 4.5 Late and close errors

Before canonical commit, a delayed/writeback, synchronization, namespace, or `close` error stops new acknowledgements, terminates the run, preserves evidence, and sends uncertain provisional outcomes to R1/R3 reconciliation. `close` is never retried. After recoverable commit and acknowledgement, a later error must **not** retroactively demote, delete, or relabel the accepted fact. It stops new acknowledgements and triggers reconciliation; if the already promised event cannot be recovered, the run/series is a correctness failure and its result is invalid under the existing recovery and interpretation authorities. Uncertainty is attached to affected pre-commit candidates or the caller's lost acknowledgement, never used to rewrite established canonical history.

## 5. Adapter/equivalence matrix

| Cell | Classification | Explicit mapping and evidence |
|---|---|---|
| **B0 × D0** | Conditional lower-bound candidate; incomplete against full R3 | Process-local request/event fields, capacity reservation, process-local sequence, vector insertion, provisional visibility/ack; no durable binding/reservation, boundary, sample, finalized commit, or reconciliation after loss. |
| **B0 × D1** | Unsupported | No OS-buffer submission or D1 acknowledgement. |
| **B0 × D2** | Unsupported | No durable reservation, synchronization boundary, durability-time sample, recoverable commit, or recovery. |
| **B0 × D3** | Unsupported | No durable membership, shared boundary, per-event sample, or recoverable member commits. |
| **B1 × D0** | Diagnostic only | Initial write-loop costs persistence work and can expose only provisional bytes. |
| **B1 × D1** | Conditionally mapped, noncanonical | Complete initial `write` loop and provisional acknowledgement; offsets/errors are evidence, with no canonical recovery obligation. Exact framing still depends on BLK-001. |
| **B1 × D2** | Incomplete; equivalence unsupported | Required section 2 mapping is known, but BLK-001/003 leave durable binding/reservation, post-boundary finalization, recoverable commit, and exact content/namespace synchronization operations unselected. |
| **B1 × controlled D3** | Incomplete; equivalence unsupported | Observable membership/shared-boundary intent is known, but each member's exact sample, finalized integrity, recoverable commit, and reconciliation realization remains unselected under BLK-001/003. |

Workload identity, semantic validation, and accounting obligations still apply to every exercised cell. They do not make unsupported cells equivalent or give B0 durable R3 semantics.

## 6. Required error-path dispositions

| Required path | Required disposition |
|---|---|
| Short write | Advance only by returned bytes; continue the same provisional unit; record calls and bytes. |
| Zero progress | Terminal run failure; stop acknowledgements and preserve residue for reconciliation. |
| `EINTR` | Retry the remaining operation without duplicating submitted bytes. |
| `ENOSPC` | Reject before progress or terminate after progress; reconcile any uncertain pre-commit residue. |
| `EDQUOT` | Same fail-closed resource disposition as `ENOSPC`. |
| `EROFS` | Configuration failure; reject or terminate and invalidate the run. |
| I/O/device loss | Stop new acknowledgements, preserve evidence, and reconcile; unrecovered promised facts are correctness/result failures. |
| Delayed/writeback error | Never retroactively demote a committed fact; stop new acknowledgements and classify violated recovery promises as correctness/result failures. |
| Synchronization failure | No dependent boundary success or new commit; reconcile uncertain pre-commit outcomes. |
| Namespace synchronization failure | No dependent boundary success or new commit; stop and reconcile. |
| `close` error | Do not retry close or demote committed facts; stop acknowledgements, reconcile uncertainty, and fail correctness if promised recovery is violated. |

## 7. Blocker disposition and continuation gate

| Item | Corrected R5 disposition |
|---|---|
| BLK-015 | Open: final placement, exact protection, and empirical survival remain unverified; the initial B1 API choice does not close the overall platform claim. |
| BLK-016 | Narrowed, not resolved: the exact B0 lower-bound operations are described, but its process-local sequence is explicitly not R3 durable reservation/binding and implementation/evidence remain gated. |
| BLK-017 | Narrowed, not resolved: initial append/error API is described; post-boundary finalization, recoverable commit, and their exact synchronization sequence depend on BLK-001/003. |
| BLK-019 | Incomplete: B0/B1 mappings now identify their semantic gaps honestly; no D2/D3 equivalence is established, and B2/B3 remain unstarted. |

R5 supplies a bounded negative design outcome: the proposed one-`fsync` D2/D3 path is insufficient, and the excluded encoding/integrity/commit choices are necessary to finish the mapping. R5 must remain incomplete, and R6 must not begin, until the governing blockers are resolved or the readiness plan explicitly reorders work through review. No implementation, execution, benchmark, fixture, capture, workflow, Cargo, durability, or performance claim is authorized.
