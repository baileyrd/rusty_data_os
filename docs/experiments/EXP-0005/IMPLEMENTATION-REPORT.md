# EXP-0005 implementation report

Date: 2026-09-09. Advisory implementation report; independent provider review pending.
No commit, push, publication or PR was made.

## Authority and source identity

Active checkout: `C:/dev/rusty_data_os-step4b`, branch
`codex/merge-step4b-protocol-facade`, unchanged HEAD
`d90b39ce42f8a852c2a1729646def511239fbb14`. Initial status was clean.
The frozen work-order file matched SHA-256
`66a53b9f66b49e287adeb6bd14c35507fd0ef7607c048141da285f4f37457d80`.
Legacy read-only source HEAD was `232b16ecb3fd89b318ae4730c8470ab6da52f330`, clean.
All implementation writes resolve within the active checkout; no original checkout was edited.

The wire fixture is the exact original 70 lines plus one provenance header (71 total).
Original fixture SHA-256: `cf92d14bdf4a9f0d829d70a79fdd49b11dea933c201536ebf0f4db0db117591f`.
The host expected-values copy is byte-identical, SHA-256:
`e0695b85ee14988f3c59ed7ae2910261ee006bdabd5061f4af5a51951b5fdced`.

## Implementation and acceptance mapping

| Requirement | Result |
|---|---|
| R0 | First mutation added the exact AGENTS §3 and Roadmap Phase 7 exception text. Phase 1 and historical research evidence remain intact. |
| R1 | Added only uc-protocol to the five existing workspace members, plus its lockfile entry. No dependencies beyond std. |
| R2/R3 | Complete protocol types and explicit-tag fixed-width codec, separate 16 MiB framing, fallible malformed/truncated/unknown-tag/trailing-byte decoding, generic Read/Write only. |
| R4/D6 | `all_66_host_literal_values_and_byte_exact_round_trips` asserts exactly 66 cases, each full decoded value and byte-exact re-encoding. Host text is paired with checked-in Rust literal cases; the transcription script never reads wire bytes or codec source. |
| R5/R6 | Complete Store method surface, including page_keys, detach_record, apply_write_op and fail-closed atomic defaults; real Transaction and Join dispatch, query/aggregate/page evaluation and exact legacy outcome/error text. |
| R7 | Shared immutable Registry, per-connection table/session/Hello state, all eleven guards, caps, three flag behaviors, original read-set commit checking through the Store contract, foreign Link/Join and Delete/batch detaches. |
| D7 | `connection::interleaving::shared_registry_two_connection_interleaving_prevents_dangling_edge` passes, with a deterministic mutex-removal negative control. |

D7 uses two independent in-memory duplex pairs and two registry clones sharing the same
mutex. Connection A captures a found far endpoint and pauses inside the check; B consumes
a Delete frame. try_lock assertions prove ownership through the foreign check, local Link,
local Delete and detach. With protection the enforced order is get → link → delete → detach,
leaving no record and no dangling edge. The negative control removes only the mutex and
forces get → delete → detach → link, reproducing a dangling edge. No sleeps or scheduling
probability determine the result; channel timeouts only bound test failure waits.

The 38 new tests cover 66 literal fixtures, malformed codec/frame cases, Store defaults and
dispatch/query/aggregate/page outcomes, Hello/auth behavior, flag combinations/version gates,
transaction precondition rollback, tracked-read conflict/repeatability after deletion,
read tracking capped at 4096, all eleven session guards with no adapter mutation calls,
4096 accepted/4097 refused staged writes and batches, cross-schema Join codes, foreign
Link checks, detach order and Storage partial state, both batch modes and shared locking.

## Proof

The full agreed eleven-command chain exited **0**. All three formatting checks and
all three locked/offline Clippy runs passed with warnings denied. Tests: **97 unified-commitment**
(59 unchanged existing + 38 new protocol), **17 convergence-memory**, **95 portable exp-0001**:
**209 total passed, 0 failed**. All **66 fixture lines passed both literal-value and byte
round-trip checks**. The named D7 interleaving test and its negative control passed.
Markdown links and git diff --check passed. Only the final results/report documentation
was updated afterward; final Markdown and whitespace checks were repeated.

Full raw output: [proof-output.txt](proof-output.txt). The agreed chain was run from the
checkout root with Windows GNU Rust 1.89.0, using && to stop on the first failure:

