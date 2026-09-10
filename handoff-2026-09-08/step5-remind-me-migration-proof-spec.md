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

### The existing, reusable export machinery (do not hand-roll a new exporter)

`remind_me_core/src/export.rs` already has a tested, production export path:
`collect_export_records` (`export.rs:311-343`) selects every `memories` column via
`crate::db::queries::prefixed_memory_columns`, deserializes into the real `Memory` struct, and
serializes with `serde_json::to_value` — i.e. the JSON shape this work order's importer must read
is exactly `Memory`'s serde output, not a hand-described approximation. `export_memories`
(`export.rs:367+`) wraps this with path validation and format selection (`render_export`,
`export.rts:351-360`, JSON or JSONL). This work order's fixtures are handwritten JSON modeled on
this exact struct shape (field names, types, `Option` nullability) rather than a live invocation of
`remind_me_core`, per D1 below — but the shape must match this real code, not a guess.

### `uc-memory`/`uc-facade`'s current Memory and Entity schemas (the import target)

`experiments/unified-commitment/crates/uc-facade/src/memory.rs`, `MemoryStore::schema()`: 13 fields
— `content (Str), category (Str), tags (StrList), source (Str), metadata_json (Str),
created_at_unix_ms (I64), updated_at_unix_ms (I64), memory_type (Str), status (Str), sensitive
(Bool), access_count (I64), deleted_at_unix_ms (I64), node_id (Str)`. `RecordId` is `[u8; 16]`
(`uc-protocol/src/types.rs:17`).

`experiments/unified-commitment/crates/uc-facade/src/entity.rs`, `EntityStore::schema()`: 4 fields
— `label (Str), kind (Str), mention_count (I64), aliases (StrList)`.

## Design decisions settled by the host

- **D1 — synthetic fixtures, not a live export.** No path dependency on `rusty_remind_me` is added
  (this project's own `AGENTS.md`/`ARCHITECTURE.md` in that repo documents exactly this mistake —
  path dependencies against a sibling repo that isn't guaranteed to exist at that relative location
  — as a removed, cautionary pattern; repeating it here would tie `rusty_data_os`'s build to a
  second repository's presence and exact layout). Fixtures are handwritten JSON files committed
  under `experiments/unified-commitment/crates/uc-facade/tests/fixtures/remind-me/`, matching
  `Memory`'s exact serde field names/types/nullability (cited above) and both id formats
  (`mem_<uuid32hex>`, and entity ids as bare 12-hex-char strings). At least 20 synthetic memory
  records and 8 synthetic entities/relations, covering: every `Option` field both present and
  absent; `sensitive=true` and `false`; a `deleted_at`-set (tombstoned) record; a record whose
  `memory_type`/`status` are `None` (pre-#198 rows, per the `models.rs:93-97` comment); at least one
  entity name requiring case-fold normalization (e.g. two source aliases differing only in case,
  both resolving to the one deterministic id per `entity.rs:44-46`).
- **D2 — field mapping for the 13 directly-representable fields.** `content`/`category`/`tags`/
  `source`/`sensitive`/`access_count`/`node_id` copy directly (verbatim). `metadata` (a JSON object
  in `remind_me`) serializes to a JSON string for `metadata_json` (a `Str` field in `uc-memory`).
  `created_at`/`updated_at` (RFC 3339 strings) parse to Unix epoch milliseconds for
  `created_at_unix_ms`/`updated_at_unix_ms` — reject (fail the import for that record, do not
  silently default) an unparseable timestamp. `memory_type`/`status` (both `Option<String>` in
  `remind_me`, per the `models.rs:99-102` comment, because some legacy rows predate the columns):
  `None` maps to `"unclassified"`/`"active"` respectively — `remind_me`'s own documented defaults
  for those columns (`schema_tables.sql:64`'s `memory_type TEXT NOT NULL DEFAULT 'unclassified'`,
  `status TEXT NOT NULL DEFAULT 'active'` — i.e. this reproduces what a fresh row would already
  contain, not an invented default). `deleted_at` (`Option<String>`, a **soft-delete tombstone
  marker** in `remind_me`'s own model, per `models.rs:112-115`'s comment — distinct from an actual
  deleted row) maps to `deleted_at_unix_ms`: `None` → `0` (sentinel for "not deleted"; `0` is
  guaranteed unambiguous since epoch-ms `0` is 1970 and no real memory predates this project),
  `Some(ts)` → the parsed epoch-ms value. **Critically: a tombstoned `remind_me` record is imported
  as a live `uc-memory` record whose `deleted_at_unix_ms` is nonzero — it is `Change::Put`, never
  `Change::Delete`.** `uc-core`'s incarnation-tracked delete is a different concept (a real,
  no-longer-existing row) than `remind_me`'s soft-delete flag (a still-existing, filtered-out row);
  conflating them would make the imported record's history diverge from the recoverable source
  data, and `remind_me`'s own query layer already filters on `deleted_at IS NULL` at read time
  rather than relying on absence.
- **D3 — id mapping, both directions verified.** Memory: strip the `mem_` prefix (reject, do not
  silently accept, any id lacking it), parse the remaining 32 hex characters as `Uuid::simple`
  bytes (16 bytes, exact fit for `RecordId`). Entity: the source id is 6 bytes (12 hex chars) —
  right-pad with 10 zero bytes to fill `RecordId`'s 16 bytes (left-aligned, source bytes first, so
  two different 6-byte source ids can never collide after padding). The original id string (for
  both kinds) is preserved verbatim in the stash (D4) specifically so identity is round-trip
  verifiable independent of the byte-mapping scheme chosen here.
