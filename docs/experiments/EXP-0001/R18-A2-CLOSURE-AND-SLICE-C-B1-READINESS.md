# R18 — Slice A2 closure and bounded Slice C/B1 readiness

**Approval:** GitHub issue #65, `EXP-0001-R18/a2-closure-b1-readiness-v1`
**Authority base:** live `main` at PR #68 merge `79cbd64a436b104835a4279c07ba2777fb06cddb`
**Status:** A2 closed; one bounded Slice C/B1 implementation prospectively authorized after this record is reviewed and merged
**Evidence classification:** governance/readiness decision; the referenced A2 result is implementation/correctness-validation evidence only

## 1. A2 closure

PR #64's historical implementation head
`d2ee72aa4ff047d4cfcaa1df82d83f13566568f2` entered `main` through merge commit
`9b5d89a36ed71d38420e9ae19f59d441a9d927aa`. Final A2 closure additionally required corrective
PR #68 at exact reviewed head `fcaf7f14c94df5a6cda1aeeb283b6726551d1844`. Its required exact-head
**Documentation validation** and **EXP-0001 Slice A** workflows both passed, and it entered `main`
through merge commit `79cbd64a436b104835a4279c07ba2777fb06cddb`. The corrected R17 gate is
therefore satisfied and Slice A2 is closed. Its result is bounded implementation/conformance and
correctness-validation evidence for the dependency-free R12/R14/R16 subset. It is not
generated-workload execution, storage, recovery, benchmark, durability, performance, or production
evidence.

## 2. M01 and R7 publication boundary

The corrected live R16 authority is unchanged by R18. M01 is the canonical, positively valid
R7-backed serialization/conformance vector. Its references bind the real 1,274-byte stream
artifact-manifest fixture and the independently supplied 1,152-byte workload-manifest artifact
fixture; their exact bytes, lengths, identities, metadata, provenance, and digests remain
authoritative. R18 neither restores any pre-correction literal nor weakens full caller-supplied
reference validation.

## 3. Readiness determination

R1/R3/R5 are sufficient for a smallest useful, process-local, file-backed raw append plus
reopen/replay **correctness** tranche below the durability boundary. BLK-015 does not block it:
the tranche exercises ordinary file API behavior and deterministic preserved-byte scanning only,
labels every successful append as D1 OS-buffer submission and noncanonical, and makes no survival
claim. A file surviving close/reopen is a test precondition/observation, never evidence of D2/D3,
stable media, `fsync`, power-loss recovery, or canonical commit.

After this R18 record is reviewed and merged, it prospectively authorizes exactly the boundary in
section 4. It does not authorize execution of a workload or benchmark.

## 4. Frozen implementation boundary

### 4.1 Repository, build, and CI

* Add exactly one experiment-local library crate at
  `experiments/exp-0001/crates/exp1-raw-append-replay`; add it as the third member and third
  default member of the existing `experiments/exp-0001` workspace. No file under `/crates/`.
* Reuse Rust 1.89.0, Edition 2024, resolver 3, the existing target, lockfile, profiles, lint policy,
  rustfmt, and unchanged **EXP-0001 Slice A** CI commands. `unsafe` and unreviewed `cfg` remain
  denied. No workflow, toolchain, target, profile, or repository-level Cargo change.
* Declare exactly one workspace path dependency on `exp1-record-format` and no external
  dependency. The external dependency allowlist remains empty. No features, build script, binary,
  examples, benches, FFI, networking, threads, async runtime, clock, randomness, serialization
  framework, database, or platform-specific third-party crate. Apart from the reviewed Slice A
  crate, use only `std`; Linux production placement/provenance validation is not implemented.

### 4.2 API and write behavior

The library exposes a single-owner raw appender whose operation accepts one already-encoded,
complete `EXP1-B1-RF1` record as `&[u8]` and returns either a D1-only submission receipt containing
the starting logical offset and complete byte count, or an explicit error containing the safely
known progress. Construction/semantic-envelope generation, sequencing, lifecycle transitions,
acknowledgement delivery, D2/D3 finalization, and canonical commit are outside the API.

The appender opens one caller-supplied test path for append/create using safe standard-library file
APIs, serializes calls through one mutable owner, and loops until the complete record is submitted.
Positive short progress advances by exactly that amount; interruption before progress retries;
zero progress, I/O error, offset overflow, invalid input, or partial terminal submission returns an
explicit failure and poisons the appender so no later append can interleave with or hide residue.
It never calls `sync_all`, `sync_data`, `fsync`, or a directory synchronization operation, and
never promises placement, namespace durability, stable storage, or recovery of acknowledged D1
bytes. Close/drop success is not an acknowledgement or durability boundary.

