# R9 — Workspace, Harness, CI, and Slice A Authorization

**Record:** `EXP-0001-R9/workspace-v1`
**Decision date:** 2026-08-28
**Authority base:** reviewed `main` at `6f7f67a` (PR #41 / R8)
**Status:** complete documentation design; authorization is prospective and effective only when this record is reviewed and merged

## 1. Bounded question and evidence class

R9 asks: **Can exactly one approved implementation slice be implemented reproducibly?** The answer is yes for the bounded Slice A plan below. This is a design and authorization decision, not implementation evidence. It creates no Cargo, Rust, fixture, executable test, CI workflow, benchmark, fault, or generated-evidence artifact.

R9 resolves BLK-020 only for the Slice A harness subset, resolves BLK-026 for that subset, and prospectively resolves BLK-027 when this change is merged. It does not make EXP-0001 executable and does not authorize measurement.

## 2. External facts and project decisions

Sources were checked on 2026-08-28. External facts justify available mechanisms; the selections in the right column are project decisions.

| Primary source and identity | External fact used | Frozen project decision |
|---|---|---|
| Rust Project, [Announcing Rust 1.89.0](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/), 2025-08-07 | Rust 1.89.0 is a released stable toolchain; the release notes identify the supported platform/toolchain changes for that release. | Pin `1.89.0`; do not track the moving `stable` channel. Use Edition 2024. |
| Rust Project, [`rust-toolchain.toml` specification](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file), checked 2026-08-28 | A checked-in toolchain file can select a channel, components, target, and minimal profile. | Put the exact toolchain declaration beside the experimental workspace and request `rustfmt`, `clippy`, and the target. |
| Rust Project, [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), checked 2026-08-28 | A virtual workspace centralizes members, resolver, profiles, and workspace package/lint settings. | Use one virtual EXP-0001 workspace with resolver `3`; admit one package for Slice A. |
| Rust Project, [Cargo.toml versus Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), checked 2026-08-28 | Applications should commit `Cargo.lock` for reproducible builds; libraries generally leave downstream selection open. | This experiment is an executable research workspace, so commit and enforce its lockfile even though Slice A's package is a library. |
| Rust Project, [Clippy in GitHub Actions](https://doc.rust-lang.org/clippy/continuous_integration/github_actions.html), checked 2026-08-28 | Clippy supports CI invocation with `-- -D warnings`. | CI uses the exact lint command in section 7. |
| GitHub, [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use), checked 2026-08-28 | Pinning an action to a full commit SHA is the immutable-action release practice. | The permitted workflow pins `actions/checkout` v4.2.2 to `11bd71901bbe5b1630ceea73d27597364c9af683`; no floating action tag is allowed. |

R9 does not depend on undocumented behavior of a CRC crate: the direct-dependency allowlist is empty.

## 3. Slice A prerequisite audit

The gate passes because each required input is already reviewed and repository-recorded, rather than because R9 restates it.

| Slice A prerequisite | Authoritative artifact and exact contribution | Audit disposition |
|---|---|---|
| Physical invariants and fail-closed scan classes | [R1](R1-PHYSICAL-RECORD-INTEGRITY-RECOVERY-REQUIREMENTS.md), especially `PR-001`–`PR-022` | Satisfied as reviewed requirements; implementations must encode these rules without widening them. |
| Deterministic vector and independent-oracle method | [R2](R2-DETERMINISTIC-WORKLOAD-BYTES-IDENTITY-REFERENCES-DIGEST-REQUIREMENTS.md) requires frozen bytes, intermediate values, negative cases, and independent reproduction | Satisfied for physical records by the later R5/R7 vectors; unresolved workload generation BLK-006/007 is outside Slice A. |
| Identity, reference, sequence, time, and retry lifecycle | [R3](R3-IDENTITY-TIME-SEQUENCING-RETRY-LIFECYCLE.md) freezes typed UUIDv4 roles, signed epoch-nanosecond times, assignment authority, monotonic sequence, gaps, and retry/uncertain-outcome rules | Satisfied for validation of supplied record values. Slice A does not generate workload identities or read clocks. |
| Versioned physical framing (BLK-001) | [R5 focused contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) freezes `EXP1-B1-RF1`, field offsets/widths/endian rules, length bounds, record kinds, flags, and scanner behavior | Satisfied; BLK-001 was resolved as documentation design by R5. |
| Versioned integrity profile (BLK-003) | The same [R5 focused contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) freezes CRC-32C/Castagnoli parameters, coverage, little-endian storage, limits, and validation order | Satisfied; every BLK-003 portion applicable to Slice A was resolved by R5. No alternative CRC profile is permitted. |
| Stable physical-record vectors | [R5 physical contract](R5-PHYSICAL-RECORD-INTEGRITY-AND-RECOVERABLE-COMMIT-CONTRACT.md) freezes profile check values and record vectors; [R7 physical-record examples](R7-PHYSICAL-RECORD-EXAMPLES.md) freezes complete valid, malformed, truncated, corrupt, lifecycle, and SHA-256 documentation examples | Satisfied. These reviewed literal bytes/values are the golden inputs and independent oracle outputs; implementation may copy them into fixture files but may not regenerate the expected side with the code under test. |
| One-way harness/provenance boundary | [R7 authority](R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md) freezes contracts → producers/adapters → capture → immutable artifacts → validators → analysis and prohibits reverse imports | Satisfied for the Slice A subset defined below. Capture, adapters, analysis, and benchmark records remain absent. |
| No unresolved ambiguity invalidates the slice | Open BLK-006–009, BLK-015, UNK-013, and workload-manifest questions govern generation, causal locality, persistence, execution, or evidence capture, not decoding/validating explicitly supplied `EXP1-B1-RF1` records | Explicitly dispositioned outside Slice A. A newly discovered ambiguity in framing, CRC, field semantics, or expected vectors fails closed and suspends authorization. |

## 4. Frozen workspace and component boundaries

After this record is merged, Slice A may create only this shape (names are normative):

```text
experiments/exp-0001/
├── Cargo.toml                 # virtual workspace; sole member below
├── Cargo.lock
├── rust-toolchain.toml
└── crates/
    └── exp1-record-format/
        ├── Cargo.toml
        ├── src/lib.rs
        └── tests/
            ├── golden_vectors.rs
            ├── round_trip.rs
            ├── rejection.rs
            ├── semantic_validation.rs
            └── data/          # literal reviewed R5/R7 inputs and oracle outputs
.github/workflows/exp0001-slice-a.yml
```

No other package, binary, example, benchmark target, build script, generated source, root Cargo file, `/crates/` production file, or evidence/results directory is authorized. `exp1-record-format` is one experiment-local library containing pure byte encoding, decoding, CRC-32C calculation, structural validation, and identity/reference/order validation. Its public surface is experimental and has no stability promise.

The independent oracle is the reviewed R5/R7 documentation output, transcribed literally into `tests/data/` with source record/section and expected byte/digest values recorded in a fixture manifest. The package-under-test must not produce expected values during a test. Tests may depend on the library; the library depends only on `core`/`std`. This preserves R7's one-way provenance: authority-derived immutable inputs → record-format implementation → validator verdicts. There is no dependency on future workload producers, adapters, capture, artifact validators, analysis, or benchmark components, and none of those may import this package under Slice A authorization.

## 5. Toolchain, target, build, and dependency freeze

- **Toolchain:** exact Rust `1.89.0`, profile `minimal`, with `rustfmt` and `clippy`. `rust-toolchain.toml` must use the literal channel `1.89.0`; moving channels and version ranges are prohibited.
- **Target:** only `x86_64-unknown-linux-gnu`, matching the R4 Fedora 44 x86-64 first target. Cross-compilation and platform claims are prohibited.
- **Workspace:** Edition 2024, resolver `3`, `rust-version = "1.89"`, one member, and explicit `default-members` containing that member. Workspace/package metadata must mark the experiment unpublished.
- **Dependencies:** no direct normal, dev, build, target-specific, git, or path dependency other than the sole workspace member itself. Standard-library facilities suffice for byte operations, UUID-shaped fixed-width values, files, and tests; the small frozen CRC-32C algorithm must be implemented transparently from the R5 parameters. This avoids supply-chain/version/feature ambiguity and permits genuinely offline locked validation.
- **Features:** no package features, optional dependencies, conditional compilation, or non-default feature sets. Cargo's implicit empty default feature set is the only configuration.
- **Lockfile:** commit `experiments/exp-0001/Cargo.lock`; every Cargo CI command uses `--locked`, and validation after initial bootstrap uses `--offline`. The lockfile may change only with a reviewed manifest/toolchain change.
- **Profiles:** set `overflow-checks = true` in `dev`, `test`, and `release`; set `debug-assertions = true` for `test`. Slice A CI uses the test profile only. Release execution and benchmarking remain prohibited.
- **Determinism flags:** CI exports `CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`, `RUST_BACKTRACE=1`, and `RUSTFLAGS=-Dwarnings`. Tests must not consult wall clocks, randomness, locale, host paths, environment-derived expected values, network, thread scheduling, or filesystem ordering. Integer parsing/arithmetic must be checked; host-native endian/width behavior may not enter the format.
- **Unsafe/platform policy:** workspace lint configuration forbids `unsafe_code` and denies unexpected `cfg` names. No `unsafe`, FFI, OS API, architecture intrinsic, build script, or platform-dependent fast path is allowed. Any need for one suspends Slice A and requires a new reviewed authorization.
- **Change control:** a dependency addition, dependency/version source change, toolchain/component/target/edition/resolver/profile/flag change, new crate, feature, build script, or CI action change revisits BLK-020/026/027 and requires a prospective R9 supersession. Dependabot-style automatic merge is not permitted for this workspace.

## 6. Correctness and test freeze

Tests are integration tests organized exactly by the files in section 4; focused private unit tests may additionally live beside implementation functions. Test names must identify the R5/R7 vector or requirement they exercise.

- **Golden vectors:** byte-for-byte encode and decode checks for every applicable R5/R7 stable valid record, including CRC check values and complete record bytes. Expected output is literal independent-oracle data with provenance, never recomputed through the subject.
- **Round trip:** for the finite reviewed corpus, require `decode(encode(value)) == value` and `encode(decode(bytes)) == bytes`; round trip never substitutes for golden tests.
- **Rejection:** cover every documented malformed kind/flag/reserved value/length, undersized header, all truncation boundaries represented by the authority, trailing/terminal damage class, one-bit corruption cases for header/payload/CRC, wrong CRC byte order/profile, arithmetic overflow, and oversize input. Validation fails closed without panics or partial acceptance.
- **Semantic validation:** enforce UUID variant/version and nil restrictions by field role; required/forbidden originating-request, causal, correction/retraction, final/commit references; sequence nonzero/strict ordering; final/commit identity and length binding; and the exact R3/R5 lifecycle ordering that is decidable from supplied records. Causal-reference locality remains out of scope and must not be invented.
- **Property tests:** no property-test framework or randomized/fuzz test is authorized because the dependency allowlist is empty and deterministic boundary enumeration plus reviewed vectors can falsify this slice's bounded claims. Exhaustive table-driven tests must enumerate record kinds, allowed flags, field boundaries, and each single-bit corruption position for each short golden record. A later property/fuzz proposal requires separate review and may supplement, never replace, golden or malformed tests.
- **Evidence expectation:** CI logs are validation status only, not EXP-0001 evidence. The committed fixture manifest records documentation provenance and expected values; CI uploads no generated fixture, benchmark, result, or evidence artifact.

## 7. Formatting, lint, CI, and exact commands

The authorized workflow has least-privilege `contents: read`, no secrets, no write permissions, no cache, and one `ubuntu-24.04` job. It installs the exact toolchain described by the checked-in file with rustup, adds the exact target/components, and uses only the full-SHA-pinned checkout action. GitHub's runner image is a validation environment, not the R4 benchmark target and supports no platform or performance claim.

The job runs these commands from the repository root in this order:

```sh
git diff --check
python3 tools/validate_markdown_links.py
cargo fmt --manifest-path experiments/exp-0001/Cargo.toml --all -- --check
cargo clippy --manifest-path experiments/exp-0001/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
cargo test --manifest-path experiments/exp-0001/Cargo.toml --workspace --all-targets --locked --offline
```

Formatting is mandatory. Clippy and rustc warnings are errors. The test command covers unit, golden/vector, round-trip, malformed/corrupt, and semantic-validation suites; no doc tests containing executable examples should be relied upon as a gate. CI must show the exact toolchain (`rustc -Vv`, `cargo -V`, `rustfmt -V`, `cargo clippy -V`) before validation and retain ordinary GitHub job logs. No binary, coverage, cache, benchmark, fixture-generation, or results artifact is uploaded.

## 8. First-implementation gate and authorization record

**Authorized first slice: Slice A — deterministic physical-record fixtures and validators**

This is prospective approval by repository governance. Approval identity is `EXP-0001-R9/workspace-v1`; decision date is 2026-08-28; addressed blockers are BLK-020, BLK-026, and BLK-027; base state is `6f7f67a`. The required reviewer is the human reviewer(s) who approve this change, and the immutable approval state is its merge commit on `main`. **The authorization is ineffective on this branch and becomes effective only after this R9 change is reviewed and merged.**

After that merge, Slice A implementation may create only the Cargo/workspace/Rust/test/fixture-manifest/data/CI artifacts and behavior expressly permitted in sections 4–7. It may transcribe reviewed oracle vectors, implement deterministic record encode/decode/CRC/validation, and add the frozen correctness tests. A review finding that any prerequisite is incomplete, any oracle transcription disagrees, or any new semantic ambiguity exists suspends work and requires correction or a superseding decision; rollback is removal/reversion of the unmerged Slice A change, not weakening a contract.

Authorization **does not extend** to Slice B or later slices; persistence; append or synchronization; B0/B1/SQLite/RocksDB execution; benchmark execution; performance claims; fault execution; D1/D2/D3 durability claims; production code under `/crates/`; server, networking, query, or distributed functionality; semantic-only workload generation; clocks; concurrency; capture; analysis; adapters; or any other architecture expansion.

Every section 6 gate item is therefore satisfied: the slice is named and bounded; R1/R3 and versioned R5 framing/CRC contracts are linked; stable R5/R7 oracle vectors exist; code remains beneath `/experiments/`; toolchain/target/build are exact; CI commands cover format/static/unit/vector/documentation checks; the empty dependency allowlist records rationale and implications; invalidating ambiguities are dispositioned; revisit/prohibition rules are explicit; and approval identity/date/reviewer mechanism/blockers/base state are recorded.

## 9. Blocker disposition and retained work

| Item | R9 disposition |
|---|---|
| BLK-020 | Resolved as documentation design **only for the Slice A harness subset** and its one-way authority → fixture → implementation → verdict flow. The executable benchmark harness remains unauthorized and later component boundaries require later slices. |
| BLK-026 | Resolved for Slice A by the exact toolchain, target, workspace, lock, flags, offline procedure, and change rules above. Benchmark-series toolchains (including R6 native dependencies) remain open. |
| BLK-027 | Gate satisfied prospectively for the one workspace/package/workflow shape above; resolved only when this record is merged. It authorizes no other bootstrap. |

BLK-006–009, BLK-015, owner-dependent fault apparatus, executable R7 capture/records/analysis, baseline implementations/effective settings/equivalence, benchmark-series build profiles, and all execution/evidence remain deliberately open. UNK-020 is narrowed for Slice A reproducibility; UNK-022 is narrowed only by authorization to test physical-record validation. No unknown is converted into empirical evidence.

## 10. Revisit and completion statement

Revisit R9 before any listed configuration or boundary changes, before admitting any dependency or second package, upon a changed R5/R7 authority/vector, upon a toolchain target defect that affects semantics, or when a semantic ambiguity appears. Later slices require their own prospective authorization and prerequisites.

R9 is complete as a bounded documentation/readiness increment. Once merged, the next permitted increment is **implementation of Slice A only** under this record. EXP-0001 remains proposed; benchmarking, persistence, fault execution, durability claims, and architecture promotion remain prohibited.
