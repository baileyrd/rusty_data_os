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

## Step 2 — Codex build (Data OS: convergence-memory experiment)

- Handoff branch commits: `288d7de` (Part B rounds, dispositions, Step 2 host decisions), `54bd1a4` (offline dependency rule). Build worktree `C:/dev/rusty_data_os-step2` on `codex/merge-step2-convergence-memory` reset to `54bd1a4`, clean.
- Environment checks: Codex sandbox executes `cargo +1.89.0-x86_64-pc-windows-gnu` (smoke: `--version` and an offline `check` of `exp1-record-format` both exit 0); the earlier 1.88 "Access is denied" was not reproducible and the toolchain ACLs are identical. Sandbox has no network, so the host pre-fetched `rusty_multimodal_db@abda0a7` (`features = ["server"]`) and its graph with 1.89.0 (83 packages; offline lockfile resolves). Cargo `target/` excluded locally via `.git/info/exclude` so build output cannot pollute the snapshot manifest.
- Spec: `step2-convergence-memory-spec.md` (unreviewed; host amendments: worktree target, legacy pin `abda0a7`, Windows toolchain spelling, exp-0001 exclusion, offline fetch rule). Proof passed to the builder: the spec's chain with the Windows spelling, `--exclude exp1-descriptive-d1-harness` on exp-0001, and `fetch --locked --offline` for the legacy workspace. Artifacts under `runs/step2-build/`. Budgets: MAX_FIX_ROUNDS=2, MAX_INSPECTION_ROUNDS=2. Awaiting completion.

### Step 2 round S2-1 result

