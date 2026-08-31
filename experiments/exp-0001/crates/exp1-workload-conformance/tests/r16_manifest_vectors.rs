mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
static DESCRIPTOR: ManifestDigestDescriptor<'static> = ManifestDigestDescriptor {
    algorithm: "SHA-256/FIPS-180-4",
    domain: MANIFEST_DOMAIN,
    profile: "EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1",
    value: "b6e9a1d2ffa65bed11bdea6a2606abd4aee2200ddced30c8ec6000bccdde2ea9",
    manifest_ref: ManifestReference {
        artifact_id: "16000000-0000-4000-8000-000000000001",
        byte_length: 3423,
        sha256: "8fcbf85b1036acdc212ee179a549107efa189b9fa09efe92981ed1601eed7178",
        uri: "https://example.invalid/exp-0001/m01.manifest.jcs",
    },
};
static STREAM_ARTIFACT: ArtifactMetadata<'static> = ArtifactMetadata {
    artifact_id: "16000000-0000-4000-8000-000000000002",
    byte_length: 818,
    sha256: "789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a",
    uri: "https://example.invalid/exp-0001/s01.stream",
    role: "configuration",
    media_type: "application/vnd.rusty-data-os.exp1-workload-stream",
    created_by_record_id: "16000000-0000-4000-8000-000000000005",
};
static MANIFEST_ARTIFACT: ArtifactMetadata<'static> = ArtifactMetadata {
    artifact_id: "16000000-0000-4000-8000-000000000001",
    byte_length: 3423,
    sha256: "8fcbf85b1036acdc212ee179a549107efa189b9fa09efe92981ed1601eed7178",
    uri: "https://example.invalid/exp-0001/m01.manifest.jcs",
    role: "configuration",
    media_type: "application/vnd.rusty-data-os.exp1-workload-manifest",
    created_by_record_id: "16000000-0000-4000-8000-000000000006",
};
fn context(stream: &[u8]) -> ValidationContext<'_> {
    static ARTIFACT_MANIFEST: &[u8] = br#"{"artifact_id":"16000000-0000-4000-8000-000000000004","artifacts":[{"artifact_id":"16000000-0000-4000-8000-000000000002","byte_length":"818","created_by_record_id":"16000000-0000-4000-8000-000000000005","media_type":"application/vnd.rusty-data-os.exp1-workload-stream","role":"configuration","sha256":"789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a","uri":"https://example.invalid/exp-0001/s01.stream"}],"provenance_edges":[{"from":"16000000-0000-4000-8000-000000000002","relation":"created_by","to":"16000000-0000-4000-8000-000000000005"},{"from":"16000000-0000-4000-8000-000000000005","relation":"derived_from","to":"16000000-0000-4000-8000-000000000001"}],"uri":"https://example.invalid/exp-0001/artifact-manifest.jcs"}"#;
    static PROVENANCE: [ProvenanceEdge<'static>; 2] = [
        ProvenanceEdge {
            from: "16000000-0000-4000-8000-000000000002",
            to: "16000000-0000-4000-8000-000000000005",
            relation: "created_by",
        },
        ProvenanceEdge {
            from: "16000000-0000-4000-8000-000000000005",
            to: "16000000-0000-4000-8000-000000000001",
            relation: "derived_from",
        },
    ];
    ValidationContext {
        stream,
        descriptor: &DESCRIPTOR,
        manifest_artifact_sha256: "8fcbf85b1036acdc212ee179a549107efa189b9fa09efe92981ed1601eed7178",
        targets: &[],
        artifact_manifest_bytes: ARTIFACT_MANIFEST,
        stream_artifact: &STREAM_ARTIFACT,
        manifest_artifact: &MANIFEST_ARTIFACT,
        provenance: &PROVENANCE,
    }
}

