use exp1_record_format::{crc32c, decode, encode};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture(s: &str) -> Vec<u8> {
    hex(s)
}
#[test]
fn r5_crc32c_check_value() {
    assert_eq!(
        format!("{:08x}", crc32c(b"123456789")),
        include_str!("data/crc32c-check.txt").trim()
    );
}
#[test]
fn r5_v1_through_v4_encode_decode_exact() {
    for (name, literal) in [
        ("V1", include_str!("data/r5-v1.hex")),
        ("V2", include_str!("data/r5-v2.hex")),
        ("V3", include_str!("data/r5-v3.hex")),
        ("V4", include_str!("data/r5-v4.hex")),
    ] {
        let bytes = fixture(literal);
        let value = decode(&bytes).unwrap_or_else(|e| panic!("R5 {name}: {e:?}"));
        assert_eq!(encode(&value), Ok(bytes), "R5 {name}");
    }
}
