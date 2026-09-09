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

Step 4b-ii was separately authorized by the owner's frozen work order identified in
[EXP-0005 §19](../experiments/EXP-0005-protocol-facade.md#19-step-4b-ii-method-2026-09-09).
It adds uc-facade's three independent table mutexes, raw-byte ID mapping, internal current/
next incarnations, guarded whole-record replacement using the shared predicate evaluator,
and one engine transact per session commit. The listener only binds 127.0.0.1:0 and passes
accepted sockets through a buffered Read + Write wrapper to the unchanged connection loop.
An explicit stop flag ends acceptance; callers close clients and all workers are joined.
Nonempty atomic WriteBatch retains the Unsupported default. Memory's mentions descriptor
has target_table None; all links here are same-table.

Inspection 1 F1 corrects an implementation omission: Memory and Entity inherit Store's
default describe_relations, preserving both the wildcard Neighbors(None) descriptor and
named labels, all with target_table None. The former custom overrides rejected working
unlabeled Joins as Malformed; two real-TCP regressions fail before this deletion and pass
afterward. This restores the existing contract without changing uc-protocol.

D7/R0's bounded exceptions add Send to core writer/hook trait objects, raise all three
domain operation caps to 4096, and accumulate Sum/Avg in i128. Out-of-range Sum returns
Malformed; Avg casts before division and keeps the empty 0.0 identity. This closes a
4b-i residual, with no wire-shape or fixture change. The
[Step 4b-ii report](../experiments/EXP-0005/STEP4BII-IMPLEMENTATION-REPORT.md) records local
correctness evidence; this ADR remains Proposed and authorizes no production graduation.
Authentication/authorization, actual legacy-client integration, cross-domain atomic
commitment and performance evidence remain outside this increment.
