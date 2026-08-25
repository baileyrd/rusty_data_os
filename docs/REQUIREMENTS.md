# Initial System Requirements Registry

These are approved research constraints for future experiments and implementations, not claims that a system currently satisfies them.

| ID | Requirement |
|---|---|
| REQ-001 | Canonical event history shall be the single authoritative source; memory state, checkpoints, indexes, and other representations shall be derived. |
| REQ-002 | Canonical events shall represent accepted facts; commands shall represent requested intent, and rejected commands shall remain separate audit/operational evidence. |
| REQ-003 | The initial local log shall assign monotonically increasing sequence numbers for deterministic total replay order without specifying future distributed ordering. |
| REQ-004 | Events shall distinguish effective, system, durability, and observation time from sequence/replay order and shall permit effective time before system time. |
| REQ-005 | Corrections and retractions shall append events referencing affected events; accepted facts shall not be overwritten. |
| REQ-006 | Events and logical information/entities shall have stable identities independent of log position, physical location, schema, and materialization. |
| REQ-007 | Schema definitions and changes shall be versioned canonical information. EXP-0001 may carry opaque payload bytes with schema identity/version but shall not execute schema semantics. |
| REQ-008 | The canonical core shall interpret only a minimal versioned envelope: ordering, temporal, identity, provenance, payload-boundary, and integrity rules—not domain payload meaning. |
| REQ-009 | Commands shall have stable request identities for duplicate detection and idempotent retry, distinct from event identities; no universal exactly-once delivery is required. |
| REQ-010 | Events shall preserve applicable source/actor, originating-request, and causal-event references. |
| REQ-011 | Ordinary compaction may reorganize, compress, or archive history but shall not silently discard meaning; destructive retention requires a future explicit auditable decision. |
| REQ-012 | A checkpoint shall identify the exact canonical-history position represented and be validatable against that history. |
| REQ-013 | Canonical commit shall occur only after the declared durability boundary; earlier visibility shall be provisional, and weaker modes shall not be labeled durable commit. |
| REQ-014 | EXP-0001 shall support only single-event commits; atomic multi-event batches remain a future capability. |
