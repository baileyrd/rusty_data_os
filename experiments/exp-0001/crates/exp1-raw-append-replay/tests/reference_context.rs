//! R24 executable correctness gate. Fixtures are synthetic authority-conformance inputs.

use exp1_raw_append_replay::{mapping::*, reference_context::*};
use exp1_workload_conformance::*;
#[allow(dead_code)]
mod vectors {
    pub const S01: &str = "52444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e000000010002000000164558502d303030312d5348413235362d4354522d763103000000209a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f04000000184558502d303030312d55554944342d5348413235362d76310500000010cf79754651a34f76b1718244bf8053db0600000010330f201aea7c4335a8ece6fe23266a1c07000000103d3c52813d4347db8825664f324e091d080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000013952444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee0500000001310600000001000700000001000800000010cf79754651a34f76b1718244bf8053db0900000010330f201aea7c4335a8ece6fe23266a1c0a000000103d3c52813d4347db8825664f324e091d0b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a";
    pub const S02: &str = "52444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e000000010002000000164558502d303030312d5348413235362d4354522d76310300000020701866aac4b5cfd4db8974593a0e4b9db5e5879a12cdda727f27885badb7696404000000184558502d303030312d55554944342d5348413235362d76310500000010fcec18c95b8e4655a5ce5463a7872f850600000010c57a25cf26e64dbaad56ea7cec2a4865070000001080f644cff28f432ab64174dfcc5cf873080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000014852444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee050000000131060000000901736f757263652d410700000008016163746f722d410800000010fcec18c95b8e4655a5ce5463a7872f850900000010c57a25cf26e64dbaad56ea7cec2a48650a0000001080f644cff28f432ab64174dfcc5cf8730b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a";
    pub const W00: &str = "52444f532d5753314558502d303030312d53454d414e5449432d4f502d7631000000000000000000000000000000000000000000000000";
    pub const W01: &str = "52444f532d5753314558502d303030312d53454d414e5449432d4f502d763100000000000000020000000000000002000000000000000000000000000002f352444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e000000010002000000164558502d303030312d5348413235362d4354522d763103000000209a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f04000000184558502d303030312d55554944342d5348413235362d76310500000010cf79754651a34f76b1718244bf8053db0600000010330f201aea7c4335a8ece6fe23266a1c07000000103d3c52813d4347db8825664f324e091d080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000013952444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000000600000001010700000004000000200800000001010900000001010a00000001010b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000000e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee0500000001310600000001000700000001000800000010cf79754651a34f76b1718244bf8053db0900000010330f201aea7c4335a8ece6fe23266a1c0a000000103d3c52813d4347db8825664f324e091d0b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a000000000000030252444f532d534f5031000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e000000010002000000164558502d303030312d5348413235362d4354522d76310300000020701866aac4b5cfd4db8974593a0e4b9db5e5879a12cdda727f27885badb7696404000000184558502d303030312d55554944342d5348413235362d76310500000010fcec18c95b8e4655a5ce5463a7872f850600000010c57a25cf26e64dbaad56ea7cec2a4865070000001080f644cff28f432ab64174dfcc5cf873080000001a4558502d303030312d454e56454c4f50452d494e5055542d7631090000014852444f532d454e5631000d010000009652444f532d4f5031000e010000000200010200000002000103000000010004000000080000000000000000050000000800000000000000010600000001010700000004000000200800000001010900000001020a00000001020b0000001000112233445546778899aabbccddeeff0c000000101021324354654768899aabbccddeef000d0000000800000000000000010e00000001000200000001310300000006666163742d410400000010eeeeeeeeeeee4eee8eeeeeeeeeeeeeee050000000131060000000901736f757263652d410700000008016163746f722d410800000010fcec18c95b8e4655a5ce5463a7872f850900000010c57a25cf26e64dbaad56ea7cec2a48650a0000001080f644cff28f432ab64174dfcc5cf8730b0000000800000000000003e80c00000001000d00000004000000000a000000184558502d303030312d5052494f522d4556454e54532d76310b000000184558502d303030312d4c4f474943414c2d54494d452d76310c0000000800000000000003e80d00000008000000000000000a";
    pub const M01: &str = r#"{"authority_revisions":[{"authority":"EXP-0000-WORKLOADS","revision":{"kind":"git_sha","value":"70a29efd46dd3aee9ea9cb0831d0285b83cdd70a"}},{"authority":"EXP-0001-R12","revision":{"kind":"git_sha","value":"e39551e64d9a799a3d15bf75aa70a323c8e40ca8"}},{"authority":"EXP-0001-R14","revision":{"kind":"git_sha","value":"78b8b35e4efda44a8097db05f396679a1265a239"}},{"authority":"EXP-0001-R16","revision":{"kind":"reviewed_authority_id","value":"documentation-vector-v1"}},{"authority":"EXP-0001-R2","revision":{"kind":"git_sha","value":"2659fb34caf054a7742a854d69d17cdd59bd2040"}},{"authority":"EXP-0001-R7","revision":{"kind":"git_sha","value":"f9d9876cf6599345a2e2244223a530ada2b9a828"}}],"counts":{"by_envelope_profile":[{"count":"1","profile":"envelope-minimal"}],"by_segment":[{"count":"1","segment":"warm_up"},{"count":"0","segment":"measured"}],"by_size_class":[{"count":"1","profile":"P1"}],"by_temporal_profile":[{"count":"1","profile":"time-monotonic-effective"}],"measured_operation_count":"0","operation_count":"1","warm_up_operation_count":"1"},"created_at_utc_ns":"1788134400000000000","generator_inputs":{"actor_provenance":{"state":"not_applicable"},"base_ns":"1000","controlled_schedule":{"state":"not_applicable"},"correction_fact_type":{"state":"not_applicable"},"envelope_semantic_version":"1","generator_version":"1","ordinary_fact_type":"fact-A","producer_count":"1","producer_id":"10213243-5465-4768-899a-abbccddeef00","reference_cardinality":"0","schema_id":"eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee","schema_version":"1","seed":"0","source_provenance":{"state":"not_applicable"},"stream_namespace":"00112233-4455-4677-8899-aabbccddeeff","unit_ns":"10","workload_contract_version":"1"},"manifest_id":"16000000-0000-4000-8000-000000000001","profiles":{"digest":"SHA-256/FIPS-180-4","envelope":"envelope-minimal","envelope_generator":"EXP-0001-ENVELOPE-INPUT-v1","identity_generator":"EXP-0001-UUID4-SHA256-v1","logical_time_generator":"EXP-0001-LOGICAL-TIME-v1","manifest":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","payload_content":"deterministic-high-variation","payload_generator":"EXP-0001-SHA256-CTR-v1","payload_size":"fixed-P1","reference_generator":"EXP-0001-PRIOR-EVENTS-v1","semantic_operation":"EXP-0001-SEMANTIC-OP-v1","size_class_order":"EXP-0000-SIZE-CLASS-ORDER-v1","temporal":"time-monotonic-effective","workload_contract":"EXP-0000-WORKLOADS-v1","workload_stream":"EXP-0001-WORKLOAD-STREAM-v1"},"record_kind":"workload_manifest","schema_version":"EXP-0001-WORKLOAD-MANIFEST-JCS-v1","stream_digest":{"algorithm":"SHA-256/FIPS-180-4","domain":"rusty-data-os/exp1/workload-stream/v1","value":"0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09"},"stream_ref":{"artifact_id":"16000000-0000-4000-8000-000000000002","artifact_manifest_ref":{"artifact_id":"16000000-0000-4000-8000-000000000009","byte_length":"1274","sha256":"b65688eb056a71bacaff1178ef4d0693b1c5ef59c43bdbdaa7b360e562f4998c","uri":"https://example.invalid/exp-0001/artifact-manifest.jcs"},"byte_length":"818","created_by_record_id":"16000000-0000-4000-8000-000000000005","media_type":"application/vnd.rusty-data-os.exp1-workload-stream","role":"configuration","sha256":"789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a","uri":"https://example.invalid/exp-0001/s01.stream"},"supersession":{"reason":{"state":"not_applicable"},"supersedes_manifest_ids":[]},"workload_id":"16000000-0000-4000-8000-000000000003"}"#;