- **D4 — the 14 remaining `Memory` fields with no `uc-memory` counterpart are stashed losslessly,
  never dropped.** `id` (original string, D3), `capture_id`, `subject`, `predicate`, `object`,
  `superseded_by`, `decay_rate`, `vitality`, `base_weight`, `accessed_at`, `doc_id`, `chunk_index`,
  `remind_at`, `client`, `source_capture_id` nest under a single reserved key,
  `"_remind_me_migration_extra"`, inside the JSON object written to `metadata_json` (alongside the
  original `metadata` object's own keys at the top level — reject the fixture/import if the
  original `metadata` already contains a colliding `"_remind_me_migration_extra"` key, rather than
  silently overwriting it). This is what makes the round-trip in Proof item 3 exact: nothing in the
  27-field source record is unrecoverable from the imported `uc-memory` record.
- **D5 — Entity import: `aliases`/`kind`/`label` map directly (`name`→`label`); `mention_count` is
  *recomputed*, not copied.** `remind_me`'s `entities` table has no `mention_count` column at all
  (it is derived elsewhere, e.g. from `memory_entities` join counts) — `uc-entity`'s `mention_count`
  is populated by counting the imported `memory_entities` rows referencing that entity after all
  memories are imported, not treated as a lossy field mapping. `entities.created_at`/`updated_at`/
  `node_id` have no `uc-entity` field to hold them; stash them the same way as D4 describes for
  Memory, using an analogous reserved key inside... **`uc-entity` has no metadata/JSON-string field
  at all to stash into** (its schema is exactly `label/kind/mention_count/aliases`, per the cited
  schema above) — so these three fields are recorded in the comparison harness's own retained
  export-fixture copy (not inside `uc-entity` itself) and the proof's comparison step verifies them
  against that copy rather than against anything read back from `uc-entity`. Disclosed, not
  silently dropped: `uc-entity`'s schema genuinely has no home for them today; extending it is out
  of scope for this proof (Non-goals).
- **D6 — `entity_relations` import uses `uc-relation`, not `uc-entity`'s own link mechanism.**
  `remind_me`'s `entity_relations` (`subject_entity_id, relation, object_entity_id`) is a
  Relation-domain concept, matching `uc-relation`'s existing purpose from Step 4a — reuse it
  directly rather than inventing a new mechanism. `memory_entities` (the memory→entity mention
  join) maps onto Step 4c's real `"mentions"` foreign edge (`uc-memory::LinkForeign`) — this is
  exactly the mechanism Step 4c built and is the reason this proof depends on Step 4c's close
  rather than an earlier step.

