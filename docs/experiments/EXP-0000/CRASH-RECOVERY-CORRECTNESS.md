# Crash/Recovery Correctness Contract

**Status:** Complete EXP-0000 readiness output; procedures and classifications only, not experimental evidence

## 1. Purpose and scope

This contract defines the reproducible correctness procedure by which EXP-0001 can determine whether recovered canonical history satisfies the selected D0–D3 mode and its recorded platform fault contract. It refines the [lifecycle and durability contract](ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), [semantic event envelope](SEMANTIC-EVENT-ENVELOPE.md), and REQ-001 through REQ-014 in the [requirements registry](../../REQUIREMENTS.md).

It does not implement a harness, execute a fault, select physical encoding, framing, integrity or identity algorithms, choose timestamps or clocks, prescribe a universal persistence API, or claim durability evidence. Retry, idempotency, sequence-gap, transaction, checkpoint-format, and distributed semantics remain unresolved. EXP-0001 remains blocked.

## 2. Recovery oracle

For each attempted command, the test controller maintains an **out-of-band lifecycle ledger**. The ledger is comparison evidence controlled independently of the faulted system; it is a test oracle, never an alternative authoritative source of system state and never input from which recovery may invent or repair canonical history.

Before injection, the controller assigns every operation exactly one expected class, updated only from lifecycle evidence observed before the fault:

| Oracle class | Meaning | Required recovery treatment |
|---|---|---|
| **Must recover** | A D2/D3 event was canonically committed and its successful canonical acknowledgement was observed before a fault covered by the declared platform contract. Canonical commit observed before acknowledgement interruption also establishes this class when independently instrumented. | Recover exactly once, with immutable identity and semantic contents, at its original sequence position. |
| **May recover: uncertain outcome** | Canonical commit may have completed, but interruption prevented the caller from receiving a conclusive acknowledgement and the controller cannot prove commit. This includes the commit-before-acknowledgement window. | Either absence or exactly one valid canonical event is permitted. Presence must be reported as a recovered canonical uncertain outcome, not an invented acknowledgement. |
| **Must not be promoted** | A constructed, sequenced, submitted, D0/D1-acknowledged, provisionally visible, or physically resident candidate lacks evidence of canonical commit. | Physical residue may be diagnosed, but must not silently enter canonical history. If the selected recovery policy cannot distinguish it from canonical data, recovery fails closed. |
| **Rejected** | Validation rejected the command. | No canonical event asserting the requested fact may appear. Rejection evidence remains noncanonical operational/audit evidence. |
| **Corrupt or undecidable** | Physical data cannot be validated or canonical status cannot be established under the declared integrity and recovery policy. | Produce an explicit recovery failure or explicitly degraded result identifying the affected extent; never silently accept or promote it. |

An unacknowledged event is not automatically noncanonical: canonical commit may precede acknowledgement return. Conversely, sequence assignment, persistence submission, physical presence, and D0/D1 acknowledgement never prove canonical status. Where instrumentation claims to observe canonical commit, the run must document that observation mechanism and show that it does not change the boundary under test.

## 3. Lifecycle fault-injection points

Each run selects one named point and a reproducible trigger. Boundary tests inject immediately before, while the operation is in progress where controllable, and immediately after observed completion. “During” is valid only when the mechanism can establish that the target operation had begun but had not observably completed.

| Lifecycle region | Required injection boundaries and oracle concern |
|---|---|
| Validation | Before validation, during validation, after rejection, and after successful validation. Rejection must never create a canonical fact. |
| Event construction | Before construction, during candidate formation, and after construction. Incomplete or complete candidates remain noncanonical. |
| Sequencing | Before assignment, during assignment, and after a sequence is assigned. A sequenced candidate may leave a reported gap but is not thereby committed. |
| Persistence submission | Before handoff, while submission is outstanding, after reported submission success, and after reported submission error. Submission and physical residue do not establish commit. |
| D0/D1 provisional acknowledgement | Immediately before return, during return/interruption, and after the caller observes it. These acknowledgements remain provisional; only a separately observed later D2/D3 canonical transition could change the oracle class. |
| Synchronization / durability-boundary completion | Before synchronization, while it is outstanding, after explicit failure, and after successful completion under the declared platform contract. A completion alone precedes canonical commit in the semantic lifecycle. |
| Canonical commit | Immediately before, during the transition where controllable, and immediately after commit. An undecidable transition fails closed; a proven completed commit must recover for a covered fault. |
| Canonical acknowledgement return | Before return after commit, during delivery, and after caller observation. The commit-complete/acknowledgement-not-observed window is explicitly an uncertain caller outcome and may contain a valid canonical event. |
| D3 group formation and shared synchronization | Before joining, immediately after joining, before and at every declared group-cut trigger, during shared submission/synchronization, after shared success or failure, during per-member commit, and before/during/after each acknowledgement return. Record membership and the one shared synchronization outcome without inferring atomic multi-event transaction behavior. |

