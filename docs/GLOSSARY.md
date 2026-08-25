# Glossary

These terms describe approved meanings or current research vocabulary; they do not assert implementation or performance evidence.

- **canonical event** — an immutable record of an accepted fact, distinct from the command requesting change.
- **canonical history** — the single authoritative ordered event history from which state and representations are derived.
- **command** — requested intent. Rejection is operational/audit evidence, not a canonical event asserting the requested fact.
- **request identity** — stable command identity for duplicate detection and idempotent retry; distinct from event identity and not a universal exactly-once guarantee.
- **event identity** — permanent event identity independent of sequence number and physical location.
- **information identity** — stable logical identity of information or an entity independent of schema and physical representation.
- **event/fact type** — the kind of accepted fact asserted by an event; it does not require the canonical core to interpret domain payload meaning.
- **schema identity / schema version** — the identity and applicable version of canonical schema information for an opaque payload; these values do not themselves define executable schema behavior.
- **envelope semantic version** — the version of the core-understood envelope contract, distinct from domain schema identity/version.
- **sequence number** — monotonically increasing position providing deterministic total order in the initial local log; it makes no distributed-order commitment.
- **effective time** — when a fact applies in the modeled domain.
- **system time** — when Data OS accepted a fact.
- **durability time** — when an event crossed its declared durability boundary.
- **observation time** — when a reader or materialization could observe an event; observation-side lifecycle metadata outside the immutable original envelope. Recording it cannot mutate the observed event, and making the observation canonical requires a separately appended event.
- **source/actor provenance** — the applicable origin or actor responsible for a fact, distinct from causation and request correlation.
- **causal-event reference** — a reference to a prior event that caused or directly motivated a fact, distinct from provenance and request correlation.
- **opaque payload boundary** — the boundary containing domain-specific fact content that the canonical core preserves but does not interpret.
- **integrity metadata** — conditional semantic metadata binding an event to the checks required by its declared integrity mode without preselecting a physical algorithm or framing; every claimed capability must remain explicit and measurable.
- **late-arriving fact** — a fact whose effective time precedes its system time.
- **correction / retraction** — a newly appended event referencing an affected event; canonical facts are not overwritten.
- **materialization** — a derived representation optimized for a workload; it is not authoritative history.
- **checkpoint** — a validatable rebuild/recovery optimization identifying the exact history position represented.
- **canonical commit** — acceptance only after the declared durability boundary; earlier memory visibility is provisional.
- **caller acknowledgement** — the result returned for a command attempt; a successful result must name its durability mode, lifecycle point, canonical/provisional status, and measured interval.
- **provisional visibility** — observation of an uncommitted candidate by a reader whose contract exposes that status and permits later disappearance; it is never canonical-reader visibility.
- **canonical-reader visibility** — availability to a reader of authoritative canonical history, never before canonical commit.
- **materializer freshness** — the committed canonical sequence or watermark incorporated and exposed by a derived representation; distinct from ingestion acknowledgement latency.
- **durability boundary** — the explicitly declared completion condition and platform contract that must be crossed before canonical commit.
- **durability group** — D3 events sharing one declared synchronization outcome; grouping is not an atomic multi-event transaction.