    pub const R7_ARTIFACT_MANIFEST: &[u8] = br#"{"body":{"artifacts":[{"artifact_id":"16000000-0000-4000-8000-000000000002","byte_length":"818","created_by_record_id":"16000000-0000-4000-8000-000000000005","logical_path":"exp-0001/series/16000000-0000-4000-8000-000000000007/runs/16000000-0000-4000-8000-000000000008/artifacts/16000000-0000-4000-8000-000000000002/configuration","media_type":"application/vnd.rusty-data-os.exp1-workload-stream","retention_state":"published","role":"configuration","sensitivity":"public","sha256":"789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a","uri":"https://example.invalid/exp-0001/s01.stream","validation_report_ids":[]}],"provenance_edges":[{"from_artifact_id":"16000000-0000-4000-8000-000000000002","relation":"generated_from","to_artifact_id":"16000000-0000-4000-8000-000000000001"}],"publication_state":"published","scope":"run","series_freeze":{"state":"not_applicable"}},"correction_reason":{"state":"not_applicable"},"created_at_utc_ns":"1788134400000000000","record_id":"16000000-0000-4000-8000-000000000004","record_kind":"artifact_manifest","run_id":{"state":"present","value":"16000000-0000-4000-8000-000000000008"},"schema_version":"EXP1-R7-JSON-JCS-1","series_id":"16000000-0000-4000-8000-000000000007","supersedes_record_id":{"state":"not_applicable"}}"#;

