# claudex-loop codex-build log — Data OS / Multimodal merge plan, 2026-09-08

## Roles, scope, budgets

- Host: Claude Code (Fable 5.1, session 011ChCoDzvZxMP1gAuRNWEk3). Builder: Codex CLI 0.153.4 (config default model `gpt-6-astra`, effort high). Inspector: fresh Codex session. No `--model`/`--effort` overrides requested.
- Spec: `data-os-multimodal-merge-plan-2026-09-08.md` (untracked in rusty_data_os). Status: **unreviewed spec** (no claudex-loop plan-review round; the user invoked codex-build directly). Delegated via `--unreviewed-spec`.
- Scope decision (host): the plan's stated "immediate sequence" is Step 1, then Step 2, then Step 3. Step 1 is split into part A (batch Link/Delete repair + Clippy + CI) and part B (recovery/isolation probes, repair-or-prevent, snapshot contract) so each build is verifiable. Work orders: `step1-batch-repair-spec.md`, `step1b-recovery-probes-spec.md`, `step2-convergence-memory-spec.md` (this directory).
- Budgets: MAX_FIX_ROUNDS=2, MAX_INSPECTION_ROUNDS=2 per work order. inspect=on.
- Baselines: rusty_multimodal_db `478eeda4544aa98c4948aebc66b5ff8fad609645` (clean; branch `codex/merge-step1-batch-cross-table` created from it, no changes yet); rusty_data_os `79d51e9404c8b35dee2662aa657b602e9061b44b` (only the untracked plan file).
- Recon facts: stable Clippy (cargo 1.98.0) fails at `src/durability/mmap_store.rs:699` (default features) and `src/server/pem.rs:110` (client feature). `~/Downloads/rusty_multimodal_db-review-evidence-2026-09-08.zip` holds the four recovery/isolation probes (reviewed at 46817b9; copied to `probes-lib.rs`). The merge-plan evidence zip with the four batch probes was not on disk; the part A work order re-derives them from the plan and the merge-plan review section 4. Rust 1.88.0 (minimal profile) installed by the host for the MSRV proof.

## Round A1 — Codex build attempt (Step 1 part A)

- Command: `runner.py build --host claude --builder codex --repo C:/dev/rusty_multimodal_db --plan step1-batch-repair-spec.md --unreviewed-spec --proof "<fmt, 3x clippy, 2x test, python unittest>" --timeout 7200`.
- Artifacts: `runs/step1a-build/claudex-w305ve5p/` (result.json exit 0, session `01a0827f-b431-7201-898f-7366b89625e5`, plan sha256 `0d6affb5…3112`, base `478eeda`).
- Outcome: **no changes** (snapshot files list empty). Codex reported every command, including `git status` and `rg --files` and the proof chain, rejected with `CreateProcess … rejected: blocked by policy`. Not a build failure; the CLI's exec policy refused all commands. This is NOT approval and NOT a completed build.
- Diagnosis (host): `codex doctor` reports `sandbox backend: disabled` on this Windows install, so with `approval_policy=never` every command needs an escalation that can never be granted. Verified fixes by one-off `-c` overrides (no global state changed):
  - `windows.sandbox="unelevated"` makes commands run in the restricted-token sandbox (smoke: file written in workspace, `git status` executed).
  - Inside that sandbox `git`, `cargo`, `python` were not found: the user's PowerShell profile (`…/OneDrive - US Army/Documents/PowerShell/Microsoft.PowerShell_profile.ps1`, lines ~85-118) rebuilds `$env:Path` from the registry plus `*_HOME` variables, dropping the naner portable-tool directories. With `shell_environment_policy.inherit="all"` and `set.GIT_HOME/RUST_HOME/PYTHON_HOME/NODE_HOME/USER_BIN` pointing at the naner vendor dirs, all four tools resolve (smoke-d.txt: git 2.55.0, cargo 1.98.0, Python 3.14.7, gcc 16.2.0).
  - Cargo also needs `CARGO_HOME` writable for its package-cache lock: `sandbox_workspace_write.writable_roots = ['C:\tools\naner\vendor\rust\.cargo']` (not yet smoke-tested).
- Blocker: persisting those settings in `~/.codex/config.toml` (shell append and the Edit tool) was denied by the Claude Code auto-mode permission classifier, twice. The runner accepts no extra CLI flags, so a per-invocation override cannot be passed through it. Backup of the untouched config: `config.toml.bak-2026-09-08`. Awaiting the user's decision; no workaround attempted.
- Note: Codex itself added `[projects.'c:\dev\rusty_multimodal_db'] trust_level = "trusted"` to the config during the smoke tests.

## Round A2 — Codex build relaunch (Step 1 part A)

