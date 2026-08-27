# EXP-0001 R7 closed physical field ledger

**Profile:** `EXP1-R7-JSON-JCS-1`
**Status:** normative documentation design; implementation and evidence absent

This is the complete physical ledger incorporated by the [R7 authority](R7-BENCHMARK-RECORDS-ARTIFACTS-INSTRUMENTATION-FAULTS.md). Objects are closed. Every table row is required unless its condition says otherwise. An empty array is a known empty set, never missing. Array order is normative. R9 may encode this ledger but may not complete or reinterpret it.

## 1. Type algebra and common envelope

| Token | Exact JSON representation, range, and unit |
|---|---|
| `text` / `nonempty` | Unicode string / string of at least one Unicode scalar value. |
| `uuid` | Lowercase RFC 4122 text `8-4-4-4-12`; variant bits `10`; version nibble `4`. |
| `sha256` / `git_sha` | Exactly 64 / 40 lowercase hexadecimal characters. |
| `i64` | Canonical decimal string in `-9223372036854775808..9223372036854775807`. |
| `u64` | Canonical decimal string in `0..18446744073709551615`. |
| `u32` / `u16` | JSON integer in `0..4294967295` / `0..65535`. |
| `safe_count` | JSON integer in `0..9007199254740991`. |
| `ns_i64` | `i64`, Unix-epoch nanoseconds. |
| `ns_u64` | `u64`, elapsed or monotonic nanoseconds. |
| `bytes_u64` | `u64`, octets. |
| `ratio` | Closed `{denominator:u64,numerator:u64}`; denominator must be nonzero. |
| `uri` | Absolute normalized `file:` or `https:` URI, without query, fragment, credentials, dot segments, or sensitive literal. |
| `state<T>` | Exactly one of `{"state":"present","value":T}`, `{"reason":nonempty,"state":"missing"}`, `{"reason":nonempty,"state":"not_collected"}`, `{"reason":nonempty,"state":"unsupported"}`, `{"reason":nonempty,"sanitized_artifact_id":uuid,"state":"redacted"}`, or `{"state":"not_applicable"}`. |

`missing` means expected but unknown/unavailable; `not_collected` means the declared measurement did not occur (including logical `unmeasured`); `unsupported` means the selected interface cannot supply it; `redacted` means captured source was replaced by the named sanitized artifact; and `not_applicable` means the fact has no meaning in this cell. Only `state<T>` admits these states. A present numeric value uses the exact range and unit of `T`.

Every record is the closed object below. Members are stored in JCS order, independently of table order.

| Member | Type / condition |
|---|---|
| `body` | The one body matching `record_kind`. |
| `correction_reason` | `state<nonempty>`; `not_applicable` for an original and `present` iff `supersedes_record_id` is present. |
| `created_at_utc_ns` | `ns_i64`. |
| `record_id` | `uuid`; immutable assigned identity. |
| `record_kind` | `environment`, `raw_result`, `artifact_manifest`, `fault_plan`, `fault_outcome`, or `validation_report`. |
| `run_id` | `state<uuid>`; present for run records; `not_applicable` only for series-scoped environment, manifest, or validation. |
| `schema_version` | Exactly `EXP1-R7-JSON-JCS-1`. |
| `series_id` | `uuid`. |
| `supersedes_record_id` | `state<uuid>`; `not_applicable` for an original; `present` for a correction; `missing` only while preserving a damaged imported chain. Other states invalid. |

## 2. Reusable closed objects

