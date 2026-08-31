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
fn context(stream: &[u8]) -> ValidationContext<'_> {
    ValidationContext {
        stream,
        descriptor: &DESCRIPTOR,
        manifest_artifact_sha256: "ca4f9ad7a3f405aba25efca556794a54bed35c7d84b37f9ee5e260b9252bfe86",
        targets: &[],
        artifact_manifest_bytes: &[],
        stream_artifact: &STREAM_ARTIFACT,
        workload_artifact_manifest_bytes: R7_WORKLOAD_ARTIFACT_MANIFEST,
        workload_artifact_manifest_ref: &WORKLOAD_ARTIFACT_MANIFEST_REF,
    }
}

#[test]
fn malformed_streams_fail_without_panics() {
    let w = decode_hex(W01).unwrap();
    for n in 0..w.len() {
        assert!(validate_stream(&w[..n]).is_err());
    }
    let mut trailing = w.clone();
    trailing.push(0);
    assert_eq!(validate_stream(&trailing), Err(Error::Encoding));
}
#[test]
fn manifest_noncanonical_fails() {
    let stream = workload_stream(&[decode_hex(S01).unwrap()], 1, 0).unwrap();
    let mut changed = M01.as_bytes().to_vec();
    changed.push(b'\n');
    assert_eq!(
        validate_manifest(&changed, &context(&stream)),
        Err(Error::Noncanonical)
    );
}
#[test]
fn supersession_rules() {
    assert!(validate_supersession("a", "w", &[], None).is_ok());
    assert_eq!(
        validate_supersession("a", "w", &[("a".into(), "w".into())], Some("fix")),
        Err(Error::ImmutableState)
    );
}

#[test]
fn manifest_closed_json_rejections() {
    let stream = workload_stream(&[decode_hex(S01).unwrap()], 1, 0).unwrap();
    let duplicate = M01.replacen("{", "{\"authority_revisions\":[],", 1);
    assert!(matches!(
        Manifest::parse(duplicate.as_bytes()),
        Err(Error::DuplicateMember)
    ));
    let unknown = M01.replacen("}", ",\"zzz\":\"x\"}", 1);
    assert!(matches!(
        Manifest::parse(unknown.as_bytes()),
        Err(Error::UnknownField) | Err(Error::Noncanonical)
    ));
    let missing = M01.replace("\"record_kind\":\"workload_manifest\",", "");
    assert_eq!(
        Manifest::parse(missing.as_bytes()),
        Err(Error::MissingField)
    );
    let version = M01.replace(
        "EXP-0001-WORKLOAD-MANIFEST-JCS-v1",
        "EXP-0001-WORKLOAD-MANIFEST-JCS-v2",
    );
    assert_eq!(Manifest::parse(version.as_bytes()), Err(Error::Unsupported));
    let mut corpus = Vec::new();
    for n in 0..M01.len() {
        corpus.push(&M01.as_bytes()[..n]);
    }
    for candidate in corpus {
        assert!(std::panic::catch_unwind(|| Manifest::parse(candidate)).is_ok());
    }
    assert!(validate_manifest(M01.as_bytes(), &context(&stream)).is_err());
}
