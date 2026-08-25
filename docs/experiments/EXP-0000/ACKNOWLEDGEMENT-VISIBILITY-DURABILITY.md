# Acknowledgement, Visibility, Fault, and Durability Contract

**Status:** Complete EXP-0000 readiness output; semantic contract only, not experimental evidence

## 1. Purpose and scope

This contract makes EXP-0001 ingestion results comparable by naming the lifecycle interval and the guarantee attached to every acknowledgement. It refines the D0–D3 candidate modes without selecting an encoding, record framing, identity algorithm, timestamp representation, clock, filesystem API, or storage implementation. EXP-0001 remains single-event commit: a D3 durability group shares persistence work but is not an atomic multi-event transaction.

The contract preserves [REQ-013](../../REQUIREMENTS.md): canonical commit follows completion of the declared durability boundary. Visibility before that point is provisional and is never durable canonical commit. D2 and D3 are canonical only when their recorded platform contract supports the declared boundary; D0 and D1 are always provisional.

## 2. Lifecycle points

These are semantic points, not required threads, queues, or pipeline stages:

| Point | Definition and status |
|---|---|
| **Command submission** | The caller presents requested intent. Submission is neither acceptance nor an event. Its instant starts submission-to-acknowledgement latency when that interval is reported. |
| **Validation and rejection** | Validation decides whether the command is eligible to become an accepted fact. Rejection ends this command attempt and remains separate operational/audit evidence; it creates no canonical event asserting the requested fact. |
| **Event construction** | The immutable-envelope candidate and opaque payload are formed. It is a candidate accepted fact, not canonical history. |
| **Sequencing** | A local monotonic replay position is assigned. Sequence is not commit and does not make a candidate recoverable. |
| **Persistence submission** | The candidate is handed to the persistence path selected by the mode. Submission does not by itself establish completion or survival. |
| **Provisional memory visibility** | A declared provisional reader may observe a candidate that is not canonically committed. Such a reader must expose the provisional status and tolerate later disappearance after rejection, error, or failure. |
| **Durability-boundary completion** | The mode's declared synchronization operation has completed successfully under its recorded platform durability contract. This establishes only the intended guarantees that contract actually states; no synchronization call is assumed universally power-loss durable. |
| **Canonical commit** | The event joins immutable authoritative history. It occurs only after durability-boundary completion and is the point after which canonical readers may be permitted to observe the event. |
| **Caller acknowledgement** | The result returned for the command attempt. A successful acknowledgement names its D-mode, lifecycle point, canonical/provisional status, and valid latency interval; the word “success” alone is insufficient. |
| **Canonical-reader visibility** | The committed event becomes observable through a reader that exposes authoritative canonical history. It cannot precede canonical commit. |
| **Materializer visibility/freshness** | A derived representation incorporates a committed event and exposes the represented canonical sequence/watermark. Its lag is distinct from ingestion acknowledgement latency. |
| **Observation time** | The time a declared reader or materializer could observe the event in a named context. It is observation-side lifecycle metadata, not part of the immutable original envelope. If an observation must become canonical information, a separate event is appended. |

## 3. Ordering and permitted relationships

For an accepted command, the required partial order is:

```text
command submission -> successful validation -> event construction -> sequencing
  -> persistence submission -> [durability-boundary completion -> canonical commit]
  -> canonical-reader visibility
                         canonical commit -> materializer visibility (for that materializer)
```

The bracketed points exist only for a canonical D2/D3 outcome. Rejection follows validation and terminates the accepted-event path. Construction and sequencing may be internal to one operation, but their semantic order remains distinguishable. Persistence submission precedes any completion attributed to it.

- D0 acknowledgement occurs after provisional memory acceptance and may precede persistence submission. D1 acknowledgement occurs after successful persistence submission to OS buffering. Both precede canonical commit because neither mode completes the required stable-storage boundary.
- D2/D3 successful canonical acknowledgement occurs no earlier than canonical commit. An implementation may commit before returning acknowledgement, so acknowledgement return need not equal commit time.
- Provisional visibility may occur after construction or sequencing according to a declared reader contract, but never implies canonical-reader visibility. Canonical-reader and materializer visibility cannot precede commit; neither visibility point is required to precede the other. Each materializer must report its own watermark/freshness.
- Sequencing candidates that later fail may leave gaps if the eventually selected sequencing contract permits them; gap policy remains unresolved. They must never be replayed as committed events merely because they received a sequence.
- Observation time follows the observation it describes. It must identify observer/context and cannot be substituted for system acceptance, durability, commit, acknowledgement, or materializer incorporation time.
- Implementations may make adjacent points coincide. Results must not infer equality unless the measurement definition and instrumentation establish it.

## 4. Benchmark durability-mode matrix

Every result labels one of these modes and records the exact platform contract and latency interval. “Intended survival” is a promised test obligation, not evidence that the behavior has already been demonstrated.

