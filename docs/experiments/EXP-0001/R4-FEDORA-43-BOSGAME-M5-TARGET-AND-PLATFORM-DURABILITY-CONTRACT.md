# R4 — Fedora 43 Bosgame M5 Target and Platform Durability Contract

**Status:** Complete as a documentation/readiness increment; target selected, machine evidence pending; BLK-014 and BLK-015 remain open
**Scope:** EXP-0001 primary bare-metal target only
**Evidence classification:** owner-approved selection plus primary-source semantics; no machine capture, implementation, fault result, or benchmark evidence
**Profile schema:** `benchmark-environment/v1`
**Source review date:** 2026-08-26

## 1. Decision boundary and invariants

R4 selects exactly one primary execution target: **a bare-metal Bosgame M5 running Fedora Linux 43**. “Bosgame M5,” “bare metal,” and “Fedora Linux 43” are owner-approved target requirements, not observations of a particular host. No repository artifact currently verifies that the selected machine is present, that Fedora 43 is installed, or any exact CPU, memory, kernel, filesystem, mount, block-device, firmware, cache, clock-resolution, or power-loss-protection value.

This record therefore does not resolve BLK-014 or BLK-015. It defines the evidence needed to resolve them and a stack-specific contract whose D2/D3 survival promises remain blocked until the owner capture, source review, and later empirical fault evidence satisfy the gates below. Documentation of an API is not evidence that a particular event survived a fault.

The following constraints are unchanged:

- canonical history is the one authoritative source of truth;
- canonical events are immutable accepted facts, not commands;
- effective time, system-acceptance time, durability time, replay sequence, acknowledgement time, and measurement time are distinct;
- D0 and D1 are provisional and noncanonical;
- D2/D3 may become canonical only after the declared boundary operation succeeds and the complete recoverable commit envelope is established under the verified stack contract;
- materializations and checkpoints are derived, rebuildable, and never alternative authority; and
- unknown platform facts are reported rather than inferred.

R4 selects no event framing, integrity/checksum algorithm, append API, synchronization API, D3 group policy, baseline binary, or fault apparatus. R5 must choose an eligible API profile without weakening this contract.

## 2. Evidence vocabulary and acceptance gate

| Classification | Meaning in this record |
|---|---|
| **Selected** | Owner-approved requirement for the future primary cell; not yet observed. |
| **Verified** | Exact value supported by retained, reproducible output from the target and linked to provenance. |
| **Evidence-pending** | Required value has no repository evidence. It cannot support a durability or equivalence claim. |
| **Conditional** | Documented semantics apply only if named preconditions are later verified. |
| **Empirically unverified** | Documentation supports the API interpretation, but required fault-survival testing has not passed. |
| **Unsupported** | The proposed claim exceeds documented semantics or available evidence. |

BLK-014 can close only when a conforming immutable environment record contains the exact host/OS/clock/placement fields below, retained capture artifacts, capture time and mechanism, repository identity, redactions, and a reviewed explanation for every unavailable field. BLK-015 can close only when filesystem, mount, full storage/cache path, firmware, flush/FUA behavior, power-loss-protection evidence, selected API sequence, acknowledgement boundary, and promised/excluded faults are exact and mutually consistent. A vendor claim must identify the exact installed model/firmware/configuration; a generic product-family page is insufficient. Successful sync calls alone cannot close BLK-015.

Any kernel, filesystem, mount, firmware, cache, placement, or protection change creates a new environment identity and requires contract review. Ambiguity starts a new benchmark series.

## 3. `benchmark-environment/v1` target profile

This profile supplies all currently knowable values. “Evidence-pending” is a reasoned missing state, not a wildcard and not a completed environment record.

### 3.1 Identity, host, OS, CPU, memory, and clocks

