# R19 — Slice C/B1 closure and descriptive D1 harness readiness

**Authority base:** `main` at Slice C/B1 merge `ef29804347faa812502f855e5cc3ffee6f4901c2`
**Status:** Slice C/B1 closed as bounded implementation/correctness-validation evidence; descriptive D1 harness implementation blocked pending one capture-interface decision
**Evidence classification:** governance/readiness decision; no workload was materialized or executed and no benchmark evidence was produced

## 1. Scope and authority audit

R19 closes only the R18-authorized raw D1 append and deterministic physical replay implementation,
then asks whether the existing R7/R12/R14/R16/R18 authorities and their reviewed implementations
uniquely determine the smallest generated-workload, descriptive D1 execution-and-capture harness.
The review followed the repository authority route and the R1–R18 links routed by the readiness
plan. Repository state, the reviewed implementation, and exact-head checks are evidence; this
record does not infer execution evidence from passing correctness tests.

R7 freezes the benchmark-record, artifact, provenance, instrumentation, environment, and
validation vocabulary. R12, R14, and R16 freeze the corrected canonical M01 generator, ordered
stream digest, manifest, and real R7-backed fixtures. R18 freezes the raw-appender/replayer
implementation boundary. These authorities are mutually consistent and permit no duplicate
codec, generator, manifest implementation, scanner, CRC implementation, or persistence
mechanism.

## 2. Exact Slice C/B1 closure

R18 prospective authorization entered `main` through PR #71:

* exact reviewed head: `21143b716de006dd5ec639c0b76a1b031d359fc1`;
* merge: `e9c292cd614d97b2bf299fd8d2637de76dcdca54`.

The authorized implementation entered `main` through PR #74:

* exact reviewed head: `5c448695f4e460cab57eaadd7f7a83bfce1559ab`;
* merge: `ef29804347faa812502f855e5cc3ffee6f4901c2`;
* exact-head **Documentation validation** passed; and
* exact-head **EXP-0001 Slice A** passed.

The reviewed crate is therefore closed only as bounded implementation/correctness-validation
evidence for:

1. complete-frame validation through the existing `exp1-record-format` implementation;
2. raw, process-local D1 write submission with starting-offset and byte-progress/error reporting;
3. poisoning after terminal write failure so later writes cannot hide or interleave residue;
4. deterministic, read-only physical accepted-prefix reopen and replay;
5. exact physical offsets, extents, ordinals, record bytes, and terminal classification; and
6. fail-closed format, resource-limit, terminal-damage, and read-I/O handling.

This closure is not workload execution or benchmark evidence. It proves no performance, stable
storage, namespace durability, acknowledged-byte survival, D2/D3, canonical recovery, fault
behavior, adapter behavior, or production readiness. The crate remains noncanonical and its
successful write result remains D1 OS-buffer submission only.

## 3. Smallest useful next boundary

The smallest useful candidate is one experiment-local binary package, provisionally named
`exp1-descriptive-d1-harness`, under
`experiments/exp-0001/crates/exp1-descriptive-d1-harness`. If separately authorized, it would be
the fourth workspace/default member and would use exactly three reviewed workspace path
dependencies: `exp1-workload-conformance`, `exp1-record-format`, and
`exp1-raw-append-replay`. Its external-dependency allowlist would remain empty. These names are a
readiness finding, **not authorization** to create the package or edit Cargo files.

The candidate would be a single-process, single-owner command with no subcommands and a closed
input boundary: an existing empty output root, caller-supplied canonical lowercase UUIDv4
`series_id` and `run_id`, the exact 40-cell R8 cell identifier restricted to the reference
single-producer P1–P3 minimal-envelope D1 cells, and explicit repository/reviewed-head identities.
It would reject unknown arguments, environment-variable configuration, reused/nonempty run
directories, symlinks, absolute artifact references outside the root, and any cell outside that
three-cell descriptive subset. It would emit only the closed run tree and a machine-readable final
success/failure disposition; it would not emit an interpretation.

