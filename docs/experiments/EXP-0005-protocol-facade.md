# EXP-0005 — Protocol-22 facade infrastructure

## 1. Identifier and title

EXP-0005: Step 4b-i domain-agnostic protocol-22 facade.

## 2. Status

Ready; local implementation/correctness proof complete, independent review pending. The [merge plan step 4](../plans/data-os-multimodal-merge-plan-2026-09-08.md), paragraph 2, and owner-approved frozen work order authorize this bounded exception to Phase 7. Work order source name: `handoff-2026-09-08/step4b-protocol-facade-spec.md`; SHA-256: `66a53b9f66b49e287adeb6bd14c35507fd0ef7607c048141da285f4f37457d80`.
All implementation paths resolve in the active checkout. This method precedes implementation.

## 3. Linked hypothesis

[HYP-0005](../hypotheses/HYP-0005-protocol-facade.md).

## 4. Research question

Can an independent std-only facade preserve the frozen protocol-22 wire values and domain-agnostic connection behavior?

## 5. Hypothesis under test

Every fixture must match both the independently decoded literal value and the original bytes. A mismatch, partially applied precondition failure, escaped session guard, or dangling edge under the controlled two-connection interleaving falsifies the affected behavior.

## 6. Independent variables

Request variant, session flags, negotiated version, registered table set, atomic/pipelined batch mode and deterministic connection interleaving.

## 7. Controlled variables

Rust 1.89 Windows GNU; unchanged existing crates; no external dependencies; pinned source facts at rusty_multimodal_db `232b16ecb3fd89b318ae4730c8470ab6da52f330`.

## 8. Workloads

Literal fixture decoding and small deterministic in-memory duplex scenarios, plus the exact 4096/4097 operation and tracked-read boundaries. No generated workload or benchmark execution.

## 9. Correctness invariants

All 66 fixture values and byte round trips agree. Decode rejects unknown tags, invalid lengths, truncation and trailing bytes. Frames are capped at 16 MiB. Transactions validate before apply; sessions preserve staging, tracked committed reads and all eleven guards. Foreign Link checks, Delete detaches and WriteBatch spans hold one registry-wide mutex across connections. Join preserves the exact Malformed/Unsupported distinctions. Atomic batches default to refusal without writes.

## 10. Benchmark metrics

None; exact values, responses, state and synchronization ordering only.

## 11. Environment requirements

Run the frozen Windows GNU Rust 1.89 proof: fmt, locked/offline Clippy with warnings denied and all-target tests for unified-commitment, convergence-memory and portable exp-0001 (excluding exp1-descriptive-d1-harness); Markdown links and git diff --check. No live sockets or host probes.

## 12. Baselines

The 66-line frozen wire corpus and host's independently derived expected-values file. Test Stores are handwritten in this crate's tests only and commit under their own mutex. Legacy source is read-only reference material, with only the explicitly authorized fixtures copied.

## 13. Predeclared interpretation criteria

All checks must pass. Passing establishes bounded protocol correctness only; independent review remains required. It does not establish domain integration, durability, production readiness or performance.

## 14. Implementation notes

uc-protocol lives inside the existing unified-commitment workspace; a byte-array RecordId supplies Ord missing from uc-core::Uuid. Handwritten codec, generic Read + Write framing/loop, Store and immutable Registry. No authentication enforcement or version content rewriting; the explicit BeginWith flag version gates remain. No domain implements Store. See [ADR-0005](../adr/ADR-0005-protocol-facade.md).

## 15. Raw result locations

[Implementation report and file accounting](EXP-0005/IMPLEMENTATION-REPORT.md); [full proof output](EXP-0005/proof-output.txt).

## 16. Results

The agreed eleven-command proof exited 0: 97 unified-commitment tests (38 new protocol),
17 convergence-memory tests and 95 portable exp-0001 tests passed, with formatting,
locked/offline Clippy, Markdown links and whitespace checks. All 66 fixture lines passed
both complete host-literal equality and byte-exact round trip. The deterministic
shared_registry_two_connection_interleaving_prevents_dangling_edge test passed; its
mutex-removal negative control reproduces the dangling edge. This is bounded correctness
evidence only. See the report for the explicitly disclosed R7 Delete/Rollback wording
interpretation and development-check corrections.

## 17. Conclusion

Hypothesis Open, experiment Ready, Phase 1 unchanged. No architectural graduation.

## 18. Follow-on questions

Step 4b-ii domain integration and eventual socket binding require a separate work order. Authentication, older-version content rewriting and cross-domain atomic commitment remain excluded.

## 19. Step 4b-ii method (2026-09-09)

The owner's frozen `handoff-2026-09-08/step4bii-domain-adapters-spec.md`, SHA-256
`995f2d6ea29af8547985673e71efd90cdf08f90e844db876f9a49f75d53c5d29`, separately
authorizes uc-facade's three Mutex-protected domain Stores and a listener bound only to
127.0.0.1:0. The four D7/R0 exceptions are Send trait-object bounds, Memory's named 4096
operation cap, Entity/Relation's 4096 caps, and wide protocol Sum/Avg accumulation.
Sum outside i64 returns Malformed; Avg divides after casting both wide sum and count to f64.

