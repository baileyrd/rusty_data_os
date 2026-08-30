use exp1_record_format::{
    Body, Error, IntegrityProfile, R5_VECTOR_DISPOSITIONS, Record, ScanLimits, ScanTermination,
    Uuid, VectorDisposition, checked_extent, decode, encode, scan, scan_with_limits,
};

fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn binding(ordinal: u64, tag: u8) -> Vec<u8> {
    let mut request = [tag; 16];
    request[6] = 0x40;
    request[8] = 0x80;
    let mut event = [tag.wrapping_add(1); 16];
    event[6] = 0x40;
    event[8] = 0x80;
    encode(&Record {
        physical_ordinal: ordinal,
        integrity: IntegrityProfile::Structural,
        body: Body::Binding {
            request_id: Uuid(request),
            event_id: Uuid(event),
            normalized_request: vec![],
        },
    })
    .unwrap()
}

#[test]
fn authoritative_v1_through_v10_have_one_explicit_disposition() {
    assert_eq!(R5_VECTOR_DISPOSITIONS.len(), 10);
    for (index, (name, _)) in R5_VECTOR_DISPOSITIONS.iter().enumerate() {
        assert_eq!(*name, format!("V{}", index + 1));
    }
    assert_eq!(
        R5_VECTOR_DISPOSITIONS[4].1,
        VectorDisposition::TerminalTruncation
    );
    for literal in [
        include_str!("data/r5-v1.hex"),
        include_str!("data/r5-v2.hex"),
        include_str!("data/r5-v3.hex"),
        include_str!("data/r5-v4.hex"),
    ] {
        assert!(decode(&hex(literal)).is_ok());
    }
}

#[test]
fn terminal_truncation_retains_only_the_valid_prefix_at_every_boundary() {
    let prefix = binding(1, 1);
    let next = binding(2, 3);
    for end in 1..next.len() {
        let artifact = [prefix.as_slice(), &next[..end]].concat();
        let outcome = scan(&artifact);
        assert_eq!(outcome.records.len(), 1, "boundary {end}");
        assert!(
            matches!(
                outcome.termination,
                ScanTermination::TerminalTruncation { .. }
            ),
            "boundary {end}: {:?}",
            outcome.termination
        );
    }
}

#[test]
fn corruption_order_conflicts_and_interior_damage_stop_without_resync() {
    let first = binding(1, 1);
    let mut corrupt = hex(include_str!("data/r5-v4.hex"));
    *corrupt.last_mut().unwrap() ^= 1;
    let later = binding(3, 8);
    let outcome = scan(&[first.as_slice(), corrupt.as_slice(), later.as_slice()].concat());
    assert_eq!(outcome.records.len(), 1);
    assert_eq!(
        outcome.termination,
        ScanTermination::Failure {
            offset: first.len() as u64,
            error: Error::CrcMismatch
        }
    );

    let duplicate = binding(1, 4);
    let outcome = scan(&[first.as_slice(), duplicate.as_slice()].concat());
    assert_eq!(outcome.records.len(), 1);
    assert!(
        matches!(
            outcome.termination,
            ScanTermination::Failure {
                error: Error::OrdinalOrder,
                ..
            }
        ),
        "{:?}",
        outcome.termination
    );

    let v4 = hex(include_str!("data/r5-v4.hex"));
    let ambiguous = [first.as_slice(), &v4[..40], b"RDE1"].concat();
    let outcome = scan(&ambiguous);
    assert_eq!(outcome.records.len(), 1);
    assert!(
        matches!(
            outcome.termination,
            ScanTermination::Failure {
                error: Error::InteriorDamage,
                ..
            }
        ),
        "{:?}",
        outcome.termination
    );
}

#[test]
fn checked_length_and_all_resource_limits_are_explicit() {
    assert_eq!(checked_extent(u64::MAX, 1), Err(Error::LengthOverflow));
    let first = binding(1, 1);
    let second = binding(2, 3);
    let artifact = [first.as_slice(), second.as_slice()].concat();
    let base = ScanLimits::default();
    for (limits, error) in [
        (
            ScanLimits {
                max_records: 1,
                ..base
            },
            Error::RecordLimit,
        ),
        (
            ScanLimits {
                max_scan_bytes: (artifact.len() - 1) as u64,
                ..base
            },
            Error::ScanByteLimit,
        ),
        (
            ScanLimits {
                max_diagnostic_bytes: first.len(),
                ..base
            },
            Error::DiagnosticLimit,
        ),
        (
            ScanLimits {
                max_record_len: first.len() - 1,
                ..base
            },
            Error::Oversize,
        ),
    ] {
        assert!(
            matches!(scan_with_limits(&artifact, limits).termination, ScanTermination::Failure { error: actual, .. } if actual == error)
        );
    }

    let mut impossible = first;
    impossible[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        scan(&impossible).termination,
        ScanTermination::Failure {
            error: Error::Oversize,
            ..
        }
    ));
}