- Result: `runs/step2-build/claudex-ho1enqt1/` (session `01a083b2-f7be-7df2-918a-903545e12119`, 1681 s, exit 0, plan sha256 `24e257e7…95fc`, base `54bd1a4`, snapshot sha256 `19e25cf4…fcae4`, 49 files: two workspaces (`cm-trace`, `cm-candidate`, `cm-harness`; `cm-legacy-runner`), CMT1 fixtures + Python reference + SHA256SUMS, CI workflow, EXP-0002 document, hypothesis H-CM-001, results index, implementation report, proof log, plan moved verbatim to `docs/plans/`, README/GLOSSARY/PROJECT-STATUS/experiments README/AGENTS.md §10 updates, and one unrequested edit to `tools/validate_markdown_links.py` (scan tracked + untracked existing Markdown so an unstaged move and new docs validate). All LF.
- Codex-reported proof: exact Windows chain exit 0 (8 candidate/shared tests, 2 legacy tests covering single/pipelined/atomic and dependent multi-write batches, 95 exp-0001 tests, links, diff check). Deviations disclosed: legacy native durability not equivalent to D1 (log `sync_data` on insert/replace/delete, none on field update) so both are labelled diagnostic baselines with no equal-durability winner; legacy internal stage timings unavailable; batch measurement width one; link validator change. 100K not run. Candidate lockfile has only the six workspace/path crates; legacy pinned to `abda0a7`.
- Host review (all Rust sources, tests, fixtures, CI, manifests and docs read): oracle is a per-field BTreeMap model independent of both engines; candidate stores CMM1 full after-images/tombstones in structural RF1 provisional frames through `RawAppender`, reads from an in-memory row map, and at `finish` replays the file and rebuilds rows and columns separately, each compared to the oracle; legacy runner calls the pinned `MemoryConnectionStore` in process with single/pipelined/atomic dispatch; per-op timing wraps only `engine.execute`; digests and I/O are outside; environment/source identity captured per run; exclusive output directories with `create_new`. Plan move byte-identical (host diff).
- Host proof: `proof-S2-1.log` (recorded below). Host measurement runs: recorded below.
- Host proof (`proof-S2-1.log`, worktree, GNU 1.89.0): candidate fmt/clippy/test PASS (8 tests); legacy networked `cargo fetch --locked` PASS, fmt/clippy/test PASS (2 tests); exp-0001 fmt/clippy/test PASS with `--exclude exp1-descriptive-d1-harness` (95 tests); `validate_markdown_links.py` PASS; `git diff --check` PASS. Release binaries built. 1K measurement series (candidate + legacy single) running into the scratchpad `exp0002-results/` so untracked output does not enter the inspection snapshot; sizes and 10K recorded below.
- Host 1K series (`measure-S2-1k.log`, release binaries, scratchpad output): trace 27,176,406 bytes; candidate 6 trials valid, 13,025 ops, measured trial 1 at 19.6k ops/s (p50: insert 71 µs, replace 73 µs, update 72 µs, delete 4.8 µs, guard 1.3 µs, get 1.2 µs; replay of the 1K history 776 ms, rows rebuild 13 ms, columns rebuild 17 ms); legacy single 6 trials valid, 526 ops/s (p50: insert 2.6 ms, replace 2.7 ms, delete 2.7 ms with per-op `sync_data`; update 4.6 µs; get 10 µs). Diagnostic only: durability settings are not equivalent (declared in every results file). Retention observed: candidate 198 MB, legacy 104 MB, trace copy 26 MB per series; store 27 MB / 11 MB per trial.
- Findings (`feedback-S2-1.md`): L1 (medium) default retention keeps store files, a trace copy and a source copy per run — a 10K series would be ~2 GB under `docs/`; minimal retention + explicit committed-evidence layout required; L2 (low) CI fetch assumes the legacy GitHub repo is reachable from Actions (visibility unverifiable from this network); results-index recording; L3 (low) hex-encoded trace size and 100K memory note; L4 (low) neutral status vocabulary. 10K series deferred to the fixed snapshot so the retained evidence comes from the final binaries. Fix round 1 launched (resume `claudex-ho1enqt1`, `--feedback feedback-S2-1.md`). Budgets: fix rounds 1/2, inspections 0/2.
- Step 2 fix round 1 first attempt `runs/step2-build/claudex-9gdf0yqx/` failed in 11 s before any work: Codex websocket to `chatgpt.com` answered HTTP 307 (network appliance interception). No snapshot; resume point stays `claudex-ho1enqt1`. Relaunch after a reachability probe.

### Step 2 round S2-2 — Codex fix round 1 result

- Result: `runs/step2-build/claudex-bd1mhd33/` (same session resumed after the 307 retry, 734 s, exit 0, snapshot sha256 `41dbfd5b…52b7`, 50 files; changed: `run.rs`, `retention.rs`, both runner `main.rs`, workflow, both workspace READMEs, results README, EXP-0002 doc, PROJECT-STATUS, implementation report; new `proof-windows-l1-l4.txt`). All LF.
- Host diff check: L1 `series_with_retention` defaults to minimal retention (no trace copy, no source tree; `source.sha256` + `source.patch` only; per-trial `store/` deleted after digests and size capture with the engine dropped first, guarded against symlinks and paths outside the trial dir), trailing `--retain-store` opt-in in both runners, `environment.txt` carries seed, input SHA-256 and the exact regeneration command, each trial's `throughput.txt` carries absolute paths and SHA-256s of `results.cmt`/`observations.cmt`; retention suite now four tests (default vs forensic layouts, invalid-series cleanup with retained diagnostics, trailing-flag parsing, plus the original). L2 workflow comment and legacy README state the GitHub reachability assumption and token setup if private; EXP-0002 §16 reworded, results README carries the committed-evidence layout and a host series entry template. L3 hex-size and 100K memory note with the observed 26 MB 1K figure. L4 neutral status wording. Codex proof: 11 candidate, 2 legacy, 95 exp-0001 tests, links, diff check, exit 0.
- Host: 10K series on the round-1 binaries still running (candidate 10K completed in 6 m 39 s, valid); host proof S2-2 and the final measurement series on the fixed binaries follow, then inspection 1.
- Host proof (`proof-S2-2.log`, snapshot `41dbfd5b…52b7`): candidate fmt/clippy/test PASS (11 tests); legacy networked fetch + fmt/clippy/test PASS (2 tests); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS. Candidate release rebuilt; legacy release rebuild deferred (binary locked by the running 10K check, which is superseded and stopped). Inspection 1 (fresh Claude CLI) launched on this snapshot; final 1K/10K series on the fixed binaries run in parallel into the scratchpad.
- Host 10K check on the round-1 binaries (`measure-S2-10k.log`, full retention layout): trace 270,990,842 bytes; candidate 6 trials valid in 6 m 39 s total, 130,115 ops per trial, measured trial 1 at 18.6k ops/s (p50 insert 76 us, replace 77 us, update 80 us; replay of the 10K history 7.9 s, rows rebuild 152 ms, columns rebuild 291 ms), retained 2.0 GB under the old layout, which confirms L1. Legacy single 10K: the warm-up trial had not finished after 14 m 47 s (store 83 MB and growing); the host stopped it because the series is superseded by the final-binary run, so legacy 10K per-trial cost is much higher than the 1K extrapolation and is measured again in the final series, last in order.

## Step 2 inspection 1 — fresh Claude CLI on snapshot `41dbfd5b…52b7`

- Result: `runs/step2-inspect/claudex-9px748k3/` (session `e03df6b5-1201-4b98-9add-6f1b1327945e`, 774 s, claude.exe 2.1.265, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE.** Substantially faithful to the work order; legacy adapter assumptions verified against the pinned source in the cargo cache. F1 (medium) `Results::encode` hard-codes the candidate D1 durability line, so legacy `observations.cmt` carries a false durability label (R5). F2 (low) lockfile also pins `Rusty-Mill/rusty_mill@cf28471` git source; CI assumption names only one. F3 (low) series label `dispatch=` vs engine label `mode=`. F4 (low) raw results footprint at 10K/100K undocumented (equal-query signatures list every digest twice; observations buffered). F5 (low) link-validator behaviour change undocumented outside the report. F6 (low) glossary entries as `##` sections, hypothesis identifier scheme `H-CM-001` vs existing `HYP-0001`, hex-encoded "hand-readable" trace. Limitations: no commands run; 10K legacy unconfirmed at snapshot time; CI never run remotely.
- Host verification: F1 confirmed in `results.rs` (`durability\t{hex(D1)}` unconditional); F2 confirmed (`Rusty-Mill/rusty_mill` in the lockfile); F3 confirmed (`dispatch=` in main.rs, `mode=` in lib.rs); F6 confirmed (`docs/hypotheses/HYP-0001-…` exists). All six accepted (`feedback-S2-2.md`): M1 engine-declared durability line + fixture regeneration; M2 both git sources named; M3 shared `label_for`; M4 document footprint with observed figures + stream `observations.cmt`; M5 document the validator behaviour; M6 glossary bullets, `HYP-0002` rename, hex decode hint. Fix round 2 (final) launched (resume `claudex-bd1mhd33`). Budgets after this: fix rounds 2/2, inspections 1/2.

### Step 2 round S2-3 — Codex fix round 2 result (final fix round)

- Result: `runs/step2-build/claudex-0m895h2r/` (same session resumed, 781 s, exit 0, snapshot sha256 `850eb388…9e6f`, 51 files; `H-CM-001` hypothesis renamed to `docs/hypotheses/HYP-0002-memory-convergence.md`; new `proof-windows-m1-m6.txt`). All LF.
- Host diff check: M1 `Results.durability` field, `Engine::durability` (candidate `D1`, legacy `DURABILITY`), decode accepts any non-empty hex UTF-8 label, `engine_durability=` line beside `candidate_durability=` in results/environment/throughput/summary, golden `small.results.cmt` regenerated by the Python reference (model label "independent model; no engine writes or durability claim") with new SHA256SUMS, candidate test asserts `D1`, conformance test round-trips an arbitrary UTF-8 label and rejects an empty one. M2 workflow comment and legacy README name both git sources. M3 `label_for(mode)` used by library and binary. M4 footprint table (1K: 1.3/0.8/27 MB; 10K: 20/15/265 MB) in README and EXP-0002 §10/§15, `observations.cmt` streamed via `StreamingResults` with a compacted header, §18 follow-on recorded. M5 validator docstring + AGENTS.md sentence. M6 glossary bullets, HYP-0002 links, `bytes.fromhex` hint. Codex proof: 12 candidate, 2 legacy, 95 exp-0001, links, diff check, exit 0.
- Host proof on this snapshot: `proof-S2-3.log` (recorded below); release binaries rebuilt in the same job. Then inspection 2 (final) and the definitive host series in parallel.
- Host proof (`proof-S2-3.log`, snapshot `850eb388…9e6f`): candidate fmt/clippy/test PASS (12 tests); legacy networked fetch + fmt/clippy/test PASS (2 tests); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS; release binaries rebuilt. Inspection 2 (final, fresh Claude CLI) launched on this snapshot; definitive host series (candidate 1K, legacy 1K single, candidate 10K, legacy 10K single) launched on the final binaries into the scratchpad `exp0002-final2/`.

## Step 2 inspection 2 — fresh Claude CLI on snapshot `850eb388…9e6f` (final)

- Result: `runs/step2-inspect/claudex-falmr786/` (session `4cf83ef9-5650-49ca-af8d-d7208ea15028`, 645 s, claude.exe 2.1.265, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE.** Oracle, canonicalization, streaming compaction, replay ordering checks and output guards judged sound; golden fixture hand-verified. F1 (medium) candidate frames use `IntegrityProfile::Structural` (no CRC-32C, unlike the B1 design's selection) and the timed "replay" stage includes CMM1 payload decoding, leaving "rows" trivial, so the predeclared stage comparison is not representative. F2 (medium) whole-trace/whole-file in-memory design: 100K projected at 10–15 GB per trial; process-lifetime peak RSS is harness-dominated and cannot distinguish engines. F3 (medium) CI never executed; legacy leg assumes anonymous fetch of two GitHub git sources. F4 (low) field update only targets `access_count`; guards only on field 10. F5 (low) exact measurement commands absent from EXP-0002 itself. F6 (low) host-observed sizes quoted without a retained series entry. F7 (low) `docs/TRACEABILITY.md` and `RESEARCH-QUESTIONS.md` lack HYP-0002/EXP-0002 rows. F8 (low) candidate label is the durability constant only. Limitations: no commands run; pinned legacy source not found by this inspector (the earlier inspector verified it in the cargo cache).
- Host assessment: F1 valid (stage attribution matters for the experiment's primary comparison; CRC choice is a disclosed design choice under the spec, needs stating); F2 valid as a follow-on (the spec only requires 100K "executed or not"); F3 cannot be closed without a push and an Actions run (owner action); F4 partly contestable (the pinned legacy adapter exposes only `access_count` through `update_field`, so per-field updates cannot be exercised through that path; guards on non-integer fields are feasible); F5/F7/F8 cheap docs/label fixes; F6 is the host's own pending action after the definitive series.
- Budgets exhausted (fix rounds 2/2, inspections 2/2). Owner asked how to close.
- Host definitive series on snapshot `850eb388…9e6f` binaries (`measure-S2-final2.log`, scratchpad `exp0002-final2/`, minimal retention): candidate 1K 38 s, 18,741 ops/s; legacy 1K single 180 s, 514 ops/s; candidate 10K 422 s, 17,871 ops/s (replay p50 8.05 s, rows 149 ms, columns 271 ms); legacy 10K single 10,883 s, 73 ops/s (insert/replace/delete p50 14–21 ms). All 24 trials valid; every results file carries its own `engine_durability` line. Host-found defect: peak RSS unavailable on this machine (`powershell` not on PATH, only `pwsh`).
- Owner decision (asked 2026-09-09 after inspection 2): extend by one Codex fix round + one fresh Claude inspection. Dispositions in `feedback-S2-3.md`: N1 split replay/decode stages and state the Structural integrity profile; N2 hash the trace once, RSS baseline line, honest 100K projection; N3 RSS probe via `pwsh`/native API with a non-panicking test; N4 CI wording; N5 non-integer guard in traces + fixture regeneration, note the `access_count`-only legacy update path; N6 commands in EXP-0002 §15; N7 TRACEABILITY/RESEARCH-QUESTIONS rows; N8 descriptive candidate label. Fix round 3 launched (resume `claudex-0m895h2r`). Budgets after this: fix rounds 3/3 (extended), inspections 2/3 (extended).

### Step 2 round S2-4 — Codex fix round 3 result (owner-extended, final)

- Result: `runs/step2-build/claudex-z7ff9vpk/` (same session resumed, 682 s, exit 0, snapshot sha256 `bd99987b…e1a3`, 54 files; adds `docs/TRACEABILITY.md` and `docs/RESEARCH-QUESTIONS.md` rows, new `proof-windows-n1-n8.txt`; regenerated `small.cmt`/`small.results.cmt`/SHA256SUMS). All LF.
- Host diff check: N1 `scan_history` (timed `replay`) separated from `decode_history` (timed `decode`); `IntegrityProfile::Structural` stated in README and EXP-0002 with the CRC cell as a §18 follow-on. N2 `LoadedTrace` hashes the input file bytes once; `rss_baseline_after_load` line; 100K projection documented. N3 probes try `pwsh` then `powershell`; `positive_rss` returns a positive integer or an `unavailable:` string; Linux VmHWM converted to bytes; tests added. N4 CI wording in PROJECT-STATUS and EXP-0002 §16. N5 text guard on `status` (field 8) in the small trace (now 59 ops, indices updated) and the seeded generator; fixtures regenerated. N6 exact Windows commands in EXP-0002 §15 with the Linux toolchain note. N7 TRACEABILITY and RESEARCH-QUESTIONS rows (RQ-001, RQ-004). N8 candidate label "candidate RF1 full after-image; IntegrityProfile::Structural; rev=<HEAD>", asserted in tests. Codex proof: 15 candidate, 2 legacy, 95 exp-0001, links, diff check, exit 0.
- Host proof on this snapshot plus release rebuild and a 1K RSS re-run: `proof-S2-4.log` (recorded below). Then inspection 3 (owner-extended, final).
- Host proof (`proof-S2-4.log`, snapshot `bd99987b…e1a3`): candidate fmt/clippy/test PASS (15 tests); legacy networked fetch + fmt/clippy/test PASS (2 tests); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS; release binaries rebuilt. 1K RSS re-run in progress (candidate valid). Inspection 3 (owner-extended, final) launched on this snapshot.
- 1K RSS re-run on snapshot `bd99987b…e1a3` binaries (`proof-S2-4.log` tail, scratchpad `exp0002-final3/`): both valid. RSS now captured via `pwsh`: baseline after load 130.8 MB; candidate peak 192.4 MB; legacy peak 131.6 MB. Stage split confirms inspector F1: physical `replay` p50 58 ms vs `decode` 878 ms at 1K (the earlier "replay 785 ms" was almost entirely CMM1 payload decoding). Evidence subsets for the four definitive series (850eb388 binaries) and this re-run pair (bd99987b binaries) staged in the scratchpad `exp0002-evidence/`.

## Step 2 inspection 3 — fresh Claude CLI on snapshot `bd99987b…e1a3` (owner-extended, final)

- Result: `runs/step2-inspect/claudex-h7vtrtm0/` (session `bca1cb50-bf33-47e0-9ebc-1664c93f769f`, 725 s, claude.exe 2.1.266, observed models claude-fable-5-1 + claude-haiku-4-5, no permission denials). **Verdict: REVISE**, summary "substantially correct and faithful … I found no correctness defect in the oracle, format or engines. The unresolved items are spec-fidelity". Legacy adapter assumptions re-verified against the pinned source in the cargo cache. F1 (medium) raw per-operation samples excluded from committed evidence by the L1 policy while R7 asks for raw result locations under `docs/experiments/EXP-0002/results/`; and EXP-0002 §16 / TRACEABILITY / RESEARCH-QUESTIONS cite host series that are not in the repository and predate N1/N3/N5. F2 (low) fixed 2M-record replay budget. F3 (low) `source.sha256` omits the exp-0001 engine crates. F4 (low) duplicated paragraphs; §16/§17 populated under status Ready; `legacy_internal_stages` line in candidate summaries. F5 (low) AGENTS §10 edited beyond the one-sentence brief; local validation sequence does not list the new proof commands.
- Host assessment: F1 is the host's own pending action (evidence subsets are staged; the series-citation wording will be corrected by the host when it writes the entries, and the raw-sample policy is an owner call per R7 vs L1); F2–F5 are small and can be carried. Budgets exhausted including the owner's extension (fix rounds 3/3, inspections 3/3). No edits made. Owner asked how to close.
- Owner decision (asked 2026-09-09 after inspection 3): host adds the staged evidence subsets and results-index entries, corrects the series citations, records the raw-sample policy as an accepted R7 deviation, commits on `codex/merge-step2-convergence-memory`, and carries F2–F5 as residuals. Host doc edits are host-authored and uninspected (logged as such).

## Step 2 closed (host evidence + commit)

- Host edits (host-authored, uninspected): copied the six evidence subsets into `docs/experiments/EXP-0002/results/2026-09-0{8,9}-<snapshot>-<cell>/` (44+22 files, ~700 KB, all LF); filled six host entries in the results index with dates (UTC), operator paths, validity, summary SHA-256s, input SHA-256s, snapshot provenance and the RSS/stage caveats; recorded the owner-accepted R7 deviation (raw per-op samples external, referenced by path + SHA-256) in the index and EXP-0002 §15; corrected EXP-0002 §16/§17, TRACEABILITY, RESEARCH-QUESTIONS, PROJECT-STATUS and the implementation report to cite only committed evidence. Link validator PASS, `git diff --check` PASS. Open follow-up flagged in every entry: raw `results.cmt`/`observations.cmt` bytes live in the session scratchpad and must be moved to durable storage by the owner.
- Residuals carried (inspection 3): R-S2 F2 fixed 2M-record replay budget; F3 `source.sha256` omits exp-0001 engine crates; F4 duplicated paragraphs, §16/§17 under status Ready, `legacy_internal_stages` line in candidate summaries; F5 AGENTS §10 validator sentence placement and missing local validation commands for the new workspaces. Also open: CI never executed remotely (needs a push and an Actions run); 100K not executed; streaming trace loader; CRC-32C profile cell.
- Committed on `codex/merge-step2-convergence-memory` (worktree `C:/dev/rusty_data_os-step2`, not pushed): `772c720` = snapshot `bd99987b…e1a3` + host evidence/docs (119 files, 10,316 insertions). Working tree clean. Step 2 legacy pin remains `abda0a7`; bump to `232b16e` once Part B is pushed.

## 2026-09-09 continuation (session 01Tup76a1duVrGxKKYhqWS92)

- Raw per-operation samples for the six EXP-0002 host series (and the input traces) copied verbatim from the previous session scratchpad to `C:/dev/rusty_data_os-evidence/EXP-0002/` (772 MB, 36 `results.cmt`; SHA-256 of `exp0002-final2/candidate-1k/measured-1/results.cmt` verified against `throughput.txt`). Results-index entries updated with the durable location (host commit on the Step 2 branch).
- Next: Step 3 work order (unify commitment and recovery) drafted from the merge plan after repository reconnaissance; branch `codex/merge-step3-unified-commitment` from the Step 2 head.
- Step 3 work order drafted: `step3-unified-commitment-spec.md` (host decisions: one transaction = one RF1 event; reuse the frozen R5 lifecycle and codec with Crc32c; D1 and D2 only; payload-agnostic core with a caller validator; new workspace `experiments/unified-commitment/` with `uc-core`/`uc-memory`/`uc-harness`; tested failure model = injected process termination + torn tails, no OS-crash/power-loss claim; incarnations; retry retention scope; `uc1` checkpoint format; EXP-0003/HYP-0003/ADR; AGENTS §10 sentence by reference to merge plan step 3). Recon facts recorded in the spec. Worktree `C:/dev/rusty_data_os-step3` on `codex/merge-step3-unified-commitment` from `e62fa69`.
- Owner decision (asked 2026-09-09): Codex read-only plan review of the work order first, then codex-build with the reviewed spec; budgets 2 fix rounds, 2 inspections. Review 1 launched (`runs/step3-review/`).
- Review 1 (`runs/step3-review/claudex-_hfpmvxe/`, session `01a085ac-de38-79b0-9057-75be426ddad8`, 222 s, plan sha256 `43a66609…f260`): **REVISE**, 6 high + 2 medium — UCR-001 lock file survives abort; UCR-002 durability_time must be sampled before Final per R5; UCR-003 interior-Final truncation indistinguishable from torn tail; UCR-004 retry ignores requested durability; UCR-005 fail-stop needed for every mutating write failure; UCR-006 retry expiry conflicts with duplicate-binding rejection; UCR-007 path deps one level short; UCR-008 shared runner hard-codes EXP-0002 metadata and misses untracked sources. All accepted; spec revised (`feedback-S3-review1.md`); review resumed for approval.
- Review 2 (`runs/step3-review/claudex-4sgcumzd/`, same session resumed, 108 s, plan sha256 `7411e120…b385`): **REVISE**, 2 medium — UCR-008 untracked sources still unrecoverable from the patch; UCR-009 digest-only normalized request conflicts with R5 (full versioned serialization required). Both accepted; spec revised (`feedback-S3-review2.md`); review resumed (round 3).
- Review 3 (`runs/step3-review/claudex-84gvw0ha/`, same session, 72 s, plan sha256 `a3075b24…6f3f`): **REVISE**, 1 medium — UCR-010 `Final.complete_envelope` must contain the payload per R5 (three copies, not two). Accepted; spec revised (`feedback-S3-review3.md`); review resumed (round 4).
- Review 4 (`runs/step3-review/claudex-c8xon1cq/`, same session, 45 s, plan sha256 `f63d332d…6045`): **REVISE**, 1 medium — UCR-011 `UCE1` envelope omitted the identity/sequence/durability-time/provenance fields R5 requires and no agreement check on recovery. Accepted; `UCE1` fully defined with an `EnvelopeMismatch` recovery check (`feedback-S3-review4.md`); review resumed (round 5, the host review cap for this spec).
- Review 5 (`runs/step3-review/claudex-cpyhrojx/`, same session, 50 s, plan sha256 `4ed583e1…9aa4`): **REVISE**, 1 high + 1 medium, both introduced by the round-4 text — UCR-012 unconditional adjacent-Commit check would turn the abort-after-Final residue into an open failure; UCR-013 the two durability bytes were never compared. Both accepted and fixed (`feedback-S3-review5.md`). Host extends its review cap by one round (6) because the defects were host-introduced spec text, not open design questions.
- Review 6 (`runs/step3-review/claudex-gbgozdfl/`, same session `01a085ac-de38-79b0-9057-75be426ddad8`, 30 s, plan sha256 `4fa291fd…c043`): **APPROVED**, no findings. Approval applies to the plan only. Six review rounds total (8 → 2 → 1 → 1 → 2 → 0 findings), all host-accepted.

## Step 3 — Codex build (Data OS: unified commitment)

- Spec: `step3-unified-commitment-spec.md` at the approved hash, `--approval runs/step3-review/claudex-gbgozdfl/result.json`. Worktree `C:/dev/rusty_data_os-step3` (branch `codex/merge-step3-unified-commitment`, base `e62fa69`, clean). Proof: the spec chain (unified-commitment + convergence-memory + exp-0001 with the harness exclusion, links, diff check). Artifacts under `runs/step3-build/`. Budgets: MAX_FIX_ROUNDS=2, MAX_INSPECTION_ROUNDS=2. Awaiting completion.

### Step 3 round S3-1 result

- Result: `runs/step3-build/claudex-uhnj61lz/` (session `01a085ba-0538-7ca0-bf4c-f538697ac07d`, 2639 s, exit 0, plan sha256 `4fa291fd…c043` (approved), base `e62fa69`, snapshot sha256 `5112e99e…965b`, 37 files: new `experiments/unified-commitment/` (`uc-core` lib/envelope/recovery/checkpoint/sha + recovery tests, `uc-memory` adapter + scenario tests, `uc-harness` binary + fault tests), `cm-trace/src/run.rs` (R8b `SeriesMeta`, recoverable patches, reconstruction test), CI third matrix entry, ADR-0003 (Proposed), EXP-0003 document, HYP-0003, results index, implementation report + three proof logs, AGENTS §10 sentence, GLOSSARY/PROJECT-STATUS/TRACEABILITY/RESEARCH-QUESTIONS/README updates). All LF.
- Codex-reported proof: exact chain exit 0; 136 tests (24 unified, 17 EXP-0002 shared, 95 exp-0001); links; diff check. Deviations disclosed: R8b patch bytes now recoverable (untracked files included) so old identifying-only patches are not byte-identical; series output must be outside the source prefixes; process tests live in `uc-harness` (where `CARGO_BIN_EXE_uc-harness` is available); Windows directory sync needs a writable handle with `FILE_FLAG_BACKUP_SEMANTICS` (probed); Windows append handles reject `set_len`, so tail repair uses a writable recovery handle; the reconstruction test uses a disposable repo with object alternates because the sandbox blocked the clone transport.
- Host review (all runtime sources, both test suites, runner diff, CI, manifests, ADR/EXP/HYP/index read): write path stages the payload on a clone before any write, appends Binding/Reservation/Provisional (D2: sync after each), samples the clock after the pre-finalization sync, writes Final (UCE1 envelope with the UCR1 bytes) immediately followed by Commit, post-finalization sync, publishes one `Arc` snapshot, returns the persisted sample with `achieved`; any write/sync failure poisons (pre-Final → `Rejected(Io)`, boundary → `Indeterminate`); recovery scans from zero, requires Crc32c and ordinal 1, runs `validate_lifecycle`, validates every Final envelope against outer fields and the binding, classifies torn tail vs interior damage, reports gaps/unresolved bindings/uncommitted finals, truncates and syncs the torn tail with a writable handle before reopening append-only; checkpoint `uc1` with position, prefix SHA-256, state, binding cache, CRC-32C, `create_new` + file and directory sync; restore prefers the newest valid checkpoint, fails closed on history shorter than checkpoint and on cache disagreement; OS advisory lock via `File::try_lock`. Scenario tests carry disabled-mechanism controls; the abort matrix covers 7 placements × 2 levels with parent-captured markers. Observations for dispositions: full `State` clone per transaction (twice) is O(records) per commit, a measured cost to document; `MAX_RECORDS = 1_000_000` bounds a history to ~200k transactions and should be stated.
- Host proof: `proof-S3-1.log` (recorded below); then fault matrix and 1K d1/d2 + 10K d2 series.
- Host proof (`proof-S3-1.log`, snapshot `5112e99e…965b`): unified-commitment fmt/clippy/test PASS (24 tests incl. the 14-abort matrix and lock release); convergence-memory fmt/clippy/test PASS (17 tests incl. patch reconstruction); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS; legacy workspace, exp-0001 and EXP-0001/EXP-0002 evidence untouched. Fault matrix re-run captured in `faults-S3-1.log`; 1K d1/d2 and 10K d2 series launched on release binaries into the scratchpad; inspection 1 (fresh Claude CLI) launched on this snapshot.
- Session 01VrCRP4gWXCqYghkw3zW5H6 (2026-09-09): the previous session was paused and ended while inspection 1 (`runs/step3-inspect/claudex-0kpffrne/`, no result, runner killed) and the EXP-0003 series (1K d1 completed valid at 6,098 ops/s; 1K d2 interrupted) were running. Both relaunched; the stale inspection record is not a verdict.
- Host series on snapshot `5112e99e` release binaries (`measure-S3-1.log`, `measure-S3-1b.log`): 1K D1 valid 6,098 ops/s; 1K D2 valid 363 ops/s (301 s for six trials; four syncs per transaction); 10K D2 stopped by the host (superseded by the fix round). 1K D1 stages: replay 846 ms, decode 788 ms, rows 18 ms, columns 22 ms, checkpoint 547 ms, open_from_checkpoint 2.16 s; amplification 87,585,029 physical bytes for 27,158,343 payload bytes (three copies).

## Step 3 inspection 1 — fresh Claude CLI on snapshot `5112e99e…965b`

- Result: `runs/step3-inspect/claudex-8wlr_qlo/` (session `86f77a8b-2fae-4448-a15d-8ed6177dbde2`, 869 s, claude.exe 2.1.266, observed models claude-fable-5-1 + claude-haiku-4-5). **Verdict: REVISE.** No high-severity defect; write path, R5 placements, fail-stop, byte-zero recovery, envelope validation, retry retention, uc1 checkpoints, incarnation and fault harness judged faithful. UC-01 (medium) torn tail containing the RF1 magic is classified interior damage by the frozen scanner and becomes permanently unopenable (payload embedded three times; unreachable for CMM2 hex payloads, untested); UC-02 (low) D1 checkpoint synced ahead of unsynced history; UC-03 runner now fails closed without git / self-referential output, undeclared; UC-04 two engine labels per series; UC-05 redundant `validate_lifecycle` pass with offset 0 and I/O mislabelled as corruption; UC-06 auto request ids collide with explicit ids after a torn-tail reopen; UC-07 environment-dependent reconstruction test; UC-08 resident history + double clone undocumented; UC-09 document placement. All accepted (`feedback-S3-1.md`, P1–P9; P8 also documents the 1M-record cap). Fix round 1 launched (resume `claudex-uhnj61lz`). Budgets: fix rounds 1/2, inspections 1/2.

### Step 3 round S3-2 — Codex fix round 1 result

- Result: `runs/step3-build/claudex-yw1vwngh/` (same session resumed, 864 s, exit 0, snapshot sha256 `35b0f0a8…c3bf`, 38 files; new `PROOF-fix-round1.txt`). All LF. Codex proof: 141 tests, fmt, clippy, links, diff check.
- Host diff check: P1 payload containing `RDE1` rejected at commit and in normalized-request decoding, with tests (`rf1_magic_payload_is_rejected_before_write_and_old_binding_is_corrupt`, `injected_torn_binding_with_magic_in_valid_uuid_remains_corrupt_undecidable`); documented residual: the magic inside valid UUID/envelope bytes can still make a torn frame fail closed (codec follow-on). P2 `writer.synchronize()` before every checkpoint write. P3 report/§14 sentence. P4 `MemoryEngine::series_label` passed to the runner, identity asserted. P5 redundant lifecycle pass removed, `LogError::Io { offset, kind }`. P6 auto ids in domain `0x41`. P7 real-checkout pass skips with a printed reason when dirty, `core.autocrlf=false` in the temporary clone, shallow-checkout comment in the workflow. P8 README/§10/§11 document resident prefix, double clone, 200k-transaction cap. P9 glossary bullet in the list, EXP-0003 status section below the header block, RQ-005 after RQ-004.
- Host proof `proof-S3-2.log` and release rebuild running; inspection 2 (final) launched on this snapshot; final series on the fixed binaries follow the rebuild.
- Host proof (`proof-S3-2.log`, snapshot `35b0f0a8…c3bf`): unified-commitment fmt/clippy/test PASS (29 tests); convergence-memory PASS (17); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS; untouched paths clean; release rebuilt. Definitive EXP-0003 series (1K d1, 1K d2, 10K d2) launched on these binaries into the scratchpad `exp0003-final/`.

## Step 3 inspection 2 — fresh Claude CLI on snapshot `35b0f0a8…c3bf` (final in budget)

- Result: `runs/step3-inspect/claudex-miy1o4e2/` (session `e50806a1-0837-48dc-a231-cdd5af577a5e`, 984 s, claude.exe 2.1.266, observed models claude-fable-5-1 + claude-haiku-4-5). **Verdict: REVISE**, "no path that publishes an uncommitted transaction or silently drops a committed one within the declared failure model". UC3-01 (medium) `source.patch` is generated with the host Git config, so `core.autocrlf=true` or diff prefix settings can make a reconstructable series fail the R8b rule; the fixture test forces autocrlf off and the real-checkout pass is skipped when dirty. UC3-02 (low) D2 injection hooks for Binding/Reservation/Provisional fire before the record sync, not after as R5 orders. UC3-03 (low) `Directory::acquire`/`Log::open` create a missing store, so a mistyped path yields an empty valid history. UC3-04 (low) the payload-magic restriction (P1) is a disclosed deviation from "payload-agnostic" whose authorization is recorded only in the implementation report, not in ADR-0003/AGENTS §10. Limitations: no execution; no Linux CI run anywhere.
- Host assessment: all four valid and cheap; UC3-01 needs explicit `-c core.autocrlf=false -c core.safecrlf=false -c diff.noprefix=false -c diff.mnemonicPrefix=false … --no-ext-diff --no-textconv` when generating both diffs plus a CRLF fixture case; UC3-02 move the hook after the sync (or add post-sync points); UC3-03 split create/open; UC3-04 record the host P1 authorization in ADR-0003 and §10. Budgets: fix rounds 1/2 used (one remains), inspections 2/2 used. Owner asked how to close.
- Owner decision (asked 2026-09-09): fix round 2 (in budget) for UC3-01–04 plus one extended fresh Claude inspection. Dispositions in `feedback-S3-2.md` (Q1 explicit Git config for both diffs + CRLF fixture case; Q2 hooks after the D2 sync or distinct appended/synced points; Q3 `Log::create` vs `Log::open` with `NotFound`; Q4 P1 authorization recorded in ADR-0003 and AGENTS §10). Fix round 2 launched (resume `claudex-yw1vwngh`). Budgets after this: fix rounds 2/2, inspections 2/3 (extended).

### Step 3 round S3-3 — Codex fix round 2 result (final fix round)

- Result: `runs/step3-build/claudex-nml7vyi7/` (same session resumed, 652 s, exit 0, snapshot sha256 `1a345528…1a89`, 39 files; new `PROOF-fix-round2.txt`). All LF. Codex proof: 142 tests, fmt, clippy, links, diff check.
- Host proof `proof-S3-3.log` running (no release rebuild: the 10K D2 series on the round-1 binaries is still running and its binary is in use); inspection 3 (owner-extended, final) launched on this snapshot.
- Host proof (`proof-S3-3.log`, snapshot `1a345528…1a89`): unified-commitment fmt/clippy/test PASS (30 tests); convergence-memory PASS (17, incl. the CRLF reconstruction case); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS; untouched paths clean. Host diff check of Q1–Q4: explicit `-c core.autocrlf=false -c core.safecrlf=false -c diff.noprefix=false -c diff.mnemonicPrefix=false … --no-ext-diff --no-textconv` on both diffs; hooks for Binding/Reservation/Provisional fire after the D2 sync; `Log::create` (fails if present) vs `Log::open` (`LogError::NotFound`, creates nothing, tested); ADR-0003 authority section and AGENTS §10 record the P1 acceptance with date and source. Inspection 3 (owner-extended, final) running; 10K D2 series (round-1 binaries) still running.
- 10K D2 series on snapshot `35b0f0a8` binaries (`measure-S3-final.log`): warm-up and measured trials 1–4 completed, trial 5 lost when the session ended; recorded as incomplete (raw files kept externally). 1K D1 valid 6,007 ops/s; 1K D2 valid 372 ops/s (287 s). Definitive series re-run on the final snapshot binaries launched (`measure-S3-final2.log`).

## Step 3 inspection 3 — fresh Claude CLI on snapshot `1a345528…1a89` (owner-extended, final)

- Result: `runs/step3-inspect/claudex-narb73x4/` (session `1af18124-f81a-4054-a33c-3666986151ac`, 798 s, claude.exe 2.1.266, observed models claude-fable-5-1 + claude-haiku-4-5). **Verdict: REVISE.** Core write path, recovery, checkpoint, retry and incarnation "match the work order closely and are consistent with the frozen RF1 codec". UC-R1 (medium) provenance: the payload-magic rejection (P1), the create/open split and the `Engine::creates_store_directory` trait addition (Q3) are authorized only by `feedback-S3-1.md`/`feedback-S3-2.md`, which are not in the Data OS repository, so the AGENTS §10 and ADR-0003 citations do not resolve in-repo. UC-R2 (low) history sync failure inside `checkpoint` does not poison. UC-R3 (low) a surviving pre-commit Binding permanently rejects its request id (undisclosed deviation from R5 §5 resume rule). UC-R4 (low) prefix-hash mismatch at a matching checkpoint position falls back instead of failing closed. UC-R5 (low) `UCE1` magic fifth byte: plan text shows a space, code uses NUL. UC-R6 (low) `store.amplification.txt` written beside the locked store via `with_extension`. UC-R7 (low) regeneration command names `cm-harness` for EXP-0003 series. Limitations: no execution; CI not run remotely; host series not reviewed.
- Host assessment: UC-R1 is host-fixable by committing the disposition records into the repository; UC-R5 is a plan rendering artifact (NUL intended, as `UCR1 `); R2–R4, R6, R7 are small hardening/doc items. Budgets exhausted including the extension (fix rounds 2/2, inspections 3/3). Owner asked how to close.
- Owner decision (asked 2026-09-09 after inspection 3): host commits the disposition records and evidence, carrying UC-R2–R4, R6, R7 as residuals. Host edits (uninspected): `docs/experiments/EXP-0003/HOST-DISPOSITIONS.md` (verbatim P1–P9, Q1–Q4, residuals, UCE1/UCR1 header bytes frozen as `55 43 45 31 00` / `55 43 52 31 00`); ADR-0003 authority section, AGENTS §10 and the implementation report Q4 row now cite the in-repo record; links PASS, diff check PASS. Evidence-rule verification: `git worktree add --detach e62fa69` + `git apply --binary` of the 1K D1 series `source.patch` reproduced all 52 `source.sha256` entries with 0 mismatches (same snapshot/patch for the 1K D2 and 10K D2 cells).
- Owner note (2026-09-09): everything will migrate to `https://github.com/Rusty-Mill/rusty_data_os.git` when the time comes; no push to that remote is authorized yet. Relevant to the CI fetch assumptions and later merge steps.

## Step 3 closed (host evidence + commit)

- Definitive series on snapshot `1a345528` binaries (`measure-S3-final2.log`): 1K D1 valid 4,417 ops/s (68 s); 1K D2 valid 351 ops/s (304 s); 10K D2 valid 319 ops/s (3,696 s; insert p50 3.5 ms; replay 34.8 s, decode 7.5 s, checkpoint 5.0 s, open_from_checkpoint 41.7 s; 873,670,322 physical bytes for 270,856,774 payload bytes). Evidence subsets (52 files, 698 KB) committed under `docs/experiments/EXP-0003/results/2026-09-09-1a345528-<cell>/`; raw samples and traces copied to `C:/dev/rusty_data_os-evidence/EXP-0003/` (1.7 GB incl. the incomplete 35b0f0a8 10K run kept as a negative record).
- Committed on `codex/merge-step3-unified-commitment` (worktree `C:/dev/rusty_data_os-step3`, not pushed): `065470c` (91 files, 19,859 insertions) plus a follow-up docs commit correcting EXP-0003 §16/§17. Host-authored, uninspected: `HOST-DISPOSITIONS.md`, ADR/AGENTS/report citation edits, results index rows, §16/§17 wording. Residuals carried: UC-R2 checkpoint sync failure does not poison; UC-R3 pre-commit Binding permanently rejects its request id (R5 resume rule not implemented); UC-R4 rewritten-prefix checkpoint falls back; UC-R6 amplification file beside the store; UC-R7 regeneration command names cm-harness. Also open: CI never executed remotely for any new workspace; 100K not run; codec follow-on for the RF1-magic torn-frame ambiguity.

## Step 4a — Codex plan review (Data OS: port Entity and Relation onto the unified core)

- Recon: two Explore agents in parallel found (a) `uc-core::Log<S>` is fully generic (one `S` per Log, apply is a `fn` pointer used identically at commit and recovery time, validate closure never runs during recovery) — no core change needed; (b) **decisive**: Entity and Relation have no cross-reference to each other anywhere in `rusty_multimodal_db` (232b16e). Entity links are same-table open-label (character-class rule quoted from `valid_relation_label`); Relation is a standalone table whose subject/object are plain unvalidated strings (ADR-0058: "nothing checks that an endpoint names an entity"). The only real cross-table link in the legacy codebase is Memory→Entity via `target_table`.
- Owner scope decision (asked 2026-09-09): 4a = extend the core only (Entity + Relation as new domains, no facade). Host design correction based on recon: port Entity/Relation to their *actual* legacy semantics (no invented Entity↔Relation link); defer genuine cross-domain atomicity (the one real case, Memory↔Entity) to a named follow-on, since it would require restructuring the already-evidence-backed `uc-memory` from Step 3.
- Work order drafted: `step4a-entity-relation-spec.md`. Two new lib crates (`uc-entity`, `uc-relation`) added to the existing `experiments/unified-commitment` workspace members list (`uc-core`, `uc-memory`, `uc-harness` verified unchanged), each own `Log`/directory, `Slot{incarnation}` pattern, own CME1/CMR1 payload formats, scenario tests mirroring `uc-memory/tests/scenarios.rs`. No `uc-core`/`uc-memory`/`uc-harness` change, no measurement series, no CI change. Worktree `C:/dev/rusty_data_os-step4` on `codex/merge-step4-entity-relation-core` from `2f05dfc`.
- Following the Step 3 pattern: launching a Codex plan review before build.
- Review 1 first attempt `runs/step4-review/claudex-kd6jjpjx/` failed in 11 s before any review: Codex websocket to `chatgpt.com` answered HTTP 307 (network appliance interception, same as seen in earlier steps). No verdict; relaunch after a reachability probe.
- Review 1 (`runs/step4-review/claudex-10w7pzob/`, session `01a086f4-e92d-7773-82cc-e71ee6666d56`, 160 s, plan sha256 `0f97894f…57ee9`): **REVISE**, 2 medium — S4A-001 R3 omitted non-empty relation and non-negative deleted_at_unix_ms (legacy `relation_from_fields` requires both); S4A-002 R2 State{slots,edges} has no persistent known-label set, so a label whose last edge is removed (endpoint deleted) would wrongly become unknown/Malformed instead of known-but-empty, and both built-in labels are seeded at construction in legacy even with zero edges. Both accepted; spec revised (`feedback-S4-review1.md`, now 322 lines); review resumed.
- Review 2 (`runs/step4-review/claudex-lruivunj/`, same session, 57 s, plan sha256 `554623ea…4f1`): **APPROVED**, no findings. Two review rounds total (2 → 0 findings), both host-accepted.

## Step 4a — Codex build (Data OS: uc-entity and uc-relation)

- Spec: `step4a-entity-relation-spec.md` at the approved hash, `--approval runs/step4-review/claudex-lruivunj/result.json`. Worktree `C:/dev/rusty_data_os-step4` (branch `codex/merge-step4-entity-relation-core`, base `2f05dfc`, clean). Proof: unified-commitment + convergence-memory + exp-0001 (harness excluded), links, diff check. Artifacts under `runs/step4-build/`. Budgets: MAX_FIX_ROUNDS=2, MAX_INSPECTION_ROUNDS=2. Awaiting completion.

### Step 4a round S4-1 result

- Result: `runs/step4-build/claudex-oyo2w_0c/` (session `01a086fd-9370-7b71-8d71-169934c49d30`, 1259 s, exit 0, plan sha256 `554623ea…074f1` (approved), base `2f05dfc`, snapshot sha256 `ea085424…d5492`, 22 files: new `experiments/unified-commitment/crates/uc-entity/` (lib + Cargo.toml + scenarios, 13 tests) and `crates/uc-relation/` (lib + Cargo.toml + scenarios, 16 tests); workspace `Cargo.toml`/`Cargo.lock` add the two members; `docs/adr/ADR-0004-entity-relation-domains.md`, `docs/experiments/EXP-0004-entity-relation-domains.md` + `EXP-0004/IMPLEMENTATION-REPORT.md` + `proof-output.txt`, `docs/hypotheses/HYP-0004-entity-relation-domains.md`, `docs/roadmap/ROADMAP.md` (new, records the Memory↔Entity atomicity follow-on); AGENTS/README/GLOSSARY/PROJECT-STATUS/RESEARCH-QUESTIONS/TRACEABILITY/experiments-README updates.
- Codex-reported proof: exact chain exit 0; 171 tests (30 existing unified-commitment, 13 Entity, 16 Relation, 17 convergence-memory, 95 exp-0001 harness-excluded); links; diff check. No core/Memory/harness change; no invented Entity↔Relation link; no cross-domain atomicity. Deviations disclosed: created the missing `docs/roadmap/ROADMAP.md` (administrative, not a replacement of the research roadmap); one seed-label test checks fresh create/full replay before adding a checkpoint case, since a never-committed Log cannot checkpoint. PowerShell login-profile writes outside the checkout were denied by the sandbox; later commands disabled the profile; no escalation or network op attempted.
- Host review (source read directly): `uc-entity` `State{slots, edges, known_labels}` — `Default` seeds `known_labels` with `relates_to`/`mentioned_with`; `Delete` cascades edge removal via `retain` but never touches `known_labels`; `Link`'s `apply` inserts into `known_labels` on success (both live-endpoint and label-validity checks run first); checkpoint decode re-inserts `known_labels` from the `label` lines, matching encode — exactly S4A-002's remediation. `uc-relation` `Relation::validate` rejects empty `subject`/`relation`/`object` and negative `deleted_at_unix_ms`; called at both the insert-mode and replace-mode `Put` sites (lines 244, 330) — exactly S4A-001's remediation. `git diff --stat` against `2f05dfc` touches no `uc-core`/`uc-memory`/`uc-harness` file.
- Host proof (`proof-S4-1.log`, `proof-S4-2.log`, `proof-S4-3.log`, `proof-S4-4.log`, snapshot `ea085424…d5492`): unified-commitment fmt/clippy/test PASS (59 tests incl. new `empty_relation_insert_rejects_before_append`, `negative_deleted_timestamp_insert_rejects_before_append`, `empty_relation_replace_rejects_before_append`, `negative_deleted_timestamp_replace_rejects_before_append`, `seeded_labels_are_known_before_any_links_and_empty_checkpoint`, `delete_cascades_edges_but_retains_labels_through_checkpoint_and_full_replay`); convergence-memory PASS (17 tests); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS. Independently confirms Codex's reported counts and outcome exactly. Inspection 1 (fresh Claude CLI) launching on this snapshot.

## Step 4a inspection 1 — fresh Claude CLI on snapshot `ea085424…d5492` (final, in budget)

- Result: `runs/step4-inspect/claudex-z1fgwvw3/` (session `1141efcd-d27b-4a20-8c5d-e4092267e888`, 365 s, claude.exe 2.1.266, observed models claude-sonnet-5 + claude-haiku-4-5). **Verdict: APPROVED, no defects.** Tooling note: the runner's structured-output tool call was schema-rejected four times (the model kept trying to add a `findings` field as trailing text inside the `summary` string instead of a real JSON key) before a fifth, schema-valid attempt landed with placeholder `findings`/`coverage`/`limitations` content, discarding the real analysis. The genuine analysis survives in attempt 2 of the raw transcript and is recorded here as the actual result, not the placeholder in `result.json`.
- Real finding (from the transcript, attempt 2): full read of both new crates' `src/lib.rs` and `tests/scenarios.rs`, `uc-core`'s public API (`Log::commit`/`recover`, checkpoint), the workspace/lockfile diff, and every touched doc file. Confirms incarnation checks run inside `apply` (not only commit-time validation), Entity's seeded labels survive cascade deletion/checkpoint/replay, canonical-only decode with re-encode verification, self-loop/unknown-label/nonexistent-endpoint rejection ordering, and Relation's four-field validation on both insert and whole-record replace with endpoints/timestamps left deliberately unrestricted. Confirms the independent oracle models never call the domain's own apply/codec (AGENTS §5 independence). Test counts in the documentation match the actual `#[test]` functions and `proof-output.txt` names. No correctness, spec-fidelity or security defect found. Disclosed limitations: could not execute the proof chain itself (covered by the host's own `proof-S4-1..4.log` re-run, identical PASS/counts); could not read the legacy `rusty_multimodal_db` repo directly from this worktree (covered by the host's own recon during spec drafting, which quoted exact file:line legacy citations).
- Host assessment: inspection corroborates the host's own diff review and proof re-run exactly; no gap between the two. Budgets: fix rounds 0/2 used (none needed), inspections 1/2 used (clean, no second round needed).

## Step 4a closed (host commit)

- Committed on `codex/merge-step4-entity-relation-core` (worktree `C:/dev/rusty_data_os-step4`, not pushed): `d90b39c` (26 files, 3,657 insertions), including the four host proof logs as evidence. No residuals carried — inspection and host review both found zero defects.
- Deferred, tracked in `docs/roadmap/ROADMAP.md` on that branch: Memory<->Entity cross-domain atomicity (the one real cross-table link in the legacy codebase), a compatibility facade, and SQL/protocol-fixture reuse — none requested yet; would need their own scoped work orders.
- Owner note carried forward: everything migrates to `https://github.com/Rusty-Mill/rusty_data_os.git` when the owner decides; still no push authorized to any remote.

## Step 4b-i — Codex plan review (Data OS: protocol-22 wire facade infrastructure, uc-protocol)

- Recon: two Explore agents in parallel plus direct host reads of `rusty_multimodal_db@232b16e`
  captured the exact wire framing/codec byte rules (u32 LE frame length; bincode-compatible
  fixint/LE encoding; enum = u32 LE variant index except `Option` = 1-byte discriminant;
  `String`/`Vec` = u64 LE length prefix), the complete `Request`(32)/`Response`(21) type surface
  and every supporting type, the `Hello` min(client,server) handshake, `ConnectionStore`'s method
  surface and generic `dispatch`, and the Memory-domain adapter's exact dispatch chain/schema
  mechanism (for reference; Memory/Entity/Relation wiring is deferred to a later work order).
- Owner scope decisions (asked 2026-09-09): (1) the facade is reimplemented inside `rusty_data_os`
  only, never touching `rusty_multimodal_db`; (2) `AGENTS.md` §3's server/networking prohibition
  and `docs/RESEARCH-ROADMAP.md`'s Phase 7 gate are amended now, as this work order's first
  change, per the merge plan's own "align active repository instructions... in the first relevant
  implementation change" directive — a named, bounded exception (EXP-0005), not a general lift.
- Work order drafted: `step4b-protocol-facade-spec.md` — 4b-i of Step 4b: wire codec, dispatch
  trait, and a per-connection protocol loop only; no real domain adapter wired (deferred to 4b-ii);
  no real `TcpListener`/socket code (D2, deferred); no `bincode`/`serde`/`uuid` dependency (D1, D3,
  matching this workspace's hand-written-codec convention); no enforced authentication (D4); no
  protocol-version downgrading (D5). New crate `uc-protocol` added to the existing
  `experiments/unified-commitment` workspace. Worktree `C:/dev/rusty_data_os-step4b` on
  `codex/merge-step4b-protocol-facade` from `d90b39c` (Step 4a's close).
- Independent verification tool: the host wrote and ran a reference decoder
  (implementing the same byte rules) against all 66 real lines of
  `rusty_multimodal_db`'s `tests/fixtures/wire-vectors.txt`; zero decode errors, every line
  produced a sane literal value. Recorded as `step4b-fixture-expected-values.txt`, required by the
  spec (R4/D6) as the literal-value ground truth every implementation must decode to, not merely
  round-trip.
- Review 1 (`runs/step4b-review/claudex-wl_7wxsb/`, session `01a08740-d1b4-7632-89ac-6cad4a2ea2b6`,
  plan sha256 `021eab10…3a76`): **REVISE**, 5 high + 2 medium. P4B-001 `Transaction` misrouted into
  the session-only fallback (legacy dispatches it for real when no session is open). P4B-002
  missing `write_batch_checked`'s fail-closed default (my draft allowed an unsafe pipelined
  fallback for atomic batches). P4B-003 missing `detach_record`/cross-table `Delete` cascade
  (dangling edges after a cross-table delete). P4B-004 no shared cross-connection relationship
  mutex (a two-connection race could still produce a dangling edge even with the P4B-003 fix).
  P4B-005 `BeginWith`'s three flag behaviors (read-your-writes, validate-on-stage, snapshot
  isolation) reduced to bare "staged writes". P4B-006 missing cross-table `Join` (`join_across`).
  P4B-007 round-trip-only fixture proof cannot catch a self-consistent wrong decoder (e.g. a
  byte-order error or two swapped same-shaped variants). All seven accepted
  (`feedback-S4B-review1.md`); spec revised to hash `4b477545…f9a359`.
- Review 2 (`runs/step4b-review/claudex-4emv6v9c/`, same session, plan sha256 `4b477545…f9a359`):
  **REVISE**, 1 high + 2 medium. P4B-008 the session-open guard set covered only `Transaction`/
  `Begin`, omitting `Insert`/`Link`/`Replace`/`Delete`/`Compact`/`ReplaceIf`/`WriteBatch`/`Use` —
  all eleven guarded requests must be rejected `SessionOpen` before any table-switch or mutation
  effect. P4B-009 missing `MAX_STAGED_OPS`/`MAX_BATCH_OPS` (both 4096) and their exact
  `SessionFull`/`Malformed` rejection semantics. P4B-010 the P4B-006 Join fix conflated
  `Malformed` and `Unsupported`, which `validate_join`'s real 4-way match keeps distinct. All
  three accepted (`feedback-S4B-review2.md`); spec revised to hash `66a53b9f…57d80`.
- Review 3 (`runs/step4b-review/claudex-x0st7q1b/`, same session, 57 s): **APPROVED**, no findings.
  Three review rounds total (7 → 3 → 0 findings), all host-accepted; ten findings closed overall.
- Build launching against the approved spec (hash `66a53b9f…57d80`,
  `runs/step4b-review/claudex-x0st7q1b/result.json` as `--approval`). Budgets: MAX_FIX_ROUNDS=2,
  MAX_INSPECTION_ROUNDS=2.

### Step 4b-i round S4B-1 result

- Result: `runs/step4b-build/claudex-w3xxslf9/` (session `01a08755-d6c9-7b11-bc03-930b56049484`, 2008 s, exit 0, plan sha256 `66a53b9f…57d80` (approved), base `d90b39c`, snapshot sha256 `53e59c59…013864e`, 37 files: new `experiments/unified-commitment/crates/uc-protocol/` (`src/{lib,types,codec,framing,dispatch,query,store,connection}.rs`, `tests/{conformance,dispatch,sessions,relationships}.rs`, `tests/support/{mod,interleaving}.rs`, `tests/fixtures/{wire-vectors.txt,step4b-fixture-expected-values.txt,generate_expected.py,expected_cases.rs}`); workspace `Cargo.toml`/`Cargo.lock`/`README.md` add the member; `docs/adr/ADR-0005-protocol-facade.md`, `docs/experiments/EXP-0005-protocol-facade.md` + `EXP-0005/{IMPLEMENTATION-REPORT.md,proof-output.txt}`, `docs/hypotheses/HYP-0005-protocol-facade.md`; AGENTS §3 and RESEARCH-ROADMAP.md Phase 7 carry the exact R0 exception text; AGENTS/README/GLOSSARY/PROJECT-STATUS/RESEARCH-QUESTIONS/TRACEABILITY/experiments-README updates.
- Codex-reported proof: exact chain exit 0; 209 tests (97 unified-commitment = 59 unchanged + 38 new protocol, 17 convergence-memory, 95 exp-0001 harness-excluded); all 66 fixture lines pass both literal-value and byte-round-trip checks; the D7 two-connection interleaving test and its mutex-removal negative control both pass; links; diff check. No core/Memory/Entity/Relation/harness change; no socket binding; no commit/push. Disclosed deviation: the host's own R7 test-requirement wording ("Delete... confirmed absent after Rollback") was internally contradictory given "zero effect" — Codex flagged it and implemented the sensible reading (the *deletion* is what's absent; the existing record and its edges remain present after Rollback), with an explicit test; asked for independent review to confirm. Also disclosed: `uc_core::Uuid` lacks `Ord` so a separate 16-byte `RecordId` was defined (matching D3); `Registry::new` rejects empty/duplicate/mismatched table names and an out-of-range primary index; legacy `page_keys` was included in `Store` to preserve the full method surface; a pre-existing fixture data quirk (an unrelated row with value 999) was caught and the literal expected result corrected to include it, not the frozen fixture itself.
- Host review (source read directly): `Registry` (`connection.rs`) — `Clone`-derived, `relationship_lock: Option<Arc<Mutex<()>>>` present only when some relation declares a `target_table` (`connection.rs:29-41`); the guard is acquired once, before any adapter access, for exactly `Link`/`Delete`/`WriteBatch`, and its scope spans the whole request handler (`connection.rs:216-229`) — matches D7/P4B-004 exactly. The full eleven-item session-guard match (`connection.rs:239-251`) matches P4B-008 exactly. `write_batch_checked`'s fail-closed default (`store.rs:148-158`) is byte-for-byte the cited legacy default — matches P4B-002. `validate_join` (`query.rs:109-130`) is an exact reproduction of the legacy 4-way match, including the `right_schema.ok_or(Malformed)` case — matches P4B-010 exactly. `Request::Transaction` has a real dispatch arm (`dispatch.rs:92`), not a fallback — matches P4B-001.
- Host proof (`proof-S4B-1..4.log`, snapshot `53e59c59…013864e`): unified-commitment fmt/clippy/test PASS (97 tests, including the named interleaving test and its mutex-scope companion, session-guard, staging-cap, and BeginWith-flag tests observed by name); convergence-memory PASS (17); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS. Independently confirms Codex's reported counts and outcome exactly. Inspection 1 (fresh Claude CLI) launching on this snapshot.

## Step 4b-i inspection 1 — fresh Claude CLI on snapshot `53e59c59…013864e` (final, in budget)

- Result: `runs/step4b-inspect/claudex-oktxlzvr/` (session `36d5022b-dbcd-476b-a813-c33a1053b9d7`, 461 s, claude.exe 2.1.266). First launch attempt (`runs/step4b-inspect/claudex-jqx66nnc/`) failed in 5 s with API 401 "WWW Authorization Required" — the same network-appliance 307-redirect interception seen earlier in this project; confirmed via curl probe, waited for it to clear (redirect gone within ~15 minutes), relaunched successfully.
- **Verdict: APPROVED, 2 low findings.** Tooling note (recurring, same as Step 4a's inspection): the runner's structured-output tool call succeeded twice with full, substantive analysis, then a third, schema-valid-but-degenerate attempt ("test"/"a"/"b"/"c") overwrote it in `result.json`. The genuine analysis survives in attempt 1/2 of the raw transcript and is recorded here as the actual result.
- Real findings (from the transcript): full review of `types.rs`/`codec.rs` (byte rules, worked examples spot-verified), `store.rs` (Store trait vs R5, including detach_record/apply_write_op/write_batch_checked's fail-closed default), `dispatch.rs` (Transaction's real arm, Join's exact validate_join 4-way match, confirming only the six per-connection-state variants use the Unsupported fallback), `connection.rs` in full (Hello negotiation, the 11-variant SessionOpen guard set, BeginWith flag gating/combinations, snapshot-isolation read tracking/overlay ordering, MAX_STAGED_OPS/MAX_BATCH_OPS enforcement, cross-table Link/Delete/detach ordering, the D7 relationship-mutex scope and drop timing), `query.rs`, and the D7 interleaving test traced in detail and confirmed as a genuine deterministic positive/negative-control test, not a flaky timing test. "No material unresolved correctness, spec-fidelity, or security defects."
  - F1 (low, `connection.rs` ~219-259): the relationship lock is acquired for any `Link`/`Delete`/`WriteBatch` *before* the session-open guard check, so a session-blocked request briefly takes the shared mutex before being rejected — literally compliant with R7's wording (not a spec violation), an undocumented minor contention cost. Confirmed accurate by host source read.
  - F2 (low, `connection.rs` ~324-338): `SESSION_VALIDATE_ON_STAGE`'s `validate_op` check runs before the `MAX_STAGED_OPS` cap check, so a validation failure exactly at the cap reports the validation error rather than `SessionFull`; the spec states no precedence for this combination and no test exercises it. Confirmed accurate by host source read.
- Host assessment: both low, both explicitly non-required fixes per the inspector's own framing, neither a spec violation. Following the established residual-carrying pattern (Step 3's low residuals): commit as-is, carry both as documented residuals rather than spending a fix round. Budgets: fix rounds 0/2 used, inspections 1/2 used (clean close, no second round needed).

## Step 4b-i closed (host commit)

- Committed on `codex/merge-step4b-protocol-facade` (worktree `C:/dev/rusty_data_os-step4b`, not pushed): protocol-facade infrastructure (`uc-protocol`: codec, framing, types, dispatch, query, store, connection/Registry; 66-fixture literal-value+round-trip conformance test; the D7 interleaving test; governance amendment to `AGENTS.md` §3 and `RESEARCH-ROADMAP.md` Phase 7; EXP-0005/ADR-0005/HYP-0005; the four host proof logs as evidence), plus this BUILD-LOG entry.
- Residuals carried: F1 relationship-lock acquired before the session-open guard (undocumented minor contention, not a spec violation); F2 validate-on-stage vs. staged-cap precedence unspecified and untested for the combined case.
- Not started: 4b-ii (wiring Memory/Entity/Relation onto this `Store` trait via `uc-memory`/`uc-entity`/`uc-relation`, plus opening a real `TcpListener`, both explicitly deferred by D2/D4/Non-goals of this work order).

## Step 4b-ii — Codex plan review (Data OS: Memory/Entity/Relation on the protocol facade, real listener)

- Recon: direct host reads of the just-built `uc-protocol`/`uc-memory`/`uc-entity`/`uc-relation`
  source (no agents needed — everything is local and already-known from Steps 3/4a/4b-i)
  confirmed: 4b-i's session/connection layer needs zero changes to accept a real `Store` (overlay
  logic operates only on `Fields`, never domain internals); every field-tag/`ValueKind` mapping
  for all three domains matches legacy's real `describe()` bodies 1:1 (Memory's `cm_trace::Memory`
  was deliberately built "Wire-equivalent," confirmed exactly); `Log::commit`/`XEngine::transact`
  require `&mut self`, so every adapter needs a `Mutex`; `Change::Update`'s `equals` is exact-match
  only, so `ReplaceIf`'s arbitrary-`CompareOp` guard must be evaluated by the adapter itself under
  its own lock, not encoded into the log.
- Work order drafted: `step4bii-domain-adapters-spec.md` — a new `uc-facade` crate wrapping
  `MemoryEngine`/`EntityEngine`/`RelationEngine` behind `Mutex`-guarded `Store` impls, plus the
  facade's first real `TcpListener`.
- Review 1 (`runs/step4bii-review/claudex-f1nzu00z/`, session `01a087a0-fd50-7082-9a00-158e6d7c52f6`,
  plan sha256 `d9484b8c…5506fd6a`): **REVISE**, 4 high + 4 medium. Two of the four high findings
  (F2, F4) revealed the work order's own central ambition — a real cross-table Memory↔Entity
  `mentions` link and a session spanning two domains — is not buildable on the frozen design at
  all: `uc-memory`'s `Change::Link` requires both endpoints to exist in Memory's *own* `State` (it
  has no foreign-edge concept and no detach operation), and `uc-protocol`'s connection loop is
  single-table by construction. This is exactly the `MEMORY-ENTITY-CROSS-DOMAIN-ATOMICITY` problem
  already named Deferred in `docs/roadmap/ROADMAP.md`. Put to the owner directly (asked
  2026-09-09): **descope** — wire each domain standalone, open the real listener, verify each
  domain's own same-table operations end-to-end; the cross-table relation stays exactly as
  deferred as it already was. The other findings, also confirmed by direct source read: F1 `Log`'s
  `Box<dyn Writer>`/`Box<dyn Fn(Point)>` have no `Send` bound, so no real engine can satisfy
  `Store: Send + Sync` behind a `Mutex` for a multi-threaded listener; F3 (moot after the descope);
  F5 cross-table batch plumbing unspecified (moot after the descope); F6 each engine caps one
  transaction at 1024 changes but the wire protocol's staging/batch caps are both 4096; F7
  `uc-memory` doesn't re-export `cm_trace::{Memory,Value}`, so `uc-facade` has no accessible named
  types without its own dependency on the already-existing `cm-trace` workspace member; F8 a real,
  pre-existing overflow-panic in already-closed 4b-i's `uc-protocol::query::reduce` (`Sum`/`Avg`
  over `i64`, this workspace's `overflow-checks = true`), only now operationally reachable once a
  real listener exposes `Aggregate` to arbitrary input. All eight accepted (`feedback-S4BII-review1.md`);
  spec revised to hash `4a2096de…c35fe4` with narrow, disclosed exceptions authorizing: a `+ Send`
  bound fix in `uc-core`, raising all three engines' operation cap to 4096, an `i128`-accumulating
  overflow-safe `Aggregate` fix in `uc-protocol`, and a direct `cm-trace` dependency for `uc-facade`
  — each a small, mechanical, citation-backed change to otherwise-frozen crates, not a design change.
- Review 2 (`runs/step4bii-review/claudex-lixbowpc/`, same session, plan sha256 `4a2096de…c35fe4`):
  **REVISE**, 3 medium. F5 (repeat, not fully resolved) the revised R6 still required a successful
  atomic `WriteBatch` even though no adapter overrides `write_batch_checked`, so any nonempty
  atomic batch is `Unsupported` by the accepted 4b-i default — R6 now requires only a successful
  pipelined batch plus an explicit refusal test for atomic. F9 the D7 `Avg` fix as worded performed
  integer division before the `f64` cast, losing the fractional part and breaking an existing
  4b-i test that asserts `5.0/3.0` — fixed to cast both operands to `f64` before dividing. F10
  `Store::neighbors`/`parent`/`children`/`neighbors_by_relation`/`list_relation_kinds` have no
  default body at all (unlike `link_records`/`detach_record`), so omitting them fails to compile —
  not unique to `RelationStore` as flagged; `MemoryStore`/`EntityStore` had the identical gap for
  `parent`/`children`. All three fixed for all three adapters; spec revised to hash
  `995f2d6e…5d53c5d29`.
- Review 3 (`runs/step4bii-review/claudex-wb7wxoo7/`, same session): **APPROVED**, no findings.
  Three review rounds total (8 → 3 → 0 findings), all host-accepted; eleven findings closed
  overall, two of them (F2/F4) resolved by an owner scope decision rather than a spec patch.
- Build launching against the approved spec (hash `995f2d6e…5d53c5d29`,
  `runs/step4bii-review/claudex-wb7wxoo7/result.json` as `--approval`). Budgets: MAX_FIX_ROUNDS=2,
  MAX_INSPECTION_ROUNDS=2.

### Step 4b-ii round S4BII-1 result

- Result: `runs/step4bii-build/claudex-4nggo5db/` (session `01a087b1-cfaf-7741-89a6-e6eb650c2fdb`, 1778 s, exit 0, plan sha256 `995f2d6e…5d53c5d29` (approved), base `5af4809`, snapshot sha256 `78e7839e…8d3fdb6`, 35 files: new `experiments/unified-commitment/crates/uc-facade/` (`src/{lib,memory,entity,relation,listener}.rs`, `tests/{adapters,tcp}.rs`, `tests/support/mod.rs`); R0 fixes to `uc-core` (`src/lib.rs`, `tests/recovery.rs`), `uc-memory`/`uc-entity`/`uc-relation` (`MAX_OPERATIONS` raised to 4096), `uc-protocol` (`src/query.rs`, `src/dispatch.rs`, `tests/dispatch.rs`); workspace `Cargo.toml`/`Cargo.lock`/`README.md`; AGENTS/PROJECT-STATUS/EXP-0004/EXP-0005/HYP-0005/ADR-0005 synchronization.
- Codex-reported proof: exact chain exit 0; 221 tests (109 unified-commitment = 59 prior + 38 protocol + 12 new/changed, 17 convergence-memory, 95 exp-0001 harness-excluded); the named R6 test `real_tcp_three_domains_crud_entity_links_batches_and_all_flags_sessions` plus four further real-socket tests (atomic-batch refusal, two-connection session conflict, 4096-staged-op boundary + reopen, aggregate overflow response) all pass; six direct adapter tests (guards, replay, same-table links, delete/reinsert cleanup) pass. Explicitly confirms: "No cross-table mentions scenario and no cross-domain session were attempted or claimed. Memory's mentions descriptor has target_table None." Disclosed deviation: the spec's "ReplaceIf guards against any field" conflicts with the frozen shared predicate validator, which admits only U32/I64/Bool/Str (not StrList, used by Memory's `tags`/Entity's `aliases`) — proposed and implemented interpretation: "any field the frozen validator admits," tested explicitly (`shared_predicate_admission_refuses_string_list_guards`) rather than extending or duplicating the validator. Also disclosed: R5's illustrative `serve` signature refined into a `LoopbackListener` wrapper plus an `AtomicBool` stop signal (5ms poll, join-on-exit) — an implementation detail, not a deviation from intent.
- Host review (source read directly): `uc-core`'s `Writer`/hook trait objects now `+ Send` at every declaration and call site (`lib.rs:200,209,322`) — matches D7 exactly. All three engines' `MAX_OPERATIONS` raised to 4096 with consistent naming — matches D7 exactly. `uc-protocol::query::reduce` now accumulates in `i128`, returns `Result<ScanValue, ErrorCode>`, maps a `Sum` outside `i64` range to `Malformed` via `i64::try_from(sum)`, and computes `Avg` as `sum as f64 / values.len() as f64` (float cast of both operands before division, not integer division first) — matches D7 exactly and fixes review round 2's F9 precisely.
- Host proof (`proof-S4BII-1..4.log`, snapshot `78e7839e…8d3fdb6`): unified-commitment fmt/clippy/test PASS (109 tests, all required test names from both review rounds observed directly, including all five real-TCP tests and the interleaving/session-guard/atomic-refusal tests); convergence-memory PASS (17); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS. Independently confirms Codex's reported counts and outcome exactly. Inspection 1 (fresh Claude CLI) launching on this snapshot.

## Step 4b-ii inspection 1 — fresh Claude CLI on snapshot `78e7839e…8d3fdb6`

- Result: `runs/step4bii-inspect/claudex-o07rjuuy/` (session `a57689c6-6d0e-4db0-a178-c00e63278504`, 634 s, claude.exe 2.1.266). First launch attempt (`claudex-fehdkhsm`) failed in 5 s with API 401 — the same network-appliance interception seen repeatedly in this project; confirmed via curl, waited for it to clear, relaunched successfully.
- **Verdict: REVISE, 1 medium finding.** F1-DESCRIBE-RELATIONS-WILDCARD-DROPPED: `MemoryStore`/`EntityStore`'s custom `describe_relations()` overrides silently dropped the generic `Neighbors(None)` wildcard descriptor the trait's own default (`default_relation_descriptors`) would have supplied for free — nothing in the work order asked for a custom override, only `target_table: None`, which the default already produces. A `Request::Join` naming the unlabeled relation against either table would wrongly get `Malformed` even though `Store::neighbors()` works. Untested (no test called `Request::Join`) and undisclosed. Confirmed by host source read. Accepted (`feedback-S4BII-inspect1.md`); fix: delete both custom overrides entirely (simpler than what they replaced) and add wildcard-relation `Join` tests for both domains.
- Fix round 1 (`runs/step4bii-build/claudex-q3cnwp44/`, same session, 481 s, exit 0, snapshot sha256 `89482490…e51e9f889`): both overrides removed, two new tests added exercising `Neighbors(None)` against Memory and Entity over real TCP. Host proof (`proof-S4BII-fix1-1..4.log`) re-run and confirms: unified-commitment fmt/clippy/test PASS (facade test count +2, wildcard-relation tests observed by name); convergence-memory PASS (17); exp-0001 PASS (95, harness excluded); links PASS; diff check PASS.
- Inspection 2 (`runs/step4bii-inspect/claudex-d0v55t5k/`, fresh session, 392 s): **APPROVED**, 2 low findings, both explicitly non-blocking. F1-MUTEX-POISON-ASYMMETRY: the frozen `Store` trait's non-`Result` read methods (`get`/`scan_all`/`list_relation_kinds`) panic on a poisoned mutex while every other method gracefully returns `Storage` — forced by the frozen trait signature, already disclosed in the implementation report, "no action required to meet this work order's scope." F2-ENTITY-SELFLOOP-UNTESTED: `EntityStore::link_records`'s self-loop rejection relies entirely on `EntityEngine::transact`'s own check falling through a generic error-mapping catch-all; no test directly confirms the resulting wire code is `Malformed`. Both confirmed real by the inspector's own source citations; host accepts both as residuals (matching the low-severity-residual pattern from every prior clean close this project). Budgets: fix rounds 1/2 used, inspections 2/2 used — clean close within budget, no extension needed.

## Step 4b-ii closed (host commit)

- Committed on `codex/merge-step4bii-memory-domain` (worktree `C:/dev/rusty_data_os-step4bii`, not pushed): the `uc-facade` crate (three real `Store` adapters, the first real `TcpListener`, eleven new tests including five real-socket scenarios), the four D7/R0 exceptions to `uc-core`/`uc-memory`/`uc-entity`/`uc-relation`/`uc-protocol`, the `describe_relations` fix, and the host proof logs as evidence.
- Residuals carried: F1 mutex-poison asymmetry between `Store`'s Result and non-Result methods (frozen-trait-forced, disclosed); F2 Entity's same-table self-loop rejection is untested at the wire-code level (documented behavior, no direct test).
- Not started: real Memory↔Entity cross-table `mentions` edge and cross-domain session (deferred per D6 to a dedicated follow-on that must first design the durable foreign-edge mechanism `ROADMAP.md` already names); binding the listener to anything beyond loopback; TLS; real authentication.

## Step 4c — Codex plan review (Data OS: a real, durable Memory→Entity `mentions` edge)

- Owner decision (asked 2026-09-09, after Step 4b-ii closed): of the three named gaps
  (deferred cross-table edge design; opening the listener beyond loopback; real
  authentication), the owner chose to design the real Memory→Entity edge next.
- Host design, before drafting: re-read legacy's own `delete_across`/`detach_across` doc comment
  (`serve.rs:3040-3050`) and found legacy does **not** promise atomic cross-table cleanup — a
  crash between an Entity delete and Memory's detach leaves a real, disclosed, temporarily
  dangling edge ("adjacency and CountEdges can still report those edges until they are detached").
  This meant a faithful design did not require inventing a distributed transaction across
  `uc-memory`'s and `uc-entity`'s independent logs — only two new same-log operations in
  `uc-memory` (`LinkForeign`, `DetachForeign`) that reproduce that same accepted tolerance,
  wired into the registry's already-built (4b-i) cross-table check/detach machinery, which had
  only ever been exercised by a synthetic test double until now.
- Work order drafted: `step4c-foreign-edges-spec.md`. Worktree `C:/dev/rusty_data_os-step4c` on
  `codex/merge-step4c-foreign-edges` from `3d794a5` (Step 4b-ii's close). New `CMM3`/`CMS3`
  physical format version in `uc-memory` (D2: a new concept, not a new value in `CMM2`/`CMS2`,
  matching this project's magic-byte-bump convention); `MemoryStore::describe_relations()`
  reverts to `target_table: Some("entity")`; no change to `uc-entity`/`uc-relation`/`uc-core`/
  `uc-harness`/`uc-protocol`/`EntityStore`/`RelationStore` (D7).
- Review 1 (`runs/step4c-review/claudex-nclrdn4q/`): **REVISE**, 2 high + 3 medium. **P4C-001
  (high)**: D3's decision not to track the far endpoint's incarnation on a foreign edge directly
  contradicts the spec's own stated promise that a dangling edge never resurrects as something
  else — if Entity E is deleted, Memory's detach hasn't yet run, and a client (which mints its own
  insert ids on the wire) reinserts a *new* Entity record at the same id before any retry/repair,
  `evaluate_join` would resolve the stale edge to the unrelated replacement record, not to a miss.
  Confirmed real and serious: this project's own established incarnation-tombstone philosophy
  (Steps 3/4a) already treats "delete then reinsert the same id" as a normal, tested, defended
  scenario for every domain's own table, not a rare freak collision — the foreign-edge case must
  hold to the same bar, and D3 does not. Closing it completely needs either a small `Store`-trait
  extension (contradicting D7) or narrowing the window (fix the retried-Delete-after-NotFound gap
  that currently skips the detach retry) plus an explicit, disclosed residual for the remaining
  narrow case (a deliberate same-id reinsert racing an in-flight, uncompleted crash recovery).
  **Put to the owner (2026-09-09); a clarifying follow-up was in progress when the session paused
  for a handoff. Unresolved — see "Open decision" below.**
  **P4C-002 (high)**: the `AlreadyLinked` duplicate-check and the D5 bidirectional
  `neighbors`/`neighbors_by_relation` lookup both treat a foreign edge's two sides as
  interchangeable (mirroring the *old*, genuinely-symmetric same-table `Link` pattern), but a
  foreign edge is directional (`from` is always a Memory id, `to` is always an Entity id) — with
  colliding raw id bytes across the two independently-numbered tables (adversarially constructed
  in the review's reproduction, not naturally occurring given random UUIDs), this produces a false
  `AlreadyLinked` on a never-committed pair and lets `Join` fabricate an unrelated pair. The
  `AlreadyLinked` half is a deterministic logic bug (checking both directions when only one is
  ever valid) — clear, zero-cost fix: check only `(from=left, to=right)`, never the swap. The
  `Join`/reverse-lookup half depends on an actual cross-table id collision, which is
  negligible-probability with random UUIDs (unlike P4C-001, which needs no collision at all — a
  client fully controls insert ids on the wire and can deliberately reuse one). Host planned to
  fix the deterministic half unconditionally and disclose the collision half as an accepted,
  negligible-probability limitation, pending confirmation alongside P4C-001's resolution.
  **P4C-003 (medium)**: Proof item 7 (a single pipelined `WriteBatch` containing both a Memory
  `Link` and an Entity `Delete`) is not expressible on protocol 22 at all — `WriteOp` carries no
  table selector; one `WriteBatch` always targets whichever table the connection last `Use`d.
  Confirmed by re-reading `uc-protocol`'s types/connection code; the fix is mechanical: two
  separate non-atomic `WriteBatch` requests (one per table, `Use`-switched between them over the
  same socket), not one spanning both. **P4C-004 (medium)**: R3 never specified rejecting
  `left == right` for the new foreign-edge path; legacy explicitly rejects a self-loop even
  across a foreign relation (`generic/store.rs:1353`, `server/memory.rs:780` → `Malformed`).
  Confirmed; mechanical fix, add the same check before submitting `LinkForeign`. **P4C-005
  (medium)**: Step 4b-ii's own F1 fix (restoring the `Neighbors(None)` wildcard relation
  descriptor for Memory via the trait default) is incompatible with restoring
  `target_table: Some("entity")` here — legacy's real Memory descriptor omits the wildcard
  entirely (`server/memory.rs:747`); once "mentions" is a real foreign relation, a bare/unlabeled
  `Join` against Memory must not silently route through the wrong (same-table) evaluation path.
  Confirmed; fix is to give Memory a custom `describe_relations()` again (listing only the named
  "mentions" descriptor), explicitly superseding 4b-ii's F1 correction for Memory specifically
  (Entity's own wildcard, which legitimately wants it, is untouched) and updating that regression
  test's Memory assertion.

## Open decision — Step 4c paused for a handoff (2026-09-09)

Session paused at the owner's request before P4C-001/002's disposition was confirmed. **Nothing
has been built yet** — this is still the plan-review phase; no code changes exist anywhere for
Step 4c. The spec (`step4c-foreign-edges-spec.md`, unrevised, hash `14521486…60c6`) and review 1's
full findings are committed for continuity (see below). Next action on resume: get the owner's
answer on P4C-001 (narrow-the-window-and-disclose vs. a small `Store`-trait extension vs.
reconsider), fold in the unconditional P4C-002 `AlreadyLinked` fix plus a disclosure decision for
its collision half, apply the mechanical P4C-003/004/005 fixes, and resubmit for review round 2.

## Step 4c — handoff resumed (2026-09-09, new machine/session)

Resumed on a different physical machine than the one that wrote the handoff prompt (confirmed:
`C:\tools\naner\home\.claude` and its runner path did not exist here; the claudex-loop skill was
instead resolved from this machine's own plugin cache at
`C:\Users\baileyrd\.claude\plugins\cache\claudex-loop\...`). Two environment defects specific to
this machine were found and fixed before any spec work, both with the owner's explicit sign-off
(both are git-config/state changes, not covered by the handoff's standing push authorization):

- **CRLF/`core.autocrlf`:** this machine had `core.autocrlf=true` globally and the repo carries no
  `.gitattributes` forcing LF, so every checked-out text file had CRLF line endings while the git
  blobs are LF. This silently broke one `cm-trace` test
  (`run::tests::retained_binary_patch_reconstructs_every_manifest_hash_in_fresh_worktree`, which
  hashes literal on-disk bytes and compares against an LF-computed manifest hash) on a completely
  unmodified Step 4b-ii baseline — confirmed pre-existing, unrelated to any Step 4c change. Fixed
  by setting `core.autocrlf false` for this repo only (not global) and re-normalizing both the
  `step4c` worktree and the main `handoff/merge-2026-09-08` worktree (`git rm -r --cached . && git
  reset --hard`, run only after confirming each worktree was clean and, for the main worktree,
  after capturing the one real in-flight edit to rewrite back afterward). All 11 proof-chain
  commands pass cleanly post-fix; logs in `proof-baseline-1..11.log`.
- **Stale `rusty_multimodal_db` clone:** the local legacy reference repo (`C:/dev/rusty_multimodal_db`)
  was 401 commits behind its own `origin/main`, missing `src/server/memory.rs` and
  `src/server/serve.rs` entirely — the exact files this spec's D3/D11/D12 cite for legacy fidelity.
  This caused review round 3 to return `BLOCKED` (see below) rather than a design finding. Fixed
  by fast-forwarding (`git merge --ff-only origin/main`, a non-destructive, no-owner-decision-needed
  update since the local branch was purely behind, not diverged). Every citation was then
  re-verified directly against the current source: all were substantively correct, but several
  exact line numbers had drifted and one verbatim quote (the `delete_across` crash-window doc
  comment) had been condensed since originally written. Corrected excerpts (with the current line
  numbers and stable ADR/spec IDs — `DEL-FR-007`, `TBL-FR-007`, `TBL-FR-008`, `DEL-FR-005`) are now
  embedded verbatim in the spec itself, so future review rounds don't depend on sandbox access to
  a second repo.

## Step 4c — review rounds 2 through 5 (2026-09-09)

Owner disposition on the two round-1 high findings (asked via `AskUserQuestion` on resume):
**P4C-001 → extend the `Store` trait** (not narrow-the-window-and-disclose) — rationale: a client
fully controls insert ids on the wire, so delete-then-reinsert-same-id is a normal, reachable case
by this project's own established standard (Steps 3/4a), not a residual-worthy freak collision.
**P4C-002 → also fix the collision half now**, not accepted as a disclosed residual, on the same
reachability reasoning. Both closed via one small extension: a new optional `Store::incarnation`
trait method (D8) plus `MemoryStore` holding a direct `Arc<dyn Store>` handle to the Entity table
(D9) — this closes P4C-001 (live incarnation freshness check on every read) and P4C-002's
collision half (deterministic live-membership direction resolution, D5 revised) with one
mechanism, not two.

- **Review round 2** (`step4c-review/claudex-cy_5akev`, plan sha256 `644fd6c8…cd2601b`):
  **REVISE**, 2 high + 2 medium. **P4C-R2-001 (high):** a live TOCTOU race — `evaluate_join`
  resolves neighbor ids (freshness-checked) then separately calls `right.get`, unlocked; a second
  connection's Delete+reinsert in that window still resurrects the wrong record. Traced to a
  one-token gap in an already-generic lock list (`connection.rs`'s `Request::Link | Delete |
  WriteBatch` match never included `Join`); closed directly (D13) rather than escalated, since the
  fix is exactly as narrow and mechanical as the trait extension the owner already approved for
  the same reasoning. **P4C-R2-002 (medium):** `AlreadyLinked` ignored the freshly-resolved
  `to_incarnation`, letting a stale edge block linking to the entity's *current* incarnation —
  fixed by comparing the full four-field tuple (D10 revised). **P4C-R2-003 (high):** same-table
  `state.edges` remains reachable via public `MemoryEngine::transact` independent of the wire
  facade, and the round-1 union-read plan would have let a colliding same-table edge leak into
  `"mentions"` as a fabricated foreign link — fixed by dropping the union entirely; `foreign_edges`
  is now the sole source for `"mentions"` reads (D14), which is simpler than the original plan, not
  an added mechanism. **P4C-R2-004 (medium):** Proof item 1's claim that self-loop rejection always
  precedes every other error ignored that the registry's own far-endpoint check runs before
  `link_records` is ever reached — revised to test all distinct wire outcomes in actual precedence
  order.
- **Review round 3** (`step4c-review/claudex-tb0owyi6`, plan sha256 `88a3c757…4584724`):
  **BLOCKED**, zero findings — not a design defect; the reviewer's sandbox had no access to
  `rusty_multimodal_db` (a separate repo from the one under review) and declined to approve
  unverifiable legacy-fidelity claims. Resolved by the stale-clone fix and citation correction
  above (embedding verbatim excerpts in the spec itself rather than relying on file-path citations
  to an inaccessible second repo).
- **Review round 4** (`step4c-review/claudex-og96zwiw`, plan sha256 `937c8fcf…7d407ad4`):
  **REVISE**, 1 medium. **P4C-R3-001:** R4's own implementation instructions still told the builder
  to reject a self-loop *before* checking `left`'s local existence, directly contradicting D11's
  already-corrected precedence text (round 3) and legacy's actual order (registry far-endpoint
  check → local `left` existence → self-loop). A reachable case this got wrong: `q` existing in
  Entity but not in Memory should be `RecordNotFound`, not `Malformed`. Fixed by reordering R4's
  instructions and adding the missing outcome to Proof item 1.
- **Review round 5** (`step4c-review/claudex-6837z2qf`, plan sha256 `af142bbf…4a36891794`):
  **APPROVED**, zero findings. "No material unresolved defects found in the proposed Step 4c
  design." Approval is for the plan only, not an implementation or completed validation, per the
  runner's own convention.

Final design, approved: D1-D14 (`step4c-foreign-edges-spec.md`, current). Two narrow, disclosed
exceptions to previously-"frozen" `uc-protocol`, both mechanical and matching the shape of
Step 4b-ii's own four D7-style exceptions: (1) `Store::incarnation`, a new optional trait method,
plus `EntityStore`'s four-line override (D8); (2) one match-arm addition extending
`Connection::request`'s existing relationship-lock acquisition to also cover `Request::Join` (D13).
`uc-entity`'s engine, `uc-relation`, `uc-core`, `uc-harness` and `RelationStore` remain completely
untouched. Total: five review rounds, eleven distinct findings across rounds 1-4 (P4C-001/002/003/
004/005, P4C-R2-001/002/003/004, P4C-R3-001), all resolved without a design reconsideration —
every fork after the initial P4C-001/002 disposition turned out to be mechanical once traced to
its exact source location.

Build launching against the approved spec (plan sha256 `af142bbf…4a36891794`,
`runs/step4c-review5/claudex-6837z2qf/result.json` as `--approval`). Budgets: MAX_FIX_ROUNDS=2,
MAX_INSPECTION_ROUNDS=2, matching every prior Step 4 increment.
