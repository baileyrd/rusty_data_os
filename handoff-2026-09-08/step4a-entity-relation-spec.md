# Work order: merge plan Step 4a — port Entity and Relation onto the unified core (Data OS)

Source plan: `docs/plans/data-os-multimodal-merge-plan-2026-09-08.md`, section
"4. Integrate existing features" (first sentence: "Port Memory, Entity and Relation first").
This is the first of several Step 4 work orders. Depends on Step 3
(`experiments/unified-commitment`, committed at `065470c`/`2f05dfc`).

Target repository: the Data OS build worktree at `C:/dev/rusty_data_os-step4`, branch
`codex/merge-step4-entity-relation-core` created from `2f05dfc` (Step 3 head). Resolve every
path against that worktree; never edit `C:/dev/rusty_data_os`, `C:/dev/rusty_data_os-step2` or
`C:/dev/rusty_data_os-step3`. Follow `AGENTS.md` (§2 measurement method before implementation,
§4, §5 correctness invariants independent of performance, §6, §7 same-change documentation,
§10 authorization ledger) and `docs/experiments/README.md` (18-section document, status
vocabulary). Do not `git commit`.

## Goal

Add Entity and Relation as two more domains on the unified transaction core built in Step 3,
each in its own `Log`/directory exactly as Memory is today, faithfully matching their existing
`rusty_multimodal_db` semantics (not an idealized redesign). This demonstrates the shared core
generalizes beyond one domain. Genuine cross-domain atomic commitment (one transaction touching
two domains) is explicitly out of scope for this work order; see "Non-goals" and the follow-on
recorded in EXP-0004 §18.

## Repository facts the design must respect (verified 2026-09-09, both repositories)

### The unified core (`experiments/unified-commitment`, unchanged by this work order)

- `uc-core::Log<S>` is generic over any `S: Clone + Default`; there is exactly one `S` per
  `Log`, held as `Arc<S>`. `S` is used opaquely — it may be a struct holding multiple named
  sub-collections (`uc_memory::State` already holds both `slots: BTreeMap<Id, Slot>` and
  `edges: BTreeSet<(Id, u64, Id, u64)>`). No core change is needed to add a domain.
- `pub type Apply<S> = fn(&mut S, &[u8]) -> Result<(), String>;` and
  `pub type Decode<S> = fn(&[u8]) -> Result<S, String>;` — **plain function pointers**, not
  closures. `Log::create(path, apply, decode)` / `Log::open(path, apply, decode)` take these.
