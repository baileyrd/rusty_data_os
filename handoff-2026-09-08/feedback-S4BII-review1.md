# Host dispositions on Codex review 1 of the Step 4b-ii work order (plan sha256 d9484b8c…5506fd6a)

All eight findings accepted. Two (F2, F4) revealed the work order's own central ambition — a real
cross-table Memory↔Entity `mentions` link and a cross-domain session — is not buildable on the
frozen 4b-i/EXP-0003/EXP-0004 design without first designing the durable foreign-edge mechanism
already named as deferred (`MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY`) in `docs/roadmap/ROADMAP.md`.
This was put to the owner directly rather than patched around; the owner chose to descope: wire
each domain standalone, open the real listener, verify each domain's own same-table operations
end-to-end, and leave the cross-table relation exactly as deferred as it already was. The other
six findings (F1, F3 as a consequence of F2, F5 as a consequence of F4, F6, F7, F8) are fixed by
spec revision below. New plan hash `4a2096de…c35fe4`; please re-review.

- P4BII-001/F1 (high, `Store: Send + Sync` unsatisfiable): confirmed — `Log<S>`'s
  `writer: Box<dyn Writer>` and `hook: Box<dyn Fn(Point)>` have no `Send` bound
  (`Writer` itself has none either), so no `Mutex<XEngine>` wrapping any real engine can be
  `Send`. New Design decision D7 authorizes adding `+ Send` to both trait-object bounds in
  `uc-core` as a narrow, additive, non-behavioral fix (every real `Writer` — file handles — is
  already `Send`); required as R0, ahead of the adapters, with the existing `uc-core` test suite
  re-run unchanged afterward to confirm nothing else moved.
- P4BII-002/F2 (high, cross-table Link impossible) and P4BII-004/F4 (high, cross-domain session
  impossible): both confirmed by direct source read. Descoped per the owner's decision above —
  see the revised "Goal," the new "Owner scope decision" addendum, and new Design decision D6.
  R2's `MemoryStore::describe()` now reports `target_table: None` (not `Some("entity")`); R6 tests
  only same-table scenarios per domain, explicitly including a same-table exercise of Entity's own
  open-label relation (the one relation scenario that never crosses tables and stays fully in
  scope).
- P4BII-003/F3 (high, silent dangling-edge on cross-table Delete): moot given the F2/F4 descope —
  no `target_table: Some(_)` is ever declared by any of this work order's three adapters, so
  `Registry::detach_across` never has anything to silently ignore. No `detach_record` override is
  required or attempted for any of the three.
- P4BII-005/F5 (medium, `write_batch_checked` unspecified / no cross-table batch op selector):
  the cross-table half is moot per the same descope. R6 now requires only single-domain
  `WriteBatch` (pipelined and atomic) scenarios; R2-R4 do not require overriding
  `write_batch_checked` (the fail-closed default from 4b-i applies, and is not tested as a defect
  here — a real atomic-batch implementation per domain is left to a later work order if needed).
- P4BII-006/F6 (medium, 1024 vs. 4096 operation-cap mismatch): confirmed — `uc-memory`'s literal
  `1024` check and `uc-entity`/`uc-relation`'s named `MAX_OPERATIONS: usize = 1024` all sit below
  the wire protocol's `MAX_STAGED_OPS`/`MAX_BATCH_OPS = 4096`. New Design decision D7 authorizes
  raising all three to 4096 (naming a `MAX_OPERATIONS` constant in `uc-memory` to match the other
  two's convention) as R0, with each engine's existing tests re-run to confirm the raised cap
  changes no existing test's expected outcome.
- P4BII-007/F7 (medium, `cm-trace` types inaccessible to `uc-facade`): confirmed — `uc-memory`
  privately imports `Memory`/`Value` without re-exporting them. R1 now adds a direct
  workspace-path dependency on `cm-trace` to `uc-facade` (an existing workspace member; no new
  external dependency), rather than requiring a re-export change to `uc-memory`.
- P4BII-008/F8 (medium, aggregate `Sum`/`Avg` overflow panic): confirmed — `overflow-checks = true`
  is set for every profile in `experiments/unified-commitment/Cargo.toml`, and `query.rs`'s
  `reduce` sums `i64` values with no overflow protection. This is a real, pre-existing defect in
  already-closed 4b-i infrastructure, only now operationally reachable because this work order
  opens the first real listener. New Design decision D7 authorizes fixing it now (accumulate in
  `i128`, narrow to `i64`/`f64` only when the result fits, an explicit disposition — not a silent
  wrap or a panic — when it does not), required as R0, logged in `BUILD-LOG.md` as a 4b-i residual
  closed here rather than a new 4b-ii defect.
