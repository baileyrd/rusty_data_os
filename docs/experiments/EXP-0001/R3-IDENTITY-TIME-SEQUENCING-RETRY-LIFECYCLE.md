# EXP-0001 R3 Identity, Time, Sequencing, and Retry Lifecycle Contract

**Status:** Complete requirements/research record; no implementation or execution authorized

**Scope:** EXP-0001 single-event ingestion lifecycle; resolves BLK-004, BLK-005, BLK-011, and BLK-012 and constrains BLK-007

**Evidence classification:** reviewed governance and experiment-local requirements, not implementation, correctness, durability, or benchmark evidence

## 1. Decision boundary and authority

This record applies the approved canonical-history constraints, the EXP-0000 semantic envelope, acknowledgement/visibility/durability contract, crash/recovery contract, workload contract, and the R1/R2 requirements. It chooses lifecycle semantics needed to remove ambiguity before Slice A/C. It does not select physical framing, synchronization APIs, a target platform, benchmark thresholds, or executable generators.

Normative terms **MUST**, **MUST NOT**, **SHOULD**, and **MAY** express EXP-0001 requirements. Canonical history remains the only authority. Request journals, attempt records, reservation watermarks, acknowledgements, and benchmark observations are lifecycle evidence; they may support reconciliation but cannot create or overwrite a canonical fact.

## 2. Invariants

1. Request, attempt, permanent-event, information, causal/reference, and sequence identities are different typed domains. Equality across domains has no meaning and implementations MUST reject implicit substitution.
2. One accepted request produces at most one canonical event in EXP-0001. A retry never constructs a second event after the request has been bound to an event identity.
3. Event identity is permanent and independent of request identity, information identity, sequence, time, and physical location.
4. Information identity denotes the modeled subject, not request bytes, event bytes, sequence, or a content digest. Multiple events may reference the same information identity.
5. Only effective time and system-acceptance time enter the immutable event envelope. Sequence enters the envelope as replay order. Durability, acknowledgement, attempt, and observation timing remain noncanonical lifecycle/measurement evidence.
6. Wall-clock values MUST NOT establish causal order, replay order, duplicate precedence, or commit order. Assigned sequence alone establishes local canonical replay order.
7. A sequence is never reused. A reserved but uncommitted sequence is a permanent gap, not a canonical event.
8. D0 and D1 outcomes are provisional and noncanonical. D2/D3 become canonical exactly once only after the applicable declared platform durability boundary completes successfully and the commit marker/state is recoverably established under the later platform contract.
9. Acknowledgement loss cannot reverse a canonical commit. Recovery may discover a committed event without inventing a delivered acknowledgement.
10. Ambiguous identity binding, invalid time, duplicate/out-of-order canonical position, conflicting request reuse, or inability to prove canonical status MUST fail closed.

## 3. Identity contract (BLK-004)

### 3.1 Common representation and profile

Request, attempt, event, and information identities use separate nominal types containing the 16 octets of an RFC 9562 UUID. Text at documentation or manifest boundaries is the canonical lowercase hyphenated form; comparison and storage use the 16 octets. The nil UUID, non-RFC variant, and versions other than 4 are invalid for these four types in EXP-0001.

Live assignment uses UUID version 4: set the RFC variant/version bits over 122 bits obtained from an operating-system cryptographically secure random generator. No uniqueness claim is empirical: the design assumes independent uniform random bits and treats collision as an explicit error. Generator unavailability, short output, or health/error indication is terminal for that construction attempt; there is no clock-, counter-, request-, or sequence-derived fallback.

The deterministic workload generator is a distinct, future BLK-007 mechanism. It MUST emit type-valid UUIDv4-shaped values from domain-separated deterministic inputs without claiming randomness. R3 fixes the type, validation, semantic stability, and collision behavior but deliberately does not select that generator algorithm or vectors.

Causal and correction/retraction references contain an `EventId`, not a new identifier algorithm. Sequence is an unsigned 64-bit integer in its own type and is never accepted where any UUID-typed identity is required.

### 3.2 Identity authority and lifecycle

