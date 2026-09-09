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