- User applied the config snippet to `~/.codex/config.toml` (confirmed by reading the file). Smoke test with the persisted config only (smoke-e.txt): `git status`, `cargo --version`, `cargo metadata --offline` (takes the package-cache lock) and `cargo fmt --check` all ran inside the sandbox.
- Relaunched the same runner command as A1 (same spec sha256, same proof, `--timeout 7200`). Artifacts under `runs/step1a-build/` (new run directory). Awaiting completion.

### Round A2 result

- Artifacts: `runs/step1a-build/claudex-wi2i56oh/` (session `01a08298-637a-7280-ad8f-67e5608b295a`, 1636 s, exit 0, snapshot sha256 `54b70cd5…e83c`, 14 files changed: 6 Rust sources, 1 test file, 6 docs, 1 new report). Codex's own verdict: proof not green because of the unchanged `insert_log` test.
- Host proof (`proof-A2.log`, `proof-A2b.log`): fmt PASS; clippy server,research / default / client PASS; python 5/5 PASS; `cargo test --features server,research` and `--features client` each FAIL only on `generic::insert_log::tests::a_version_1_log_reads_as_items_and_upgrades_on_append` (Windows `Access is denied` at `insert_log.rs:552`, root cause `File::open(&tmp)?.sync_data()?` at line 187). Confirmed pre-existing: same test fails on Codex's untouched baseline archive. Full `server,research` suite with that test skipped: all targets pass except two more pre-existing Windows-only failures (`dog_server` cert-path `:` split; live Python test needs `python3`). Memory integration 17/17 including the four new regressions.
- MSRV: first attempt used the `1.88.0` msvc-host toolchain and failed at link (no MSVC linker on this box), not a code failure. Installed `1.88.0-x86_64-pc-windows-gnu`; `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research` PASS (1m12s) on the A2 snapshot.
- Host inspection of the diff (all 14 files read): batch Link/Delete repair, intra-batch existence overlay and cross-table detach are correct and tested over real sockets. Findings sent as `feedback-A2.md`: F1 (high) server mutex serializes all requests including reads → restrict to Link/Delete/WriteBatch; F2 (high) Reminder atomic batches now refuse with Unsupported (default `write_batch_checked`) → implement for Reminder + regression, document refusal for reference adapters; F3 (medium) authorize one-line Windows fix for `insert_log.rs:187` (open with write access before `sync_data`); F4 (low) recover a poisoned request mutex via `into_inner`; F5 (low) refresh report proof table.

## Round A3 — Codex fix round 1

- Command: same as A2 plus `--resume runs/step1a-build/claudex-wi2i56oh/result.json --feedback feedback-A2.md`.
- Result: `runs/step1a-build/claudex-otxqvcjg/` (same session resumed, 678 s, exit 0, snapshot sha256 `e2770bdd…f7e5`, 16 tracked files + 1 new report). F1: lock now taken only for Link/Delete/WriteBatch (`matches!` in `handle_connection`), docs updated with the "only Delete removes records" argument. F2: `ReminderConnectionStore` gains `write_batch`/`write_batch_checked` (prepare then apply under `with_exclusive`), two socket regressions in `tests/server_reminder_integration.rs`; refusal for Dog/Order/Employee documented in ADR-0060 and SERVER-001. F3: `insert_log.rs:187` opens the temp file with write access before `sync_data`. F4: `unwrap_or_else(|p| p.into_inner())` plus a unit test that a held section does not block reads and poison recovers. F5: report refreshed.
- Host proof on this snapshot (`proof-A3.log`): fmt PASS; clippy ×3 PASS; `cargo test --features server,research --no-fail-fast`: lib 519/519, all integration targets pass including memory 17/17 and reminder, except the two pre-existing Windows-only failures (`dog_server` cert-path `:` split; live Python test wants `python3`) which the host excludes by disposition; `cargo test --features client` PASS (exit 0); python 5/5 PASS; `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research` PASS.
- Host diff review of the round: all five dispositions implemented as asked; no new concerns.

## Inspection — fresh Claude CLI (the other provider from the Codex builder)

- First attempt (`runs/step1a-inspect/claudex-a786zujv`) failed in 0.15 s: runner refused the npm `claude.CMD` shim. Relaunched with `--cli C:/tools/naner/home/.npm-global/node_modules/@anthropic-ai/claude-code/bin/claude.exe` (version 2.1.263 verified). - Result: `runs/step1a-inspect/claudex-myyqu817/` (session `ea7b3371-76d5-470b-950b-bb3e28fab7e5`, 377 s, observed models claude-fable-5-1 + claude-haiku-4-5, snapshot sha256 `e2770bdd…f7e5`). **Verdict: APPROVED.** Six low findings: F1 `dispatch`/trait docs drift (table-local vs served path); F2 Employee atomic batches now refused (undocumented, untested); F3 concurrent test drops JoinHandles so assertion failures surface as timeouts; F4 atomic detach `Storage` answered as `TransactionFailed` overloads the "nothing applied" meaning; F5 insert-log fix authorization not traceable from the repo; F6 review vocabulary in spec/index docs. Limitations: no test execution (host ran proof); compaction fold not read line-by-line; self-target overlay verified by unit test only.
- Host dispositions (`feedback-A3.md`): accept all six; F4 resolved as `BatchResults` with `Failed(Storage)` in the Delete slot (matches pipelined, avoids a false "nothing applied" retry); F5 by recording the authorization and committing the insert-log fix separately.

