# R5 — B0/B1 Physical Profiles and Adapter Contracts

**Status:** complete as documentation design; not implemented or empirically validated  
**Scope:** EXP-0001 B0 and B1 only  
**Authority:** R1–R4, ADR-0002, and the EXP-0000 semantic, baseline, workload, recovery, environment, raw-result, interpretation, and methodology contracts

## 1. Decision boundary

This record resolves BLK-016 and BLK-017 and the B0/B1 portions of BLK-019 at the design level. It does not close BLK-015, prove equivalence, authorize implementation or execution, or make a durability or survival claim. R6 documentation is the next readiness boundary.

Canonical history remains the sole authority. A canonical event is an accepted fact, never a command or provisional candidate. Effective, system, durability, observation, sequence/replay, and lifecycle times remain distinct. Derived memory, indexes, materializations, and checkpoints remain rebuildable.

The profiles deliberately do not select record encoding, framing, integrity/checksum algorithm, digest or identity serialization, numeric grouping limits, benchmark apparatus, final paths, or an execution series. SQLite and RocksDB are outside R5.

## 2. Common lifecycle and evidence vocabulary

For one already validated semantic candidate, the adapter records these ordered milestones without conflating them:

1. **Construction** produces the candidate's complete semantic fields and eventual physical bytes; it accepts no fact.
2. **Insertion/submission** transfers the complete unit into the selected memory structure or through the complete B1 write loop.
3. **Physical finalization** means the selected record bytes have been fully submitted; it is not a durability boundary.
4. **Boundary success** is the successful return of the declared synchronization operation, if any.
5. **Durability-time capture** occurs immediately after boundary success and before canonical commit. It records clock observation, not proof beyond the declared boundary.
6. **Canonical commit** is the adapter's explicit per-event state transition after every cell-specific prerequisite succeeds.
7. **Visibility** publishes only committed events to canonical readers; a separate provisional view may expose D0/D1 candidates while labeling them noncanonical.
8. **Acknowledgement** reports the cell's declared outcome only after its required transition. An error or uncertain outcome never invents an accepted fact.

Each operation must expose outcome, errno where applicable, byte counts, offsets, boundary identity, group identity/membership where applicable, milestone timestamps, acknowledgement class, and accounting counters. Later byte survival cannot retroactively turn D0 or D1 into canonical history.

## 3. B0 — exact D0 in-memory lower bound

### 3.1 Structure and operation

B0 is one process, one owning thread, and one preallocated growable contiguous vector of entries. Each entry contains the complete semantic candidate plus its sequence and lifecycle/accounting metadata. The measured operation is exactly:

1. validate the candidate against the frozen semantic envelope;
2. construct one entry outside the vector;
3. reserve capacity, if required, before assigning or publishing the sequence;
4. assign the next process-local sequence without advancing it on failure;
5. move the entry once into the vector tail; and
6. publish its index only to the explicitly provisional observer and return a **provisional D0 acknowledgement**.

Construction includes semantic validation, field population, sequence assignment, and entry allocation/copying. Insertion includes capacity reservation, the single move into the tail, length publication, and provisional-index publication. Accounting includes elapsed lifecycle stages, allocation/capacity changes, bytes logically constructed and retained, one attempted outcome, and outcome class.

### 3.2 Semantics and exclusions

D0 has no physical finalization or durability boundary, captures no durability time, performs no canonical commit, and grants no canonical-reader visibility. Its acknowledgement means only that the process-local provisional entry was inserted. It provides no persistence or recovery; process loss loses every entry.

No serialization performed solely for another baseline, file or synchronization call, background promotion, hidden flush, retry worker, index, materialization, checkpoint, recovery artifact, or durability work belongs in B0. Common semantic work required by all cells remains included and separately accounted.

Validation failure, allocation failure, capacity/size overflow, sequence exhaustion, or any resource limit returns a typed rejection before publication. If insertion cannot be proven complete, the candidate is removed or the whole B0 run fails closed; it receives no acknowledgement and no sequence is reusable as an accepted fact. Panics or process loss produce no recovery claim.

