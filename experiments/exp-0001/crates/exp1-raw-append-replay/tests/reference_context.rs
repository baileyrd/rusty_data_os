use exp1_raw_append_replay::reference_context::*;
use exp1_workload_conformance::{hex, manifest_digest_v2, sha256, workload_digest_v2};

const MANIFEST: &[u8] =
    include_bytes!("../../exp1-workload-conformance/tests/data/r26-v2/manifest.jcs");
const WS_HEX: &str = include_str!("../../exp1-workload-conformance/tests/data/r26-v2/ws.hex");
const NS: [u8; 16] = [0x25, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 1];

fn decode_hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn metadata(id: &str, bytes: &[u8], role: &str, media: &str) -> Vec<u8> {
    format!(concat!(
        "{{\"body\":{{\"artifacts\":[{{\"artifact_id\":\"{}\",\"byte_length\":\"{}\",",
        "\"created_by_record_id\":\"16000000-0000-4000-8000-000000000006\",",
        "\"logical_path\":\"exp-0001/artifacts/{}\",\"media_type\":\"{}\",\"retention_state\":\"published\",",
        "\"role\":\"{}\",\"sensitivity\":\"public\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/{}\",\"validation_report_ids\":[]}}],",
        "\"provenance_edges\":[],\"publication_state\":\"published\",\"scope\":\"run\",\"series_freeze\":{{\"state\":\"not_applicable\"}}}},",
        "\"correction_reason\":{{\"state\":\"not_applicable\"}},\"created_at_utc_ns\":\"1788134400000000000\",",
        "\"record_id\":\"16000000-0000-4000-8000-000000000010\",\"record_kind\":\"artifact_manifest\",",
        "\"run_id\":{{\"state\":\"present\",\"value\":\"16000000-0000-4000-8000-000000000008\"}},",
        "\"schema_version\":\"EXP1-R7-JSON-JCS-1\",\"series_id\":\"16000000-0000-4000-8000-000000000007\",",
        "\"supersedes_record_id\":{{\"state\":\"not_applicable\"}}}}"),
        id, bytes.len(), id, media, role, hex(&sha256(bytes)), id).into_bytes()
}

struct Fixture {
    ws: Vec<u8>,
    descriptor: Vec<u8>,
    scope_digest: Vec<u8>,
    manifest_digest: Vec<u8>,
    manifest_meta: Vec<u8>,
    stream_meta: Vec<u8>,
}

fn fixture() -> Fixture {
    let ws = decode_hex(WS_HEX);
    let descriptor = format!(
        concat!(
            "{{\"cell_id\":\"PC-D1-raw-v2\",\"members\":[{{",
            "\"manifest_digest\":\"{}\",\"manifest_id\":\"16000000-0000-4000-8000-000000000001\",",
            "\"stream_artifact_sha256\":\"{}\",\"stream_byte_length\":\"{}\",",
            "\"stream_digest\":\"{}\",\"stream_namespace\":\"25000000-0000-4000-8000-000000000001\",",
            "\"workload_id\":\"16000000-0000-4000-8000-000000000003\"}}],",
            "\"record_kind\":\"closed_stream_scope\",\"schema_version\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v2\",",
            "\"scope_id\":\"27000000-0000-4000-8000-000000000001\"}}"
        ),
        hex(&manifest_digest_v2(MANIFEST)), hex(&sha256(&ws)), ws.len(),
        hex(&workload_digest_v2(&ws)),
    ).into_bytes();
    let scope_value = {
        let mut p = b"rusty-data-os/exp1/closed-stream-scope/v2\0".to_vec();
        p.extend(&descriptor);
        hex(&sha256(&p))
    };
    let scope_digest = format!(concat!(
        "{{\"algorithm\":\"SHA-256/FIPS-180-4\",\"domain\":\"rusty-data-os/exp1/closed-stream-scope/v2\",",
        "\"profile\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v2\",\"scope_ref\":{{\"artifact_id\":\"27000000-0000-4000-8000-000000000002\",\"byte_length\":\"{}\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/scope.jcs\"}},\"value\":\"{}\"}}"
    ), descriptor.len(), hex(&sha256(&descriptor)), scope_value).into_bytes();
    let manifest_digest = format!(
        "{{\"algorithm\":\"SHA-256/FIPS-180-4\",\"domain\":\"rusty-data-os/exp1/workload-manifest/v2\",\"manifest_ref\":{{\"artifact_id\":\"16000000-0000-4000-8000-000000000001\",\"byte_length\":\"{}\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/16000000-0000-4000-8000-000000000001\"}},\"profile\":\"EXP-0001-WORKLOAD-MANIFEST-DIGEST-v2\",\"value\":\"{}\"}}",
        MANIFEST.len(), hex(&sha256(MANIFEST)), hex(&manifest_digest_v2(MANIFEST))
    ).into_bytes();
    let manifest_meta = metadata(
        "16000000-0000-4000-8000-000000000001",
        MANIFEST,
        "workload_manifest",
        "application/vnd.rusty-data-os.exp1-workload-manifest+jcs",
    );
    let stream_meta = metadata(
        "16000000-0000-4000-8000-000000000002",
        &ws,
        "configuration",
        "application/vnd.rusty-data-os.exp1-workload-stream",
    );
    Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    }
}

