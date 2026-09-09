# EXP-0004 — Entity and Relation domains

## 1. Identifier and title

EXP-0004: port Entity and Relation onto the unified core.

## 2. Status

Ready. The [merge plan, step 4](../plans/data-os-multimodal-merge-plan-2026-09-08.md)
first sentence and the approved Step 4a work order authorize this bounded implementation.
The supplied work order is identified by SHA-256
`554623ea9c42c4a2ea7f5f162c5cf75381dc14fe417ce9b94e8d7698096074f1`
and source name `handoff-2026-09-08/step4a-entity-relation-spec.md`.
All implementation paths resolve inside the active checkout. This method precedes implementation.

## 3. Linked hypothesis

[HYP-0004](../hypotheses/HYP-0004-entity-relation-domains.md).

## 4. Research question

Do Entity and Relation reproduce their existing single-table semantics on the unified
core with the same recovery/incarnation guarantees Memory has?

## 5. Hypothesis under test

The unchanged core generalizes to both domains. Any independent-oracle mismatch or
lost/duplicated commit under the existing injected-fault model falsifies the affected domain.

## 6. Independent variables

Domain, requested D1/D2, named deterministic scenario and checkpoint versus full replay.
Each domain uses its own Log and directory.

## 7. Controlled variables

Unchanged uc-core, uc-memory, uc-harness and existing tests; Rust 1.89; one writer;
fixed hand-written operations; stable explicit request identities; no runtime legacy dependency.

## 8. Workloads

Small deterministic test sequences only, approximately twenty operations per domain,
checked after every operation against independent BTreeMap models. No generated workload,
trace format, release binary, measurement series or harness change.

## 9. Correctness invariants

- Entity insert, replacement and mention_count update retain all other fields; incarnation
  must match for replacement/update/delete and increase on recreation. Duplicate live inserts fail.
- Entity links are symmetric, same-table and open-label: non-empty, at most 64 bytes,
  first byte not `-`, ASCII alphanumeric/underscore/hyphen only. Invalid labels, self-loops,
  missing endpoints and stale incarnations reject before append. Reversed duplicate links
  identify the same undirected edge; deleting an endpoint cascades every incident edge.
- Entity starts with `relates_to` and `mentioned_with`; successful links retain new labels
  permanently even when deletion removes every edge. Known-but-empty labels return an empty
  neighbor list; unknown labels return Malformed. Full replay and checkpoints retain this distinction.
- Relation subject/relation/object must be non-empty and deleted_at_unix_ms nonnegative,
  on both insert and replacement. All other signed timestamps and non-empty strings are
  unrestricted. Endpoints never require records elsewhere. Only updated_at_unix_ms has an update verb.
- Both domains reject predecessor updates after delete/reinsert, also after checkpoint/reopen.
  Entity predecessor links cannot attach to recreated records. No detach verb exists.
- Canonical CME1/CMR1 payloads round-trip; malformed, noncanonical, truncated, empty and
  oversized/count-exceeding transactions fail. Every invariant runs inside apply during replay.
- Rejected batches append no bytes and publish no partial state. Stable retries return the
  original outcome without duplicate append. Replacement supersedes older field updates.
- Checkpoint state bytes and SHA-256 digests equal full replay, including Entity known_labels.

Tests label deterministic validation/replay, simulated response loss or injected byte truncation
explicitly. Negative controls disable/rebind the relevant mechanism where applicable. Existing
domain-agnostic injected process-termination and corruption tests remain the core fault coverage.

## 10. Benchmark metrics

None. Correctness measurements are exact outcomes, history byte lengths, record/edge/label
equality and canonical state digests. No latency, throughput or performance claim.

## 11. Environment requirements

Run the work order's locked, offline Windows GNU Rust 1.89 proof from the checkout root:
fmt, Clippy with warnings denied, and all-target tests for unified-commitment and
convergence-memory; the same for exp-0001 excluding exp1-descriptive-d1-harness; Markdown
links and git diff --check. The exclusion preserves the existing Linux/x86_64 compile gate.
These tests supply no OS-crash, power-loss or platform durability evidence.

## 12. Baselines

The supplied source inspection pins rusty_multimodal_db at `232b16e`: generic/server Entity,
generic/server Relation, generic store label handling and ADR-0058. Independent test models
are written from those operation semantics, without calling either domain's apply.
Existing Memory tests establish the adapter coverage pattern, not the new domains' oracle.
Entity-to-Relation linking does not exist in the source system and is not invented here.
Memory-to-Entity cross-domain atomicity is a named deferred follow-on requiring a shared-state
or cross-log design; neither is attempted here.

## 13. Predeclared interpretation criteria

Every invariant must pass at both requested durability levels where applicable. A mismatch
invalidates that scenario; retain failures and limitations. Passing proves only bounded
implementation/correctness on these fixtures, without architecture promotion.

## 14. Implementation notes

Two lib crates depend only on uc-core. Each State retains incarnation tombstones and immutable
Arc records. Entity additionally retains canonical undirected labeled edges and known_labels.
CME1/CMR1 use one operation per line, lowercase hex for strings/UUID bytes and canonical decimal
numbers, with 1–1024 operations at Step 4a (raised to 4096 by Step 4b-ii D7/R0 in
[EXP-0005](EXP-0005-protocol-facade.md)) and uc-core's MAX_PAYLOAD byte limit. CES1/CRS1 checkpoint blobs
encode sorted slots and, for Entity, labels and edges; decoding verifies exact re-encoding.
Plain apply/decode function pointers reconstruct all semantics without a recovery validator.
The core's RF1-magic rejection and framing apply automatically. Checkpoint plus full history
replay supplies the legacy compact mapping; no separate compact operation is introduced.
See [ADR-0004](../adr/ADR-0004-entity-relation-domains.md).

## 15. Raw result locations

The [implementation report](EXP-0004/IMPLEMENTATION-REPORT.md) and
[full proof output](EXP-0004/proof-output.txt) retain correctness validation and file accounting.
No benchmark results index entries or measurement artifacts.

## 16. Results

The agreed local proof passed with exit code 0: 59 unified-commitment tests (30 existing,
13 Entity, 16 Relation), 17 convergence-memory tests and 95 portable exp-0001 tests.
All three workspaces passed formatting and Clippy; Markdown links and git diff --check passed.
Both handwritten 22-operation models agreed at every step for D1 and D2. See the
[implementation report](EXP-0004/IMPLEMENTATION-REPORT.md) for scope and test corrections.

## 17. Conclusion

The tested fixtures establish bounded implementation/correctness evidence for both independent
domains on the unchanged core. Hypothesis Open and experiment Ready. No performance, platform
durability or production conclusion.

## 18. Follow-on questions

MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY is Deferred in the
[follow-on roadmap](../roadmap/ROADMAP.md). The legacy Memory mentions relationship targets
Entity; faithful atomic commitment across these domains requires a shared State or cross-Log
coordination, touching Step 3's Memory adapter and requiring a separate bounded design.
Entity-to-Relation linking does not exist in the source system and is not invented here.
Compatibility facade, protocol/server, SQL/query layers, other domains and measurement generators
remain outside this work order.
