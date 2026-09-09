# EXP-0002 result retention

Six host series are cataloged in the host series entries section below. Status
remains Ready.
The 100K measurement has not been executed. Small/1K automated correctness tests
are proof checks, not retained performance evidence.

Candidate: **D1 ordinary writes, no fsync, no crash-survival claim**.
Legacy native insert/replace/delete log syncs are not equivalent to candidate D1;
field updates use mmap without per-operation flush. No power-loss claim is made.

Use the exact [manual commands](../../../../experiments/convergence-memory/README.md)
from the repository root, directing output to the operator's scratch directory
outside this checkout. Each run requires a new output directory. Default output
retains series `environment.txt`, `source.sha256`, `source.patch`, `trials.csv`,
`summary.txt`, and per-trial `environment.txt`, `throughput.txt`, `results.cmt`,
`observations.cmt`. Store files are deleted after digest comparisons and size
capture, with engine handles dropped first; no trace or source tree is copied.
The environment records the seed, input SHA-256 and exact regeneration command.
Append `--retain-store` to keep stores and a trace copy for forensic runs; no
source copy is made in either mode. Retain failed/negative raw results and any
`failure.txt`. Do not pool dispatch modes, builds or environments.

## Committed-evidence layout

For each selected series create a uniquely named directory under this results
directory and copy **only** these files, preserving relative trial paths:

```text
<series-id>/environment.txt
<series-id>/source.sha256
<series-id>/source.patch
<series-id>/trials.csv
<series-id>/summary.txt
<series-id>/warmup-0/throughput.txt
<series-id>/measured-1/throughput.txt
<series-id>/measured-2/throughput.txt
<series-id>/measured-3/throughput.txt
<series-id>/measured-4/throughput.txt
<series-id>/measured-5/throughput.txt
```

Do not copy per-trial `environment.txt`, stores, traces, source trees,
`results.cmt` or `observations.cmt` into committed evidence. The two raw
per-operation files stay with the operator's output directory. Each trial's
`throughput.txt` records their absolute paths and SHA-256s; verify those hashes
when collecting evidence. If raw files are relocated, record the new location
alongside the original identity in the host entry. Retain that external evidence
for reproductions; a path without retained bytes is not a durable reference.
For failed/incomplete series, keep raw failures externally and state which
subset files are missing and why. Do not invent successful trial files.

## Host series entry template

Copy and fill this template when cataloging a host series. Leave entries to the
host; this template is not an observation.

```text
Series ID:
Engine: candidate | legacy
Mode: single | pipelined | atomic
Size: small | 1K | 10K | 100K
Date (UTC):
Operator output location (absolute path or durable external reference):
Validity: all six trials valid | invalid/incomplete (reason):
Committed subset directory:
Summary SHA-256 (hash of summary.txt bytes):
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any):
100K executed/not-executed (when applicable):
```

Link the committed `summary.txt` and `trials.csv` in the filled entry. Hash with
`Get-FileHash -Algorithm SHA256` on Windows or `sha256sum` on Linux. Add corrections
as new entries with a reference to the original; do not overwrite observations.

## Host series entries

Six host series are cataloged below and their committed subsets live in this
directory. All were run on the Windows build box (GNU 1.89.0 release binaries,
`rev=54bd1a479b1e38cda6a15955de6eef3ec1b54cd0` plus the uncommitted change of the
named snapshot). Four were produced with binaries from snapshot `850eb388`
(before the replay/decode stage split, the RSS probe fix and the trace change of
the final round) and two with binaries from snapshot `bd99987b`, the revision
committed with this change. Entries are descriptive only; the predeclared
methodology forbids an equal-durability winner claim and none is made here.