The candidate deterministic run directory would be
`exp-0001/series/<series_id>/runs/<run_id>/`. The two caller-assigned IDs, frozen cell identity,
workload identity, repository head, toolchain/build identity, and all effective settings would be
bound into every applicable record and manifest. Reusing an identity with different inputs or an
existing directory would fail closed. R7 record IDs would also be caller/authority assigned UUIDv4
values unless a later owner decision uniquely freezes an assignment mechanism; content-derived,
randomly improvised, timestamp-derived, or silently regenerated identities are forbidden.

## 4. Candidate materialization, execution, and oracle sequence

If the blocker in section 7 is resolved without changing the authorities, a later authorization
may freeze this exact sequence:

1. Validate the closed CLI, empty output root, cell, identities, repository state, effective
   settings, and capture capability before creating evidence.
2. Use `exp1-workload-conformance`—not copied algorithms—to materialize the corrected canonical M01
   operations, S01 stream, 3,423-byte M01 manifest, 1,274-byte stream artifact-manifest fixture,
   and independent 1,152-byte workload-manifest artifact fixture. Validate exact bytes, counts,
   identities, bindings, lengths, provenance endpoints, artifact digests, workload-stream digest,
   and manifest digest against R12/R14/R16.
3. Convert each ordered semantic operation to its already-frozen EXP1-B1-RF1 physical frame only
   through `exp1-record-format`. Validate each complete frame before presenting it to the appender.
   No second encoder, CRC, or framing path is permitted.
4. Capture the pre-run environment and effective settings, then perform exactly one process-local
   descriptive D1 pass for the selected P1, P2, or P3 minimal-envelope cell. Submit frames in
   stream order through one `RawAppender`; retain every receipt and abort on the first failure.
   Never retry a failed run under the same run identity.
5. Drop/close the appender without synchronization, reopen the raw artifact read-only, and call
   `reopen_and_replay` with the reviewed limits. Clean EOF is required for a successful observation.
6. Independently compare input frames, receipts, raw bytes, replay records, and accepted prefix.
   Validate counts, byte counts, offsets, extents, physical ordinals, accepted-prefix identity,
   exact replay bytes, and no invention, loss, duplication, or reordering. Classify the run only as
   D1 and noncanonical.
7. Finalize validation reports and the run artifact/provenance graph only after every referenced
   byte length and digest recomputes. Missing or inconsistent metadata, capture, record, artifact,
   digest, reference, or edge makes the run invalid and prevents publication.

No timing threshold, throughput/latency conclusion, comparison, estimator, confirmatory rule, or
performance interpretation is evaluated. Numeric observations required by the R7 raw-result
schema are descriptive raw observations only.

## 5. Required candidate records and artifact graph

The candidate cannot weaken R7. A complete descriptive observation requires stored JCS bytes for
the applicable `environment`, `raw_result`, `artifact_manifest`, and `validation_report` records;
the corrected workload manifest and both R7-backed fixture records; the materialized workload
stream; the ordered EXP1-B1-RF1 input-frame artifact; the raw appended artifact; the receipt and
lifecycle ledgers; replay bytes/report; effective-setting capture; and immutable provenance.
Fault-plan and fault-outcome records are not applicable because this tranche forbids fault actions.

Every artifact has an assigned identity, logical path, exact byte length, R7 domain-separated
SHA-256 digest, media type, role, sensitivity, retention state, creating record, and validation
reports. Required `generated_from` and `validated_by` edges connect materialized inputs to the
workload authority, input frames to the stream, the raw artifact to input frames and receipts,
replay output to the raw artifact, and records to their capture inputs. Raw workload, appended,
receipt, lifecycle, replay, environment, and validation artifacts are retained even for invalid or
inconclusive runs. No deletion, redaction, relocation, publication service, or Git storage is
selected by this record.

Lifecycle is closed and monotonic:

```text
preflight -> staged -> materialized -> validated-input -> submitting
          -> reopened -> replay-validated -> captured -> validated -> published
```

Any failure transitions once to `invalid` (or `inconclusive` only where R7 uniquely requires it),
preserves safely captured artifacts, and cannot resume or publish under that run identity.
Successful publication requires the R7 publication gate; process exit, close, reopen, or clean EOF
alone is not publication or durability.

## 6. Candidate deterministic correctness and CI boundary