| Logical field | State | Value or evidence required |
|---|---|---|
| `schema.name`, `schema.version` | Selected | `benchmark-environment`, `1`. |
| `environment_id`, `captured_at`, `capture_mechanism`, `record_producer`, `artifact_manifest`, `redactions` | Evidence-pending | Immutable record identity and capture provenance; retain raw outputs and digest them after algorithms/serialization are authorized. |
| `repository.commit`, `repository.dirty_state` | Evidence-pending for execution | Exact future execution checkout; this R4 document is not a run record. |
| `host.label` | Selected form; label pending | Pseudonymous stable label. |
| `host.execution_form` | Selected | Bare metal; capture must verify no VM/container layer rather than infer it from the product name. |
| host manufacturer/product/board/BIOS/UEFI | Evidence-pending | Exact observed identifiers and firmware revisions; redact serial numbers and UUIDs. |
| `cpu.model`, architecture/features, microcode | Evidence-pending | Exact target output, including microcode provenance. |
| `cpu.topology`, NUMA, SMT, visible CPUs | Evidence-pending | Packages/dies/cores/threads/nodes and benchmark-visible set. |
| CPU frequency policy, boost, affinity/isolation | Evidence-pending | Effective driver/governor/min/max/boost plus boot and task placement. |
| `memory.capacity_bytes`, topology, speed, limits | Evidence-pending | OS-visible bytes, NUMA and DIMM/channel facts where available; never substitute advertised product capacity. |
| huge pages, THP, swap, overcommit | Evidence-pending | Effective settings and observed use. |
| `os.name`, distribution, version | Selected; unverified | Fedora Linux 43 must be installed on the captured host. Exact edition/release files remain pending. |
| `os.kernel`, architecture | Evidence-pending | Exact release/build; “Fedora 43” does not select one kernel. |
| boot parameters, mitigations, scheduler/limits, power/thermal/background activity | Evidence-pending | Effective target state for the series. |
| `clocks` | Evidence-pending | APIs actually used, clocksource, `clock_getres` results, realtime synchronization/steps, monotonic placement, and conversion. R3 permits OS realtime for engine-assigned canonical times and run-relative monotonic time for lifecycle measurements; it does not verify target resolution. |

### 3.2 Filesystem, device, cache, and placement

| Logical field | State | Value or evidence required |
|---|---|---|
| `data_location` | Evidence-pending | Pseudonymized exact data, log/WAL, temporary, controller-ledger, and artifact paths; mount and underlying device for each. |
| `storage.stack_path` | Evidence-pending | Ordered application/runtime → Linux VFS → filesystem → dm/LVM/RAID if any → block driver/device/controller/cache → medium. |
| `filesystem` | Evidence-pending | Type and implementation/version, creation/features, allocation/discard behavior, capacity state, and complete effective mount options. No ext4, XFS, Btrfs, or other filesystem is presumed. |
| `storage.device` | Evidence-pending | Exact model, firmware, interface, capacity, medium, namespace/topology, and pseudonymized stable identity. No NVMe device is presumed. |
| controller/bridge/RAID/dm/LVM | Evidence-pending | Exact model, firmware, topology, mode, cache and battery/capacitor state, or verified absence. |
| logical/physical block and filesystem allocation/I/O sizes | Evidence-pending | Report each separately from target output. |
| scheduler/queue | Evidence-pending | Scheduler, hardware queues/depth, request settings, merges and read-ahead for every leaf device. |
| cache and write-cache behavior | Evidence-pending | Page cache, filesystem barriers/flushes, device/controller volatile write cache, FUA/flush support and effective mode. |
| `storage.power_loss_protection` | Evidence-pending | Exact installed device/controller evidence and scope; unknown until verified. Successful `fsync` is not proof. |
| network storage | Inapplicable by selection | Primary target is local bare metal; discovery of remote-backed data/log placement violates the selected target and blocks the series. |
| build/software/instrumentation/preconditioning fields | Evidence-pending and outside R4 selection | Must be frozen by their later increments before execution. R4 authorizes none. |
| `durability_contract_ref` | Conditional | This document plus a future immutable revision containing the verified exact stack and selected R5 API profile. |

## 4. Safe owner capture procedure

Run these commands **on the intended bare-metal target**, from a non-mutating administrative session, after substituting actual data and log paths. Redirecting output to owner-controlled evidence storage is acceptable; do not write capture output into the benchmark data device when that would perturb later state. These commands are observational, but privileged reads can expose identifiers. Do not run formatting, mounting/remounting, cache dropping, write-cache changes, firmware commands, SMART self-tests, benchmarks, or power faults for R4.

### 4.1 Provenance, host, OS, CPU, memory, and clocks

