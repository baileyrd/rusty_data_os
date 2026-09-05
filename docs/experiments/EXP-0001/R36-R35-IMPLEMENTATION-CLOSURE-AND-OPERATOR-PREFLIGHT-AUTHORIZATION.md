# R36 — R35 Implementation Closure and Operator Preflight Authorization

**Status:** Complete implementation closure; exactly one immediate operator-runner implementation PR is authorized
**Scope:** EXP-0001 v2 target-preflight closure and minimal operator-only example boundary
**Evidence classification:** bounded deterministic v2 construction, serialization, lifecycle, failure-ordering, retention, and synthetic correctness evidence only; prospective runner governance design
**Authority date:** 2026-09-05

## 1. Exact implementation closure

PR #119 merged reviewed head `44a61290db7b9929502e630608c0fea860105e96` as merge commit
`dd9f8ae6c81681ea5943058b63f5e74117b234c8`. Both workflows succeeded for that exact reviewed
head. The merged R35 v2 target-preflight implementation and its exhaustive synthetic tests are
therefore closed as bounded deterministic evidence for construction, serialization, lifecycle,
failure ordering, retention, and synthetic correctness only.

This closes and consumes the single implementation authorization granted by R35. It does not
establish any live host observation, Fedora validation, benchmark or workload execution, or
performance evidence. In particular, successful compilation and synthetic testing do not prove
the effective host or the Fedora release.

## 2. Exactly one immediate runner authorization

This authority exists to unlock executable progress and the first controlled real preflight, not
to begin another planning chain. Exactly one immediate code PR is authorized. It may add only:

```text
experiments/exp-0001/crates/exp1-descriptive-d1-harness/examples/target_preflight.rs
```

No other file may change in that implementation PR. In particular, Rust library sources, Cargo
manifests, the lockfile, workflows, fixtures, dependencies, and toolchain files remain unchanged.
The example must use only the standard library and require no manifest or dependency change.

## 3. Frozen command and dispatch contract

The executable command is exactly:

```text
target_preflight <repository-revision> <build-identity> <measured-file-identity> <measured-file-path>
```

The example must:

1. call the existing `run_target_preflight` rather than duplicate or bypass its behavior;
2. hardcode the frozen expected values `Fedora44Linux` and `X86_64`;
3. use locked stdout as the retention sink;
4. emit to stdout only the exact JSON-lines artifact emitted by the existing retention boundary;
5. emit only fixed diagnostics to stderr;
6. never echo, serialize, log, diagnose, or otherwise retain the measured-file path;
7. perform no file mutation, workload action, record production, or additional host probing; and
8. introduce no non-standard-library use, manifest change, or dependency change.

The runner may translate arguments into the existing request and translate the returned disposition
to an exit status. It must not enrich, parse, reserialize, or otherwise alter the retained artifact.

## 4. Frozen exit behavior

The process exits with exactly these meanings:

| Exit status | Meaning |
|---:|---|
| `0` | A retained `preflight_subset_ready` artifact. |
| `2` | A retained `blocked` or `invalid` artifact. |
| `64` | Invalid CLI arguments or request; no live dispatch is permitted. |
| `70` | Serialization or retention failure. |

No other semantic exit classification is authorized. Fixed stderr diagnostics may distinguish only
these operator-relevant outcomes without containing supplied argument values or host observations.

## 5. Synthetic test and CI gate

The runner PR must provide synthetic coverage of argument handling, dispatch selection, and every
exit-status mapping. Tests must exercise an extracted pure or injected dispatch boundary and must
not invoke `run_target_preflight`'s live boundary in tests or CI. `cargo --all-targets` may compile
the example, but neither that command nor any CI step may run it. The unchanged R9 validation
sequence and `git diff --check` remain mandatory.

## 6. First controlled operator invocation

Only after the authorized runner PR is reviewed and merged, one manual operator invocation is
permitted on the intended Linux/x86_64 experiment target using a pre-existing disposable measured
file. The operator must supply the four frozen arguments, must not permit the runner to create or
modify the measured file, and must retain stdout according to the existing v2 boundary.

That single first invocation is a staged diagnostic. It is **not** an R7 environment record, a
benchmark result, Fedora-release proof, effective-host validation, workload execution, or
performance evidence. Any interpretation or subsequent invocation requires later authority.

## 7. Preserved exclusions

Except for the one post-merge manual invocation above, every live workload or benchmark action,
record production, tracefs use, capture publication, durability or recovery claim, `fsync`, D2/D3,
fault action, baseline or adapter expansion, production work, and performance claim remains blocked
or unauthorized. The runner cannot materialize a workload, append a record, execute a measured
action, validate Fedora, or promote any experimental component. All R33–R35 fail-closed,
path-non-retention, and evidence-classification boundaries remain controlling.
