# Step 4b-ii work order — Memory, Entity and Relation on the protocol facade, and a real listener

## Source plan

[Merge plan step 4](../data-os-multimodal-merge-plan-2026-09-08.md), first sentence ("Port Memory,
Entity and Relation first") and paragraph 2 (the compatibility facade). This is 4b-ii of Step 4b:
wiring the three domains already ported onto the unified core (`uc-memory` from EXP-0003,
`uc-entity`/`uc-relation` from EXP-0004) onto the protocol facade built in 4b-i (EXP-0005:
`uc-protocol`'s `Store` trait, dispatch, and connection loop), and opening the facade's first real
`TcpListener`. D2/D4 of the 4b-i spec explicitly deferred both to this follow-on.

## Owner scope decision (host, 2026-09-09)

All three domains together, not split further: 4b-i's connection/dispatch/session layer needs zero
changes to accept a real `Store` (verified below — session overlay logic operates only on the
`Fields` a `Store::get` returns, never inspecting domain internals), so the remaining work per
domain is "one adapter implementing one trait," and Entity/Relation are each markedly smaller than
Memory (4 and 7 data fields respectively, vs. Memory's 13, and neither has Memory's
read-your-writes/session-overlay stakes).

**Revised after Codex review round 1 (2026-09-09):** the original version of this paragraph also
argued for bundling all three domains so Memory's real `mentions` relation to Entity could be
exercised end-to-end. Review found that structurally impossible on the frozen design — see the
scope correction in "Goal" and D6 below. The three-domains-together rationale above still holds on
its own (adapters are small and mechanical once the pattern is set); the cross-table justification
does not, and is withdrawn.

## Target repository

Worktree `C:/dev/rusty_data_os-step4bii`, branch `codex/merge-step4bii-memory-domain`, base
`5af480971b006b3f04ac14fc30127a0361f371ae` (Step 4b-i's closing commit). Initial `git status` is
clean.

## Goal

Three new `Store` implementations in a new lib crate `uc-facade` (depends on `uc-protocol`,
`uc-memory`, `uc-entity`, `uc-relation`, `uc-core`, and `cm-trace` — all existing workspace
members, no new external dependency), each wrapping its domain engine behind a `Mutex` and mapping
`uc-protocol`'s generic wire types to that engine's real `Change`/`State`. A `Registry` (from 4b-i)
constructed with all three, served over a real `TcpListener` bound to `127.0.0.1:0` (OS-assigned
port, loopback only — no public exposure, no new capability beyond what 4b-i's D2 already named
as the deferred "thin wrapper"). An end-to-end integration test drives real TCP traffic through
`uc_protocol::framing`/`codec` (not the legacy crate — still no dependency on
`rusty_multimodal_db`, matching 4b-i's D1) covering each domain's own **same-table** operations
(including Entity's own open-label same-table links, which stay entirely within Entity's own
state and need nothing further) — all over one real socket.

**Scope correction after Codex review round 1 (2026-09-09):** the real cross-table `mentions`
`Link`/`Join`/detach-cascade between Memory and Entity, and a session staging writes across two
domains at once, are **out of scope for this work order** (see "Cross-table `mentions` is
deferred, not descoped by accident" below). Review found both structurally impossible on the
frozen 4b-i/EXP-0003/EXP-0004 design: `uc-memory`'s `Link` requires both endpoints to already
exist in Memory's *own* `State` (it was built for same-table edges only, like `uc-entity`'s
open-label links — never a foreign-table edge), and `uc-protocol`'s connection loop is single-table
by construction (no table selector on any staged op, `Use` guarded `SessionOpen` mid-session).
Building a real Memory↔Entity foreign edge durably (surviving restart, replaying correctly) is the
already-named `MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY` follow-on in
[`docs/roadmap/ROADMAP.md`](../../docs/roadmap/ROADMAP.md) — this work order does not attempt it,
and does not claim the `mentions` relation works end-to-end. It remains exactly as deferred as it
already was.

## Repository facts

### `uc-protocol::Store` — the exact trait this work order implements (already built, 4b-i)

`experiments/unified-commitment/crates/uc-protocol/src/store.rs`, full trait verified by direct
read (required methods, no default): `get`, `filter_eq`, `scan_field`, `update_field`, `parent`,
`children`, `neighbors`, `neighbors_by_relation`, `list_relation_kinds`, `describe`, `validate_op`,
`scan_all`, `apply_transaction`. Legacy-matching defaults (`Err(Unsupported)` unless overridden):
`insert_record`, `link_records`, `replace_record`, `replace_record_if`, `delete_record`,
`detach_record`, `compact`, `count_edges`, `write_batch_checked`. Non-trivial defaults:
`describe_relations`, `table_name`, `page`, `page_keys`, `write_batch`/`apply_write_op`.
`Fields = Vec<(FieldRef, ScanValue)>`, `RecordId(pub [u8; 16])` (`types.rs:17`).

### The session/connection layer needs nothing domain-specific from `Store`

`connection.rs`'s `GetById` handling (verified by direct read) calls `store.get(id)` once, then
performs read-your-writes overlay and snapshot-isolation tracking purely against the returned
`Fields` — matching `(FieldRef, value)` pairs and comparing `std::mem::discriminant` for type
safety, never touching anything domain-specific. No change to 4b-i's crate is required or
in scope.

### `Log::commit`/`XEngine::transact` require `&mut self` — every adapter needs a `Mutex`

`uc-core::Log::commit(&mut self, ...)` (`uc-core/src/lib.rs:337`) and, mirroring it,
`MemoryEngine::transact(&mut self, ...)`, `EntityEngine`/`RelationEngine`'s equivalents all take
`&mut self`. `Store`'s methods are all `&self`. Every adapter must hold its engine behind
`Mutex<XEngine>`, locked for the duration of any method that reads or writes — one lock per table,
mirroring this workspace's and legacy's own "hold exclusive access for the operation's entire
duration" philosophy (`GenericProductionStore::with_exclusive`'s own doc comment, quoted in the
4a/4b-i specs), simplified to one `Mutex` rather than legacy's finer-grained `RwLock` split between
reads and writes. This is a disclosed simplification (correctness first, per AGENTS §5), not a
performance claim; legacy's own read/write lock split is not reproduced here.

### Field-tag and value-kind mapping — verified 1:1 against legacy's real `describe()` bodies

**Memory** (`src/server/memory.rs:39-53` tags, `:820-871` schema; `cm_trace::Memory` fields
already built "Wire-equivalent," `cm-trace/src/lib.rs:11-25,47-50`):

| tag | name | `ValueKind` | `cm_trace::Value` variant | capabilities |
|---|---|---|---|---|
| 0 | content | Str | Text | read-only |
| 1 | category | Str | Text | filter_eq |
| 2 | tags | StrList | Tags | read-only |
| 3 | source | Str | Text | read-only |
| 4 | metadata_json | Str | Text | read-only |
| 5 | created_at_unix_ms | I64 | Int | read-only |
| 6 | updated_at_unix_ms | I64 | Int | read-only |
| 7 | memory_type | Str | Text | read-only |
| 8 | status | Str | Text | read-only |
| 9 | sensitive | Bool | Bool | read-only |
| 10 | access_count | I64 | Int | scan+update |
| 11 | deleted_at_unix_ms | I64 | Int | read-only |
| 12 | node_id | Str | Text | read-only |

`ScanValue::Str(s) <-> Value::Text(s)`, `::StrList(v) <-> Value::Tags(v)`, `::I64(i) <-> Value::Int(i)`,
`::Bool(b) <-> Value::Bool(b)`. `ScanValue::U32`/`::F64` never appear for a real Memory field.
Relation: `mentions`, `neighbors: true`, `target_table: Some("entity")`
(`MEMORY_FOREIGN_TABLE`, `generic/memory.rs:170`), symmetric (`uc-memory`'s `Change::Link` is
already unlabeled/single-relation, matching legacy's one-relation-only domain).

**Entity** (`src/server/entity.rs:65-69` tags, `:668-722` schema; `uc-entity::Entity`,
`crates/uc-entity/src/lib.rs:14-20`):

| tag | name | `ValueKind` | Rust field | capabilities |
|---|---|---|---|---|
| 0 | label | Str | `label: String` | filter_eq |
| 1 | kind | Str | `kind: String` | filter_eq |
| 2 | mention_count | I64 | `mention_count: i64` | scan+update |
| 3 | aliases | StrList | `aliases: Vec<String>` | read-only |

Relations: open-label (`known_labels`), `neighbors: true`, `target_table: None` for every one of
them (same-table only, matching legacy's own `RelationDescriptor { target_table: None, .. }` for
Entity's links — no cross-table declaration on Entity's side; Entity is the *target* of Memory's
`mentions`, never a foreign-declaring source itself).

**Relation** (`src/server/relation.rs:27-35` tags, `:606-650` schema; `uc-relation::Relation`,
`crates/uc-relation/src/lib.rs:12-21`):

| tag | name | `ValueKind` | Rust field | capabilities |
|---|---|---|---|---|
| 0 | subject | Str | `subject: String` | filter_eq |
| 1 | relation | Str | `relation: String` | read-only |
| 2 | object | Str | `object: String` | read-only |
| 3 | created_at_unix_ms | I64 | `created_at_unix_ms: i64` | read-only |
| 4 | updated_at_unix_ms | I64 | `updated_at_unix_ms: i64` | scan+update |
| 5 | node_id | Str | `node_id: String` | read-only |
| 6 | deleted_at_unix_ms | I64 | `deleted_at_unix_ms: i64` | read-only |

No relation layer at all (`RelationCapabilities { parent_children: false, neighbors: false }`,
`describe_relations()` empty) — matches ADR-0058's "nothing checks that an endpoint names an
entity."

### The three engines' `Change` shapes (already built; the adapters translate `Store` calls into these)

`uc-memory::Change` (`Put{record,incarnation,insert}`, `Update{id,incarnation,field: usize,
value,equals: Option<Value>}`, `Delete{id,incarnation}`, `Link{from,from_incarnation,to,
to_incarnation}` — unlabeled, single relation). `uc-entity::Change` (`Put{...}`,
`UpdateMentionCount{id,incarnation,value: i64}`, `Delete{...}`, `Link{relation: String, left,
left_incarnation, right, right_incarnation}` — open-label). `uc-relation::Change` (`Put{...}`,
`UpdateTimestamp{id,incarnation,value: i64}`, `Delete{...}` — no `Link` variant at all).

### `ReplaceIf`'s guard cannot be expressed as `Change::Update`'s `equals` alone

`Change::Update`'s `equals: Option<Value>` is exact-match only (a CAS on one field), but
`Request::ReplaceIf`'s `guard: Predicate` carries an arbitrary `CompareOp`
(`Eq`/`Ne`/`Lt`/`Le`/`Gt`/`Ge`) against any field, evaluated against the **whole current record**
before a **whole-record replace** — e.g. the consumer's `guard = { updated_at, Lt, my_updated_at }`
last-writer-wins merge. Do not attempt to encode this into the log's own `equals` field. Instead:
under the adapter's own `Mutex` (already held for the operation), read the current record, evaluate
the `Predicate` in ordinary Rust against its current field value (mirroring `CompareOp::is_ordering`
gating `Lt`/`Le`/`Gt`/`Ge` to `I64`/`U32`-kind fields only, `Malformed` otherwise — this validation
already happens in `uc-protocol::query`'s shared predicate evaluation, reuse it, do not
reimplement it), and only if it holds, submit a `Change::Put{insert: false, ..}` for the whole
record. Atomicity comes from the `Mutex` covering "read current + evaluate guard + submit
change" as one critical section, not from anything inside the log codec.

### Opening a real listener — the exact shape 4b-i's D2 deferred

`serve.rs:2474-2522` (`handle_connection`) and `:3011-3029`/`:2482-2510` (`serve_tables`, already
cited in the 4b-i spec) describe the target shape: bind, `for incoming in listener.incoming()`,
spawn one thread per accepted connection, hand it the shared `Registry`. `uc-protocol::connection`
already implements the per-connection state machine over any `Read + Write` (4b-i, D2) — this work
order's listener is exactly the "thin wrapper" D2 promised: `TcpListener::bind`, accept loop,
`thread::spawn` per connection calling the existing connection-handling entry point with a cloned
`Registry` and the accepted `TcpStream` split into a buffered reader/writer pair, mirroring
`framing::read_message`/`write_message`'s own `Read`/`Write` genericity (4b-i, R3).

## Design decisions settled by the host

- **D1 — one `uc-facade` crate for all three adapters plus the listener**, not three crates. They
  share nothing but the pattern (wrap an engine in a `Mutex`, translate `Store` calls to `Change`),
  and the listener needs all three constructed together to build one `Registry`.
- **D2 — `RecordId([u8;16])` converts to each engine's `Id`/`Uuid` by raw bytes**, never by
  re-parsing a string. `uc-protocol::RecordId(pub [u8;16])` and `cm_trace::Id = [u8;16]` convert
  by `.0` directly; `uc_core::Uuid`/`uc-entity`/`uc-relation`'s `Id` likewise (verify the exact
  conversion each engine expects — `Uuid::from_bytes`-equivalent — before assuming raw equality of
  representation; do not assume a specific byte order without checking, since `uc_core::Uuid`'s own
  encoding rules from Step 3/4a govern this, not this work order's).
- **D3 — incarnation is adapter-internal, never exposed on the wire.** None of `Request::Insert`/
  `Replace`/`UpdateField`/`Delete`/`Link` carry an incarnation; the legacy protocol has no such
  concept. Each adapter tracks the current incarnation per id internally (from the engine's own
  `State::incarnation`/`current`) and supplies it to every `Change` itself — a wire-level `Insert`
  always uses "next incarnation" (matching the engine's own `insert: true` semantics), and a
  wire-level `Replace`/`UpdateField`/`Delete`/`Link` always uses "the id's current live
  incarnation," which the adapter reads under its own `Mutex` immediately before submitting the
  `Change` (matching Steps 3/4a's own incarnation-tombstone design intent — a wire client should
  never need to know incarnations exist).
- **D4 — the listener binds `127.0.0.1:0` only, in this work order.** No configurable bind address,
  no TLS, no authentication beyond 4b-i's D4 (`Authenticate` always `Ok`). This stays a
  loopback-only, test-driven capability, not a deployable server — matching the narrow "thin
  wrapper" scope 4b-i's D2 named, and not overclaiming readiness for real exposure.
- **D5 — end-to-end verification uses `uc_protocol`'s own codec as the test client, never the
  legacy crate.** Proving "an existing client's real operations" against the *actual* legacy Rust/
  Python client binaries is out of scope here (it would mean building/running
  `rusty_multimodal_db` against this workspace, a cross-repo integration concern better suited to
  Step 5's "migrate one application"); this work order proves protocol-level and behavioral
  fidelity over a real socket using this crate's own already-fixture-verified codec, which is the
  honest, narrower claim.
- **D6 (host, after review round 1) — cross-table `mentions` is deferred, not descoped by
  accident.** `uc-memory`'s `Change::Link` (`crates/uc-memory/src/lib.rs:225-227`) requires
  `state.current(from, ..)` **and** `state.current(to, ..)` to both resolve inside Memory's own
  `State` — it has no notion of an edge whose far endpoint lives in a different table's `State`
  entirely, and there is no `Change::Detach`-equivalent variant to remove one edge without deleting
  the whole record. A real foreign edge would need either a new engine-level mechanism (a
  `Change` variant that trusts the write-time cross-table check and never re-verifies the far
  endpoint at replay, since `Log::recover` only replays `apply` against Memory's own history and
  has no way to consult Entity's) or a facade-owned side-structure with its own durability story —
  either is real, undecided design work, which is exactly what
  `MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY` in `docs/roadmap/ROADMAP.md` already names as deferred.
  This work order does not attempt it. `Registry`'s cross-table `Link`-existence-check and
  `Delete`-detach-cascade machinery from 4b-i stays real and tested (by its own synthetic
  test-double `Store`, from 4b-i) — this work order simply never calls it with two *real* domains
  at once, since no relation here declares a `target_table` between two of `MemoryStore`/
  `EntityStore`/`RelationStore` (`describe_relations()` reports `target_table: None` for all three
  in this work order, overriding the field-mapping tables' earlier `Some("entity")` note for
  Memory below — see the R2 correction).
- **D7 (host, after review round 1) — narrow, disclosed exceptions to "no change to uc-core/
  uc-memory/uc-entity/uc-relation/uc-protocol," each a single mechanical fix, not a design
  change:**
  - `uc-core`: `Log<S>`'s `writer: Box<dyn Writer>` and `hook: Box<dyn Fn(Point)>` fields
    (`crates/uc-core/src/lib.rs:198,209`; `pub trait Writer: Write` has no `Send` bound,
    `crates/uc-core/src/lib.rs:156`) block `Mutex<XEngine>: Send`, which blocks
    `Arc<XStore>: Send + Sync`, which `uc_protocol::Store: Send + Sync` requires for any adapter
    used by a multi-threaded listener (4b-i's own two-connection interleaving test already relies
    on a `Send` store — its synthetic test double happened to satisfy this trivially). Add
    `+ Send` to both trait-object bounds (`Box<dyn Writer + Send>`, `Box<dyn Fn(Point) + Send>`)
    and to every call site that constructs one. This is additive and non-behavioral: every real
    `Writer` implementation (file handles) is already `Send`; no existing test's semantics change.
  - `uc-memory`/`uc-entity`/`uc-relation`: the wire protocol's `MAX_STAGED_OPS`/`MAX_BATCH_OPS`
    are both 4096 (4b-i), but each engine independently caps one transaction at 1024 changes
    (`uc-memory/src/lib.rs:155` literal `1024`; `uc-entity`/`uc-relation`'s named
    `MAX_OPERATIONS: usize = 1024`, `crates/uc-entity/src/lib.rs:11`,
    `crates/uc-relation/src/lib.rs:7`). A legally-staged 1025-op session `Commit` would be accepted
    by the protocol layer and then rejected by the engine, an internal contradiction. Raise all
    three to 4096 (name a `MAX_OPERATIONS: usize = 4096` constant in `uc-memory` to match the
    other two's existing convention). Do not chunk a transaction across multiple `transact` calls
    (that breaks the atomicity `apply_transaction` promises) — one raised constant, one `transact`
    call per commit, unchanged otherwise.
  - `uc-protocol`: `query.rs`'s `reduce` (`~line 322`) computes `Sum`/`Avg` with
    `.sum::<i64>()` under this workspace's `overflow-checks = true` profile setting
    (`experiments/unified-commitment/Cargo.toml:15,17,20`) — two ordinary `i64` field values whose
    sum exceeds `i64::MAX` panic the connection-handling thread instead of returning a response.
    This is a real, pre-existing defect in already-closed 4b-i infrastructure that only becomes
    operationally reachable once a real listener exposes `Aggregate` to arbitrary input (this work
    order's own point). Fix by accumulating in `i128` and returning `ScanValue::I64` only if the
    result fits (otherwise a disposition the implementer states explicitly and tests — e.g. an
    `ErrorCode::Malformed`-equivalent response — rather than silently wrapping or panicking);
    `Avg` casts the wide `i128` sum **and** the count to `f64` first and divides those two floats
    (`sum_i128 as f64 / count as f64`) — **never** `(sum_i128 / count) as f64` or any other integer
    division before the cast, which silently truncates the fractional part (confirmed against the
    existing `aggregate_count_sum_avg_extremes_groups_and_empty_identity` test,
    `uc-protocol/tests/dispatch.rs:455`, which already asserts a fractional result of `5.0/3.0` for
    values `3, -1, 3` — integer division would wrongly yield `1.0`). Preserve the existing
    empty-input identity (`0.0`) unchanged. Add explicit test coverage for a fractional, a
    negative, and an overflow-range `Avg`/`Sum` case. Log this fix in `BUILD-LOG.md` as a 4b-i
    residual closed here, not a new 4b-ii defect.

## Required changes

**R0 — the four narrow exceptions from D7, before anything else.** Apply the `Send` bound fix in
`uc-core`, raise all three engines' operation cap to 4096, and fix the aggregate overflow in
`uc-protocol`, exactly as D7 specifies, each as its own small, disclosed, mechanical change — not
folded silently into R2-R6's own diffs. Run each affected crate's existing test suite unchanged
afterward to confirm no existing behavior moved.

**R1 — new crate.** Add `crates/uc-facade` to `experiments/unified-commitment/Cargo.toml`'s
`members` list (current members: `uc-core`, `uc-memory`, `uc-entity`, `uc-relation`, `uc-harness`,
`uc-protocol`). `uc-facade` depends on `uc-core`, `uc-memory`, `uc-entity`, `uc-relation`,
`uc-protocol`, and `cm-trace` (a direct workspace-path dependency — `uc-memory` depends on
`cm-trace` privately and does not re-export `Memory`/`Value`, so `uc-facade` needs its own
dependency on the same existing workspace member to name and construct those types; this adds no
new external crate) — nothing else.

**R2 — `MemoryStore`.** `pub struct MemoryStore(Mutex<MemoryEngine>)` (or equivalent). Implements
every `Store` method per the field-mapping table above: `get`/`scan_all` read the current `State`
via the engine and convert `Memory.fields` to `Fields` field-by-field; `filter_eq` only for
`category` (`Malformed`/`Unsupported` for any other tag, matching legacy's `FIELD_CATEGORY`-only
capability); `scan_field`/`update_field` only for `access_count`, validated non-negative before
submitting `Change::Update{field: 10, equals: None, ..}`; `insert_record`/`replace_record` decode
all 13 fields from the wire `Fields`, validate every tag is present exactly once and every
`ScanValue` variant matches its `ValueKind`, and submit `Change::Put{insert: true/false, ..}`;
`replace_record_if` per the guard-evaluation citation above; `delete_record` submits
`Change::Delete` (cascades same-table edges automatically, per `uc-memory`'s own `apply`);
`neighbors`/`neighbors_by_relation`/`list_relation_kinds`/`count_edges` read `State.edges`;
**`parent`/`children` have no default in `Store` (they are required methods, `store.rs:17-18` —
verify this for every method named "no override"/"defaults apply" anywhere in R2-R4, do not
assume a method is optional without checking) and must explicitly return
`Err(ErrorCode::Unsupported)`, matching legacy's `MemoryConnectionStore::parent`/`children`
exactly** (Memory has no `ChildOf` relation); `link_records` validates `relation == "mentions"`
(`Malformed` otherwise) before submitting `Change::Link` (same-table only — see D6; this work
order never registers a second table whose records could actually satisfy `Change::Link`'s own
same-state existence check, so this path is implemented faithfully but only exercisable with two
Memory-table ids in this work order's own tests, not a real Memory→Entity edge); `describe`/
`describe_relations` return the literal schema from the mapping table above **with
`target_table: None`, not `Some("entity")`** (D6 supersedes the mapping table's earlier note — do
not declare a foreign target this work order cannot back); `table_name` returns `"memory"`;
`apply_transaction` maps each `TransactionOp` to a `Change::Update` and submits them as one
`transact` call, translating the engine's rejection into the `(usize, ErrorCode)` shape
`Store::apply_transaction` requires.

**R3 — `EntityStore`.** Same pattern, `pub struct EntityStore(Mutex<EntityEngine>)`, per the Entity
field-mapping table: `update_field`/`scan_field` only for `mention_count` →
`Change::UpdateMentionCount`; **`parent`/`children` explicitly return `Err(Unsupported)`**
(Entity has no `ChildOf` relation either — `RelationCapabilities { parent_children: false,
neighbors: true }`); `link_records` validates via the *same* `valid_relation_label` rule
`uc-entity` already enforces internally (do not duplicate the character-class check at the
adapter layer — let `EntityEngine::transact` reject an invalid label itself and translate its
error) before submitting `Change::Link{relation, ..}`; `table_name` returns `"entity"`;
`describe_relations` reports every label currently in `known_labels` (`EntityEngine`'s existing
`list_relation_kinds`-equivalent), each with `target_table: None`.

**R4 — `RelationStore`.** Same pattern, `pub struct RelationStore(Mutex<RelationEngine>)`, per the
Relation field-mapping table: `update_field`/`scan_field` only for `updated_at_unix_ms` →
`Change::UpdateTimestamp`. Relation has no relation layer at all
(`RelationCapabilities { parent_children: false, neighbors: false }`), but `parent`, `children`,
`neighbors`, and `neighbors_by_relation` are all **required** `Store` methods with no default
body (`store.rs:14-24`) — each must have an explicit implementation returning
`Err(ErrorCode::Unsupported)`; `list_relation_kinds` (also required, no default) returns `vec![]`;
`link_records`/`detach_record`/`count_edges` *do* have legacy-matching `Unsupported` defaults
(`store.rs:47-71`, verified) and need no override. `table_name` returns `"relation"`;
`insert_record`/`replace_record` submit `Change::Put`, relying on `RelationEngine::transact` to
enforce the four non-empty/non-negative checks from Step 4a's spec and translating its rejection.

**R5 — real listener.** A function (e.g. `pub fn serve(listener: TcpListener, registry: Registry)`)
in `uc-facade` (or a thin `src/bin/` example if that fits this workspace's convention better —
match whatever `uc-harness` already does for a binary target, check before choosing) that accepts
connections in a loop, spawns one thread per connection, and hands each the existing 4b-i
connection-handling entry point with the accepted stream and a cloned `Registry`. Bind
`127.0.0.1:0` in tests (D4) and read back the OS-assigned port via `TcpListener::local_addr`.

**R6 — end-to-end integration test.** Build one `Registry` with `MemoryStore`, `EntityStore`, and
`RelationStore` all registered (primary: memory), start `serve` in a background thread on an
ephemeral port, and connect a plain `TcpStream` driven by `uc_protocol::framing`/`codec` (D5) —
require, at minimum: `Hello` negotiation over the real socket; insert/get/replace/delete on each of
the three domains individually, over the real socket; `Use` switching the active table; a
same-table `Link`/`neighbors`/`neighbors_by_relation`/`list_relation_kinds` exercise on Entity's
own open-label relation (two real Entity ids, one real label — this is the one relation scenario
that *is* in scope, since it never crosses tables); a **pipelined** `WriteBatch` (`atomic: false`)
containing only same-table ops on one domain, which must succeed per-op via `apply_write_op`; and,
separately, an explicit test that a **nonempty atomic** `WriteBatch` (`atomic: true`) on any of the
three domains is refused `TransactionFailed(0, Unsupported)` — this is the correct, accepted
behavior given no adapter overrides `write_batch_checked` in this work order (its 4b-i fail-closed
default applies), not a gap to route around; do not write a test expecting a successful atomic
batch mutation on any of these three adapters. A single-table session (`BeginWith` with all three
flags, each domain exercised in its own separate session) staging updates and committing. Do
**not** attempt a cross-table `mentions` `Link`/`Join`/detach or a session spanning two domains —
both are out of scope per D6/the scope correction above; a test that happens to "pass" by hitting
an `Unsupported`/`SessionOpen` refusal instead of the real behavior is not evidence of anything and
should not be written.

## Non-goals

No change to `uc-core`, `uc-memory`, `uc-entity`, `uc-relation`, `uc-harness`, or `uc-protocol`
beyond the four narrow, disclosed exceptions in D7/R0 (each a single mechanical fix with its own
citation; nothing else in those crates changes). No real Memory↔Entity cross-table `mentions`
relation and no cross-domain session — both deferred, per D6, to a dedicated follow-on work order
that would first need to design the durable foreign-edge mechanism `ROADMAP.md` already names as
unsolved. No TLS, no configurable bind address, no authentication beyond 4b-i's existing
`Authenticate` stub (D4). No dependency on `rusty_multimodal_db`, no legacy client binary, no
cross-repo test (D5). No SQL parser. No Dog/Order/Employee/Reminder domain. No production
deployment claim, no performance/benchmark series. No change to CI (the existing `--workspace`
proof commands cover a new member crate).

## Proof

Same eleven-command chain as 4b-i (unified-commitment fmt/clippy/test; convergence-memory
fmt/clippy/test; exp-0001 fmt/clippy/test, harness excluded; markdown links; `git diff --check`),
via `cargo +1.89.0-x86_64-pc-windows-gnu`, exit 0 required. Plus: the R6 end-to-end test must be
named explicitly in the implementation report, and the report must state, in its own words, that
each domain's own same-table operations (including Entity's open-label relation) ran over a real
`TcpListener`-accepted socket, not an in-process call — this is what this work order exists to
prove that 4b-i's synthetic test double could not. The report must also explicitly confirm that no
cross-table `mentions` scenario and no cross-domain session were attempted or claimed, per D6.
