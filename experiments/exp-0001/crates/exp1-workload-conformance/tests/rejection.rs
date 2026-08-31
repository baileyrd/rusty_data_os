mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
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
        validate_manifest(&changed, M01.as_bytes(), &stream),
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