- `Log::commit(&mut self, txn: Transaction, validate: impl FnOnce(&S) -> Result<(), Rejection>) -> Result<Outcome, LogError>`
  — the closure sees only the *current committed* state and must accept/reject before any
  append; it is never invoked during recovery. **`Log::recover` calls only `apply`** on each
  replayed event's payload, in position order, never `validate`. Consequence: every invariant
  (existence checks, incarnation checks, label validity) must be re-derivable deterministically
  by `apply` from raw payload bytes and prior state alone — exactly the pattern `uc_memory`'s
  `Change::Link` already uses (the incarnation check lives inside `apply_changes`, not only in
  the adapter's extra validator).
- Public types (all `pub`, used today by the separate `uc-memory` crate): `Transaction { request_id: Uuid, payload: Vec<u8>, durability: Durability }`,
  `Durability { D1 = 1, D2 = 2 }`, `Rejection { RequestIdReuse, Uncommitted, Validation(String), Io(String) }`,
  `Outcome { Committed { sequence, physical_ordinal, durability_time, achieved }, Rejected { reason }, Indeterminate { request_id } }`,
  `LogError` (ten variants incl. `NotFound`, `Owned`, `Poisoned`, `Limit`, `Invalid`, `Corrupt`,
  `EnvelopeMismatch`, `HistoryShorterThanCheckpoint`, `CheckpointCacheMismatch`), `Point`
  (`Binding, Reservation, Provisional, Final, Commit, Synced, Checkpoint`), `Position`. Other
  public `Log` methods: `snapshot() -> Arc<S>`, `is_poisoned`, `position`, `history_bytes`,
  `next_request_id() -> Uuid` (domain byte `0x41`), `set_hook`, `wrap_writer`,
  `checkpoint(&self, encoder: impl FnOnce(&S) -> Result<Vec<u8>, String>) -> Result<CheckpointRef, LogError>`.
- The `RDE1`-magic payload rejection and the `UCR1`/`UCE1` envelope framing live entirely inside
  `uc-core` at the `Transaction.payload` level — they apply to **any** domain's raw bytes
  automatically. No domain-level code is needed to get this protection.
- Constants: `MAX_PAYLOAD = 4 MiB`, `MAX_HISTORY = 1 GiB`, `MAX_RECORDS = 1_000_000`,
  `HISTORY_FILE = "history.rf1"`.
- `uc-memory`'s existing `State`/`Slot`/`Change`/`encode_changes`/`decode_changes` are specific
  to Memory's 13-field schema; this work order does not modify `uc-memory`, `uc-harness` or
  their tests, and does not require the new domains' internals to reuse `uc-memory`'s private
  functions (only the public `uc-core` API is a dependency).
- `experiments/unified-commitment/tests/uc-core/tests/recovery.rs` (19 tests) and
  `uc-memory/tests/scenarios.rs` (8 tests) are the established coverage pattern: injected
  process-termination/torn-tail/envelope-mismatch tests at the core layer (already proven,
  domain-agnostic — not re-run per domain), and domain-level scenario tests (conflict rejection,
  delete-then-reopen, replace-supersedes-with-checkpoint, delete+recreate incarnation rejection,
  truncated-batch-final, retry/response-loss) at the adapter layer.
- The workspace `experiments/unified-commitment/Cargo.toml` lists exactly
  `members = ["crates/uc-core", "crates/uc-memory", "crates/uc-harness"]` (verified 2026-09-09);
  adding two more members here needs no CI change, since `.github/workflows/convergence-memory.yml`'s
  existing matrix entry for this workspace already builds/tests every member.

### The legacy Entity and Relation domains (`rusty_multimodal_db` at `232b16e`, reference only — do not add a dependency on this repository)

- **Entity** (`src/generic/entity.rs`, `src/server/entity.rs`): fields `id: Uuid`,
  `label: String` (filterable, not scannable/updatable), `kind: String` (open string, not
  updatable since protocol change to v2), `mention_count: i64` (the **only** durably-updatable
  field, via `UpdateField`), `aliases: Vec<String>` (read-only `StrList`, protocol 11+). Any
  `TransactionOp` on `label`/`kind`/`aliases` is `Unsupported`; only `(mention_count, I64)` is
  accepted.
- **Entity linking is always same-table, open-label.** `valid_relation_label(label)`
  (`src/generic/store.rs:1094`): non-empty, ≤ 64 bytes, first byte not `-`, every byte ASCII
  alphanumeric, `_` or `-`. Any string passing this check creates a new label at first use
  (`LNK-FR-009`); Entity has no fixed relation set at runtime (two labels exist only as seed
  data). `link_records(left, right, relation)`: `Malformed` if the label fails
  `valid_relation_label`; otherwise `MultiLink::link` — `UnknownRecord` if either endpoint has
  no record **in this same store** (Entity never declares a foreign relation), `SelfLoop` if
  `left == right`. `neighbors`/`neighbors_by_relation`/`list_relation_kinds` return the union or
  a filtered/enumerated view of same-table edges. Entity has **no `detach_record`** override
  (falls to the trait default, `Unsupported`).
- **Relation** (`src/generic/relation.rs`, `src/server/relation.rs`, ADR-0058) is its own
  independent record domain, **not** a generic edge type referencing other tables by id.
  Fields: `id: Uuid`, `subject: String`, `relation: String`, `object: String`,
  `created_at_unix_ms: i64`, `updated_at_unix_ms: i64` (the only durably-updatable field),
  `node_id: String` (`""` = unattributed), `deleted_at_unix_ms: i64` (`0` = live). `subject` and
  `object` are **plain strings holding the consumer's id verbatim, with no existence check
  against any table** — ADR-0058 states this as a deliberate, named consequence: *"nothing
  checks that an endpoint names an entity — exactly the consumer's own posture."* Relation has
  **no `link_records`, no `neighbors`, no `detach_record`** (all fall to `Unsupported`);
  `list_relation_kinds()` returns empty. Out-edges/in-edges are queried by `FilterEq subject` /
  `object` equality, not a link mechanism.
- **The only real cross-table link anywhere in the legacy codebase is Memory → Entity**, via
  `RelationDescriptor.target_table` (`Some("entity")` on Memory's `"mentions"` relation) and
  `MultiSymmetric::with_foreign_labels`. This mechanism does not exist for Entity's own labels
  or for Relation at all (Relation is never named as a `target_table` by anything, and it never
  declares any relation of its own). **Entity and Relation have no defined way to reference
  each other today.** A `relationship_mutex` (serve.rs) exists only because this one cross-table
  case needs a lock spanning two separate per-table stores; the unified core's whole point is
  that a single shared `Log`/`State` would not need such a mutex, but that restructuring is
  deferred (see Non-goals).
- The protocol-22 fixture baseline (66 pinned wire vectors, `tests/fixtures/wire-vectors.txt`)
  and the client/server wire protocol are irrelevant to this work order — there is no network
  server or wire format here, only the unified core's own crates.

## Design decisions settled by the host (do not relitigate)

- **Two new, independent domains, each its own `Log`/directory**, exactly parallel to
  `uc-memory`: `uc-entity` and `uc-relation`, both new crate members of the existing
  `experiments/unified-commitment` workspace, both lib crates (tests only, no binary, matching
  `uc-memory`) that depend only on `uc-core` (path dependency within the workspace). Neither
  depends on the other or on `uc-memory`. No new workspace, no CI change.
- **Faithful semantics, not an idealized redesign.** Entity: same-table open-label symmetric
  links (character-class rule quoted above), `mention_count` the only mutable field,
  `label`/`kind`/`aliases` immutable after insert (changeable only by whole-record `Replace`),
  no detach. Relation: standalone record domain, `subject`/`object` plain strings with
  **no existence check against any other table or domain, by design** (this is the legacy
  behavior, not a limitation to fix here), `updated_at_unix_ms` the only mutable field, no
  linking at all.
- **Incarnation on both domains.** Both `uc-entity` and `uc-relation` use the same
  `Slot { incarnation: u64, record: Option<...> }` pattern as `uc-memory`, so a delete followed
  by re-insert of the same id starts a new incarnation and rejects updates/edges naming the old
  one — this is a core hazard the repository already treats as general (Step 3's ADR), not
  specific to Memory.
- **`compact` has no separate verb at this layer.** The legacy `compact()` maps to `Log::checkpoint`
  plus full replay from history; do not invent a distinct "compact" operation.
- **No measurement series, no trace format, no `uc-harness` change.** There is no CMT1-equivalent
  multi-size generator for Entity or Relation; building one is out of scope. Correctness evidence
  is unit/scenario tests (mirroring `uc-memory/tests/scenarios.rs`) plus a small hand-written
  deterministic sequence per domain checked against a plain in-memory model written from the
  operation semantics (not calling the domain's own `apply`), matching AGENTS §5's independence
  requirement. No release binaries, no host-run series, no results index entries for this work
  order.
- **Genuine cross-domain atomicity (Memory ↔ Entity, or any other domain) is deferred.** The
  legacy codebase's only real cross-table case is Memory → Entity; reproducing it faithfully at
  the unified-core layer requires either combining domains into one shared `State` or a
  cross-`Log` coordination protocol, either of which is a materially bigger change touching the
  already-evidence-backed `uc-memory` from Step 3. Record this explicitly as a named follow-on
  in EXP-0004 §18 and in `docs/roadmap/ROADMAP.md`; do not attempt it in this work order and do
  not claim it works.

## Required changes

### R1. Workspace additions

Add to `experiments/unified-commitment/Cargo.toml`'s `[workspace] members`:
`crates/uc-entity`, `crates/uc-relation`. Each crate: `Cargo.toml` (edition/rust-version/lints
matching `uc-memory`'s manifest exactly; `[dependencies] uc-core = { path = "../uc-core" }`;
no other dependencies), `src/lib.rs`, `tests/scenarios.rs`. Do not modify `uc-core`, `uc-memory`,
`uc-harness`, their `Cargo.toml` files, `Cargo.lock` beyond the new members' own entries, or any
existing test file.

### R2. `uc-entity`

- `pub struct Entity { pub id: Uuid, pub label: String, pub kind: String, pub mention_count: i64, pub aliases: Vec<String> }`.
- `pub struct Slot { pub incarnation: u64, pub record: Option<Arc<Entity>> }`;
  `pub struct State { pub slots: BTreeMap<Id, Slot>, pub edges: BTreeSet<(Id, u64, String, Id, u64)>, pub known_labels: BTreeSet<String> }`
  (edge tuple includes the label, since Entity supports multiple labels unlike Memory's fixed
  edge shape). **`known_labels` is a legacy fidelity requirement, not an edge-derived view**:
  `src/generic/entity.rs:222-239` seeds both built-in labels (`RELATION_LABELS =
  ["relates_to", "mentioned_with"]`) at construction even with zero edges, and
  `src/generic/store.rs:1497-1526` retains a label's key when every edge under it is removed
  (e.g. by deleting an endpoint), so `list_relation_kinds`/`neighbors_by_relation` distinguish
  "known label, currently no edges" (`Ok(vec![])`) from "unknown label" (`Malformed`) — do not
  derive the known-label set from `edges` alone (`Default::default()` seeds `known_labels` with
  the two built-in strings, not empty).
- `pub enum Change { Put { record: Box<Entity>, incarnation: u64, insert: bool }, UpdateMentionCount { id: Id, incarnation: u64, value: i64 }, Delete { id: Id, incarnation: u64 }, Link { relation: String, left: Id, left_incarnation: u64, right: Id, right_incarnation: u64 } }`.
  `apply` rules: `Put{insert:true}` requires the next incarnation and rejects a duplicate live
  id; `Put{insert:false}`, `UpdateMentionCount` require the caller's incarnation to equal the
  slot's current live incarnation; `Delete` requires the same incarnation match, sets the slot's
  `record` to `None`, and **removes every edge tuple naming that id from `edges`** (cascade,
  same-table — matching the "only Delete removes records/edges" pattern from Step 1) **without
  touching `known_labels`**; `Link` requires `valid_relation_label(relation)` (implement the exact
  character-class rule quoted above), rejects `left == right` as a self-loop, requires both ids
  to be live at the given incarnation in `slots` (same-table only, matching legacy — never a
  foreign check), and on success both inserts the edge tuple **and** inserts `relation` into
  `known_labels` (a set insert, idempotent, never removed by any operation including `Delete`).
  This must happen inside `apply` itself (not only a commit-time validator), since recovery
  replays only `apply` and must reconstruct the identical `known_labels` set from history alone.
- A versioned text payload format `CME1` (own header line, one line per operation), encode/decode
  round-trip tested, rejecting non-canonical input the same way `uc-memory`'s `CMM2` does
  (canonical-only `decode`, an operation-count cap consistent with `uc-core`'s `MAX_PAYLOAD`).
- `pub struct EntityEngine { ... }` with `create`/`open`/`transact` mirroring `MemoryEngine`'s
  shape (own `Durability`, own directory, `Log<State>`). `neighbors_by_relation(label, id)`:
  `Malformed` if `label` is not in `known_labels`; otherwise `Ok(edges filtered by label and id)`
  (possibly empty). `list_relation_kinds()` returns `known_labels` sorted. The checkpoint encoder
  and decoder must include `known_labels` in the encoded state blob so a reopen from checkpoint
  reconstructs it exactly, and a fresh `State::default()` (used by both a brand-new store and
  full replay's starting point) must seed the two built-in labels — full replay from history and
  checkpoint restore must produce byte-identical `known_labels` sets, tested.
- `tests/scenarios.rs`, each with a stated failure-model label and, where applicable, a
  negative/disabled-mechanism control (mirroring `uc-memory`'s pattern):
  - insert, open-label link, reopen serves the same edges and record.
  - a link with an unknown-label-character-set is rejected before any write.
  - a self-loop link is rejected.
  - a link naming a nonexistent id is rejected (`RecordNotFound`-equivalent `Validation` reason).
  - delete then reinsert the same id: an update naming the old incarnation is rejected, edges
    from the old incarnation do not attach to the new one, including after a checkpoint and
    reopen (mirror `uc-memory`'s `deleted_recreated_uuid_rejects_predecessor_updates_and_edges…`).
  - a fresh, empty `EntityEngine::create` already reports both built-in labels via
    `list_relation_kinds`, and `neighbors_by_relation` on either returns `Ok(vec![])`, not
    `Malformed` — before any Link is ever committed.
  - link two ids under a brand-new label, delete one endpoint, then: `edges` under that label
    for the deleted id are gone, but the label is still in `list_relation_kinds` and
    `neighbors_by_relation(that_label, other_id)` returns `Ok(...)` (known-but-empty or
    whatever remains), never `Malformed`; `neighbors_by_relation("truly-unknown", id)` is
    `Malformed`. Verify this distinction survives a checkpoint and reopen, and separately that
    full replay (no checkpoint) reconstructs the identical `known_labels` set.
  - checkpoint then full replay produce identical state digests, including `known_labels`.
  - a small hand-written deterministic operation sequence (≈20 ops covering insert, two
    distinct labels, a rejected self-loop, mention_count update, delete+reinsert) checked against
    a plain `BTreeMap`-based oracle written independently from the operation semantics.

### R3. `uc-relation`

- `pub struct Relation { pub id: Uuid, pub subject: String, pub relation: String, pub object: String, pub created_at_unix_ms: i64, pub updated_at_unix_ms: i64, pub node_id: String, pub deleted_at_unix_ms: i64 }`.
- `pub struct Slot { pub incarnation: u64, pub record: Option<Arc<Relation>> }`;
  `pub struct State { pub slots: BTreeMap<Id, Slot> }` — no edge set; Relation has no linking.
- `pub enum Change { Put { record: Box<Relation>, incarnation: u64, insert: bool }, UpdateTimestamp { id: Id, incarnation: u64, value: i64 }, Delete { id: Id, incarnation: u64 } }`.
  `apply` rules mirror R2's Put/Delete rules (Delete removes the slot's record but there is no
  edge set to cascade-clear). `Put` (both `insert: true` and `insert: false`, i.e. insert and
  whole-record replace) must reject before any write, matching
  `src/server/relation.rs:154-213`'s `relation_from_fields` exactly: `subject` non-empty,
  `relation` non-empty, `object` non-empty, `deleted_at_unix_ms >= 0`. `created_at_unix_ms` and
  `updated_at_unix_ms` remain unrestricted signed values (no range check); `subject`/`object`
  are validated only for non-emptiness, never for existence against any table or domain — state
  explicitly in the doc comment that this is deliberate, matching ADR-0058's stance verbatim
  ("nothing checks that an endpoint names an entity"). Do not add any restriction beyond these
  four checks (arbitrary non-empty relation labels, arbitrary non-empty endpoint strings and
  signed timestamps are all intentionally unrestricted, matching legacy).
- A versioned text payload format `CMR1`, same round-trip/canonical-decode bar as R2.
- `pub struct RelationEngine { ... }` mirroring `EntityEngine`'s shape.
- `tests/scenarios.rs`:
  - insert with arbitrary (including nonexistent-looking) `subject`/`object` strings succeeds —
    one test must explicitly assert that a `subject` naming no real record anywhere still
    inserts successfully, with a comment citing ADR-0058's "asserts nothing" stance as the
    reason this is correct, not a bug.
  - an insert or a whole-record replace with an empty `relation` string is rejected before any
    write is appended (test both `insert: true` and `insert: false` separately).
  - an insert or a whole-record replace with `deleted_at_unix_ms < 0` is rejected before any
    write is appended (test both modes separately); `deleted_at_unix_ms == 0` and positive
    values, and any signed `created_at_unix_ms`/`updated_at_unix_ms` (including negative), are
    accepted.
  - `updated_at_unix_ms` update, reopen serves the update.
  - delete then reinsert the same id: incarnation increments; an update naming the old
    incarnation is rejected, including after checkpoint and reopen.
  - checkpoint then full replay produce identical state digests.
  - a small hand-written deterministic operation sequence checked against an independent
    `BTreeMap`-based oracle, analogous to R2's.

### R4. Documents (same change, AGENTS §7)

- `docs/experiments/EXP-0004-entity-relation-domains.md` (18 sections, status `Ready`):
  research question = "do Entity and Relation reproduce their existing single-table semantics on
  the unified core with the same recovery/incarnation guarantees Memory has?"; correctness
  invariants = the scenario list above per domain; explicitly state in §12 (baselines) and §18
  (follow-on) that Entity-to-Relation linking does not exist in the source system and is not
  invented here, and that Memory-to-Entity cross-domain atomicity is a named, deferred follow-on
  requiring a shared-state or cross-log design not attempted in this work order.
- `docs/hypotheses/HYP-0004-entity-relation-domains.md` (Open): the unified core's
  recovery/incarnation/checkpoint guarantees generalize to Entity and Relation without any core
  change; a mismatch against either domain's independent oracle or a lost/duplicated commit
  under the existing injected-fault model falsifies this for that domain.
- `docs/adr/ADR-0004-entity-relation-domains.md` (Proposed): records the "faithful semantics, no
  invented cross-reference" decision and the deferred cross-domain-atomicity follow-on,
  authority = merge plan step 4 (first sentence) and this approved work order.
- `docs/experiments/EXP-0003/HOST-DISPOSITIONS.md`-style precedent is not required here (no
  inspection-driven deviation from an approved plan is anticipated), but if any host disposition
  during fix rounds narrows or changes a settled decision above, record it in a
  `docs/experiments/EXP-0004/HOST-DISPOSITIONS.md` the same way.
- `docs/GLOSSARY.md`, `docs/TRACEABILITY.md`, `docs/RESEARCH-QUESTIONS.md`,
  `docs/experiments/README.md` registry row, `docs/PROJECT-STATUS.md`, `experiments/README.md`,
  root `README.md`: the same style of update Steps 2 and 3 made, in the existing list/bullet
  conventions (append after existing entries; do not duplicate paragraphs; keep neutral status
  vocabulary — no "review", "advisory" or "pending" wording in status-bearing documents).
  `AGENTS.md` §10: one sentence authorizing EXP-0004 by reference to merge plan step 4, leaving
  the rest of the paragraph intact, in the same style as the Step 3 sentence already there.
- `docs/roadmap/ROADMAP.md`: add a `MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY` (or similarly named)
  row recording the deferred follow-on as `Deferred`, citing this work order and EXP-0004 §18.

## Non-goals

- No Entity-to-Relation link of any kind (the source system has none).
- No Memory-to-Entity or any other cross-domain atomic transaction; no shared `State` spanning
  two domains; no cross-`Log` coordination protocol.
- No compatibility facade, no network server, no wire protocol, no protocol-fixture reuse, no
  SQL parser/query-layer work (all later Step 4 parts).
- No `uc-core`, `uc-memory` or `uc-harness` changes. No new dependency on `rusty_multimodal_db`.
- No measurement series, no release binaries, no new trace/generator format.
- No changes to Dog, Order, Employee or Reminder (not in scope for "port Memory, Entity and
  Relation first").

## Proof

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

`convergence-memory` and `exp-0001` are included only to prove they remain unaffected (this
work order does not touch either); their proof must be identical in outcome to Step 3's. No
Windows-vs-Linux CI distinction beyond the existing single matrix entry for
`experiments/unified-commitment`, which needs no change since it already builds the whole
workspace.
