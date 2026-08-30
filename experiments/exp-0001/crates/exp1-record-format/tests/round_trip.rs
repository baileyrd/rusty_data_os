use exp1_record_format::{decode, encode};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn r5_reviewed_finite_corpus_round_trips_both_directions() {
    for literal in [
        include_str!("data/r5-v1.hex"),
        include_str!("data/r5-v2.hex"),
        include_str!("data/r5-v3.hex"),
        include_str!("data/r5-v4.hex"),
    ] {
        let bytes = hex(literal);
        let value = decode(&bytes).unwrap();
        assert_eq!(decode(&encode(&value).unwrap()), Ok(value));
        assert_eq!(encode(&decode(&bytes).unwrap()), Ok(bytes));
    }
}
