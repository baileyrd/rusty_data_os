# Handoff prompt — Data OS + Rusty Multimodal DB merge, Step 4c closed, Step 5 next (2026-09-09)

Paste this into a fresh Claude Code session started in `C:\dev\rusty_data_os`, or hand it to
Codex. It describes the exact state, the workflow being used, and the next action. Updated in
place 2026-09-09 (Step 4c's own resumption and close) — supersedes the version of this file that
described Step 4c as "paused mid-review," and `data-os-multimodal-merge-handoff-2026-09-08.md`,
which is now stale (Step 1 era).

## Your task

Continue implementing `data-os-multimodal-merge-plan-2026-09-08.md` (repo root) via the
claudex-loop `codex-build` workflow: draft a work order, run it through Codex plan review to
approval, delegate the build to Codex, independently run the proof, get a fresh Claude CLI
inspection, iterate within bounded fix/inspection rounds (2 each; ask the owner before
extending). Every step so far has run its full diff past Codex plan review *before* any code was
written — do not skip that step. Commit on the step's own feature branch; **all branches through
this point have now been pushed to `origin` (`baileyrd/rusty_data_os`)**, so pushing further
commits on an already-pushed branch is a normal continuation, not a first-time authorization
decision — but confirm with the owner before pushing to `main`, opening a PR, or touching the
future `Rusty-Mill/rusty_data_os` migration target (still empty, not yet authorized).

## State of play

| Step | State |
|---|---|
| 1 (repair legacy defects) | Done, in `rusty_multimodal_db` (a separate repo/session, not this one). |
| 2 (shared Memory experiment, EXP-0002) | Done and committed on `codex/merge-step2-convergence-memory` (pushed). |
| 3 (unify commitment/recovery, EXP-0003: `uc-core`, `uc-memory`) | Done, reviewed, inspected, committed on `codex/merge-step3-unified-commitment` (pushed). Evidence series recorded; 5 low residuals carried (see `BUILD-LOG.md`). |
| 4a (Entity/Relation on the core, EXP-0004: `uc-entity`, `uc-relation`) | Done, reviewed (2 rounds), inspected (0 findings), committed on `codex/merge-step4-entity-relation-core` (pushed). |
| 4b-i (protocol-22 wire facade infra, EXP-0005: `uc-protocol`) | Done, reviewed (3 rounds, 10 findings closed), inspected (2 low residuals), committed on `codex/merge-step4b-protocol-facade` (pushed). Governance note: `AGENTS.md` §3 and `docs/RESEARCH-ROADMAP.md`'s Phase 7 gate carry a named, bounded exception for this facade. |
| 4b-ii (Memory/Entity/Relation wired to `uc-protocol::Store`, first real `TcpListener`) | Done, reviewed (3 rounds, 11 findings closed — 2 by an owner scope decision, not a patch), inspected + one fix round + re-inspected (2 low residuals), committed on `codex/merge-step4bii-memory-domain` (pushed). Real cross-table Memory↔Entity link explicitly **descoped**, not built — see next row. |
| 4c (a real, durable Memory→Entity `mentions` edge) | Done. Resumed from a paused-mid-review handoff on a different machine (see "Environment facts" for the two machine-specific defects found and fixed there). Owner disposed both round-1 high findings toward closing the gap rather than a disclosed residual (extend the `Store` trait for P4C-001; also fix P4C-002's collision half). Reviewed 5 rounds total (11 findings across rounds 1-4, all mechanical once traced to source — no further owner escalation needed), approved round 5 with zero findings. Built by Codex, inspected (fresh Claude CLI, 0 findings, clean close no fix round needed), committed on `codex/merge-step4c-foreign-edges` (pushed). Full detail in `BUILD-LOG.md`. |
| **5 (migrate one application)** | **Not started — next action.** Pick which application to migrate onto the unified `experiments/unified-commitment` stack (`uc-core`/`uc-memory`/`uc-entity`/`uc-relation`/`uc-protocol`/`uc-facade`) and draft a work order the same way: Codex plan review to approval, delegate build, independent proof, fresh inspection. |
| 6 (consolidate the projects) | Not started. |

Also open, not started: opening the listener beyond loopback; real authentication. Both were
named alongside 4c as options when the owner was last asked which gap to tackle next.

`handoff-2026-09-08/BUILD-LOG.md` has every round, session id, verdict, and disposition for every
step above — read it before doing anything else. It is long; the last ~120 lines cover Step 4c.

## Environment facts you need (carried forward, still true 2026-09-09)

- **The runner path, and several other paths below, are per-machine — verify, don't assume.**
  This handoff was written on a machine with `C:\tools\naner\home\.claude\...`; a later session
  resumed on a *different* machine where that path didn't exist at all. Resolve the runner from
  wherever the `claudex-loop` skill is actually installed on the machine you're on (e.g. `C:\Users\<user>\.claude\plugins\cache\claudex-loop\claudex-loop\<version>\skills\claudex-loop\scripts\runner.py`)
  before assuming the path below. Subcommands used: `review`, `build`, `inspect`. Always pass
  `--artifacts <dir outside the checkout>`. `--plan` and `--artifacts` must be **absolute paths**
  — the runner resolves a relative `--plan` against `--repo`, not your shell's cwd. A resumed
  `build` needs both `--resume <prior result.json>` **and** `--approval <review result.json>`
  together (the runner rejects `--resume` alone with "Supply --approval").
- **Two more machine-specific environment defects found and fixed during Step 4c's resumption
  (2026-09-09), both worth checking for on any new machine before trusting a clean baseline:**
  (1) `core.autocrlf=true` with no repo `.gitattributes` forcing LF silently corrupted every
  checked-out text file to CRLF, which broke one real `cm-trace` test that hashes literal on-disk
  bytes against an LF-computed manifest — looked exactly like a pre-existing test failure on an
  untouched baseline. Fix (owner-approved): `git config core.autocrlf false` (repo-local, not
  global) then `git rm -r --cached . && git reset --hard` per worktree, only after confirming each
  worktree is clean (or capturing any in-flight edit first). (2) The local `rusty_multimodal_db`
  legacy reference clone was 401 commits behind its own `origin/main`, missing the exact
  `src/server/memory.rs`/`src/server/serve.rs` files this project's specs cite for legacy
  fidelity — this caused a Codex plan review round to return `BLOCKED` (unverifiable citations)
  rather than a real finding. Fix: `git fetch && git merge --ff-only origin/main` in that repo, then
  re-verify every citation and embed corrected excerpts directly in the spec (see
  `step4c-foreign-edges-spec.md`'s "Legacy's own crash-window tolerance" section for the pattern)
  so future review rounds don't depend on sandbox access to a second repo.
- **Network interception (recurring, not new):** an Army web-security appliance intermittently 307-redirects `api.anthropic.com`, `claude.ai`, `api.openai.com`, `chatgpt.com`, and even plain `github.com` (git push/fetch) through `http://DMZ-WSA-DATA.ad.msl.army.mil/...`. Symptom for Claude CLI: fails in ~5s with API 401 "WWW Authorization Required". Symptom for git: `fatal: could not read Username for 'http://DMZ-WSA-DATA...'`. Fix: probe with `curl -s -o /dev/null -w "%{http_code} redirect_url=%{redirect_url}" <url>` — empty `redirect_url` means healthy; poll every 60s (it has always cleared within 5–30 minutes) and relaunch/retry the exact same command. Do not reduce scope or give up because of this — it is always transient.
- **A recurring tool bug, not a real defect:** the inspection CLI's structured-output tool call has twice, in this project, produced 1–2 fully substantive analyses and then overwritten them with a degenerate placeholder (`"summary": "test"`, fields like `"a"`/`"b"`/`"c"`) on a later attempt that still validates against the schema. When `result.json`'s `response` looks like a stub, **do not trust it** — find the real transcript at `C:\tools\naner\home\.claude\projects\C--dev-rusty-data-os-step<N>\<session_id>.jsonl` and read the `StructuredOutput` tool_use entries directly; the earlier, substantive one is the real result. This has been reported as product feedback twice already (queued locally via `SendFeedback`, not yet sent).
- Toolchain: `cargo +1.89.0-x86_64-pc-windows-gnu` (bare `+1.89.0` resolves to msvc host, which has no linker here). MSRV checks (not used in the Step 4 proof chain) use `+1.88.0-x86_64-pc-windows-gnu`.
- The proof chain used for every Step 3+ round (11 commands, run from repo root, `&&`-chained):
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
  (`exp1-descriptive-d1-harness` is Linux-only; always excluded here.) Always re-run this
  independently on the host after every Codex build/fix round — never trust the reported count
  alone. Host proof logs generated while iterating on a spec (still on `handoff/merge-2026-09-08`,
  before a worktree/branch exists yet, or in a different step's worktree) are untracked scratch —
  don't commit them there. Once a step's own worktree has a build to close, copy/regenerate that
  step's final proof logs *into that worktree* as `handoff-2026-09-08/proof-S<N>-1..11.log` and
  commit them as part of the closing commit — every prior step (e.g. `codex/merge-step4bii-memory-domain`)
  does this; it's a real part of the evidence trail, not just scratch.
- On a fresh machine, the pinned GNU toolchain (`+1.89.0-x86_64-pc-windows-gnu`) may need
  installing (`rustup toolchain install 1.89.0-x86_64-pc-windows-gnu`) before the proof chain can
  link at all — check `rustup toolchain list` first; don't assume it's present. Confirm which
  toolchain family actually has a working linker on the machine you're on (MSVC needs real Visual
  Studio Build Tools; the GNU target's rustup-installed toolchain bundles its own linker) before
  trusting either "bare `+1.89.0` resolves to msvc, no linker" note below, which was true on the
  original machine specifically, not a universal fact.
- `pwsh` (PowerShell 7) is on PATH; bare `powershell` is not.
- The inspector for a Codex build is a fresh **Claude** CLI, not Codex — pass `--cli <path>` from
  `where claude` on the machine you're on (the documented path
  `C:/tools/naner/home/.npm-global/node_modules/@anthropic-ai/claude-code/bin/claude.exe` was
  specific to the original machine and did not exist on a later one); the runner refuses the npm
  `claude.CMD` shim, so make sure whatever `where claude` returns is a real `.exe`, not a shim.
- Each step gets its **own new worktree and branch**, based on the previous step's closing commit
  (never `handoff/merge-2026-09-08` itself, which carries only planning docs/BUILD-LOG, not code):
  `C:/dev/rusty_data_os-step2` … `-step4`, `-step4b`, `-step4bii`, `-step4c`. `git worktree add -b
  codex/merge-step<N>-<slug> C:/dev/rusty_data_os-step<N> <previous-step-branch>`.
- Owner-standing decisions (see `[[merge-plan-loop-owner-decisions]]` in this session's memory,
  and repeated throughout `BUILD-LOG.md`): bounded fix/inspection rounds, ask before extending;
  commit-and-carry low-severity residuals rather than spending extra rounds on them; every "frozen
  crate" exception must be narrow, mechanical, and explicitly disclosed in both the spec and
  `BUILD-LOG.md`; a genuine architecture fork (not just a missing spec detail) gets asked, not
  silently patched — Step 4b-ii's cross-table descope and Step 4c's P4C-001 are both examples.
