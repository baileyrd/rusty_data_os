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

## Addendum — state as of 2026-09-09 (session 015iJ5WR5HBzxPMq8koUZgzw)

Read `handoff-2026-09-08/BUILD-LOG.md` for every round, verdict, session id, snapshot
hash and disposition. Summary:

| Item | State |
|---|---|
| Step 1 part B (recovery/isolation probes, repair-or-prevent, snapshot contract, part A residuals) | **Done and committed**, not pushed: `232b16e` on `rusty_multimodal_db` branch `codex/merge-step1-batch-cross-table` (on top of `abda0a7`). Three Codex rounds, three fresh Claude inspections; final verdict REVISE on documentation only, carried as residuals R-B1–R-B5 in the build log (SERVER-002 wire-spec wording, FR-036 row text, two stale comments, ADR-0061 status convention, poisoned-gate error message). The two Windows-only test failures remain excluded by disposition. |
| Step 2 (convergence-memory experiment, Data OS) | **Done and committed**, not pushed: `772c720` on branch `codex/merge-step2-convergence-memory`, checked out in the worktree `C:\dev\rusty_data_os-step2` (created from `54bd1a4` on `handoff/merge-2026-09-08`). Four Codex rounds, three fresh Claude inspections; final verdict REVISE with no correctness defect. Six host measurement series (1K/10K, both engines, 36 trials, all valid) are committed as evidence subsets with index entries; raw per-op samples stay in the session scratchpad by owner-accepted deviation from R7 and **must be moved to durable storage** (open). Residuals R-S2 F2–F5 in the build log. CI workflow added but never executed remotely. 100K not executed. |
| Legacy pin | Step 2's legacy runner pins `abda0a7` (part A). Bump to `232b16e` after Part B is pushed, and rerun the legacy proof and series. |
| Steps 3–6 | Not started. Write work orders from `docs/plans/data-os-multimodal-merge-plan-2026-09-08.md` (moved there by Step 2). |

Environment facts learned this session (in addition to the list above):

- The network appliance intermittently 307-redirects `api.anthropic.com` and `claude.ai` as well as the OpenAI hosts; a Claude CLI inspection then fails in seconds with API 401 "WWW Authorization Required". Probe both providers before launching and retry when the redirect clears (it cleared within 5–30 minutes each time).
- Rust 1.89.0 is installed as `1.89.0-x86_64-pc-windows-gnu` only; spell `cargo +1.89.0-x86_64-pc-windows-gnu`. `exp1-descriptive-d1-harness` is Linux-only, so exp-0001 checks on this box use `--workspace --exclude exp1-descriptive-d1-harness`.
- The Codex sandbox has no network. Pre-fetch git dependencies into `CARGO_HOME` from the host and tell the builder to use `--offline`.
- This box has `pwsh` but no `powershell` on PATH; probes that shell out must try `pwsh` first.
- Cargo `target/` directories under `experiments/` are not ignored by the repo; `.git/info/exclude` in the main checkout adds `target/` locally so build output never enters a runner snapshot.
- The legacy 10K single series takes about three hours (73 ops/s with per-op `sync_data`); plan measurement runs accordingly.

Next actions: push both feature branches when the owner says so and open PRs; move the raw
measurement samples out of the scratchpad; then write the Step 3 work order (durable unified
commitment) from the merge plan and run the same codex-build loop.

## Addendum — state as of 2026-09-09 evening (sessions 01Tup76a…, 01VrCRP4…, 01EasZM6…)

| Item | State |
|---|---|
| Step 3 (unify commitment and recovery, Data OS) | **Done and committed**, not pushed: `065470c` (implementation, evidence, host dispositions) and a follow-up docs commit on branch `codex/merge-step3-unified-commitment`, checked out in the worktree `C:\dev\rusty_data_os-step3` (from `e62fa69`). Work order approved by Codex review after six rounds; built by Codex over three rounds; three fresh Claude inspections; final verdict REVISE on provenance (closed by the host committing `docs/experiments/EXP-0003/HOST-DISPOSITIONS.md`) and five low items carried as residuals (UC-R2, R3, R4, R6, R7 in that record). Host evidence: 1K D1, 1K D2 and 10K D2 series on the committed snapshot, 18 trials valid; reconstruction from `e62fa69` + `source.patch` verified. Raw samples at `C:\dev\rusty_data_os-evidence\EXP-0003\`. |
| Step 2 evidence | Raw samples relocated to `C:\dev\rusty_data_os-evidence\EXP-0002\`; index updated in `e62fa69`. |
| Owner note | Everything will migrate to `https://github.com/Rusty-Mill/rusty_data_os.git` (currently empty). No push authorized yet; Steps 4–6 work orders must plan for the remote change. |
| Steps 4–6 | Not started. Next: Step 4 work order (integrate existing features: port Memory/Entity/Relation onto the shared core, compatibility facade, protocol fixtures) from the merge plan; recommend the same Codex plan review before building. |

Additional environment facts:

- A session shutdown kills background runner and measurement processes; long series and inspections must be relaunched (the runner result stays `running` with no process). Check `Get-Process` before assuming a job is alive.
- D2 series cost on this box: about four synchronizations per transaction, 1K in 5 minutes, 10K in 62 minutes.
- The reconstruction verification for a series: `git worktree add --detach <tmp> <revision>`, `git -c core.autocrlf=false apply --binary <series>/source.patch`, hash every `source.sha256` entry.