| Identity | Assignment authority and capture point | Input/domain | Stable across | Canonical content | Duplicate, conflict, or generator failure |
|---|---|---|---|---|---|
| `RequestId` | Caller assigns before first submission; ingestion validates at receipt | UUIDv4; request-correlation domain | Every retry of byte-for-byte and semantically identical intent | Yes, as correlation only; it is not fact identity | Same ID/same normalized request reconciles; same ID/different normalized request is terminal conflict. Invalid ID is rejection. Caller generator failure means no valid request exists. |
| `AttemptId` | Attempt boundary assigns immediately after basic request-ID parsing and before validation | UUIDv4; lifecycle-attempt domain | Never; every submission/retry gets a new value | No | Collision with any recorded attempt fails that attempt; generator failure rejects before mutation. |
| `EventId` | Event constructor assigns once after semantic validation succeeds and before sequence reservation | UUIDv4; permanent-event domain | All later persistence attempts, retries, recovery, and references for the bound request | Yes | Collision with different canonical/candidate event fails closed; equal ID with identical request binding reconciles only, never appends. Generator failure leaves the request validated but unconstructed/retryable. |
| `InformationId` | Domain/caller authority supplies before validation; ingestion validates but does not derive it | UUIDv4; modeled-information domain | All facts about the same declared subject | Yes | Reuse is expected for the same subject. Domain-level conflicting meaning is rejected when detectable; R3 grants no global semantic registry. Missing/invalid is rejection. |
| Causal/correction reference | Event constructor copies a supplied prior `EventId` after reference validation | Event-ID domain | Immutable with constructed event | Yes, when applicable | Unknown, forward, self, wrong-type, or disallowed reference fails validation; exact locality mechanism remains UNK-013. |
| `Sequence` | Single local sequence authority durably reserves after event construction and before persistence submission | Unsigned 64-bit local-authority epoch | Permanent for the candidate/event; never reused | Yes for committed event | Exhaustion is terminal/fail-closed. Reservation uncertainty requires recovery/reconciliation before more assignment. |

“Same normalized request” means equality of the complete versioned semantic request inputs that can affect validation or event construction, excluding attempt identity and observation timing. Its future exact serialization/digest is BLK-007/008/009 work; until frozen, an implementation cannot claim automated same-request reconciliation.

### 3.3 Canonical versus lifecycle evidence

Canonical event content includes request ID, event ID, information ID, applicable causal/correction event references, assigned sequence, effective time, and system-acceptance time. Attempt ID, generator diagnostics, validation observations, reservation records/watermarks, persistence attempts, durability completion, acknowledgement delivery, lookup history, and benchmark timestamps remain noncanonical. A later canonical audit fact would require an ordinary separately accepted event; lifecycle evidence never mutates the original.

## 4. Time contract (BLK-005)

### 4.1 Representation and normalization

Canonical effective and system-acceptance times are signed 64-bit counts of nanoseconds from the Unix epoch on the UTC timescale, with no leap-second label. Inputs with offsets MUST be converted to this integer; a textual leap second, fractional precision finer than one nanosecond, an unrepresentable value, or a value requiring rounding is rejected. Equal timestamps are valid and have no ordering implication. The valid integer range is the exact `i64` range; arithmetic MUST be checked and overflow fails closed.

Canonical precision is one nanosecond, but recorded resolution is a separate required descriptor of the source. Values MUST NOT imply accuracy or resolution finer than their source. EXP-0001 does not rewrite, clamp, or monotonize wall time.

Lifecycle durations and measurement instants use unsigned 64-bit nanoseconds since the run's monotonic-clock origin. They are comparable only within that run and clock domain. Subtraction is checked; overflow or a monotonic-clock regression invalidates affected timing evidence.

### 4.2 Time authority and capture

| Temporal value | Representation / clock class | Authority and capture point | Canonical? | Failure/anomaly rule |
|---|---|---|---|---|
| Effective/domain time | `i64` Unix-epoch ns UTC; caller/domain source plus declared source resolution | Supplied with intent and frozen when semantic validation accepts it, before event construction | Yes | Missing when required, invalid normalization, or out-of-range rejects. May precede/equal/follow system time. |
| System-acceptance time | `i64` Unix-epoch ns UTC from OS realtime wall clock; source resolution recorded | Ingestion authority samples once after semantic validation and immediately before event-ID assignment/construction | Yes | Unavailable/out-of-range rejects construction. Rollback/jump is recorded as lifecycle anomaly but value is retained if valid; it never changes ordering. Retry reuses the bound value after construction. |
| Replay/order sequence | `u64`, not a clock | Sequence authority reserves after construction | Yes | See section 5; never inferred from any timestamp. |
| Durability-boundary timing | `u64` monotonic ns since run origin, capture immediately before submission and when declared boundary returns | Persistence observer under future platform contract | No | Missing/regressing/overflowed samples invalidate latency evidence, not an otherwise provable commit. Boundary error controls lifecycle outcome. |
| Acknowledgement timing | `u64` monotonic ns since run origin, capture before delivery attempt and after delivery result where observable | Caller-interface observer | No | Delivery interruption yields uncertain outcome when commit already may have occurred. Timing failure invalidates measurement only. |
| Observation/measurement time | `u64` monotonic ns since the observing run origin; optional wall-clock annotation is nonauthoritative | Named observer captures at the actual observation | No | Clock-domain/run ID and resolution required; cross-run/platform ordering is forbidden. |