## Round A4 — Codex fix round 2 (final fix round; MAX_FIX_ROUNDS reached)

- Command: resume `runs/step1a-build/claudex-otxqvcjg/result.json` with `--feedback feedback-A3.md`.
- Result: `runs/step1a-build/claudex-bv73ptpj/` (544 s, exit 0, snapshot sha256 `9c9a8409…d64c`, 17 tracked files + report; adds `src/server/employee.rs`). Host diff check: I1 docs added; I2 Employee doc line + research-gated unit test; I3 writer threads joined; I4 atomic detach `Storage` now `BatchResults` with `Failed(Storage)` in the Delete slot (loop continues over later deletes); I5 authorization recorded; I6 SERVER-001 bumped to v0.50.1 with a change-history entry in spec vocabulary, index banners replaced by one neutral pointer each, no review vocabulary left in repo docs.
- Host proof (`proof-A4.log`): fmt PASS; clippy ×3 PASS; `cargo test --features server,research --no-fail-fast`: 958 passed, 2 failed — exactly the two excluded Windows-only tests; `cargo test --features client` exit 0; python 5/5; `cargo +1.88.0-x86_64-pc-windows-gnu check` PASS.

## Inspection round 2 — fresh Claude CLI on snapshot `9c9a8409…d64c`

- Result: `runs/step1a-inspect/claudex-vfmvlvve/` (session `d5319f38-89bc-4c65-b8bf-8f13d31508e5`, 543 s). **Verdict: REVISE.** F1 (medium) Employee served atomic batch refuses where single Link succeeds (regression vs baseline on a research-gated adapter); F2 (low) SERVER-002 §20 and FR-060 traceability row stale; F3 (low) relationship mutex also taken on single-table/no-foreign-relation servers and across whole pipelined batches; F4 (low, pre-existing) pipelined `ReplaceIf` skips `validate_predicate`, so an invalid guard answers `GuardFailed` instead of the single request's error.
- Budgets (2 fix rounds, 2 inspections) exhausted. Owner decision (asked, 2026-09-08): extend by one fix round + one inspection; implement Employee atomic support rather than accept the refusal.
- Host dispositions (`feedback-A4.md`): J1 implement Employee `write_batch_checked` + agreement test; J2 SERVER-002 §20 + traceability sentence; J3 skip the section when no registered relation has a `target_table`, document remaining serialization, bench as follow-up; J4 validate `ReplaceIf` guard in the pipelined path + assertion.

## Round A5 — Codex fix round 3 (owner-extended)

- First launch (`runs/step1a-build/claudex-hq82ggfy`) failed in 11 s with no changes: the network redirected `chatgpt.com`/`api.openai.com` (HTTP 307) to an Army web-security appliance and Codex's token refresh failed. Re-probed later: redirect gone. Relaunched.
- Result: `runs/step1a-build/claudex-w1ywosv1/` (592 s, exit 0, snapshot sha256 `f384eb83…f39d`, 19 files incl. `SERVER-002-wire-format.md`). Host diff check: J1 Employee `write_batch_checked` (links only, existence overlay, self-loop Malformed, other ops Unsupported at preflight) with agreement test; J2 SERVER-002 §20 + traceability row; J3 `relationship_mutex` created only when a registered table declares a `target_table` relation, `relationship_section` taken only for Link/Delete/WriteBatch, unit test; J4 `validate_predicate` for pipelined `ReplaceIf` in `apply_write_op`. Residual note (host): a single-table Memory server still declares `mentions -> entity`, so it still creates the mutex; conservative, not a defect.
- Host proof (`proof-A5.log`): fmt PASS; clippy ×3 PASS; server,research suite 959 passed, 2 failed (the two excluded Windows-only tests); client suite exit 0; python OK; MSRV check result recorded below.

## Inspection round 3 — fresh Claude CLI on snapshot `f384eb83…f39d`

