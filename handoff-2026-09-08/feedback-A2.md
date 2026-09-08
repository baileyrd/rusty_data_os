# Host dispositions after inspecting build round A2 (Step 1 part A)

The host (Claude) read the full diff relative to `478eeda` and independently ran the proof
chain. Results: fmt PASS; all three Clippy runs PASS; Python 5/5 PASS; `cargo test` for both
feature sets fails ONLY on the pre-existing Windows-only `insert_log` test (confirmed failing
on the untouched baseline archive in `target/batch-repair-baseline`); with that test skipped,
the full `server,research` suite passes except two more pre-existing Windows-only failures
(`dog_server` certificate-path `:` split; `python3` binary name in the live Python test).
The new regressions and the corrected legacy batch assertion pass (17/17 memory integration).

Fix the items below in this same checkout, then rerun the exact proof chain and report the
raw outputs again. Do not commit. Do not change scope beyond these items.

## F1 (high) — the server request mutex serializes every request, including reads

`handle_connection` now takes one `Mutex<()>` around every dispatched request (reads, Page,
Query, Join, sessions, Commit, Compact, and every write), for all connections and all tables.
That turns the whole server into a single-lane executor and is a throughput regression for
the shipped consumers that the work order did not ask for and the merge plan warns against.

Required: hold the server-level section only for requests that can change record existence
or edges across tables, i.e. `Link`, `Delete` and `WriteBatch` (both modes). Justification you
must verify and document: only `Delete` (single or batched) removes a record, so a far-endpoint
check that passed cannot become dangling unless a delete interleaves, and deletes take the same
section; `Insert`/`Replace`/`ReplaceIf`/`UpdateField`/`Transaction`/`Commit` never remove records
or edges of other tables; reads need no section because the per-adapter locks already give
each individual read a consistent view. Keep `Compact` outside unless you find a concrete
reason and state it. Update the ADR-0060 acceptance entry, the `serve_tables` docs, the report
and the "including reads" statements accordingly; keep the concurrent regression test, and add
a small test showing a read on one connection is not blocked by another connection's open
session (or an equivalent non-serialization check) if practical.

## F2 (high) — `Reminder` atomic batches now refuse with `Unsupported`

The default `write_batch_checked` returns `Err((0, Unsupported))`, and `ReminderConnectionStore`
(consumer-facing, `rusty_remind_me`, runtime inserts/replaces/deletes, `GenericProductionStore`
with `with_exclusive`) has no override, so an atomic `WriteBatch` on the `reminder` table that
applied per-op at the baseline now fails outright. Implement `write_batch_checked` for
`Reminder` like `Relation` (prepare every op, run `check(i)` in order, apply under one
`with_exclusive`; it has no links, so no endpoint overlay is needed). Add an integration
regression in `tests/server_reminder_integration.rs`: atomic success applies all; atomic
rejection (e.g. a bad field list at op 1) applies nothing; pipelined stays per-op. Leave
`Dog`/`Order`/`Employee` refusing, but say so in ADR-0060 and the SERVER-001 spec text so the
change from "per-op with abort" to "refuse" for adapters without an atomic implementation is an
explicit, documented behavior change.

## F3 (medium) — pre-existing Windows failure that blocks the agreed proof

`src/generic/insert_log.rs:187` does `std::fs::File::open(&tmp)?.sync_data()?` on a read-only
handle, which Windows refuses (`Access is denied`). The host authorizes this one-line repair
as a deviation from the "no recovery changes" rule because it is a durability helper, not
replay semantics, and it blocks the mandatory proof on the development platform: open the
temporary file with write access (`OpenOptions::new().write(true).open(&tmp)?` or read+write)
before `sync_data`, with no other behavior change. Mention it in the report. Do NOT touch the
`dog_server` `:`-split test or the `python3` binary name; list them as known Windows-only
failures in the report's limitations.

## F4 (low) — poisoned request mutex closes the connection

On `request_lock.lock()` returning `Err`, the connection answers `Storage` and returns. The
guard protects a unit value, so recover with `unwrap_or_else(|p| p.into_inner())` instead of
turning one panic on another connection into a permanent refusal. Apply the same to any
other place you added a poison branch.

## F5 (low) — report wording

The report says "the mandatory proof is not green" because of the `insert_log` test. After F3
it should be green locally except the two listed Windows-only failures, which are excluded
from the agreed proof by the host's disposition. Update the verification table with the new
raw results and keep the "isolated baseline reproduces both defects" evidence.

## Confirmed and accepted as built (no change requested)

- Table-aware batch Link validation, cross-table detach on batched Delete, intra-batch
  existence overlay, earliest-index rejection, atomic rejection applying nothing.
- Both Clippy sites via `as_chunks`; MSRV: `as_chunks` is stable since 1.88 (host check result
  is recorded separately).
- The four new integration tests and the corrected legacy assertion.