Across platforms and runs, canonical epoch integers retain the defined UTC interpretation, while accuracy and resolution are properties of recorded source metadata. Monotonic values require the run/clock-domain identity and cannot be translated into canonical wall time. Clock rollback, forward jump, equality, or coarse resolution MUST be surfaced; none permits reordering or rejecting an otherwise valid prior canonical event during replay.

## 5. Sequencing and gaps (BLK-011)

### 5.1 Assignment order

The required order is: parse request identity → assign attempt ID → validate request and references → capture system-acceptance time → assign/bind event ID and construct the immutable candidate → durably reserve exactly one sequence → submit/persist → complete declared durability boundary → establish canonical commit → attempt acknowledgement. Validation or construction failure consumes no sequence. Reservation occurs before any event bytes are submitted.

The reservation authority MUST recover a high-water mark that is at least every sequence it ever exposed or reserved. Advancing that watermark and associating the reservation with the event candidate must be recoverable under the later physical/platform contract. If this cannot be proven after a fault, assignment stops pending reconciliation. Concrete storage APIs remain R4/R5 work.

### 5.2 Gap decision table

| Condition | Sequence treatment | Representation/reporting | Replay/recovery action |
|---|---|---|---|
| Rejection or pre-construction failure | None assigned | Attempt evidence only | No gap expected. |
| Construction failure before reservation | None assigned | Request/attempt outcome | No gap expected. |
| Failure after durable reservation but before commit | Reserved number abandoned permanently | Noncanonical reservation/failure evidence; adjacent canonical events reveal the numeric gap | Accept gap, report range/cause if evidence exists; never synthesize an event. |
| Process crash after reservation | Reservation remains consumed even if candidate absent | Recovered allocator watermark plus any lifecycle evidence | Continue above watermark; missing candidate is a legal gap. |
| Canonical event with next position greater than prior + 1 | Legal gap | Replay reports gap range; cause may be unknown | Continue deterministic replay in increasing recorded order. Unknown cause is not itself corruption. |
| Duplicate position, decreasing position, zero/invalid position, or position above proven allocator watermark | Invalid | Corruption/conflict diagnostic | Fail closed; do not choose a winner, renumber, or skip. |
| Same event ID at a different position or different event IDs at one position | Invalid identity/order conflict | Diagnostic evidence | Fail closed. |
| Candidate arrives out of reservation order | Provisional only until append policy can preserve increasing canonical order | Lifecycle evidence | MUST NOT commit out of sequence; queue, reject, or abandon per later mechanism. |

Position 0 is reserved as “unassigned”; first assignable position is 1. Exhaustion at `u64::MAX` permanently closes assignment. Gaps never become canonical facts and are not filled or reused. Canonical replay sorts/reads by recorded increasing sequence and validates strict increase; it does not require contiguity.

## 6. Request/event lifecycle (BLK-012)

### 6.1 Normative state machine

