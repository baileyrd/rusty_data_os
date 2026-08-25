# Specifications

Specifications define behavior that has become sufficiently understood and stable to constrain implementation.

This directory intentionally begins without engine specifications.

**Registry status:** no specifications exist. Approved research constraints are recorded in [ADR-0002](../adr/ADR-0002-foundational-canonical-history-constraints.md) and the [requirements registry](../REQUIREMENTS.md), not promoted to an implementation specification.

## Promotion rule

Do not write a specification merely to make an exploratory idea look official.

A foundational specification should normally be supported by:

- one or more hypotheses;
- completed experiments or other credible evidence;
- an accepted architecture decision where appropriate;
- known correctness and compatibility requirements.

## Candidate future specifications

Possible future areas include:

- canonical event envelope;
- event ordering semantics;
- durability classes;
- segment/log format;
- checkpoint format;
- replay semantics;
- materializer contract;
- schema evolution;
- embedded engine API;
- server protocol.

These are placeholders for research topics, not commitments.
