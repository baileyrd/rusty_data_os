# Assumptions and Unknowns

## Research assumptions

| ID | Assumption | Validation path |
|---|---|---|
| ASM-001 | Canonical history can be replayed into derived state. | Experiment 0 defines semantics; later recovery experiments measure it. |
| ASM-002 | Independent materializations may provide useful workload specialization. | HYP-0001 and later materialization experiments. |
| ASM-003 | Explicit durability boundaries can be implemented and compared fairly. | Experiment 0 definitions, then EXP-0001. |

These assumptions are neither benchmark evidence nor accepted empirical conclusions.

## Open unknowns

| ID | Unknown |
|---|---|
| UNK-001 | Concrete event binary encoding and record framing. |
| UNK-002 | Event-identity algorithm, such as UUID or ULID. |
| UNK-003 | Timestamp representation and clock source. |
| UNK-004 | Executable schema semantics and evolution mechanism. |
| UNK-005 | Concurrency, batching, and multi-event transaction design. |
| UNK-006 | Checkpoint format and validation mechanism. |
| UNK-007 | Query, materialization, retention, replication, and distributed designs. |
| UNK-008 | Quantitative thresholds for acceptable performance and complexity across selected workloads. |
| UNK-009 | Permanent event-identity assignment authority and exact lifecycle point. |
| UNK-010 | Exact system-acceptance-time capture point relative to sequencing. |
| UNK-011 | Observation-side metadata retention, observer/context identity and multiplicity rules, and criteria for appending an observation as a separate canonical event. |
| UNK-012 | Minimum required integrity policy, supported integrity modes, semantic coverage, finalization point, and physical algorithm/framing. |
| UNK-013 | Validation and locality rules for causal-event and correction/retraction references. |
| UNK-014 | Platform-specific durability contracts and the empirical survival behavior of each synchronization primitive across OS, filesystem, mount, device, and cache configurations. |
| UNK-015 | Crash/fault-injection procedure and deterministic recovery treatment for partial, torn, truncated, or uncertain physical outcomes. |
| UNK-016 | Retry and uncertain-outcome policy after persistence or D3 group synchronization errors. |
| UNK-017 | Sequencing gap policy for candidates that fail before canonical commit. |