Every applicable point is exercised across repeated seeds and placements rather than relying on a single timing race. A mechanism that cannot prove its placement yields an invalid run; one that proves placement but cannot reproduce the claimed failure boundary may yield an inconclusive result. This contract deliberately does not decide how callers retry an uncertain attempt or how request identities affect a retry.

## 4. Fault matrix

The platform durability contract for a configuration narrows the cells below; it cannot broaden them by implication. **Promised** means correctness must be demonstrated when that exact fault is claimed. **Permitted loss** applies only to noncanonical/provisional candidates; invalid promotion or silent corruption is never permitted. **No claim** means the mode supplies no survival evidence for that fault.

| Mode | Process termination | OS/kernel crash | Power loss/reset | Partial/torn/truncated data | Explicit persistence/sync error |
|---|---|---|---|---|---|
| **D0** | Permitted loss; no recovery promise | No claim | No claim | Test only if residue can exist through an undeclared later path; never promote it | Error must not be converted into canonical success; D0 may already have returned a clearly provisional acknowledgement |
| **D1** | Promised only when the recorded OS-buffer contract explicitly covers process loss; otherwise no claim | Permitted loss / no survival claim | Permitted loss / no survival claim | Detect/classify if present; never promote invalid or undecidable residue | Submission error cannot produce D1 success; later persistence/sync errors cannot retroactively make D1 canonical |
| **D2** | Promised when included in the platform contract | Promised only when included | Promised only when included and justified for the complete cache/device path | Inject or construct independently and combine with covered crash classes; deterministic detection/classification is required within the declared integrity capability | Failure cannot produce canonical acknowledgement; uncertain physical outcome is classified explicitly and fails closed if canonical status is undecidable |
| **D3** | As D2 for every committed member | As D2 | As D2 | As D2, with physical prefixes classified per event and group evidence retained; no group atomicity is inferred | Shared failure gives no member a successful canonical acknowledgement; partial/uncertain physical outcomes are classified explicitly |

A process kill while the OS continues running is never evidence of OS-crash or power-loss durability. Reports label mechanisms separately as **injected** (an operation/error hook), **simulated** (a software model), **virtualized** (for example VM reset), or **physical** (real machine power interruption). They record which volatile layers were actually lost. Simulation and virtualization can test logic but cannot alone establish behavior of unmodeled kernels, filesystems, controllers, device caches, or power-loss protection. Physical faults remain limited to the recorded hardware and method; none establishes universal durability.

Partial, torn, and truncated data are physical conditions, not interchangeable fault causes. Tests state whether each condition was injected directly or observed after another fault. Unsupported faults are recorded as not tested, never passed.

## 5. Pre-fault preparation and run record

Before arming a fault, persist the controller ledger outside the fault domain where practical. Every run records:

- repository commit SHA and experiment/document revision;
- D-mode and the complete platform durability contract, including the promised and excluded fault classes;
- workload and configuration identity, operation count, concurrency, queueing, and batching parameters;
- permanent event identity and request identity for every operation, while keeping their meanings distinct;
- assigned sequence number where available;
- highest lifecycle point observed per operation, including validation/rejection, construction, sequencing, submission, boundary completion, commit, and relevant visibility;
- acknowledgement result, provisional/canonical status, named interval, and observation time without selecting a timestamp representation or clock source;
- for D3, group identity and membership, join/cut evidence, shared synchronization outcome, per-member commit evidence, and acknowledgement result;
- fault class, mechanism label, injection point, trigger condition, seed, repetition number, and evidence that the trigger occurred as declared;
- environment and storage/cache assumptions required by the benchmark methodology, including OS/kernel, filesystem/mount, storage/device/cache path, power-loss protection knowledge, virtualization/container layers, and cache/preconditioning state; and
- controller/oracle location and its independence from the faulted storage.

The run records timestamp and clock limitations but this procedure selects neither representation nor source. Missing measurement-critical evidence makes the run invalid rather than inviting reconstruction from recovered data.

## 6. Recovery procedure

For every run, perform these steps in order:

1. Stop further mutation. Preserve a read-only copy or snapshot of the faulted storage image before repair, truncation, retry, or recovery mutation where practical. If preservation is impractical, document why and record every mutation the recovery path can make.
2. Retain the untouched controller ledger and fault-injection evidence separately. Do not copy oracle facts into system state.
3. Restart through the configuration's declared recovery path, with the same semantic and platform configuration unless the test explicitly studies compatibility.
4. Scan canonical history from its declared beginning or from a later validated checkpoint. A checkpoint must identify its exact canonical position and validate against canonical history; failure falls back only through a declared fail-closed path.
5. Validate each candidate record according to the configuration's still-to-be-selected physical framing and integrity policy. Semantically classify data as complete/terminal, partial, corrupt, duplicate, missing/gapped, out of order, or undecidable. This contract requires those outcomes but does not prescribe how bytes establish them.
6. Reconstruct the recovered canonical event set and its monotonically increasing order. Preserve permanent identity and all semantic envelope contents. Report sequence gaps without automatically calling them corruption while gap policy remains unresolved.
7. Compare the recovered set and order with every oracle class, including the uncertain commit-before-acknowledgement set. Account for every recovered identity and position; physical residue outside canonical history remains diagnostic only.
8. Replay into a fresh derived state at least twice from the same preserved input and configuration, and repeat the scan/classification. Compare classifications, canonical set/order, and replay result for determinism. Recovery must not overwrite history or add observation metadata to recovered events.
9. Record recovery duration and scan and replay throughput as separate measurements from correctness. Do not interpret performance from a correctness-failing configuration.

