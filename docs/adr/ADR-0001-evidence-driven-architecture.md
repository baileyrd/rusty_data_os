# ADR-0001 — Evidence-Driven Architecture Process

**Status:** Accepted  
**Date:** 2026-08-25

## Context

Rusty Data OS is exploring foundational database concepts where attractive ideas can easily become large implementation commitments before their real performance, durability, recovery, and complexity costs are known.

The project explicitly intends to avoid "boiling the ocean" only to discover that the premise was wrong.

## Decision

Major architectural claims will be treated as hypotheses until tested.

The default progression is:

```text
Idea -> Hypothesis -> Experiment -> Measurement -> Evidence -> Decision -> Architecture
```

For durable core architecture, decisions should normally be recorded through an ADR and, when stable interface/behavior is required, a specification.

Negative results will be preserved rather than deleted.

## Evidence

This ADR defines engineering governance rather than an empirical engine behavior. Its evidence is the project requirement that decisions be fact-based and data-driven rather than based on feelings or assumptions.

## Alternatives considered

### Architecture-first implementation

Define the intended complete database architecture and implement toward it.

Rejected because early assumptions would become expensive to reverse and benchmark results could be confounded by unrelated system complexity.

### Prototype without formal hypotheses

Implement promising ideas quickly and benchmark afterward.

Rejected as the default because post-hoc interpretation makes it easier to move success criteria and harder to distinguish what a benchmark actually tested.

## Consequences

Positive:

- assumptions remain visible;
- experiments can be small;
- negative findings become reusable knowledge;
- architectural promotion has traceable rationale;
- benchmark methodology is treated as a first-class artifact.

Costs:

- documentation overhead;
- slower initial feature accumulation;
- some experiments will intentionally produce code that never reaches production.

These costs are accepted.