    pub const R7_WORKLOAD_ARTIFACT_MANIFEST: &[u8] = br#"{"body":{"artifacts":[{"artifact_id":"16000000-0000-4000-8000-000000000001","byte_length":"3423","created_by_record_id":"16000000-0000-4000-8000-000000000006","logical_path":"exp-0001/series/16000000-0000-4000-8000-000000000007/runs/16000000-0000-4000-8000-000000000008/artifacts/16000000-0000-4000-8000-000000000001/workload_manifest","media_type":"application/vnd.rusty-data-os.exp1-workload-manifest+jcs","retention_state":"published","role":"workload_manifest","sensitivity":"public","sha256":"ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86","uri":"https://example.invalid/exp-0001/m01.manifest.jcs","validation_report_ids":[]}],"provenance_edges":[],"publication_state":"published","scope":"run","series_freeze":{"state":"not_applicable"}},"correction_reason":{"state":"not_applicable"},"created_at_utc_ns":"1788134400000000000","record_id":"16000000-0000-4000-8000-000000000010","record_kind":"artifact_manifest","run_id":{"state":"present","value":"16000000-0000-4000-8000-000000000008"},"schema_version":"EXP1-R7-JSON-JCS-1","series_id":"16000000-0000-4000-8000-000000000007","supersedes_record_id":{"state":"not_applicable"}}"#;
}

fn leak<T>(v: T) -> &'static T {
    Box::leak(Box::new(v))
}
fn text(v: String) -> &'static str {
    Box::leak(v.into_boxed_str())
}
fn bytes(v: Vec<u8>) -> &'static [u8] {
    Box::leak(v.into_boxed_slice())
}

