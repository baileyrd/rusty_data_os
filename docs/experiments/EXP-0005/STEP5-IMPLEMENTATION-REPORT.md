# Step 5 — synthetic core-memory migration proof

Advisory implementation report, 2026-09-09. Independent provider review is required.
This is a non-destructive synthetic fidelity proof, not a consumer migration or a
claim that rusty_remind_me can run on uc-*.

## Authority and scope

Frozen work order SHA-256:
`4edd5ecdbdcc3fedd113b418d93bf3aeac40fdc8bf24bc745b76a220d94baa49`.
The supplied plan file's hash was verified. All edits resolve within this checkout,
`C:/dev/rusty_data_os-step5`, branch `codex/merge-step5-migration-proof`, base
`cb754c142eca02fdb8d2f47708f09a2ae2eeffa2`. No commit, push or publication.

Method was frozen in the attached order before implementation: insert and link over
real loopback TCP; compare records and relationships; checkpoint/reopen with exact
acceptance assertions; copy closed store files and reopen; remove only those copied
checkpoint files and independently replay; compare counts and sorted SHA-256 tuples.
Any missing field, wrong edge, checkpoint rejection, digest mismatch or change to the
original files fails the test. D1 writes; no crash, power-loss or performance claim.

The single flat export fixture has 20 memories, 8 entities, 8 entity relations and
20 memory/entity links. It models the existing exporter's `include_deleted: true`,
`include_graph: true` output, with assistant roles on untagged memories and tagged
graph records. No exporter binary, sibling path dependency or live export was added.
Only source code in rusty_remind_me was read as reference; no database/service data.

## Implementation and coverage

Test suite: [remind_me_migration.rs](../../../experiments/unified-commitment/crates/uc-facade/tests/remind_me_migration.rs).
Fixture: [export.json](../../../experiments/unified-commitment/crates/uc-facade/tests/fixtures/remind-me/export.json).

| Frozen proof item | Executable coverage |
|---|---|
| 1 | `synthetic_export_socket_checkpoint_backup_and_independent_rebuild`: all memories remain gettable through four socket comparison stages. All 13 projections and the two-key envelope are checked; every original Memory field is reconstructed and compared individually. Tombstone/supersession and original nanosecond-precision strings survive. `fixture_covers_optional_metadata_precision_tombstone_and_case_identity` locks in Option coverage and the None/default collision. |
| 2 | Fixture coverage test and import assertions verify whitespace-collapse/lowercase/SHA-256 identity; two case-only aliases resolve to one deterministic Entity ID. ID reverse mapping and literal UUID bytes are also tested. |
| 3 | Each Entity's mention count is recomputed from source joins, compared with stored fields, and checked against actual socket neighbors and complete Join results at every stage. Counts are 3 for the first four entities, 2 for the last four. |
| 4 | Every Relation is read with Get and queried independently by subject, relation and object, plus the complete triple. Endpoints are original unpadded strings. Full directed Memory-to-Entity Join rows, edge count and both neighbor APIs in both directions are checked at every stage. |
| 5 | Both checkpoint and backup reopen assert each OpenReport checkpoint equals the exact saved CheckpointRef.position and rejected_checkpoints is empty. Full rebuild requires None for all three. Actual CheckpointRef paths identify the only deleted files; byte maps prove append logs and original files are unchanged. |
| 6 | Source and final replay counts and all 56 sorted per-record SHA-256 tuples are equal and printed. Source tuples come from the fixture independently of imported records. Final tuples come from socket Get/Join results. |
| 7 | Distinct null, array and scalar metadata fixtures, an ordinary object, and an object containing both envelope-like key names survive the fixed envelope. JSON strings, Unicode and numbers are retained without float conversion. |
| 8 | `rejects_bad_memory_ids_and_timestamps_before_insert` checks missing prefix, bad hex/length, invalid created/updated/deleted timestamps, invalid dates and offsets. It also covers pre-epoch millisecond flooring, leap-year dates and timezone offsets. |

`projections_match_literal_frozen_field_tags` independently fixes the expected field
tags/values for all three domains, rather than relying only on a mapper comparing
against itself. `json_parser_canonical_roundtrip_and_rejections` covers the test-local
JSON parser, escapes, surrogate pairs, duplicate keys, malformed/truncated input and
an independent SHA-256 known-answer vector. There are five new tests.

### Sidecar and digest boundaries

Entity kind nullability/timestamps, Relation original timestamps and link created_at
are read into keyed, in-memory sidecars during import, compared once, then consumed
and dropped before checkpoints or file backup. Their claim is only that import code
read the source fields correctly. They are not recovered from uc-* and are not claimed
to survive backup/replay. Entity has no metadata field; Relation has no metadata field;
foreign edges have no timestamp field. Graph node_id is already absent from export.

Digest encoding is UTF-8 compact JSON arrays of `[field-name, value]` pairs, sorted by
field name; nested objects use lexicographic keys, arrays retain order, and number
lexemes are preserved. This is a deterministic fixture tuple encoding, not a JCS or
general numeric-normalization claim. Records sort by domain and original identity.

- Memory: all 28 reconstructed original fields, including original metadata and
  every original Option/timestamp in the 21-field stash.
- Entity: original id, name, aliases, plus the edge-derived mention_count.
- Relation: original id, subject_entity_id, relation, object_entity_id.
- Link: original memory_id and entity_id.

Entity original kind and Relation original timestamps cannot be reconstructed exactly
from persistent data. They are excluded from digest tuples along with other sidecar
fields. Projected kind, all seven Relation fields (including lossy timestamp values
and sentinels), and every other stored field are still compared separately at all
four stages. Digests do not turn lossy projections into original values.

