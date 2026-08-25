# Glossary

These terms describe approved meanings or current research vocabulary; they do not assert implementation or performance evidence.

- **canonical event** — an immutable record of an accepted fact, distinct from the command requesting change.
- **canonical history** — the single authoritative ordered event history from which state and representations are derived.
- **command** — requested intent. Rejection is operational/audit evidence, not a canonical event asserting the requested fact.
- **request identity** — stable command identity for duplicate detection and idempotent retry; distinct from event identity and not a universal exactly-once guarantee.
- **event identity** — permanent event identity independent of sequence number and physical location.
- **information identity** — stable logical identity of information or an entity independent of schema and physical representation.
- **sequence number** — monotonically increasing position providing deterministic total order in the initial local log; it makes no distributed-order commitment.
- **effective time** — when a fact applies in the modeled domain.
- **system time** — when Data OS accepted a fact.
- **durability time** — when an event crossed its declared durability boundary.
- **observation time** — when a reader or materialization could observe an event.
- **late-arriving fact** — a fact whose effective time precedes its system time.
- **correction / retraction** — a newly appended event referencing an affected event; canonical facts are not overwritten.
- **materialization** — a derived representation optimized for a workload; it is not authoritative history.
- **checkpoint** — a validatable rebuild/recovery optimization identifying the exact history position represented.
- **canonical commit** — acceptance only after the declared durability boundary; earlier memory visibility is provisional.
