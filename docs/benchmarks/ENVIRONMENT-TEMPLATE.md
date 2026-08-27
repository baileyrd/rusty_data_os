# Benchmark Environment Record Contract

**Logical schema:** `benchmark-environment/v1`
**Status:** EXP-0000 measurement contract; no environment has been selected or recorded

**EXP-0001 physical profile:** The fields below are serialized, validated,
retained, and redacted according to the documentation-only
[R7 authority](../experiments/EXP-0001/R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md).
This remains the logical contract; R7 creates no schema, validator, record, or evidence.

## 1. Conformance

This versioned logical record identifies the execution context shared by a benchmark series. It is not evidence that a run occurred. **R** means required, **C** conditionally required when applicable, **M** recommended when obtainable and material, and **N** inapplicable only with a reason. `unknown`, `unavailable`, `unsupported`, and `inapplicable` each require a reason and are distinct from zero. Values must never be invented.

Serialization, timestamp representation, identity/digest algorithms, canonical field names, and validation tooling remain unresolved. Values and units must nevertheless be machine-processable in the eventual representation. Sensitive user, hostname, address, and path data must be excluded or consistently pseudonymized while preserving material topology and configuration; redactions and their reproducibility impact are recorded.

## 2. Normative fields

### Identity and provenance

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `schema.name`, `schema.version` | Logical schema name and version. |
| R | `environment_id` | Immutable complete-record identity; algorithm declared by the record. |
| R | `captured_at`, `capture_mechanism` | Capture time/representation and manual or tool name/version. |
| R | `repository.commit`, `repository.dirty_state` | Full commit and `clean`, `dirty`, or reasoned unknown; dirty state references patch/status artifacts. |
| C | `repository.submodules` | Exact identities when used. |
| R | `record_producer` | Pseudonymous operator/automation role and tool version. |
| R | `artifact_manifest` | Immutable URI, media type, byte size, digest algorithm/value, producer and role for configurations, capture output, patches, lockfiles, and supporting artifacts. |
| R | `redactions` | Field, replacement policy, reason, and reproducibility impact; empty if none. |

### Host, CPU, and memory

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `host.label`, `host.execution_form` | Stable anonymized label; bare metal, VM, container, or nested stack. |
| C | `host.virtualization` | Hypervisor/runtime versions, guest type, vCPU/pinning, limits, sharing/overcommit, passthrough, and nested layers when not bare metal. |
| R | `cpu.model`, `cpu.architecture`, `cpu.microcode` | Vendor/model, architecture/features, and microcode revision or explicit unavailable reason. |
| R | `cpu.topology` | Sockets/packages/dies, NUMA nodes, physical cores, threads, SMT and benchmark-visible CPUs. |
| R | `cpu.frequency_policy`, `cpu.boost` | Scaling driver/governor, min/max policy, fixed settings, turbo/boost state. |
| R | `cpu.affinity`, `cpu.isolation` | Process/thread placement and isolated/reserved CPU controls; explicitly none when absent. |
| R | `numa.policy` | CPU/memory binding or interleave, node capacities and distances where available. |
| R | `memory.capacity_bytes`, `memory.topology` | Host/visible bytes, NUMA and channel/DIMM population where available, and effective limits. |
| M | `memory.speed` | Reported rate and source; never inferred. |
| R | `memory.huge_pages`, `memory.swap` | Page sizes/reservation/THP policy; swap devices, capacity, policy and observed-use state. |
| R | `memory.limits` | Address-space, locked-memory, cgroup/job and overcommit limits. |

### Operating system and execution policy

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `os.name`, `os.distribution`, `os.version` | OS and distribution/image release. |
| R | `os.kernel` | Kernel release/build/architecture and custom artifact reference. |
| R | `os.boot_parameters`, `os.security_mitigations` | Effective performance-relevant boot arguments and mitigations. |
| R | `scheduler.policy`, `limits` | Classes, priorities, tunables and effective process/file/thread/memory/I/O/container limits. |
| R | `power.policy` | Firmware/OS plan, CPU governor and idle-state policy. |
| R | `thermal.state` | Start/end or observed temperature/throttling where available, cooling and anomalies. |
| R | `background_activity` | Material services, daemons, co-tenants, scheduled workloads and isolation controls. |

### Build and measured software

