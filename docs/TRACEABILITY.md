# Traceability Registry

This registry links intent to planned validation without inventing evidence.

| Source | Traces to | Status |
|---|---|---|
| Vision: represent once, materialize many | P-001, P-002, P-003, P-006, P-010; REQ-001; RQ-001 | Research direction |
| Vision: evidence-driven success | P-011, P-014, P-015; ADR-0001 | Accepted governance |
| REQ-001 through REQ-014 | ADR-0002; HYP-0001 where empirical feasibility is involved | Approved constraints; feasibility unproven |
| HYP-0001 | RQ-001; EXP-0000; EXP-0001; later representation experiments | Active / unproven |
| EXP-0000 | RQ-002; EXP-0001 prerequisites; [single-event semantic envelope](experiments/EXP-0000/SEMANTIC-EVENT-ENVELOPE.md) | Running; semantic-envelope output complete, all other outputs incomplete |
| EXP-0000 semantic envelope fields and observation-side lifecycle metadata | REQ-002 through REQ-010, REQ-013, REQ-014; ADR-0002 | Approved-semantics elaboration; the original envelope is immutable after commit, provenance and integrity are conditional where applicable, and physical choices and listed semantic questions remain unresolved |
| EXP-0001 | RQ-003; HYP-0001 (partial evidence only) | Proposed; blocked by EXP-0000 |
| ADR-0001 | Research lifecycle and benchmark methodology | Accepted governance decision |
| ADR-0002 | REQ-001 through REQ-014 and terminology | Accepted research constraints; no empirical validation claim |

No listed link represents completed experiment evidence.
