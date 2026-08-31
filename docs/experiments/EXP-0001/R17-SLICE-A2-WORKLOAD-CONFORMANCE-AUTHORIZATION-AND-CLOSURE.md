# R17 — Slice A2 workload-conformance authorization and closure

**Approval:** GitHub issue #63, `EXP-0001-A2/workload-conformance-v1`
**Authority base:** `27e5141d48991ecce2770bfa1720d03ac4ae67da` (PR #62 / R16)
**Status:** closed as bounded implementation/correctness-validation evidence by R18 after exact-head review and CI

## Boundary and result

Issue #63 prospectively authorized one bounded Slice A2 implementation. The resulting
`exp1-workload-conformance` experiment crate is the second and only additional EXP-0001 workspace
member. It uses Rust 1.89.0, Edition 2024, resolver 3, the existing target, profiles, lint policy,
and unchanged CI. It has no dependencies, features, build script, binary, examples, benchmarks,
unsafe code, FFI, networking, clock, randomness, threads, OS access, or production surface.
The exact reviewed PR #64 implementation head is
`d2ee72aa4ff047d4cfcaa1df82d83f13566568f2`; merge commit
`9b5d89a36ed71d38420e9ae19f59d441a9d927aa` contains it on `main`. Both required exact-head
workflows passed. R18 records the authoritative post-merge closure.

The crate makes R12 payload, typed identity, scalar, and logical-time generation executable;
validates and constructs R14 semantic-operation and ordered stream bytes and SHA-256 bindings;
and validates the bounded R16 canonical-manifest byte, stream, digest, reference, and immutable
supersession boundary. Literal R12 P10–P12/I01/T01–T05, R14 S01/S02/W00/W01, and R16 M01
expected values are authority-derived constants independent of code under test. Deterministic
negative tables cover malformed/truncated streams, scalar rejection, canonical-byte rejection,
and supersession conflicts without randomized tooling.

Because the corrected exact head passed review and CI, BLK-006, BLK-007, BLK-008, and BLK-009 are implemented only for this experiment-local
conformance subset; their original documentation-design resolutions remain distinct from this
correctness evidence. BLK-020, BLK-026, and BLK-027 extend only to this dependency-free A2 member
under the existing harness and CI. The closure gate confirmed that the reviewed contracts' covered vectors
are executable and mutually consistent. They are not generated workload, execution, benchmark,
persistence, durability, storage, or performance evidence.

## Validation and retained gates

The closure gate is the unchanged R9 sequence: link validation, formatting, locked/offline clippy,
locked/offline workspace tests, and `git diff --check`, plus exact-head **Documentation validation**
and **EXP-0001 Slice A** GitHub checks. Both exact-head workflows passed; R18 binds that result to the reviewed and merge commits above.

BLK-015, harness/capture for execution, descriptive execution, confirmatory execution,
faults, D2/D3 durability, adapters, production crates, and architecture promotion remain blocked
and unauthorized. R18 separately authorizes only its bounded, non-durable Slice C/B1 correctness
implementation; that later authority does not broaden A2 evidence. No workload or benchmark was run. The recommended next
substantial tranche is a separately reviewed, smallest-useful readiness increment addressing an
open gate; R17 does not automatically authorize storage, harness implementation, or execution.