fn fixture() -> (ClosedScopeInput<'static>, [u8; 16], &'static [u8]) {
    let operation = decode_hex(vectors::S01).unwrap();
    let stream = bytes(workload_stream(&[operation], 1, 0).unwrap());
    let manifest = vectors::M01.as_bytes();
    let descriptor = leak(ManifestDigestDescriptor {
        algorithm: "SHA-256/FIPS-180-4",
        domain: MANIFEST_DOMAIN,
        profile: "EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1",
        value: text(hex(&manifest_digest(manifest))),
        manifest_ref: ManifestReference {
            artifact_id: "16000000-0000-4000-8000-000000000001",
            byte_length: manifest.len() as u64,
            sha256: text(hex(&artifact_digest(manifest))),
            uri: "https://example.invalid/exp-0001/m01.manifest.jcs",
        },
    });
    let stream_artifact = leak(ArtifactMetadata {
        artifact_id: "16000000-0000-4000-8000-000000000002",
        byte_length: stream.len() as u64,
        sha256: text(hex(&artifact_digest(stream))),
        uri: "https://example.invalid/exp-0001/s01.stream",
        role: "configuration",
        media_type: "application/vnd.rusty-data-os.exp1-workload-stream",
        created_by_record_id: "16000000-0000-4000-8000-000000000005",
    });
    let wam_ref = leak(ManifestReference {
        artifact_id: "16000000-0000-4000-8000-000000000011",
        byte_length: vectors::R7_WORKLOAD_ARTIFACT_MANIFEST.len() as u64,
        sha256: text(hex(&artifact_digest(
            vectors::R7_WORKLOAD_ARTIFACT_MANIFEST,
        ))),
        uri: "https://example.invalid/exp-0001/workload-artifact-manifest.jcs",
    });
    let validation = leak(ValidationContext {
        stream,
        descriptor,
        manifest_artifact_sha256: descriptor.manifest_ref.sha256,
        targets: &[],
        artifact_manifest_bytes: vectors::R7_ARTIFACT_MANIFEST,
        workload_artifact_manifest_bytes: vectors::R7_WORKLOAD_ARTIFACT_MANIFEST,
        workload_artifact_manifest_ref: wam_ref,
        stream_artifact,
    });
    let mut provenance = Vec::new();
    provenance.extend((vectors::R7_ARTIFACT_MANIFEST.len() as u64).to_be_bytes());
    provenance.extend(vectors::R7_ARTIFACT_MANIFEST);
    provenance.extend((vectors::R7_WORKLOAD_ARTIFACT_MANIFEST.len() as u64).to_be_bytes());
    provenance.extend(vectors::R7_WORKLOAD_ARTIFACT_MANIFEST);
    let provenance = bytes(provenance);
    let ns = parse_uuid("00112233-4455-4677-8899-aabbccddeeff").unwrap();
    let member = leak(ClosedScopeMemberInput {
        stream_namespace: ns,
        workload_id: "16000000-0000-4000-8000-000000000003",
        manifest_id: "16000000-0000-4000-8000-000000000001",
        cell_id: "PC-D1-B1-F1",
        stream,
        manifest,
        manifest_validation: validation,
        resolved_metadata_bytes: provenance,
    });
    let scope_descriptor = text(format!(
        "{{\"cell_id\":\"PC-D1-B1-F1\",\"members\":[{{\"manifest_digest\":\"{}\",\"manifest_id\":\"16000000-0000-4000-8000-000000000001\",\"stream_artifact_sha256\":\"{}\",\"stream_byte_length\":\"{}\",\"stream_digest\":\"{}\",\"stream_namespace\":\"00112233-4455-4677-8899-aabbccddeeff\",\"workload_id\":\"16000000-0000-4000-8000-000000000003\"}}],\"record_kind\":\"closed_stream_scope\",\"schema_version\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v1\",\"scope_id\":\"24000000-0000-4000-8000-000000000001\"}}",
        descriptor.value,
        hex(&artifact_digest(stream)),
        stream.len(),
        hex(&workload_digest(stream))
    ));
    let scope_bytes = scope_descriptor.as_bytes();
    let scope_sha = text(hex(&artifact_digest(scope_bytes)));
    let scope_r7 = text(format!(
        "{{\"body\":{{\"artifacts\":[{{\"artifact_id\":\"24000000-0000-4000-8000-000000000002\",\"byte_length\":\"{}\",\"created_by_record_id\":\"24000000-0000-4000-8000-000000000003\",\"logical_path\":\"exp-0001/scopes/24000000-0000-4000-8000-000000000002/configuration\",\"media_type\":\"application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs\",\"retention_state\":\"published\",\"role\":\"configuration\",\"sensitivity\":\"public\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/scope\",\"validation_report_ids\":[]}}],\"provenance_edges\":[],\"publication_state\":\"published\",\"scope\":\"run\",\"series_freeze\":{{\"state\":\"not_applicable\"}}}},\"correction_reason\":{{\"state\":\"not_applicable\"}},\"created_at_utc_ns\":\"1788134400000000000\",\"record_id\":\"24000000-0000-4000-8000-000000000003\",\"record_kind\":\"artifact_manifest\",\"run_id\":{{\"state\":\"present\",\"value\":\"24000000-0000-4000-8000-000000000005\"}},\"schema_version\":\"EXP1-R7-JSON-JCS-1\",\"series_id\":\"24000000-0000-4000-8000-000000000004\",\"supersedes_record_id\":{{\"state\":\"not_applicable\"}}}}",
        scope_bytes.len(),
        scope_sha
    ));
    let mut di = b"rusty-data-os/exp1/closed-stream-scope/v1\0".to_vec();
    di.extend(scope_bytes);
    let digest = text(hex(&sha256(&di)));
    let members: &'static [ClosedScopeMemberInput<'static>] =
        Box::leak(vec![member.clone()].into_boxed_slice());
    (
        ClosedScopeInput {
            descriptor: scope_bytes,
            scope_digest: ScopeDigestDescriptor {
                algorithm: "SHA-256/FIPS-180-4",
                domain: "rusty-data-os/exp1/closed-stream-scope/v1",
                profile: "EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v1",
                value: digest,
                scope_ref: ScopeReference {
                    artifact_id: "24000000-0000-4000-8000-000000000002",
                    byte_length: scope_bytes.len() as u64,
                    sha256: scope_sha,
                    uri: "https://example.invalid/scope",
                },
            },
            scope_artifact: ScopeArtifactMetadata {
                artifact_id: "24000000-0000-4000-8000-000000000002",
                byte_length: scope_bytes.len() as u64,
                sha256: scope_sha,
                uri: "https://example.invalid/scope",
                role: "configuration",
                media_type: "application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs",
                created_by_record_id: "24000000-0000-4000-8000-000000000003",
                metadata_bytes: scope_r7.as_bytes(),
            },
            members,
        },
        ns,
        stream,
    )
}