Before writing, the input must be exactly one complete R5 frame: validate the 32-byte header,
checked lengths and configured 16 MiB limit, exact extent, physical profile/type eligibility,
body-shape rules applicable without external lifecycle state, and CRC32C-1 when selected. Profile
0 is accepted only for R5's provisional types 1, 3, and 4 with a zero integrity field. Types 2, 5,
and 6 require profile 1, but their presence never makes this D1 tranche canonical. Unknown or
unsupported values, trailing bytes, malformed lengths, CRC failure, or multiple concatenated
frames fail before any write. Existing Slice A codec/scanner behavior MUST be reused through that workspace path dependency;
normative framing, CRC, and scanner logic must not be forked inconsistently.

### 4.3 Reopen, replay, and failure semantics

Reopen/replay opens the caller-supplied artifact read-only, scans from byte zero using the exact R1
and R5 limits and `EXP1-B1-RF1` framing/integrity rules, and returns a deterministic report plus the
accepted **physical** prefix. Replay means ordered re-emission of validated record bytes and safely
decoded metadata; it is not canonical-event recovery. The report preserves offsets, extents,
physical ordinals, profile/type, CRC disposition, terminal classification, and all safely known
errors. It never mutates, truncates, repairs, skips, searches for later magic, or invents records.

Clean EOF at a boundary accepts the complete physical prefix. A terminal header fragment or a
valid header whose extent crosses EOF reports terminal truncation, excludes the incomplete suffix,
and retains the preceding accepted physical prefix. Impossible/malformed lengths, trailing
garbage, CRC failure, unsupported identity/profile/type, non-increasing physical ordinal,
body-shape conflict, interior/ambiguous damage, resource-limit or I/O failure stop fail-closed at
the last validated prefix and return an overall unsuccessful scan. Later bytes are never promoted.
Repeated scans of identical bytes and limits must return identical reports and replay bytes.

Canonical-history invariants remain strict: canonical history is the sole authority; canonical
events are accepted facts rather than commands; event/effective/system/durability time meanings
remain distinct; derived/materialized state remains rebuildable and non-authoritative; and no D1
record or replay result is labeled canonical. Type-5/type-6 pairs may be structurally reported but
canonical eligibility is deliberately undecidable without the excluded lifecycle/platform proof,
so this API does not emit canonical events.

### 4.4 Deterministic correctness gate

Tests use test-owned temporary paths with deterministic names/content and clean them up. They must
cover: empty artifact; one and adjacent R5 stable vectors; exact offsets/byte counts; create,
append, close, reopen, and byte-identical ordered replay; profile-0 eligibility; CRC32C check vector;
positive-short-write, interrupted-before-progress, zero-progress, partial-write, and injected I/O
state-machine paths through an internal deterministic writer seam; poison/no-later-write behavior;
every header/body truncation boundary; terminal truncation after a valid prefix; malformed,
overflowing, excessive, unknown, CRC-corrupt, trailing-garbage, non-increasing-ordinal, and
interior/ambiguous cases; resource limits; repeat-scan identity; and input immutability. Fixtures
and expected outcomes remain authority-derived and independent of code under test.

The implementation PR must pass the unchanged repository link, formatting, locked/offline clippy,
locked/offline workspace-test, `git diff --check`, changed-path, dependency, and exclusion audits
at its exact reviewed head. Passing proves only the bounded behavior tested.

## 5. Retained blockers and exclusions

BLK-017 is implemented only for the raw D1 append/replay correctness subset after the authorized
implementation passes review; BLK-020/026/027 extend only to the third external-dependency-free workspace member with exactly one reviewed workspace path dependency on `exp1-record-format` under unchanged CI. BLK-015 remains open and continues to block every D2/D3, filesystem
placement/protection, `fsync` survival, physical fault, and power/reset claim. BLK-020 remains open
for benchmark/capture architecture; BLK-026/027 remain open for any later/native/series expansion.

Not authorized: generated workloads or workload execution; descriptive or confirmatory execution;
benchmarks, timing, throughput, latency, or performance interpretation; `fsync`/D2/D3; canonical
commit/recovery claims; physical reset/power loss, destructive or synthetic fault injection;
production promotion; adapters, SQLite, RocksDB, server, network, query, distributed, security,
authentication, secrets, or unrelated workspace/toolchain work. Slice C/B1 implementation itself
is not performed by R18, and its completion authorizes no subsequent tranche.
