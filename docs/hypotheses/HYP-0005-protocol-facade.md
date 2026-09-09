# HYP-0005 — Independent protocol facade

Status: Open.

An independent std-only protocol facade can preserve all 66 protocol-22 literal wire
values and their exact bytes, plus the specified domain-agnostic session and cross-table
behaviors. Any fixture mismatch, partial precondition apply, escaped session guard or
dangling edge under the controlled shared-registry interleaving falsifies the affected
behavior. [EXP-0005](../experiments/EXP-0005-protocol-facade.md) predeclares the method.
Step 4b-ii adds a bounded integration check: each real domain's same-table operations,
separate all-flags sessions and Entity's open-label links must agree over a loopback TCP
socket, with atomic transaction/refusal and reopen checks. Its separately frozen work
order and method are recorded in EXP-0005 §19. Local correctness evidence establishes only
that tested scope; cross-table mentions, cross-domain sessions, production readiness,
authentication, durability and performance are not established. Independent review remains
pending; status stays Open.

Inspection 1's accepted F1 found missing wildcard neighbor descriptors on Memory/Entity.
The corrective tests assert complete same-table unlabeled Join rows over real TCP, failing
before removal of the overrides and passing with the inherited descriptors. This closes
that local regression only; independent re-review remains pending.