fn first_operation(stream: &[u8]) -> &[u8] {
    let n = u64::from_be_bytes(stream[55..63].try_into().unwrap()) as usize;
    &stream[63..63 + n]
}

fn rebind_scope(input: &mut ClosedScopeInput<'static>, descriptor: &'static [u8]) {
    input.descriptor = descriptor;
    let sha = text(hex(&artifact_digest(descriptor)));
    input.scope_digest.scope_ref.byte_length = descriptor.len() as u64;
    input.scope_digest.scope_ref.sha256 = sha;
    input.scope_artifact.byte_length = descriptor.len() as u64;
    input.scope_artifact.sha256 = sha;
    let mut domain = b"rusty-data-os/exp1/closed-stream-scope/v1\0".to_vec();
    domain.extend(descriptor);
    input.scope_digest.value = text(hex(&sha256(&domain)));
    input.scope_artifact.metadata_bytes = text(format!(
        "{{\"body\":{{\"artifacts\":[{{\"artifact_id\":\"24000000-0000-4000-8000-000000000002\",\"byte_length\":\"{}\",\"created_by_record_id\":\"24000000-0000-4000-8000-000000000003\",\"logical_path\":\"exp-0001/scopes/24000000-0000-4000-8000-000000000002/configuration\",\"media_type\":\"application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs\",\"retention_state\":\"published\",\"role\":\"configuration\",\"sensitivity\":\"public\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/scope\",\"validation_report_ids\":[]}}],\"provenance_edges\":[],\"publication_state\":\"published\",\"scope\":\"run\",\"series_freeze\":{{\"state\":\"not_applicable\"}}}},\"correction_reason\":{{\"state\":\"not_applicable\"}},\"created_at_utc_ns\":\"1788134400000000000\",\"record_id\":\"24000000-0000-4000-8000-000000000003\",\"record_kind\":\"artifact_manifest\",\"run_id\":{{\"state\":\"present\",\"value\":\"24000000-0000-4000-8000-000000000005\"}},\"schema_version\":\"EXP1-R7-JSON-JCS-1\",\"series_id\":\"24000000-0000-4000-8000-000000000004\",\"supersedes_record_id\":{{\"state\":\"not_applicable\"}}}}",
        descriptor.len(), sha
    )).as_bytes();
}

fn set_member_cell(input: &mut ClosedScopeInput<'static>, cell: &'static str) {
    let mut member = input.members[0].clone();
    member.cell_id = cell;
    input.members = Box::leak(vec![member].into_boxed_slice());
}

#[test]
fn valid_construction_mapping_and_exactly_once_advancement() {
    let (input, ns, stream) = fixture();
    let context = construct_reference_context(input, ns).unwrap();
    assert_eq!(context.catalog().stream_count(), 1);
    let before = context.initial_state().clone();
    let catalog = context.catalog().clone();
    let mapped =
        map_semantic_operation_with_context(first_operation(stream), 1, 1, &catalog, &before)
            .unwrap();
    assert_eq!(before.accepted_count(), 0);
    assert_eq!(mapped.next_state().accepted_count(), 1);
    assert_eq!(mapped.next_state().previous_sequence(), 1);
    assert_eq!(catalog, context.catalog().clone());
    assert_eq!(
        map_semantic_operation_with_context(
            first_operation(stream),
            2,
            2,
            &catalog,
            mapped.next_state()
        ),
        Err(ContextualMappingError::Exhaustion)
    );
}
#[test]
fn caller_cannot_invent_cell_authority() {
    let (mut input, ns, _) = fixture();
    let changed = text(
        String::from_utf8(input.descriptor.to_vec())
            .unwrap()
            .replace("PC-D1-B1-F1", "invented"),
    );
    rebind_scope(&mut input, changed.as_bytes());
    set_member_cell(&mut input, "invented");
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidCellAuthority)
    );
}

