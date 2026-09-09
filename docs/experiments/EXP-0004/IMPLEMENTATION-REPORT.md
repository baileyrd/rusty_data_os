# EXP-0004 implementation report

Date: 2026-09-09. This report is advisory; independent final-change assessment is separate.

## Authority and checkout

Implemented the supplied frozen Step 4a work order, identified by SHA-256
`554623ea9c42c4a2ea7f5f162c5cf75381dc14fe417ce9b94e8d7698096074f1`,
under the [merge plan step 4](../../plans/data-os-multimodal-merge-plan-2026-09-08.md), first sentence.
The supplied plan text was the implementation input; its external source file was not opened
or edited and its supplied hash was not independently recomputed.
Checkout: `C:/dev/rusty_data_os-step4`, branch `codex/merge-step4-entity-relation-core`,
base `2f05dfc6867460b6626d1b2e4d3b14a21571377f`. Initial git status was clean.
All source paths resolved against this checkout. No commit, push, publication or remote CI run.

## Implemented behavior

- uc-entity and uc-relation are independent lib crates with only a uc-core path dependency.
  Each owns a Log and requested D1/D2 setting, using the existing create/open split.
- Entity preserves immutable label/kind/aliases except whole-record replacement, mention_count
  updates, same-table symmetric open-label edges, incarnation checks and deletion cascade.
  The two seeded labels and every successfully introduced label survive empty adjacency,
  full replay and checkpoint restore. Symmetric edges use canonical endpoint order.
- Relation preserves all eight fields and only the four specified Put checks in both modes.
  Subject/object strings assert no endpoint existence. Only updated_at_unix_ms has an update
  verb; positive deleted_at is metadata, not physical deletion.
- CME1/CMR1 canonical text payloads and CES1/CRS1 state blobs round-trip by exact re-encoding.
  Transactions are bounded to 1–1024 operations and MAX_PAYLOAD bytes. All domain invariants
  run in raw-byte apply, which uc-core stages before append and uses during recovery.
- The 13 Entity and 16 Relation tests cover deterministic admission/replay, checkpoint equality,
  immutable snapshots, atomic rejection, retry/response loss and injected truncated Final.
  Each domain has a handwritten 22-operation independent BTreeMap model, exercised at D1/D2.
  Entity's oracle uses two-way adjacency instead of the implementation's canonical edge tuples.

No core, Memory, harness, existing test, CI, convergence-memory or exp-0001 source was changed.
The lockfile adds only the two new packages. No invented Entity-to-Relation links, cross-domain
commitment, detach/compact verb, external dependency, generator, measurement series or production code.

## Files changed

Workspace and crates:

- [workspace manifest](../../../experiments/unified-commitment/Cargo.toml)
- [workspace lockfile](../../../experiments/unified-commitment/Cargo.lock)
- [uc-entity manifest](../../../experiments/unified-commitment/crates/uc-entity/Cargo.toml)
- [uc-entity implementation](../../../experiments/unified-commitment/crates/uc-entity/src/lib.rs)
- [uc-entity scenarios](../../../experiments/unified-commitment/crates/uc-entity/tests/scenarios.rs)
- [uc-relation manifest](../../../experiments/unified-commitment/crates/uc-relation/Cargo.toml)
- [uc-relation implementation](../../../experiments/unified-commitment/crates/uc-relation/src/lib.rs)
- [uc-relation scenarios](../../../experiments/unified-commitment/crates/uc-relation/tests/scenarios.rs)

Authorities, registries and documentation:

- [AGENTS.md](../../../AGENTS.md)
- [root README](../../../README.md)
- [executable experiments README](../../../experiments/README.md)
- [glossary](../../GLOSSARY.md)
- [project status](../../PROJECT-STATUS.md)
- [research questions](../../RESEARCH-QUESTIONS.md)
- [traceability](../../TRACEABILITY.md)
- [experiment registry](../README.md)
- [EXP-0004 definition](../EXP-0004-entity-relation-domains.md)
- [HYP-0004](../../hypotheses/HYP-0004-entity-relation-domains.md)
- [ADR-0004](../../adr/ADR-0004-entity-relation-domains.md)
- [deferred follow-on roadmap](../../roadmap/ROADMAP.md)
- this report and [full proof output](proof-output.txt)

## Proof

The agreed sequence ran from the checkout root with short-circuit `&&` semantics and
**exited 0**. Full stdout/stderr is retained in [proof-output.txt](proof-output.txt).
No benchmark evidence is inferred from test timing output.

| Workspace / check | Result |
|---|---|
| unified-commitment fmt / Clippy / tests | Pass; 59 tests (30 existing, 13 Entity, 16 Relation) |
| convergence-memory fmt / Clippy / tests | Pass; 17 tests, matching Step 3's outcome |
| portable exp-0001 fmt / Clippy / tests | Pass; 95 tests, matching Step 3's outcome; specified harness exclusion retained |
| Markdown links | All repository-relative Markdown links resolve |
| git diff --check | Pass, no output |

All 171 tests passed with zero failures. Final report/result-only documentation updates
are followed by another Markdown-link and whitespace check. No Rust source changed after
the successful full sequence.

```powershell
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

## Blocked/denied actions and deviations

No required work-order action was denied. The first PowerShell login profile attempted
Terminal-Icons preference writes outside the checkout and a user PATH registry update;
the sandbox denied those startup side effects. All later shell commands disabled the login
profile. No permission escalation, original-checkout write or network operation was attempted.

No semantic work-order deviation was required. The requested `docs/roadmap/ROADMAP.md` did
not exist, so it was created with the required Deferred row. This is an administrative addition,
not a replacement of the existing research roadmap. No host disposition was needed.

Intermediate development checks caught a generated test-constant typo, a temporary Python
file-writing encoding error, and an extra test's incorrect assumption that a never-committed
Log can checkpoint. These were corrected before the final proof. The unchanged core requires
a commit: the seed-label test now checks fresh create/full replay first, then a checkpoint
after insert/delete with zero live records and no Link. The frozen seed-label requirement
never required checkpoint creation before a commit. No core behavior was weakened.

No platform durability, OS-crash, power-loss or performance conclusion; hypothesis remains
Open, experiment Ready, ADR Proposed. Memory-to-Entity atomicity remains a named Deferred follow-on.
