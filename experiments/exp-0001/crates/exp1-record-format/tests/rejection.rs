use exp1_record_format::{Error, HEADER_LEN, MAX_RECORD_LEN, decode};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn v4() -> Vec<u8> {
    hex(include_str!("data/r5-v4.hex"))
}
#[test]
fn r5_v5_every_truncation_boundary_fails_closed() {
    let bytes = v4();
    for end in 0..bytes.len() {
        assert!(
            matches!(
                decode(&bytes[..end]),
                Err(Error::TruncatedHeader | Error::TruncatedRecord)
            ),
            "boundary {end}"
        );
    }
}
#[test]
fn r5_v6_v7_and_v8_documented_mutations() {
    let mut b = v4();
    b[8..12].copy_from_slice(&31u32.to_le_bytes());
    assert_eq!(decode(&b), Err(Error::InvalidLength));
    let mut b = v4();
    b[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert_eq!(decode(&b), Err(Error::UnsupportedVersion));
    let mut b = v4();
    b[7] = 2;
    assert_eq!(decode(&b), Err(Error::UnsupportedIntegrity));
    let mut b = v4();
    *b.last_mut().unwrap() = 0x40;
    assert_eq!(decode(&b), Err(Error::CrcMismatch));
}
#[test]
fn structural_header_kind_profile_reserved_and_trailing_matrix() {
    let source = hex(include_str!("data/r5-v1.hex"));
    for (offset, value, error) in [
        (0, 0, Error::BadMagic),
        (6, 0, Error::UnknownKind),
        (6, 7, Error::UnknownKind),
        (7, 2, Error::UnsupportedIntegrity),
        (24, 1, Error::NonzeroReserved),
        (28, 1, Error::NonzeroStructuralIntegrity),
    ] {
        let mut b = source.clone();
        b[offset] = value;
        assert_eq!(decode(&b), Err(error));
    }
    let mut b = source;
    b.push(0);
    assert_eq!(decode(&b), Err(Error::TrailingBytes));
}
#[test]
fn length_bounds_and_oversize_fail_before_body_access() {
    let mut b = v4();
    b[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(decode(&b), Err(Error::InvalidLength));
    let mut b = v4();
    b[8..12].copy_from_slice(&((MAX_RECORD_LEN as u32) + 1).to_le_bytes());
    assert_eq!(decode(&b), Err(Error::Oversize));
    assert_eq!(decode(&[0; HEADER_LEN - 1]), Err(Error::TruncatedHeader));
}
#[test]
fn wrong_crc_byte_order_and_crc_bytes_are_rejected() {
    let mut b = v4();
    b[28..32].reverse();
    assert_eq!(decode(&b), Err(Error::CrcMismatch));
    for bit in 0..32 {
        let mut b = v4();
        b[28 + bit / 8] ^= 1 << (bit % 8);
        assert_eq!(decode(&b), Err(Error::CrcMismatch), "CRC bit {bit}");
    }
}
#[test]
fn every_single_bit_position_in_short_crc_golden_records_is_rejected() {
    for literal in [
        include_str!("data/r5-v3.hex"),
        include_str!("data/r5-v4.hex"),
    ] {
        let original = hex(literal);
        for bit in 0..original.len() * 8 {
            let mut b = original.clone();
            b[bit / 8] ^= 1 << (bit % 8);
            assert!(
                decode(&b).is_err(),
                "accepted bit {bit} of {} bytes",
                original.len()
            );
        }
    }
}
