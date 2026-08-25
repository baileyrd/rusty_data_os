# AGENTS.md — Rusty Data OS Working Agreement

This file defines how human contributors and AI coding agents must work in this repository.

## 1. Authority order

Before proposing or implementing work, read these files in order:

1. `/AGENTS.md`
2. `/CHATGPT_WORKFLOW.md`
3. `/docs/PROJECT-STATUS.md`
4. `/docs/VISION.md`
5. `/docs/PRINCIPLES.md`
6. `/docs/ARCHITECTURE.md`
7. `/docs/RESEARCH-ROADMAP.md`
8. Applicable hypotheses, experiment definitions, benchmark methodology, ADRs, and specifications.

Repository state on `main` is authoritative over chat history or unstaged ideas.

## 2. Research before architecture

Do not promote an architectural idea because it sounds elegant or familiar.

Every significant technical claim should move through:

```text
idea -> hypothesis -> experiment -> evidence -> decision
```

A hypothesis must be falsifiable. An experiment must state its measurement method before implementation. A decision must cite the evidence that justifies it.

## 3. No premature production code

Experimental implementations belong under `/experiments/`.

Reusable production-oriented code belongs under `/crates/` only after the relevant experiment has produced evidence and an ADR or specification has authorized graduation.

Do not add server/networking layers, SQL interfaces, generalized plugin systems, distributed coordination, or unrelated abstractions unless the research roadmap and current experiment require them.

## 4. Benchmark integrity

Performance claims must follow `/docs/benchmarks/METHODOLOGY.md`.

At minimum:

- capture hardware, OS, filesystem, compiler, toolchain, and build configuration;
- distinguish warm-up from measured samples;
- report distributions, not only averages;
- separate throughput from latency;
- identify durability semantics for every write benchmark;
- preserve raw benchmark results when practical;
- compare against explicit baselines;
- avoid changing multiple independent variables in the same experiment unless the design requires it.

## 5. Correctness precedes speed

A faster implementation that violates declared durability, ordering, replay, or recovery semantics is not a successful optimization.

Each experiment must define correctness invariants and tests independently of performance targets.

## 6. Failed experiments are valuable

Do not delete or obscure negative results. Mark the experiment conclusion clearly and preserve enough evidence to explain why an approach was rejected, deferred, or constrained.

## 7. Documentation synchronization

When an experiment changes project knowledge, update all relevant artifacts in the same change:

- experiment result/conclusion;
- associated hypothesis status;
- ADR if a decision is made;
- `PROJECT-STATUS.md`;
- architecture/specification only if the evidence warrants promotion.

## 8. Smallest useful increment

Prefer bounded experiments that isolate one research question. Avoid "build the database" tasks. The smallest experiment that can falsify a hypothesis is preferred.

## 9. Rust implementation expectations

When Rust code begins:

- favor explicit ownership and concurrency semantics;
- make unsafe code exceptional and justified;
- benchmark release builds;
- keep experimental dependencies isolated;
- document platform-specific behavior;
- prefer deterministic replay and testability over hidden background behavior;
- expose durability boundaries explicitly in APIs rather than implying them.

## 10. Current constraint

The project is currently documentation-first. No engine architecture is considered final. Documentation and measurement prerequisites, followed by `EXP-0000` (Experiment 0), must be completed before `EXP-0001` is ready to implement. Do not create Cargo files or prescribe Cargo validation commands until a Cargo project exists. During this documentation-only phase, run `git diff --check` from the repository root.