| Object | Exact members and rules |
|---|---|
| `ref` | `{artifact_id:uuid,byte_length:bytes_u64,sha256:sha256,uri:uri}`. |
| `record_ref` | `{record_id:uuid,record_sha256:sha256}`. |
| `source` | `{artifact_id:state<uuid>,mechanism:nonempty,version:state<nonempty>}`. |
| `fact<T>` | `{source:source,value:state<T>}`. |
| `named_fact<T>` | `{name:enum,source:source,value:state<T>}`; arrays sort by ASCII `name`, names unique. |
| `quantity` | `{method:nonempty,unit:enum,value:state<i64-or-u64>}`; the field-specific rule below fixes sign and unit. |
| `configuration` | `{name:nonempty,source:source,type:boolean|i64|u64|text,value:state<boolean-or-i64-or-u64-or-text>}`; `type` fixes the present union arm; arrays sort by UTF-8 `name`, names unique. |
| `artifact` | `{artifact_id:uuid,byte_length:bytes_u64,created_by_record_id:uuid,logical_path:nonempty,media_type:nonempty,retention_state:staged|published|superseded|expired|deleted,role:workload_manifest|environment_record|raw_result|fault_plan|fault_outcome|lifecycle_ledger|apparatus_capture|recovery_capture|validation_report|sanitized_derivative|interpretation|configuration|deletion_evidence,sensitivity:public|sanitized|access_sensitive,sha256:sha256,uri:uri,validation_report_ids:[uuid]}`; report IDs ascending; artifact arrays sort by UTF-8 `logical_path`, paths are unique, and path rules are R7 section 4. |
| `edge` | `{from_artifact_id:uuid,relation:generated_from|validated_by|corrects|supersedes|sanitizes|decodes_to|interprets,to_artifact_id:uuid}`; sort by the three fields in that order; no duplicate. |
| `check` | `{check_id:nonempty,evidence_artifact_id:state<uuid>,message:state<nonempty>,outcome:pass|fail|not_tested}`; sort by `check_id`. |
| `error` | `{byte_offset:state<u64>,code:io|length|utf8|json-syntax|duplicate-member|non-ijson|noncanonical|unsupported-version|unknown-field|missing-field|type|range|enum|ordering|duplicate-or-conflict|reference|digest|supersession-cycle|policy,message:nonempty}`; sort by present byte offset (missing last), then code, then message. |
| `interval` | `{clock_id:nonempty,elapsed_ns:ns_u64,end:state<i64-or-u64>,method:nonempty,precision_ns:state<u64>,source:source,start:state<i64-or-u64>,time_domain:monotonic|utc}`; the domain fixes both endpoint arms; present endpoints are both required and `end >= start`; elapsed is measured rather than inferred and any disagreement is invalid. |
| `count` | `{method:nonempty,unit:events|operations,value:state<u64>}`. |
| `rate` | `{denominator_ns:ns_u64,interval:interval,method:nonempty,numerator:u64,unit:bytes_per_second|operations_per_second,value:state<{denominator:u64,numerator:u64}>}`; denominator is nonzero for a present rate, the rational value is numerator operations/bytes per denominator seconds, and all duplicated inputs must agree. |
| `metric` | `{method:nonempty,scope:nonempty,source:source,unit:allocations|bytes|events|nanoseconds|operations|ratio,value:state<i64-or-u64-or-ratio>}`; each field below fixes its arm and unit. |
| `artifact_set` | `{artifacts:[ref],manifest_ref:ref}`; artifact references sort by `artifact_id`, are unique, and the manifest contains matching immutable media type, role, producer, relation, and retention metadata. |
| `provenance_ref` | `{edge_artifact_ref:ref,endpoint_artifact_ids:[uuid]}`; IDs sort ascending, are unique, include the raw-result artifact and every directly referenced artifact, and resolve to the typed edges required by the logical contract. |

## 3. `environment` body

The closed body is `{artifact_manifest,authority_revisions,baseline,build,capture,clocks,configuration_refs,cpu,data_locations,deviations,durability_contract_ref,fault_apparatus,host,instrumentation,memory,os,preparation,record_producer,redactions,repository,scheduler_security,storage}`.

