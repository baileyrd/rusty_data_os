# Step 4c advisory implementation report

Frozen plan SHA-256: `af142bbfa0f53071e3f187b356a9807e2a68c0dc330b5f2e1cd8f84a36891794`.
The source file hash matched the attachment. Base HEAD is
`3d794a5efa4e2af94e817b951777b3ccdd77c148`. All edits are in this checkout;
the original checkout was read only for the explicitly supplied plan and shared Git objects.
No commit, push, publication, dependency addition or benchmark series was performed.

## Implementation

CMM3/CMS3 adds LinkForeign/DetachForeign and canonical foreign checkpoint lines. Local
Delete removes outgoing foreign edges. The original same-table primitive remains intact.
MemoryStore receives the registered Entity Arc, records its live incarnation at Link,
filters stale tuples on every mentions read, and resolves ambiguous IDs as live local
Memory records. AlreadyLinked compares the full directional tuple. Memory exposes only
named mentions targeting entity. Registry Delete performs real detach; Join now holds
its existing relationship lock across neighbor resolution and row fetch. Entity's only
change is the four-line incarnation override; protocol query evaluation is unchanged.

Replay reproduces foreign tuples without Entity access. Entity deletion and Memory detach
remain separate commits; a crash-window tuple may remain stored. Reinserted same-id Entity
records cannot inherit it, and relinking stores a fresh tuple alongside the inert old one.
Reads never remove tuples. Nonempty atomic WriteBatch remains Unsupported.

## Required coverage

Tests are in uc-facade/tests/tcp.rs unless otherwise identified.

| Proof | Named test and assertion |
|---|---|
| 1 | real_tcp_foreign_link_precedence_and_missing_registry: real Link success, both-live self-loop Malformed, Entity-only self-loop RecordNotFound, absent far endpoint RecordNotFound, unregistered Entity Unsupported; missing local and bad label too. |
| 2 | real_tcp_foreign_neighbors_join_cascades_and_use_switched_pipeline: both neighbor APIs in both directions. |
| 3 | Same test: complete JoinedRow IDs and fields across Memory/entity. |
| 4 | Same test: Memory delete removes outgoing edges; uc-memory/tests/foreign.rs also asserts raw set deletion. |
| 5 | Same test: Entity registry delete invokes detach, neighbors/count miss and a subsequent direct detach removes zero. |
| 6 | uc-memory/tests/foreign.rs::foreign_checkpoint_full_replay_cascades_and_local_incarnation_validation: checkpoint reopen and replay of every history payload equal the complete snapshot, including far incarnation 99 absent locally. Log includes LinkForeign and idempotent DetachForeign. |
| 7 | real_tcp_foreign_neighbors_join_cascades_and_use_switched_pipeline: two non-atomic WriteBatch requests sent together with intervening Use and exact Linked/Deleted responses. |
| 8 | uc-memory/tests/foreign.rs::cmm3_cms3_magic_limits_and_canonical_foreign_checkpoint: old/new magic mismatch, frozen old prefix gate, malformed operations, operation limits and noncanonical/invalid checkpoint rejection. |
| 9 | real_tcp_stale_incarnation_misses_and_relink_keeps_both_tuples: bypass registry detach deliberately to model crash window; missing/reinserted far endpoint yields no neighbors, Join rows or count. |
| 10 | real_tcp_double_collision_uses_local_direction_and_never_swapped_duplicate: adversarial double collision, no incoming-edge blending, swapped Link succeeds and exact pairs result. |
| 11 | real_tcp_unlabeled_memory_join: exact sole foreign descriptor and bare Join Malformed; real_tcp_unlabeled_entity_join unchanged. |
| 12 | uc-protocol/tests/support/interleaving.rs::join_holds_relationship_lock_until_row_fetch_finishes_before_delete_reinsert: actual mutex try_lock returns WouldBlock inside paused far get, B consumes Delete, channel gate releases A, ordered get/release/delete events and old joined fields precede reinserted fields. No sleep race; existing test double/instrumentation only. |
| 13 | real_tcp_stale_incarnation_misses_and_relink_keeps_both_tuples: second Link succeeds, raw full replay contains both incarnation tuples, exact fresh result/count, duplicate is AlreadyLinked, detach count is two then zero. |
| 14 | raw_same_table_edge_never_contributes_to_mentions: raw engine Change::Link exists but colliding Entity is absent from named neighbors/count and real-socket Join. |

