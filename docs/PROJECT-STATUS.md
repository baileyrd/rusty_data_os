# Project Status

**Project:** Rusty Data OS
**Status:** Phase 0 — Foundation and Measurement Design
**North star:** Represent once. Materialize many. Optimize always.
**Verified starting `main` checkpoint:** `05b77516b9155aaa419093c7fa860e37d5517ecc`

## 1. Current facts

This repository is documentation-only. No engine implementation, Cargo baseline, CI configuration, or benchmark evidence exists. No event encoding, identity algorithm, timestamp representation, clock source, concurrency model, checkpoint format, transaction model, query language, or distributed design has been selected.

The conceptual architecture is a research direction, not a benchmark-validated design.

## 2. Approved foundation

The primary unproven research claim is that a single canonical information history can support multiple independently optimized representations with acceptable performance and complexity.

The approved semantic constraints are recorded in [ADR-0002](adr/ADR-0002-foundational-canonical-history-constraints.md) and [REQ-001 through REQ-014](REQUIREMENTS.md). In summary:

- canonical events are accepted facts, while commands are requested intent and rejected commands remain separate evidence;
- canonical history alone is authoritative; memory, checkpoints, indexes, and materializations are derived;
- local monotonic sequence provides deterministic replay without committing future distributed ordering;
- temporal, permanent identity, provenance, correction/retraction, schema-version, payload-boundary, compaction, checkpoint, and durability semantics are explicit;
- EXP-0001 is restricted to single-event commit and opaque payloads with schema identity/version.

These are constraints on research and correctness, not evidence that the architecture performs acceptably.

## 3. Active hypothesis

[HYP-0001](hypotheses/HYP-0001-event-log-as-canonical-state.md) asks whether one canonical information history can support multiple independently optimized representations with acceptable performance and complexity. It is active and unproven. No implementation or experimental result supports or refutes it yet.

## 4. Active and next incomplete increments

[EXP-0000 — Measurement and Semantics Readiness](experiments/EXP-0000-measurement-and-semantics-readiness.md), also called Experiment 0, is in progress. Its [minimal single-event semantic envelope](experiments/EXP-0000/SEMANTIC-EVENT-ENVELOPE.md), [reproducible workload contract](experiments/EXP-0000/WORKLOADS.md), [EXP-0001 baseline contract](benchmarks/BASELINES.md), [acknowledgement, visibility, fault, and durability contract](experiments/EXP-0000/ACKNOWLEDGEMENT-VISIBILITY-DURABILITY.md), and [crash/recovery correctness contract](experiments/EXP-0000/CRASH-RECOVERY-CORRECTNESS.md) are complete. The [environment](benchmarks/ENVIRONMENT-TEMPLATE.md) and [raw-result](benchmarks/RAW-RESULT-TEMPLATE.md) record contracts are also complete. Predeclared interpretation criteria remain incomplete. These are documentation and measurement-readiness outputs, not implementation or evidence.

The baseline checkpoint selects B0 in-memory (D0 only), B1 raw OS append (primary D1/D2/controlled D3), SQLite WAL, and RocksDB WAL. It freezes semantic profiles, equivalence classifications, adapter fairness, series identity, correctness gates, and exclusion/replacement policy without selecting binaries, implementations, schemas, encodings, platform sync mechanisms, or claiming evidence. SQLite/RocksDB D2 remains conditional; atomic multi-event transactions and opaque group commit are not strict D3 equivalence. Predeclared interpretation criteria are the recommended next bounded output; that recommendation neither configures a product nor authorizes implementation or execution.

[EXP-0001 — Immutable Event Ingestion](experiments/EXP-0001-immutable-event-ingestion.md) remains proposed and planned, but is blocked by completion of Experiment 0 and its documented prerequisites. EXP-0001 must not begin during this increment.

## 5. Decision policy

Foundational empirical claims follow:

```text
Hypothesis -> Experiment -> Evidence -> ADR -> Specification -> Core code
```

Governance and approved research constraints may be decided before empirical validation when their evidence classification is explicit. They must not be presented as proven performance claims.

## 6. Continuity and navigation

Read [AGENTS.md](../AGENTS.md) and [CHATGPT_WORKFLOW.md](../CHATGPT_WORKFLOW.md) first, then the authorities in the order they prescribe. Supporting registries are the [glossary](GLOSSARY.md), [assumptions and unknowns](ASSUMPTIONS-AND-UNKNOWNS.md), [research questions](RESEARCH-QUESTIONS.md), [requirements](REQUIREMENTS.md), and [traceability registry](TRACEABILITY.md).

The latest `main` branch is authoritative over conversation memory. The checkpoint above records the verified repository starting point for this continuity increment; it is not experiment evidence.