```text
date --iso-8601=ns
uname -a
cat /etc/os-release
cat /proc/cmdline
systemd-detect-virt
hostnamectl
lscpu --extended --all
lscpu
cat /proc/cpuinfo
cat /sys/devices/system/cpu/cpu0/microcode/version
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_driver
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
cat /sys/devices/system/cpu/cpufreq/boost
numactl --hardware
free --bytes
cat /proc/meminfo
cat /sys/kernel/mm/transparent_hugepage/enabled
swapon --show --bytes
cat /proc/sys/vm/overcommit_memory
cat /sys/devices/system/clocksource/clocksource0/current_clocksource
getconf CLK_TCK
systemctl list-unit-files --type=service | sed -n '/chronyd\|systemd-timesyncd/p'
systemctl list-units --type=service --all | sed -n '/chronyd\|systemd-timesyncd/p'
sudo dmidecode --type system --type baseboard --type bios --type memory
```

`getconf CLK_TCK` is not a `clock_getres` measurement. Because R4 may not add a collector, the owner must additionally retain output from an independently reviewed read-only utility that calls `clock_getres` for every proposed `clockid_t`, naming its source/version. Until then API resolution remains evidence-pending.

First use the two `systemctl` discovery commands to identify the installed and active realtime-synchronization provider; do not assume one from the distribution. If `chronyd.service` is installed and active, capture only read-only chrony status and configuration provenance:

```text
chronyc tracking
chronyc -n sources -v
systemctl show chronyd.service --property=ActiveState --property=SubState --property=FragmentPath
systemctl cat chronyd.service
```

`chronyc -n` suppresses hostname lookups; redact any remaining addresses, hostnames and reference identifiers consistently. If `systemd-timesyncd.service` is instead installed and active, capture:

```text
systemctl show systemd-timesyncd.service --property=ActiveState --property=SubState --property=NTPSynchronized --property=StatusText --property=FragmentPath
systemctl cat systemd-timesyncd.service
systemd-analyze cat-config systemd/timesyncd.conf
```

Do not run provider commands that add/delete sources, step or slew clocks, write configuration, or restart/reload a service. If neither provider is installed and active, record that result and identify any other provider for separate primary-manual review; synchronization state remains evidence-pending until its read-only status and configuration provenance are captured.

### 4.2 Placement, filesystem, block path, queues, and caches

Run the path-sensitive commands once for every proposed data, log/WAL, temporary, and controller-ledger path. Plain `dmsetup table` retains the default key suppression; `--showkeys` is prohibited. `dmsetup status --noflush` is required because unqualified thin-pool status can commit metadata. If discovery shows a dm-crypt mapping, `sudo cryptsetup status <MAPPING>` may be captured for non-secret status only; do not use any option or command that exposes key material.

```text
findmnt --json --output-all --target <PATH>
stat --file-system <PATH>
df --block-size=1 --output=source,fstype,size,used,avail,target <PATH>
lsblk --json --output-all
lsblk --fs --bytes
sudo blkid --probe --output export <LEAF-DEVICE>
udevadm info --query=property --name=<LEAF-DEVICE>
lspci -nnk
sudo dmsetup ls --tree
sudo dmsetup table
sudo dmsetup status --noflush
sudo blockdev --getss <LEAF-DEVICE>
sudo blockdev --getpbsz <LEAF-DEVICE>
cat /sys/class/block/<LEAF>/queue/logical_block_size
cat /sys/class/block/<LEAF>/queue/physical_block_size
cat /sys/class/block/<LEAF>/queue/scheduler
cat /sys/class/block/<LEAF>/queue/nr_requests
cat /sys/class/block/<LEAF>/queue/read_ahead_kb
cat /sys/class/block/<LEAF>/queue/write_cache
cat /sys/class/block/<LEAF>/queue/fua
cat /sys/class/block/<LEAF>/queue/rotational
```

If—and only if—the discovered leaf is NVMe, additionally capture read-only identification and health/configuration output with the installed `nvme-cli` version:

```text
nvme version
sudo nvme list --verbose --output-format=json
sudo nvme id-ctrl <CONTROLLER> --output-format=json
sudo nvme id-ns <NAMESPACE> --output-format=json
sudo nvme smart-log <NAMESPACE> --output-format=json
sudo nvme get-feature <CONTROLLER> --feature-id=6 --human-readable
```

