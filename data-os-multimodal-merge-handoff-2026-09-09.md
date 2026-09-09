# Handoff prompt — Data OS + Rusty Multimodal DB merge, Step 4c paused mid-review (2026-09-09)

Paste this into a fresh Claude Code session started in `C:\dev\rusty_data_os`, or hand it to
Codex. It describes the exact state, the workflow being used, and the next action. Supersedes
`data-os-multimodal-merge-handoff-2026-09-08.md`, which is now stale (Step 1 era).

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
| **4c (a real, durable Memory→Entity `mentions` edge)** | **Paused mid-review, unapproved.** Work order drafted (`handoff-2026-09-08/step4c-foreign-edges-spec.md`, committed, hash in `BUILD-LOG.md`), worktree `C:/dev/rusty_data_os-step4c` on `codex/merge-step4c-foreign-edges` (pushed, but contains *zero code changes* — the worktree is still clean at the base commit; only the plan review has run). Review round 1: **REVISE**, 2 high + 3 medium findings, all logged in detail in `BUILD-LOG.md`. **Next action, in order:** (1) get the owner's disposition on P4C-001 (a real gap — D3's no-incarnation-tracking design lets a deleted-then-recreated Entity resurrect a stale edge as a different real record; three options are laid out in `BUILD-LOG.md`'s "Open decision" section and were mid-`AskUserQuestion` when this session paused); (2) fold in the P4C-002 `AlreadyLinked`-direction fix (unconditional, zero-cost) and a disclosure decision for its negligible-probability collision half; (3) apply the mechanical P4C-003/004/005 fixes (WriteBatch has no table selector — split the cross-table batch proof into two `Use`-switched requests; add a self-loop `Malformed` check to the new foreign-link path; give Memory back a custom `describe_relations()` that omits the wildcard, since it's real now — Step 4b-ii's own F1 fix was correct for Entity but is wrong for Memory once "mentions" is a real foreign relation); (4) resubmit for review round 2. |
| 5 (migrate one application) | Not started. |
| 6 (consolidate the projects) | Not started. |

Also open, not started: opening the listener beyond loopback; real authentication. Both were
named alongside 4c as options when the owner was last asked which gap to tackle next.

`handoff-2026-09-08/BUILD-LOG.md` has every round, session id, verdict, and disposition for every
step above — read it before doing anything else. It is long; the last ~120 lines cover Step 4c.

## Environment facts you need (carried forward, still true 2026-09-09)

- Runner: `C:\tools\naner\home\.claude\plugins\cache\claudex-loop\claudex-loop\2.1.0\skills\claudex-loop\scripts\runner.py`. Subcommands used: `review`, `build`, `inspect`. Always pass `--artifacts <dir outside the checkout>`. `--plan` and `--artifacts` must be **absolute paths** — the runner resolves a relative `--plan` against `--repo`, not your shell's cwd. A resumed `build` needs both `--resume <prior result.json>` **and** `--approval <review result.json>` together (the runner rejects `--resume` alone with "Supply --approval").
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
  alone. Every host proof log for every step is in `handoff-2026-09-08/proof-*.log` (untracked,
  scratch — not committed, matching this project's whole-session convention).
- `pwsh` (PowerShell 7) is on PATH; bare `powershell` is not.
- The inspector for a Codex build is a fresh **Claude** CLI, not Codex — pass
  `--cli "C:/tools/naner/home/.npm-global/node_modules/@anthropic-ai/claude-code/bin/claude.exe"`
  (the runner refuses the npm `claude.CMD` shim).
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