| Member | Closed structure, values, condition, and order |
|---|---|
| `artifact_manifest` | `[ref]`, sorted by `artifact_id`; at least one configuration/capture reference or an explicit manifest artifact containing the empty set. |
| `authority_revisions` | `[{path:nonempty,sha256:sha256}]`, sorted by UTF-8 `path`; paths unique. |
| `baseline` | `state<{effective_configuration:[configuration],identity:[named_fact<text>]}>`; present iff a baseline is used. Identity names: `adapter`, `binding`, `build`, `commit`, `executable`, `product`, `release`. |
| `build` | `{dependencies:[configuration],facts:[named_fact<text>]}`. Fact names: `command`, `compiler`, `compiler_flags`, `identity`, `linker`, `linker_flags`, `profile`, `rust_components`, `rust_host`, `rust_target`, `rust_toolchain`, `target`, `target_features`. Rust names are `not_applicable` when Rust is not measured. |
| `capture` | `{captured_at_utc_ns:ns_i64,mechanism:nonempty,mechanism_version:state<nonempty>}`. |
| `clocks` | `[{clock_id:nonempty,clock_class:cpu|monotonic|realtime|timer,conversion:state<nonempty>,placement:nonempty,precision_ns:state<u64>,resolution_ns:u64,source:source,synchronization:state<nonempty>}]`, sorted by `clock_id`, unique; all used clocks present. |
| `configuration_refs` | `[{configuration_kind:adapter|instrumentation|orchestration|platform_contract|subject|workload,reference:ref}]`, sorted by `configuration_kind`, unique. |
| `cpu` | `{facts:[named_fact<text>],topology:[named_fact<u64>]}`. Fact names: `affinity`, `architecture`, `boost`, `features`, `frequency_governor`, `frequency_max_hz`, `frequency_min_hz`, `isolation`, `microcode`, `model`, `scaling_driver`, `smt`. Topology names: `benchmark_visible_cpus`, `dies`, `numa_nodes`, `physical_cores`, `sockets`, `threads`. Frequency text includes an exact decimal hertz value or missing state. |
| `data_locations` | `[{location_role:data|log|temp|wal,permissions:nonempty,pseudonym:nonempty,stack_leaf:nonempty}]`, sorted by `location_role`, unique; absent roles use entries whose values say `not applicable` only through a referenced storage fact, never a sensitive path. |
| `deviations` | `[{deviation_id:nonempty,impact:nonempty,reason:nonempty}]`, sorted by ID; empty means none. |
| `durability_contract_ref` | `ref`. |
| `fault_apparatus` | `state<{facts:[named_fact<text>],references:[ref]}>`; present for fault/recovery cells. Fact names: `control_host`, `injector`, `mechanism`, `recovery_apparatus`, `topology`, `trigger`, `version`. |
| `host` | `{execution_form:bare_metal|container|nested|virtual_machine,label:nonempty,virtualization:state<{facts:[named_fact<text>],layers:[nonempty]}>}`. Virtualization fact names: `guest_type`, `limits`, `overcommit`, `passthrough`, `pinning`, `runtime`, `sharing`, `vcpu`; layers outermost to innermost; bare metal requires `not_applicable`. |
| `instrumentation` | `[{enabled:boolean,identity:nonempty,overhead_status:not_measured|measured|unsupported,privilege:state<nonempty>,sampling:state<nonempty>,source:source}]`, sorted by identity, unique. |
| `memory` | `{capacity_bytes:fact<bytes_u64>,facts:[named_fact<text>],limits:[named_fact<bytes_u64>],numa_nodes:[{capacity_bytes:bytes_u64,node:u32}],speed_mt_s:fact<u64>}`. Fact names: `channel_population`, `dimm_population`, `huge_pages`, `numa_policy`, `overcommit_policy`, `swap_devices`, `swap_observed_use`, `swap_policy`, `thp_policy`, `topology`; limit names: `address_space`, `cgroup`, `job`, `locked`, `swap`, `visible`; nodes sort ascending. Speed unit is megatransfers/second. |
| `os` | `{facts:[named_fact<text>]}`. Names: `background_activity`, `boot_parameters`, `distribution`, `image_release`, `kernel_architecture`, `kernel_build`, `kernel_release`, `name`, `power_policy`, `security_mitigations`, `thermal_state`, `version`. |
| `preparation` | `{cache_initial_state:nonempty,cleanup_reuse_policy:nonempty,preconditioning:nonempty,subject_initial_state:nonempty}`. |
| `record_producer` | `{role:nonempty,tool:nonempty,version:state<nonempty>}`. |
| `redactions` | `[{field:nonempty,impact:nonempty,policy:nonempty,reason:nonempty,sanitized_artifact_id:uuid}]`, sorted by field; empty means none. |
| `repository` | `{commit:git_sha,dirty_state:clean|dirty|unknown,patch_artifact_id:state<uuid>,submodules:[{commit:git_sha,path:nonempty}]}`; submodules sorted by path; dirty requires patch present, clean requires `not_applicable`, unknown requires `missing`. |
| `scheduler_security` | `{facts:[named_fact<text>]}`. Names: `container_limits`, `io_limits`, `memory_limits`, `open_file_limits`, `privilege_posture`, `scheduler_class`, `scheduler_priority`, `scheduler_tunables`, `security_policy`, `thread_limits`. |
| `storage` | `{block_sizes:[named_fact<bytes_u64>],facts:[named_fact<text>],free_space_bytes:fact<bytes_u64>,stack_path:[{layer_ordinal:u32,layer_type:application|runtime|vfs|filesystem|volume|raid|block_device|controller|cache|medium,pseudonym:nonempty}]}`. Block names: `filesystem_allocation`, `filesystem_io`, `logical_sector`, `physical_sector`. Fact names: `allocation`, `barrier_flush`, `cache_layers`, `controller`, `controller_battery`, `controller_cache`, `controller_firmware`, `controller_mode`, `device_capacity_bytes`, `device_firmware`, `device_interface`, `device_medium`, `device_model`, `discard`, `filesystem_creation_features`, `filesystem_type`, `filesystem_version`, `initial_capacity_use`, `mount_options`, `network_failure_domains`, `network_protocol`, `network_versions`, `power_loss_evidence`, `power_loss_protection`, `queue_count`, `queue_depth`, `queue_merge`, `queue_read_ahead_bytes`, `queue_scheduler`, `stable_device_pseudonym`, `volatile_layers`; stack ascending, contiguous from zero. |

