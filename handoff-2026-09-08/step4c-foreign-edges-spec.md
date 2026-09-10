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

## Owner decision (2026-09-09, handoff resume) — review round 1 dispositions

Review round 1 (`step4c-review/claudex-nclrdn4q`, plan sha256 `14521486…60c6`) returned REVISE, 2
high + 3 medium (full findings recorded in `BUILD-LOG.md`'s Step 4c section). Session paused there
for a handoff; on resume, the owner was asked for a disposition on the two high findings and gave:

- **P4C-001 (incarnation gap): extend the `Store` trait.** Not the narrow-the-window-and-disclose
  option — the owner's rationale: a client fully controls insert ids on the wire, so a deliberate
  same-id reinsert during an in-flight crash recovery is a normal, reachable case, not a freak
  collision; this project's own precedent (Steps 3/4a) already treats delete-then-reinsert-same-id
  as a defended scenario for every domain's own table, and the foreign-edge case must meet the
  same bar. A small, narrow, disclosed `Store` trait addition (matching the shape of four prior
  D7-style exceptions in Step 4b-ii) is preferred over leaving a real, reachable gap as a residual.
- **P4C-002 (collision half): also fix now**, not accepted as a disclosed residual. The host found
  that the same mechanism needed for P4C-001 (a live incarnation/existence check against the far
  table, held via a direct reference rather than routed through the registry) also yields a
  principled, non-probabilistic fix for the Join/reverse-lookup collision case — see D8/D9 below.
  This is not a probability-reduction fix; it is a deterministic disambiguation rule that removes
  the ambiguity class entirely, including the adversarially-constructed double-collision case the
  review's reproduction used.

D3, D5 and D7 below are revised accordingly (superseding the review-round-1 spec's original text);
D9/D10/D11 are new, addressing P4C-002's first half, P4C-004 and P4C-005 respectively. Proof item 7
(P4C-003) is revised in the Proof section.

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
- **D3 (revised, review round 1 P4C-001) — `LinkForeign` records `to_incarnation`, resolved live
  at link time, and every read path re-verifies it live before trusting the edge.** The original
  round-1 spec recorded only `to: [u8; 16]` with no incarnation for the far side, reasoning that
  Memory's own replay has no access to Entity's log — true for *replay*, but the *live* adapter
  path (D7/D8 below) does have a direct handle to the Entity `Store` and can check it at both link
  time and every read time. `LinkForeign` now records `to_incarnation: u64`, the Entity id's live
  incarnation observed at link time via `Store::incarnation` (D8). Replay never re-validates it
  (D1/D2 unchanged — a checkpoint/replay reproduces the stored tuple exactly, including whatever
  `to_incarnation` was recorded live at the time); only the *live* adapter re-checks it against the
  Entity table's current incarnation on every read (D9), which is what closes P4C-001: a
  delete-then-reinsert-same-id at the far endpoint changes its live incarnation, so a stale edge's
  recorded `to_incarnation` no longer matches and the edge reads as a miss — never as the wrong,
  unrelated record. The crash-window dangling edge itself is unchanged and still tolerated exactly
  as legacy's own doc comment allows (a delete-without-reinsert still correctly misses via a
  missing live record, same as the original D3); what's newly closed is only the reinsert case.
- **D4 — `DetachForeign` matches by id only, not by any tracked incarnation, exactly like the
  existing same-table `Delete` cascade** (`state.edges.retain(|(a, _, b, _)| a != id && b != id)`
  already ignores incarnation when cascading — mirror that same pattern for `foreign_edges`).
- **D5 (revised, review round 1 P4C-002 second half) — direction is resolved by live local
  membership, not by blind raw-byte matching.** The original round-1 spec reused the same-table
  lookup's symmetric `if *a==id {..} else if *b==id {..}` pattern directly against
  `foreign_edges`, which review round 1 found unsound: `from` and `to` positions are not
  interchangeable once one side is a genuinely different table's id space, and with colliding raw
  bytes across the two independently-numbered tables (adversarially constructible; negligible
  probability with random UUIDs but not zero), blind position matching lets a query for one
  record's neighbors return another, unrelated record's edges. The fix: given a queried id `q`,
  first check whether `q` currently resolves as a **live Memory record** (`Self::current(&state,
  q)`, already available, no new capability needed) — if so, resolve `q` as the `from` side
  (return the Entity ids it mentions, each individually revalidated per D9's freshness check). If
  `q` does not resolve locally, resolve it as the `to` side (return the Memory ids that mention it,
  again each revalidated per D9 — a `to`-side query also freshness-checks itself: if `q`'s own live
  incarnation via `Store::incarnation` (D8) no longer matches any edge's recorded `to_incarnation`,
  that edge does not match). This is a deterministic priority rule, not a probability reduction: a
  queried id that happens to collide as both a live Memory id and a live Entity id now
  deterministically resolves as the local (`from`) interpretation only, never a blended or
  fabricated result mixing both tables' edges — this is what closes P4C-002's Join/reverse-lookup
  half completely, including the adversarially-constructed double-collision case, not just the
  common case. `count_edges` uses the same live-membership rule to decide, for each stored tuple,
  whether it is still a live, currently-valid edge before counting it.
- **D6 — `MemoryStore::describe_relations()` reverts to `target_table: Some("entity")` for
  `"mentions"`.** This supersedes Step 4b-ii's D6 exactly as anticipated there ("Memory's mentions
  descriptor has `target_table: None`. Real foreign-edge durability remains deferred").
- **D7 (revised, review round 1 P4C-001/002 disposition) — no change to `write_batch_checked`'s
  fail-closed default; `EntityStore` gains one small, disclosed override (D8), not zero changes.**
  Atomic `WriteBatch` support stays out of scope, unchanged from Step 4b-ii's own disposition (F5)
  — only the already-working *pipelined* cross-table `WriteBatch` path is expected to now succeed
  for real Memory+Entity ops (as two separate `Use`-switched requests per P4C-003 in Proof).
  Entity's own engine (`uc-entity`), `uc-relation`, `uc-core`, `uc-harness` and `RelationStore` are
  still completely untouched — the only exception this work order now takes against a previously
  "frozen" surface is the new optional `Store` trait method (D8) and `EntityStore`'s override of
  it, both narrow and mechanical, matching the shape (not the specifics) of Step 4b-ii's own four
  D7-style exceptions there.
- **D8 (new, closes P4C-001 owner-approved trait extension) — `Store` gains one new optional
  method, `fn incarnation(&self, _id: RecordId) -> Option<u64> { None }`.** Added to
  `uc-protocol::store::Store` (`store.rs`) alongside its other optional/default-bodied methods
  (`detach_record`, `count_edges`, etc. — same section, same pattern). Every existing `Store` impl
  compiles unchanged (default returns `None`, meaning "this table does not expose incarnation" —
  correct for `RelationStore` and for `MemoryStore` itself, since nothing in this table's own
  table ever needs to ask its own incarnation this way). `EntityStore` overrides it, delegating
  directly to its already-existing private `Self::current` helper (`entity.rs:46-53`), mapping
  `Err(RecordNotFound)` to `None`:
  ```rust
  fn incarnation(&self, id: RecordId) -> Option<u64> {
      let engine = self.0.lock().expect("entity engine mutex poisoned");
      Self::current(&engine.log().snapshot(), id).ok()
  }
  ```
  Four lines, delegating to logic that already exists and is already tested (Step 4a). No other
  `Store` impl needs to override this method for this work order's scope.
- **D9 (new, wiring for D3/D5/D8) — `MemoryStore` holds a direct `Arc<dyn Store>` handle to the
  Entity table, supplied at construction, not routed through the registry per call.** The registry
  (`uc-protocol::connection::Registry::check_link`/`detach`) is unchanged — it still performs its
  own existing existence check and detach dispatch exactly as before (D1/D6 unaffected). This
  handle is a separate, `uc-facade`-internal wiring path used only for D3's live incarnation
  capture at link time and D5/D9's live freshness/membership checks at read time (`neighbors`,
  `neighbors_by_relation`, `count_edges`, and transitively `Join` via `evaluate_join`, which reads
  through `neighbors_by_relation` — no change needed in `uc-protocol::query`). `MemoryStore::new`
  gains a second constructor parameter (e.g. `MemoryStore::new(engine, entity: Arc<dyn Store>)`);
  the facade's assembly wiring passes the already-constructed `EntityStore` in when both tables are
  registered. This does not create a new coupling to `uc-entity`'s engine internals — only to the
  already-object-safe `Store` trait, the same abstraction the registry itself already programs
  against. Both existing and new `foreign_edges` reads (D5) apply the freshness check: a tuple's
  recorded `to_incarnation` must equal `entity.incarnation(to)` at read time, or the tuple is
  treated as not currently matching (a miss, never a wrongly-resolved pair) — it is **not**
  deleted or cleaned up by a read (reads stay side-effect-free); only `DetachForeign` (D1/D4,
  unchanged) removes an entry.
- **D10 (new, review round 1 P4C-002 first half, unconditional zero-cost fix) —
  `MemoryStore::link_records`'s `AlreadyLinked` check for `"mentions"` checks only the single
  direction `(left.0, from_incarnation, right.0)`, never the swapped tuple.** The existing
  same-table code (`memory.rs:344-350`) checks both `(left, right)` and `(right, left)` because
  same-table `Link` is genuinely symmetric; a foreign edge is directional by construction (`from`
  is always the Memory-side id, `to` is always the raw Entity bytes) and checking the swap risks a
  false `AlreadyLinked` against an unrelated, never-committed pair under colliding raw bytes. Pure
  deletion of the second `state.edges.contains(...)` disjunct for the foreign-edge path; no new
  capability needed.
- **D11 (new, review round 1 P4C-004, mechanical) — `link_records` rejects a self-loop
  (`left.0 == right.0`) for `"mentions"` with `Malformed`, checked before any other work.** Matches
  legacy's own explicit rejection of a self-loop even across a foreign relation
  (`generic/store.rs:1353`, `server/memory.rs:780` → `Malformed`), which review round 1 found the
  original spec never specified.
- **D12 (new, review round 1 P4C-005, mechanical) — `MemoryStore::describe_relations()` is
  overridden again, listing only the named `"mentions"` descriptor
  (`target_table: Some("entity")`), explicitly omitting the generic `Neighbors(None)` wildcard the
  trait default (`default_relation_descriptors`) would otherwise supply.** Step 4b-ii's own F1 fix
  (deleting Memory's and Entity's custom overrides so both fall back to the trait default, which
  supplies the wildcard) was correct for Entity — Entity has no foreign relation and legitimately
  wants the generic wildcard neighbor descriptor — but is wrong for Memory once `"mentions"` is a
  real foreign relation: legacy's actual Memory descriptor omits the wildcard entirely
  (`server/memory.rs:747`), because a bare/unlabeled `Join` against Memory must not silently route
  through the generic same-table `Neighbors(None)` evaluation path once a real foreign relation
  exists. This explicitly supersedes 4b-ii's F1 correction for Memory specifically; Entity's own
  wildcard, and 4b-ii's regression test for Entity, are untouched. The existing wildcard regression
  test's Memory-side assertion must be updated to expect the wildcard's absence again.

## Required changes

**R1 — `uc-protocol::store::Store` and `uc-facade::EntityStore`: the D8 trait extension.** Add
`fn incarnation(&self, _id: RecordId) -> Option<u64> { None }` to the `Store` trait (`store.rs`),
grouped with the other optional/default-bodied methods. `EntityStore` overrides it per D8's exact
four-line body. No other `Store` impl changes for this work order (default `None` is correct for
`RelationStore` and `MemoryStore`).

**R2 — `uc-memory`: two new `Change` variants, `LinkForeign` now carrying `to_incarnation`.**

```rust
Change::LinkForeign { from: Id, from_incarnation: u64, to: [u8; 16], to_incarnation: u64 }
Change::DetachForeign { to: [u8; 16] }
```

`apply_changes`' new arms: `LinkForeign` requires `state.current(from, from_incarnation)` to
resolve (the *local* side only — this is Memory's own existence/incarnation check) and inserts
`(from, from_incarnation, to, to_incarnation)` into a new `state.foreign_edges:
BTreeSet<(Id, u64, [u8; 16], u64)>`; no re-check at all of `to`/`to_incarnation` during replay
(D3 — the *live* adapter resolved and recorded them at submit time via D8/D9; replay only
reproduces the stored tuple verbatim, it cannot re-contact Entity's log). `DetachForeign { to }`
removes every `foreign_edges` entry whose third element equals `to`, matching by id only (D4), and
succeeds unconditionally (including when there is nothing to remove — an idempotent no-op,
matching legacy's own "an adapter... has nothing to drop and is skipped" framing). Extend
`Change::Delete`'s existing cascade to also remove every `foreign_edges` entry whose `from`
matches the deleted id (same-log cascade, D1 — Memory cleaning up its own outgoing foreign edges
when its own record is deleted needs no cross-log coordination at all).

**R3 — `uc-memory`: `CMM3`/`CMS3` codec.** New magic strings for both the transaction-log payload
codec and the checkpoint codec (D2). `decode_changes`/`encode_changes` gain `linkforeign`/
`detachforeign` line forms (the `linkforeign` form now serializes four fields, including
`to_incarnation`); `encode_state`/`decode_state` gain a `foreign_edge` line form (four fields),
canonical-order verified exactly like every existing line kind (`decode_state` re-encodes and
compares, per the existing `CMS2` pattern). `MemoryEngine::create`/`open` switch to the new
magic/decode/encode functions. `MAX_OPERATIONS` (already 4096, Step 4b-ii) is unchanged and applies
to the new variants too.

**R4 — `uc-facade::MemoryStore`.**

- Constructor gains the D9 handle: `MemoryStore::new(engine: MemoryEngine, entity: Arc<dyn
  Store>)`. The facade assembly wiring (wherever `MemoryStore::new`/`EntityStore::new` are both
  called to build the registered tables) passes the already-constructed Entity `Store` in.
- `describe_relations()`: overridden per D12 — only the named `"mentions"` descriptor,
  `target_table: Some("entity")`, no wildcard.
- `link_records(left, right, relation)`: when `relation == "mentions"`: reject `left.0 ==
  right.0` as `Malformed` first (D11); verify `left`'s current incarnation in Memory's own state
  (`Self::current`, unchanged); resolve `to_incarnation = self.entity.incarnation(right)`,
  `RecordNotFound` if `None` (D8/D9 — this is the live existence+incarnation check, replacing the
  registry-only boolean check as the value source for what gets recorded, though the registry's
  own `check_link` still runs first and unchanged); check `AlreadyLinked` against `foreign_edges`
  in the single `(left.0, from_incarnation, right.0, ..)` direction only (D10 — no swapped-tuple
  check); submit `Change::LinkForeign { from: left.0, from_incarnation, to: right.0,
  to_incarnation }`.
- `detach_record(relation, id)`: validate `relation == "mentions"` (`Malformed` otherwise,
  matching legacy's `DEL-FR-005` citation), submit `Change::DetachForeign { to: id.0 }`, return
  `Ok(count)` of edges actually removed (read the pre-submit count via a snapshot diff, or have
  the engine report it — implementer's choice, tested either way).
- `neighbors`/`neighbors_by_relation`/`count_edges`: for `"mentions"`, read `foreign_edges` using
  D5's live-membership direction rule (check `Self::current(&state, queried_id)` first to decide
  `from`- vs `to`-side interpretation) and D9's freshness filter (`self.entity.incarnation(to) ==
  recorded to_incarnation` before treating a tuple as a current match), in addition to (not
  instead of) the existing same-table `edges` read — a wire client never distinguishes
  "same-table" vs "foreign" adjacency, so both sets contribute to the same relation's answer. (In
  practice only `foreign_edges` will ever be populated for `"mentions"` once this lands, since no
  code path inserts a same-table `mentions` edge for Memory — but reading both costs nothing and
  stays correct if that ever changes.) `Join` needs no direct change (`uc-protocol::query`
  untouched) — it calls `neighbors`/`neighbors_by_relation` and inherits the fix transitively.

## Non-goals

No change to `uc-entity`'s engine, `uc-relation`, `uc-core`, `uc-harness`, or
`uc-facade::RelationStore` (D7). The only exceptions to "frozen" surfaces this work order takes
are the one new optional `Store` trait method (D8) and `EntityStore`'s four-line override of it
(D8) — both narrow, mechanical and disclosed here and in `BUILD-LOG.md`, not a broader redesign.
No atomic cross-table `WriteBatch` support (still `write_batch_checked`'s 4b-i fail-closed default,
unchanged). No migration path for an existing `CMM2` Memory store (D2 — none exists with real data
yet). No new relation kind, no SQL, no listener/bind/auth change (all Step 4b-ii scope, untouched
here). No measurement/benchmark series. No cleanup-on-read for a stale foreign edge (D9 — reads
stay side-effect-free; only `DetachForeign` removes an entry, unchanged from D1/D4).

## Proof

Same eleven-command chain as every prior Step 4 increment (unified-commitment fmt/clippy/test;
convergence-memory fmt/clippy/test; exp-0001 fmt/clippy/test, harness excluded; markdown links;
`git diff --check`), via `cargo +1.89.0-x86_64-pc-windows-gnu`, exit 0 required. Additional
required test coverage, named explicitly in the implementation report:

1. A real cross-table `Link(memory_id, entity_id, "mentions")` over a real socket succeeds when
   the Entity table is registered and the id exists; `Malformed`/`Unsupported`/`RecordNotFound` in
   the same shapes Step 4b-i's synthetic-double tests already established for the unregistered/
   missing-endpoint cases, now against real adapters. `left.0 == right.0` is rejected `Malformed`
   before any other check (D11/P4C-004).
2. `neighbors`/`neighbors_by_relation("mentions")` against the Memory table finds the Entity id
   after linking, **and** the same query given the Entity id as input finds the Memory id back
   (D5's dual-direction, live-membership-resolved lookup).
3. A real `Join` across Memory and Entity via `"mentions"` (registered `right_table: Some
   ("entity")`) returns the linked pair.
4. Deleting the *Memory* record cascades removal of its own outgoing `foreign_edges` (same-log
   cascade, R2).
5. Deleting the *Entity* record triggers the registry's existing `detach_across`, which calls
   `MemoryStore::detach_record` for real; a subsequent `neighbors`/`count_edges` on Memory no
   longer reports the edge.
6. Checkpoint then full replay of Memory's own log (containing at least one `LinkForeign`,
   including its `to_incarnation`, and one `DetachForeign`) reproduce an identical `foreign_edges`
   set — pure single-log replay, D1/D2/D3.
7. (Revised, P4C-003 — `WriteOp` carries no table selector; one `WriteBatch` always targets
   whichever table the connection last `Use`d, so a single pipelined batch cannot span both
   tables.) Two separate pipelined `WriteBatch` requests over the same socket, `Use`-switched
   between them — one containing a real cross-table `Link` on Memory, the other a real
   cross-table-detaching `Delete` on Entity — both succeed end-to-end (D7 — the already-built
   pipelined path, now exercised for real, just not as a single combined batch).
8. A `CMM2`-only decode attempt against a `CMM3` log (or vice versa) fails closed with a magic
   mismatch, never a misinterpretation (D2).
9. (New, P4C-001 closure) Delete the Entity endpoint, then — before its detach retry completes —
   reinsert a *new* Entity record at the same id (a different live incarnation). `neighbors`/
   `neighbors_by_relation`/`Join`/`count_edges` against the stale edge now report a miss, never the
   new unrelated record (D3/D9's freshness check).
10. (New, P4C-002 closure) A queried id constructed to collide (same raw 16 bytes) with both a
    live Memory record and a live Entity record resolves `neighbors`/`neighbors_by_relation`
    deterministically as the local (`from`-side, Memory) interpretation only — never a blended or
    fabricated pair mixing both tables' edges (D5). A `Link` attempt that would only be
    `AlreadyLinked` under the swapped-tuple check succeeds (not falsely rejected) (D10).
11. (New, P4C-005 regression) `describe_relations()` on Memory lists only the named `"mentions"`
    descriptor, no `Neighbors(None)` wildcard; a bare/unlabeled `Join` against Memory is
    `Malformed`, exactly reproducing legacy's own descriptor shape (D12). Entity's own wildcard
    descriptor (from the trait default, Step 4b-ii's F1) is unaffected — update the existing
    wildcard regression test's Memory-side assertion to expect the wildcard's absence again.
