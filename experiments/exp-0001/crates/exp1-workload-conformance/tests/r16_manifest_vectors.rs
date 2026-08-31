mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
static DESCRIPTOR: ManifestDigestDescriptor<'static> = ManifestDigestDescriptor {
    algorithm: "SHA-256/FIPS-180-4",
    domain: MANIFEST_DOMAIN,
    profile: "EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1",
    value: "68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594",
    manifest_ref: ManifestReference {
        artifact_id: "16000000-0000-4000-8000-000000000001",
        byte_length: 3423,
        sha256: "ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86",
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
static WORKLOAD_ARTIFACT_MANIFEST_REF: ManifestReference<'static> = ManifestReference {
    artifact_id: "16000000-0000-4000-8000-000000000011",
    byte_length: 1152,
    sha256: "d49627606be85859b5067962eb4b793a0c757774c5d9c32bf5f5658355d0418e",
    uri: "https://example.invalid/exp-0001/workload-artifact-manifest.jcs",
};
fn context_with<'a>(
    stream: &'a [u8],
    artifact_manifest_bytes: &'a [u8],
    workload_artifact_manifest_bytes: &'a [u8],
    workload_artifact_manifest_ref: &'a ManifestReference<'a>,
) -> ValidationContext<'a> {
    ValidationContext {
        stream,
        descriptor: &DESCRIPTOR,
        manifest_artifact_sha256: "ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86",
        targets: &[],
        artifact_manifest_bytes,
        workload_artifact_manifest_bytes,
        workload_artifact_manifest_ref,
        stream_artifact: &STREAM_ARTIFACT,
    }
}

fn context(stream: &[u8]) -> ValidationContext<'_> {
    context_with(
        stream,
        R7_ARTIFACT_MANIFEST,
        R7_WORKLOAD_ARTIFACT_MANIFEST,
        &WORKLOAD_ARTIFACT_MANIFEST_REF,
    )
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
                artifact_id: "16000000-0000-4000-8000-000000000009".into(),
                byte_length: 1274,
                sha256: "b65688eb056a71bacaff1178ef4d0693b1c5ef59c43bdbdaa7b360e562f4998c".into(),
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
    assert_eq!(R7_ARTIFACT_MANIFEST.len(), 1274);
    assert_eq!(
        hex(&artifact_digest(R7_ARTIFACT_MANIFEST)),
        "b65688eb056a71bacaff1178ef4d0693b1c5ef59c43bdbdaa7b360e562f4998c"
    );
    assert_eq!(M01.len(), 3423);
    assert_eq!(
        hex(&manifest_digest(M01.as_bytes())),
        "68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594"
    );
    let typed = Manifest::from_typed(typed_m01()).unwrap();
    assert_eq!(typed.canonical_bytes(), M01.as_bytes());
    assert!(validate_manifest(M01.as_bytes(), &context(&stream)).is_ok());
}

#[test]
fn r7_closed_record_and_reference_fail_closed() {
    let stream = workload_stream(&[decode_hex(S01).unwrap()], 1, 0).unwrap();
    let cases = [
        (
            "\"record_kind\":\"artifact_manifest\"",
            "\"record_kind\":\"environment\"",
        ),
        ("\"scope\":\"run\"", "\"scope\":\"series\""),
        (
            "\"publication_state\":\"published\"",
            "\"publication_state\":\"staged\"",
        ),
        ("\"logical_path\":", "\"unknown\":\"x\",\"logical_path\":"),
        ("\"sensitivity\":\"public\",", ""),
        ("\"from_artifact_id\"", "\"from\""),
    ];
    for (old, new) in cases {
        let fixture = String::from_utf8(R7_ARTIFACT_MANIFEST.to_vec())
            .unwrap()
            .replacen(old, new, 1);
        assert!(
            validate_manifest(
                M01.as_bytes(),
                &context_with(
                    &stream,
                    fixture.as_bytes(),
                    R7_WORKLOAD_ARTIFACT_MANIFEST,
                    &WORKLOAD_ARTIFACT_MANIFEST_REF
                )
            )
            .is_err()
        );
    }

    let mut missing_entry = String::from_utf8(R7_WORKLOAD_ARTIFACT_MANIFEST.to_vec()).unwrap();
    let entry_start = missing_entry.find("\"artifacts\":[").unwrap() + "\"artifacts\":[".len();
    let entry_end = missing_entry.find("],\"provenance_edges\"").unwrap();
    missing_entry.replace_range(entry_start..entry_end, "");
    let missing_digest = hex(&artifact_digest(missing_entry.as_bytes()));
    let missing_reference = ManifestReference {
        artifact_id: WORKLOAD_ARTIFACT_MANIFEST_REF.artifact_id,
        byte_length: missing_entry.len() as u64,
        sha256: &missing_digest,
        uri: WORKLOAD_ARTIFACT_MANIFEST_REF.uri,
    };
    assert!(
        validate_manifest(
            M01.as_bytes(),
            &context_with(
                &stream,
                R7_ARTIFACT_MANIFEST,
                missing_entry.as_bytes(),
                &missing_reference
            )
        )
        .is_err()
    );

    let workload_cases = [
        ("000000000001\",", "000000000099\","),
        ("\"byte_length\":\"3423\"", "\"byte_length\":\"3424\""),
        (
            "ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86",
            "aa4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86",
        ),
        ("m01.manifest.jcs", "wrong.manifest.jcs"),
        ("workload_manifest\",", "configuration\","),
        (
            "application/vnd.rusty-data-os.exp1-workload-manifest+jcs",
            "application/json",
        ),
        ("000000000006\",", "000000000099\","),
        ("\"sensitivity\":\"public\"", "\"sensitivity\":\"secret\""),
        (
            "\"validation_report_ids\":[]",
            "\"validation_report_ids\":[],\"unknown\":\"x\"",
        ),
    ];
    for (old, new) in workload_cases {
        let fixture = String::from_utf8(R7_WORKLOAD_ARTIFACT_MANIFEST.to_vec())
            .unwrap()
            .replacen(old, new, 1);
        let digest = hex(&artifact_digest(fixture.as_bytes()));
        let reference = ManifestReference {
            artifact_id: WORKLOAD_ARTIFACT_MANIFEST_REF.artifact_id,
            byte_length: fixture.len() as u64,
            sha256: &digest,
            uri: WORKLOAD_ARTIFACT_MANIFEST_REF.uri,
        };
        assert!(
            validate_manifest(
                M01.as_bytes(),
                &context_with(
                    &stream,
                    R7_ARTIFACT_MANIFEST,
                    fixture.as_bytes(),
                    &reference
                )
            )
            .is_err(),
            "accepted workload artifact mutation: {old}"
        );
    }

    assert!(
        validate_manifest(
            M01.as_bytes(),
            &context_with(
                &stream,
                &R7_ARTIFACT_MANIFEST[..1273],
                R7_WORKLOAD_ARTIFACT_MANIFEST,
                &WORKLOAD_ARTIFACT_MANIFEST_REF,
            )
        )
        .is_err()
    );
}
