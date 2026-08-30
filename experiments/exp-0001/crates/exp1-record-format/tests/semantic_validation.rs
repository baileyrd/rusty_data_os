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