| State | Event / guard | Next state | Required outcome |
|---|---|---|---|
| Received | Invalid request ID or malformed intent | Rejected | Conclusive terminal rejection; no event/sequence. |
| Received | Valid ID; assign fresh attempt ID | Validating | Record noncanonical attempt evidence. |
| Validating | Semantic/reference validation fails | Rejected | Conclusive terminal for those exact inputs; same unchanged request is not retryable absent changed authority/state. |
| Validating | Existing request ID with unequal normalized request | Conflict | Conclusive terminal conflict; no mutation. |
| Validating | Existing binding/commit with equal normalized request | Reconciling | Return known result or explicit lookup-needed/uncertain result; never reconstruct another event. |
| Validating | Success and no binding | Constructing | Capture system time, assign event ID, atomically bind request to candidate. |
| Constructing | Clock/generator/construction failure before binding | Retryable failure | No sequence; same request may retry with new attempt ID. |
| Constructing | Binding succeeds | Reserving | Bound event ID and canonical inputs are thereafter stable. |
| Reserving | Reservation succeeds | Submitted | Candidate has permanent sequence; submit exactly that candidate. |
| Reserving | Failure conclusively before reservation | Retryable failure | Bound event reused; no known sequence. |
| Reserving | Reservation outcome uncertain | Reconciling | Stop assignment; lookup/recovery required before retry. |
| Submitted | D0/D1 completion | Provisional | Conclusive provisional result only; never canonical acknowledgement. Later stronger attempt reuses candidate. |
| Submitted | Persistence or sync conclusively fails before commit | Retryable failure | Preserve binding and reservation; retry exact event/sequence only if later mechanism proves safe, otherwise reconcile. |
| Submitted | Persistence/sync outcome uncertain | Reconciling | No success/failure claim; recover/lookup canonical status before resubmission. |
| Submitted | Declared D2/D3 boundary and commit establishment succeed | Committed | Canonical exactly once; immutable thereafter. |
| Committed | Acknowledgement delivered | Acknowledged | Conclusive success names D-mode, canonical status, event ID, and sequence. |
| Committed | Delivery lost, times out, or caller interrupted | Reconciling | Caller outcome uncertain; canonical event remains committed. |
| Reconciling | Lookup finds matching commit | Acknowledged or committed/ack-unknown | Return the original event ID/sequence and commit status; never append. |
| Reconciling | Lookup proves bound uncommitted candidate safe to resubmit | Submitted | New attempt ID; exact bound event ID, sequence, times, and bytes reused. |
| Reconciling | Lookup proves no binding/reservation/commit | Validating | New attempt ID; original request semantics retained. |
| Reconciling | Status cannot be proven | Reconciling | Terminal operational impasse requiring explicit recovery; fail closed. |
| Recovery | Valid committed record | Committed | Rebuild canonical order; delivered acknowledgement is not inferred. |
| Recovery | Provisional/invalid/ambiguous record | Provisional, abandoned, or failed closed | Apply R1 scanning rules; never promote it. |

### 6.2 Outcome contract

- **Same-request retry:** same request ID and identical normalized inputs, fresh attempt ID. It returns the known result or resumes only the exact bound candidate. Once bound, event ID, information ID, references, effective/system times, sequence (if reserved), and canonical bytes never change.
- **Conflicting reuse:** same request ID with any normalized semantic difference is a conclusive terminal conflict, regardless of whether the first request committed.
- **Duplicate accepted fact:** matching request binding or matching event ID reconciles to the one event. A different request/event that happens to state similar payload about the same information is not automatically a duplicate; no content-based deduplication is authorized.
- **Unknown request:** lookup with no durable binding returns `unknown`, not `rejected` or `not committed`. After recovery proves absence and allocator safety, a retry may begin validation; mere cache absence is insufficient.
- **Commit before acknowledgement:** result is uncertain to the caller and terminally canonical to the engine. Retry/lookup returns the original success; it cannot undo or duplicate the event.
- **Retryability:** only a conclusive pre-commit failure with proven reservation/submission safety is directly retryable. Any possible commit or reservation uncertainty requires lookup/recovery first.

No exactly-once transport claim is made. The contract provides at-most-one canonical event per durable request binding when its physical realization later satisfies these requirements.

## 7. Failure-scenario review

| Scenario | Required classification and behavior |
|---|---|
| Validation rejection | Conclusive rejected; attempt evidence only; no event or sequence. |
| Pre-sequence construction failure | Conclusive retryable if no binding ambiguity; new attempt, stable request, and bound event reused if binding already completed. |
| Post-sequence/pre-commit failure | Permanent gap if abandoned; exact candidate may be retried only after safety proof; never reuse sequence for another event. |
| Persistence error | Conclusive pre-commit error is retryable only with safe resubmission proof; ambiguous partial write requires reconciliation and R1 recovery. |
| Sync error | D2/D3 commit is not claimed; because completion may be uncertain, reconcile before retry. D0/D1 data remains provisional. |
| Canonical commit, lost acknowledgement | Canonical event remains; caller gets uncertain outcome and must lookup/retry same request to recover original result. |
| Same-request retry | Fresh attempt ID; compare normalized request; return/resume original binding only. |
| Conflicting request-ID reuse | Terminal conflict; no new candidate, sequence, or overwrite. |
| Duplicate event identity | Identical binding reconciles; any differing event content/request/position fails closed. |
| Clock anomaly | Record anomaly; valid wall value remains content. Clock cannot control order. Unavailable/invalid acceptance clock prevents construction. Measurement-clock failure invalidates timing evidence only. |
| Crash/recovery | R1 scans valid records, restores committed events in strict sequence order and reservation high-water mark, reports gaps, leaves D0/D1/noncommitted candidates noncanonical, and never fabricates acknowledgements. Ambiguity fails closed. |

## 8. Fail-closed rules