## Required changes

**R1 — new test fixtures.** `experiments/unified-commitment/crates/uc-facade/tests/fixtures/
remind-me/memories.json` and `entities.json` (D1), committed, hand-written to match the cited
`remind_me_core` shapes exactly.

**R2 — a new integration test suite, `uc-facade/tests/remind_me_migration.rs`.** Not a standalone
binary (D1 already ruled out any dependency on `remind_me_core`'s code, so there is no real
consumer for a general-purpose importer binary yet — a test suite proves the mechanism without
speculatively building a tool with no second caller). Steps, all over a real socket (matching this
project's established real-TCP testing convention, Steps 4b-ii/4c):

1. Read the fixtures (R1), apply the D2-D6 mapping in test code, and submit each resulting record
   as a real `Insert`/`Link` over the wire to a fresh `uc-facade` listener (Memory, Entity, and
   Relation tables registered together, matching Step 4c's registry wiring).
2. Read every record back (`Get`, `Join`, `neighbors_by_relation("mentions")`) and assert the D2-D6
   mapping round-trips exactly, including recovering every D4/D5-stashed field from
   `metadata_json`/the retained fixture copy and confiring it matches the original.
3. Checkpoint and reopen (matching Step 4c's own replay proof) — re-verify every comparison from
   step 2 against the reopened store.
4. Copy the store's data files to a second directory (a file-level backup, per the plan's own
   "create and restore a backup into another directory" language) and reopen *that* copy — re-verify
   again. Confirm the original directory is untouched by this (a real restore-from-backup proof, not
   just a second read of the same files).
5. Record counts, sorted per-record digests (SHA-256 over each record's canonical field tuple) for
   both the fixture source and the final reopened-from-backup store, and assert they match exactly
   — the plan's own literal "sorted per-record digests" comparison method (Step 5, sub-step 3).

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

## Proof

Same eleven-command chain as every prior Step 4/5 increment (unified-commitment fmt/clippy/test;
convergence-memory fmt/clippy/test; exp-0001 fmt/clippy/test, harness excluded; markdown links;
`git diff --check`), via `cargo +1.89.0-x86_64-pc-windows-gnu`, exit 0 required. Additional required
test coverage, named explicitly in the implementation report:

1. Every fixture memory record round-trips through insert → real-socket read → checkpoint/reopen →
   file-backup/restore-reopen with all 27 original fields recoverable (13 direct, 14 stashed) —
   including the tombstoned (`deleted_at`-set) record staying a live, gettable `uc-memory` row with
   its soft-delete marker intact (D2), and the pre-#198 record with `None` `memory_type`/`status`
   landing exactly on `remind_me`'s own documented column defaults.
2. Entity import: two differently-cased aliases of the same source name resolve to the one
   deterministic entity id (D3's padding scheme applied to `entity.rs:44-46`'s id), matching
   `remind_me`'s own case-fold-then-hash identity rule, not a naive per-alias id.
3. `mention_count` after import equals the real count of imported `memory_entities` edges
   referencing that entity (D5) — not a copied, absent, or default value.
4. `entity_relations` import produces real `uc-relation` records queryable by `subject`/`relation`/
   `object`, and `memory_entities` import produces real `"mentions"` foreign edges (Step 4c's
   mechanism) queryable by `Join`/`neighbors_by_relation` in both directions.
5. Sorted per-record SHA-256 digests match exactly between the source fixtures and the final
   reopened-from-backup store (the plan's own literal comparison method).
6. A record whose `metadata` object already contains the reserved `"_remind_me_migration_extra"`
   key is rejected by the importer (D4), not silently overwritten.
7. A memory id fixture missing the `mem_` prefix, or an unparseable `created_at`/`updated_at`, is
   rejected (D2/D3), not silently defaulted or truncated.
