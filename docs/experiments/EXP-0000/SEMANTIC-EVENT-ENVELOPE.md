# EXP-0000 Semantic Event Envelope Contract

**Scope:** minimal conceptual envelope for one canonical event  
**Status:** frozen for Experiment 0 measurement preparation  
**Evidence classification:** approved-semantics elaboration; not an implementation specification or experimental result

## 1. Purpose and authority

This contract gives later Experiment 0 work one unambiguous conceptual event to use when defining workloads, durability, recovery, and measurements. It elaborates the constraints approved by [REQ-001 through REQ-014](../../REQUIREMENTS.md) and [ADR-0002](../../adr/ADR-0002-foundational-canonical-history-constraints.md). It neither selects a physical representation nor claims that an implementation satisfies the contract.

The envelope describes exactly one accepted fact. Commands and rejected requests are not canonical events. Atomic metadata or semantics spanning multiple events are deferred by REQ-014.

Cardinality terms in the table mean:

- **required** — exactly one value belongs to every canonical event;
- **optional** — zero or one value may be present without another stated predicate;
- **conditional** — the stated fact relationship requires one or more values; otherwise none is required;
- **engine-assigned** — the engine supplies exactly one value during the stated lifecycle stage rather than accepting it as caller authority.

“Immutable” means immutable after canonical commit. Before commit, a candidate envelope may be abandoned; it is not canonical history.

## 2. Field contract

