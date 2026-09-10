# Step 5 work order — a verified, non-destructive core-memory migration proof (`rusty_remind_me`)

## Host decision (2026-09-09, autonomous per the owner's `/goal` directive)

The owner set an explicit goal ("complete the merge and migration without my interaction") and
directed the host not to pause and ask. Before drafting a literal "migrate `rusty_remind_me`" work
order per the 2026-09-08 merge plan's Step 5 text, the host investigated `rusty_remind_me`'s
**actual current state** — the plan itself required this ("subject to checking its actual current
integration when the pilot begins. The retained spike alone does not establish what it currently
runs"). That investigation found the plan's premise no longer holds, and the host made the
following scope decision rather than either (a) silently attempting a literal migration that would
regress the owner's live, daily-used personal memory system, or (b) stopping to ask.

**What was found:** `rusty_remind_me` (`C:/dev/rusty_remind_me`, `main`, commit `593f793`) is a
mature, 24-table SQLite (schema version 29, `remind_me_core/src/db/migrations.rs:49`) system with
capabilities that have no equivalent anywhere in `rusty_data_os`'s experimental
`uc-core`/`uc-memory`/`uc-entity`/`uc-relation`/`uc-protocol`/`uc-facade` stack:

- Full-text search (`memories_fts`, an FTS5 virtual table synced to `memories`,
  `schema_tables.sql:65-69`).
- ACT-R-style vitality/decay scoring (`decay_rate`, `vitality`, `base_weight`, `accessed_at` on
  every memory row; `vitality.rs`).
- A markdown wiki (`wiki_pages`/`wiki_links`/`wiki_meta`, `wiki.rs`/`wiki_import.rs`).
- Vector embeddings and ANN search (`vec_chunks`/`embedding_meta`, `ann_index.rs`, `reranker.rs`,
  `embedder.rs`).
- Multi-node sync through a central hub (`sync_log`/`sync_outbox`/`sync_sends`/`sync_flags`,
  `remind_me_hub` as its own binary with an optional Postgres store) — this is not a hypothetical:
  the owner completed a real production migration of this exact hub/connector/dashboard onto a new
  host (MS-01 LXC 106) very recently, per this session's own memory of that work.
- Saved searches, memory revisions, feedback, analytics snapshots, and three distinct importer
  paths (chat/DBS/mempalace).

`rusty_data_os`'s Memory domain (`experiments/unified-commitment/crates/uc-facade/src/memory.rs`)
has exactly **13 fields** — a close subset of `remind_me`'s own `memories` table, evidently modeled
on it, but with no representation at all for the other 23 tables or any of the capabilities above.

**Decision:** a literal "migrate `rusty_remind_me`" as the 2026-09-08 plan's Step 5 describes it —
culminating in "point the consumer at the new engine and exercise its normal workflow" — is not
something to attempt right now. Doing so would either silently discard real, currently-relied-on
capability (search, decay-based ranking, wiki, sync, vectors), or require a large, separately-scoped
feature-parity effort on `uc-*` that was never actually specified by any prior step's work order
(Steps 4a-4c only ported the 13-field record shape and simple relations, never remind_me's
differentiated features). Inventing and executing that much net-new, unscoped design work
unsupervised, against the owner's real production system, is not a reasonable reading of "complete
the migration" — it substitutes the host's own judgment for a decision (accept a real capability
regression, or fund a multi-week engineering effort) that is the owner's to make, not something to
silently decide by proceeding.

**What this work order does instead:** build and verify, for real, the *mechanism* the plan's Step 5
sub-steps 1-4 require (export, import, comparison, reopen/rebuild, backup/restore) for the one slice
of data that genuinely has a home in `uc-memory`/`uc-entity` today — the core `memories` and
`entities`/`entity_relations`/`memory_entities` tables — proven against realistic **synthetic**
fixtures modeled precisely on `remind_me_core`'s real schema and id-generation code (cited below),
never against the owner's live database (this dev machine has no local copy of it, confirmed by
search; nothing in this work order reads, exports, or otherwise touches the real service or its
data). Sub-step 5 (point the consumer at the new engine) is explicitly **not** performed here — see
"Non-goals."