## 4. B1 — conditional Linux raw append

### 4.1 Placement, open, ownership, and descriptor lifecycle

Before a run, an operator-selected regular file must be beneath one of the R4 intended local placements. Open its parent directory with `open(..., O_RDONLY|O_DIRECTORY|O_CLOEXEC)` and the data file with `openat(parent_fd, name, O_WRONLY|O_APPEND|O_CLOEXEC|O_CREAT, mode)`. Do not use `O_DIRECT`, `O_SYNC`, or `O_DSYNC`; the declared boundaries below are explicit. Existing files require verified identity and an already validated R1 valid prefix before appending.

Immediately after opening, bind the descriptor to evidence from `fstat`, `/proc/self/fd/<fd>`, `statx` where available, and the applicable mount/block-stack observations required by R4. Record device/inode, resolved path, file type, mount identity/options, filesystem, and stack identity. A symlink, non-regular file, unexpected device/mount/stack, path escape, identity change, replacement, remount, or configuration drift invalidates the cell and stops it before acknowledgement. Recheck at series/run boundaries and after any namespace operation; execution design must add detection adequate to prevent silent placement changes.

Exactly one owning writer thread holds the descriptor. No process, thread, library, or background worker may append concurrently. `O_APPEND` prevents accidental explicit-offset placement but is not the concurrency model. The writer maintains the expected logical end offset from the validated prefix and verifies `fstat` size at controlled boundaries. The descriptor stays open for one run/segment and is closed only after all required synchronization and error checks. `close` errors are recorded; because `close` must not be retried, uncertainty fails the run closed.

### 4.2 Complete record submission and errors

For each as-yet-unselected complete encoded record, call `write(fd, remaining, remaining_len)` in a loop. Positive short writes advance the buffer pointer and expected offset and continue. `EINTR` before progress retries the remaining call. A zero return while bytes remain is zero progress and terminal for the run. No later record may interleave with an incomplete record.

`ENOSPC`, `EDQUOT`, `EROFS`, `EFBIG`, size/offset overflow, permission loss, or other permanent resource/configuration errors reject the candidate when no bytes were submitted; after any bytes were submitted they create provisional residue and terminate the run for R1 recovery. `EIO`, device disappearance/loss, filesystem shutdown, or an unexpected descriptor/path/stack condition is terminal. Delayed/writeback errors observed by later `write`, synchronization, or `close` are attributed to the affected run/boundary conservatively; all outcomes whose canonical status cannot be proven fail closed.

The R1 scanner alone determines the valid prefix. An incomplete, malformed, integrity-failing, or otherwise unverifiable tail is terminal damage; bytes after it are not searched for valid records. Provisional residue may be truncated only by a separately controlled recovery action to the proven prefix. No unresolved encoding or integrity mechanism is selected here, and no successful syscall proves torn-write prevention or survival.

### 4.3 D1 — buffered provisional append

D1 ends after the complete write loop. Complete submission is physical finalization for accounting, but there is no synchronization call, declared durability-boundary success, durability time, or canonical commit. Only an explicitly provisional D1 view may observe the submitted candidate. Its acknowledgement says **complete OS-buffer submission, noncanonical**. It has no recovery obligation, even if bytes later survive and form a valid record. Fault observations are diagnostic only.

### 4.4 D2 — conditional per-event boundary

For each event, D2 performs the complete write loop and then exactly one `fsync(data_fd)`. `EINTR` retries `fsync`; any other failure, including `ENOSPC`, `EDQUOT`, `EROFS`, `EIO`, device loss, or delayed/writeback error, prevents boundary success and canonical commit and terminates the run when status is uncertain. No subsequent event begins before the outcome is known.

After `fsync` returns success, capture durability time, transition that one event to canonical committed, publish it to canonical readers, and acknowledge it. The per-event acknowledgement therefore follows one per-event boundary. Physical finalization, successful `fsync`, durability-time observation, commit, visibility, and acknowledgement remain separate evidence fields.