A later implementation would require deterministic tests for: closed CLI parsing; identity and
directory collision rejection; exact M01/fixture materialization; all three permitted cells;
invalid cell/profile rejection; stream/manifest/artifact digest mismatch; frame validation before
write; exact receipt/count/offset/ordinal accounting; byte-identical accepted prefix and replay;
no invention/loss/duplication/reordering; D1/noncanonical labels; missing capture and provenance;
unknown/missing R7 fields; inconsistent references; partial output retention; nonpublication after
failure; and repeatable output bytes given identical supplied identities and captured fact values.

The CI boundary would remain the unchanged Rust 1.89.0 workspace formatting, locked/offline clippy,
locked/offline all-target tests, documentation links, `git diff --check`, and changed-path,
dependency, and forbidden-scope audits. Tests may use deterministic test-owned temporary paths and
synthetic capture values; they may not execute M01 as an observation, benchmark the mechanism,
alter machine configuration, or claim that synthetic records are evidence.

## 7. Readiness determination and exact blocker

**The descriptive D1 harness is blocked and is not prospectively authorized by R19.** The workload,
frame, append, replay, oracle, layout, and record schemas are sufficiently constrained, but the
required effective environment/instrumentation capture implementation is not uniquely realizable
inside the mandated dependency boundary.

R7 requires direct Linux capture including `clock_gettime(CLOCK_MONOTONIC_RAW)`,
`clock_gettime(CLOCK_REALTIME)`, `getrusage`, process/procfs facts, file metadata, and explicit
availability/loss dispositions. It deliberately does not select a Rust crate. The existing
workspace forbids unsafe code, contains no platform-interface dependency, and exposes no reviewed
capture implementation. Safe `std` does not uniquely expose all named R7 interfaces or their
required effective-setting facts. Selecting an FFI crate, permitting local unsafe bindings,
substituting `std::time`, invoking external commands, or marking collectable required fields
unsupported would each be a consequential new implementation/platform policy. R7 and R18 do not
choose among them. Silently choosing one would invent behavior and would invalidate the claimed
R7-compliant capture boundary.

The smallest decision needed is one owner-reviewed, repository-recorded capture-interface freeze
that:

1. selects the exact safe implementation path for every required R7 environment, clock, resource,
   filesystem, and instrumentation field in this descriptive D1 subset;
2. names any permitted external crate with exact version/license/build policy **or** explicitly
   authorizes and bounds unavoidable platform-specific unsafe FFI (or freezes an equally exact
   dependency-free process/procfs method);
3. freezes unavailable-field and command/tool-version behavior without weakening R7;
4. freezes caller/owner assignment and capture for series, run, record, artifact, and provenance
   UUIDv4 identities; and
5. confirms that the resulting implementation remains valid for the Fedora 44 target without
   making BLK-015 survival or fault claims.

Until that decision merges, no fourth crate, binary, Cargo/workspace edit, executable R7 record
producer/validator, workload materialization, descriptive run, or evidence capture is authorized.
This keeps UNK-022 and the executable-capture portion of BLK-020/021/026/027 open. The frozen
candidate boundary above is the maximum scope that the smallest decision may unlock; any broader
dependency, platform, instrumentation, identity, or execution design requires a new readiness
review.

## 8. Retained exclusions and continuation gate

R19 does not implement or execute the harness. It authorizes no actual workload or benchmark run,
confirmatory execution, threshold application, interpretation, performance claim, D2/D3,
`fsync`, crash/reset/power-loss/storage-error action, BLK-015-dependent survival claim, SQLite or
RocksDB adapter, production crate, server, network, query, distributed, security, authentication,
secret handling, machine change, or later tranche.

Continuation requires the exact section 7 decision, a new documentation/governance authorization
against then-current `main`, and review of the exact candidate boundary before any Cargo or Rust
change. Implementation completion would require exact-head CI plus review of every frozen record,
artifact, failure, dependency, and exclusion obligation. Even then, implementation would authorize
no execution: a separately reviewed descriptive execution gate must validate a real target
environment, series identity, cell, effective capture, artifact location/retention, and publication
path before one observation is run. Confirmatory work remains a separate later gate.