fn context() -> (ReferenceContextV2, Vec<Vec<u8>>) {
    let Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    let context = construct_reference_context_v2(
        ClosedScopeInputV2 {
            scope: ScopeDigestInput {
                descriptor: &descriptor,
                artifact_metadata: &scope_digest,
            },
            members: &[binding],
        },
        NS,
    )
    .unwrap();
    let mut position = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2".len() + 24;
    let mut operations = Vec::new();
    for _ in 0..4 {
        let length = u64::from_be_bytes(ws[position..position + 8].try_into().unwrap()) as usize;
        position += 8;
        operations.push(ws[position..position + length].to_vec());
        position += length;
    }
    (context, operations)
}

#[test]
fn r26_literals_construct_an_immutable_catalog_and_initial_state() {
    let Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    let context = construct_reference_context_v2(
        ClosedScopeInputV2 {
            scope: ScopeDigestInput {
                descriptor: &descriptor,
                artifact_metadata: &scope_digest,
            },
            members: &[binding],
        },
        NS,
    )
    .unwrap();
    assert_eq!(context.catalog().stream_count(), 1);
    assert_eq!(context.catalog().operation_count(), 4);
    assert_eq!(context.catalog().identity_entry_count(), 12);
    assert_eq!(context.catalog().source_bytes(), 3139);
    assert_eq!(context.initial_state().accepted_operations(), 0);
}

#[test]
fn construction_is_closed_and_digest_bound() {
    let Fixture {
        ws,
        descriptor,
        mut scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    *scope_digest.last_mut().unwrap() = b'!';
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &descriptor,
                    artifact_metadata: &scope_digest
                },
                members: &[binding]
            },
            NS
        )
        .unwrap_err(),
        ContextConstructionError::InvalidScopeDigest
    );
}

#[test]
fn both_segment_bootstraps_and_prior_references_map_transactionally() {
    let (context, operations) = context();
    let mut state = context.initial_state().clone();
    for (index, operation) in operations.iter().enumerate() {
        let before = state.clone();
        let mapped = map_semantic_operation_v2_with_context(
            operation,
            (index + 1) as u64,
            (index + 1) as u64,
            context.catalog(),
            &state,
        )
        .unwrap();
        assert_eq!(state, before);
        assert_eq!(
            mapped.next_state().accepted_operations(),
            (index + 1) as u64
        );
        assert_eq!(mapped.next_state().previous_sequence(), (index + 1) as u64);
        assert_eq!(
            mapped.next_state().previous_physical_ordinal(),
            (index + 1) as u64
        );
        assert_eq!(
            exp1_record_format::decode(mapped.frame()).unwrap(),
            *mapped.record()
        );
        state = mapped.next_state().clone();
    }
    assert_eq!(
        map_semantic_operation_v2_with_context(&operations[3], 5, 5, context.catalog(), &state)
            .unwrap_err(),
        ContextualMappingError::Exhaustion
    );
}