| Mode | Acknowledgement and status | Visibility | Intended survival | Excluded guarantees | Group semantics | Valid per-event latency | Recovery/correctness obligation |
|---|---|---|---|---|---|---|---|
| **D0 — process-memory provisional acceptance** | After insertion into the declared process-memory structure; always provisional, never canonical commit. | Only named provisional readers; no canonical reader or committed-history materializer visibility. | Normal process operation only. | Process termination, OS crash, reset/power loss, stable storage, and recovery are not promised. | None; implementation batching after acknowledgement does not strengthen D0. | Command submission to acknowledgement return, labeled **provisional-accept latency**. | Acknowledged candidates may disappear after process failure; tests must ensure they are never recovered or exposed as committed unless a later, separately observed canonical transition occurred. |
| **D1 — OS-buffer provisional acceptance** | After the persistence submission is accepted by the OS without explicit declared stable-storage synchronization; always provisional, never canonical commit. | Named provisional readers only; no canonical reader or committed-history materializer visibility. | Process termination may be intended only if the OS and persistence contract says buffered bytes remain recoverable while the OS continues running. | OS/kernel crash, reset/power loss, stable-media persistence, and canonical commit are not promised. | Submission batching may be recorded, but no batch is a durability group and membership strengthens no guarantee. | Command submission to acknowledgement return, labeled **OS-buffer-accept latency**. | Recovery tests target only declared process-crash survival; missing buffered events after stronger faults do not violate D1. Recovered bytes must not be promoted silently if integrity/recovery rules reject them. |
| **D2 — per-event declared stable-storage synchronization** | After that event's declared synchronization completes and canonical commit occurs; canonical only under an explicit platform durability contract. | Canonical readers may observe after commit; materializers may lag and report freshness. No pre-commit observation is canonical. | The fault classes explicitly covered by the platform contract, potentially including process/OS crash and reset/power loss only when stated and justified. | Any unrecorded fault, device/cache behavior, metadata operation, or hardware guarantee; universal power-loss durability is never implied. | Exactly one event owns the synchronization outcome; unrelated implementation coalescing must be disclosed and must not turn the result into mislabeled D2. | Command submission to canonical acknowledgement return, labeled **per-event commit latency**; optional component intervals may supplement it. | Every acknowledged committed event must recover after each claimed fault; no unacknowledged candidate may be invented as committed. Partial/error cases follow deterministic recovery and integrity rules. |
| **D3 — grouped declared stable-storage synchronization** | After the group's declared synchronization completes, the event's group outcome succeeds, and canonical commit occurs; canonical only under an explicit platform durability contract. | As D2 after commit; provisional exposure before group completion must remain labeled provisional. | The same explicitly recorded fault classes as the group's platform contract promises. | Atomic multi-event transaction semantics, guarantees outside the contract, and universal power-loss durability. | Membership and one shared durability outcome are mandatory as defined below. | Each event's submission to its canonical acknowledgement return, labeled **group-commit latency**; it includes that event's wait to join/fill the group and shared synchronization. | Every member acknowledged committed must recover after each claimed fault. Group errors acknowledge no member as committed; recovery must deterministically classify any physical prefix/partial result without inventing atomic transaction semantics. |

All modes additionally record: repository/configuration identity; OS and kernel/build; filesystem and relevant mount options; storage model/interface and device-cache behavior (including volatile cache and power-loss protection information when known); synchronization primitive and scope, including required data/metadata ordering; virtualization/container layers; cache/preconditioning state; and explicit assumptions, limitations, and unsupported faults. Intended guarantees and results of fault testing must be reported separately.

## 5. Minimum fault taxonomy

Each benchmark configuration maps its intended and excluded guarantees to these distinct faults:

1. **Process termination/crash:** abrupt loss of the benchmark process while the OS remains operating.
2. **OS or kernel crash:** loss of kernel state and volatile OS buffers without necessarily removing machine power.
3. **Power loss or machine reset:** loss/restart of the machine, including relevant controller/device caches and their power-loss behavior.
4. **Partial, torn, or truncated write:** only part of a logical or physical write is present or valid. This is a resulting storage condition and may accompany another fault.
5. **Explicit storage or I/O error:** an operation reports failure, including submission or synchronization errors; it must not be converted into successful acknowledgement.

The chosen crash-injection procedure and integrity policy remain separate incomplete EXP-0000 outputs. Therefore this taxonomy defines obligations and reporting dimensions, not a claim that a particular fault has been simulated or survived. A sync primitive's successful return supports only the platform contract declared for the exact OS, filesystem, mount, device/cache path, and operation scope.

## 6. D3 group-commit contract

- **Joining:** an event joins exactly one identified durability group at the documented cut point, no earlier than sequencing and no later than the group's persistence/synchronization submission. The implementation must record whether the cut point is count-, byte-, time-, queue-, or explicit-flush-triggered.
- **Acknowledgement:** no member receives a canonical acknowledgement until the shared declared durability boundary completes successfully and that member is canonically committed. Implementations may return acknowledgements separately after that common outcome.
- **Shared outcome:** all recorded members share one durability-operation outcome. Success makes every valid member eligible for individual canonical commit; failure makes none eligible for successful canonical acknowledgement. This is measurement grouping, not a general atomic transaction promise.
- **Errors:** validation errors occur before joining. A persistence or synchronization error fails the group, is surfaced to every member, and cannot be retried invisibly as though the original attempt committed. Retry and uncertain-outcome policy must be declared later. Recovery may find a physical prefix, but exposes events canonically only under the eventual deterministic recovery contract.
- **Latency:** the primary sample for each event runs from that event's command submission to its own canonical acknowledgement return. It includes queueing and group-formation delay. Supplementary submission-to-group-cut, group-cut-to-sync-completion, and completion-to-return intervals may be reported but cannot replace the primary interval.
- **Policy recording:** report maximum/actual event and byte counts, time-window start/cut semantics and duration, early-flush triggers, queue thresholds, concurrency, outstanding depth, group membership per event, actual fill distribution, synchronization count, and error/timeout behavior.

## 7. Required result declaration

Before a result is comparable, its configuration must state: D-mode; exact acknowledgement point and canonical/provisional status; eligible reader classes; primary lifecycle interval and timestamp/clock limitations; intended and excluded fault survival; recovery/correctness checks and their outcomes; batching/group policy; and the complete platform/storage contract above. A throughput figure must identify the same acknowledgement boundary as its latency samples. Results with different modes or platform contracts remain useful but are not durability-equivalent.

This document defines no integrity algorithm, fault-injection procedure, workload distribution, baseline, transaction design, or empirical conclusion. Those remain bounded follow-on work.