## 4. `raw_result` body

The closed body has exactly the following members (shown in JCS name order):
`{ack_boundary,adapter_ref,allocations,amplification,artifacts,background_work,baseline_id,canonical_status,configuration_refs,correctness,cpu,d_mode,deviations,encoded_bytes,environment_ref,equivalence,errors,execution_observations,experiment_ref,fault_contract,hypothesis_refs,interval,io,latency,lifecycle_interval,logical_bytes,memory,operation_counts,operations,phase,physical_bytes,platform_contract_ref,producer_record,profile_id,provenance,recovery,repetition_id,repository,requirement_refs,resource_measurements,result_classification,sample_id,sample_population,subject_id,synchronization,throughput,time_meanings,validation,visibility,workload_ref}`.

| Member | Closed structure and rule |
|---|---|
| Identity/input fields | `profile_id`, `subject_id`, `repetition_id`, `sample_id`, and `experiment_ref` are nonempty; `baseline_id` is `state<nonempty>`; `environment_ref` is `record_ref`; workload, adapter, platform, and producer fields are `ref`; `configuration_refs` is a nonempty artifact-set; `hypothesis_refs` and `requirement_refs` are sorted unique `[nonempty]`; `repository` is `{commit:git_sha,dirty_state:clean|dirty|unknown,patch_artifact_id:state<uuid>}` with the environment rules. Bare IDs never satisfy a reference. |
| `d_mode` / `canonical_status` | `d0|d1|d2|d3`; `provisional|canonical_committed`. D0/D1 require provisional. |
| `ack_boundary` | `{name:nonempty,source:source}`; the same boundary governs latency and throughput. |
| `interval` / `lifecycle_interval` | `interval`; the general observation interval and exact acknowledgement-latency endpoints respectively. Lifecycle method names its start/end events and includes D3 group wait when applicable. |
| `time_meanings` | Closed `{durability:state<{source:source,value_ns:ns_i64}>,effective:state<{source:source,value_ns:ns_i64}>,observation:state<{source:source,value_ns:ns_i64}>,system_acceptance:state<{source:source,value_ns:ns_i64}>}`. Each meaning is independently sourced; inapplicable values are explicit. |
| `operation_counts` | Closed `{accepted:count,acknowledged:count,attempted:count,committed:count,corrupt:count,failed:count,missing:count,provisional:count,recovered:count,rejected:count,uncertain:count}`. Every unit is `operations`; partition/double-count definitions are supplied by each method and conflicts invalidate the record. |
| Byte accounts | `logical_bytes`, `encoded_bytes`, and `physical_bytes` are `[{domain:enum,bytes:state<bytes_u64>,method:nonempty}]`. Logical domains are `envelope|key|payload|value`; encoded domains `complete_event|framing|integrity`; physical domains `checkpoint|compaction|database|manifest|other|read|requested_io|sst|synchronized|temporary|wal|written`. Arrays use displayed order and contain every domain exactly once. |
| `throughput` | `{bytes:rate,operations:rate}`; byte rate unit is `bytes_per_second`, operation rate unit `operations_per_second`; each repeats the exact general `interval`. Missing rate values retain numerator/denominator/method evidence. |
| `latency` | `{algorithm:nonempty,evidence:state<{kind:distribution|histogram|raw_samples,reference:ref}>,interval:lifecycle_interval,loss:state<{lost:u64,reason:nonempty}>,method:nonempty,population:u64,rounding:nonempty,unit:nanoseconds}`. Evidence is required for measured performance; explicit non-performance missing is permitted. Quantiles alone are invalid. |
| `cpu` | Closed `{system:metric,user:metric,wall:metric}`; nanoseconds and declared process/thread scope. |
| `allocations` | Closed `{bytes:metric,count:metric}`; units bytes and allocations. |
| `memory` | Closed `{cache:metric,peak_resident:metric,resident:metric,virtual:metric}`; byte units and declared sampling scope. |
| `io` | `[{bytes:metric,count:metric,layer:application|vfs|filesystem|block|device|other,operation:read|write}]`; displayed layer then operation order, every combination present or explicitly missing; methods distinguish requested/completed/failed. |
| `synchronization` | `{completed:metric,failed:metric,group_scope:state<nonempty>,primitive:nonempty,requested:metric,wait:metric}`; counts use operations and wait uses nanoseconds. |
| `amplification` | Closed `{read:metric,space:metric,write:metric}`; present values are ratios whose method fixes numerator, denominator, and scope; zero denominators are invalid. |
| `resource_measurements` | `[counter]`, sorted by `(source_name,scope,unit)` with no duplicate; this recommended extension never substitutes for required metric members. |
| `execution_observations` | Closed `{backpressure:metric,checkpoints:metric,compactions:metric,errors:metric,flushes:metric,partial_writes:metric,retries:metric,stalls:metric}`; units are events or nanoseconds as fixed by method and every category is present or explicitly missing. `background_work` is `[{count:u64,duration_ns:state<ns_u64>,kind:nonempty,source:source}]`, sorted by kind; `errors` has the same shape. |
| `visibility` | `{first_visible_monotonic_ns:state<ns_u64>,probe:nonempty,status:not_observed|observed|unsupported}`; status and state agree. |
| `fault_contract` | `state<{fault_plan_record_id:uuid,promised_fault_classes:[process_termination|kernel_crash|power_loss|resulting_condition|storage_error]}>`; enum order; present for fault phase. |
| `phase` | `{name:setup|warm_up|measured|fault|recovery|cleanup,observation_role:warm_up|measured|non_performance}`. |
| `sample_population` | `{included:u64,lost:u64,omission_reason:state<nonempty>,total:u64}`; included + lost = total and equals latency population where latency is measured. |
| `operations` | `[operation]`, sorted by `workload_ordinal`, unique. Empty only outside warm-up/measured/fault/recovery. |
| `recovery` | `state<{classification:clean|damaged_tail|corrupt|unrecoverable,corrupt:u64,duplicates:u64,invented:u64,missing:u64,recovered:u64,replay_ns:ns_u64,scan_ns:ns_u64,time_to_ready_ns:ns_u64,uncertain:u64}>`; present for recovery cells. |
| `correctness` | `{checks:[check],gate:pass|fail|inconclusive,oracle_artifact_id:uuid,oracle_version:nonempty}`; pass iff every required check passes. |
| `equivalence` | `{classification:equivalent_candidate|conditionally_equivalent|diagnostic|non_equivalent,conditions:[nonempty],reasons:[nonempty]}`; arrays sorted and unique. |
| `result_classification` | `{labels:[valid|invalid|failed|negative|inconclusive|diagnostic],reasons:[nonempty]}`; labels in enum order, unique; reasons sorted. |
| `deviations` | Same environment deviation object. |
| `artifacts` / `provenance` | `artifact_set` / `provenance_ref`. Together they make all logical artifacts and typed provenance edges immutable and substitution-detectable; no bare `artifact_ids` array is allowed. |
| `validation` | `{findings:[nonempty],integrity:[{algorithm:sha256,artifact_id:uuid,outcome:pass|fail}],status:inconclusive|not_validated|pass|fail,validated_at_utc_ns:state<ns_i64>,validator_configuration_ref:state<ref>,validator_identity:state<nonempty>,validator_version:state<nonempty>}`. Findings sort; integrity sorts by artifact ID. Pass requires present validator fields, nonempty integrity, every outcome pass, and empty findings. |