The implementation MUST stop the affected request, replay, or assignment domain rather than guess when: typed identity validation fails; a collision has unequal binding/content; normalized-request equality cannot be established; an event reference is invalid under the selected locality rule; canonical time cannot be represented; allocator watermark/reservation status is uncertain; sequence decreases or duplicates; event identity and sequence disagree; commit status may have changed but cannot be reconciled; physical validity/commit boundary is unprovable; or recovery would require inventing, renumbering, filling, overwriting, or reordering an event.

## 9. Alternatives, rationale, and evidence classification

| Topic | Selected policy | Alternatives considered and reason not selected now |
|---|---|---|
| Identity shape | Typed UUIDv4, OS CSPRNG live profile | UUIDv7/ULID couple identity to fallible wall time and invite ordering inference; sequence-derived IDs collapse domains; content hashes collapse event/information identity and require unfrozen serialization/digest; central counters couple identity to locality. |
| Information identity | Caller/domain-assigned typed UUID, not derived | Payload digest cannot represent stable information across corrections; request ID identifies intent, not subject. |
| Canonical time | UTC Unix-epoch signed ns plus source-resolution metadata | Text timestamps permit multiple encodings; unsigned time excludes pre-epoch values; monotonic time has no cross-run domain meaning; clock-clamping hides anomalies. |
| Ordering | Durable `u64` reservation, gaps legal, no reuse | Gapless assignment at commit complicates concurrent physical ordering and hides abandoned work; timestamp order is nondeterministic; gap records would elevate lifecycle failures into canonical facts. |
| Retry | Durable request-to-event binding plus reconciliation | Blind retry can duplicate facts; treating timeout as failure loses commit-before-ack uncertainty; content dedupe conflates similar facts. |

These are deductive design choices required to preserve approved semantics, not measured evidence. UUID collision probability, clock quality, allocator cost, reconciliation cost, and performance remain unmeasured. The contract makes no security claim beyond requiring a CSPRNG for live UUID assignment and no authenticity claim for UUIDs.

## 10. Assumptions, risks, unresolved questions, and revisit conditions

Assumptions: a later physical design can durably bind request/event/reservation state without becoming canonical authority; a single local sequence authority is sufficient for EXP-0001; callers can supply stable request and information IDs; the platform exposes realtime and monotonic clocks plus source resolution.

Risks include UUID collision or entropy failure, clock anomalies, unbounded lifecycle evidence, reservation gaps under repeated faults, allocator contention, inability of a baseline to expose adequate reconciliation, and a physical design that cannot atomically/recoverably preserve binding and reservation invariants.

Still unresolved: exact normalized-request serialization and equality mechanism; deterministic UUID-shaped generator and vectors (BLK-007); causal-reference locality (UNK-013); lifecycle evidence retention and lookup representation; physical reservation/binding/commit mechanism; platform clock API and verified resolution; target durability contract; all R4–R9 decisions.

Revisit this record if REQ-003/004/006/009/013 changes, multi-event commits or multiple sequencing authorities enter scope, a selected baseline cannot preserve binding/reservation semantics, UUIDv4 validation is incompatible with stable vector requirements, nanosecond epoch representation is not portable to the selected platform, or fault validation disproves recoverability. Revision requires synchronized blocker/unknown/traceability updates and must preserve historical rationale.

## 11. Blocker resolution

| Blocker | Outcome | Why |
|---|---|---|
| BLK-004 | **Resolved by R3** | Exact typed UUIDv4 live profiles, authorities, capture points, stability, validation, collision/conflict and generator-failure behavior are fixed; identity domains cannot collapse. |
| BLK-005 | **Resolved by R3** | Exact canonical/lifecycle representations, normalization, clock classes, authorities, capture points, anomaly/range/equality and cross-run rules are fixed. Platform API verification remains R4, not semantic ambiguity. |
| BLK-011 | **Resolved by R3** | Reservation timing, permanent legal gaps, no reuse, reporting, replay validation, duplicate/out-of-order handling, exhaustion, and fail-closed recovery are fixed. |
| BLK-012 | **Resolved by R3** | The state machine classifies rejection through recovery, including retry, conflict, uncertainty, duplicate, lookup, D-mode, commit, and acknowledgement behavior. |
| BLK-007 | **Open; additionally constrained by R3** | The future generator must produce domain-separated, typed UUIDv4-shaped identities and exact lifecycle/time/reference inputs consistent with retry stability and canonical/noncanonical boundaries. Algorithm, serialization, digest, and stable vectors remain deliberately open. |

R3 closes no implementation gate by itself. R4 remains next; R9 alone may authorize one implementation slice after every prerequisite is resolved.
