# Architecture Decision Records

Architecture Decision Records capture decisions that have moved beyond speculation.

## When to create an ADR

Create an ADR when a decision materially constrains future architecture, interfaces, data formats, correctness semantics, or engineering process.

Foundational technical ADRs should cite experiment evidence whenever the decision is empirical.

## ADR status

- Proposed
- Accepted
- Rejected
- Superseded
- Deprecated

## Template

```markdown
# ADR-NNNN — Title

**Status:** Proposed
**Date:** YYYY-MM-DD

## Context

## Decision

## Evidence

## Alternatives considered

## Consequences

## Follow-up
```

## Rule

An ADR records why a choice was made under known evidence and constraints. It is not proof that the choice can never change.

## Registry

| ADR | Status | Scope |
|---|---|---|
| [ADR-0001](ADR-0001-evidence-driven-architecture.md) | Accepted | Evidence-driven governance process. |
| [ADR-0002](ADR-0002-foundational-canonical-history-constraints.md) | Accepted | Approved semantic research constraints; not empirical validation. |