### Normative `benchmark-raw-result/v1` crosswalk

This table is exhaustive. Physical members may not be inferred from `operations`, free-form counters, or bare IDs.

| Logical field(s) | Exact physical location | Missing-state behavior |
|---|---|---|
| `schema.name`, `schema.version`, `result_id`, correction fields | envelope `record_kind`, `schema_version`, `record_id`, `supersedes_record_id`, `correction_reason` | Envelope states in section 1; never omitted. |
| Experiment, hypothesis, requirement, subject/profile/baseline/series/repository | same-named body fields plus envelope `series_id` | Only `baseline_id` and patch admit their declared states. |
| Environment, workload, adapter, configurations, platform, producer | `environment_ref`, `workload_ref`, `adapter_ref`, `configuration_refs`, `platform_contract_ref`, `producer_record` | Required digest-bearing references; no bare-ID or generic artifact substitute. |
| Run/repetition/phase/sample and observation role | envelope `run_id`; body `repetition_id`, `phase`, `sample_id` | Required; run present for raw results. |
| Durability/commit, operation and D3 coordinates | `d_mode`, `canonical_status`, `operations` | Conditional operation fields use their typed states. |
| General interval, time meanings, lifecycle interval | `interval`, `time_meanings`, `lifecycle_interval` | Required structures; individual inapplicable time meanings are explicit. |
| Operation counts | `operation_counts` eleven named members | Every count uses `state<u64>`; no omitted partition member. |
| Logical, encoded, and physical bytes | `logical_bytes`, `encoded_bytes`, `physical_bytes` | Every domain has a typed state. |
| Throughput and latency | `throughput`, `latency` | Rate/evidence values use typed states with retained method/population/loss metadata. |
| CPU, allocations, memory, I/O, synchronization, amplification | same-named closed body members | Every required leaf uses `metric.value`; unsupported/uncollected/missing remains explicit. |
| Resource measurements | `resource_measurements` | Empty means known none; never replaces required metrics. |
| Execution observations | `execution_observations`, `background_work`, `errors` | Eight named categories required; empty arrays mean observed none. |
| Correctness, faults, recovery, equivalence, result classification | same-named fields, with fault/oracle/D3 detail in referenced plan/evidence and operations | Conditional records use typed states; required classifications never omitted. |
| Artifacts and provenance edges | `artifacts`, `provenance` | Manifest and edge artifact references are always digest-bearing. |
| Validation status and integrity | `validation` | `not_validated`/`inconclusive` retain explicit absent-field reasons; pass/fail rules are closed above. |