D2 is only conditionally eligible on the exact R4 Fedora/XFS/LVM/NVMe profile, intended placement, validated namespace state, and still-open BLK-015 prerequisites. `fsync` success expresses only the declared Linux boundary; it does not establish PLP, stable media, power-loss safety, atomic/torn-write prevention, or empirical recovery.

### 4.5 Controlled D3 — conditional shared boundary

Controlled D3 uses a single writer and a predeclared deterministic maximum member count **or** maximum formation interval, whose numeric value remains for a later freeze. A candidate may join only after complete semantic validation and before the earlier cut condition fires. The writer records membership and join time before writing. Once cut, membership is immutable; arrivals after the cut join the next group. Empty groups are forbidden.

Members are written serially and contiguously with the complete write loop. After all members reach physical finalization, exactly one `fsync(data_fd)` is the shared boundary. On success, capture one post-boundary observation and derive distinct per-event durability-time records tied to that boundary, then commit, publish, and acknowledge every member individually in sequence order. Each event's latency includes formation wait. One shared outcome is recorded for the exact membership.

Before shared boundary success, no member is canonical or acknowledged. A write or synchronization failure fails every uncommitted member closed and sends any partial/provisional tail to R1 recovery; none may be selectively acknowledged. This is a set of individual single-event commits sharing one boundary, **not** an atomic multi-event transaction: readers may observe individual publication order, and R5 promises no all-or-nothing application semantics. Strict D3 is conditionally equivalent only to that controlled grouping contract, never to opaque background group commit or a database transaction.

### 4.6 File content versus namespace durability

`fsync(data_fd)` is the selected file-content boundary. When a file is newly created, its parent directory must also receive `fsync(parent_fd)` before any event in it can pass a D2/D3 declared boundary; the file is synchronized first, then the parent directory. A parent-directory failure prevents commit.

For link, rename, replace, rotation, or deletion, synchronize affected file content first when that content must survive, perform the namespace syscall, then `fsync` every affected parent directory (both old and new parents when distinct). Replacement/rotation synchronizes the new file before rename, then all affected parents; deletion synchronizes the containing parent after unlink. A newly created destination followed by rename does not inherit namespace durability from file `fsync`. R5 does not prescribe rotation/deletion during a measured run; if later allowed, each operation is an explicit boundary and invalidates results unless its work and failures are accounted. Directory synchronization unsupported by the selected filesystem makes the corresponding profile unsupported.

## 5. Adapter/equivalence matrix

“Conditionally equivalent” means the design can satisfy the EXP-0000 semantic cell after every prerequisite and correctness gate passes; it is not empirical proof.

| Cell | Classification | Physical mapping and acknowledgement | Visibility/recovery/fault/accounting | Preconditions, exclusions, invalidators |
|---|---|---|---|---|
| **B0 × D0** | Equivalent lower-bound design; empirically unproved | Vector-tail insert; provisional D0 acknowledgement | Provisional view only; no recovery; allocation/resource failures reject; semantic and memory work counted | One owner and exact structure; persistence, promotion, indexes and hidden work excluded; structure/allocator/path changes start a new profile |
| **B0 × D1** | Unsupported | B0 performs no OS-buffer append | No D1 evidence or acknowledgement | Adding persistence ceases to be B0 |
| **B0 × D2** | Unsupported | No synchronization boundary | No canonical visibility, recovery, or durability time | Cannot infer D2 from memory survival |
| **B0 × D3** | Unsupported | No shared persistence boundary | No membership/boundary evidence | In-memory batching is not D3 |
| **B1 × D0** | Diagnostic only | Complete write-loop cost may not be labeled D0 | Provisional only; R1 prefix diagnosis; write faults recorded | Persistence work makes it nonequivalent to B0 D0 |
| **B1 × D1** | Conditionally equivalent | `write` loop through complete submission; provisional D1 acknowledgement | Provisional only; no recovery obligation; error paths and bytes/offsets counted | Exact descriptor, ownership and placement profile; any sync, concurrent writer, stack drift, or hidden flush invalidates |
| **B1 × D2** | Conditionally equivalent | Complete write loop + one successful per-event `fsync`; post-success durability time, per-event commit/visibility/ack | Recover every canonically acknowledged event under R1 oracle; all write/sync faults fail closed; formation excluded | Exact R4 stack plus BLK-015, final placement, namespace and empirical gates; grouping/config drift invalidates |
| **B1 × controlled D3** | Conditionally equivalent to controlled grouping only | Frozen join/cut membership, serial writes, one shared `fsync`, then per-event commit/visibility/ack | All members share boundary outcome; R1 recovery; formation wait and per-event work counted; no transaction atomicity | Exact D2 prerequisites plus frozen group policy and observable membership; opaque grouping, concurrent writers, policy/stack drift invalidates |