## Target repository

New worktree `C:/dev/rusty_data_os-step5`, branch `codex/merge-step5-migration-proof`, based on
Step 4c's closing commit (`cb754c1`, `codex/merge-step4c-foreign-edges`). This work order touches
only `rusty_data_os`; `rusty_remind_me` is read-only reference material (cited, never modified).

## Repository facts

### `remind_me_core`'s real schema and id formats (source of truth for the mapping)

`C:/dev/rusty_remind_me/crates/remind_me_core/src/db/schema_tables.sql:55-63`, the `memories`
table, 27 columns total (`id` plus 26 data columns); the full Rust shape used for JSON export is
`models.rs:60-115`'s `Memory` struct (`Serialize`/`Deserialize`), reproduced here field-by-field in
the "Design decisions" section below.

Id generation, verbatim:

- Memory: `queries.rs:101`, `format!("mem_{}", uuid::Uuid::new_v4().simple())` — a `mem_` prefix
  plus a 32-hex-character (no-dash) UUID.
- Entity: `entity.rs:44-46`, `sha256::digest(normalize_entity_name(name))[..12].to_string()` — the
  **first 12 hex characters (6 bytes) of a SHA-256 digest of the case-folded name**, not a UUID at
  all, and deterministic (same name always yields the same id; `upsert_entity` at `entity.rs:58-75`
  keys on it for exactly this reason — it is real domain logic this migration must not break by
  assigning entities a different, colliding id scheme).
- `entities` table: `schema_tables.sql:35-43` — `id, name, kind, aliases (JSON array), created_at,
  updated_at, node_id`.
- `entity_relations` table: `schema_tables.sql:45-52` — `id, subject_entity_id, relation,
  object_entity_id, created_at, updated_at, node_id`.
- `memory_entities` table: `schema_tables.sql:80-85` — a pure join table, `(memory_id, entity_id,
  created_at)`.

### The existing, reusable export machinery, and its real output shape (do not hand-roll a new
### exporter, and do not model a shape it doesn't actually produce)

`remind_me_core/src/export.rs` already has a tested, production export path. Its real output is a
**flat, mixed-record JSON array**, not a clean table dump — the importer must handle exactly this:

- `collect_export_records` (`export.rs:311-343`) selects every `memories` column, deserializes
  into the real `Memory` struct, serializes each with `serde_json::to_value`, then **injects
  `"role": "assistant"` onto every memory record** (`export.rs:330-334`) — untagged by
  `record_type`, this is how a memory record is distinguished from a graph record in the flat
  array. `models.rs:1021-1028`'s doc comment on `ExportInput::include_deleted` states this is
  **deliberate and load-bearing**: exported records are stamped as live content specifically so a
  normal re-import treats them as such, and `include_deleted` therefore defaults to `false` — "a
  round-trip of an export that carried tombstones and superseded facts would resurrect them as
  fresh live memories" is the documented risk of turning it on.
