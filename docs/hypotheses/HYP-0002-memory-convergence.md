# HYP-0002 — Memory convergence

Status: Open; correctness validation does not establish performance.

For the fixed CMT1 traces, candidate operations and independently replayed row and
column states match the per-field model and pinned legacy Memory adapter. Any
per-operation outcome, query, or final-field mismatch falsifies this claim for
that cell. See [EXP-0002](../experiments/EXP-0002-convergence-memory.md) for the
predeclared measurement and interpretation method.

Candidate: D1 ordinary writes, no fsync, no crash-survival claim.