The new engine tests also reject missing/stale local incarnations without publication.
Existing Step 3 scenario tests are unchanged. Prior facade Memory same-table assertions
are superseded by the new foreign tests; Entity same-table tests retain their assertions.

## Proof output

All eleven frozen checks passed. The exact agreed test invocation exited 0:

```text
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline
```

[Final workspace test output](step4c-proof-output.txt) records Cargo's explicit exit 0:
119 tests passed, zero failed. The [other ten checks](step4c-additional-proof-output.txt)
each record exit 0: all three fmt checks, all three warnings-denied Clippy checks,
17 convergence-memory tests, 95 portable EXP-0001 tests (harness excluded), Markdown
links and git diff --check. Total: 231 passing tests. The
[development output](step4c-development-output.txt) is retained separately. Only result documentation changed after proof; [final documentation checks](step4c-final-documentation-check.txt)
recheck links and whitespace. This report is advisory, not independent acceptance.

## Deviations and denied/blocked actions

R1b's literal "no other line changes" conflicts with the required rustfmt check: appending
Join makes the match list too long, and rustfmt wraps the existing alternatives onto four
lines. Proposed deviation: retain that formatting-only expansion; the sole semantic
connection.rs change is the new Join arm. No other redesign was needed. D1's "nothing
existing changes" is read together with R2's explicit authorization to extend Delete's
cascade and R3's required version bump; those specific changes are implemented.

The remind-me search and end-of-task auto-capture were denied because the tools require approval and this environment
has approval policy never. No escalation was attempted. Git was absent from PATH; an
existing executable was found at C:/tools/naner_22/vendor/git/cmd/git.exe and supplied
through process-local PATH. Git warns that the global user ignore file is inaccessible.
Some executable-discovery reads of user directories were denied. Cleanup of the generated experiments/exp-0001/target directory was blocked by execution policy; those untracked build artifacts remain, outside the source-change list. No implementation or validation remains blocked. A discovered Meshroom
Python launcher was unusable because its base interpreter path does not exist; the
installed Unity Python 3.13.3 is usable instead. No tools were installed.

The first development test exposed superseded Memory assertions, which were updated.
A later run without Git on PATH failed the existing harness source-identity test with
Windows error 123; the unchanged test was rerun with Git on PATH. The successful development wrapper later returned exit 1 from a trailing executable-discovery search; a separate final invocation records Cargo itself and its exit explicitly. A preliminary whitespace check found mixed line endings in appended documentation; those were normalized. These development
failures are not represented as passing proof.

## Files changed

Relative paths, grouped for review:

- experiments/unified-commitment/crates/uc-memory/src/lib.rs; new tests/foreign.rs.
- experiments/unified-commitment/crates/uc-facade/src/memory.rs, src/entity.rs,
  tests/support/mod.rs, tests/adapters.rs, tests/tcp.rs, README.md.
- experiments/unified-commitment/crates/uc-protocol/src/store.rs, src/connection.rs,
  tests/support/interleaving.rs.
- AGENTS.md; docs/PROJECT-STATUS.md; docs/roadmap/ROADMAP.md;
  docs/experiments/EXP-0005-protocol-facade.md;
  docs/hypotheses/HYP-0005-protocol-facade.md;
  docs/adr/ADR-0005-protocol-facade.md; handoff-2026-09-08/BUILD-LOG.md.
- docs/experiments/EXP-0005/STEP4C-IMPLEMENTATION-REPORT.md,
  step4c-development-output.txt, step4c-proof-output.txt,
  step4c-additional-proof-output.txt and step4c-final-documentation-check.txt.

uc-entity, uc-relation, uc-core, uc-harness, RelationStore, manifests, lockfiles and
prior evidence files are unchanged. Hypothesis Open, experiment Ready; independent
provider review is required before acceptance.
