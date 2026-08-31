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
    let typed = validate_manifest(M01.as_bytes(), &context(&stream)).unwrap();
    assert_eq!(typed.canonical_bytes(), M01.as_bytes());
}