All eight cells retain workload identity, semantic validation, request/event identities, sequence/replay rules, retry/uncertain-outcome rules, and accounting required by R1–R4 and EXP-0000. A configuration change creates a new profile/series; it cannot silently inherit equivalence.

## 6. Error-path mapping review

| Required path | Required disposition |
|---|---|
| Short write | Advance only by returned bytes; continue the same record; count every call and byte. |
| Zero progress | Terminal run failure with no acknowledgement; preserve provisional residue for recovery. |
| `EINTR` | Retry the remaining write or synchronization without duplicating submitted bytes. |
| `ENOSPC` | Reject before progress or terminate after progress; no uncertain canonical fact. |
| `EDQUOT` (quota failure) | Same fail-closed resource disposition as `ENOSPC`. |
| `EROFS` | Configuration failure; reject/terminate and invalidate the run. |
| I/O/device loss | Terminal; stop appends, withhold commit/ack, invoke R1 recovery. |
| Delayed/writeback error | Surface at later write/sync/close, conservatively invalidate affected outcomes, fail closed. |
| Synchronization failure | No boundary success, durability time, commit, visibility, or acknowledgement. |
| Namespace synchronization failure | No dependent D2/D3 commit; stop and retain evidence for controlled recovery. |

## 7. Traceability and disposition

| Item | R5 disposition |
|---|---|
| BLK-016 | Resolved at design level by section 3; implementation and validation remain gated. |
| BLK-017 | Resolved at design level by section 4; BLK-015 and execution evidence remain open. |
| BLK-019 | B0/B1 portions resolved at design level by section 5; B2/B3 remain for R6. |
| REQ-001–REQ-006 | Canonical authority, accepted-fact boundary, order, time, identity/provenance, and explicit durability lifecycle preserved. |
| REQ-009 | Opaque payload and unresolved physical encoding remain separate. |
| REQ-013/REQ-014 | Recovery oracle, valid-prefix/terminal-damage rules, and explicit boundary evidence preserved. |
| RQ-003 | Physical design is now specified; cost/correctness evidence remains unanswered. |
| UNK-014 | B1 is bound conditionally to R4; final execution placement/provenance remain open. |
| UNK-020 | B0/B1 mapping portion narrowed; implementations, versions, harness, and B2/B3 remain open. |
| UNK-021 | B1 D2/D3 conditions are explicit; SQLite/RocksDB eligibility remains open. |

This record depends on ADR-0002; EXP-0000 baseline, acknowledgement/visibility/durability, crash/recovery, workload, environment, raw-result, interpretation, and methodology contracts; and R1–R4. Those authorities prevail if a conflict is found.

## 8. Eligibility, evidence, and prohibited claims

R5 establishes design eligibility only. Equivalence becomes evidence only after implementation authorization, independent oracle validation, exact environment/series capture, applicable fault execution, retained raw results, and the descriptive or confirmatory gate. BLK-015 remains open, as do BLK-001/003 and every execution/evidence gate.

No documentation statement or successful `write`, `fsync`, directory `fsync`, or `close` establishes stable-media residence, PLP/controller protection, power-loss safety, torn-write prevention, exactly-once behavior, atomic multi-event transactions, empirical survival, acceptable performance, or production readiness. No code, Cargo file, workflow, fixture, vector, validator, harness, adapter, instrumentation, benchmark, fault execution, machine change, evidence archive, or sensitive identifier is part of R5.