`operation` is the closed object `{acknowledgement,assigned_sequence,byte_accounts,d3,durability_time_ns,effective_time_ns,error,event_id,lifecycle,observation_time_ns,operation_id,producer_id,request_id,system_time_ns,thread_id,workload_ordinal}`.

| Operation member | Type / rule |
|---|---|
| IDs/ordinal | `operation_id`, `request_id` are `uuid`; `event_id` is `state<uuid>`; `producer_id`, `thread_id` are nonempty; `workload_ordinal` is `u64`. |
| Times | Effective/system/durability/observation are `state<ns_i64>` and retain distinct meanings. |
| `assigned_sequence` | `state<u64>`; local replay sequence, not time. |
| `lifecycle` | `[{monotonic_ns:ns_u64,point:validation_start|validation_end|construction_start|construction_end|sequence_reserved|persistence_submitted|synchronization_start|synchronization_end|canonical_commit|acknowledgement|visibility_probe}]`; displayed point order, each at most once, nondecreasing. Missing applicable points add an operation error and make the result invalid rather than disappearing silently. |
| `acknowledgement` | `{boundary:nonempty,monotonic_ns:state<ns_u64>,status:acknowledged|failed|uncertain}`; time is present iff acknowledged. |
| `d3` | `state<{cut_reason:nonempty,eligible_member_ids:[uuid],group_id:uuid,member_ids:[uuid],shared_sync:pass|fail|uncertain,shared_sync_monotonic_ns:state<ns_u64>}>`; present iff D3; ID lists ascending and unique. |
| `byte_accounts` | `[{bytes:state<bytes_u64>,domain:encoded|logical_envelope|logical_key|logical_payload|logical_value|physical_checkpoint|physical_compaction|physical_database|physical_manifest|physical_other|physical_read|physical_sst|physical_synchronized|physical_temporary|physical_wal|physical_written,method:nonempty}]`; displayed domain order; every domain required. |
| `error` | `state<{code:nonempty,message:nonempty,retry_count:u64}>`. |

