# Host dispositions after the independent inspection (fix round 2, final)

The independent inspector (fresh Claude CLI, session ea7b3371-76d5-470b-950b-bb3e28fab7e5)
returned APPROVED with six low-severity findings. The host proof on the current snapshot is
green (fmt, clippy ×3, both test runs except the two excluded Windows-only tests, python 5/5,
`cargo +1.88.0 check` on the GNU toolchain PASS). Apply the items below exactly; nothing else.
Do not commit. Rerun the proof chain and report raw outputs.

## I1 (accept) — `dispatch` is table-local; say so

Add one sentence to `dispatch`'s `WriteBatch` arm comment and to the `ConnectionStore::write_batch`
and `write_batch_checked` docs: `dispatch` and `write_batch` are table-local low-level entry
points; the served path is `write_batch_across`, which adds registry-aware Link checks and
Delete cascades. Update the stale trait doc sentence about the atomic fallback ("aborts at op
0 with Unsupported … because the default cannot hold one lock") to describe the current
default (`write_batch_checked` refuses non-empty atomic batches). Docs only; do not change
`dispatch` behavior.

## I2 (accept, minimal) — Employee atomic batches

Keep the refusal. Add one line to `EmployeeConnectionStore`'s docs (research-gated reference
material) stating that non-empty atomic batches are refused at index 0 with `Unsupported`
while pipelined ops, including `collaborates_with` links, keep working. Add a unit test (in
`src/server/employee.rs` or serve.rs tests, whichever already has an Employee fixture) pinning
`write_batch_across(.., atomic=true)` => `TransactionFailed { index: 0, code: Unsupported }`
with nothing applied, and pipelined Link => `Linked`.

## I3 (accept) — concurrent test hygiene

In `batch_concurrent_link_and_delete_leave_no_dangling_edges`, keep the two `JoinHandle`s,
`join()` them after the `recv_timeout` waits, and propagate a panic so an assertion failure in
a writer thread reports as that assertion, not as the 10 s "lock cycle" timeout. Alternatively
return `Result`s from the threads and assert in the test body.

## I4 (accept, option B) — detach `Storage` in atomic mode

In `write_batch_across` atomic mode, when a cross-table detach after a successful own-table
apply fails with `Storage`, answer `Response::BatchResults` with `WriteResult::Failed(Storage)`
in that Delete's slot (and the real results elsewhere), exactly as the pipelined path does,
instead of `TransactionFailed`. Rationale: `TransactionFailed` tells the client nothing was
applied, which is false here and would drive a retry that duplicates work. Update the unit
test `batch_detach_failures_match_single_delete_and_not_found_skips_detach` expectation for
`(Some(true), Storage)`, the ADR-0060 paragraph and the report. Precondition failures keep
`TransactionFailed`.

## I5 (accept) — record the insert-log authorization

In the report's deviation paragraph and the ADR entry, state that the `insert_log.rs`
write-handle fix was authorized by the host coordinator in the build feedback of 2026-09-08
(F3 in that feedback), outside the work order's file list, and that the host will commit it
separately from the batch repair. Docs only.

## I6 (accept) — spec and index wording

Move the SERVER-001 prose that now sits above the `Version:` line into the spec's change
history as a `v0.50.1` entry written in the spec's own vocabulary (what WriteBatch guarantees,
the relationship mutex, adapters with/without atomic support, detach failure reporting). Bump
the `Version:` line to v0.50.1 if the spec's convention is to bump for a behavior clarification;
if the convention requires an SPEC-REGISTRY/TRACEABILITY version cell change, make it. Replace
the "Host revision F1/F2" and "pending independent review" banners in PROJECT-STATUS, ROADMAP,
SPEC-REGISTRY and TRACEABILITY with one neutral sentence each pointing at the report (or
remove them where the registry row already carries the pointer). No review-process vocabulary
in repo documents.

## Proof (unchanged)

```
cargo fmt --all -- --check
cargo clippy --all-targets --features server,research -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features client -- -D warnings
cargo test --features server,research
cargo test --features client
python -m unittest discover -s clients/python/tests -v
```

Known, host-excluded Windows-only failures: `dog_server` certificate-path `:` split and the
live Python test's `python3` name. Everything else must pass.
