# ADR-0002 — Foundational Canonical-History Research Constraints

**Status:** Accepted
**Date:** 2026-08-25

## Context

The research program needs stable semantic constraints so experiments study the same proposition. These constraints define what is being researched; they do not prove that an event-centered architecture has acceptable performance or complexity.

## Decision

Adopt `REQ-001` through `REQ-014` in the [initial requirements registry](../REQUIREMENTS.md) as research and correctness constraints. In particular, canonical events are accepted facts rather than commands; canonical history alone is authoritative; identity, provenance, temporal meanings, correction/retraction, checkpoints, compaction, and durability boundaries remain explicit.

One accepted command may eventually create an atomic multi-event batch, but EXP-0001 is restricted to single-event commits. The initial local sequence is total and deterministic without constraining a future distributed order. Schema is canonical, versioned information, while EXP-0001 payload interpretation is deferred.

## Evidence classification

This is an approved scope and semantic decision that keeps research coherent. Like ADR-0001's governance decision, it is not an empirical architecture validation. HYP-0001 remains active and unproven; no implementation, benchmark result, or complexity measurement supports it yet.

## Consequences

- Experiments must preserve these semantics or explicitly propose a superseding decision.
- Weaker acknowledgement modes may be measured but cannot be reported as durable canonical commit.
- Concrete encoding, identity algorithm, timestamp/clock choice, schema execution, multi-event transaction mechanics, and distributed ordering remain unresolved.
- Experiment 0 must make EXP-0001 measurement-ready before implementation begins.