Recovery is fail-closed whenever canonical status or valid ordering cannot be established. It must never silently invent an event, accept corrupt/undecidable history, reorder or overwrite events, use a materialization as authority, or use a checkpoint as an alternative authority.

## 7. Correctness invariants

A valid run evaluates all applicable invariants:

1. Every canonically acknowledged D2/D3 event required by the declared fault contract recovers exactly once.
2. D0/D1 acknowledgement alone never proves canonical status or permits canonical recovery/visibility.
3. Rejected commands and merely provisional candidates do not recover as canonical facts.
4. A commit-before-acknowledgement interruption may recover exactly one canonical event although the caller observed an uncertain outcome; absence is permitted only when commit was not proven by the oracle.
5. Every recovered event retains its permanent identity and semantic envelope/payload contents unchanged.
6. Recovered ordering is deterministic and sequence positions are monotonically increasing.
7. Sequence gaps are reported, but are not automatically corruption until gap policy is resolved.
8. Duplicate permanent event identities and duplicate sequence positions are detected and never silently accepted.
9. Partial, corrupt, or undecidable history is explicitly classified and never silently accepted.
10. Repeated recovery yields the same classification, canonical set/order, and replay result.
11. Observation metadata remains external to the recovered event and cannot mutate it.
12. Checkpoints, if later used, remain derived, identify an exact canonical position, and validate against canonical history.
13. Explicit submission or synchronization failure never produces a successful canonical acknowledgement.
14. No recovered canonical event lacks a permitted oracle explanation, and no canonical reader observes an uncommitted candidate.

## 8. D3-specific oracle rules

D3 records both **per-event evidence** (request/event identity, sequence, join point, commit state, acknowledgement outcome) and **group evidence** (group identity, exact membership, cut trigger, shared persistence/synchronization invocation and outcome). The shared durability operation is not an atomic multi-event transaction guarantee.

- Shared synchronization success makes valid members eligible for individual canonical commit; it does not prove that all member commits or acknowledgement returns were observed before the fault.
- Shared synchronization failure produces no successful canonical acknowledgement for any member. Any physical prefix or member whose canonical status is uncertain is reported explicitly and is not promoted from group membership or physical presence alone.
- A member proven committed before interruption is **must recover** for a covered fault. A member whose commit might have occurred is **may recover: uncertain outcome**. Other joined or persisted members remain **must not be promoted** unless canonical status can be established under the declared recovery rules.
- Recovery assesses identities and positions individually while checking them against the recorded shared outcome. It neither rolls back a valid recovered prefix solely to simulate atomicity nor promotes a suffix to make a group appear whole.
- Retry and idempotency behavior after failed synchronization or an uncertain acknowledgement remains unresolved and must not be inferred from recovery results.

## 9. Result classifications

Each repetition receives exactly one top-level classification, plus detailed per-event and storage-condition classifications:

| Result | Criteria |
|---|---|
| **Pass** | The fault and lifecycle placement occurred as declared, required evidence is complete, and every applicable oracle obligation and invariant for the declared mode/platform fault contract is satisfied. |
| **Fail** | A promised event is lost; a rejected, provisional, corrupt, or otherwise invalid event is promoted; identity or semantic contents change; order changes; a duplicate is silently accepted; an error produces canonical success; recovery does not fail closed; or repeated classification/replay is nondeterministic. |
| **Invalid** | The declared fault or placement did not occur, required environment/oracle evidence is missing, the mechanism contaminated the boundary under test, or the run/configuration violated its predeclared procedure. Repeat only as a new run; do not count it as pass or fail. |
| **Inconclusive** | Evidence establishes what was attempted, but the platform or fault mechanism cannot demonstrate the claimed failure boundary—for example, a virtual reset cannot establish loss of a physical device cache. The limitation and narrower claims, if any, are reported. |

“Not tested” is reported for matrix cells outside the declared procedure and is not a pass. Any performance result from a configuration with a correctness **Fail** is **invalid for performance interpretation**, not a faster or slower valid result. Invalid or inconclusive correctness runs cannot support a durability claim.

## 10. Deliberately unresolved inputs

This procedure leaves open concrete event encoding and framing; minimum integrity policy and algorithms; event-identity algorithms; timestamp representation and clocks; platform-specific persistence/synchronization APIs; checkpoint format; retry/idempotency and uncertain-outcome handling; sequencing-gap policy; transactions; and distributed semantics. Those choices must be declared by later bounded work before applicable execution, without weakening this semantic oracle.