- Result: `runs/step1a-inspect/claudex-tubzdkss/` (session `f9060d8f-f8e6-4dd8-bb03-ae8d2b5d49ae`, 371 s). **Verdict: APPROVED.** Four low findings, none material: F1 SERVER-001 FR-060 requirement sentence (line ~98) still describes the pre-repair WBT-FR-003 contract; F2 pipelined batches hold the relationship mutex for their whole duration (owner trade-off; bench row suggested); F3 an atomic batch may spuriously answer RecordNotFound for a far record inserted concurrently (never a dangling edge; document or include Insert in the section); F4 `SchemaDrivenClient::write_batch` and the Python client docstrings do not mention the atomic `Failed(Storage)` detach slot.
- Budgets exhausted (owner extension used). Host disposition: accept the approval; carry F1–F4 as residuals into the part B work order (all documentation or measured trade-offs). No further edits to the inspected snapshot.

## Part A closed

- MSRV on the final snapshot: `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research` PASS.
- Committed on `codex/merge-step1-batch-cross-table` (not pushed): `c1d2640` insert-log write-handle fix (separate, per inspector F5), `abda0a7` batch repair + Clippy + docs + tests. Working tree clean at `abda0a7`.
- Residuals F1–F4 from inspection round 3 added to the part B work order as R6.

## Part B — Codex build (Step 1 part B: recovery/isolation probes, repair-or-prevent, snapshot contract)

- Spec: `step1b-recovery-probes-spec.md` (unreviewed spec, `--unreviewed-spec`), baseline `abda0a7`. Artifacts under `runs/step1b-build/`. Same proof chain. Budgets reset: MAX_FIX_ROUNDS=2, MAX_INSPECTION_ROUNDS=2. Awaiting completion.

## Pending

- Step 1 part A build (Codex) → host proof + fresh Codex inspection → commit on the feature branch (commit/push not yet authorized beyond "implement the plan"; host will commit on the branch and not push unless told).
- Step 1 part B, then Step 2 (Data OS), same loop.

## Part B — resumed by a new host session (2026-09-08, session 015iJ5WR5HBzxPMq8koUZgzw)

- Found: the first Part B run (`claudex-6ndzci7p`, old scratchpad, base `abda0a7`, plan sha256 `d095d77a…57a7`) was killed at owner shutdown with `result.json` status `running`; no completed turn, so `--resume` is impossible. Codex had rewritten seven adapters through a Python `write_text` script, turning every touched file into a whole-file CRLF diff (real content ~386 lines: a `with_mutation` checkpoint-around-mutation helper, `CommitGroup::admission`/`mutation`, and `tests/server_recovery_probes.rs`). Its `recovery-baseline.log` confirms all four probes fail at `abda0a7` (count 99 vs 1; `Replay { batch: 0, index: 0, code: RecordNotFound }`; 10 vs 20; snapshot read observes later commit).
- Disposition: follow the handoff recommendation and relaunch from the clean tree. The partial work stays in `stash@{0}` (not applied, not dropped) as evidence only; it was never proved or inspected.
- Spec change: one hygiene sentence added to `step1b-recovery-probes-spec.md` (keep LF line endings; no whole-file rewrites). Unreviewed spec, new sha recorded by the runner.
- Pre-launch checks: no Codex/runner process alive; `api.openai.com` answers 401 with no redirect; Part A branch is in sync with `origin`; codex-cli 0.153.4, claude.exe 2.1.265, runner 2.1.0.
- Part B relaunched: `runs/step1b-build/claudex-p6py8dbc/` (fresh session, base `abda0a7`, `--timeout 7200`, same seven-command proof). Awaiting completion.

## Step 2 preparation (host, while Part B builds)

- Installed `1.89.0-x86_64-pc-windows-gnu` (minimal, rustfmt+clippy). rustup's default host is msvc with no linker, so `cargo +1.89.0` must be spelled `+1.89.0-x86_64-pc-windows-gnu` on this box.
- exp-0001 baseline on Windows with that toolchain (`exp0001-precheck.log`, `exp0001-precheck2.log`): fmt PASS; full-workspace clippy FAILS only because `exp1-descriptive-d1-harness` is Linux-only by design; with `--workspace --exclude exp1-descriptive-d1-harness` clippy PASS and tests PASS (every target 0 failed). CI on Linux covers the full workspace.
- Step 2 spec amended (host decisions, unreviewed spec): target is a build worktree `C:/dev/rusty_data_os-step2` on branch `codex/merge-step2-convergence-memory` created from `2ca5b61` (main `79d51e9` + the committed merge plan/handoff docs, so the plan file the spec moves to `docs/plans/` exists); legacy runner pins `abda0a7e94a9727e410001e9724edc741f6e0d31` (part A, pushed; part B is not pushed so cannot be pinned); Windows proof spelling and the exp-0001 exclusion recorded in the Proof section. Worktree created, clean at `2ca5b61`.

### Part B round B1 result (relaunch)