## Proof output

All eleven agreed checks exited 0. The exact requested command was run from the
checkout root, with no extra arguments:

```text
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline
```

It passed 124 tests (119 existing + 5 new), zero failures. Convergence-memory passed
17 and portable EXP-0001 passed 95 with exp1-descriptive-d1-harness excluded: 236
passing tests across the three workspaces. All three fmt checks and warnings-denied
Clippy checks, Markdown links and git diff --check passed. The existing Memory
1K scenario took 297.78 seconds; it passed unchanged.

[Full eleven-command output](step5-proof-output.txt) records each command and exit
code. [Focused test/digest output](step5-digests-output.txt) records all three exact
checkpoint positions, empty rejection lists, checkpoint-free None reports, counts
`[20, 8, 8, 20]` and all 56 source/rebuilt SHA-256 pairs. A supplemental independent
Python standard-library json/hashlib check reproduced all 56 hashes. This is still
local advisory evidence, not the required independent-provider review.

## Deviations, interpretations and blocked actions

1. The order says 27 Memory fields, but its own explicit list and the cited real
   Memory struct have 28: six exact direct fields + metadata + 21 stash fields.
   Implemented the complete explicit field list; no source field is discarded.
2. D5 asks for an Entity node_id sentinel although the frozen Entity schema has
   only label/kind/mention_count/aliases. It is impossible to write node_id without
   an unauthorized schema change. Proposed deviation: no assignment; disclose that
   this field is absent from both exported Entity and target Entity.
3. Proof item 4's literal bidirectional Join wording exceeds Step 4c: only Memory
   advertises a foreign target, and Join is directed Memory-to-Entity. Proposed
   deviation: exact forward Join plus both-direction Neighbors and
   NeighborsByRelation from the Memory table, preserving the frozen engine/protocol.
4. Facade wrappers privately own engines and expose no checkpoint API. The test
   closes the first listener, opens raw engines to write checkpoints, then closes
   and reopens for socket verification. No public API is changed.
5. The test-only RFC3339 parser explicitly rejects leap-second `:60`; the target
   POSIX millisecond projection has no unique leap-second representation. Proposed
   bounded deviation: fail closed rather than invent a leap-second folding policy.
   The fixture uses ordinary timestamps emitted by the cited Utc::now path. No
   general production RFC3339 importer is claimed.
6. R1 says the fixture is committed. The user's explicit no-commit instruction
   takes precedence: all new files remain uncommitted for independent review.

No dependencies were added. SHA-256 reuses cm-trace; JSON/timestamp helpers are private
test code. No change to uc-core, uc-protocol, uc-harness, engine/facade implementation,
manifests, lockfiles, Step 1–4 decisions, or consumer configuration. Documentation
additions record bounded knowledge only; hypothesis Open and experiment Ready.

Git was absent from PATH; the existing `C:/tools/naner_22/vendor/git/cmd/git.exe`
was supplied through process-local PATH. Git warns that the global ignore file is
inaccessible. One executable-discovery read of AppData/Local/Programs and a CIM
process-parent query while checking proof progress were denied; Get-Process still
showed the existing long-running Memory scenario making progress. No escalation
or installation was attempted. The existing Unity Python runs the
Markdown validator. Memory search/capture tools were intentionally not invoked:
the frozen work order excludes touching the real memory service or its data.

Tombstoned/superseded source rows are intentionally imported as live Put records.
This reproduces the documented resurrection hazard if treated as a real importer;
read filtering or actual deletion is not implemented. FTS, vitality computation,
wiki, vectors, sync and other application capabilities remain out of scope.

## Files changed

- New uc-facade test and JSON fixture linked above.
- This report, step5-proof-output.txt and step5-digests-output.txt.
- Additive Step 5 notes in PROJECT-STATUS.md, EXP-0005-protocol-facade.md and
  HYP-0005-protocol-facade.md. No previous decision is superseded.

Cargo also left an untracked generated `experiments/exp-0001/target/` directory.
These build artifacts are not source changes or migration evidence and were not
deleted. No implementation or proof action remains blocked.

Uncommitted implementation SHA-256 values for review:

```text
remind_me_migration.rs  4a8d5f03e1007c39c64135302bd34bf76397fd4e412765282c01c19345e8a596
export.json            d20f50461d3e0df736364c2fb16d749edffeeb30974fe518d8b4bd40fb0bbe63
```

## Inspection 1 (fresh Claude CLI): APPROVED, one low finding, fixed

**Verdict: APPROVED.** "The Step 5 change is purely additive ... and does not touch
uc-core/uc-protocol/uc-facade production source, matching the plan's Non-goals ... The
D2-D6 mapping ... is implemented faithfully and internally consistent."

One low-severity finding (S5-1): `export.json`'s graph records modeled the order
`entity -> entity_relation -> memory_entity`, but `remind_me_core/src/export.rs`'s real
`collect_graph_records` actually emits `entity -> memory_entity -> entity_relation`
(`records = entities; records.extend(links); records.extend(relations);`,
`export.rs:298-307`). No functional effect — `Fixture::read()` classifies records by
`record_type` into independent lists and only requires entities to precede both
dependent groups, true under either ordering, and no test assertion depends on the
inter-group order. Fixed directly by the host (a pure JSON reordering, no logic change):
`export.json`'s graph block now matches the real emission order exactly. Re-verified:
`uc-facade`'s full test suite (including `remind_me_migration.rs`'s five tests) passes
unchanged after the reorder, plus a targeted `fmt`/`clippy`/`git diff --check` recheck —
all exit 0. Clean close within budget: fix round 1 of 2 used, inspection round 1 of 2
used.
