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
fn context(stream: &[u8]) -> ValidationContext<'_> {
    ValidationContext {
        stream,
        descriptor: &DESCRIPTOR,
        manifest_artifact_sha256: "8fcbf85b1036acdc212ee179a549107efa189b9fa09efe92981ed1601eed7178",
        targets: &[],
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
    assert!(validate_manifest(M01.as_bytes(), &context(&stream)).is_ok());
}