- Result: `runs/step1b-build/claudex-p6py8dbc/` (session `01a0835a-4032-7ec3-9e55-66d994b01fb6`, 1648 s, exit 0, plan sha256 `42ff80fb…c5c5`, snapshot sha256 `b9ecabcc…36a6`, 25 files: 6 runtime sources, protocol/client docstrings, Python docstring, 3 tests incl. new `tests/server_recovery_probes.rs` (12 tests), 10 docs incl. new ADR-0061 and report). All LF, no CRLF churn.
- Codex approach: mutation gate (`CommitGroup::lock_mutations`) held from exclusive preflight (validate + read-set check) through append/fsync/apply; `commit_serialized` truncates a refused attempt back to the prior offset; every bypass mutation runs in `with_mutation` (checkpoint = image flush + journal truncate before and after, under gate + exclusive section); replay reports a complete inapplicable batch; snapshot sessions preserve and serve first tracked values (`record_read_set` keeps the first value, overlay in `handle_connection`, even after deletion). Negative control: entity.rs + serve.rs restored to `abda0a7` gives 0/4 probes passing.
- Host proof (`proof-B1.log`): fmt PASS; clippy ×3 PASS; server,research 741 passed / 0 failed (incl. the two formerly excluded Windows tests, because Codex patched them); client 238 passed; python 5/5; MSRV `+1.88.0-x86_64-pc-windows-gnu check` PASS.
- Host review (all 25 files read): journal cleanup bookkeeping consistent (`len` is bytes, sequence counters untouched); every trait mutator on the four adapters goes through the gate (Entity/Relation/Dog have no `detach_record` override, Memory's takes the gate; served pipelined ops call the per-op methods). Findings in `feedback-B1.md`: K1 (medium) revert out-of-scope `dog_server.rs` and `server_python_client.rs` edits per the owner's standing disposition; K2 (medium) post-mutation checkpoint redundant under the gate, doubles flush cost on journaled Memory and hides an applied mutation behind `Storage`; K3 (low) review vocabulary reintroduced in index/spec docs; K4 (low) report links into ignored `target/`; K5 (low) change-history order, docstring space, Relation's redundant read-set re-check. Documented trade-off accepted by the host: writer serialization supersedes GRP-FR-002 grouping on the four repaired adapters (spec R1 allowed it; measured follow-up recorded in ROADMAP).
- Fix round 1 launched (resume of the same session, `--feedback feedback-B1.md`). Budgets: fix rounds used 1/2, inspections 0/2.

### Part B round B2 — Codex fix round 1 result

- Result: `runs/step1b-build/claudex-qklsm873/` (same session resumed, 491 s, exit 0, snapshot sha256 `ee3392c7…4409`, 23 files; `src/bin/dog_server.rs` and `tests/server_python_client.rs` restored byte-for-byte to `abda0a7`, verified by the host with `git diff abda0a7` on both paths).
- Host diff check of the round: K1 done; K2 `CommitGroup::checkpoint` returns early when `len == HEADER_LEN`, all four post-mutation checkpoint calls removed, new `consecutive_journaled_bypass_writes_do_not_truncate_an_empty_journal` (truncation counter: two standalone writes on an empty journal truncate 0 times, the first bypass write after a transaction truncates once, replace/delete after it truncate 0 times), contract text tightened in ADR-0061, ADR-0025/0026 amendments, SERVER-001 amendment and `ErrorCode::Storage`/`Journal` docs (a bypass checkpoint error precedes the mutation); K3 neutral vocabulary in PROJECT-STATUS, ROADMAP, TRACEABILITY, ADR-0061 status "Accepted (implemented 2026-09-08)", SERVER-001 change history; K4 `target/` logs named as uncommitted local artifacts, no links; K5 0.50.2 entry now follows 0.50.1, docstring space fixed, Relation's redundant read-set re-check removed. Report refreshed with an honest proof table (exact chain stops at the first excluded Windows test; `--no-fail-fast` run 740 passed / 2 excluded).
- Host proof on this snapshot: `proof-B2.log` (result recorded below).
- Host proof (`proof-B2.log`): fmt PASS; clippy ×3 PASS; server,research `--no-fail-fast` 740 passed / 2 failed (exactly the two host-excluded Windows tests: `dog_server` cert-path `:` split, `server_python_client` `python3`); client 238 passed; python 5/5; MSRV `+1.88.0-x86_64-pc-windows-gnu check` PASS. Inspection 1 (fresh Claude CLI) launched on snapshot `ee3392c7…4409`; no edits to the tree until it returns.

## Inspection 1 — fresh Claude CLI on snapshot `ee3392c7…4409`

- Result: `runs/step1b-inspect/claudex-ao4ag9la/` (session `ef1de44d-6de4-43f8-a2ed-1c7944093f0e`, 379 s, claude.exe 2.1.265, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE.** F1 (medium) `reminder_server` builds `ReminderConnectionStore::with_journal` from the same `SERVER_TXN_JOURNAL_PATH` and Reminder keeps the old commit path and ungated bypass mutations, so probes 1–3 still apply to a shipped configuration; F2 (medium) `update_field` can now answer `Err(Journal)`/`Err(Storage)` on a protocol-1 request but `downgrade_for_version` only rewrites `TransactionFailed`, while the amended protocol table claims `Err` is downgraded; F3 (low) `SERVER-JOURNAL-GROUP-COMMIT-DESIGN.md`/`SERVER-TRANSACTION-SESSION-DESIGN.md` not amended, SPEC-REGISTRY verification column lacks the probes test; F4 (low) generic `DogConnectionStore<S>` with a fallible `update_age` could lose a partially applied batch's redo after `commit_serialized` cleanup; journaled/unjournaled error mapping inconsistent. Limitations: no test execution; negative control taken from the report.
- Host verification: F1 confirmed (`src/bin/reminder_server.rs:89-96`; `reminder.rs` mutators at 344-431 and `apply_transaction` at 513-540 unchanged); F2 confirmed (`downgrade_for_version` at serve.rs:1224-1279 rewrites only `TransactionFailed`; `dispatch` maps `update_field` errors to `Response::Err`). All four accepted (`feedback-B2.md`): I1 repair Reminder with the Memory pattern + configuration and concurrency coverage; I2 downgrade `Err(Journal)` below 4 and `Err(Storage)` below 13 to `Unsupported` with tests and corrected docs; I3 design-doc amendments + registry; I4 document the fallible-`S` limit and align Dog's error mapping.
- Fix round 2 launched (resume of `claudex-qklsm873`, `--feedback feedback-B2.md`). Budgets after this: fix rounds 2/2 used, inspections 1/2 used.

### Part B round B3 — Codex fix round 2 result (final fix round)

- Result: `runs/step1b-build/claudex-qfglthl4/` (same session resumed, 651 s, exit 0, snapshot sha256 `a9ba9d50…74c4`, 26 files: adds `src/server/reminder.rs`, `docs/design/SERVER-JOURNAL-GROUP-COMMIT-DESIGN.md`, `docs/design/SERVER-TRANSACTION-SESSION-DESIGN.md`). All LF.
- Host diff check: I1 `ReminderConnectionStore` gains `with_mutation` on update_field/insert/replace/replace_if/delete/compact/write_batch_checked and the gated preflight + `commit_serialized` transaction path (identical shape to Memory); `tests/server_recovery_probes.rs` adds `journal_enabled_reminder_shipped_configuration` and includes Reminder in the concurrent read-set test (four-value status enum handled). I2 `downgrade_for_version` rewrites `Response::Err { Journal }` below 4 and `Response::Err { Storage }` below 13 to `Err(Unsupported)`; unit tests for every older version plus a socket-level test with an injected failing adapter at versions 3/4 and 12/13; protocol table rows 4 and 13 and the `Journal`/`Storage` doc comments now state exactly what is rewritten. I3 both design docs carry an "Amended by ADR-0061" note naming the five adapters; SPEC-REGISTRY verification column lists the probes test. I4 `DogConnectionStore::with_journal` docstring and ADR-0061 state the fallible-`S` limit; both Dog `update_field` arms map non-NotFound failures to `Storage`. Report refreshed (744 passed / 2 excluded per Codex; host proof below).
- Host proof (`proof-B3.log`): fmt PASS; clippy ×3 PASS; server,research `--no-fail-fast` 744 passed / 2 failed (the two host-excluded Windows tests only); client 238 passed; python 5/5; MSRV `+1.88.0-x86_64-pc-windows-gnu check` PASS. Inspection 2 (final, fresh Claude CLI) launched on snapshot `a9ba9d50…74c4`; budgets: fix rounds 2/2 used, inspections 2/2 after this.
- Inspection 2 first attempt `runs/step1b-inspect/claudex-o5730wtd/` failed in 6 s before any review: Claude CLI `api_error_status: 401`, "Failed to authenticate. API Error: Notification: WWW Authorization Required" (the network appliance interception noted in the handoff). Not a verdict; relaunched after a network probe.

## Inspection 2 — fresh Claude CLI on snapshot `a9ba9d50…74c4` (retry after the 401)

- Result: `runs/step1b-inspect/claudex-hvqhcz31/` (session `abb22155-a15c-4e7e-9660-8c7acb51282e`, 362 s, claude.exe 2.1.265, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE.** Core mechanics judged sound (lock order relationship mutex → gate → store lock → group state, no inversion; repeatable reads; five configurations; R6 closed). F1 (medium) `commit_serialized`: when the size-checkpoint flush succeeds and the following `truncate` fails, the fully applied and durable batch is answered `TransactionFailed { Journal, "…nothing was applied" }` and the adapter is fail-stopped; the same unchanged message also answers the indeterminate-cleanup path, contradicting ADR-0061. F2 (low) `with_mutation` checkpoints before pure-request validation, so a Malformed/UnknownField write does a flush+truncate and a checkpoint failure masks the validation error. F3 (low) stale comments: Dog `validate_batch` "outside any lock … never deletes", `insert_record` trait doc, `memory.rs` GRP-FR citation, generic adapters `with_journal` docs. F4 (low) SPEC-REGISTRY: test file placed in the status column instead of the evidence column. Limitations: no test execution; research-gated Order/Employee keep the old path by scope.
- Host verification of F1: confirmed in `commit_serialized` (`if result.is_ok() && flushed { self.truncate().map_err(|e| { unusable=true; Journal })?; }`) and `error_message(Journal)` unchanged.
- Budgets exhausted (fix rounds 2/2, inspections 2/2). No edits made. Owner asked whether to extend by one fix round + one inspection.
- Owner decision (asked 2026-09-08 after inspection 2): extend by one Codex fix round + one fresh Claude inspection (same extension as Part A). Dispositions in `feedback-B3.md`: J1 return `Ok` on a post-apply truncate failure after a successful flush (valid redo over a flushed image, replay idempotent), keep the pre-bypass fail-stop, reword `error_message(Journal)`, unit test + ADR text; J2 pure-request validation before gate/checkpoint on all five adapters with a no-I/O assertion; J3 stale comments; J4 registry columns.
- Fix round 3 launched (resume of `claudex-qfglthl4`, `--feedback feedback-B3.md`). Budgets after this: fix rounds 3/3 (extended), inspections 2/3 (extended).

### Part B round B4 — Codex fix round 3 result (owner-extended, final)

- Result: `runs/step1b-build/claudex-vxg1kry4/` (same session resumed, 499 s, exit 0, snapshot sha256 `b637002f…a8a7`, 26 files, all LF).
- Host diff check: J1 `commit_serialized` now `let _ = self.truncate()` after a successful apply + flush (comment explains retained redo is valid and idempotent; next size checkpoint retries), `unusable` untouched on that path; pre-bypass `checkpoint` fail-stop kept; `error_message(Journal)` reworded ("pre-apply refusals apply nothing, but failed durable cleanup leaves the transaction outcome indeterminate and writes are refused until reopen"); new `post_apply_truncation_failure_preserves_success_and_idempotent_replay` (40k-op checkpoint-due batch, injected read-only handle, Ok asserted, group still admits writes, replay over a flushed fixed-width image leaves the value unchanged, later checkpoint truncates); no fixture pins the message. J2 pure-request validation (field/kind/value, record parsing, label, status discriminant) precedes the gate and checkpoint on Memory/Entity/Relation/Reminder and Dog's journaled `update_field`; `exercise_configuration` asserts the journal length is unchanged after each refusal ("pure request refusals must not checkpoint pending redo"). J3 Dog `validate_batch`, `ConnectionStore::insert_record`, Memory transaction comment and the four `with_journal` docs point at ADR-0061. J4 registry row: Accepted / Implemented / Verified, probes test appended to the evidence column.
- Host proof on this snapshot: `proof-B4.log` (recorded below).
- Host proof (`proof-B4.log`): fmt PASS; clippy ×3 PASS; server,research `--no-fail-fast` 745 passed / 2 failed (the two host-excluded Windows tests only); client 238 passed; python 5/5; MSRV `+1.88.0-x86_64-pc-windows-gnu check` PASS. Inspection 3 (owner-extended, final, fresh Claude CLI) launched on snapshot `b637002f…a8a7`.

## Inspection 3 — fresh Claude CLI on snapshot `b637002f…a8a7` (owner-extended, final)

- Result: `runs/step1b-inspect/claudex-jmxkf2yc/` (session `fb70e261-861c-4b32-ada7-34b29a8726fa`, 502 s, claude.exe 2.1.265, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE**, summary "the repair is sound on the code side … the unresolved defects are documentation fidelity … no high-severity findings". Confirmed sound: gate + pre-append validation (probe 1), checkpoint-then-truncate before every bypass mutation (probes 2/3), repeatable read keys with commit validation (probe 4), lock order acyclic across cross-table paths, cleanup/fail-stop consistent with the documented model, Err downgrades covered, six configurations exercised, part A residuals closed.
- Findings: F1 (medium) `SERVER-002-wire-format.md` still lists `UpdateField` responses as Ok/NotFound and says `TransactionFailed` means "nothing was applied"; needs `Err{Journal}`(4)/`Err{Storage}`(13) with their downgrades and the qualified Journal wording (no shape change, patch note only). F2 (low) SERVER-001 FR-036 row text still describes latest-read tracking inline. F3 (low) Dog `check_read_set` comment and journal.rs header sentence stale. F4 (low) ADR-0061 status "Accepted" without an explicit owner promotion; suggest Proposed-implemented or cite the authorizing plan. F5 (low) a poisoned mutation gate is surfaced as `Journal` with an I/O message; a distinct "writer panicked; restart" message and a docs note suggested.
- Budgets exhausted including the owner's extension (fix rounds 3/3, inspections 3/3). No edits made. Owner asked how to close.
- Owner decision (asked 2026-09-08 after inspection 3): commit snapshot `b637002f…a8a7` now and carry F1–F5 as residuals (all documentation plus one error-message string); no further Part B rounds.

## Part B closed

- Committed on `codex/merge-step1-batch-cross-table` (not pushed): `232b16e` = snapshot `b637002f…a8a7` (26 files, 2500 insertions, 410 deletions). Working tree clean.
- Residuals carried forward (inspection 3, all documentation plus one message string): R-B1 SERVER-002 wire spec UpdateField responses and TransactionFailed wording (Err Journal/Storage reachable, downgrades, Journal no longer certifies nothing applied); R-B2 SERVER-001 FR-036 row text inline; R-B3 Dog check_read_set comment and journal.rs header sentence; R-B4 ADR-0061 status vs owner promotion convention; R-B5 poisoned mutation gate surfaced as Journal with an I/O message. Also open from Part A: the two Windows-only test portability fixes (reverted in K1, separate change).
- Step 2 legacy pin stays `abda0a7` (pushed); bump to `232b16e` once the owner pushes Part B.

## Step 4b-ii — local implementation, 2026-09-09

Work order SHA-256 `995f2d6ea29af8547985673e71efd90cdf08f90e844db876f9a49f75d53c5d29`,
implemented only in rusty_data_os-step4bii from `5af480971b006b3f04ac14fc30127a0361f371ae`.
No commit, push or publication. Independent provider review remains pending.

R0 exceptions are separately disclosed: core writer/hook Send bounds including callers;
Memory's named 4096 operation cap; Entity/Relation's 4096 caps; protocol Sum/Avg wide
accumulation. **4b-i residual closed here:** an overflowing i64 Sum/Avg previously panicked.
Sum now returns Malformed when the i128 sum is outside i64; Avg casts sum/count before
division and preserves empty 0.0. Explicit fractional, negative and both overflow-range
regressions pass, including continued use of a real TCP connection after an overflow error.

uc-facade adds all three independent Stores and a 127.0.0.1:0-only listener. Ten new
correctness tests passed locally, five over real TCP. Entity open-label links are same-table;
no cross-table mentions or cross-domain session was attempted. Atomic nonempty WriteBatch
correctly refuses with TransactionFailed(0, Unsupported). Full proof and file accounting:
[advisory report](../docs/experiments/EXP-0005/STEP4BII-IMPLEMENTATION-REPORT.md).

Final source review found a work-order wording conflict: "any field" ReplaceIf guards
cannot include StrList while reusing unchanged query::validate_predicate, which rejects
StrList even for Eq/Ne. Proposed disposition: preserve the frozen shared admission rule;
Memory tags/Entity aliases guards return Malformed without mutation. An eleventh facade
test pins this boundary. Any expansion needs separate authorization; no second predicate
evaluator or additional uc-protocol change was introduced.

Final agreed eleven-command proof exited 0: 109 unified-commitment, 17 convergence-memory
and 95 portable exp-0001 tests (221 total), all formatting and warnings-denied Clippy,
Markdown links and git diff --check passed. The final source includes eleven facade tests.
Both full proof runs and the R0 outputs are retained with the advisory report. Only result
documentation changed after the final source proof; final links/whitespace were rechecked.

## Step 4b-ii inspection 1 — accepted F1 correction

Host accepted F1-DESCRIBE-RELATIONS-WILDCARD-DROPPED at inspection snapshot
`78e7839e…8d3fdb6`. Both custom describe_relations overrides omitted Neighbors(None),
silently causing an unlabeled same-table Join to return Malformed. Removed the Memory
and Entity overrides; the unchanged trait default includes wildcard plus named descriptors,
all target_table None. Two real-TCP regression tests reproduce the rejection before the
fix (exit 101) and pass afterward (exit 0), checking exact symmetric Join rows and both
Entity labels. Full corrective proof and changed files are recorded in the
[advisory report](../docs/experiments/EXP-0005/STEP4BII-IMPLEMENTATION-REPORT.md#inspection-1-f1-correction).
No other finding reopened, new deviation, denied action, commit, push or publication.

F1 corrective eleven-command proof exited 0: 111 unified-commitment, 17 convergence-memory
and 95 portable exp-0001 tests (223 total), formatting, warnings-denied locked/offline
Clippy, Markdown links and git diff --check passed. All thirteen facade tests are included.
Only result documentation changed afterward; final links/whitespace were rechecked.