Use filesystem-native read-only inspection only after discovery: for XFS, `xfs_info <PATH>`; for ext4, `sudo tune2fs -l <LEAF-DEVICE>`; for Btrfs, `btrfs filesystem show <PATH>` and `btrfs filesystem usage --bytes <PATH>`. A filesystem not listed here still remains eligible only after equivalent official semantics and feature evidence are reviewed.

### 4.3 Expected output, retention, and redaction

Expected output is complete command/version metadata plus exact values or an explicit permission/not-supported/not-present result. Preserve stdout and stderr separately, command text, exit status, UTC capture time, package version, and the association between every path, mount, mapper and leaf. Missing utilities are reported; do not install or change the target merely to make this R4 capture look complete.

Redact host/user names, serial numbers, WWNs, UUIDs, IP addresses, absolute user paths, and asset tags consistently. Preserve pseudonymous equality and topology: the same device/path must retain the same token across outputs. Do not redact model, firmware, kernel, filesystem type/features, mount options, queue/cache modes, block sizes, or protection evidence. Record every redaction rule and its reproducibility impact. Secret material must never be requested, displayed, captured, retained, or committed; `dmsetup --showkeys` and equivalent key-disclosing options are prohibited. If encrypted mapping parameters cannot be safely retained, record that limitation and preserve only non-secret topology.

The command review is fail-closed: omit and report any command whose installed version cannot be verified as read-only for the exact device/stack. Do not use commands or options that flush or commit metadata, initiate device self-tests, change clocks/caches/queues/power settings, reload services, mount or remount filesystems, or otherwise mutate target state. Package/manual differences remain evidence-pending rather than permission to try a potentially mutating command.

## 5. Primary-source semantics and limitations

The following are documented semantics, not empirical survival results. Version/date refers to the source version visible at the cited primary source and the R4 review date above; the later execution record must archive or immutably identify the exact source revision applicable to its installed kernel/filesystem/device.

