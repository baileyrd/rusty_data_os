# Step 4b work order — protocol-22 wire facade infrastructure (uc-protocol)

## Source plan

[Merge plan step 4](../data-os-multimodal-merge-plan-2026-09-08.md), paragraph 2: "Reuse suitable
indexes, relationship operations, structured query execution, Rust/Python clients, and protocol
fixtures. Preserve the reviewed protocol 22 behavior and its 66-fixture baseline where applicable...
A `rusty_multimodal_db` compatibility facade can retain familiar imports and APIs while delegating
to the shared engine." The same plan's "Working approach" section: "Briefly align active repository
instructions with that goal in the first relevant implementation change while preserving existing
research evidence and useful correctness requirements."

This is **4b-i of Step 4b**: the domain-agnostic wire codec, dispatch trait, and connection-handling
logic only. No Memory/Entity/Relation adapter is wired in this work order — that is 4b-ii, a separate
work order, once this one closes. This mirrors how Step 3 (`uc-core`) landed before Step 4a wired
Entity/Relation onto it.

## Owner decisions already made (2026-09-09)

1. **Location:** the facade is reimplemented inside `rusty_data_os`, never inside
   `rusty_multimodal_db`. No cross-repo dependency of any kind. `rusty_multimodal_db` is a read-only
   fact source for this work order; nothing in it is imported, vendored, or path-dependent.
2. **Governance:** `AGENTS.md` §3's server/networking prohibition and
   `docs/RESEARCH-ROADMAP.md`'s Phase 7 gate ("network protocol... only after the embedded core is
   sufficiently characterized") are both amended, now, as this work order's first required change
   (R0), to carve out a named, bounded exception for this migration-driven facade — per the merge
   plan's own "align active repository instructions... in the first relevant implementation change"
   instruction. This does **not** lift any other Phase 7/§3 restriction and does **not** change the
   project's Phase 1 status.

## Target repository

