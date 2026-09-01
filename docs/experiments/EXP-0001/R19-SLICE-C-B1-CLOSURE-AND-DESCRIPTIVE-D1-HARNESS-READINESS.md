# R19 — Slice C/B1 closure and descriptive D1 harness readiness

**Authority base:** `main` at Slice C/B1 merge `ef29804347faa812502f855e5cc3ffee6f4901c2`
**Status:** Slice C/B1 closed as bounded implementation/correctness-validation evidence; descriptive D1 harness implementation blocked pending semantic-to-physical mapping and live Linux capture decisions
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
existing directory would fail closed. The selected identity boundary is caller/authority assignment. Preflight therefore requires a complete
caller-supplied identity manifest covering every series, run, record, artifact, validation, and
provenance ID needed for the closed run. The harness must validate each ID's type, UUIDv4 shape,
uniqueness in its applicable domain, ownership/assignment evidence, and consistency across every
record, reference, artifact, and provenance edge. It must reject missing, duplicate, conflicting,
wrong-domain, or inconsistently bound identities. Harness-generated identities, CSPRNG use by the
harness, content-, timestamp-, counter-, or path-derived identities, and silent regeneration are
forbidden.

## 4. Candidate materialization, execution, and oracle sequence

If both blockers in section 7 are resolved without changing the authorities, a later authorization
may freeze this conditional sequence:

1. Validate the closed CLI, empty output root, cell, identities, repository state, effective
   settings, and capture capability before creating evidence.
2. Use `exp1-workload-conformance`—not copied algorithms—to materialize the corrected canonical M01
   operations, S01 stream, 3,423-byte M01 manifest, 1,274-byte stream artifact-manifest fixture,
   and independent 1,152-byte workload-manifest artifact fixture. Validate exact bytes, counts,
   identities, bindings, lengths, provenance endpoints, artifact digests, workload-stream digest,
   and manifest digest against R12/R14/R16.
3. Before any generated M01 frame or descriptive D1 execution is authorized, a later authority must
   freeze and validate the exact, deterministic mapping from each M01 `SemanticOperation` to one or
   more EXP1-B1-RF1 `Record` values. Only after that mapping passes its independent correctness gate
   may `exp1-record-format` encode and validate the resulting complete frames. No second encoder,
   CRC, or framing path is permitted.
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

**The descriptive D1 harness is blocked and is not prospectively authorized by R19.** At least two
independent decisions remain open.

### 7.1 Semantic-operation-to-physical-record mapping blocker

Current authority freezes canonical M01 semantic-operation bytes, the EXP1-B1-RF1 physical record
formats and validators, raw append/replay, and stable physical test vectors. It does **not** uniquely
freeze or implement how an M01 `SemanticOperation` becomes an EXP1-B1-RF1 `Record`. In particular,
no authority selects the record kind; body selection and field mapping; physical ordinal
assignment; provisional/final/commit lifecycle representation; semantic identity/reference
placement; logical, system, and durability time treatment; integrity profile; or encoding-failure
behavior. Stable fixture frames demonstrate the physical codec; they are not a semantic-to-physical
mapping.

`exp1-record-format` can encode and validate an already-constructed `Record`, but it cannot supply
this missing authority. A separate owner-reviewed decision must freeze the exact deterministic
mapping and its validation vectors and correctness gate before generated M01 physical frames,
workload materialization into physical records, or descriptive D1 execution can be authorized. R19
does not invent or prospectively authorize that mapping.

### 7.2 Live Linux capture implementation blocker

R7 requires direct Linux capture including `CLOCK_MONOTONIC_RAW`, `getrusage`, `statx` where
required, `perf_event_open`/tracefs integration, process/procfs facts, ordinary file metadata,
privilege and loss behavior, and explicit unavailable-field dispositions. Safe Rust and `std` can
implement portions such as procfs reads and ordinary file metadata; R19 does not claim that all
Linux capture is impossible under safe `std`. The unresolved boundary is the exact implementation
for required direct Linux interfaces that safe `std` does not expose, together with privilege/loss
behavior and unavailable-field policy. The existing external-dependency-free workspace contains
reviewed workspace path dependencies, forbids unsafe code, and exposes no reviewed implementation
of that direct-interface subset.

A separate owner-reviewed capture decision must select one authority-compliant alternative:

1. an exact reviewed external platform crate, including version, license, feature, build, and
   dependency policy;
2. tightly bounded, reviewed, platform-specific unsafe FFI;
3. another authority-compliant implementation with its complete interface and evidence policy; or
4. if R7 permits it, a formally reduced descriptive capture subset with exact unavailable-field,
   privilege, and loss dispositions.

That decision must also confirm the Fedora 44 target without making BLK-015 survival or fault
claims. Ordinary safe-`std` capture portions do not resolve the direct-interface decision.

### 7.3 Identity boundary is selected, not blocked

Section 3 selects caller/authority-supplied identities. The complete caller-supplied identity
manifest and its type, uniqueness, ownership/assignment-evidence, and cross-record consistency
validation are therefore requirements, not an open assignment decision. Harness identity
generation and CSPRNG assignment are prohibited and are not implementation blockers.

Until both blocking decisions merge, no fourth crate, binary, Cargo/workspace edit, executable R7 record
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

Continuation requires both exact section 7 blocking decisions, a new documentation/governance authorization
against then-current `main`, and review of the exact candidate boundary before any Cargo or Rust
change. Implementation completion would require exact-head CI plus review of every frozen record,
artifact, failure, dependency, and exclusion obligation. Even then, implementation would authorize
no execution: a separately reviewed descriptive execution gate must validate a real target
environment, series identity, cell, effective capture, artifact location/retention, and publication
path before one observation is run. Confirmatory work remains a separate later gate.
