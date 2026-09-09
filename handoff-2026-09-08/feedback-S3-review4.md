# Host dispositions on Codex review 4 of the Step 3 work order (plan sha256 f63d332d…6045)

UCR-011 (medium) accepted. R5 now defines `UCE1` completely and in order: version marker;
`request_id`; `event_id`; `sequence` (u64 LE); `durability_time` (i64 LE, the persisted
sample); requested durability byte; `source_descriptor_count` (u16) with `(u16 length, bytes)`
descriptors; `reference_count` (u16) with 16-byte event-id references; then the `UCR1`
normalized request bytes containing the payload. Both counts are the explicit empty
representation (`0`) in this bounded core, stated in the document. On recovery, `uc-core`
decodes every accepted `Final`'s envelope and fails closed with `EnvelopeMismatch {
physical_ordinal }` unless `request_id`, `event_id`, `sequence` and `durability_time` equal
the outer `Final`'s, `event_id` matches the adjacent `Commit`, and the `UCR1` bytes equal the
bound `Binding.normalized_request`; a unit test exercises each mismatch. Please re-review the
revised plan (new hash) for approval.