Before implementation, the measurement method is exact response/state assertions using
the existing codec/framing over real TCP, plus direct adapter validation, atomic read-set
conflict/refusal, 4096-operation commit and reopen checks. Each domain gets a separate
all-flags session; Entity's open-label links stay same-table. Nonempty atomic WriteBatch
must refuse with TransactionFailed(0, Unsupported) without changes. Existing R0 suites
run before integration, followed by the owner's eleven-command locked/offline proof.
Any response mismatch, partial precondition apply, incorrect replay or lost session update
falsifies the affected behavior. These are D1 correctness tests, not measurements or a
crash-survival claim. Memory declares target_table None: no cross-table mentions and no
cross-domain session are attempted. Authentication and all other research gates remain
unchanged. Local results are advisory pending independent review.

## 20. Step 4b-ii local result

All eleven new facade tests pass, exercising real engine Stores over TCP plus direct
correctness and reopen assertions. The final eleven-command proof exited 0: 109 unified,
17 convergence-memory and 95 portable exp-0001 tests, all formatting/Clippy checks,
Markdown links and whitespace. See the [Step 4b-ii report](EXP-0005/STEP4BII-IMPLEMENTATION-REPORT.md)
for proof output and the proposed interpretation of ReplaceIf's "any field" wording:
the frozen shared validator rejects StrList predicates, including Eq/Ne guards on Memory
tags and Entity aliases. That boundary is preserved and explicitly tested; expanding it
requires separate authorization. No cross-table mentions, cross-domain session, legacy
client, performance or production claim follows. Hypothesis Open, experiment Ready,
independent review pending.

## 21. Inspection 1 F1 correction

The host accepted F1-DESCRIBE-RELATIONS-WILDCARD-DROPPED against inspection snapshot
`78e7839e…8d3fdb6`. Memory and Entity's custom descriptors omitted Neighbors(None), so
unlabeled same-table Join returned Malformed despite supported neighbors traversal.
The correction deletes both overrides and inherits the unchanged Store default, which
includes the wildcard and named descriptors with target_table None throughout.

The predeclared regression compares complete JoinedRows over real TCP: four inserted
records, two symmetric links, exact four directed result rows, and no unlinked record.
Entity uses two distinct labels to verify wildcard traversal across labels. One test per
domain fails before deletion and passes afterward. Proof and file accounting are in the
[Step 4b-ii report](EXP-0005/STEP4BII-IMPLEMENTATION-REPORT.md#inspection-1-f1-correction).
The corrective eleven-command proof exited 0 with 111 unified-commitment, 17
convergence-memory and 95 portable exp-0001 tests passing (223 total), plus formatting,
warnings-denied Clippy, Markdown links and whitespace checks.
No other finding is reopened and no cross-table scenario is attempted. Re-review remains
pending; this adds no architecture or production promotion.

## 22. Step 4c frozen method and local implementation

Owner work order: `handoff-2026-09-08/step4c-foreign-edges-spec.md`, SHA-256
`af142bbfa0f53071e3f187b356a9807e2a68c0dc330b5f2e1cd8f84a36891794`.
The attached order supersedes Step 4b-ii's same-table Memory mentions deferral and
Memory-only wildcard correction. Before implementation, its fourteen proof cases define
exact real-socket response/state assertions, isolated Memory checkpoint/full replay,
malformed/version/operation-limit rejection, deliberate same-id reincarnation and double
collisions, and a channel-controlled two-connection Join/Delete interleaving. Any wrong
pair, replay disagreement, stale resurrection or unlocked Join falsifies the affected claim.

Memory now records outgoing LinkForeign/DetachForeign in CMM3/CMS3, with both endpoint
incarnations. Replay verifies only local membership. A direct Entity Store handle supplies
live incarnation capture/filtering; local Memory membership resolves ambiguous raw IDs.
Only foreign_edges contributes to mentions. Delete cascades locally or via the existing
registry detach. Memory advertises only named mentions targeting entity. Atomic cross-table
commitment remains unsupported: a crash between Entity delete and Memory detach can leave
stored stale tuples, hidden by freshness checks and removed by a later detach.

Narrow protocol exceptions are Store::incarnation, EntityStore's override, and adding Join
to the existing relationship-lock list. No query evaluator, Entity engine, Relation engine,
core, harness, listener, authentication or dependency changes. CMM2/CMS2 migration is absent.
See the [advisory Step 4c report](EXP-0005/STEP4C-IMPLEMENTATION-REPORT.md) for proof,
file accounting, formatting deviation and environment limitations. Hypothesis Open,
experiment Ready; independent review pending, with no architectural graduation.

The Step 4c eleven-check proof passed: 119 unified-commitment, 17 convergence-memory,
95 portable EXP-0001 tests; formatting, warnings-denied Clippy, links and whitespace
all succeeded. The exact agreed workspace test records exit 0. This is local bounded
correctness evidence only, pending independent review.