#[test]
fn foreign_workload_and_foreign_cell_are_distinct_from_member_binding_failure() {
    let (mut input, ns, _) = fixture();
    let mut member = input.members[0].clone();
    member.workload_id = "26000000-0000-4000-8000-000000000003";
    input.members = Box::leak(vec![member].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ForeignWorkloadOrCell)
    );

    let (mut input, ns, _) = fixture();
    set_member_cell(&mut input, "PC-D1-B1-F2");
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ForeignWorkloadOrCell)
    );

    let (mut input, ns, _) = fixture();
    let mut member = input.members[0].clone();
    member.manifest_id = "26000000-0000-4000-8000-000000000001";
    input.members = Box::leak(vec![member].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidMemberBinding)
    );
}

#[test]
fn exact_r8_registry_edges_are_enforced() {
    for cell in ["PC-D0-B0-F1", "PC-D1-B3-MW", "PC-D2-B2-ME", "PC-D3-B1-F3"] {
        let (mut input, ns, _) = fixture();
        let changed = text(
            String::from_utf8(input.descriptor.to_vec())
                .unwrap()
                .replace("PC-D1-B1-F1", cell),
        );
        rebind_scope(&mut input, changed.as_bytes());
        set_member_cell(&mut input, cell);
        assert!(
            construct_reference_context(input, ns).is_ok(),
            "allowed {cell}"
        );
    }
    for cell in ["PC-D0-B1-F1", "PC-D3-B2-F1", "PC-D1-B4-F1", "PC-D1-B1-F4"] {
        let (mut input, ns, _) = fixture();
        let changed = text(
            String::from_utf8(input.descriptor.to_vec())
                .unwrap()
                .replace("PC-D1-B1-F1", cell),
        );
        rebind_scope(&mut input, changed.as_bytes());
        set_member_cell(&mut input, cell);
        assert_eq!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::InvalidCellAuthority),
            "disallowed {cell}"
        );
    }
}

#[test]
fn scope_r7_closed_schema_jcs_and_provenance_rejections() {
    let mutations = [
        ("\"scope\":\"run\"", "\"scope\":\"run\",\"unknown\":true"),
        (
            "\"provenance_edges\":[]",
            "\"provenance_edges\":[{\"from_artifact_id\":\"24000000-0000-4000-8000-000000000002\",\"relation\":\"generated_from\",\"to_artifact_id\":\"24000000-0000-4000-8000-000000000002\"}]",
        ),
        (
            "\"publication_state\":\"published\"",
            "\"publication_state\":\"published\",\"publication_state\":\"published\"",
        ),
        (
            "\"role\":\"configuration\"",
            "\"role\":\"configuration\",\"role\":\"capture\"",
        ),
    ];
    for (from, to) in mutations {
        let (mut input, ns, _) = fixture();
        input.scope_artifact.metadata_bytes = text(
            String::from_utf8(input.scope_artifact.metadata_bytes.to_vec())
                .unwrap()
                .replace(from, to),
        )
        .as_bytes();
        assert_eq!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::ScopeReferenceFailure)
        );
    }
    let (mut input, ns, _) = fixture();
    let s = String::from_utf8(input.scope_artifact.metadata_bytes.to_vec()).unwrap();
    input.scope_artifact.metadata_bytes = text(s.replacen(
        "\"body\":",
        "\"record_kind\":\"artifact_manifest\",\"body\":",
        1,
    ))
    .as_bytes();
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ScopeReferenceFailure)
    );
}

#[test]
fn scope_r7_rejects_unescaped_or_injected_uri_values() {
    for bad_uri in [
        "https://example.invalid/\"scope",
        "https://example.invalid/\\scope",
        "https://example.invalid/scope\u{1f}",
        "https://example.invalid/scope\u{7f}",
        "https://example.invalid/scope\",\"unknown\":true,\"x\":\"",
    ] {
        let (mut input, ns, _) = fixture();
        input.scope_digest.scope_ref.uri = text(bad_uri.to_owned());
        input.scope_artifact.uri = text(bad_uri.to_owned());
        input.scope_artifact.metadata_bytes = bytes(
            String::from_utf8(input.scope_artifact.metadata_bytes.to_vec())
                .unwrap()
                .replace("https://example.invalid/scope", bad_uri)
                .into_bytes(),
        );
        assert_eq!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::ScopeReferenceFailure),
            "URI {bad_uri:?} must not become canonical authority evidence"
        );
    }
}