- `collect_graph_records` (`export.rs:222-306`), only when `include_graph` (defaults `true`, per
  `models.rs:1017-1020`), appends `record_type`-tagged records: `{"record_type": "entity", "id",
  "name", "kind" (nullable), "aliases", "created_at", "updated_at"}` (**no `node_id`** —
  `export.rs:236-253` simply never selects it); `{"record_type": "entity_relation", "id",
  "subject_entity_id", "relation", "object_entity_id", "created_at", "updated_at"}` (**no
  `node_id`** either, `export.rs:258-274`); `{"record_type": "memory_entity", "memory_id",
  "entity_id", "created_at"}` (`export.rs:298-306`). Entities are emitted before the links/relations
  that reference them (`export.rs:213-215`'s doc comment).
- `export_memories` (`export.rs:367+`) wraps this with path validation and format selection
  (`render_export`, `export.rs:351-360`, JSON or JSONL).

This work order **reuses `ExportInput`'s own configuration knobs, deliberately set away from their
safe production defaults, and discloses that explicitly**: `include_deleted: true` (so a fixture
can include a tombstoned/superseded record at all — otherwise the exporter would silently exclude
it, defeating D2's soft-delete round-trip test) and `include_graph: true` (the default). **This is
a data-fidelity proof only, not a template for a real importer** — see "Non-goals" for why a real
importer would need read-path filtering or an actual delete, not a bare `Put`, before ever running
against production data with `include_deleted: true`.

### `uc-memory`/`uc-facade`'s current Memory and Entity schemas (the import target)

`experiments/unified-commitment/crates/uc-facade/src/memory.rs`, `MemoryStore::schema()`: 13 fields
— `content (Str), category (Str), tags (StrList), source (Str), metadata_json (Str),
created_at_unix_ms (I64), updated_at_unix_ms (I64), memory_type (Str), status (Str), sensitive
(Bool), access_count (I64), deleted_at_unix_ms (I64), node_id (Str)`. `RecordId` is `[u8; 16]`
(`uc-protocol/src/types.rs:17`).

`experiments/unified-commitment/crates/uc-facade/src/entity.rs`, `EntityStore::schema()`: 4 fields
— `label (Str), kind (Str), mention_count (I64), aliases (StrList)`.

`experiments/unified-commitment/crates/uc-facade/src/relation.rs:12-26`, `RelationStore::schema()`:
7 fields — `subject (Str), relation (Str), object (Str), created_at_unix_ms (I64),
updated_at_unix_ms (I64), node_id (Str), deleted_at_unix_ms (I64)`. `subject`/`object` are plain
`Str` value fields here (not `RecordId` cross-references) — `uc-relation` stores an opaque string
on each side, matching `entity_relations.subject_entity_id`/`object_entity_id`'s own `TEXT` typing.

### Relation identity is deterministic too, same pattern as Entity

`remind_me_core/src/entity.rs:747-764`, `entity_relation_id`, verbatim logic:
`sha256::digest(format!("{subject_entity_id}|{normalized_relation}|{object_entity_id}"))[..12]` —
another 12-hex-character (6-byte) deterministic id, exactly Entity's own scheme (D3) applied to a
different key. `entity_relations` has **no `deleted_at` column at all** (`schema_tables.sql:45-52`)
— `uc-relation`'s `deleted_at_unix_ms` field has no source concept to map from, not merely a null
value to carry (see D6).

## Design decisions settled by the host

- **D1 (revised, review round 1 R4) — synthetic fixtures modeling the real, flat, mixed export
  envelope, not a hand-described table dump.** No path dependency on `rusty_remind_me` is added
  (this project's own `AGENTS.md`/`ARCHITECTURE.md` in that repo documents exactly this mistake —
  path dependencies against a sibling repo that isn't guaranteed to exist at that relative location
  — as a removed, cautionary pattern; repeating it here would tie `rusty_data_os`'s build to a
  second repository's presence and exact layout). One fixture file,
  `experiments/unified-commitment/crates/uc-facade/tests/fixtures/remind-me/export.json`, a single
  flat JSON array matching exactly what `export_memories` with `include_deleted: true,
  include_graph: true` produces (cited above): untagged `Memory`-shaped objects each carrying the
  injected `"role": "assistant"`, followed by `record_type: "entity"` / `"entity_relation"` /
  `"memory_entity"` objects, entities before the edges referencing them. At least 20 memory records
  and 8 entities/relations/links, covering: every `Option` field both present and absent; a
  `deleted_at`-set (tombstoned) record and a `superseded_by`-set record (only reachable in the
  fixture because `include_deleted: true` is frozen in, per the Repository facts section); a record
  whose `memory_type`/`status` are `None` (pre-#198 rows, per the `models.rs:93-97` comment); at
  least one entity name requiring case-fold normalization (two source aliases differing only in
  case, both resolving to the one deterministic id per `entity.rs:44-46`); three records whose
  `metadata` is respectively `null`, a JSON array, and a bare scalar (review round 2 S5-002 —
  `metadata` is `serde_json::Value`, not guaranteed to be an object) alongside the common case of a
  real object; a `created_at`/`updated_at` pair with sub-millisecond precision (`Utc::now().to_rfc3339()`,
  `queries.rs:100`, is realistic sub-millisecond input) to exercise D2's exact-fidelity requirement,
  not just millisecond-aligned timestamps that would hide truncation.
- **D2 (revised, review round 1 R1/R2) — the 13 `uc-memory` fields are lossy *queryable
  projections*; exact fidelity lives entirely in D4's stash, never in the projections themselves.**
  `content`/`category`/`tags`/`source`/`sensitive`/`access_count` copy directly (verbatim, no
  lossiness). `metadata` (a JSON object in `remind_me`) serializes to a JSON string for
  `metadata_json`, with D4's stash nested inside it (below) — the injected `"role": "assistant"`
  export-envelope artifact (Repository facts) is recognized and discarded by the importer, never
  written anywhere, since it is a `remind_me`-export-format artifact, not domain data. `node_id`
  (`Option<String>`) and, for a fresh record, `memory_type`/`status` (`Option<String>`, per
  `models.rs:99-102`'s comment on legacy rows predating those columns): the *projection* uses
  `""` (empty string) for `node_id: None`, and `remind_me`'s own documented fresh-row defaults for
  `memory_type`/`status: None` (`schema_tables.sql:64`'s `DEFAULT 'unclassified'`/`DEFAULT
  'active'`) — but the **projection is not what Proof items 1/5 compare for exactness**; the
  original `Option` state (including `None`) for all three fields is separately captured in D4's
  stash, so a projection collision (e.g. a real `memory_type: Some("unclassified")` versus a
  projected `None`) is always distinguishable from the stash, never from the projection alone.
  `created_at`/`updated_at` (RFC 3339 strings, sub-millisecond precision possible) parse to Unix
  epoch milliseconds for the projection fields — a **known-lossy floor to the millisecond**; the
  original RFC 3339 strings are D4-stashed verbatim and are what Proof items 1/5 actually compare.
  Reject (fail the import for that record, do not silently default) an unparseable timestamp.
  `deleted_at` (`Option<String>`, a **soft-delete tombstone marker** in `remind_me`'s own model, per
  `models.rs:112-115`'s comment — distinct from an actual deleted row) maps its projection to
  `deleted_at_unix_ms`: `None` → `0` (sentinel; unambiguous since epoch-ms `0` is 1970), `Some(ts)`
  → the parsed epoch-ms value; the original `Option<String>` is D4-stashed. **A tombstoned
  `remind_me` record is imported as a live `uc-memory` record — `Change::Put`, never
  `Change::Delete`.** `uc-core`'s incarnation-tracked delete is a different concept (a real,
  no-longer-existing row) than `remind_me`'s soft-delete flag (a still-existing, filtered-out row);
  conflating them would make the imported record's history diverge from the recoverable source
  data. This is explicitly a fidelity proof, not a safe import default — see Non-goals.
- **D3 — id mapping, both directions verified.** Memory: strip the `mem_` prefix (reject, do not
  silently accept, any id lacking it), parse the remaining 32 hex characters as `Uuid::simple`
  bytes (16 bytes, exact fit for `RecordId`). Entity and Relation (same scheme — both are 12-hex,
  6-byte deterministic ids, `entity.rs:44-46`/`entity.rs:747-764`): right-pad with 10 zero bytes to
  fill `RecordId`'s 16 bytes (left-aligned, source bytes first, so two different 6-byte source ids
  can never collide after padding). This padding applies only to a record's **own** identity
  (its `RecordId` in the registered table); `uc-relation`'s `subject`/`object` fields are plain
  `Str` values, not `RecordId`s (Repository facts), and carry the **original, unpadded 12-hex
  entity id string** verbatim — padding them too would silently break any future cross-reference
  back to the Entity table by string equality. The original id string (all three kinds) is
  preserved verbatim in D4/D6's stash specifically so identity is round-trip verifiable independent
  of the byte-mapping scheme chosen here.
- **D4 (revised, review round 1 R1/R2; revised again round 2 S5-002) — every `Memory` field
  without an exact-fidelity home in the 13-field projection is stashed losslessly, inside a fixed
  envelope object, never dropped, never assuming `metadata`'s shape.** `remind_me`'s `metadata` is
  `serde_json::Value` (`models.rs:66`), not guaranteed to be a JSON object — `db/queries.rs:295-299`
  writes whatever value is supplied without an object check, and the reader deserializes arbitrary
  JSON, so a real exported record's `metadata` can legally be `null`, an array, or a scalar. Nesting
  the stash *into* the original metadata's own top-level keys (the round-1 design) has no valid
  encoding for those shapes. Instead, `metadata_json` always holds a **fixed two-key envelope
  object**, regardless of what `metadata` originally was:
  ```json
  {"original_metadata": <metadata value, verbatim, whatever shape it is>,
   "_remind_me_migration_extra": { ... }}
  ```
  This has no collision case to reject (unlike the round-1 design) — the original value is nested
  as a value under `"original_metadata"`, never merged into a shared top level, so it cannot collide
  with the stash key regardless of its own shape or contents. `"_remind_me_migration_extra"` holds:
  the id string (D3), `capture_id`, `subject`, `predicate`, `object`, `superseded_by`, `decay_rate`,
  `vitality`, `base_weight`, `accessed_at`, `doc_id`, `chunk_index`, `remind_at`, `client`,
  `source_capture_id` (14 fields with no projection at all), **plus** the *original* `memory_type`,
  `status`, `node_id` (`Option<String>`, including `None` explicitly, not merely their lossy
  projected value — D2), **plus** the *original* `created_at`, `updated_at`, `deleted_at` RFC 3339
  strings exactly as given (D2) — 21 fields total. This is what makes the round-trip in Proof items
  1 and 6 exact: nothing in the 27-field source record is unrecoverable from the imported
  `uc-memory` record (including `metadata` itself, in any shape), and no projection's lossiness
  (millisecond flooring, a `None`-vs-default collision) is mistaken for the authoritative value
  during comparison.
- **D5 (revised, review round 1 R2/R4) — Entity import: `aliases`/`label` map directly
  (`name`→`label`); `kind`'s projection uses `""` for `None`, with the original `Option<String>`
  retained separately (below); `mention_count` is *recomputed*, not copied.** `remind_me`'s
  `entities` table has no `mention_count` column at all (it is derived elsewhere, e.g. from
  `memory_entities` join counts) — `uc-entity`'s `mention_count` is populated by counting the
  imported `memory_entities` rows referencing that entity after all memories are imported, not
  treated as a lossy field mapping. **`uc-entity` has no metadata/JSON-string field at all to stash
  into** (its schema is exactly `label/kind/mention_count/aliases`) — so `kind`'s original
  `Option<String>`, plus `created_at`/`updated_at` (both present in the real export's `entity`
  record), are recorded only in the comparison harness's own retained fixture copy, not inside
  `uc-entity` itself, and the proof's comparison step verifies them against that retained copy
  rather than against anything read back from `uc-entity`. **`node_id` has no source data at all to
  preserve** — the real exported `entity` record never carries it (`export.rs:236-253` simply never
  selects the column; confirmed a genuine, pre-existing gap in `remind_me`'s own exporter, not a
  mapping decision made here) — so `uc-entity` import always uses the `""` sentinel for it with
  nothing to stash, disclosed as real information already absent from the export format this work
  order reuses, not information this migration proof fails to carry forward.
- **D6 (revised, review round 1 R3) — `entity_relations` import uses `uc-relation`, fully specified;
  `memory_entities` import uses Step 4c's `"mentions"` foreign edge, fully specified.**
  `entity_relation` records (`id, subject_entity_id, relation, object_entity_id, created_at,
  updated_at` — no `node_id` in the real export, same gap as Entity, D5) map to `uc-relation` as:
  `id` → the relation's own `RecordId` (D3's padding scheme applied to `entity_relation_id`'s
  12-hex id); `subject_entity_id`/`object_entity_id` → `subject`/`object` (the original, unpadded
  12-hex entity id strings, D3); `relation` → `relation` (verbatim); `created_at`/`updated_at` →
  `created_at_unix_ms`/`updated_at_unix_ms` (parsed; the original RFC 3339 strings are retained in
  the harness's fixture copy, same treatment as D5, since `uc-relation` has no metadata field
  either); `node_id` → `""` (no source data, same disclosed gap as D5); `deleted_at_unix_ms` → `0`
  unconditionally — `entity_relations` has **no `deleted_at` column at all** (Repository facts), so
  this is a schema field with no source concept whatsoever, not a null value being defaulted.
  `memory_entity` records (`memory_id, entity_id, created_at`) map onto `Change::LinkForeign`
  (Step 4c): `from` = the memory's `RecordId` (D3), `to` = the entity's `RecordId` (D3's padding) —
  this is exactly the mechanism Step 4c built and is why this proof depends on Step 4c's close
  rather than an earlier step. The join's own `created_at` has **no field at all** in
  `foreign_edges`' `(from, from_incarnation, to, to_incarnation)` tuple (Step 4c's design) — retained
  in the harness's fixture copy only, disclosed, not silently dropped.

## Required changes

**R1 — one new test fixture.** `experiments/unified-commitment/crates/uc-facade/tests/fixtures/
remind-me/export.json` (D1), committed, matching the real flat mixed-record export envelope
exactly (Repository facts).

**R2 — a new integration test suite, `uc-facade/tests/remind_me_migration.rs`.** Not a standalone
binary (D1 already ruled out any dependency on `remind_me_core`'s code, so there is no real
consumer for a general-purpose importer binary yet — a test suite proves the mechanism without
speculatively building a tool with no second caller). Steps, all over a real socket (matching this
project's established real-TCP testing convention, Steps 4b-ii/4c):

1. Read the fixture (R1), separate the flat array into memory records (untagged, `role` stripped)
   and graph records (by `record_type`), apply the D2-D6 mapping in test code, and submit each
   resulting record as a real `Insert`/`Link` over the wire to a fresh `uc-facade` listener (Memory,
   Entity, and Relation tables registered together, matching Step 4c's registry wiring). Retain the
   D5/D6-noted harness-only fields (Entity `kind`-nullability/`created_at`/`updated_at`; Relation
   `created_at`/`updated_at`; the `memory_entity` join's `created_at`) as an in-memory sidecar keyed
   by original id, for later comparison — never written into `uc-*`.
2. Read every record back (`Get`, `Join`, `neighbors_by_relation("mentions")`) and assert the D2-D6
   mapping round-trips exactly: for Memory, reconstruct the *original* 27-field record from the 13
   projection fields plus D4's 21-field envelope stash (not from the projection fields alone, per
   D2/D4's revision) and compare field-by-field against the fixture; for Entity/Relation, compare
   the `uc-*`-held fields plus the retained sidecar (D5/D6) against the fixture. **This step's
   sidecar-dependent comparisons (D5/D6's Entity `kind`-nullability/timestamps, Relation
   timestamps, join `created_at`) are import-time-only** — see the scope note after step 5.
3. Checkpoint and reopen (matching Step 4c's own replay proof). Retain the checkpoint position each
   domain's `checkpoint()` call reports (`uc-core/src/lib.rs:542-549`). Assert the reopened store's
   `OpenReport.checkpoint == Some(<that exact position>)` and `OpenReport.rejected_checkpoints` is
   empty for every domain (review round 2 S5-003 — proving the checkpoint was genuinely accepted
   and used, not silently bypassed via `uc-core/src/checkpoint.rs`'s reject-and-fall-back-to-replay
   path) — then re-verify every **`uc-*`-held-field** comparison from step 2 (D2's 13 projections,
   D4's envelope stash, D6's `uc-relation` fields) against the reopened store.
4. Copy the store's data files to a second directory (a file-level backup, per the plan's own
   "create and restore a backup into another directory" language) and reopen *that* copy. Assert
   the same checkpoint-acceptance condition as step 3 (S5-003) — a backup that silently lost or
   corrupted its checkpoint file must be caught here, not masked by a fallback replay that happens
   to still produce the right data. Re-verify every `uc-*`-held-field comparison again. Confirm the
   original directory is untouched by this (a real restore-from-backup proof, not just a second
   read of the same files).
5. (Review round 1 R5 — independent reconstruction, not a reused materialized checkpoint.) From the
   same backup copy, delete only the checkpoint file(s) for each domain (`uc-core`'s
   `checkpoint::write`-produced file, not the append-log) before reopening, and assert the
   resulting `OpenReport.checkpoint` is `None` (`uc-core/src/recovery.rs:19`'s field, confirming a
   full history replay actually happened, not a reused pre-materialized checkpoint state per
   `uc-core/src/lib.rs:268-272`'s checkpoint-then-only-later-events restore path) — then repeat
   every `uc-*`-held-field comparison from step 2 against *this* independently-rebuilt store. This
   is what makes the restore proof match the merge plan's own Step 5.4 language ("reopen and
   independently rebuild derived representations"), not merely a second load of an
   already-materialized state.

   **Scope note (review round 2 S5-001):** D5/D6's sidecar-only fields (Entity `kind`-nullability/
   `created_at`/`updated_at`; Relation `created_at`/`updated_at`; the `memory_entity` join's
   `created_at`) live only in the test process's own memory — they were never written into any
   `uc-*` table (D5/D6 disclose this; `uc-entity`/`uc-relation` genuinely have no field to hold
   them). Comparing them after a file-copy backup or a checkpoint-free rebuild would silently
   compare the fixture against itself (the sidecar survives because the *test process* is still
   running, not because the data survived the backup) — that would prove nothing about backup
   fidelity and must not be reported as if it did. Steps 3-5 therefore compare **only** the fields
   actually persisted inside `uc-*` (D2's 13 projections + D4's 21-field envelope stash for Memory;
   `uc-relation`'s 7 real fields; `uc-entity`'s `label`/`aliases`/recomputed `mention_count`, plus
   its projected `kind`). The sidecar-only fields are verified once, at step 2 (immediately after
   import, still within the same process) — their fidelity claim is scoped to "the import code read
   them correctly from the fixture," not "they survive a backup or replay," and the implementation
   report must state this distinction explicitly rather than imply broader coverage.
6. Record counts, sorted per-record digests (SHA-256 over each record's canonical, *reconstructed
   original* field tuple, restricted to the `uc-*`-persisted fields per the scope note above — not
   the lossy projection tuple, and not the sidecar-only fields) for both the fixture source and the
   final checkpoint-free-rebuilt store (step 5), and assert they match exactly — the plan's own
   literal "sorted per-record digests" comparison method (Step 5, sub-step 3).

## Non-goals

**Not performed:** exporting or reading any real `rusty_remind_me` data (live or backed up); adding
any dependency (path or otherwise) on the `rusty_remind_me` repository; pointing any real consumer
at the new engine; replicating full-text search, vitality/decay computation, the wiki, vector
embeddings/ANN, multi-node sync, saved searches, revisions, feedback, or analytics — none of these
have any home in `uc-*` today and building one is unscoped, substantial, separate work. No change
to `uc-core`, `uc-protocol`, `uc-harness`, or any Step 1-4c decision; this work order is purely
additive test/fixture code in `uc-facade`. No opening the listener beyond loopback, no real
authentication (still open from Step 4c's own handoff). No claim that this proof establishes
`rusty_remind_me` migration readiness beyond the narrow slice of data and mechanism it actually
exercises — the "Host decision" section above states plainly what remains undone and why.

**Explicitly not a safe real-import path yet.** This proof deliberately imports tombstoned/
superseded records as live `Change::Put`s (D2) with `include_deleted: true` (Repository facts) —
exactly the resurrection risk `remind_me`'s own `models.rs:1021-1028` documents and defends against
by defaulting `include_deleted` to `false`. That is acceptable *here* because the goal is proving
field-level fidelity end-to-end, and nothing downstream of this test suite ever reads the result as
live data. A real importer built on this mechanism would need to either filter reads on
`deleted_at_unix_ms != 0`/a stashed `superseded_by`, or submit an actual `Change::Delete` for a
tombstoned source record instead of a bare `Put` — neither is implemented here, and using this
proof's mapping code unmodified against real data would reproduce the exact hazard `remind_me`'s
own maintainers already identified and guarded against.

## Proof

Same eleven-command chain as every prior Step 4/5 increment (unified-commitment fmt/clippy/test;
convergence-memory fmt/clippy/test; exp-0001 fmt/clippy/test, harness excluded; markdown links;
`git diff --check`), via `cargo +1.89.0-x86_64-pc-windows-gnu`, exit 0 required. Additional required
test coverage, named explicitly in the implementation report:

1. Every fixture memory record round-trips through insert → real-socket read → checkpoint/reopen →
   file-backup/restore-reopen → checkpoint-free independent rebuild (R2 step 5), with all
   `uc-*`-persisted fields (13 lossy projections + D4's 21-field envelope stash) exactly
   reconstructable at every stage — including the tombstoned/superseded record staying a live,
   gettable `uc-memory` row whose original `deleted_at`/`superseded_by` strings are recovered
   exactly from the stash (D2/D4, not merely "some nonzero marker present"), the
   sub-millisecond-timestamp record's original RFC 3339 strings recovered byte-for-byte (not just
   millisecond-equal), and the pre-#198 record's original `None` `memory_type`/`status`
   distinguished from a real `"unclassified"`/`"active"` value via the stash, not conflated by the
   projection. Sidecar-only Entity/Relation fields (D5/D6) are verified once, at step 2 only, per
   R2's scope note — not claimed to survive backup/restore/rebuild.
2. Entity import: two differently-cased aliases of the same source name resolve to the one
   deterministic entity id (D3's padding scheme applied to `entity.rs:44-46`'s id), matching
   `remind_me`'s own case-fold-then-hash identity rule, not a naive per-alias id.
3. `mention_count` after import equals the real count of imported `memory_entities` edges
   referencing that entity (D5) — not a copied, absent, or default value.
4. `entity_relations` import produces real `uc-relation` records queryable by `subject`/`relation`/
   `object`, with `subject`/`object` holding the original, unpadded 12-hex entity id strings (D3),
   and `memory_entities` import produces real `"mentions"` foreign edges (Step 4c's mechanism)
   queryable by `Join`/`neighbors_by_relation` in both directions.
5. (Revised, review round 2 S5-003.) Both the checkpoint/reopen (R2 step 3) and backup/reopen (R2
   step 4) stages report `OpenReport.checkpoint == Some(<the exact position `checkpoint()`
   returned>)` and empty `rejected_checkpoints`, for every domain — a checkpoint that was silently
   rejected or corrupted in the backup must fail this assertion even if a fallback replay happens
   to still produce correct data. The independent rebuild (R2 step 5) separately reports
   `OpenReport.checkpoint == None` for every domain, proving that stage is a genuine full-history
   replay, not a reused materialized checkpoint.
6. Sorted per-record SHA-256 digests, computed over each record's *reconstructed original* field
   tuple restricted to `uc-*`-persisted fields (not the lossy projection tuple, not the sidecar-only
   fields — R2's scope note), match exactly between the source fixture and the final
   checkpoint-free-rebuilt store (the plan's own literal comparison method, satisfied against
   independently-rebuilt state per item 5).
7. (Revised, review round 2 S5-002.) A fixture memory record whose `metadata` is `null`, a JSON
   array, and a bare scalar (three separate cases, not just an object) each round-trip exactly
   through the `{"original_metadata": ..., "_remind_me_migration_extra": {...}}` envelope (D4) —
   the envelope design has no collision case to test, since the original value is always nested as
   a value, never merged into a shared top level.
8. A memory id fixture missing the `mem_` prefix, or an unparseable `created_at`/`updated_at`, is
   rejected (D2/D3), not silently defaulted or truncated.
