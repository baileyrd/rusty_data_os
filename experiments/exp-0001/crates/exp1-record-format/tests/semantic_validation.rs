use exp1_record_format::{
    Body, Error, IntegrityProfile, Record, Uuid, decode, encode, validate_lifecycle,
};
fn id(n: u8) -> Uuid {
    let mut b = [n; 16];
    b[6] = (b[6] & 15) | 0x40;
    b[8] = (b[8] & 63) | 0x80;
    Uuid(b)
}
#[test]
fn uuid_role_version_variant_and_nil_matrix() {
    for (uuid, error) in [
        (Uuid([0; 16]), Error::NilUuid),
        (
            {
                let mut x = id(1);
                x.0[6] = 0x50;
                x
            },
            Error::UuidVersion,
        ),
        (
            {
                let mut x = id(1);
                x.0[8] = 0;
                x
            },
            Error::UuidVariant,
        ),
    ] {
        let r = Record {
            physical_ordinal: 1,
            integrity: IntegrityProfile::Structural,
            body: Body::Binding {
                request_id: uuid,
                event_id: id(2),
                normalized_request: vec![],
            },
        };
        assert_eq!(encode(&r), Err(error));
    }
}
#[test]
fn sequence_group_and_membership_constraints_are_table_driven() {
    let base = Record {
        physical_ordinal: 1,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Reservation {
            request_id: id(1),
            event_id: id(2),
            sequence: 1,
            high_water: 1,
        },
    };
    for body in [
        Body::Reservation {
            request_id: id(1),
            event_id: id(2),
            sequence: 0,
            high_water: 1,
        },
        Body::Reservation {
            request_id: id(1),
            event_id: id(2),
            sequence: 2,
            high_water: 1,
        },
        Body::Provisional {
            event_id: id(2),
            sequence: 1,
            group_id: 0,
            member_index: 1,
            member_count: 1,
            stable_core: vec![],
        },
    ] {
        let mut r = base.clone();
        r.body = body;
        assert!(encode(&r).is_err());
    }
    let m = Record {
        physical_ordinal: 1,
        integrity: IntegrityProfile::Structural,
        body: Body::Membership {
            group_id: 1,
            members: vec![(id(2), 2), (id(3), 1)],
        },
    };
    assert_eq!(encode(&m), Err(Error::SequenceOrder));
}
#[test]
fn final_commit_adjacency_identity_crc_and_sequence_binding() {
    let binding = Record {
        physical_ordinal: 1,
        integrity: IntegrityProfile::Structural,
        body: Body::Binding {
            request_id: id(1),
            event_id: id(2),
            normalized_request: vec![],
        },
    };
    let reservation = Record {
        physical_ordinal: 2,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Reservation {
            request_id: id(1),
            event_id: id(2),
            sequence: 9,
            high_water: 9,
        },
    };
    let final_record = Record {
        physical_ordinal: 3,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Final {
            event_id: id(2),
            request_id: id(1),
            sequence: 9,
            durability_time: 12,
            complete_envelope: b"opaque".to_vec(),
        },
    };
    let encoded = encode(&final_record).unwrap();
    let crc = u32::from_le_bytes(encoded[28..32].try_into().unwrap());
    let commit = Record {
        physical_ordinal: 4,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Commit {
            event_id: id(2),
            sequence: 9,
            final_ordinal: 3,
            final_crc32c: crc,
            group_id: 0,
            member_index: 0,
            member_count: 1,
        },
    };
    let prefix = [binding.clone(), reservation.clone()];
    assert_eq!(
        validate_lifecycle(&[
            prefix[0].clone(),
            prefix[1].clone(),
            final_record.clone(),
            commit.clone()
        ]),
        Ok(())
    );
    let mut wrong = commit.clone();
    if let Body::Commit { event_id, .. } = &mut wrong.body {
        *event_id = id(3)
    }
    assert_eq!(
        validate_lifecycle(&[
            prefix[0].clone(),
            prefix[1].clone(),
            final_record.clone(),
            wrong
        ]),
        Err(Error::FinalIdentityMismatch)
    );
    let mut wrong = commit.clone();
    if let Body::Commit { final_crc32c, .. } = &mut wrong.body {
        *final_crc32c ^= 1
    }
    assert_eq!(
        validate_lifecycle(&[
            prefix[0].clone(),
            prefix[1].clone(),
            final_record.clone(),
            wrong
        ]),
        Err(Error::FinalCrcMismatch)
    );
    assert_eq!(validate_lifecycle(&[commit]), Err(Error::FinalNotAdjacent));
    assert_eq!(decode(&encoded), Ok(final_record));
}

#[test]
fn physical_ordinals_and_commit_order_must_strictly_advance() {
    let a = decode(&{
        let s = include_str!("data/r5-v1.hex").trim();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    })
    .unwrap();
    let mut b = a.clone();
    b.physical_ordinal = a.physical_ordinal;
    assert_eq!(validate_lifecycle(&[a, b]), Err(Error::OrdinalOrder));
}

fn binding(ordinal: u64, request: Uuid, event: Uuid) -> Record {
    Record {
        physical_ordinal: ordinal,
        integrity: IntegrityProfile::Structural,
        body: Body::Binding {
            request_id: request,
            event_id: event,
            normalized_request: vec![ordinal as u8],
        },
    }
}