Worktree `C:/dev/rusty_data_os-step4b`, branch `codex/merge-step4b-protocol-facade`, base
`d90b39ce42f8a852c2a1729646def511239fbb14` (Step 4a's closing commit). Initial `git status` is
clean. All new source paths below resolve against this checkout.

## Goal

A new lib crate, `uc-protocol`, added to the existing `experiments/unified-commitment` workspace,
implementing:

- a hand-written wire codec (no `bincode`/`serde` dependency — see Design decision D1) that is
  byte-for-byte compatible with `rusty_multimodal_db`'s frozen protocol-22 conformance corpus;
- the complete `Request`/`Response` type surface and every type they reference;
- a domain-agnostic `Store` trait mirroring `rusty_multimodal_db`'s `ConnectionStore` trait's
  method surface, and a generic `dispatch` function routing a decoded `Request` to a `&dyn Store`
  (or `&impl Store`) and back to a `Response`;
- a shared, registry-wide `Registry` type (D7) and a per-connection protocol loop, generic over
  any `Read + Write` stream and constructed against that shared registry — **not** yet bound to a
  real `TcpListener`/`TcpStream` (Design decision D2) — implementing `Hello` negotiation,
  multi-table `Use`/`ListTables`, a real one-shot `Request::Transaction` path, sessions
  (`Begin`/`BeginWith`/`Commit`/`Rollback` with staged writes and all three `BeginWith` flag
  behaviors — read-your-writes, validate-on-stage, snapshot isolation with a tracked read set),
  and the full cross-table effect set a multi-table registry requires: `Link` endpoint-existence
  checking, `Delete`'s cross-table detach cascade, cross-table `Join`, and — serializing all three
  against every connection sharing the registry, not just the current one — one shared
  relationship mutex held across each request's entire cross-table span.

Verified by: decoding and re-encoding every one of the 66 real fixture lines copied from
`rusty_multimodal_db`'s `tests/fixtures/wire-vectors.txt` against both the host's independently
derived literal expected values and byte-exact round-trip (D6), plus a deterministic
two-connection interleaving test proving the shared relationship mutex prevents a dangling edge
(D7).

No real domain (Memory/Entity/Relation) is wired to `Store` in this work order. `uc-core`,
`uc-memory`, `uc-entity`, `uc-relation`, `uc-harness` are unchanged.

## Repository facts

All citations are against `rusty_multimodal_db` at `232b16ecb3fd89b318ae4730c8470ab6da52f330`
(2026-09-08 20:02:08 -0500), the local checkout's actual `HEAD`, which is later than the merge
plan's own pinned review commit `478eeda`; line numbers may drift a few lines from the plan's own
citations, matching the same caveat Step 4a's spec already recorded.

### Framing (outer layer)

`src/server/framing.rs:1-5`:

> Length-prefixed binary framing over any `Read`/`Write` (in practice, `std::net::TcpStream`): a
> 4-byte little-endian length, then that many `bincode`-encoded bytes.

`MAX_FRAME_BYTES: u32 = 16 * 1024 * 1024` (`framing.rs:19`). `write_message` (`framing.rs:60-69`)
writes the 4-byte little-endian `u32` length of the encoded payload, then the payload.
`read_message` (`framing.rs:82-92`) reads exactly 4 bytes as a little-endian `u32` length, then
reads exactly that many bytes and decodes them. Reject any length over `MAX_FRAME_BYTES`.

### Codec (inner layer) — the exact byte rules to reproduce by hand

`src/codec.rs:70-76`, the `bincode` configuration used throughout: `with_fixint_encoding()`,
`with_little_endian()`, `with_no_limit()`, `reject_trailing_bytes()`. Documented rule
(`codec.rs:23-34`) and pinned examples (`codec.rs:155-168`), confirmed independently against the
real fixture bytes during recon (see the four traced examples below):

- Every fixed-width integer (`u8`/`u16`/`u32`/`u64`/`i64`/`usize`) is encoded **little-endian, at
  its natural width** — never a varint. `u16` `0x1234` → `34 12`; `u32` `0xdeadbeef` → `ef be ad
  de`; `u64` `1` → `01 00 00 00 00 00 00 00`.
- `bool` is one byte (`0x00`/`0x01`).
- `f64` is 8 bytes of IEEE-754 bits, little-endian.
- An enum (including `Request`, `Response`, `ScanValue`, `ErrorCode`, `JoinRelation`, `WriteOp`,
  `WriteResult`, `ValueKind`, every enum below) encodes as its **0-based declaration-order variant
  index, as a fixed 4-byte little-endian `u32`**, then that variant's fields in declaration order
  — regardless of how few variants the enum has.
- The one exception: `Option<T>` encodes as a **single byte** discriminant (`0x00` = `None`,
  `0x01` = `Some` followed by `T`'s own encoding) — not the 4-byte tag every other enum uses.
  Confirmed at `protocol.rs:1299-1302`'s own comment on `Request::Query`'s `limit: Option<usize>`
  field: "the first `Option` field on this wire; bincode encodes it as one byte (`0x00`/`0x01`),
  not the four-byte variant index every other enum here uses, confirmed empirically rather than
  assumed."
- `String` and `Vec<T>` are both encoded as an **8-byte little-endian `u64` element/byte count**,
  then that many elements/bytes in order (raw UTF-8 bytes for `String`).
- A tuple (e.g. `(FieldRef, ScanValue)`) is its fields back to back in order, no extra framing.
- `Uuid` (used as `RecordId`) is treated as a 16-byte byte string: an 8-byte little-endian `u64`
  value `16`, then the 16 raw bytes of the UUID in its standard big-endian byte layout (i.e.
  `Uuid::as_bytes()` order — `Uuid::from_u128(1)` is 15 zero bytes then `0x01`). This is the same
  `Vec<u8>`-shaped length-then-bytes rule as `String`, just with a fixed length of 16.

Byte-for-byte traced examples (fixture file, `tests/fixtures/wire-vectors.txt`; hex is the payload
only, i.e. after the outer 4-byte frame-length prefix has already been stripped):

```
Request/FilterEq	1	010000000100000000002a000000
Request/Authenticate	1	080000000600000000000000733363723374
Response/RecordList	1	010000000200000000000000100000000000000000000000000000000000000000000001100000000000000000000000000000000000000000000002
Request/Insert	13	15000000100000000000000000000000000000000000000000000001010000000000000000000300000008000000000000006c61627261646f72
```

- `FilterEq`: `01000000` = variant index 1 (`Request::FilterEq` is the 2nd variant, 0-based 1) ·
  `0100` = `u16` field tag `1` · `00000000` = `ScanValue::U32`'s variant index 0 · `2a000000` =
  `u32` `42`.
- `Authenticate`: `08000000` = variant index 8 · `0600000000000000` = `u64` string length `6` ·
  `733363723374` = ASCII `s3cr3t`.
- `RecordList`: `01000000` = variant index 1 (`Response::RecordList`) · `0200000000000000` = `u64`
  vec length `2` · then two 24-byte `Uuid` encodings back to back (`10 00 00 00 00 00 00 00` length
  prefix + 16 raw bytes each).
- `Insert`: `15000000` = variant index 21 (0x15) · 24 bytes = the `RecordId` · `0100000000000000`
  = `u64` vec length `1` for `fields: Vec<(FieldRef, ScanValue)>` · `0100` = field tag `0`... — the
  full tuple is `0100` (tag) `03000000` (`ScanValue::Str`'s variant index 3) `0800000000000000`
  (string length 8) `6c61627261646f72` (ASCII `labrador`).

### `PROTOCOL_VERSION` and the `Hello` handshake

`protocol.rs:114`: `pub const PROTOCOL_VERSION: u32 = 22;`. Client side (`client.rs:829-889`): the
very first frame on a fresh connection is `Request::Hello { protocol_version: PROTOCOL_VERSION }`.
Server side (`serve.rs:2508-2522`):

```rust
if let Request::Hello { protocol_version } = &req {
    let resp = if !first_frame || *protocol_version == 0 {
        err_response(ErrorCode::Malformed)
    } else {
        negotiated = (*protocol_version).min(PROTOCOL_VERSION);
        Response::Hello { protocol_version: negotiated }
    };
    first_frame = false;
    if !send_response(&mut writer, &resp) { return; }
    continue;
}
first_frame = false;
```

Negotiation is **always `min(client, server)`**, confirmed directly in that line. Nothing closes
the connection on a version mismatch. The only two error paths (`Response::Err{Malformed,..}`,
connection left open) are `protocol_version == 0` or `Hello` sent when it is not the connection's
first frame. A connection that never sends `Hello` at all is served at version 1 implicitly.

### `Request`/`Response` — complete variant list, verbatim field shapes, 0-based indices

`protocol.rs:570-954` (`Request`, 32 variants, indices 0–31) and `protocol.rs:957-1073`
(`Response`, 21 variants, indices 0–20):

```rust
pub enum Request {
    GetById { id: RecordId },                                                          // 0
    FilterEq { field: FieldRef, value: ScanValue },                                     // 1
    ScanField { field: FieldRef },                                                      // 2
    UpdateField { id: RecordId, field: FieldRef, value: ScanValue },                     // 3
    Parent { id: RecordId },                                                            // 4
    Children { id: RecordId },                                                          // 5
    Neighbors { id: RecordId },                                                         // 6
    DescribeSchema,                                                                     // 7
    Authenticate { token: String },                                                     // 8
    Transaction { updates: Vec<TransactionOp> },                                        // 9
    Hello { protocol_version: u32 },                                                    // 10
    Begin,                                                                              // 11
    Commit,                                                                             // 12
    Rollback,                                                                           // 13
    BeginWith { flags: u32 },                                                           // 14
    Query { select: Selection, filter: Vec<Predicate>, limit: Option<usize> },          // 15
    Aggregate { group_by: Vec<FieldRef>, filter: Vec<Predicate>,
                aggregates: Vec<AggregateSpec>, limit: Option<usize> },                  // 16
    NeighborsByRelation { id: RecordId, relation: String },                             // 17
    ListRelationKinds,                                                                  // 18
    Join(JoinSpec),                                                                     // 19
    DescribeRelations,                                                                  // 20
    Insert { id: RecordId, fields: Vec<(FieldRef, ScanValue)> },                        // 21
    Link { left: RecordId, right: RecordId, relation: String },                         // 22
    Replace { id: RecordId, fields: Vec<(FieldRef, ScanValue)> },                       // 23
    Use { table: String },                                                              // 24
    ListTables,                                                                        // 25
    Delete { id: RecordId },                                                            // 26
    Compact,                                                                            // 27
    ReplaceIf { id: RecordId, fields: Vec<(FieldRef, ScanValue)>, guard: Predicate },   // 28
    Page { order_by: FieldRef, after: Option<(ScanValue, RecordId)>, limit: u64 },      // 29
    CountEdges { relation: String },                                                    // 30
    WriteBatch { ops: Vec<WriteOp>, atomic: bool },                                     // 31
}

pub enum Response {
    Record { id: RecordId, fields: Vec<(FieldRef, ScanValue)> },                        // 0
    RecordList { records: Vec<RecordId> },                                              // 1
    ScanValues { values: Vec<ScanValue> },                                               // 2
    Id { id: RecordId },                                                                // 3
    Schema(DomainSchema),                                                               // 4
    NotFound,                                                                           // 5
    NoParent,                                                                           // 6
    Ok,                                                                                 // 7
    Err { code: ErrorCode, message: String },                                           // 8
    TransactionFailed { index: usize, code: ErrorCode, message: String },               // 9
    Hello { protocol_version: u32 },                                                    // 10
    Staged { index: u32 },                                                              // 11
    Rows { rows: Vec<(RecordId, Vec<(FieldRef, ScanValue)>)> },                         // 12
    Groups { groups: Vec<AggregateGroup> },                                             // 13
    RelationKinds { kinds: Vec<String> },                                               // 14
    JoinedRows { rows: Vec<JoinedRow> },                                                // 15
    Relations { relations: Vec<RelationDescriptor> },                                   // 16
    Tables { names: Vec<String>, primary: String },                                     // 17
    Compacted { records: u64, slots_reclaimed: u64,
                log_entries_folded: u64, edge_logs_folded: u64 },                        // 18
    Count { count: u64 },                                                               // 19
    BatchResults { results: Vec<WriteResult> },                                         // 20
}
```

### Supporting types referenced above — exact shapes

```rust
pub type RecordId = uuid::Uuid;      // reimplement without an external `uuid` dependency; see D3
pub type FieldRef = u16;

pub enum ScanValue { U32(u32), I64(i64), Bool(bool), Str(String), F64(f64), StrList(Vec<String>) }

pub struct TransactionOp { pub id: RecordId, pub field: FieldRef, pub value: ScanValue }

pub enum WriteOp {
    Insert { id: RecordId, fields: Vec<(FieldRef, ScanValue)> },
    Replace { id: RecordId, fields: Vec<(FieldRef, ScanValue)> },
    ReplaceIf { id: RecordId, fields: Vec<(FieldRef, ScanValue)>, guard: Predicate },
    Delete { id: RecordId },
    Link { left: RecordId, right: RecordId, relation: String },
}
pub enum WriteResult {
    Inserted, Duplicate, Replaced, NotFound, GuardFailed, Linked, AlreadyLinked, Deleted,
    Failed(ErrorCode),
}

pub enum Selection { All, Fields(Vec<FieldRef>) }
pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }
pub struct Predicate { pub field: FieldRef, pub op: CompareOp, pub value: ScanValue }

pub enum AggregateFn { Count, Sum, Avg, Min, Max }
pub struct AggregateSpec { pub func: AggregateFn, pub field: Option<FieldRef> }
pub struct AggregateGroup { pub key: Vec<(FieldRef, ScanValue)>, pub values: Vec<ScanValue> }

pub enum JoinRelation { Neighbors(Option<String>), Parent, Children }
pub struct JoinSpec {
    pub relation: JoinRelation, pub right_table: Option<String>,
    pub left: Selection, pub right: Selection,
    pub left_filter: Vec<Predicate>, pub right_filter: Vec<Predicate>,
    pub limit: Option<usize>,
}
pub struct JoinedRow {
    pub left_id: RecordId, pub left: Vec<(FieldRef, ScanValue)>,
    pub right_id: RecordId, pub right: Vec<(FieldRef, ScanValue)>,
}
pub struct RelationDescriptor { pub name: String, pub kind: JoinRelation, pub target_table: Option<String> }
pub enum ParentLookup { Parent(RecordId), NoParent, ChildNotFound }

pub enum ValueKind { U32, I64, Bool, Str, StrList }
pub struct FieldCapabilities { pub filter_eq: bool, pub scan: bool, pub update: bool }
pub struct FieldDescriptor { pub tag: FieldRef, pub name: String, pub value_kind: ValueKind,
                              pub capabilities: FieldCapabilities }
pub struct RelationCapabilities { pub parent_children: bool, pub neighbors: bool }
pub struct DomainSchema { pub fields: Vec<FieldDescriptor>, pub relations: RelationCapabilities }

pub enum ErrorCode {
    UnknownField, Unsupported, Malformed, Unauthenticated, Unauthorized, RecordNotFound,
    NoSession, SessionOpen, SessionFull, Journal, Conflict, Duplicate, Storage, GuardFailed,
}

pub type PageRow = (RecordId, Vec<(FieldRef, ScanValue)>);
```

`ErrorCode`'s 14 variants above are in the exact declaration order from `protocol.rs:487-570`
(`UnknownField, Unsupported, Malformed, Unauthenticated, Unauthorized, RecordNotFound, NoSession,
SessionOpen, SessionFull, Journal, Conflict, Duplicate, Storage, GuardFailed`) — this order is
part of the wire format (it is the enum's variant-index encoding) and must not be reordered.

### `ConnectionStore` trait and generic dispatch — method surface and match arms to mirror

`serve.rs:98-489`, `pub trait ConnectionStore: Send + Sync`. Required methods (no default):
`get`, `filter_eq`, `scan_field`, `update_field`, `parent`, `children`, `neighbors`,
`neighbors_by_relation`, `list_relation_kinds`, `describe`, `validate_op`, `scan_all`,
`apply_transaction`. Methods with a legacy-documented default answering `Err(Unsupported)`:
`insert_record`, `link_records`, `replace_record`, `replace_record_if`, `delete_record`,
`compact`, `count_edges`. Methods with a legacy-documented non-trivial default:
`describe_relations` (derives from `describe()` + `list_relation_kinds()`), `table_name`
(returns `"table"`), `page` (falls back to a full-scan `page_by_scan` helper).

`dispatch<S: ConnectionStore + ?Sized>(store: &S, req: Request) -> Response` at `serve.rs:2009`.
Representative match arms to reproduce exactly (full list of citations in the recon record; the
following establish the pattern for every domain-agnostic variant):

```rust
Request::GetById { id } => match store.get(id) {
    Some(fields) => Response::Record { id, fields },
    None => Response::NotFound,
},
Request::Insert { id, fields } => match store.insert_record(id, fields) {
    Ok(InsertOutcome::Inserted) => Response::Ok,
    Ok(InsertOutcome::Duplicate) => err_response(ErrorCode::Duplicate),
    Err(code) => err_response(code),
},
Request::Link { left, right, relation } => match store.link_records(left, right, &relation) {
    Ok(LinkOutcome::Linked | LinkOutcome::AlreadyLinked) => Response::Ok,
    Err(code) => err_response(code),
},
Request::Query { select, filter, limit } =>
    match validate_query(&store.describe(), &select, &filter) {
        Ok(()) => Response::Rows { rows: evaluate_query(store.scan_all(), &select, &filter, limit) },
        Err(code) => err_response(code),
    },
```

`Request::Authenticate`/`Hello`/`Begin`/`BeginWith`/`Commit`/`Rollback` are **never routed through
`dispatch`** in the real server — `handle_connection` intercepts them directly because they need
per-connection state a bare `&S` doesn't carry. The arms for them inside `dispatch` exist only as
an exhaustiveness fallback answering `ErrorCode::Unsupported`, documented as unreachable in the
real connection loop. **`Request::Transaction` is not in this set** — see the correction below;
`dispatch` has a real arm for it, and it is reachable through the real connection loop whenever no
session is open.

### Cross-table link checking (needed later for Memory→Entity, built now as domain-agnostic plumbing)

`serve.rs:3091-3146` (`link_across`/`check_link_across`): for a `Link`/batched-`Link` whose
relation descriptor names a `target_table`, the far endpoint must exist in that *other*
registered table (`RecordNotFound` otherwise; `Unsupported` if no such table is registered on
this connection) — checked by the connection-handling layer against the connection's whole
registered table set, not by any single `ConnectionStore` implementation. This is why `Use`/
`ListTables` (protocol 16, `serve.rs` multi-table state) and this check belong in the connection
loop, not in any one domain's dispatch.

### One-shot `Transaction` — a real dispatch arm, not a session-only fallback

`serve.rs`, the actual `dispatch` arm (not a fallback):

```rust
// A one-shot `Request::Transaction` has no session, so no
// snapshot-isolation read set to re-check — `ISO-FR-001` is
// exclusively a session (`BeginWith`) feature.
Request::Transaction { updates } => match store.apply_transaction(&updates, &[]) {
    Ok(()) => Response::Ok,
    Err((index, code)) => Response::TransactionFailed { index, code, message: error_message(code).to_string() },
},
```

The connection loop rejects `Transaction` with `SessionOpen` only while a session is currently
open on that connection (mirroring `Begin`'s own exclusivity rule); otherwise it reaches this real
dispatch arm. Only `Authenticate`, `Hello`, `Begin`, `BeginWith`, `Commit`, `Rollback` are the
per-connection-state variants dispatch answers with an `Unsupported` exhaustiveness fallback,
never reached through the real connection loop — `Transaction` is not one of them.

### `write_batch_checked` — the atomic-batch hook and its fail-closed default

`serve.rs` doc comment and body, verbatim:

```rust
/// Apply an atomic batch with server-resolved cross-table preconditions.
/// `check` runs for each op before local validation, in operation order.
/// The caller must keep the other tables stable until this returns.
/// The default refuses nonempty atomic batches at index 0 with
/// `Unsupported` without writing; empty batches succeed.
/// `dispatch` and `write_batch` are table-local low-level entry points;
/// the served path is `write_batch_across`, which adds registry-aware
/// Link checks and Delete cascades.
fn write_batch_checked(
    &self,
    ops: &[WriteOp],
    _check: &dyn Fn(usize) -> Result<(), ErrorCode>,
) -> Result<Vec<WriteResult>, (usize, ErrorCode)> {
    if ops.is_empty() { return Ok(Vec::new()); }
    Err((0, ErrorCode::Unsupported))
}
```

`write_batch(&self, ops, atomic)`'s two paths: pipelined (`atomic: false`) maps each op through
`apply_write_op` (a default that routes each `WriteOp` variant to its single-shot method —
`insert_record`/`replace_record`/`replace_record_if`/`delete_record`/`link_records` — exactly as
each variant "carries the exact body of the single-shot request it names"), continuing past a
per-op failure (`WriteResult::Failed(code)`); atomic (`atomic: true`) delegates entirely to
`write_batch_checked`, so a domain that does not override it correctly refuses the whole batch
before writing anything, rather than falling back to the (unsafe, partially-applying) pipelined
path.

`write_batch_across` (`serve.rs:3150-3220`), the connection-loop-level served path for `WriteBatch`,
adds registry-aware `Link` existence checks (via the same `check_link_across` used by single-shot
`Link`) before each atomic op's local validation, and, after a successful `Delete` (pipelined:
immediately per-op; atomic: after the whole batch's local apply succeeds, in `ops` order), calls
the same cross-table detach cascade described next — reporting a detach failure as
`WriteResult::Failed(code)` on that op without undoing the already-applied delete.

### Cross-table `Delete` cascade (`delete_across`/`detach_across`)

`serve.rs:3040-3080`, verbatim:

```rust
/// `DEL-FR-007` (ADR-0051): [`Request::Delete`] on a `serve_tables`
/// server — the table's own `delete_record` (which drops the record's
/// edges within that table), then, only on `Ok`, every *other* table's
/// `detach_record` for each of its relations whose `target_table` is
/// this one: the consumer's `DELETE FROM memory_entities WHERE
/// entity_id = ?`. An adapter answering `Unsupported`/`Malformed` for
/// the detach has nothing to drop and is skipped; a `Storage` failure
/// there is reported in the delete's place, the record itself already
/// gone (the one partial state, named in the design).
fn delete_across(tables: &[(String, Arc<dyn ConnectionStore>)], store: &dyn ConnectionStore, id: RecordId) -> Response {
    let resp = dispatch(store, Request::Delete { id });
    if resp != Response::Ok { return resp; }
    match detach_across(tables, store, id) {
        Ok(()) => Response::Ok,
        Err(code) => err_response(code),
    }
}

/// Detach in registration order, after the own-table delete has succeeded.
/// On Storage, the record and any earlier detaches remain applied.
fn detach_across(tables: &[(String, Arc<dyn ConnectionStore>)], store: &dyn ConnectionStore, id: RecordId) -> Result<(), ErrorCode> {
    let this = store.table_name();
    for (name, other) in tables {
        if name == this { continue; }
        for relation in other.describe_relations() {
            if relation.target_table.as_deref() != Some(this) { continue; }
            // detach_record(&relation.name, id) on `other`, continuing on Unsupported/Malformed
        }
    }
    Ok(())
}
```

### Shared relationship mutex — the cross-connection synchronization boundary

`serve.rs`, `relationship_mutex`/`relationship_section`/`serve_tables` doc comment, verbatim
(elided only where noted):

```rust
fn relationship_mutex(tables: &[(String, Arc<dyn ConnectionStore>)]) -> Option<Arc<Mutex<()>>> {
    tables.iter().any(|(_, store)| {
        store.describe_relations().iter().any(|relation| relation.target_table.is_some())
    }).then(|| Arc::new(Mutex::new(())))
}

fn relationship_section<'a>(lock: Option<&'a Mutex<()>>, request: &Request) -> Option<MutexGuard<'a, ()>> {
    if !matches!(request, Request::Link { .. } | Request::Delete { .. } | Request::WriteBatch { .. }) {
        return None;
    }
    lock.map(|lock| lock.lock().unwrap_or_else(|p| p.into_inner()))
}
```

> When any registered relation declares a target table, Link, Delete and WriteBatch share a
> relationship mutex acquired before adapter locks and held through cross-table checks, apply and
> detaches. Registries without foreign relations omit this mutex entirely. Adapter sections never
> overlap across tables; detaches visit tables in registration order. Only Delete removes records,
> so other requests cannot invalidate a successful far-endpoint existence check. Reads, other
> writes, sessions and Compact use adapter locks alone.

This is the piece that makes the existence check race-free **across connections**, not just within
one: the mutex is one `Arc<Mutex<()>>` shared by every connection spawned off the same
`serve_tables` call (one per listener, cloned into each spawned thread), acquired for the request's
*entire* cross-table span (existence check through apply through detach), not released and
re-acquired between steps.

### `BeginWith` flags — the three session behaviors, not just staged writes

`protocol.rs:117-139`, verbatim:

```rust
/// `Request::BeginWith` flag bit 0 (protocol 5, `RYW-FR-001`, ADR-0027):
/// the session's own point reads (`GetById`) see its staged writes.
/// Every other bit is unknown to this build and makes the request `Malformed`.
pub const SESSION_READ_YOUR_WRITES: u32 = 1;

/// `Request::BeginWith` flag bit 1 (protocol 6, `STV-FR-001`): every
/// `UpdateField` staged in the session is validated when it is staged —
/// `ConnectionStore::validate_op` — and refused, nothing staged, with the
/// code `Commit` would have reported for it. Unknown below protocol 6.
pub const SESSION_VALIDATE_ON_STAGE: u32 = 2;

/// `Request::BeginWith` flag bit 2 (protocol 7, `ISO-FR-001`, ADR-0033):
/// read-set validated, repeatable for read keys. A found `GetById` records
/// the first committed value of each tracked `(id, field)`; repeated reads
/// serve those values, even after deletion, then overlay read-your-writes.
/// Unread keys, absent records, and keys beyond `MAX_TRACKED_READS` are
/// not frozen; scans and relation reads remain live. This is not a full
/// database snapshot. `Commit` validates the original read set atomically
/// with applying the writes and refuses mismatches with `ErrorCode::Conflict`.
/// Unknown below protocol 7.
pub const SESSION_SNAPSHOT_ISOLATION: u32 = 4;
```

Bits combine freely (the fixture's `BeginWith(all three bits)` line uses `flags = 7`); any bit
outside `{1, 2, 4}` is `Malformed` and opens no session; a bit whose introducing protocol version
exceeds the connection's negotiated version is `Malformed` (rule 3's downgrade posture — this
crate does not implement per-version downgrading per D5, so treat any of these three bits as
always available once negotiated ≥ their own introducing version, and never available below it,
without needing general content-rewriting).

### Session-open guards — the full request set, and the two operation-count caps

`serve.rs:2790-2849`, verbatim excerpts (elided requests follow the identical `session.is_some()`
pattern):

```rust
Request::Transaction { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::Insert { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::Link { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::Replace { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::Delete { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::Compact if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::ReplaceIf { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
Request::WriteBatch { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
// `Use` inside a session is `SessionOpen`, since a session's staged
// writes belong to one table.
Request::Use { .. } if session.is_some() => err_response(ErrorCode::SessionOpen),
```

Every one of these checks runs, and rejects, **before** the request's own effect (a table switch
for `Use`, a mutation for the rest) — none of them ever reach the mutation/table-switch logic
while a session is open on that connection. `Begin`/`BeginWith` while a session is already open is
`SessionOpen` too (the same "one batch at a time per connection" rule already cited for `Begin`).
The full guarded set is therefore: `Begin`, `BeginWith`, `Transaction`, `Insert`, `Link`,
`Replace`, `Delete`, `Compact`, `ReplaceIf`, `WriteBatch`, `Use`. `Commit`/`Rollback` are the
inverse case (`NoSession` when no session is open) and are unaffected by this list.

Two operation-count caps, `protocol.rs:141-155`, verbatim:

```rust
/// The most `UpdateField`s one connection may stage between Begin and
/// Commit (`SESS-FR-004`): the `MAX_STAGED_OPS + 1`-th is answered
/// `ErrorCode::SessionFull` and not staged, the session staying open.
pub const MAX_STAGED_OPS: usize = 4096;

/// The most write operations one `Request::WriteBatch` may carry
/// (`WBT-FR-001`, protocol 22) — a batch over this is `Malformed`,
/// applying nothing.
pub const MAX_BATCH_OPS: usize = 4096;
```

Enforcement (`serve.rs`, verbatim): staging the `MAX_STAGED_OPS + 1`-th `UpdateField` answers
`ErrorCode::SessionFull` and does not stage it — the session stays open with its existing staged
writes intact, so a subsequent `Commit` still applies exactly what was successfully staged before
the cap was hit. A `WriteBatch` whose `ops.len() > MAX_BATCH_OPS` is rejected `Malformed` before
any op is applied, whether pipelined or atomic:

```rust
Request::WriteBatch { ref ops, .. } if ops.len() > MAX_BATCH_OPS => err_response(ErrorCode::Malformed),
```

### `Join` — self-join through `dispatch`, cross-table through the connection loop

`serve.rs:2863`: `Request::Join(spec) if spec.right_table.is_some() => join_across(tables, store, &spec)`
— the connection loop intercepts a `Join` **only when `right_table.is_some()`**, before it would
ever reach generic `dispatch`. When `right_table` is `None` (a self-join), the request falls
through to `dispatch`'s own `Join` arm (`serve.rs:2122`), which calls
`validate_join(&store.describe(), &store.describe_relations(), None, &spec)` — passing `None` for
the right-hand schema, since there is no other table to resolve.

`join_across` (`serve.rs:3011-3029`), the connection-loop-level implementation for
`right_table: Some(name)`: resolves `name` against the connection's registered table set, calls
`validate_join` again with the resolved right-hand schema, and on success evaluates the join
using the *left* adapter's relation traversal and the *right* adapter's `get`.

**`validate_join`'s exact error distinction — `Malformed` and `Unsupported` are not
interchangeable.** `serve.rs`, verbatim:

```rust
/// Both sides are validated exactly as `validate_query` does
/// (`UnknownField`/`Malformed`); a relation the adapter does not list is
/// `Malformed`; one it lists with a `target_table` (another table's
/// rows) is `Unsupported` while `right_table` is `None`.
fn validate_join(schema: &DomainSchema, relations: &[RelationDescriptor],
                  right_schema: Option<&DomainSchema>, spec: &JoinSpec) -> Result<(), ErrorCode> {
    validate_query(schema, &spec.left, &spec.left_filter)?;
    let relation = relations.iter().find(|r| r.kind == spec.relation).ok_or(ErrorCode::Malformed)?;
    match (&spec.right_table, &relation.target_table) {
        // Within one table: the right rows are this table's.
        (None, None) => validate_query(schema, &spec.right, &spec.right_filter),
        // The relation's rows live elsewhere and the caller did not say
        // where — `Unsupported`.
        (None, Some(_)) => Err(ErrorCode::Unsupported),
        // A cross-table join must name exactly the table the descriptor
        // names, and that table must be one this server registered
        // (`right_schema` is `Some`).
        (Some(named), Some(target)) if named == target => {
            let right = right_schema.ok_or(ErrorCode::Malformed)?;
            validate_query(right, &spec.right, &spec.right_filter)
        }
        (Some(_), _) => Err(ErrorCode::Malformed),
    }
}
```

So: an unknown relation name is always `Malformed`; a same-table relation (`target_table: None`)
with no `right_table` named validates against the connection's own schema; a cross-table relation
(`target_table: Some(t)`) with no `right_table` named is `Unsupported` (the caller didn't say
where); a `right_table` named that matches the relation's declared target but isn't actually
registered on this connection is `Malformed` (not `Unsupported`); a `right_table` named that does
not match the relation's declared target (or is named against a same-table relation at all) is
`Malformed`. These four outcomes are distinct wire codes, not two interchangeable "unresolved
target" cases.

## Design decisions settled by the host

- **D1 — no `bincode`/`serde` dependency.** Every codec in this workspace so far (`uc-core`'s RF1/
  envelope, `uc-memory`'s CMM2, `uc-entity`'s CME1, `uc-relation`'s CMR1) is a hand-written,
  dependency-free codec. `uc-protocol` follows the same convention: implement the byte rules quoted
  above directly with `std`, matching `bincode`'s documented fixint/little-endian/no-limit
  configuration byte-for-byte, without adding `bincode` or `serde` as a dependency. This also avoids
  needing crates.io network access inside the offline build sandbox.
- **D2 — no socket code in this work order.** The connection-handling loop (Hello negotiation,
  multi-table state, sessions, cross-table link checking) is generic over `Read + Write`, testable
  with an in-memory duplex pair (e.g. two ends of a pipe, or a paired in-memory buffer), and binds
  to no `TcpListener`/`TcpStream`. Opening a real socket is deferred to 4b-ii or later, at which
  point it is a thin wrapper (`TcpListener::accept` → hand the stream to this already-tested loop).
  This keeps the actual new "networking" surface in this specific work order at zero live sockets.
- **D3 — no external `uuid` crate dependency.** `RecordId` needs only: 16 raw bytes, an `Ord`/`Eq`
  impl, and the wire encoding above (length-16-prefixed byte string). Define a minimal
  `pub struct RecordId([u8; 16])` (or reuse `uc_core::Uuid` if its shape already fits — check before
  defining a new type; do not add a `uuid` crate dependency either way).
- **D4 — no enforced authentication.** `Request::Authenticate` always answers `Response::Ok`; no
  credential store exists anywhere in this workspace. This is a disclosed, deliberate limitation
  (real authentication/authorization is explicitly Phase 7 scope per the roadmap and is not
  established by this work order), not a security claim about anything this crate produces.
- **D5 — no protocol-version downgrading.** The server always negotiates `min(client, server)` per
  the Hello rule above and behaves identically regardless of the negotiated version (no
  `downgrade_for_version`-equivalent content rewriting for older negotiated versions). This is
  sufficient for the plan's "selected existing clients" (the current protocol-22 client) and is a
  disclosed non-goal for any hypothetical older-protocol client.
- **D6 — round-trip *and* literal-value fixture testing.** Round-trip alone (decode then
  re-encode, compare bytes) is insufficient: a decoder with a consistent but wrong byte-order
  reading, or with two same-shaped variants swapped (e.g. `Parent`/`Children`, whose payloads are
  both a bare `RecordId`), would still round-trip every fixture line byte-for-byte while decoding
  to the wrong semantic value. The host has independently decoded all 66 lines by hand-verified
  reference rules and recorded the exact expected value of every field for every line in
  `handoff-2026-09-08/step4b-fixture-expected-values.txt` (committed alongside this spec). The
  required proof is **both**: (a) decode each line's hex payload and assert it equals the literal
  expected value recorded there field-by-field (variant name and every field, not just successful
  parsing); (b) re-encode that decoded value and assert the result is byte-identical to the
  original hex. Strip any parenthetical suffix in a fixture's name column (e.g.
  `BeginWith(all three bits)`, `Err(Conflict)`) for reporting only — it never changes which
  decode/encode path runs or which literal value it is checked against.
- **D7 — one shared relationship mutex per registry, not per-connection or per-store locking.**
  `Link`/`Delete`/`WriteBatch` cross-table effects (existence check, own-table apply, cross-table
  detach) must be serialized across *every* connection sharing the same table registry, not just
  within one connection — otherwise connection A's existence check and connection B's concurrent
  delete-and-detach can interleave into a dangling edge (a fresh edge created after its target
  was already removed). Model the registry itself as a first-class type
  (`pub struct Registry { tables: Vec<(String, Arc<dyn Store>)>, relationship_lock: Option<Arc<Mutex<()>>> }`,
  the `Option` present only when some registered relation declares a `target_table`, mirroring
  `relationship_mutex`'s own opt-out for foreign-relation-free registries) that every connection
  handler is constructed against by reference/clone — never a private, per-connection lock. The
  lock is acquired once per `Link`/`Delete`/`WriteBatch` request, before any cross-table check,
  and held through that request's entire cross-table span (check → own-table apply → detach),
  exactly as `relationship_section` does. This must be provable with a two-connection interleaving
  test over two independent in-memory duplex pairs sharing one `Registry`, matching the
  requirement in R7 below.

## Required changes

**R0 — governance amendment (documentation only, do first).**

In `AGENTS.md` §3, after the existing "Do not add server/networking layers..." sentence, add:

> The Data OS + Rusty Multimodal DB merge plan's Step 4 compatibility facade
> (`docs/experiments/EXP-0005-protocol-facade.md`) is a named, bounded exception: a protocol-22
> wire-compatible facade under `/experiments/` is authorized now, without waiting for Research
> Roadmap Phase 7, scoped exactly as that experiment document states. This does not authorize any
> other server/networking work and does not change the project's current phase status.

In `docs/RESEARCH-ROADMAP.md`, immediately under the "## Phase 7 — Server adapter" heading, add:

> EXP-0005 (2026-09-09) is a bounded, migration-driven exception to this phase ordering: the merge
> plan's Step 4 compatibility facade for existing `rusty_multimodal_db` clients is authorized now,
> scoped to protocol-22 wire compatibility for the domains the merge plan ports. It does not advance
> this roadmap's Phase 2–6 status and authorizes no other server/networking work.

**R1 — new crate.** Add `crates/uc-protocol` to `experiments/unified-commitment/Cargo.toml`'s
`members` list (current members: `uc-core`, `uc-memory`, `uc-entity`, `uc-relation`, `uc-harness` —
verify unchanged). `uc-protocol`'s own `Cargo.toml` depends only on `uc-core` (for `Uuid` if it fits
D3's need, or nothing beyond `std` otherwise) — no other workspace member, no external crate.

**R2 — codec module.** Implement encode/decode for every type in "Supporting types" and the full
`Request`/`Response` enums, exactly matching the byte rules quoted above. Provide, at minimum:
`encode_request(&Request) -> Vec<u8>`, `decode_request(&[u8]) -> Result<Request, String>`,
`encode_response(&Response) -> Vec<u8>`, `decode_response(&[u8]) -> Result<Response, String>`
(payload bytes only — the outer frame-length prefix is a separate concern, R3). Decoding must
reject trailing bytes (matching `reject_trailing_bytes()`) and an out-of-range/unknown variant
index, with a `Malformed`-equivalent error, not a panic.

**R3 — framing module.** `write_message<W: Write>(&mut W, payload: &[u8]) -> io::Result<()>`
(4-byte little-endian length prefix, then the payload) and
`read_message<R: Read>(&mut R) -> io::Result<Vec<u8>>` (read the 4-byte length, reject over
`MAX_FRAME_BYTES = 16 * 1024 * 1024`, read exactly that many bytes). Generic over any `Read`/`Write`,
not `TcpStream`-specific.

**R4 — fixture conformance test.** Copy `rusty_multimodal_db`'s `tests/fixtures/wire-vectors.txt`
verbatim (all 70 lines, including the 4 header comments) into
`crates/uc-protocol/tests/fixtures/wire-vectors.txt`, with one added header comment recording its
provenance (`rusty_multimodal_db@232b16ecb3fd89b318ae4730c8470ab6da52f330`,
`tests/fixtures/wire-vectors.txt`, copied 2026-09-09). Also copy
`handoff-2026-09-08/step4b-fixture-expected-values.txt` (the host's independently-derived literal
decode of every line — see D6) into `crates/uc-protocol/tests/fixtures/` alongside it. Write a
test that parses every one of the 66 data lines (`name<TAB>version<TAB>hex`) and, per D6:

1. decodes the hex bytes with `decode_request` (if the name starts `Request/`) or
   `decode_response` (if `Response/`), and asserts every field of the decoded value equals the
   literal value recorded for that line in `step4b-fixture-expected-values.txt` — variant identity
   and every field, not merely "decoding did not error";
2. re-encodes that decoded value and asserts the result equals the original hex bytes exactly.

All 66 lines must pass both checks. This is the work order's central acceptance criterion — treat
any line that fails either check as a codec defect to fix, not a fixture to skip.

**R5 — `Store` trait.** Mirror `ConnectionStore`'s method surface from the citations above
(required methods, and the same legacy-documented defaults for the methods that have them),
including — the review found these missing from the original inventory —
`detach_record(&self, relation: &str, id: RecordId) -> Result<usize, ErrorCode>` (default
`Err(ErrorCode::Unsupported)`) and `apply_write_op(&self, op: &WriteOp) -> WriteResult` (default:
route each `WriteOp` variant to its single-shot method — `insert_record`/`replace_record`/
`replace_record_if`/`delete_record`/`link_records` — mapping that method's `Ok`/`Err` to the
matching `WriteResult` variant, exactly as each `WriteOp` variant "carries the exact body of the
single-shot request it names"). Also require:

- `write_batch(&self, ops: &[WriteOp], atomic: bool) -> Result<Vec<WriteResult>, (usize, ErrorCode)>`:
  `atomic: false` maps every op through `apply_write_op`, continuing past a per-op failure
  (`WriteResult::Failed(code)`), never aborting the batch; `atomic: true` delegates entirely to
  `write_batch_checked` below — do **not** fall back to the pipelined path for `atomic: true`,
  since that can apply an earlier op and then fail a later one, violating the atomic contract.
- `write_batch_checked(&self, ops: &[WriteOp], check: &dyn Fn(usize) -> Result<(), ErrorCode>) -> Result<Vec<WriteResult>, (usize, ErrorCode)>`,
  with the exact fail-closed default quoted above (`Ok(Vec::new())` for an empty batch,
  `Err((0, ErrorCode::Unsupported))` for any nonempty batch) — a domain that has not overridden
  this method must refuse every nonempty atomic batch, applying nothing, not silently downgrade to
  pipelined execution.

**R6 — generic `dispatch`.** `dispatch<S: Store + ?Sized>(store: &S, req: Request) -> Response`
covering every domain-agnostic `Request` variant per the match-arm citations above, specifically
including a **real** arm for `Request::Transaction { updates }` calling
`store.apply_transaction(&updates, &[])` (empty read-set — a one-shot transaction has no session)
and mapping its `Ok(())`/`Err((index, code))` to `Response::Ok`/`Response::TransactionFailed`
exactly as quoted above, and a **real** arm for `Request::Join(spec)` calling `validate_join`'s
exact 4-way match (quoted verbatim above) with `right_schema` always `None` (bare dispatch has no
other table to resolve). Implementing that match faithfully, without simplifying it, is what
produces the correct code in every case reaching generic dispatch directly: `right_table: None`
against a same-table relation validates and evaluates the self-join; `right_table: None` against a
relation whose descriptor names a `target_table` is `Unsupported`; **any** `right_table: Some(_)`
reaching bare dispatch (i.e. one the real connection loop did not intercept — see R7) is
`Malformed`, whether or not the named table matches the relation's declared target, because bare
dispatch never has a resolved `right_schema` to supply. Do not special-case `right_table: Some`
as a blanket `Unsupported` — that is the exact confusion the second review round flagged.
`Authenticate`/`Hello`/`Begin`/`BeginWith`/`Commit`/`Rollback` are the **only** variants answered
by dispatch's exhaustive `Unsupported` fallback (per-connection state a bare `&S` cannot carry) —
`Transaction` is explicitly excluded from that fallback set.

**R7 — connection loop and shared registry.** Define the `Registry` type from D7
(`tables: Vec<(String, Arc<dyn Store>)>`, an `Option<Arc<Mutex<()>>>` relationship lock present
only when some registered relation declares a `target_table`, and a primary/default table index).
A connection-handling function, generic over a duplex `Read + Write` stream and taking a shared
`&Registry` (not owning its own private lock), implements:

- `Hello` negotiation (`min(client, server)` against this crate's own `PROTOCOL_VERSION = 22`,
  `Malformed` on `protocol_version == 0` or a non-first-frame `Hello`, connection stays open
  either way); `Authenticate` (D4).
- Multi-table `Use`/`ListTables` per-connection state, starting on the registry's primary table.
- Session state for `Begin`/`BeginWith`/`Commit`/`Rollback`: staged `UpdateField`-shaped writes,
  `Response::Staged`/`Response::TransactionFailed`. **The full session-open guard set** (citation
  above) — `Begin`, `BeginWith`, `Transaction`, `Insert`, `Link`, `Replace`, `Delete`, `Compact`,
  `ReplaceIf`, `WriteBatch`, `Use` — is refused `SessionOpen` while a session is already open on
  that connection, checked and rejected **before** any table switch or mutation effect (`Use`
  never changes the active table, and no write is ever applied, refused, or partially applied,
  while a session is open — verify this with a scenario that opens a session, sends each guarded
  request, and confirms zero effect and zero table change). `MAX_STAGED_OPS = 4096`: staging the
  4097th `UpdateField` answers `SessionFull` and is not staged, the session staying open with its
  prior staged writes intact (test the boundary at exactly 4096 accepted and the 4097th refused
  without disturbing the first 4096, then `Commit` applying exactly those 4096). `BeginWith`'s
  three flag bits (`SESSION_READ_YOUR_WRITES = 1`, `SESSION_VALIDATE_ON_STAGE = 2`,
  `SESSION_SNAPSHOT_ISOLATION = 4`, freely combinable, any other bit `Malformed`) per the citation
  above: read-your-writes overlays the session's own staged writes onto this connection's own
  `GetById` reads; validate-on-stage runs `Store::validate_op` at stage time, refusing a bad write
  before it is even staged; snapshot isolation records the first committed value of each
  `(id, field)` this session's `GetById` reads (bounded by `MAX_TRACKED_READS = 4096`, matching
  legacy), replays that frozen value on a repeated read of the same key even after a deletion, and
  `Commit` validates the whole tracked read set against current state atomically with applying the
  batch, refusing a mismatch with `ErrorCode::Conflict` and applying nothing.
- For every `Link`, `Delete`, and `WriteBatch` request: acquire the registry's relationship lock
  (if present) for the *entire* request — cross-table existence check, own-table apply, and
  cross-table detach — before releasing it, exactly matching `relationship_section`'s scope (D7).
  Requests outside that set never touch the lock.
- The cross-table `Link` endpoint-existence check (citation above) and the cross-table `Delete`
  detach cascade (`delete_across`/`detach_across`, citation above: after a successful own-table
  delete, call `detach_record` on every *other* registered table whose relation descriptor names
  this table as `target_table`, in registration order, continuing past `Unsupported`/`Malformed`,
  surfacing a `Storage` failure in the delete's place without undoing the already-applied delete).
  `WriteBatch` applies the same two behaviors per `Link`/`Delete` op it contains, exactly matching
  `write_batch_across`'s pipelined-vs-atomic split (pipelined: check/detach per op as it applies;
  atomic: checks folded into `write_batch_checked`'s per-op `check` callback before any write, then
  detaches run after the whole batch's local apply succeeds, in `ops` order, with a detach failure
  recorded as `WriteResult::Failed` on that specific op without undoing the batch). Also enforce
  `MAX_BATCH_OPS = 4096` on `WriteBatch`: `ops.len() > 4096` is `Malformed`, applying nothing,
  checked before either the pipelined or atomic path runs (test the boundary at exactly 4096
  accepted and 4097 refused with zero effect).
- `Join` with `right_table: Some(name)`: intercepted here, before generic `dispatch`. Resolve
  `name` against the registry and re-run `validate_join`'s exact 4-way match (citation above) with
  the resolved `right_schema` — do **not** treat `Malformed` and `Unsupported` as interchangeable:
  a named table that does not match the relevant relation's declared `target_table` (or is named
  against a same-table relation at all) is `Malformed`; a named table that matches the declared
  target but was never registered on this connection is also `Malformed` (`right_schema` stays
  `None` even though the name matched); only the *absence* of `right_table` against a relation
  that declares a `target_table` is `Unsupported`, and that case is reached through generic
  `dispatch` (`right_table: None` always falls through unchanged), not through this interception,
  since there is no `name` to resolve. On success, evaluate using the left adapter's relation
  traversal and the right adapter's `get` (`join_across`, citation above).

Test this loop end-to-end over in-memory duplex pairs and a minimal, host-visible test-double
`Store` implementation written for this crate's own tests only (not a real domain) — cover at
least: `Hello` negotiation at/above/below 22 and version 0; a one-shot `Transaction` that succeeds
and one whose second update's precondition fails, leaving nothing applied and naming the right
index; a session, opened with each of the three `BeginWith` bits individually and combined, that
demonstrates read-your-writes overlay, stage-time validation rejection, and a snapshot-isolation
`Commit` that both succeeds (no conflicting write) and fails with `Conflict` (a concurrent commit
changed a tracked read); an open session sent each of the eleven guarded requests
(`Begin`/`BeginWith`/`Transaction`/`Insert`/`Link`/`Replace`/`Delete`/`Compact`/`ReplaceIf`/
`WriteBatch`/`Use`) in turn, each refused `SessionOpen` with zero effect and (for `Use`) zero
table change, including a `Delete` sent this way and confirmed absent after `Rollback`; staging
exactly `MAX_STAGED_OPS` (4096) `UpdateField`s successfully and the 4097th refused `SessionFull`
with the first 4096 undisturbed, then `Commit` applying exactly those 4096; a `WriteBatch` of
exactly `MAX_BATCH_OPS` (4096) ops accepted and 4097 refused `Malformed` with zero effect; a
`Link` whose relation declares a `target_table` that either does or
does not have the far endpoint; a `Delete` that cascades a detach on another table, including the
adjacency/`CountEdges` state on that other table before and after; a `WriteBatch` (pipelined and
atomic) containing a cross-table `Link` and a cross-table-detaching `Delete`; the four `Join`
outcomes from `validate_join`'s exact match, each asserted against its precise code: a successful
cross-schema join with `right_table: Some(name)` naming a registered table matching the relation's
declared target; `Malformed` for `right_table: Some(name)` naming that same target when it is
*not* registered on this connection; `Malformed` for `right_table: Some(name)` naming a table that
does not match the relation's declared target; `Unsupported` for `right_table: None` against a
relation whose descriptor declares a `target_table` at all; **and, proving D7:** two connections
constructed against
the same shared `Registry` over two independent in-memory duplex pairs, deterministically
interleaved (e.g. by explicit hand-off between two threads or a single-threaded step simulation)
so that connection A's `Link` existence check and connection B's `Delete`-and-detach of the same
far endpoint cannot both succeed against a state where the edge ends up dangling.

## Non-goals

No real domain adapter (Memory/Entity/Relation) implements `Store` in this work order — that is
4b-ii. No `TcpListener`/`TcpStream`/socket code (D2). No `bincode`/`serde`/`uuid` dependency (D1,
D3). No real authentication/authorization (D4). No protocol-version content downgrading (D5). No
SQL parser — `Request::Query`/`Aggregate`/`Join` already arrive structurally pre-parsed; this crate
never sees SQL text, matching the plan's own client/server split. No change to `uc-core`,
`uc-memory`, `uc-entity`, `uc-relation`, or `uc-harness`. No CI workflow change (the existing
`--manifest-path experiments/unified-commitment/Cargo.toml --workspace` commands already cover a
new member crate). No measurement/benchmark series.

## Proof

From the checkout root, using `cargo +1.89.0-x86_64-pc-windows-gnu` throughout, exit 0 required:

```
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/unified-commitment/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/convergence-memory/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/exp-0001/Cargo.toml --all -- --check
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline -- -D warnings
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline
python tools/validate_markdown_links.py
git diff --check
```

Plus: the R4 fixture test must show all 66 lines passing both the literal-value and the round-trip
check (either as one parameterized test whose failure output names the failing line, or as an
explicit count assertion) — report this count in the implementation report even though it is
already implied by the `cargo test` pass above. Plus: the R7 two-connection interleaving test (D7)
must be present and passing, and the implementation report must name it explicitly and describe
what it demonstrates, since it is the proof for the highest-severity review finding this work
order closes.

Also required, alongside the code: `docs/adr/ADR-0005-protocol-facade.md` (Status: Proposed),
`docs/experiments/EXP-0005-protocol-facade.md`, `docs/hypotheses/HYP-0005-protocol-facade.md`,
following this workspace's existing EXP/ADR/HYP format (see EXP-0004/ADR-0004/HYP-0004 for the
pattern), and the usual `AGENTS.md`/`README.md`/`GLOSSARY.md`/`PROJECT-STATUS.md`/
`RESEARCH-QUESTIONS.md`/`TRACEABILITY.md`/`experiments/README.md`/`docs/experiments/README.md`
synchronization updates (AGENTS §7).