#[test]
fn member_r7_semantics_are_revalidated_after_consistent_wrapper_updates() {
    for replacement in [
        (
            "\"publication_state\":\"published\"",
            "\"publication_state\":\"staged\"",
        ),
        (
            "\"provenance_edges\":[",
            "\"unknown\":true,\"provenance_edges\":[",
        ),
    ] {
        let (mut input, ns, _) = fixture();
        let mut member = input.members[0].clone();
        let bad = bytes(
            String::from_utf8(member.manifest_validation.artifact_manifest_bytes.to_vec())
                .unwrap()
                .replace(replacement.0, replacement.1)
                .into_bytes(),
        );
        let mut validation = member.manifest_validation.clone();
        validation.artifact_manifest_bytes = bad;
        member.manifest_validation = leak(validation);
        let other = member.manifest_validation.workload_artifact_manifest_bytes;
        let mut wrapper = Vec::new();
        wrapper.extend((bad.len() as u64).to_be_bytes());
        wrapper.extend(bad);
        wrapper.extend((other.len() as u64).to_be_bytes());
        wrapper.extend(other);
        member.resolved_metadata_bytes = bytes(wrapper);
        input.members = Box::leak(vec![member].into_boxed_slice());
        assert!(matches!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::SemanticValidation(_))
        ));
    }
}
#[test]
fn missing_or_malformed_r7_evidence_fails_closed() {
    let (mut input, ns, _) = fixture();
    input.scope_artifact.metadata_bytes = b"";
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ScopeReferenceFailure)
    );
    let (mut input, ns, _) = fixture();
    let mut member = input.members[0].clone();
    member.resolved_metadata_bytes = b"";
    input.members = Box::leak(vec![member].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidMemberBinding)
    );
}
#[test]
fn descriptor_uuid_and_ijson_invariants() {
    for bad in [
        "00000000-0000-0000-0000-000000000000",
        "24000000-0000-4000-0000-000000000001",
        "24000000-0000-4000-8000-00000000000A",
    ] {
        let (mut input, ns, _) = fixture();
        input.descriptor = bytes(
            String::from_utf8(input.descriptor.to_vec())
                .unwrap()
                .replace("24000000-0000-4000-8000-000000000001", bad)
                .into_bytes(),
        );
        assert_eq!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::InvalidScopeEncoding)
        );
    }
    let (mut input, ns, _) = fixture();
    let mut d = input.descriptor.to_vec();
    d.insert(10, 1);
    input.descriptor = bytes(d);
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidScopeEncoding)
    );
}
#[test]
fn discontinuity_is_transactional_and_legacy_r20_is_unchanged() {
    let (input, ns, stream) = fixture();
    let context = construct_reference_context(input, ns).unwrap();
    let state = context.initial_state().clone();
    let catalog = context.catalog().clone();
    assert_eq!(
        map_semantic_operation_with_context(b"bad", 1, 1, &catalog, &state),
        Err(ContextualMappingError::SemanticValidation(Error::Encoding))
    );
    assert_eq!(state, context.initial_state().clone());
    assert_eq!(
        map_semantic_operation(first_operation(stream), 1, 1, MappingState::initial())
            .unwrap()
            .record
            .physical_ordinal,
        1
    );
}
#[test]
fn descriptor_resource_limit_precedes_parsing() {
    let bytes = vec![b'x'; 262_145];
    let (mut input, ns, _) = fixture();
    input.descriptor = &bytes;
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ResourceLimit)
    );
}

fn replace_descriptor(input: &mut ClosedScopeInput<'static>, from: &str, to: &str) {
    let descriptor = text(
        String::from_utf8(input.descriptor.to_vec())
            .unwrap()
            .replace(from, to),
    );
    rebind_scope(input, descriptor.as_bytes());
}

#[test]
fn constructor_exact_set_matrix_reaches_public_error_taxonomy() {
    let (mut input, ns, _) = fixture();
    input.members = &[];
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::OmittedStream)
    );

    let (mut input, ns, _) = fixture();
    let mut extra = input.members[0].clone();
    extra.stream_namespace = parse_uuid("10112233-4455-4677-8899-aabbccddeeff").unwrap();
    input.members = Box::leak(vec![input.members[0].clone(), extra].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ExtraStream)
    );

    let (mut input, ns, _) = fixture();
    input.members =
        Box::leak(vec![input.members[0].clone(), input.members[0].clone()].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::DuplicateStreamNamespace)
    );

    let (mut input, ns, _) = fixture();
    replace_descriptor(
        &mut input,
        "00112233-4455-4677-8899-aabbccddeeff",
        "10112233-4455-4677-8899-aabbccddeeff",
    );
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::SubstitutedStream)
    );
}

