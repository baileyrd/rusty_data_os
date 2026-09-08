# Work order: merge plan Step 2 — the shared Memory experiment (Data OS)

Source plan: `C:/dev/rusty_data_os/data-os-multimodal-merge-plan-2026-09-08.md`, section
"2. Run the shared Memory experiment". Depends on Step 1 part A being landed in
rusty_multimodal_db (the legacy runner pins that reviewed revision; if part A is merged first,
pin the merged commit instead and say which).

Target repository: `C:/dev/rusty_data_os` (baseline `79d51e9404c8b35dee2662aa657b602e9061b44b`).
Follow `AGENTS.md` (§2 measurement method before implementation, §4 benchmark-integrity
checklist, §5 independent correctness invariants, §6 keep negative results, §7 update every
related document in the same change, §10 authorization ledger) and `docs/experiments/README.md`
(18-section experiment document, status vocabulary). Do not `git commit`.

## Goal

Two standalone workspaces execute identical, versioned Memory traces against (a) the Data OS
ordinary-write D1 candidate and (b) the reviewed legacy multimodal engine, both checked against
an independent expected-state model; CI exercises both; retained measurements show where the
candidate improves or regresses. `Memory` here means the existing multimodal application record
type (`rusty_multimodal_db::generic::memory::Memory`, `Memory@2`, 13 fields: content, category,
tags, source, metadata_json, created_at_unix_ms, updated_at_unix_ms, memory_type, status,
sensitive, access_count, deleted_at_unix_ms, node_id); it does not restrict the database to that
domain.

## Repository facts the design must respect (verified 2026-09-08)

- `experiments/exp-0001/` is a Rust 1.89.0 workspace (`rust-toolchain.toml` there only; sibling
  directories inherit nothing), edition 2024, `Cargo.lock` with only the four workspace crates,
  CI `.github/workflows/exp0001-slice-a.yml` runs fmt/clippy/test with hard-coded
  `--manifest-path experiments/exp-0001/Cargo.toml`, `--locked --offline`, `CARGO_NET_OFFLINE=true`,
  `RUSTFLAGS=-Dwarnings`, and installs 1.89.0 via rustup directly.
- Portable reusable code: `exp1-record-format` (RF1 codec, `B0Store`), `exp1-raw-append-replay`
  (`RawAppender::open/append`, `reopen_and_replay`, the ordinary-write D1 path),
  `exp1-workload-conformance`. NOT portable: `exp1-descriptive-d1-harness` (`compile_error!` off
  Linux x86_64); its `history_views.rs` (`rows_from_events`, `columns_from_events`, independent
  oracle `expected`, `run_trial`, run-metadata capture) and `examples/history_views.rs` are the
  templates to copy, not depend on.
- No Memory-shaped record, field catalog or typed multi-field entity exists in Data OS; the
  exploratory payload is `Event { entity_id, logical_time, value }`.
- `docs/GLOSSARY.md` defines neither "Memory" nor "B0"; `experiments/README.md:9`, root
  `README.md:70-72`, `crates/README.md:5` and `docs/PROJECT-STATUS.md` contain stale "no
  implementation exists" statements. `tools/validate_markdown_links.py` must pass for every link.
- `AGENTS.md` §10 currently declares generated workloads, benchmark execution and adapters
  outside the internal capture adapter "unauthorized". The merge plan (owner-authored,
  2026-09-08) authorizes this experiment; record that in §10 in this change, briefly, while
  preserving the existing research evidence and correctness requirements.

## Required changes

### R1. Workspaces

| Path | Contents |
|---|---|
| `experiments/convergence-memory/` | `Cargo.toml` (workspace, edition 2024, `rust-version = "1.89"`, same lints/profile as exp-0001), `rust-toolchain.toml` (1.89.0, minimal, rustfmt+clippy; no Linux-only target pin so it builds on the Windows dev box), `Cargo.lock`, crates: `cm-trace` (versioned trace/results format + independent expected-state model), `cm-candidate` (Memory record codec over RF1 + D1 append/replay + independent row/column rebuilds, path-depending on `exp1-record-format` and `exp1-raw-append-replay`), `cm-harness` (runner binary/example: executes traces, validates, measures, writes results) |
| `experiments/convergence-memory-legacy/` | `Cargo.toml` (own workspace, own `Cargo.lock`), `rust-toolchain.toml` (1.89.0), one crate `cm-legacy-runner` depending on `rusty_multimodal_db = { git = "https://github.com/baileyrd/rusty_multimodal_db", rev = "<pinned>", features = ["server"] }` and on `cm-trace` by path; executes the same traces through `MemoryConnectionStore` (single-shot requests and, where equivalent, protocol-22 batches) and emits results in the shared format |

The candidate workspace must stay external-dependency-free (lockfile with workspace/path
crates only) so it runs `--locked --offline` like exp-0001. The legacy workspace cannot: its
CI job must `cargo fetch --locked` with network before any `--offline` step (see R5).

### R2. Trace/results format (`cm-trace`)

