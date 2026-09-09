# ADR-0004 — Faithful independent Entity and Relation domains

**Status:** Proposed
**Date:** 2026-09-09

## Context and authority

The [merge plan step 4](../plans/data-os-multimodal-merge-plan-2026-09-08.md), first sentence,
and approved Step 4a work order identified in [EXP-0004 §2](../experiments/EXP-0004-entity-relation-domains.md#2-status)
authorize these experimental adapters. The supplied source facts pin rusty_multimodal_db
at `232b16e`; this decision preserves them rather than graduating a production architecture.

## Decision

uc-entity and uc-relation each own one Log/directory and depend only on uc-core.
Entity preserves same-table symmetric open-label links, seeded/retained known labels and
mention_count-only updates. Relation is an independent record domain with arbitrary non-empty
endpoint strings and no linking; ADR-0058 states that "nothing checks that an endpoint names
an entity". No Entity-to-Relation cross-reference is invented. Both retain incarnation
tombstones and replay-time validation. Whole-record replacement is the only way to change
fields lacking an update verb. Checkpoint plus full replay supplies compact semantics.

## Evidence and consequences

[EXP-0004](../experiments/EXP-0004-entity-relation-domains.md) predeclares independent oracles,
canonical payload/checkpoint checks and the bounded failure model. Existing core tests cover
domain-independent injected faults. No core, Memory, harness, CI or production changes and
no measurement series are authorized here. Hypothesis remains Open.

## Deferred follow-on

MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY requires a shared-state or cross-log design for the
legacy Memory mentions link to Entity. It is explicitly Deferred in the
[roadmap](../roadmap/ROADMAP.md) and is not established by two separate logs.
