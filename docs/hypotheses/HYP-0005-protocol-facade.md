# HYP-0005 — Independent protocol facade

Status: Open.

An independent std-only protocol facade can preserve all 66 protocol-22 literal wire
values and their exact bytes, plus the specified domain-agnostic session and cross-table
behaviors. Any fixture mismatch, partial precondition apply, escaped session guard or
dangling edge under the controlled shared-registry interleaving falsifies the affected
behavior. [EXP-0005](../experiments/EXP-0005-protocol-facade.md) predeclares the method.
Passing local correctness tests does not establish domain integration, production readiness,
authentication, durability or performance; independent review remains pending.