A small, versioned, dependency-free binary or line format (`CMT1`) with: a header (format
version, trace id, seed, record counts), operations, and a results section. Operations cover
insert, get, field update, whole-record replacement, guarded update (compare-and-replace on a
declared field/version), delete and reinsert (same UUID), equality lookup, column aggregation
(count/sum/min/max on an integer field), ordered pagination by `updated_at_unix_ms` with
duplicate keys and explicit UUID tie-breaking. Every op carries a deterministic id. Results
record per-op outcomes (Inserted/Duplicate/Replaced/GuardFailed/NotFound/…), query results as
sorted per-record digests, and final-state digests (sorted per-record digest list plus per-column
digests). Include a golden fixture set with SHA-256s and a round-trip test.

### R3. Independent expected-state model

A plain in-memory model (BTreeMap by UUID, per-field values) that applies the trace with the
documented semantics and produces expected per-op outcomes, query results and digests. It
must be written from the operation semantics, not by calling either engine, and tested on the
small deterministic trace by hand-checked assertions.

### R4. Traces

- `small-deterministic`: ~50 ops, hand-readable, covering every op kind including duplicate
  ordering keys, guard failure, delete+reinsert, and a query after each mutation class.
- `repeated-mutations-{1k,10k,100k}`: seeded generator (splitmix64 or similar, no crate) over
  1K/10K/100K distinct records with repeated field updates, replacements, guarded updates,
  deletes and reinserts; realistic field sizes (content 200-2000 bytes, up to 8 tags, metadata
  JSON 50-500 bytes) with the shape parameters recorded in the trace header.

### R5. Candidate and legacy runners

Both runners: load a trace, execute it, compare per-op outcomes and query digests against the
expected model (fail loudly on mismatch, keep going to record all mismatches), and emit a
results file plus measurements. The candidate additionally performs independent row and
column reconstruction from the replayed history and compares both to the expected model, not
only to each other. The candidate uses the ordinary-write D1 path and must be labelled
"D1 ordinary writes, no fsync, no crash-survival claim" in every results file and document.
The legacy runner declares its durability setting (the mmap store's per-op behavior) so the
comparison names equivalent settings, and must not imply either engine survives power loss.

Measure complete per-operation cost and the append, replay, rebuild (rows, columns) and query
stages separately; record peak RSS (portable approximation acceptable, method stated), on-disk
size, dataset shape, compiler and dependency versions, hardware, warm/cold conditions, and 1
warm-up + at least 5 measured trials per size with distributions (not just medians). Follow
the run-metadata pattern of `examples/history_views.rs` (exclusive output directory,
`create_new`, git revision and porcelain status embedded).

### R6. CI

Add `.github/workflows/convergence-memory.yml` (or extend the existing workflow) running for
both manifests: fmt check; clippy `--workspace --all-targets --locked -- -D warnings`; test
`--workspace --all-targets --locked`; candidate with `--offline`; legacy with a preceding
`cargo fetch --locked --manifest-path experiments/convergence-memory-legacy/Cargo.toml` step
that has network, then `--offline` for build/test. Install 1.89.0 via rustup as the existing
workflow does. Do not run the 100K measurement in CI; run the small trace and 1K trace as
tests, and gate the measurement runs behind an example/binary invoked manually with the exact
command recorded in the experiment document.

### R7. Documents (same change, per AGENTS.md §7)

- `docs/experiments/EXP-0002-convergence-memory.md` following the 18-section structure, status
  `Ready` (or `Running` if measurements are included), predeclared interpretation criteria,
  raw result locations under `docs/experiments/EXP-0002/results/`.
- Register EXP-0002 in `docs/experiments/README.md`; add `Memory` (application record type from
  rusty_multimodal_db) and `B0` to `docs/GLOSSARY.md`.
- Update `experiments/README.md`, root `README.md` "Current phase", `crates/README.md` if
  touched, and `docs/PROJECT-STATUS.md`; add the merge plan to `docs/` (move the root file to
  `docs/plans/data-os-multimodal-merge-plan-2026-09-08.md` and link it) so the repo records the
  authorization; amend `AGENTS.md` §10 with one sentence authorizing EXP-0002 by reference to
  that plan, leaving the rest intact.
- Preserve everything under `experiments/exp-0001/` and `docs/experiments/EXP-0001/` unchanged.

## Non-goals

- No fsync/durable acknowledgement (step 3), no networking, no general schema catalog, no
  claim of database throughput superiority. No changes to exp-0001 code.

## Proof

```
cargo +1.89.0 fmt --manifest-path experiments/convergence-memory/Cargo.toml --all -- --check
cargo +1.89.0 clippy --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0 test --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0 fetch --locked --manifest-path experiments/convergence-memory-legacy/Cargo.toml
cargo +1.89.0 fmt --manifest-path experiments/convergence-memory-legacy/Cargo.toml --all -- --check
cargo +1.89.0 clippy --manifest-path experiments/convergence-memory-legacy/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0 test --manifest-path experiments/convergence-memory-legacy/Cargo.toml --workspace --all-targets --locked --offline
cargo +1.89.0 fmt --manifest-path experiments/exp-0001/Cargo.toml --all -- --check
cargo +1.89.0 clippy --manifest-path experiments/exp-0001/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo +1.89.0 test --manifest-path experiments/exp-0001/Cargo.toml --workspace --all-targets --locked --offline
python tools/validate_markdown_links.py
git diff --check
```

The host installs Rust 1.89.0 if absent (it is not installed on this machine yet) and runs the
measurement command for the 1K and 10K sizes once to check the results files are produced; the
100K run is reported as executed or not.
