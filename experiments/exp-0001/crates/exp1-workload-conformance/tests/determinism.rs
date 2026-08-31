mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
#[test]
fn repeatability() {
    let s = decode_hex(S01).unwrap();
    for _ in 0..100 {
        let stream = workload_stream(std::slice::from_ref(&s), 1, 0).unwrap();
        assert_eq!(
            hex(&workload_digest(&stream)),
            "0c1634abb76bc9ab70b864ba11154a704f83df42caca9556f90b2704fe3b8f09"
        );
        assert_eq!(
            hex(&manifest_digest(M01.as_bytes())),
            "a696eae0b2a85d2d3f89b51bbacfa2d5564448ea849bece9eeaad7ec21a9ee56"
        );
    }
}
