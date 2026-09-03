# EXP-0001 R28 — R27 closure and end-to-end D1 integration authorization

**Contract:** `EXP-0001-R28-END-TO-END-D1-CORRECTNESS-v1`
**Status:** R27 implementation closed; one test-only integration increment prospectively authorized
**Evidence classification:** bounded implementation/correctness-validation evidence and governance
authorization; not workload execution, benchmark evidence, environment capture, durability evidence,
recovery evidence, or a performance result
**Decision date:** 2026-09-03

## 1. R27 implementation closure

R28 closes the pure v2 reference-context implementation authorized by R27. PR #98's reviewed
exact head is `67715b3efc4732542152ea9d935d92ebdb2ca0d6`; its merge commit is
`f5cde575cbd82bb788b9519c4efc56e4d1186131`. The **Documentation validation** and **EXP-0001
Slice A** workflows both succeeded for that exact reviewed head. The merged implementation uses
the two existing reviewed path dependencies and implements the frozen homogeneous-v2 closed-scope
constructor, immutable catalog, private caller-owned accepted prefix, and transactional contextual
SOP2-to-RF1 mapper while preserving v1 behavior.

This closes the remaining R20 reference-context correctness gate only as bounded
implementation/correctness-validation evidence. It proves conformance of the pure mapper to the
R21–R23 and R25–R27 contracts; it does not prove append integration, workload or benchmark
execution, stable storage, canonical recovery, D2/D3 durability, performance, capture, faults,
adapter behavior, or production readiness.

## 2. Smallest useful next question

The independently reviewed pieces now cover literal SOP2 conformance, contextual mapping, RF1
encoding, raw D1 append submission, and deterministic physical reopen/replay. They have not yet
been exercised in one test that proves their first complete correctness path. The smallest useful
next increment is therefore one deterministic integration test using the merged literal oracle and
the existing public APIs. It changes no implementation and measures no performance.

R28 authorizes exactly one follow-on PR. Its non-documentation change may update only:

* `experiments/exp-0001/crates/exp1-raw-append-replay/tests/reference_context.rs`.

That PR may additionally update only synchronized status, readiness, and traceability documentation
at closure. No production Rust source, Cargo manifest, lockfile, dependency, workspace membership,
toolchain, or workflow change is authorized or expected.

## 3. Frozen positive integration test

The test must use the checked-in, merged R26 literal manifest and WS2 fixtures as its independent
oracle; it must not generate replacements or copy fixture values into production source. It shall:

1. load and validate the literal v2 closed-scope descriptor and its manifest, WS2, artifact
   metadata, digest, and provenance bindings, then construct the real R27 v2 closed-scope context
   for the fixture's selected stream;
2. extract all four literal SOP2 operations from that validated WS2, without regenerating or
   rewriting them;
3. starting from `context.initial_state()`, map the operations sequentially in WS2 order with both
   assigned sequence and physical ordinal values `1`, `2`, `3`, and `4`, carrying the returned
   caller-owned state into the next call;
4. for every successful mapping, assert that the borrowed input state remains unchanged, the next
   state advances exactly once, and the returned record and frame equal the independently reviewed,
   byte-exact RF1 expectations;
5. create a fresh, test-owned temporary file using only `std`, with collision-safe exclusive
   creation and no repository or shared path;
6. append the four already-mapped complete frames in order through one `RawAppender`, retaining all
   four receipts;
7. assert each receipt's starting offset and byte count, assert offsets are contiguous from zero,
   and assert the appender remains non-poisoned after every successful append;
8. drop the appender without requesting synchronization, call `reopen_and_replay` with explicit
   limits sufficient for exactly these frames, and require `CleanEof`;
9. assert that accepted-prefix bytes equal the concatenation of the four mapped frames and that
   scanned bytes equal that length; and
10. assert record count, each physical offset and extent, physical ordinals `1` through `4`, decoded
    record values, exact frame bytes, and stream order against the mapped inputs, including an
    explicit one-to-one comparison proving no invention, loss, duplication, or reordering.

Expected RF1 bytes must be derived in the test through the already-reviewed authority/codec path or
checked against existing reviewed literals; the test must not add a second encoder, scanner,
mapping algorithm, or CRC implementation. Explicit replay limits are part of the test input and
must not rely on permissive defaults.

## 4. Frozen pre-append failure test

Before any successful append in the same test-owned artifact lifecycle, the test must offer a
contextually invalid literal operation/state combination to
`map_semantic_operation_v2_with_context` and require the exact applicable contextual error. It must
prove that this failed mapping:

* returns no mapped record/frame and therefore creates no append receipt;
* leaves the caller-owned accepted-prefix state unchanged;
* performs no `RawAppender::append` call; and
* leaves the fresh file length at zero.

Only after those assertions may the positive four-operation path proceed. This is the required
transaction boundary: a mapping failure cannot leak an output, advance state, or grow the physical
artifact.

## 5. Temporary-file and failure cleanup contract

The test must use only `std` for temporary-path ownership and file creation. It must fail closed if
exclusive creation collides rather than truncate or reuse a file. A test-local RAII cleanup guard
must own the path immediately after creation and remove it in `Drop`, so cleanup runs during normal
completion and assertion unwinding. The test may explicitly remove the file at successful
completion only if the guard is then disarmed; cleanup must tolerate the file already being absent
without masking the primary assertion failure. No external temporary-file dependency is authorized.

## 6. Evidence boundary and exclusions

This follow-on is strictly deterministic integration/correctness testing. Its file I/O occurs only
inside a test to connect already-reviewed components. Dropping the appender without synchronization
and observing `CleanEof` proves only same-process D1 physical reopen/replay for submitted bytes; it
is not `fsync`, stable-storage, namespace-durability, acknowledged-byte-survival, canonical recovery,
crash/power-loss, environment-capture, workload-execution, benchmark, latency, throughput, or other
performance evidence.

R28 does not authorize a fourth crate, executable or descriptive D1 harness, executable R7 record
production, live Linux capture implementation, generated workload materialization or real
experimental execution, `fsync`, D2/D3, faults, adapters, SQLite or RocksDB execution, production
code, unsafe code, server/network/query/distributed work, architecture promotion, or any expansion
of EXP-0001 evidence claims. The live-Linux-capture decision named by R19 remains open and continues
to block the descriptive D1 harness independently of this test.

## 7. Completion gate

Closure of the authorized follow-on requires the exact path boundary above, the complete positive
and negative assertions in sections 3–5, synchronized documentation, review of the exact
implementation head, both existing workflows green for that head, the unchanged R9 format,
locked/offline clippy, and locked/offline all-target test commands, plus `git diff --check` from the
repository root. Closure may record only bounded deterministic integration/correctness evidence;
every exclusion in section 6 remains in force.