fn reservation(ordinal: u64, request: Uuid, event: Uuid, sequence: u64) -> Record {
    Record {
        physical_ordinal: ordinal,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Reservation {
            request_id: request,
            event_id: event,
            sequence,
            high_water: sequence,
        },
    }
}

fn final_and_commit(ordinal: u64, request: Uuid, event: Uuid, sequence: u64) -> [Record; 2] {
    let final_record = Record {
        physical_ordinal: ordinal,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Final {
            event_id: event,
            request_id: request,
            sequence,
            durability_time: sequence as i64,
            complete_envelope: vec![sequence as u8],
        },
    };
    let encoded = encode(&final_record).unwrap();
    let commit = Record {
        physical_ordinal: ordinal + 1,
        integrity: IntegrityProfile::Crc32c,
        body: Body::Commit {
            event_id: event,
            sequence,
            final_ordinal: ordinal,
            final_crc32c: u32::from_le_bytes(encoded[28..32].try_into().unwrap()),
            group_id: 0,
            member_index: 0,
            member_count: 1,
        },
    };
    [final_record, commit]
}

#[test]
fn mixed_lifecycle_and_prefix_failures_preserve_rules_and_precedence() {
    let (request_a, event_a) = (id(10), id(20));
    let (request_b, event_b) = (id(11), id(21));
    let [final_a, commit_a] = final_and_commit(5, request_a, event_a, 1);
    let valid = vec![
        binding(1, request_a, event_a),
        reservation(2, request_a, event_a, 1),
        Record {
            physical_ordinal: 3,
            integrity: IntegrityProfile::Structural,
            body: Body::Provisional {
                event_id: event_a,
                sequence: 1,
                group_id: 0,
                member_index: 0,
                member_count: 1,
                stable_core: vec![7],
            },
        },
        Record {
            physical_ordinal: 4,
            integrity: IntegrityProfile::Structural,
            body: Body::Membership {
                group_id: 9,
                members: vec![(event_a, 1)],
            },
        },
        final_a.clone(),
        commit_a.clone(),
    ];
    assert_eq!(validate_lifecycle(&valid), Ok(()));

    let mut candidate = binding(7, request_a, event_b);
    assert_eq!(
        validate_lifecycle(&[valid.clone(), vec![candidate.clone()]].concat()),
        Err(Error::DuplicateIdentity)
    );
    candidate = reservation(7, request_b, event_b, 2);
    assert_eq!(
        validate_lifecycle(&[valid.clone(), vec![candidate]].concat()),
        Err(Error::MissingBinding)
    );

    let mut duplicate_sequence_prefix = valid.clone();
    duplicate_sequence_prefix.push(binding(7, request_b, event_b));
    duplicate_sequence_prefix.push(reservation(8, request_b, event_b, 1));
    assert_eq!(
        validate_lifecycle(&duplicate_sequence_prefix),
        Err(Error::DuplicateSequence)
    );

    let mut missing_reservation = valid.clone();
    missing_reservation.push(binding(7, request_b, event_b));
    missing_reservation.push(final_and_commit(8, request_b, event_b, 2)[0].clone());
    assert_eq!(
        validate_lifecycle(&missing_reservation),
        Err(Error::MissingReservation)
    );

    let mut duplicate_final = valid.clone();
    let mut repeated_final = final_a.clone();
    repeated_final.physical_ordinal = 7;
    duplicate_final.push(repeated_final);
    assert_eq!(
        validate_lifecycle(&duplicate_final),
        Err(Error::DuplicateFinal)
    );

    let mut nonadjacent = commit_a.clone();
    nonadjacent.physical_ordinal = 7;
    assert_eq!(
        validate_lifecycle(&[valid, vec![nonadjacent]].concat()),
        Err(Error::FinalNotAdjacent)
    );
}

#[test]
fn ordinal_arithmetic_commit_order_and_competing_errors_are_stable() {
    let first = binding(u64::MAX, id(30), id(40));
    let second = binding(1, id(31), id(41));
    assert_eq!(
        validate_lifecycle(&[first, second]),
        Err(Error::LengthOverflow)
    );

    let (request_a, event_a) = (id(50), id(60));
    let (request_b, event_b) = (id(51), id(61));
    let [final_a, commit_a] = final_and_commit(5, request_a, event_a, 2);
    let [final_b, commit_b] = final_and_commit(9, request_b, event_b, 1);
    let stream = vec![
        binding(1, request_a, event_a),
        reservation(2, request_a, event_a, 2),
        binding(3, request_b, event_b),
        reservation(4, request_b, event_b, 1),
        final_a,
        commit_a,
        Record {
            physical_ordinal: 7,
            integrity: IntegrityProfile::Structural,
            body: Body::Provisional {
                event_id: event_b,
                sequence: 1,
                group_id: 0,
                member_index: 0,
                member_count: 1,
                stable_core: vec![],
            },
        },
        Record {
            physical_ordinal: 8,
            integrity: IntegrityProfile::Structural,
            body: Body::Membership {
                group_id: 3,
                members: vec![(event_b, 1)],
            },
        },
        final_b,
        commit_b,
    ];
    assert_eq!(validate_lifecycle(&stream), Err(Error::SequenceOrder));

    let mut bad_ordinal_and_missing_binding = reservation(4, id(70), id(71), 1);
    bad_ordinal_and_missing_binding.physical_ordinal = 9;
    assert_eq!(
        validate_lifecycle(&[binding(1, id(72), id(73)), bad_ordinal_and_missing_binding]),
        Err(Error::OrdinalOrder)
    );
}
