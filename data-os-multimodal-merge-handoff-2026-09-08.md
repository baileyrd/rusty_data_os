# Handoff prompt — Data OS + Rusty Multimodal DB merge, Step 1 in progress (2026-09-08)

Paste this into a fresh Claude Code session started in `C:\dev\rusty_data_os`, or hand it to
Codex. It describes the exact state, the workflow being used, and the next actions.

## Your task

Continue implementing `data-os-multimodal-merge-plan-2026-09-08.md` (repo root, untracked)
using the claudex-loop `codex-build` workflow: Codex builds from a host-written work order,
the host (you) independently runs the proof, and a fresh session of the *other* provider
inspects the final snapshot. Keep fix rounds bounded (2) and inspections bounded (2); ask
the owner before extending. Commit on feature branches, push only when asked.

## State of play

| Item | State |
|---|---|
| Merge plan | `C:\dev\rusty_data_os\data-os-multimodal-merge-plan-2026-09-08.md` (Revision 2). Not committed anywhere yet; Step 2's work order moves it to `docs/plans/`. |
| Step 1 part A (batch Link/Delete repair, Clippy, CI) | **Done, inspected APPROVED, committed and pushed** to `rusty_multimodal_db` branch `codex/merge-step1-batch-cross-table`: `c1d2640` (insert-log Windows write-handle fix, separate) and `abda0a7` (batch repair, docs, tests). No PR opened yet. Base was `478eeda` (PR #229). |
| Step 1 part B (four recovery/isolation probes, repair-or-prevent, snapshot contract, part A doc residuals) | **Codex build was started and then stopped** (owner shutdown, ~12 commands in). Its partial edits (journal.rs, the adapters, a new `tests/server_recovery_probes.rs`, two scratch logs) are in `git stash@{0}` on `codex/merge-step1-batch-cross-table` in the multimodal repo; the working tree is clean at `abda0a7`. Recommended: drop the stash and relaunch part B from scratch with `step1b-recovery-probes-spec.md` (a fresh Codex session, not `--resume`). If you prefer to salvage, `git stash show -p stash@{0}` first; it was never proved or inspected. |
| Step 2 (convergence-memory experiments in Data OS) | Work order written (`step2-convergence-memory-spec.md`), not started. Requires Rust 1.89.0 (not installed on this machine yet; `rustup toolchain install 1.89.0 --profile minimal --component rustfmt,clippy`). |
| Steps 3–6 | Not started; write work orders from the plan when Step 2 lands. |

Supporting files (specs, feedback rounds, build log, probe sources, proof logs, runner
artifacts) were copied to `C:\dev\rusty_data_os\handoff-2026-09-08\` (untracked). Read
`BUILD-LOG.md` there first; it has every round, verdict, session id and disposition.

## Environment facts you need (all verified today)

- Runner: `C:\tools\naner\home\.claude\plugins\cache\claudex-loop\claudex-loop\2.1.0\skills\claudex-loop\scripts\runner.py` (Python 3.14). Subcommands used: `build`, `inspect`. Always pass `--artifacts <dir outside the checkout>` and `--timeout 7200` for builds.
- Codex CLI 0.153.4, config model `gpt-6-astra`, effort high. `~/.codex/config.toml` was extended today so noninteractive `codex exec` works on this Windows box: `[windows] sandbox = "unelevated"`, `shell_environment_policy.inherit = "all"` plus `set.GIT_HOME/RUST_HOME/PYTHON_HOME/NODE_HOME/USER_BIN` pointing at the naner vendor dirs (the user's PowerShell profile rebuilds PATH from those), and `sandbox_workspace_write.writable_roots = ['C:\tools\naner\vendor\rust\.cargo']`. Without these every command is "blocked by policy".
- The inspector for a Codex build is a fresh **Claude** CLI. The runner refuses the npm `claude.CMD` shim; pass `--cli "C:/tools/naner/home/.npm-global/node_modules/@anthropic-ai/claude-code/bin/claude.exe"` (2.1.263).
- Network: an Army web-security appliance intermittently 307-redirects `chatgpt.com`/`api.openai.com`; Codex then fails with "access token could not be refreshed". Probe with `curl -sS -o /dev/null -w "%{http_code} -> %{redirect_url}\n" https://api.openai.com/v1/models` (401 with no redirect is healthy) and retry later.
- Toolchains: stable = cargo 1.98.0 (gnu). MSRV check must use the GNU host: `cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research` (the msvc-host 1.88.0 has no linker here).
- Local proof for the multimodal repo (Windows substitutes `server,research` for `--all-features`, which needs Linux-only crates):

```
cargo fmt --all -- --check
cargo clippy --all-targets --features server,research -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features client -- -D warnings
cargo test --features server,research --no-fail-fast
cargo test --features client --no-fail-fast
python -m unittest discover -s clients/python/tests -v
cargo +1.88.0-x86_64-pc-windows-gnu check --all-targets --features server,research
```

- Two pre-existing Windows-only test failures are excluded by host disposition and must not be "fixed" as part of these work orders: `dog_server` `tests::certificate_classes_from_env_values_follows_the_documented_table` (splits paths on `:`) and `tests/server_python_client.rs` (expects a `python3` binary). CI on Linux passes both.

## Decisions already made (do not relitigate without new evidence)

- Step 1 was split into part A and part B; Step 2 follows; Step 3 after that. Plan order is "batch repair + CI, then the shared Memory experiment, then durable unified commitment".
- Atomic batch contract after part A: relationship mutex only for Link/Delete/WriteBatch and only when a registered table declares a `target_table` relation; five adapters implement atomic (Memory, Entity, Relation, Reminder, Employee), Dog/Order refuse; post-apply detach `Storage` failures are per-op `Failed(Storage)` in `BatchResults`; pipelined `ReplaceIf` validates its guard. Contract text lives in ADR-0060 and SERVER-001 v0.50.1.
- Part B disposition for the snapshot flag: prefer repeatable reads for keys already read plus honest docs (no wire change); "prevent" is acceptable only where repair is unbounded.
- Commits go on `codex/...` feature branches with the Claude co-author and session trailers; push only on request. The owner asked for the part A branch to be pushed (done).

## How to run the next round

Part B, if the tree is dirty with its output:

```
python RUNNER inspect --host claude --builder codex --repo C:/dev/rusty_multimodal_db --plan <handoff>/step1b-recovery-probes-spec.md --base abda0a7e94a9727e410001e9724edc741f6e0d31 --cli <claude.exe> --artifacts <dir> --timeout 3600
```

Fixes: `python RUNNER build ... --resume <previous build result.json> --feedback <host-written dispositions.md>`.

Part B, if the tree is clean:

```
python RUNNER build --host claude --builder codex --repo C:/dev/rusty_multimodal_db --plan <handoff>/step1b-recovery-probes-spec.md --unreviewed-spec --proof "<the && chain of the seven proof commands above, without the MSRV line>" --artifacts <dir> --timeout 7200
```

Step 2 (Data OS): install Rust 1.89.0 first, create branch `codex/merge-step2-convergence-memory` from `main` (`79d51e9`), then the same build command with `--repo C:/dev/rusty_data_os --plan <handoff>/step2-convergence-memory-spec.md`. Its proof is listed at the end of that spec.

## Reporting expectations

Lead with what was verified independently. Say which tests were excluded and why. Record every runner artifact dir, session id and snapshot sha in the build log. Never present an earlier inspection as covering later edits.
