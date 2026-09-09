# Executable Experiments

This directory contains bounded experimental implementations that test hypotheses.

Experimental code is allowed to be narrow, disposable, duplicated, or deliberately non-general when doing so improves measurement clarity.

Code does **not** graduate into `/crates` merely because it works. Graduation requires sufficient evidence and an explicit architectural decision or specification when the change is foundational.

The existing `exp-0001` workspace provides bounded codec, conformance, ordinary-write D1 append/replay and exploratory harness implementations. The standalone [convergence-memory](convergence-memory/README.md) and [legacy](convergence-memory-legacy/README.md) workspaces implement [EXP-0002](../docs/experiments/EXP-0002-convergence-memory.md), authorized by the [merge plan](../docs/plans/data-os-multimodal-merge-plan-2026-09-08.md). Candidate: D1 ordinary writes, no fsync, no crash-survival claim. No production engine has graduated.