#[test]
fn constructor_member_cross_binding_matrix_has_exact_errors() {
    for (needle, replacement, label) in [
        (
            "16000000-0000-4000-8000-000000000001",
            "26000000-0000-4000-8000-000000000001",
            "manifest id",
        ),
        (
            "68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594",
            "78fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594",
            "manifest digest",
        ),
        (
            "0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09",
            "1c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09",
            "stream digest",
        ),
        (
            "789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a",
            "889769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a",
            "stream artifact digest",
        ),
        ("\"818\"", "\"819\"", "stream length"),
    ] {
        let (mut input, ns, _) = fixture();
        assert!(
            String::from_utf8_lossy(input.descriptor).contains(needle),
            "fixture contains {label}"
        );
        replace_descriptor(&mut input, needle, replacement);
        assert_eq!(
            construct_reference_context(input, ns),
            Err(ContextConstructionError::InvalidMemberBinding),
            "{label}"
        );
    }

    let (mut input, ns, _) = fixture();
    let mut member = input.members[0].clone();
    member.stream_namespace = parse_uuid("10112233-4455-4677-8899-aabbccddeeff").unwrap();
    input.members = Box::leak(vec![member].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::SubstitutedStream)
    );
}

#[test]
fn constructor_scope_and_selected_namespace_failures_are_transactional() {
    let (mut input, ns, _) = fixture();
    input.scope_digest.value = "00";
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidScopeDigest)
    );

    let (input, _, _) = fixture();
    let absent = parse_uuid("10112233-4455-4677-8899-aabbccddeeff").unwrap();
    assert_eq!(
        construct_reference_context(input, absent),
        Err(ContextConstructionError::SelectedStreamMissing)
    );
}

#[test]
fn every_mapping_failure_preserves_state_and_catalog() {
    let (input, ns, stream) = fixture();
    let context = construct_reference_context(input, ns).unwrap();
    let catalog = context.catalog().clone();
    let catalog_before = catalog.clone();
    let state = context.initial_state().clone();
    let state_before = state.clone();
    for (operation, sequence, ordinal, expected) in [
        (
            b"bad".as_slice(),
            1,
            1,
            ContextualMappingError::SemanticValidation(Error::Encoding),
        ),
        (
            first_operation(stream),
            0,
            1,
            ContextualMappingError::Mapping(MappingError::State(StateError::ZeroSequence)),
        ),
        (
            first_operation(stream),
            1,
            2,
            ContextualMappingError::Mapping(MappingError::State(
                StateError::NonconsecutivePhysicalOrdinal,
            )),
        ),
    ] {
        assert_eq!(
            map_semantic_operation_with_context(operation, sequence, ordinal, &catalog, &state),
            Err(expected)
        );
        assert_eq!(state, state_before);
        assert_eq!(catalog, catalog_before);
    }
}

#[test]
fn aggregate_limits_with_stricter_public_preconditions_are_explicit() {
    // Descriptor/member equality requires at least one canonical member entry per supplied
    // stream.  The smallest closed entry in this profile is 362 bytes, so 256 entries fit below
    // 262,144 bytes; 257 is rejected by MAX_STREAMS (and by the descriptor parser) before any
    // member authority allocation. Exercise that public first-error branch without forged R16
    // evidence.
    let (mut input, ns, _) = fixture();
    let member = input.members[0].clone();
    input.members = Box::leak(vec![member; 257].into_boxed_slice());
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ResourceLimit)
    );

    // Exact descriptor success is exercised by every valid fixture. One-over is checked before
    // parsing, while an exact-length non-document reaches parsing rather than ResourceLimit.
    let exact = vec![b'x'; 262_144];
    let (mut input, ns, _) = fixture();
    input.descriptor = &exact;
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::InvalidScopeEncoding)
    );
    let over = vec![b'x'; 262_145];
    let (mut input, ns, _) = fixture();
    input.descriptor = &over;
    assert_eq!(
        construct_reference_context(input, ns),
        Err(ContextConstructionError::ResourceLimit)
    );
}
