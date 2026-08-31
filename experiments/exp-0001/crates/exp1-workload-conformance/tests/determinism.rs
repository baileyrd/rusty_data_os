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
            "68fb7283923c5f661845e2439544f4345fe5ba6782d8dd5bc28b2cfab5e10594"
        );
    }
}
