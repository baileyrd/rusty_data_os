# Host dispositions after Part B build round 1 (snapshot b9ecabcc…36a6, base abda0a7)

Host proof on your snapshot (independent, Windows, GNU toolchain): fmt PASS; clippy ×3 PASS;
`cargo test --features server,research --no-fail-fast` 741 passed / 0 failed; `--features client`
238 passed; Python 5/5; `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features
server,research` PASS. The four probes pass and the negative control is accepted.

Implement every item below in this checkout (still no commit), rerun the full proof chain, and
refresh the report. Keep LF line endings.

## K1 (medium, scope) — revert the two out-of-scope Windows portability edits

The owner's standing disposition for these work orders is that the two pre-existing Windows-only
failures (`dog_server` `certificate_classes_from_env_values_follows_the_documented_table`, path
list split on `:`; `tests/server_python_client.rs`, `python3` launcher) are excluded by the host
and must not be fixed as part of this work order. Restore `src/bin/dog_server.rs` and
`tests/server_python_client.rs` exactly to their `abda0a7` content (read it with
`git show abda0a7:<path>` and write the file; do not run `git checkout` or `git reset`). Then:

- In the report, replace deviation 4 with: the two Windows-only tests fail on this machine as
  before and are excluded by host disposition (CI on Linux passes both); a separate portability
  change may address them later. Do not claim "no exclusions".
- The report's proof table must say the server suite passes except those two, and state that
  the host runs it with `--no-fail-fast`.

## K2 (medium, efficiency and error contract) — the post-mutation checkpoint is redundant

With the mutation gate held, the pre-mutation `checkpoint` leaves the journal header-only and
nothing can append until the gate is released (every append path takes the gate). So the second
`checkpoint` in `with_mutation` (Entity/Memory/Relation) and in Dog's journaled `update_field`
never has redo to drop; it only adds one image flush plus one truncate+sync per bypass write,
doubling the cost on the shipped journal-enabled Memory server, and it turns an error after an
already-applied mutation into a `Storage`/`Journal` answer that hides success.

Fix: make `CommitGroup::checkpoint` return `Ok(())` without flushing or truncating when the
journal is already header-only (check `len == HEADER_LEN` under the state lock), and drop the
post-mutation call sites (or keep them; they become free). Then tighten the contract back: a
`Storage`/`Journal` error from a bypass mutation means nothing was applied, apart from the
mutation's own I/O failure exactly as before this change. Update ADR-0061 (decision and
failure-model text), the `ErrorCode::Storage`/`Journal` docs in `src/server/protocol.rs`, the
ADR-0025/0026 amendment paragraphs and the SERVER-001 amendment paragraph accordingly. Keep
every header-only assertion in the tests; add one unit test showing consecutive bypass writes on
a journaled adapter perform no truncation (for example count `checkpoint_flush` calls through a
wrapper, or assert on the journal file's length and modification time), or state why that is not
testable without more plumbing. If you disagree with the analysis, say exactly which path can
append while the gate is held instead of implementing.

## K3 (low, docs) — no review/inspection vocabulary in spec and index documents

Part A's inspection disposition (I6) removed review vocabulary from spec/index docs. Apply the
same rule here: `docs/PROJECT-STATUS.md` banner, `docs/roadmap/ROADMAP.md` intro sentence and
the `LEGACY-RECOVERY-ISOLATION-REPAIR` exit gate/evidence cells, the `TRACEABILITY.md` row
(PR/release and State cells), ADR-0061's Status line, and the SERVER-001 0.50.2 change-history
entry ("part A's final inspection residuals"). Use neutral status vocabulary: ADR-0061
"Status: Accepted (implemented 2026-09-08)"; state "Implemented"; describe residual closures as
"the four FR-060 documentation follow-ups (FR-060 text, client docstrings, concurrent far
insert, throughput measurement)". The report itself may keep "advisory" wording.

## K4 (low, docs) — dead links into `target/`

`docs/reports/2026-09-08-recovery-isolation-repair.md` links four logs under `../../target/`
(ignored, never committed). Replace the links with plain text naming the local log files as
uncommitted artifacts, keeping the inline results (the table already carries them). The
negative-control paragraph keeps its four failure lines inline.

## K5 (low, nits)

- SERVER-001 change history: the 0.50.2 entry sits above 0.1.0 while 0.50.1 is the last entry;
  move 0.50.2 directly after 0.50.1 and remove the doubled blank line.
- `clients/python/rusty_multimodal_db/client.py` docstring: missing space in "sent.In atomic".
- `src/server/relation.rs` journaled `apply_transaction` still re-runs `check_read_set` inside
  the apply closure while Entity/Memory/Dog dropped it; remove it for consistency (the gate makes
  it redundant) or add a one-line comment saying why it stays.
