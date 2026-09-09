# Step 4c work order — a real, durable Memory→Entity `mentions` edge

## Source plan

[Merge plan step 4](../data-os-multimodal-merge-plan-2026-09-08.md): "Port Memory, Entity and
Relation first." This closes the one gap Step 4b-ii's own spec deferred
(`step4bii-domain-adapters-spec.md`, Design decision D6): a real, durable cross-table edge from a
Memory record to an Entity record, matching legacy's `mentions` relation
(`MEMORY_FOREIGN_TABLE = "entity"`). It is also the `MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY` item
already recorded Deferred in `docs/roadmap/ROADMAP.md`.

## Owner decision (2026-09-09)

Proceed with a real design, not a further deferral. The host's own re-reading of legacy's
`delete_across`/`detach_across` doc comment (quoted below) found that legacy itself does **not**
guarantee atomic cross-table cleanup — a crash between an Entity delete committing and Memory's
detach committing leaves a real, disclosed, temporarily dangling edge. This means a faithful
design does not require inventing a distributed transaction across `uc-memory`'s and
`uc-entity`'s independent logs; it only requires two new same-log operations in `uc-memory` that
reproduce that same, already-accepted tolerance. This is the design authorized below.

## Target repository

Worktree `C:/dev/rusty_data_os-step4c`, branch `codex/merge-step4c-foreign-edges`, base
`3d794a5efa4e2af94e817b951777b3ccdd77c148` (Step 4b-ii's closing commit). Initial `git status` is
clean.

## Goal

Add a durable, replay-correct foreign-edge mechanism to `uc-memory` only (Entity/Relation need no
change — Memory is the only domain with a real foreign relation), then wire it into
`uc-facade::MemoryStore` so the registry's already-built cross-table `Link`-existence-check and
`Delete`-detach-cascade machinery (from 4b-i, exercised only by a synthetic test double until now)
finally operates on real Memory and Entity records. Verified by real-socket tests: a `Link` from a
real Memory id to a real Entity id; `neighbors`/`neighbors_by_relation`/`Join` finding it from
either side; a `Delete` of the Entity endpoint cascading a real detach on Memory; checkpoint and
full replay of Memory's own log reproducing the identical foreign-edge set with no access to
Entity's log at all.

## Repository facts

### Why `Change::Link` cannot represent this edge (the blocker Step 4b-ii's review found)

`experiments/unified-commitment/crates/uc-memory/src/lib.rs`, `apply_changes`'s `Change::Link`
arm (unchanged since Step 3):

```rust
Change::Link { from, from_incarnation, to, to_incarnation } => {
    state.current(from, *from_incarnation)?;
    state.current(to, *to_incarnation)?;
    state.edges.insert((*from, *from_incarnation, *to, *to_incarnation));
}
```

Both `from` and `to` must resolve in Memory's *own* `state.slots` — this was built (Step 3,
before Entity existed) as a generic same-table symmetric link, never a foreign-table edge. An
Entity id can never satisfy `state.current(to, ..)` here. This is the exact structural fact Step
4b-ii's plan review (P4B-002/F2) found and the owner descoped around at the time.

### Legacy's own crash-window tolerance — the bar this design must match, not exceed

`rusty_multimodal_db/src/server/serve.rs:3040-3050`, `delete_across`'s doc comment, verbatim:

> the table's own `delete_record` (which drops the record's edges within that table), then, only
> on `Ok`, every *other* table's `detach_record` for each of its relations whose `target_table` is
> this one... An adapter answering `Unsupported`/`Malformed` for the detach has nothing to drop
> and is skipped; a `Storage` failure there is reported in the delete's place, the record itself
> already gone (the one partial state, named in the design). **A crash between the two steps
> leaves edges to a record no table holds: Join skips missing rows (`evaluate_join`'s `get` miss),
> but adjacency and CountEdges can still report those edges until they are detached.**

Legacy does not promise atomic cross-table cleanup. A design that tolerates the identical
crash-window dangling edge (never silently resurrecting it as something else, always cleanly
detachable once the detach transaction lands) is faithful, not a lesser approximation.

### `uc-facade::MemoryStore`'s current relation methods (Step 4b-ii, unchanged since)

`experiments/unified-commitment/crates/uc-facade/src/memory.rs`:

- `describe_relations()` returns `target_table: None` for `"mentions"` (Step 4b-ii's D6 — the
  explicit, disclosed workaround this work order supersedes).
- `link_records(left, right, relation)` requires **both** `left` and `right` to resolve via
  `Self::current(&state, ..)` against Memory's own snapshot (`memory.rs:~331-350`) — the adapter
  inherited the engine's own same-table assumption; it never distinguished "my own id" from "the
  far endpoint the registry already checked."
- `neighbors`/`neighbors_by_relation`/`count_edges` (`memory.rs:283-322`) read `state.edges`
  (the same-table `BTreeSet<(Id,u64,Id,u64)>`) with a symmetric lookup — matching either tuple
  position against the queried id, which is exactly the pattern legacy's own `neighbors` doc
  comment describes ("the entity ids a memory mentions — or, given an entity id, the memories
  that mention it, since the edge is stored in both directions") and which the new foreign-edge
  lookup must reproduce (a client may legally query the Memory table's `neighbors_by_relation`
  with either a Memory id or an Entity id).
- `detach_record` is not overridden — the 4b-i default `Err(ErrorCode::Unsupported)` applies.

### The registry's cross-table machinery already exists and is unaffected

`experiments/unified-commitment/crates/uc-protocol/src/connection.rs` (4b-i): `check_link_across`
(existence check on the *far* endpoint against the relation's declared `target_table`),
`detach_across` (calls `detach_record` on every other registered table whose relation names this
one as `target_table`, after a successful own-table delete), and the shared `relationship_lock`
(held across the whole check→apply→detach span for `Link`/`Delete`/`WriteBatch`) are all already
built, reviewed, and tested (with a synthetic double in 4b-i; never exercised with two real
domains, since Step 4b-ii's D6 declared `target_table: None` for exactly this reason). None of
this needs to change — restoring `target_table: Some("entity")` on `MemoryStore::describe()` is
what turns this machinery on for real.

## Design decisions settled by the host

- **D1 — purely additive to `uc-memory`; nothing existing changes.** New `Change` variants
  (`LinkForeign`, `DetachForeign`), a new `State` field (`foreign_edges`), and new checkpoint
  lines. `Change::Put`/`Update`/`Delete`, the existing `edges` field, and every existing Step 3
  scenario test are untouched — this reproduces the exact "narrow, disclosed, additive exception"
  shape every prior touch of a frozen crate in this project has used (Step 4b-ii's D7/R0), just
  with a real new capability behind it rather than a mechanical constant/bound fix.
- **D2 — a new physical format version, `CMM3`/`CMS3`, not a silent extension of `CMM2`/`CMS2`.**
  This is a new *concept* (an edge whose far side is never locally verified), not a new value
  within an existing one — matching this project's established convention of a magic-byte bump
  for a structural change (RF1/RDE1, UCE1/UCR1, CME1/CMR1). An old `CMM2`/`CMS2`-only decoder must
  reject a log/checkpoint containing the new operations outright (fail closed), never
  misinterpret them. `Log::create`/`Log::open` for `MemoryEngine` now use the `CMM3`/`CMS3`
  decode/encode functions; there is no migration path for an existing `CMM2` store in this work
  order (none exists yet in any committed evidence — Steps 2/3's stores are measurement
  artifacts, not a production database with real data to carry forward).
- **D3 — no `to_incarnation` on a foreign edge.** A same-table `Link` tracks both sides'
  incarnations because both are locally re-verifiable; a foreign edge's far side never is (Memory's
  own replay has no access to Entity's log). `LinkForeign` records only `to: [u8; 16]` (the raw
  Entity id bytes) — never an incarnation for that side — matching legacy's own `check_link_across`,
  which is a plain existence check at link time with no incarnation concept at all.
- **D4 — `DetachForeign` matches by id only, not by any tracked incarnation, exactly like the
  existing same-table `Delete` cascade** (`state.edges.retain(|(a, _, b, _)| a != id && b != id)`
  already ignores incarnation when cascading — mirror that same pattern for `foreign_edges`).
- **D5 — the lookup is symmetric on raw bytes, matching legacy's documented dual-direction
  query.** `neighbors`/`neighbors_by_relation` against the `"mentions"` label check a queried id
  against *either* position of a `foreign_edges` tuple (the Memory-side `from` or the raw Entity
  `to`), exactly reproducing the existing same-table lookup's own `if *a==id {..} else if
  *b==id {..}` pattern and legacy's own documented behavior (a client connected to the Memory
  table may query with either a Memory id or an Entity id).
- **D6 — `MemoryStore::describe_relations()` reverts to `target_table: Some("entity")` for
  `"mentions"`.** This supersedes Step 4b-ii's D6 exactly as anticipated there ("Memory's mentions
  descriptor has `target_table: None`. Real foreign-edge durability remains deferred").
- **D7 — no change to `write_batch_checked`'s fail-closed default, no change to `EntityStore`.**
  Entity needs zero code changes (it never declares a foreign relation itself; the registry's
  generic `get`-based existence check and `detach_record` dispatch already work against it
  unmodified). Atomic `WriteBatch` support stays out of scope, unchanged from Step 4b-ii's own
  disposition (F5) — only the already-working *pipelined* cross-table `WriteBatch` path is
  expected to now succeed for real Memory+Entity ops, and is tested as such.

## Required changes

**R1 — `uc-memory`: two new `Change` variants.**

```rust
Change::LinkForeign { from: Id, from_incarnation: u64, to: [u8; 16] }
Change::DetachForeign { to: [u8; 16] }
```

`apply_changes`' new arms: `LinkForeign` requires `state.current(from, from_incarnation)` to
resolve (the *local* side only — this is Memory's own existence/incarnation check, unaffected by
D3) and inserts `(from, from_incarnation, to)` into a new `state.foreign_edges:
BTreeSet<(Id, u64, [u8; 16])>`; no check at all on `to` (D3 — the registry already checked it
before calling `link_records`, and replay cannot re-check it). `DetachForeign { to }` removes
every `foreign_edges` entry whose third element equals `to`, matching by id only (D4), and
succeeds unconditionally (including when there is nothing to remove — an idempotent no-op,
matching legacy's own "an adapter... has nothing to drop and is skipped" framing). Extend
`Change::Delete`'s existing cascade to also remove every `foreign_edges` entry whose `from`
matches the deleted id (same-log cascade, D1 — Memory cleaning up its own outgoing foreign edges
when its own record is deleted needs no cross-log coordination at all).

**R2 — `uc-memory`: `CMM3`/`CMS3` codec.** New magic strings for both the transaction-log payload
codec and the checkpoint codec (D2). `decode_changes`/`encode_changes` gain `linkforeign`/
`detachforeign` line forms; `encode_state`/`decode_state` gain a `foreign_edge` line form,
canonical-order verified exactly like every existing line kind (`decode_state` re-encodes and
compares, per the existing `CMS2` pattern). `MemoryEngine::create`/`open` switch to the new
magic/decode/encode functions. `MAX_OPERATIONS` (already 4096, Step 4b-ii) is unchanged and applies
to the new variants too.

**R3 — `uc-facade::MemoryStore`.**

- `describe_relations()`: `target_table: Some("entity")` for `"mentions"` (D6).
- `link_records(left, right, relation)`: when `relation == "mentions"`, verify `left`'s current
  incarnation in Memory's own state (`Self::current`, unchanged), do **not** attempt to verify
  `right` against Memory's state at all, and submit `Change::LinkForeign { from: left.0,
  from_incarnation, to: right.0 }`. Preserve the existing `AlreadyLinked` check (now against
  `foreign_edges`, matching on `(left.0, .., right.0)` in either stored position per D5) before
  submitting. (`right` may legally *not* be a Memory id at all — do not call `Self::current` on
  it; the registry already confirmed it exists in the `target_table` before this method runs.)
- `detach_record(relation, id)`: validate `relation == "mentions"` (`Malformed` otherwise,
  matching legacy's `DEL-FR-005` citation), submit `Change::DetachForeign { to: id.0 }`, return
  `Ok(count)` of edges actually removed (read the pre-submit count via a snapshot diff, or have
  the engine report it — implementer's choice, tested either way).
- `neighbors`/`neighbors_by_relation`/`count_edges`: for `"mentions"`, read `foreign_edges` with
  the symmetric lookup from D5, in addition to (not instead of) the existing same-table `edges`
  read — a wire client never distinguishes "same-table" vs "foreign" adjacency, so both sets
  contribute to the same relation's answer. (In practice only `foreign_edges` will ever be
  populated for `"mentions"` once R3's `link_records` change lands, since no code path inserts a
  same-table `mentions` edge for Memory — but reading both costs nothing and stays correct if that
  ever changes.)

## Non-goals

No change to `uc-entity`, `uc-relation`, `uc-core`, `uc-harness`, `uc-protocol`, or
`uc-facade::EntityStore`/`RelationStore` (D7). No atomic cross-table `WriteBatch` support (still
`write_batch_checked`'s 4b-i fail-closed default, unchanged). No migration path for an existing
`CMM2` Memory store (D2 — none exists with real data yet). No new relation kind, no SQL, no
listener/bind/auth change (all Step 4b-ii scope, untouched here). No measurement/benchmark series.

## Proof

Same eleven-command chain as every prior Step 4 increment (unified-commitment fmt/clippy/test;
convergence-memory fmt/clippy/test; exp-0001 fmt/clippy/test, harness excluded; markdown links;
`git diff --check`), via `cargo +1.89.0-x86_64-pc-windows-gnu`, exit 0 required. Additional
required test coverage, named explicitly in the implementation report:

1. A real cross-table `Link(memory_id, entity_id, "mentions")` over a real socket succeeds when
   the Entity table is registered and the id exists; `Malformed`/`Unsupported`/`RecordNotFound` in
   the same shapes Step 4b-i's synthetic-double tests already established for the unregistered/
   missing-endpoint cases, now against real adapters.
2. `neighbors`/`neighbors_by_relation("mentions")` against the Memory table finds the Entity id
   after linking, **and** the same query given the Entity id as input finds the Memory id back
   (D5's dual-direction lookup).
3. A real `Join` across Memory and Entity via `"mentions"` (registered `right_table: Some
   ("entity")`) returns the linked pair.
4. Deleting the *Memory* record cascades removal of its own outgoing `foreign_edges` (same-log
   cascade, R1).
5. Deleting the *Entity* record triggers the registry's existing `detach_across`, which calls
   `MemoryStore::detach_record` for real; a subsequent `neighbors`/`count_edges` on Memory no
   longer reports the edge.
6. Checkpoint then full replay of Memory's own log (containing at least one `LinkForeign` and one
   `DetachForeign`) reproduce an identical `foreign_edges` set — pure single-log replay, D1/D2.
7. A pipelined `WriteBatch` containing a real cross-table `Link` and a real cross-table-detaching
   `Delete` succeeds end-to-end (D7 — the already-built pipelined path, now exercised for real).
8. A `CMM2`-only decode attempt against a `CMM3` log (or vice versa) fails closed with a magic
   mismatch, never a misinterpretation (D2).
