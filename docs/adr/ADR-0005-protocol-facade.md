# ADR-0005 — Bounded protocol-22 facade infrastructure

**Status:** Proposed
**Date:** 2026-09-09

## Context and authority

The [merge plan step 4](../plans/data-os-multimodal-merge-plan-2026-09-08.md), paragraph 2,
and owner-approved frozen Step 4b-i work order identified in [EXP-0005 §2](../experiments/EXP-0005-protocol-facade.md#2-status)
authorize the R0 exception to AGENTS §3 and Roadmap Phase 7. Phase 1 and every other gate
remain unchanged. Legacy fact source: rusty_multimodal_db at
`232b16ecb3fd89b318ae4730c8470ab6da52f330`, read-only.

## Decision

Implement uc-protocol only under experiments/unified-commitment. Handwrite fixed-width
little-endian payload encoding, separate bounded framing, full wire types, Store and generic
dispatch. RecordId owns 16 bytes with Ord; uc-core::Uuid lacks Ord and is unchanged.
The crate has no dependency beyond std. Registry holds immutable registration order,
a primary table and one optional relationship mutex shared by all its connections/clones.
Link/Delete/WriteBatch hold it from before foreign checks through apply and detaches;
other requests retain adapter-only locking. Atomic Store defaults refuse nonempty batches
before writing. Implementers must validate transaction read sets and writes atomically.
The serving loop is generic Read + Write, using no socket API.

## Evidence and consequences

[EXP-0005](../experiments/EXP-0005-protocol-facade.md) fixes exact 66-line host-literal and
byte-round-trip checks and deterministic duplex tests. The two-connection interleaving
includes a test-only negative control that removes the mutex and reproduces a dangling
edge. [Local proof](../experiments/EXP-0005/IMPLEMENTATION-REPORT.md) is bounded correctness
evidence, advisory pending independent review. This Proposed ADR does not graduate code.

The scope deliberately answers Authenticate with Ok and performs no older-version content
rewriting. Hello still negotiates min(client,22), and BeginWith bits honor their explicit
version gates. No real domain implements Store. Delete detach failures can report Storage
after the record and earlier detaches are already applied; no rollback is invented.

## Deferred follow-on

Step 4b-ii must separately authorize real Memory/Entity/Relation wiring and any socket
wrapper. Authentication/authorization, older-client compatibility, cross-domain atomic
commitment and performance evidence remain outside this increment.