fn typed_m01() -> TypedManifest {
    let revisions = [
        (
            "EXP-0000-WORKLOADS",
            RevisionKind::GitSha,
            "70a29efd46dd3aee9ea9cb0831d0285b83cdd70a",
        ),
        (
            "EXP-0001-R12",
            RevisionKind::GitSha,
            "e39551e64d9a799a3d15bf75aa70a323c8e40ca8",
        ),
        (
            "EXP-0001-R14",
            RevisionKind::GitSha,
            "78b8b35e4efda44a8097db05f396679a1265a239",
        ),
        (
            "EXP-0001-R16",
            RevisionKind::ReviewedAuthorityId,
            "documentation-vector-v1",
        ),
        (
            "EXP-0001-R2",
            RevisionKind::GitSha,
            "2659fb34caf054a7742a854d69d17cdd59bd2040",
        ),
        (
            "EXP-0001-R7",
            RevisionKind::GitSha,
            "f9d9876cf6599345a2e2244223a530ada2b9a828",
        ),
    ]
    .into_iter()
    .map(|(a, kind, value)| AuthorityRevision {
        authority: a.into(),
        kind,
        value: value.into(),
    })
    .collect();
    TypedManifest {
        authority_revisions: revisions,
        counts: ManifestCounts {
            by_envelope_profile: vec![Distribution {
                name: "envelope-minimal".into(),
                count: 1,
            }],
            by_segment: vec![
                Distribution {
                    name: "warm_up".into(),
                    count: 1,
                },
                Distribution {
                    name: "measured".into(),
                    count: 0,
                },
            ],
            by_size_class: vec![Distribution {
                name: "P1".into(),
                count: 1,
            }],
            by_temporal_profile: vec![Distribution {
                name: "time-monotonic-effective".into(),
                count: 1,
            }],
            measured: 0,
            total: 1,
            warm_up: 1,
        },
        created_at_utc_ns: 1_788_134_400_000_000_000,
        generator_inputs: GeneratorInputs {
            actor_provenance: InputState::NotApplicable,
            base_ns: 1000,
            controlled_schedule: InputState::NotApplicable,
            correction_fact_type: InputState::NotApplicable,
            envelope_semantic_version: "1".into(),
            ordinary_fact_type: "fact-A".into(),
            producer_id: "10213243-5465-4768-899a-abbccddeef00".into(),
            reference_cardinality: 0,
            schema_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee".into(),
            schema_version: "1".into(),
            seed: 0,
            source_provenance: InputState::NotApplicable,
            stream_namespace: "00112233-4455-4677-8899-aabbccddeeff".into(),
            unit_ns: 10,
        },
        manifest_id: "16000000-0000-4000-8000-000000000001".into(),
        profiles: ManifestProfiles {
            envelope: "envelope-minimal".into(),
            payload_content: "deterministic-high-variation".into(),
            payload_generator: "EXP-0001-SHA256-CTR-v1".into(),
            payload_size: "fixed-P1".into(),
            temporal: "time-monotonic-effective".into(),
        },
        stream_digest: DigestValue {
            value: "0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09".into(),
        },
        stream_ref: StreamReference {
            artifact_id: "16000000-0000-4000-8000-000000000002".into(),
            artifact_manifest_ref: ArtifactReference {
                artifact_id: "16000000-0000-4000-8000-000000000004".into(),
                byte_length: 4096,
                sha256: "1111111111111111111111111111111111111111111111111111111111111111".into(),
                uri: "https://example.invalid/exp-0001/artifact-manifest.jcs".into(),
            },
            byte_length: 818,
            created_by_record_id: "16000000-0000-4000-8000-000000000005".into(),
            sha256: "789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a".into(),
            uri: "https://example.invalid/exp-0001/s01.stream".into(),
        },
        supersession: Supersession {
            reason: InputState::NotApplicable,
            manifest_ids: vec![],
        },
        workload_id: "16000000-0000-4000-8000-000000000003".into(),
    }
}

#[test]
fn m01_literal() {
    let s01 = decode_hex(S01).unwrap();
    let stream = workload_stream(&[s01], 1, 0).unwrap();
    assert_eq!(stream.len(), 818);
    assert_eq!(
        hex(&workload_digest(&stream)),
        "0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09"
    );
    assert_eq!(
        hex(&artifact_digest(&stream)),
        "789769303a70ae2a5f77682e7ad82cf01db34ffd3283fa0757805e46feb6586a"
    );
    assert_eq!(M01.len(), 3423);
    assert_eq!(
        hex(&manifest_digest(M01.as_bytes())),
        "b6e9a1d2ffa65bed11bdea6a2606abd4aee2200ddced30c8ec6000bccdde2ea9"
    );
    let typed = Manifest::from_typed(typed_m01()).unwrap();
    assert_eq!(typed.canonical_bytes(), M01.as_bytes());
    validate_manifest(M01.as_bytes(), &context(&stream)).unwrap();
}
