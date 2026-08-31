use exp1_workload_conformance::*;
fn anchor(content: Content) -> OperationInput {
    OperationInput {
        segment: Segment::WarmUp,
        seed: 0,
        ordinal: 0,
        size_class: 1,
        content,
        envelope: Envelope::Minimal,
        temporal: Temporal::Monotonic,
        stream_namespace: parse_uuid("00112233-4455-4677-8899-aabbccddeeff").unwrap(),
        producer_id: parse_uuid("10213243-5465-4768-899a-abbccddeef00").unwrap(),
        producer_ordinal: 0,
        controlled_schedule: None,
    }
}
#[test]
fn normalization_and_sha() {
    assert_eq!(parse_seed("0"), Ok(0));
    assert_eq!(parse_seed("18446744073709551615"), Ok(u64::MAX));
    for bad in ["+0", "-0", "00", "01", "", " 0"] {
        assert_eq!(parse_seed(bad), Err(Error::SeedSyntax));
    }
    assert_eq!(parse_seed("18446744073709551616"), Err(Error::SeedRange));
    assert_eq!(
        hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn payload_anchors() {
    assert_eq!(
        hex(&payload(&anchor(Content::High)).unwrap()),
        "9a06d1077fd2e1119719444421b9df11bdbf3131aa90e8ab5a291cca55202e7f"
    );
    assert_eq!(
        hex(&payload(&anchor(Content::Low)).unwrap()),
        "3802fbfd223852b13802fbfd223852b13802fbfd223852b13802fbfd223852b1"
    );
    assert_eq!(payload(&anchor(Content::Zero)).unwrap(), vec![0; 32]);
}
#[test]
fn identity_anchors() {
    let a = anchor(Content::High);
    assert_eq!(
        format_uuid(identity(&a, IdentityKind::Request).unwrap()),
        "cf797546-51a3-4f76-b171-8244bf8053db"
    );
    assert_eq!(
        format_uuid(identity(&a, IdentityKind::Event).unwrap()),
        "330f201a-ea7c-4335-a8ec-e6fe23266a1c"
    );
    assert_eq!(
        format_uuid(identity(&a, IdentityKind::Information).unwrap()),
        "3d3c5281-3d43-47db-8825-664f324e091d"
    );
}
#[test]
fn logical_vectors() {
    assert_eq!(
        (0..4)
            .map(|i| logical_time(Temporal::Monotonic, i, 1000, 10).unwrap())
            .collect::<Vec<_>>(),
        [1000, 1010, 1020, 1030]
    );
    assert_eq!(
        (98..102)
            .map(|i| logical_time(Temporal::EqualBurst, i, 1000, 10).unwrap())
            .collect::<Vec<_>>(),
        [1000, 1000, 1010, 1010]
    );
    assert_eq!(
        (8..11)
            .map(|i| logical_time(Temporal::Late, i, 1000, 10).unwrap())
            .collect::<Vec<_>>(),
        [1080, 90, 1100]
    );
    assert_eq!(
        (0..4)
            .map(|i| logical_time(Temporal::OutOfOrder, i, 1000, 10).unwrap())
            .collect::<Vec<_>>(),
        [1000, 1020, 1010, 1030]
    );
}