| Conceptual field | Semantic purpose | Cardinality / owner | Authority | First known | Immutable? | Semantic participation | Unresolved physical choices |
|---|---|---|---|---|---|---|---|
| Permanent event identity | Identifies this accepted fact permanently, independent of replay position or storage location. It is not a request identity. | Required; assignment authority is unresolved. | REQ-006; ADR-0002 | No later than fact/event construction; exact assignment point is unresolved. | Yes | replay correlation; correction; provenance; integrity association | ID algorithm, width, representation, generation authority, and collision handling |
| Local sequence / replay position | Establishes the monotonically increasing deterministic total replay order in the initial local log. It is not event identity or a distributed-order commitment. | Engine-assigned; exactly one | REQ-003; ADR-0002 | Sequenced | Yes | replay | Integer/other representation, width, allocation mechanism, first value, gap policy, and rollover behavior |
| Logical information/entity identity | Identifies the stable modeled information or entity affected by the fact, independent of event identity, schema, or representation. | Required; supplied or derived during construction under unresolved rules | REQ-006; ADR-0002 | Fact/event constructed | Yes | replay; correction interpretation; provenance; temporal interpretation | ID algorithm, cardinal domain, derivation/validation rules, and representation |
| Event/fact type | States what kind of fact the event asserts without requiring the core to interpret its domain meaning. | Required; constructed from validated intent | REQ-002; REQ-008; ADR-0002 | Fact/event constructed | Yes | replay; correction interpretation; provenance | Type namespace, identifier representation, registry, and evolution rules |
| Schema identity | Names the schema information under which the opaque payload was produced. It does not select executable schema behavior. | Required | REQ-007; REQ-008; ADR-0002 | Fact/event constructed | Yes | replay; provenance; integrity of interpretation | Identity algorithm, namespace, representation, registry, and resolution mechanism |
| Schema version | Identifies the applicable version of the named schema information. It does not make the core execute that schema. | Required | REQ-007; REQ-008; ADR-0002 | Fact/event constructed | Yes | replay; provenance; integrity of interpretation | Version representation, ordering rules, compatibility rules, and resolution mechanism |
| Effective time | States when the fact applies in the modeled domain; it may precede system acceptance time. | Required; provided or derived under domain validation | REQ-004; ADR-0002 | No later than fact/event construction | Yes | replay interpretation; correction; temporal interpretation | Timestamp representation, precision, range, timezone treatment, clock/source authority, and uncertainty expression |
| System acceptance time | States when Data OS accepted the fact. It is distinct from durability and observation time. | Engine-assigned; exactly one | REQ-004; ADR-0002 | During acceptance processing; whether before or after sequencing is unresolved | Yes | replay interpretation; provenance; temporal interpretation | Capture point, timestamp representation/precision, clock source, monotonicity, and clock-adjustment policy |
| Durability time | States when the event crossed its declared durability boundary; it is not earlier provisional visibility. | Engine-assigned; exactly one for a canonically committed event | REQ-004; REQ-013; ADR-0002 | Declared durability boundary crossed | Yes | replay audit; temporal interpretation; integrity of commit claims | Boundary definition (a later Experiment 0 output), capture point, timestamp representation/precision, clock source, and failure semantics |
| Observation time | States when a particular reader or materialization could observe the event; it is not system acceptance or durability time. | Conditional; one or more observations may exist when an observer records them, and the values are observation context rather than a single intrinsic creation value | REQ-004; REQ-013; ADR-0002 | Observable by the applicable reader/materializer | Yes for a recorded observation; absent before that observation | provenance; temporal interpretation | Whether stored in or alongside history, observer identity, multiplicity encoding, capture point, timestamp representation, and clock source |
| Originating request identity | Correlates the event to the accepted command for duplicate detection and idempotent retry without becoming the event identity or promising universal exactly-once delivery. | Required for the command-originated events in current scope | REQ-009; REQ-010; ADR-0002 | Command received | Yes | provenance; request correlation | ID algorithm, representation, generation authority, retention, and duplicate-detection window |
| Source/actor provenance | Identifies the applicable origin and/or actor responsible for the accepted fact. It is distinct from request correlation and causation. | Required semantic provenance; one or more values as applicable | REQ-008; REQ-010; ADR-0002 | Command received or validation; finalized by fact/event construction | Yes | provenance; integrity/audit interpretation | Provenance vocabulary, number and nesting of actors/sources, identity format, authentication binding, and encoding |
| Causal-event reference(s) | References prior event identities that caused or directly motivated this fact; it is not request correlation or general provenance. | Conditional; one or more when an event has represented causal predecessors, otherwise none | REQ-010; ADR-0002 | Fact/event constructed | Yes | replay interpretation; correction context; provenance; causation | Single/list/graph encoding, ordering, locality validation, missing-reference policy, and cross-history rules |
| Correction/retraction target | References the permanent identity of each affected canonical event so correction or retraction appends history rather than overwriting it. | Conditional; one or more for correction/retraction event types, otherwise none | REQ-005; REQ-006; ADR-0002 | Fact/event constructed | Yes | replay; correction; provenance | Single/list encoding, target validation, cross-history rules, and semantics of multiple targets |
| Opaque payload boundary | Contains the domain-specific fact content while marking the boundary beyond which the canonical core does not interpret meaning. | Required; may be zero-length if a future declared event type permits it | REQ-007; REQ-008; ADR-0002 | Fact/event constructed | Yes | replay; correction interpretation; integrity coverage | Binary encoding, serialization library, compression, size limits, canonicalization, and storage framing |
| Integrity metadata | Semantically binds an event to the integrity checks declared for its envelope/payload so corruption or truncation can be detected within a documented capability. It does not promise an algorithm not yet selected. | Required at the semantic level; engine-produced value(s) are finalized when the protected representation is finalized | REQ-008; ADR-0002 | No later than persistence attempt; exact finalization point depends on unresolved integrity coverage/framing | Yes | replay validation; integrity | Checksum/hash/MAC algorithm, coverage, placement, keying, granularity, representation, and torn-record strategy |
| Envelope semantic version | Identifies the semantic contract used to interpret envelope fields, independently of schema identity/version and domain payload behavior. | Required | REQ-008; ADR-0002 | Fact/event constructed | Yes | replay; provenance; integrity of interpretation | Version representation, compatibility policy, upgrade path, and encoding |

No row selects a binary layout, serialization, storage framing, ID algorithm, timestamp representation, clock source, checksum algorithm, or executable schema system. Those remain design inputs to later bounded work.

