# Host dispositions on Codex review 1 of the Step 4a work order (plan sha256 0f97894f…57ee9)

Both findings accepted; the work order was revised as follows. Please re-review the revised
plan (new hash) for approval.

- S4A-001 (medium, Relation validation gaps): R3's `apply` rules for `Put` now require, before
  any write, exactly what `src/server/relation.rs:154-213`'s `relation_from_fields` requires:
  `subject` non-empty, `relation` non-empty, `object` non-empty, `deleted_at_unix_ms >= 0`.
  `created_at_unix_ms`/`updated_at_unix_ms` stay unrestricted signed values; `subject`/`object`
  stay unchecked for existence (deliberately, per ADR-0058). Two new negative scenario tests
  added (empty `relation`, negative `deleted_at_unix_ms`), each covering both insert and
  whole-record replace.
- S4A-002 (medium, Entity label lifecycle): R2's `State` now carries `known_labels:
  BTreeSet<String>`, distinct from `edges`, seeded with the two legacy built-in labels
  (`relates_to`, `mentioned_with`) at construction even with zero edges, and never shrunk by
  any operation including `Delete` (which still cascades to remove edge tuples naming the
  deleted id, but leaves `known_labels` untouched). `Link`'s `apply` inserts into
  `known_labels` on success, so recovery (which replays only `apply`) reconstructs the same set
  as commit-time. `neighbors_by_relation`/`list_relation_kinds` are now specified in terms of
  `known_labels` (unknown label → `Malformed`; known label with no current edges → `Ok(vec![])`).
  Checkpoint encode/decode must round-trip `known_labels`. Three new scenario tests added:
  fresh-store built-in labels, known-but-empty-after-delete vs. unknown-label distinction
  (checked across checkpoint/reopen and across full replay), and the checkpoint/full-replay
  digest test now explicitly includes `known_labels`.
