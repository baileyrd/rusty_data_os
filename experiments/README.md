# Executable Experiments

This directory contains bounded experimental implementations that test hypotheses.

[Unified commitment](unified-commitment/README.md) implements owner-authorized
[EXP-0003](../docs/experiments/EXP-0003-unified-commitment.md): a bounded RF1 D1/D2
transaction core, Memory incarnation, retry, checkpoints and injected-fault recovery.
Platform durability evidence and manual descriptive measurements remain pending;
no OS-crash/power-loss claim or production graduation.

[EXP-0004](../docs/experiments/EXP-0004-entity-relation-domains.md), Ready, adds
[uc-entity](unified-commitment/crates/uc-entity/src/lib.rs) and
[uc-relation](unified-commitment/crates/uc-relation/src/lib.rs) to the same workspace.
Each domain has its own Log/directory, canonical payloads, incarnation and checkpoint/replay
scenario tests with independent models. Memory-to-Entity atomicity is
[Deferred](../docs/roadmap/ROADMAP.md); no measurement series or production graduation.

[EXP-0005](../docs/experiments/EXP-0005-protocol-facade.md), Ready, adds
[uc-protocol](unified-commitment/crates/uc-protocol/src/lib.rs): std-only protocol-22
codec/framing, Store/dispatch and generic stream sessions with shared registry cross-table
coordination. Its 66-fixture oracle and in-memory duplex tests are correctness evidence
only. No real domain adapter, live socket, authentication or benchmark series.

Experimental code is allowed to be narrow, disposable, duplicated, or deliberately non-general when doing so improves measurement clarity.

Code does **not** graduate into `/crates` merely because it works. Graduation requires sufficient evidence and an explicit architectural decision or specification when the change is foundational.

The existing `exp-0001` workspace provides bounded codec, conformance, ordinary-write D1 append/replay and exploratory harness implementations. The standalone [convergence-memory](convergence-memory/README.md) and [legacy](convergence-memory-legacy/README.md) workspaces implement [EXP-0002](../docs/experiments/EXP-0002-convergence-memory.md), authorized by the [merge plan](../docs/plans/data-os-multimodal-merge-plan-2026-09-08.md). Candidate: D1 ordinary writes, no fsync, no crash-survival claim. No production engine has graduated.