`counter` is the closed object `{enabled_ns:state<ns_u64>,end_raw:state<u64>,initial_raw:state<u64>,method:nonempty,multiplexed:state<boolean>,running_ns:state<ns_u64>,scope:process|thread|cgroup|cpu|device,source_name:nonempty,unit:allocations|bytes|context_switches|cpu_cycles|events|faults|instructions|io_operations|joules_nano|nanoseconds|sync_operations}`. Counters are unsigned totals; deltas require end >= initial unless referenced wrap evidence exists.

## 5. Remaining four record bodies

| Kind | Complete closed `body` |
|---|---|
| `artifact_manifest` | `{artifacts:[artifact],provenance_edges:[edge],publication_state:staged|published|superseded|expired|deleted,scope:series|run,series_freeze:state<{authority_artifact_ids:[uuid],environment_record_id:uuid,fault_contract_artifact_ids:[uuid],instrument_artifact_ids:[uuid],permitted_deviation_ids:[nonempty],profile_artifact_ids:[uuid],r8_authority_artifact_id:state<uuid>,subject_ids:[nonempty],validator_artifact_ids:[uuid],workload_manifest_artifact_id:uuid}>}`. All ID arrays ascending and unique, deviation/subject arrays UTF-8 ascending. `series_freeze` present for series scope and `not_applicable` for run scope. Artifact and edge ordering follows section 2. State transitions follow R7 section 5. |
| `fault_plan` | `{authorization_state:state<{authorization_artifact_id:uuid,authorized_by:nonempty,expires_at_utc_ns:ns_i64,scope:nonempty}>,contamination_controls:[nonempty],control_plane:state<nonempty>,d_mode:d0|d1|d2|d3,excluded_layers:[application|process|kernel|filesystem|controller|device|power],fault_class:process_termination|kernel_crash|power_loss|resulting_condition|storage_error,lifecycle_injection_point:nonempty,mechanism:state<nonempty>,mechanism_label:nonempty,oracle_obligations:[nonempty],plan_version:nonempty,preconditions:[nonempty],profile_id:nonempty,promised_layers:[application|process|kernel|filesystem|controller|device|power],restart_recovery:[nonempty],self_tests:[nonempty],trigger:state<nonempty>}`. All text arrays sort UTF-8; layer arrays follow displayed order; no overlap. Authorization `present` is necessary but never sufficient to execute. Unsupported apparatus uses the applicable missing state. |
| `fault_outcome` | `{apparatus_self_test:pass|fail|not_tested,armed_at_monotonic_ns:state<ns_u64>,classification:pass|fail|invalid|inconclusive,classification_reasons:[nonempty],contamination:state<nonempty>,fault_plan_record_id:uuid,not_tested_cells:[nonempty],observed_condition:state<nonempty>,oracle_artifact_id:uuid,placement_class:before_eligibility|eligible_not_committed|canonical_committed|unknown,recovery_artifact_ids:[uuid],trigger_evidence:state<ref>}`. Text and UUID arrays ascending and unique. `pass` requires passed self-test, present trigger evidence/condition, no contamination, and empty not-tested cells for the claimed contract. |
| `validation_report` | `{byte_length:bytes_u64,errors:[error],outcome:valid|invalid,profile_checks:[check],sha256:sha256,validated_artifact_id:uuid,validated_record_id:state<uuid>,validation_started_at_utc_ns:ns_i64,validator_identity:nonempty,validator_version:nonempty}`. `valid` requires empty errors and all checks pass; `invalid` requires at least one error or failed check. `validated_record_id` is present for a record artifact, otherwise `not_applicable`. A report cannot target itself. |

