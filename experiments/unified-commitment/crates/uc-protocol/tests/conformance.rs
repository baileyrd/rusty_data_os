use uc_protocol::{codec::*, framing::*, *};
#[path = "fixtures/expected_cases.rs"]
mod expected;

fn unhex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn all_66_host_literal_values_and_byte_exact_round_trips() {
    let wire: Vec<_> = include_str!("fixtures/wire-vectors.txt")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();
    let host: Vec<_> = include_str!("fixtures/step4b-fixture-expected-values.txt")
        .lines()
        .skip(2)
        .filter(|l| !l.is_empty())
        .collect();
    let cases = expected::cases();
    assert_eq!(wire.len(), 66);
    assert_eq!(host.len(), 66);
    assert_eq!(cases.len(), 66);
    for ((line, host), (literal, value)) in wire.iter().zip(host).zip(cases) {
        assert_eq!(host, literal, "literal transcription drift");
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 3);
        assert!(
            literal.starts_with(&format!("{} {} ", columns[0], columns[1])),
            "fixture pairing: {line}"
        );
        let payload = unhex(columns[2]);
        match value {
            expected::Expected::Request(expected) => {
                let decoded = decode_request(&payload).unwrap_or_else(|e| panic!("{line}: {e}"));
                assert_eq!(decoded, expected, "literal value: {line}");
                assert_eq!(encode_request(&decoded), payload, "round trip: {line}");
            }
            expected::Expected::Response(expected) => {
                let decoded = decode_response(&payload).unwrap_or_else(|e| panic!("{line}: {e}"));
                assert_eq!(decoded, expected, "literal value: {line}");
                assert_eq!(encode_response(&decoded), payload, "round trip: {line}");
            }
        }
    }
}
#[test]
fn every_fixture_rejects_truncation_and_trailing_bytes() {
    for line in include_str!("fixtures/wire-vectors.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let columns: Vec<_> = line.split('\t').collect();
        let payload = unhex(columns[2]);
        let accepts = |b: &[u8]| {
            if columns[0].starts_with("Request/") {
                decode_request(b).is_ok()
            } else {
                decode_response(b).is_ok()
            }
        };
        for end in 0..payload.len() {
            assert!(!accepts(&payload[..end]), "truncation {end}: {line}");
        }
        let mut trailing = payload;
        trailing.push(0);
        assert!(!accepts(&trailing), "trailing: {line}");
    }
}
#[test]
fn malformed_tags_lengths_and_utf8_are_errors() {
    assert!(decode_request(&32u32.to_le_bytes()).is_err());
    assert!(decode_response(&21u32.to_le_bytes()).is_err());
    assert!(decode::<ScanValue>(&6u32.to_le_bytes()).is_err());
    assert!(decode::<ErrorCode>(&14u32.to_le_bytes()).is_err());
    assert!(decode::<bool>(&[2]).is_err());
    assert!(decode::<Option<u32>>(&[2]).is_err());
    assert!(decode::<String>(&u64::MAX.to_le_bytes()).is_err());
    assert!(decode::<Vec<Request>>(&u64::MAX.to_le_bytes()).is_err());
    assert!(decode::<RecordId>(&15u64.to_le_bytes()).is_err());
    let mut utf8 = 1u64.to_le_bytes().to_vec();
    utf8.push(255);
    assert!(decode::<String>(&utf8).is_err());
}
#[test]
fn primitive_widths_option_and_float_bits_are_literal() {
    assert_eq!(encode(&0x1234u16), [0x34, 0x12]);
    assert_eq!(encode(&0xdeadbeefu32), [0xef, 0xbe, 0xad, 0xde]);
    assert_eq!(encode(&Some(0x1234u16)), [1, 0x34, 0x12]);
    assert_eq!(encode(&None::<u16>), [0]);
    for bits in [0u64, 1, u64::MAX, 0x8000000000000000, 0x7ff8000000000042] {
        assert_eq!(encode(&f64::from_bits(bits)), bits.to_le_bytes());
        assert_eq!(decode::<f64>(&bits.to_le_bytes()).unwrap().to_bits(), bits);
    }
    assert_eq!(encode(&1usize), [1, 0, 0, 0, 0, 0, 0, 0]);
}
#[test]
fn all_supporting_enum_tags_and_write_variants() {
    let writes = vec![
        WriteOp::Insert {
            id: RecordId::from_u128(1),
            fields: vec![],
        },
        WriteOp::Replace {
            id: RecordId::from_u128(2),
            fields: vec![],
        },
        WriteOp::ReplaceIf {
            id: RecordId::from_u128(3),
            fields: vec![],
            guard: Predicate {
                field: 1,
                op: CompareOp::Ne,
                value: ScanValue::Str("x".into()),
            },
        },
        WriteOp::Delete {
            id: RecordId::from_u128(4),
        },
        WriteOp::Link {
            left: RecordId::from_u128(5),
            right: RecordId::from_u128(6),
            relation: "r".into(),
        },
    ];
    for (tag, value) in writes.iter().enumerate() {
        let b = encode(value);
        assert_eq!(&b[..4], &(tag as u32).to_le_bytes());
        assert_eq!(decode::<WriteOp>(&b).unwrap(), *value);
    }
    macro_rules! tags { ($ty:ty, [$($v:expr),*])=>{ for (tag,value) in [$($v),*].iter().enumerate() {let b=encode(value);assert_eq!(&b[..4],&(tag as u32).to_le_bytes());assert_eq!(decode::<$ty>(&b).unwrap(),*value);} } }
    tags!(
        CompareOp,
        [
            CompareOp::Eq,
            CompareOp::Ne,
            CompareOp::Lt,
            CompareOp::Le,
            CompareOp::Gt,
            CompareOp::Ge
        ]
    );
    tags!(
        AggregateFn,
        [
            AggregateFn::Count,
            AggregateFn::Sum,
            AggregateFn::Avg,
            AggregateFn::Min,
            AggregateFn::Max
        ]
    );
    tags!(
        ValueKind,
        [
            ValueKind::U32,
            ValueKind::I64,
            ValueKind::Bool,
            ValueKind::Str,
            ValueKind::StrList
        ]
    );
    tags!(
        ErrorCode,
        [
            ErrorCode::UnknownField,
            ErrorCode::Unsupported,
            ErrorCode::Malformed,
            ErrorCode::Unauthenticated,
            ErrorCode::Unauthorized,
            ErrorCode::RecordNotFound,
            ErrorCode::NoSession,
            ErrorCode::SessionOpen,
            ErrorCode::SessionFull,
            ErrorCode::Journal,
            ErrorCode::Conflict,
            ErrorCode::Duplicate,
            ErrorCode::Storage,
            ErrorCode::GuardFailed
        ]
    );
    tags!(
        WriteResult,
        [
            WriteResult::Inserted,
            WriteResult::Duplicate,
            WriteResult::Replaced,
            WriteResult::NotFound,
            WriteResult::GuardFailed,
            WriteResult::Linked,
            WriteResult::AlreadyLinked,
            WriteResult::Deleted,
            WriteResult::Failed(ErrorCode::Malformed)
        ]
    );
    tags!(
        ParentLookup,
        [
            ParentLookup::Parent(RecordId::from_u128(1)),
            ParentLookup::NoParent,
            ParentLookup::ChildNotFound
        ]
    );
    tags!(
        JoinRelation,
        [
            JoinRelation::Neighbors(Some("x".into())),
            JoinRelation::Parent,
            JoinRelation::Children
        ]
    );
}
#[test]
fn framing_limits_truncation_and_exact_prefix() {
    let mut frame = vec![];
    write_message(&mut frame, &[3, 4, 5]).unwrap();
    assert_eq!(frame, [3, 0, 0, 0, 3, 4, 5]);
    assert_eq!(read_message(&mut frame.as_slice()).unwrap(), [3, 4, 5]);
    for end in 0..frame.len() {
        assert!(read_message(&mut &frame[..end]).is_err());
    }
    assert!(read_message(&mut &(MAX_FRAME_BYTES + 1).to_le_bytes()[..]).is_err());
    let max = vec![0; MAX_FRAME_BYTES as usize];
    let mut framed = Vec::new();
    write_message(&mut framed, &max).unwrap();
    assert_eq!(read_message(&mut framed.as_slice()).unwrap(), max);
    assert!(write_message(&mut Vec::new(), &vec![0; MAX_FRAME_BYTES as usize + 1]).is_err());
}