| Source | Documented support used by this contract | Limitation / R4 interpretation |
|---|---|---|
| LVM2 project, [`dmsetup(8)` manual](https://man7.org/linux/man-pages/man8/dmsetup.8.html) | The default table output suppresses encryption keys; `--showkeys` exposes them. `status --noflush` suppresses thin-pool metadata commit associated with status. | R4 prohibits `--showkeys` and requires `--noflush`; installed-version semantics must be checked before capture. Secret material is never an evidence artifact. |
| chrony project, [`chronyc` manual](https://chrony-project.org/doc/4.8/chronyc.html) | `tracking` and `sources` report clock/source state; `-n` avoids resolving addresses to hostnames. The command also has mutating operations. | Only the listed status subcommands are permitted, conditionally when chronyd is active. The exact installed chrony version must be recorded; this citation does not assert Fedora's installed provider/version. |
| systemd project, [`systemctl` manual](https://www.freedesktop.org/software/systemd/man/latest/systemctl.html) | `list-unit-files`, `list-units`, `show`, and `cat` provide unit discovery, properties, and file content. | Only the listed observational forms are permitted; no start/restart/reload/enable operation is allowed. Provider absence or inactivity is evidence-pending, not a configured result. |
| Linux man-pages 6.15, [`write(2)`](https://man7.org/linux/man-pages/man2/write.2.html) | `write` may transfer fewer bytes; successful return does not guarantee space was reserved or data reached disk; delayed errors may surface later, including at `fsync`. | R5 must loop over short writes, preserve offsets/order, handle interruption/errors, and never treat append return as D2. Concurrent append atomicity wording does not select a record format or recovery rule. |
| Linux man-pages 6.15, [`open(2)`](https://man7.org/linux/man-pages/man2/open.2.html) | `O_APPEND` positions and writes atomically for the file operation on supported local filesystems; NFS may simulate it and corrupt under concurrent append. `O_DSYNC`/`O_SYNC` describe synchronized-I/O flags. | Primary local storage is required. R5 must still choose concurrency, flags and error handling; `O_APPEND` does not make a complete record or canonical commit atomic. |
| Linux man-pages 6.15, [`fsync(2)`](https://man7.org/linux/man-pages/man2/fsync.2.html) and [`fdatasync(2)`](https://man7.org/linux/man-pages/man2/fdatasync.2.html) | `fsync` flushes file data and associated metadata; `fdatasync` may omit metadata not needed for later data retrieval. A file sync does not necessarily persist the directory entry; the directory also needs `fsync`. Errors can report writeback failure. | Exact file creation/rename/replacement sequence determines whether a directory sync is required. Successful return supports only the verified stack contract and is not universal power-loss proof. |
| Linux man-pages 6.15, [`syncfs(2)`](https://man7.org/linux/man-pages/man2/syncfs.2.html) | Synchronizes the filesystem containing the file descriptor and can report filesystem writeback errors. | Broader scope is not automatically a per-event D2 boundary; R5 must justify membership, isolation and error attribution if proposed. |
| Linux man-pages 6.15, [`sync_file_range(2)`](https://man7.org/linux/man-pages/man2/sync_file_range.2.html) | The page explicitly warns that it does not flush disk write caches and is not suitable for data-integrity operations. | Unsupported as the sole D2/D3 boundary. |
| Linux man-pages 6.15, [`rename(2)`](https://man7.org/linux/man-pages/man2/rename.2.html) | Rename replacement is atomic with respect to the named object, with filesystem-specific flags and errors. | Namespace atomicity is not durability; file and relevant directory synchronization remain separately required by the physical design. |
| Linux man-pages 6.15, [`clock_getres(2)`](https://man7.org/linux/man-pages/man2/clock_getres.2.html) | Reports implementation clock resolution and distinguishes realtime clocks, which can jump, from monotonic clocks. | Exact target resolution must be captured; resolution is not accuracy. R3's distinct time meanings remain intact. |
| Linux kernel 6.18.0 documentation, [block writeback cache control](https://docs.kernel.org/6.18/block/writeback_cache_control.html) | Describes REQ_PREFLUSH and REQ_FUA ordering through volatile write-back caches and notes filesystems must assume volatile caches by default. | Applicable only if the selected kernel/driver/device path honors these operations; evidence and physical fault tests remain required. |
| Linux kernel 6.18.0 documentation, [ext4 admin guide](https://docs.kernel.org/6.18/admin-guide/ext4.html) | Documents ext4 data modes, barriers, commit behavior and mount options. | Conditional reference only; ext4 is not selected. Exact installed kernel/filesystem and effective options govern. `nobarrier` or unsupported caches would materially change the contract. |
| Linux kernel 6.18.0 documentation, [XFS delayed logging design](https://docs.kernel.org/6.18/filesystems/xfs/xfs-delayed-logging-design.html) | Describes XFS log durability/order mechanisms and use of cache flush/FUA in covered designs. | Conditional reference only; XFS is not selected. It does not prove the installed hardware path survives loss. |
| Fedora Project, [Fedora Linux Releases](https://docs.fedoraproject.org/en-US/releases/) | Identifies Fedora Linux releases and lifecycle documentation. | Supports the distribution naming only. It does not establish the target edition, installed release, kernel, filesystem default, or effective configuration. |
| NVM Express, [NVM Express specifications](https://nvmexpress.org/specifications/) | Provides the authoritative specification family from which exact controller commands/features must be interpreted. | No NVMe device/version is selected. Exact installed model, firmware, applicable spec revision, volatile-write-cache feature and vendor protection evidence must be captured before any claim. |

Where a later selected filesystem or device is absent from this table, R4 must be revised with its primary documentation before BLK-015 can close. Documentation describes intended interfaces; only admissible apparatus-specific results can support actual process-crash, kernel-reset, or abrupt-power-loss survival.

## 6. Platform durability contract

### 6.1 Common lifecycle and error obligations

A later physical profile must establish and recoverably validate the entire event envelope before canonical commit. Sequence reservation and permanent gaps follow R3. Event bytes, integrity/finalization marker(s), and any metadata required to recognize the record as complete must be submitted in the selected order. The canonical durability time is captured **after** the declared durability-boundary operation returns success, while canonical commit occurs only after the complete envelope can be recoverably classified as valid. Durability time never substitutes for the boundary or finalization evidence and cannot be backdated to submission.

Every write path must handle partial/short writes, `EINTR`, deferred/writeback errors, out-of-space/quota conditions, read-only transitions, device loss, and synchronization errors. A zero-progress or ambiguous result fails closed. An explicit boundary failure yields no canonical acknowledgement. Physical residue remains noncanonical unless the recovery contract independently proves complete commit. The commit-before-acknowledgement window remains an uncertain caller outcome and may recover one canonical event.

File content durability and namespace durability are separate. Any design that creates, links, renames, replaces, rotates, or deletes a log/segment must identify and synchronize every required file and parent directory in order. D3 records exact group membership before the shared boundary, one shared outcome, individual commit eligibility and acknowledgement. D3 is not an atomic multi-event transaction.

### 6.2 Mode mapping for this selected target

| Mode | Eligible later Linux API families | Acknowledgement and canonical status | Permitted claim now |
|---|---|---|---|
| **D0** | Process-memory operations only. | May acknowledge after in-process acceptance; provisional, noncanonical, no recovery promise. | Permitted semantic label; no target evidence required and no durability claim. |
| **D1** | Complete `write`/`writev`/`pwrite`/`pwritev`-family submission into Linux-managed buffers without a declared stable-storage sync; buffered append is a candidate, not selected. | May acknowledge only after the chosen submission rule succeeds; provisional and noncanonical. | May conditionally claim OS-buffer acceptance for the exact API/error profile. Process-crash survival is empirically unverified and never promotes canonical status; kernel reset/power loss are unsupported. |
| **D2** | A later R5 choice among `fsync`, `fdatasync` plus required namespace operations, synchronized-open/write flags, or another primary-source-justified Linux interface; `sync_file_range` alone is excluded. | Per event, after complete finalization and the exact boundary returns success, capture durability time, establish canonical commit, then return canonical acknowledgement. | Semantics documented but **blocked and empirically unverified**. No process/kernel/power-loss survival may yet be promised. |
| **Controlled D3** | Same eligible boundary families as D2, invoked once for a predeclared, observable group whose complete members were finalized before submission. | Shared success makes each valid member eligible for individual canonical commit; capture each durability time after shared success under R3, then acknowledge individually. Shared failure acknowledges none canonically. | Semantics documented but **blocked and empirically unverified**. No atomic batch claim; no survival promise yet. |

R5 must explicitly choose between data-and-metadata and data-only synchronization, state why omitted metadata cannot affect recovery, define append offsets and concurrency, and list every directory sync. It must also bind the file descriptor to the intended local mount/device stack and prevent silent fallback to a different placement. R5 cannot claim D2/D3 until the exact filesystem/mount/device/cache/firmware/protection preconditions are verified and the later correctness/fault gate passes.

### 6.3 Conditional and unsupported claims

- **Conditional:** documented completion of `fsync`/`fdatasync` or appropriately defined synchronized I/O orders Linux-visible writes according to the exact kernel, filesystem, mount and block path semantics.
- **Empirically unverified:** survival of process termination, kernel panic/reset, abrupt power loss, torn/partial writes, or volatile-cache loss on the selected physical host.
- **Unsupported now:** “on stable media,” “power-loss safe,” “no torn writes,” “atomic event append,” “exactly once,” or D2/D3 canonical survival based solely on a successful syscall, distribution/product defaults, generic device-family claims, or vendor marketing.
- **Blocked:** any D2/D3 claim while filesystem/mount, full device/controller/cache path, firmware, flush/FUA behavior, placement, or power-loss protection is unknown.

## 7. Contract-to-fault matrix

Every proposed D2/D3 claim is currently blocked. “Valid” below means a complete integrity-valid canonical event at its original sequence; “provisional” means residue that cannot be promoted; “terminal damage” means a deterministic valid prefix followed by an invalid/partial terminal extent under R1; “ambiguous/fail closed” means canonical status cannot be established.

| Proposed claim / fault | Promised surviving state and acknowledgement point | Required recovery outcome | Required apparatus/evidence | Current status |
|---|---|---|---|---|
| D2, process crash after canonical acknowledgement | The one acknowledged event, after its per-event boundary success and commit. | Valid exactly once; otherwise fail. Pre-boundary residue is provisional. | Verified exact stack/API plus independently controlled process kill at lifecycle points, retained oracle, repeated recovery. | Documented lifecycle; empirically unverified and blocked by BLK-014/015/017/022. |
| D2, kernel panic/reset after canonical acknowledgement | Same one event only if the exact contract promises this fault. | Valid exactly once; terminal damage only outside promised committed extent; ambiguity fails closed. | Validated panic/reset mechanism proving the kernel stopped, independent controller/oracle, exact cache-path evidence. | Empirically unverified; unsupported until selected and tested. |
| D2, abrupt power loss after canonical acknowledgement | Same one event only if the exact verified stack contract and apparatus promise loss of all relevant volatile layers. | Valid exactly once; loss/corruption is failure; undecidable state fails closed. | Physical power cut with independent control/ledger, verified device/controller cache and PLP scope, repetitions and untouched images. | Blocked; no PLP or physical apparatus evidence. Successful sync is insufficient. |
| D2, short/partial/torn/truncated write before boundary | No canonical event promised; no canonical acknowledgement. | Provisional residue or deterministic terminal damage; never promotion; ambiguous is fail closed. | R5 short-write/error injection plus R1-selected framing/integrity profile and preserved image. | Recovery requirement documented; mechanism/encoding empirically unverified and blocked. |
| D2, cache loss after reported boundary | Event survives only if flush/FUA and every volatile layer are verified within promise. | Valid exactly once or fail; ambiguity fails closed. | Layer-specific cache-loss method, kernel trace/device evidence, firmware/vendor primary evidence, physical validation. | Unsupported/blocked with current unknown cache path. |
| D2, boundary error or uncertain return | No successful canonical acknowledgement; physical completion may be uncertain. | Provisional unless independent commit evidence satisfies the future contract; otherwise ambiguous/fail closed. | Injected and native errors, oracle at submission/boundary/commit/ack points, writeback-error observation. | Documented; empirically unverified and blocked. |
| D3, process crash after member acknowledgement | Every individually committed/acknowledged member of the recorded group. Group membership alone promises nothing. | Each promised member valid exactly once; joined/uncommitted members provisional or uncertain according to oracle. | Exact membership/cut/shared outcome/per-member commit ledger and controlled process kill. | Documented lifecycle; empirically unverified and blocked. |
| D3, kernel panic/reset after shared boundary or acknowledgements | Individually committed members only, if fault is promised. | Per-member valid/provisional/terminal-damage classification; ambiguity fails closed; no fabricated all-or-none result. | Validated reset placement around shared sync and each commit/ack, exact stack/cache evidence. | Empirically unverified; unsupported until selected and tested. |
| D3, abrupt power loss/cache loss | Individually committed members only if exact contract covers every volatile layer. | Same per-member rules as D2; shared success is not proof of power-loss survival or transaction atomicity. | Physical apparatus, independent ledger, cache/PLP evidence and repeated recovery across group boundaries. | Blocked/unsupported now. |
| D3, partial/torn/truncated member or shared-sync error | No member receives canonical acknowledgement after shared error; already proven commits retain their oracle class. | Complete valid members assessed individually; invalid terminal member is terminal damage; unknown commit is ambiguous/fail closed. | R1 integrity/framing, injected partial/error placements, exact group ledger and preserved images. | Documented requirement; physical mechanism empirically unverified and blocked. |

A future result uses the EXP-0000 top-level classifications: valid execution can pass or fail; missing environment/apparatus evidence makes it invalid; inability to demonstrate the declared physical boundary is inconclusive. “Not tested” never means pass.

## 8. Blocker disposition and next boundary

- **BLK-014 remains open:** the target form is selected, but no completed environment record or retained owner capture verifies the particular Bosgame M5, Fedora 43 installation, CPU/memory/firmware/kernel/clock/placement facts.
- **BLK-015 remains open:** filesystem, mount, block path, device/controller firmware, cache behavior, flush/FUA propagation, PLP, and empirical survival are unknown; no exact R5 API profile exists.
- **R4 documentation is complete:** it supplies the selected target, evidence-pending conforming profile, capture procedure, primary-source semantics, conditional platform contract, and complete proposed D2/D3 fault mapping.

The next action is owner capture and review of the resulting immutable evidence. That is not R5 authorization. R5–R9, implementation, fixtures, tooling, CI expansion, benchmark execution, and performance claims remain prohibited until separately authorized by the readiness plan.