Accepted deviation from work-order R7 ("raw result locations under
`docs/experiments/EXP-0002/results/`"): raw per-operation samples
(`results.cmt`, `observations.cmt`) are not committed; only the subset defined
above is. Each trial's `throughput.txt` carries their absolute path and SHA-256.
The host recorded the owner's acceptance of this deviation on 2026-09-09.

```text
Series ID: 2026-09-08-850eb388-candidate-1k
Engine: candidate
Mode: n/a (candidate has no dispatch mode)
Size: 1K
Date (UTC): 2026-09-09 02:36:18
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final2\candidate-1k (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-08-850eb388-candidate-1k/
Summary SHA-256 (hash of summary.txt bytes): bcfec6f86f9e144770fc298a21be4b2c5163ea8539b21e75bee846064ee6bf21
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot 850eb388 (before the N1 stage split, so the `replay` stage there includes CMM1 payload decoding; before the N5 trace change). Peak RSS is `unavailable: powershell: program not found` on every trial (probe fixed in snapshot bd99987b; see the re-run entries).
100K executed/not-executed (when applicable): not executed
Input: 1K trace, 13,025 operations, input_sha256 4335b5c902e859aa26c29a680b1a4cf2141f15534f530dad7e16b1e11c12dc0c
```
Linked files: [summary.txt](2026-09-08-850eb388-candidate-1k/summary.txt), [trials.csv](2026-09-08-850eb388-candidate-1k/trials.csv).

```text
Series ID: 2026-09-08-850eb388-legacy-1k-single
Engine: legacy
Mode: single
Size: 1K
Date (UTC): 2026-09-09 02:36:56
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final2\legacy-1k-single (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-08-850eb388-legacy-1k-single/
Summary SHA-256 (hash of summary.txt bytes): 8093cd99c60450ffbb92f184e54c8f62bdcb01cb5270355d3abd576dd827c25b
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot 850eb388 (before the N5 trace change). Peak RSS unavailable on every trial (see above).
100K executed/not-executed (when applicable): not executed
Input: 1K trace, 13,025 operations, input_sha256 4335b5c902e859aa26c29a680b1a4cf2141f15534f530dad7e16b1e11c12dc0c
```
Linked files: [summary.txt](2026-09-08-850eb388-legacy-1k-single/summary.txt), [trials.csv](2026-09-08-850eb388-legacy-1k-single/trials.csv).

```text
Series ID: 2026-09-08-850eb388-candidate-10k
Engine: candidate
Mode: n/a (candidate has no dispatch mode)
Size: 10K
Date (UTC): 2026-09-09 02:40:17
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final2\candidate-10k (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-08-850eb388-candidate-10k/
Summary SHA-256 (hash of summary.txt bytes): ceb21198224bd5a113f8ac404a4ebaa6818efe4fc8a860b269ac2698c54db7dc
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot 850eb388 (pre-N1 stage attribution, pre-N5 trace). Peak RSS unavailable on every trial. Wall time for the six trials: 422 s.
100K executed/not-executed (when applicable): not executed
Input: 10K trace, 130,115 operations, input_sha256 781b356b60a5365c39fafff708bd663c48ddafb02743d0e43171136cc59a5469
```
Linked files: [summary.txt](2026-09-08-850eb388-candidate-10k/summary.txt), [trials.csv](2026-09-08-850eb388-candidate-10k/trials.csv).

```text
Series ID: 2026-09-08-850eb388-legacy-10k-single
Engine: legacy
Mode: single
Size: 10K
Date (UTC): 2026-09-09 02:47:19
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final2\legacy-10k-single (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-08-850eb388-legacy-10k-single/
Summary SHA-256 (hash of summary.txt bytes): f4298a2aa8ea0589d4113e96c3838501c117cb971f9d9f5f76915dad1c70b4fa
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot 850eb388 (pre-N5 trace). Peak RSS unavailable on every trial. Wall time for the six trials: 10,883 s (3 h 1 m); each measured trial about 29 minutes.
100K executed/not-executed (when applicable): not executed
Input: 10K trace, 130,115 operations, input_sha256 781b356b60a5365c39fafff708bd663c48ddafb02743d0e43171136cc59a5469
```
Linked files: [summary.txt](2026-09-08-850eb388-legacy-10k-single/summary.txt), [trials.csv](2026-09-08-850eb388-legacy-10k-single/trials.csv).

```text
Series ID: 2026-09-09-bd99987b-candidate-1k
Engine: candidate
Mode: n/a (candidate has no dispatch mode)
Size: 1K
Date (UTC): 2026-09-09 09:40:25
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final3\candidate-1k (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-09-bd99987b-candidate-1k/
Summary SHA-256 (hash of summary.txt bytes): 4c7da459ec5ab6ca3f3dfa08dda6f289172813fffd882dc237e04453e0987bbe
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot bd99987b (the revision committed with this change): `replay` and `decode` are separate stages and peak RSS is captured (baseline after load 130,764,800 bytes; trial high water 192-194 MB). Re-run of the 1K cell only.
100K executed/not-executed (when applicable): not executed
Input: 1K trace, 15,025 operations, input_sha256 53201589e30fd48d4aeab077a6435bddc86902c25bd35bf70d1f39044cd070d0
```
Linked files: [summary.txt](2026-09-09-bd99987b-candidate-1k/summary.txt), [trials.csv](2026-09-09-bd99987b-candidate-1k/trials.csv).

```text
Series ID: 2026-09-09-bd99987b-legacy-1k-single
Engine: legacy
Mode: single
Size: 1K
Date (UTC): 2026-09-09 09:41:00
Operator output location (absolute path or durable external reference): C:\tools\naner\home\.tmp\claude\C--dev-rusty-data-os\a0e90d1e-eff7-404d-a1f1-c14358341946\scratchpad\exp0002-final3\legacy-1k-single (host scratch directory; see raw relocation below)
Validity: all six trials valid
Committed subset directory: 2026-09-09-bd99987b-legacy-1k-single/
Summary SHA-256 (hash of summary.txt bytes): a2aabfacb77b2552b9e4bee33df228d0150a977ae2b764dbb25d89f65ba39907
Raw references: paths and SHA-256s in each trial's throughput.txt
Raw relocation / missing files (if any): results.cmt and observations.cmt remain at the operator scratch path above, which is session-local and not durable; the owner must copy them to durable storage or accept that only their SHA-256s survive (open follow-up). Binaries from snapshot bd99987b; peak RSS captured (baseline after load 131,624,960 bytes; trial high water 131.6 MB). Re-run of the 1K cell only; 10K was not re-run on these binaries.
100K executed/not-executed (when applicable): not executed
Input: 1K trace, 15,025 operations, input_sha256 53201589e30fd48d4aeab077a6435bddc86902c25bd35bf70d1f39044cd070d0
```
Linked files: [summary.txt](2026-09-09-bd99987b-legacy-1k-single/summary.txt), [trials.csv](2026-09-09-bd99987b-legacy-1k-single/trials.csv).

The predeclared [methodology](../../EXP-0002-convergence-memory.md) controls
interpretation. Native durability mismatch and width-one batch measurements are
explicit limitations, not evidence of equal-durability throughput superiority.

[Implementation and validation report](../IMPLEMENTATION-REPORT.md) records the
passed Windows offline proof and complete [N1-N8 proof output](proof-windows-n1-n8.txt).
The [M1-M6 proof output](proof-windows-m1-m6.txt) is preserved.
The [L1-L4 proof output](proof-windows-l1-l4.txt) is preserved.
The [initial proof output](proof-windows.txt) remains preserved.
These are correctness/build evidence, not operator measurements.