```powershell
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/unified-commitment/Cargo.toml --all -- --check &&
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings &&
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/unified-commitment/Cargo.toml --workspace --all-targets --locked --offline &&
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/convergence-memory/Cargo.toml --all -- --check &&
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings &&
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/convergence-memory/Cargo.toml --workspace --all-targets --locked --offline &&
cargo +1.89.0-x86_64-pc-windows-gnu fmt --manifest-path experiments/exp-0001/Cargo.toml --all -- --check &&
cargo +1.89.0-x86_64-pc-windows-gnu clippy --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline -- -D warnings &&
cargo +1.89.0-x86_64-pc-windows-gnu test --manifest-path experiments/exp-0001/Cargo.toml --workspace --exclude exp1-descriptive-d1-harness --all-targets --locked --offline &&
python tools/validate_markdown_links.py &&
git diff --check
```

Development checks initially found an Option::take name collision (renamed the codec method
to read_wire), a missing guard mapping in the literal transcription helper, and Clippy
condition-style diagnostics; these were fixed before the agreed proof. One new scan test
expected only the modified row but the fixture also contained an original row with value 999;
the literal expected result was corrected to include both. No frozen fixture was changed.
An intermediate documentation edit used the Windows default encoding; original UTF-8 text
was restored and the final existing-document diffs contain only intended additions.

## Deviations and implementation choices

One contradictory R7 test sentence says a session-blocked Delete has zero effect and is
“confirmed absent after Rollback.” If that means the existing record becomes absent, both
conditions cannot hold. Reported during implementation; proposed interpretation implemented:
the deletion effect is absent, so the existing record and its edges remain after Rollback.
The guard test verifies this explicitly. Independent review should confirm that interpretation.
No other work-order requirement was changed.

Authorized choices: uc-core::Uuid lacks Ord, so D3 permits a separate 16-byte RecordId and
std-only crate. Registry construction rejects empty/duplicate/mismatched table names and
invalid primary indexes; this keeps registration and detach routing consistent. Legacy
page_keys was included to preserve the full Store method surface. R5's fail-closed atomic
write_batch default intentionally replaces the unsafe legacy low-level pipelining behavior
as explicitly directed by the work order.

## Denied/blocked actions and retained limits

No implementation or agreed-proof action required an approval escalation. The first shell's
startup profile attempted unrelated Terminal-Icons preference-file and user-PATH registry
writes; the sandbox denied those actions. Further shells used login=false to bypass the
profile. rg was unavailable; file/text inspection used Git, PowerShell and Python instead.
No network access, original-checkout write, commit, push, publish or socket binding was attempted.

No real Memory/Entity/Relation Store is wired. Existing uc-core, uc-memory, uc-entity,
uc-relation, uc-harness, other workspaces and CI source files are unchanged. Authentication
always succeeds; no protocol-version content rewriting is supplied, with the explicitly
required BeginWith flag gates preserved. Store implementers remain responsible for adapter
atomicity and stable metadata; Registry only coordinates served relationship spans. A
Storage detach failure leaves the prior delete/earlier detaches applied, as frozen.
No performance, durability, production or complete Step 4 migration claim follows.

## Files changed

12 modified tracked files and 25 new files (37 total). Paths below are relative to the active checkout.

- `AGENTS.md`
- `README.md`
- `docs/GLOSSARY.md`
- `docs/PROJECT-STATUS.md`
- `docs/RESEARCH-QUESTIONS.md`
- `docs/RESEARCH-ROADMAP.md`
- `docs/TRACEABILITY.md`
- `docs/adr/ADR-0005-protocol-facade.md`
- `docs/experiments/EXP-0005-protocol-facade.md`
- `docs/experiments/EXP-0005/IMPLEMENTATION-REPORT.md`
- `docs/experiments/EXP-0005/proof-output.txt`
- `docs/experiments/README.md`
- `docs/hypotheses/HYP-0005-protocol-facade.md`
- `experiments/README.md`
- `experiments/unified-commitment/Cargo.lock`
- `experiments/unified-commitment/Cargo.toml`
- `experiments/unified-commitment/README.md`
- `experiments/unified-commitment/crates/uc-protocol/Cargo.toml`
- `experiments/unified-commitment/crates/uc-protocol/README.md`
- `experiments/unified-commitment/crates/uc-protocol/src/codec.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/connection.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/dispatch.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/framing.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/lib.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/query.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/store.rs`
- `experiments/unified-commitment/crates/uc-protocol/src/types.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/conformance.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/dispatch.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/fixtures/expected_cases.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/fixtures/generate_expected.py`
- `experiments/unified-commitment/crates/uc-protocol/tests/fixtures/step4b-fixture-expected-values.txt`
- `experiments/unified-commitment/crates/uc-protocol/tests/fixtures/wire-vectors.txt`
- `experiments/unified-commitment/crates/uc-protocol/tests/relationships.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/sessions.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/support/interleaving.rs`
- `experiments/unified-commitment/crates/uc-protocol/tests/support/mod.rs`
