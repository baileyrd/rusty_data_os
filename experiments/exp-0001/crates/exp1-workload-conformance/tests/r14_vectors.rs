mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
#[test]
fn operation_vectors() {
    let a = decode_hex(S01).unwrap();
    let b = decode_hex(S02).unwrap();
    assert_eq!((a.len(), b.len()), (755, 770));
    validate_record(&a, b"RDOS-SOP1", 13).unwrap();
    validate_record(&b, b"RDOS-SOP1", 13).unwrap();
    assert_eq!(
        hex(&sha256(&a)),
        "efa80d1b021e590b8ac02b49a9bb0e68277cf39f32f3849aceabb33e2ec9b83c"
    );
    assert_eq!(
        hex(&sha256(&b)),
        "85a917fe5d4ef24e1904cb6b8ac2554fa60f99ae6f0c69db5e72cf6d81628ddf"
    );
}
#[test]
fn stream_vectors() {
    let empty = decode_hex(W00).unwrap();
    assert_eq!(empty.len(), 55);
    assert_eq!(workload_stream(&[], 0, 0).unwrap(), empty);
    assert_eq!(
        hex(&workload_digest(&empty)),
        "6ed7e39756dab1b00e5860365288a35b7b8d40f92bc8d219de50eb633144d387"
    );
    let w = decode_hex(W01).unwrap();
    assert_eq!(w.len(), 1596);
    assert_eq!(validate_stream(&w), Ok((2, 2, 0)));
    assert_eq!(
        hex(&workload_digest(&w)),
        "81dbc6b6e33ee775d4b36aeaa0aca45b9649c987f180e378b5d5fbcf1bc3b024"
    );
    assert_eq!(
        workload_stream(&[decode_hex(S01).unwrap(), decode_hex(S02).unwrap()], 2, 0).unwrap(),
        w
    );
}