| Level | Logical field | Meaning / condition |
|---|---|---|
| C | `rust.toolchain` | `rustc`, Cargo, channel/file, host, target and components when Rust is measured. |
| R | `compiler` | Compiler family/version and complete effective flags for measured components. |
| R | `build.profile`, `build.target`, `build.linker` | Optimization/LTO/codegen/assertion settings, target/features, and linker/version/flags. |
| R | `build.identity`, `dependencies` | Reproducible command/manifest, executable/library digests, lockfile/source and native dependency identities/options. |
| C | `baseline.identity` | Product, release/tag/commit and digest, build, executable/library, binding and adapter identities when a baseline is used. |
| C | `baseline.effective_configuration` | Requested and queried/effective configuration when a baseline is used. |

### Filesystem, storage, and placement

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `data_location` | Pseudonymized data/log/WAL/temp locations, device relationships and material permissions. |
| R | `storage.stack_path` | Ordered application buffering, runtime/VFS, filesystem, volume/RAID, block device, controller/cache and physical/virtual medium. |
| R | `filesystem` | Type/version, creation features, effective mount options, discard/allocation behavior and initial capacity use. |
| R | `storage.device` | Model, firmware, interface/topology, capacity, medium/virtual status and anonymized stable identity. |
| C | `storage.controller` | HBA/RAID/controller model, firmware, topology, mode and cache/battery state when present. |
| R | `storage.block_sizes_bytes` | Logical/physical sectors and filesystem allocation/I/O sizes where available. |
| R | `storage.queue` | Scheduler, queue count/depth, merge and read-ahead settings. |
| R | `storage.cache` | OS/device/controller cache and write-cache modes, barriers/flush behavior and volatile layers. |
| R | `storage.power_loss_protection` | Declared protection and evidence source, or reasoned unknown; never inferred from successful sync. |
| C | `network_storage` | Protocol, client/server versions/configuration, path and failure domains for remote storage. |

### Measurement, semantics, and preparation

| Level | Logical field | Meaning / condition |
|---|---|---|
| R | `clocks` | Wall/monotonic/CPU/timer sources used, resolution/precision, synchronization, conversion and placement; none is selected here. |
| R | `instrumentation` | Profiler/counter/tracer/monitor identities, versions/configuration, sampling, privileges, enabled state and overhead status. |
| R | `durability_contract_ref` | Immutable platform contract, acknowledgement boundary, promised fault classes and synchronization path. |
| C | `fault_apparatus` | Injector, trigger/control host, power/crash mechanism, recovery apparatus, versions and topology for fault/recovery work. |
| R | `preconditioning`, `cache_initial_state` | Storage/database preparation and application/OS/device cache state and procedure. |
| R | `subject_initial_state`, `cleanup_reuse_policy` | Database/log/WAL/checkpoint/file state, then deletion/reset/rotation/reuse between repetitions. |
| R | `configuration_refs` | Digested workload, subject/profile, adapter, platform-contract, instrumentation and orchestration artifacts. |

## 3. Identity and series freeze rules

A correction never mutates a published environment record: it creates a new identity with `supersedes` and a reason. Any changed value, artifact, resolved unknown, or recapture creates a new environment identity. Records may share an equivalence label only after documented review.

A new **benchmark series** is mandatory when a change can affect semantics, equivalence, execution, correctness, timing, resources, or interpretation. This includes code/dirty state, executable/dependency, baseline/binding/adapter, effective configuration, workload, D-mode/platform contract, VM/container allocation, CPU/NUMA/affinity/frequency, memory limits, OS/kernel/mitigations, filesystem/mount/storage/cache/firmware, build, instrumentation, preparation/cache/initial state, or fault apparatus. Administrative labels or safe redaction may retain a series only when technical values and digests are identical and the rationale is recorded. Ambiguity starts a new series; distinct series are never silently pooled.

## 4. Non-normative illustration (not a measurement)

```yaml
schema: {name: benchmark-environment, version: 1}
environment_id: "illustrative-id; algorithm-unresolved"
captured_at: {value: "illustrative timestamp", representation: unresolved}
repository: {commit: "<full-sha>", dirty_state: clean}
host: {label: host-a, execution_form: bare-metal}
storage:
  power_loss_protection: {state: unknown, reason: "evidence not collected"}
durability_contract_ref: {uri: "artifact:contract", digest: "<algorithm>:<digest>"}
artifact_manifest: []
```

These are placeholders. YAML is illustrative, not the selected serialization or validator.
