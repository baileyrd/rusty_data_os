# Incremental lifecycle-validation comparison

**Classification:** owner-authorized bounded exploratory implementation and elapsed-time observation; not confirmatory EXP-0001 evidence or an architecture decision.

## Authorization and isolated change

The owner explicitly authorized this one implementation-and-execution increment despite the older workload prohibitions. The authorization covers only replacing physical replay's repeated whole-prefix lifecycle validation with one scan-local incremental state and running the unchanged history-views workload before and after. Every broader restriction remains in force.

The before implementation called `validate_lifecycle` after appending every decoded candidate, causing 50,005,000 lifecycle transition visits for 10,000 provisional records (the sum 1 through 10,000). That count is derived from control flow, not measured CPU attribution. The after implementation applies the shared lifecycle transition once per decoded candidate, before publication to the accepted prefix. A deterministic test observes exactly 32 transition attempts for 32 valid candidates; another proves a failing candidate remains excluded.

## Controlled method and provenance

The run order was before, implementation plus correctness validation, then after. Both executions used the exact command below with distinct, previously nonexistent directories on the same overlay filesystem:

```text
cargo +1.89.0 run --release --manifest-path experiments/exp-0001/Cargo.toml --locked --offline --package exp1-descriptive-d1-harness --example history_views -- NEW_OUTPUT_DIRECTORY
```

Both binaries report repository base revision `1820861f9805561a21840899a4270a8a029d24b1`. The before metadata records only its newly created result directory and Cargo target as untracked. The after metadata records the record-format implementation/test edits and both result directories as dirty. No exact dirty-diff digest was captured at execution time, so none is invented here; the committed implementation is the retained after source. The runtime-observed toolchain was Rust 1.89.0 (`rustc` commit `29483883eed69d5fb4db01964cdf2af4d86e9cb2`, LLVM 20.1.7), release optimized with `debug_assertions=false`, Linux x86_64, kernel 6.18.35, Intel Xeon Platinum 8272CL, and overlay filesystem. Embedded metadata truthfully leaves Cargo-unembedded facts unknown while the invocation records `--release --locked --offline`.

The workload remained deterministic and single-threaded: 17 entities; 100, 1,000, and 10,000 events; one warm-up then five measured trials per size and version. All 30 measured trials and six warm-ups passed the unchanged midpoint/final independent-oracle and physical-replay gates. Failure files contain headers and no failures. All 18 corresponding before/after RF1 artifacts are byte-identical; their independently computed SHA-256 values and lengths are retained in [`results/container-20260908-incremental-validation-artifact-sha256.txt`](results/container-20260908-incremental-validation-artifact-sha256.txt).

## Replay observation

Values are medians with measured min–max in milliseconds. Throughput is event count divided by median elapsed time. Ratio is before median / after median, so values above 1 indicate lower observed after time.

| Events | Before replay ms | After replay ms | Before throughput | After throughput | Before/after ratio |
|---:|---:|---:|---:|---:|---:|
| 100 | 0.119032 (0.103202–0.145677) | 0.062490 (0.045804–0.107879) | 840K events/s | 1.60M events/s | 1.90x |
| 1,000 | 5.584829 (5.545372–6.084391) | 0.379115 (0.160368–0.535545) | 179K events/s | 2.64M events/s | 14.73x |
| 10,000 | 542.289996 (526.597487–567.061188) | 2.502494 (2.275321–17.027527) | 18.4K events/s | 4.00M events/s | 216.70x |

The controlled observation establishes a large elapsed replay effect correlated with removal of repeated prefix validation, most pronounced at 10,000 records. It does not attribute all replay CPU time to lifecycle validation, establish asymptotic complexity for mixed record types, or show stable tail behavior. In particular, identity/reservation/final/commit collections retain linear searches, and the after 10,000-event replay range includes one 17.0 ms observation.

## Other component observations

Each cell is before median (min–max) → after median (min–max), in milliseconds. These untargeted components were retained rather than interpreted as regressions or improvements.

| Events | Encode | Append | Row rebuild | Column rebuild | Direct row | Direct column |
|---:|---:|---:|---:|---:|---:|---:|
| 100 | 0.035092 (0.033935–0.075136) → 0.034081 (0.033411–0.035274) | 0.111780 (0.085119–0.123877) → 0.094778 (0.089278–0.112006) | 0.003960 (0.003550–0.056289) → 0.003823 (0.003445–0.005091) | 0.006302 (0.006203–0.010537) → 0.006310 (0.005963–0.008160) | 0.001829 (0.001758–0.003378) → 0.001768 (0.001698–0.004312) | 0.003569 (0.003463–0.005101) → 0.003542 (0.003421–0.003725) |
| 1,000 | 0.317105 (0.315609–0.413441) → 0.328249 (0.312251–0.366653) | 0.731644 (0.688199–0.778549) → 0.702609 (0.662154–0.740019) | 0.026474 (0.026059–0.028672) → 0.026330 (0.025742–0.026609) | 0.027803 (0.027672–0.029205) → 0.027415 (0.027193–0.029536) | 0.011718 (0.011604–0.011806) → 0.011726 (0.011477–0.011820) | 0.012746 (0.012631–0.012938) → 0.012467 (0.011994–0.012838) |
| 10,000 | 3.480870 (3.210914–4.793742) → 3.444466 (3.320246–3.548878) | 6.701669 (6.463946–9.045934) → 7.720862 (6.595909–8.405969) | 0.334334 (0.311209–0.426990) → 0.302988 (0.280100–1.008672) | 0.273553 (0.261550–0.321716) → 0.276417 (0.237536–0.343186) | 0.142274 (0.109146–0.152376) → 0.111800 (0.109168–0.166968) | 0.103000 (0.100120–0.247016) → 0.101190 (0.100529–0.153008) |

## Retained evidence and limits

Raw environment, warm-up, measured-trial, failure, summary, and RF1 records are retained separately in [`results/container-20260908-incremental-validation-before/`](results/container-20260908-incremental-validation-before/) and [`results/container-20260908-incremental-validation-after/`](results/container-20260908-incremental-validation-after/). Cache state, co-tenancy, CPU frequency, storage hardware, mount details, and power mode were uncontrolled or unavailable. Five samples support descriptive medians and ranges, not tail claims. Appends are D1 ordinary writes without `fsync`; this is not stable-storage, crash-survival, recovery, canonical-commit, target-machine, competitive-database, or general HYP-0001 evidence.
