# uc-facade — Step 4b-ii

Experimental MemoryStore, EntityStore and RelationStore implement uc-protocol::Store over
the existing domain engines. Construct an engine with its explicit create/open and D1/D2
choice, then pass ownership to the corresponding Store::new. Each engine has its own
directory/log and table mutex. No production, performance or stronger durability claim.

Use one Registry containing the three Arc-backed Stores with memory as primary.
LoopbackListener::bind always binds 127.0.0.1:0; local_addr returns the assigned address.
serve(listener, registry, stop) accepts on that listener, starts a thread per connection,
and uses buffered reader/writer halves with the existing handle_connection state machine.
The stop AtomicBool ends acceptance; close clients before joining serve. All worker handles
are joined on exit. Connection I/O errors end only that connection; a worker panic makes
serve return an error after joining. This thin test listener has no deployment configuration,
authentication enforcement, admission limit or idle-client shutdown facility.

Writes map wire UUID bytes directly to raw domain IDs/Uuid. Live incarnations and next
insert incarnations come from engine state under the mutex. Session commits check tracked
reads before write preconditions, report the first invalid update index, and submit all
changes in one transact. Empty transactions only validate reads. Whole-record ReplaceIf
uses uc-protocol's shared guard validation/evaluation under the same lock as replacement.
That frozen validator rejects StrList predicates even for Eq/Ne: guards on Memory tags
and Entity aliases return Malformed. The work order's broader "any field" wording cannot
be met simultaneously with its shared-validator/no-protocol-change requirements; the
report proposes retaining this admission boundary pending independent review.
Field sets must contain every tag exactly once with the declared type; wire order is free.
Memory's access_count must be nonnegative; other domain constraints stay in their engines.

Engine validation errors map to the matching NotFound/Conflict/Duplicate/GuardFailed code
where available, otherwise Malformed. Log errors, I/O refusals and indeterminate outcomes
map to Storage. The wire cannot express the core's stable request identity or resolve an
indeterminate commit. No retry is invented. Error-returning methods map a poisoned mutex
to Storage; infallible get/scan_all/Entity label metadata fail by panic instead of exposing
potentially inconsistent data as an empty result. This is fail-stop behavior.

Nonempty atomic WriteBatch returns (0, Unsupported) without writes. Non-atomic batches
remain per-operation. Construct EntityStore first and pass the same Arc<dyn Store> to
MemoryStore::new(engine, entity) and Registry. Memory mentions targets entity and lists
only the named descriptor; a bare Join is Malformed. Entity retains same-table wildcard
and named relations. Memory reads only foreign_edges, checks the far live incarnation,
and resolves raw-id collisions as the local Memory direction. Count excludes stale edges.
Detach removes all tuples targeting an id; Memory deletion removes its outgoing tuples.

Memory logs/checkpoints use CMM3/CMS3, with no CMM2/CMS2 migration. Replay never accesses
Entity. A crash between Entity delete and Memory detach may leave stale tuples, which
live reads hide even after same-id reinsertion. Reads never clean up stored tuples.
The existing registry mutex spans Join as well as Link/Delete/WriteBatch, serializing
row fetch against relationship mutations. Call writes through that shared registry.
Parent/children and compact remain Unsupported; Relation has no relation layer.

The [method](../../../../docs/experiments/EXP-0005-protocol-facade.md) and
[Step 4c report](../../../../docs/experiments/EXP-0005/STEP4C-IMPLEMENTATION-REPORT.md)
record authority, proof and limitations. Tests use D1 ordinary writes and real TCP.
No legacy client, cross-domain session, atomic cross-log cleanup or performance claim.