## 6. Closed publication and deletion controls

These are profile control records, not a seventh or eighth evidence `record_kind`. Each is a standalone JCS value, media type `application/vnd.rusty-data-os.exp1-r7+jcs`, with no BOM or surrounding bytes, and uses the record digest domain. They are artifacts with roles `configuration` (descriptor) and `deletion_evidence`; manifests reference them by immutable `ref`/artifact entry. Unknown fields and all ordinary parse/profile failures fail closed.

### Publication descriptor

Closed object: `{authorization,created_at_utc_ns,descriptor_id,generation,manifest,object_kind,predecessor_descriptor_sha256,run_id,schema_version,scope,series_id,validation_report_ids}`.

| Member | Rule |
|---|---|
| `object_kind` / `schema_version` | Exactly `publication_descriptor` / `EXP1-R7-JSON-JCS-1`. |
| `descriptor_id`, `series_id` | `uuid`; `run_id` is `state<uuid>`, present iff run scope. |
| `scope`, `created_at_utc_ns` | `series|run`; `ns_i64`. |
| `generation` | `u64`. Generation `0` requires predecessor `not_applicable`; generation `n>0` requires `present` predecessor and exactly predecessor generation + 1. |
| `predecessor_descriptor_sha256` | `state<sha256>`. A successor must preserve series/scope/run identity and name the exact previously authoritative descriptor digest. |
| `manifest` | `ref`; its bytes must be a valid matching-scope manifest in `published` state. |
| `validation_report_ids` | Ascending unique `[uuid]`, nonempty; each resolves to a valid report for the manifest/descriptor dependencies. |
| `authorization` | `{authorization_artifact_id:uuid,authorized_by:nonempty,scope:publish|replace}`; generation zero requires `publish`, successors `replace`. |

Only one valid successor of the current generation may become authoritative. Concurrent successors are a fork: neither replaces the current descriptor until a later authorized recovery publication explicitly names the selected predecessor under a new reviewed series policy. Discovery or atomic-write failure leaves the previous descriptor authoritative.

### Deletion-evidence control record

Closed object: `{authorization,completed_at_utc_ns,deletion_evidence_id,method,object_kind,outcome,reason,requested_at_utc_ns,schema_version,scope,target,verification}`.

| Member | Rule |
|---|---|
| `object_kind` / `schema_version` | Exactly `deletion_evidence` / `EXP1-R7-JSON-JCS-1`. |
| `deletion_evidence_id` | `uuid`, never reused. |
| `authorization` | `{authorization_artifact_id:uuid,authorized_at_utc_ns:ns_i64,authorized_by:nonempty,expires_at_utc_ns:ns_i64,scope:artifact|run|series}`; request/completion must be within the inclusive authorization interval. |
| `scope` | `{artifact_ids:[uuid],run_id:state<uuid>,series_id:uuid}`; artifact IDs ascending/nonempty; run is present for artifact/run scope and `not_applicable` for series scope. It must be a subset of authorization scope. |
| `target` | `[{artifact_id:uuid,byte_length:bytes_u64,prior_sha256:sha256,uri:uri}]`, sorted by artifact ID and exactly equal to `scope.artifact_ids`. |
| `requested_at_utc_ns`, `completed_at_utc_ns` | `ns_i64`; completion >= request. |
| `reason`, `method` | Nonempty text; method identifies the actual deletion primitive and verification procedure, not an intent. |
| `outcome` | `deleted|failed|inconclusive`. Only `deleted` permits a later manifest to mark the exact targets deleted. |
| `verification` | `{checked_at_utc_ns:ns_i64,checker:nonempty,findings:[nonempty],result:absent|present|partial|unverifiable}`; findings sorted. `deleted` requires `absent`; other combinations do not transition retention state. |

Deletion evidence is itself retained and cannot authorize its own deletion. A digest/length/identity mismatch, unresolved dependency, expired or wrong-scope authorization, `present`, `partial`, `unverifiable`, or inaccessible target is failure evidence, not proof of deletion. A retry creates a new identity and retains every attempt.
