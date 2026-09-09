# HYP-0003 — Unified commitment and recovery

Status: Open.

A single RF1 recoverable-commit history with per-transaction R5 sync recovers every
merge-plan scenario under injected process termination and torn tails and serves
Memory traces at a declared D2 level. Any invalid promotion, lost required commit,
retry mismatch or oracle divergence falsifies the affected cell regardless of speed.
See [EXP-0003](../experiments/EXP-0003-unified-commitment.md) for the predeclared method.
The platform durability contract remains evidence-pending; no OS-crash or power-loss claim.
