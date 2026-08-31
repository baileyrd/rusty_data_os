# R17 — Slice A2 workload-conformance authorization and closure

**Approval:** GitHub issue #63, `EXP-0001-A2/workload-conformance-v1`
**Authority base:** `27e5141d48991ecce2770bfa1720d03ac4ae67da` (PR #62 / R16)
**Status:** prospective implementation/correctness-validation closure, contingent on corrected exact-head PR review and CI

## Boundary and result

Issue #63 prospectively authorized one bounded Slice A2 implementation. The resulting
`exp1-workload-conformance` experiment crate is the second and only additional EXP-0001 workspace
member. It uses Rust 1.89.0, Edition 2024, resolver 3, the existing target, profiles, lint policy,
and unchanged CI. It has no dependencies, features, build script, binary, examples, benchmarks,
unsafe code, FFI, networking, clock, randomness, threads, OS access, or production surface.
The exact reviewed implementation head is the pull-request head recorded by GitHub after this
commit; embedding that content-addressed commit in its own contents is impossible, so exact-head
review and both required checks remain the authoritative head binding.

The crate makes R12 payload, typed identity, scalar, and logical-time generation executable;
validates and constructs R14 semantic-operation and ordered stream bytes and SHA-256 bindings;
and validates the bounded R16 canonical-manifest byte, stream, digest, reference, and immutable
supersession boundary. Literal R12 P10–P12/I01/T01–T05, R14 S01/S02/W00/W01, and R16 M01
expected values are authority-derived constants independent of code under test. Deterministic
negative tables cover malformed/truncated streams, scalar rejection, canonical-byte rejection,
and supersession conflicts without randomized tooling.

After the corrected exact head passes review and CI, BLK-006, BLK-007, BLK-008, and BLK-009 are implemented only for this experiment-local
conformance subset; their original documentation-design resolutions remain distinct from this
correctness evidence. BLK-020, BLK-026, and BLK-027 extend only to this dependency-free A2 member
under the existing harness and CI. The closure gate must confirm that the reviewed contracts' covered vectors
are executable and mutually consistent. They are not generated workload, execution, benchmark,
persistence, durability, storage, or performance evidence.

## Validation and retained gates

The closure gate is the unchanged R9 sequence: link validation, formatting, locked/offline clippy,
locked/offline workspace tests, and `git diff --check`, plus exact-head **Documentation validation**
and **EXP-0001 Slice A** GitHub checks. The PR completion report records their exact results.

BLK-015, harness/capture for execution, Slice C/B1, descriptive execution, confirmatory execution,
faults, persistence, recovery, durability, adapters, production crates, and architecture promotion
remain blocked and unauthorized. No workload or benchmark was run. The recommended next
substantial tranche is a separately reviewed, smallest-useful readiness increment addressing an
open gate; R17 does not automatically authorize storage, harness implementation, or execution.

## Owner-selected M01 correction

The reopened A2 review selected option 1: M01 remains canonical and positively valid. This corrective tranche replaces its synthetic R7 reference with the literal 1131-byte closed R7 record and validates the actual R7 envelope, artifact-manifest body, full artifact entries, and authoritative provenance field names. Corrected tests require full M01 success and retain the documentation-design versus bounded implementation/correctness-evidence distinction. Closure remains contingent on corrected exact-head review and CI; all later-work exclusions remain unchanged.