## 3. Required distinctions and invariants

1. **Identity is not position or correlation.** Event identity, local sequence, logical information/entity identity, and originating request identity have different meanings and cannot substitute for one another.
2. **Replay order is explicit.** Sequence establishes deterministic local replay order. Effective time does not reorder canonical history, and the local sequence makes no future distributed-order decision.
3. **Time meanings remain separate.** Effective time may precede system acceptance time. System acceptance, durability, and observation time describe different lifecycle facts and are not interchangeable.
4. **Commit follows declared durability.** An event becomes canonically committed only after its declared durability boundary is crossed. Any earlier memory visibility is provisional and cannot be labeled durable canonical commit.
5. **History is append-only in meaning.** Corrections and retractions are new events that reference affected permanent event identities; they never overwrite an affected event.
6. **Payload meaning stays outside the core.** The core understands envelope semantics and an opaque payload boundary, not domain-specific payload meaning. Schema identity and version identify canonical schema information but do not define executable schema behavior.
7. **Relationships stay distinct.** Provenance says where/from whom a fact came, causation points to causal events, and request identity correlates the originating command. These concepts must not collapse into one ambiguous identifier.
8. **Scope is one event.** This contract supplies no transaction, batch, or atomic multi-event identifier. Such metadata remains deferred.

## 4. Conceptual lifecycle

The lifecycle describes semantic availability, not a required pipeline implementation:

```text
command received
-> validated
-> fact/event constructed
-> sequenced
-> persistence attempted
-> declared durability boundary crossed
-> canonical commit
-> observable by readers/materializers
```

| Stage | Newly available or resolved information | Status of the candidate fact |
|---|---|---|
| Command received | Originating request identity and initial source/actor provenance are available. | Requested intent only; not a canonical event. |
| Validated | Acceptance eligibility and validated provenance/input facts are known. | Still not a canonical event; rejection remains separate operational/audit evidence. |
| Fact/event constructed | Logical information/entity identity, event/fact type, schema identity/version, effective time, opaque payload, envelope semantic version, applicable causal references and correction/retraction targets, and finalized provenance are known. Permanent event identity is known no later than this stage, though its exact assignment authority/timing remains unresolved. | Candidate accepted fact; not yet canonically committed. |
| Sequenced | Deterministic local sequence/replay position is assigned. | Ordered candidate; still not canonically committed. |
| Persistence attempted | The selected persistence path is attempted; integrity metadata is available no later than this stage, subject to unresolved coverage/framing. | Failure may abandon the candidate; earlier memory visibility remains provisional. |
| Declared durability boundary crossed | Durability time becomes known under the separately declared durability contract. | Eligible for canonical commit. |
| Canonical commit | The envelope and payload become immutable canonical history. | Authoritative accepted fact. |
| Observable by readers/materializers | Observation time becomes known for each recorded observer/context. | Committed fact is visible; materialization remains derived. |

The authorities require system acceptance time but do **not** decide whether its capture occurs before or after sequencing, or whether a single clock can validly compare it with durability and observation timestamps. That ambiguity is measurement-critical: later acknowledgement/visibility/fault/durability work must fix the capture points, clocks, and comparison rules before EXP-0001 can be ready. This contract does not silently decide them.

## 5. Deferred questions

- Who assigns permanent event identity, and precisely when?
- Is observation time canonical envelope data, observation-side metadata, or both, and how are multiple observers represented?
- What exact capture point defines system acceptance time relative to sequencing?
- Which clock sources and comparison/uncertainty rules apply to system, durability, and observation time?
- What declared durability boundary makes durability time and canonical commit valid for each measured mode?
- What integrity capability and semantic coverage are required before selecting an algorithm or framing?
- What validation applies to causal and correction/retraction references, including missing or cross-history targets?
- What namespaces, registries, and compatibility policies govern event type, schema identity/version, and envelope semantic version?

These questions do not reopen the approved distinctions. They identify inputs that later Experiment 0 outputs must resolve before implementation or comparable measurement.
