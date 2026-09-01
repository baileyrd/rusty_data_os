use exp1_raw_append_replay::mapping::*;
use exp1_record_format::{Body, IntegrityProfile, Record, Uuid, decode};
use exp1_workload_conformance::{
    Content, Envelope, EnvelopeInput, IdentityKind, OperationInput, ReferenceSemantics, Segment,
    SemanticOperation, Temporal, envelope_input, hex, identity, parse_uuid, payload, sha256,
};

#[allow(clippy::too_many_arguments)] // Mirrors the frozen EnvelopeInput fixture dimensions.
fn operation(
    ordinal: u64,
    size_class: u8,
    envelope: Envelope,
    semantics: ReferenceSemantics,
    fact_type: &str,
    source: Option<&str>,
    actor: Option<&str>,
    references: &[[u8; 16]],
) -> Vec<u8> {
    let input = OperationInput {
        segment: Segment::WarmUp,
        seed: 0,
        ordinal,
        size_class,
        content: Content::High,
        envelope,
        temporal: if envelope == Envelope::Provenance {
            Temporal::EqualBurst
        } else {
            Temporal::Monotonic
        },
        stream_namespace: parse_uuid("00112233-4455-4677-8899-aabbccddeeff").unwrap(),
        producer_id: parse_uuid("10213243-5465-4768-899a-abbccddeef00").unwrap(),
        producer_ordinal: ordinal,
        controlled_schedule: None,
    };
    let request_id = identity(&input, IdentityKind::Request).unwrap();
    let event_id = identity(&input, IdentityKind::Event).unwrap();
    let information_id = identity(&input, IdentityKind::Information).unwrap();
    let env1 = envelope_input(&EnvelopeInput {
        operation: &input,
        semantic_version: "1",
        fact_type,
        schema_id: parse_uuid("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
        schema_version: "1",
        source,
        actor,
        request_id,
        event_id,
        information_id,
        effective_time: if envelope == Envelope::Provenance {
            1000
        } else {
            1000 + i64::try_from(ordinal).unwrap() * 10
        },
        semantics,
        references,
    })
    .unwrap();
    SemanticOperation {
        op1: input.encode().unwrap(),
        payload_profile: "EXP-0001-SHA256-CTR-v1",
        payload: payload(&input).unwrap(),
        request_id,
        event_id,
        information_id,
        env1,
        base_ns: 1000,
        unit_ns: 10,
    }
    .encode()
    .unwrap()
}

fn map(bytes: &[u8], sequence: u64, ordinal: u64, state: MappingState) -> MappedRecord {
    map_semantic_operation(bytes, sequence, ordinal, state).unwrap()
}

#[test]
fn r20_documentation_vectors_are_exact() {
    let s01 = operation(
        0,
        1,
        Envelope::Minimal,
        ReferenceSemantics::None,
        "fact-A",
        None,
        None,
        &[],
    );
    let first = map(&s01, 1, 1, MappingState::initial());
    assert_eq!(first.frame.len(), 827);
    assert_eq!(
        hex(&first.frame[..72]),
        "52444531010003003b0300001b03000001000000000000000000000000000000330f201aea7c4335a8ece6fe23266a1c0100000000000000000000000000000000000100f3020000"
    );
    assert_eq!(
        hex(&sha256(&first.frame)),
        "32b63591f5e20ac37e25478d3cdcaca5ad7310be07c32ccdbb3c28bad2c1c9b7"
    );
    assert_eq!(decode(&first.frame).unwrap(), first.record);
    assert_eq!(
        first.record,
        Record {
            physical_ordinal: 1,
            integrity: IntegrityProfile::Structural,
            body: Body::Provisional {
                event_id: Uuid(parse_uuid("330f201a-ea7c-4335-a8ec-e6fe23266a1c").unwrap()),
                sequence: 1,
                group_id: 0,
                member_index: 0,
                member_count: 1,
                stable_core: s01
            }
        }
    );

    let s02 = operation(
        1,
        1,
        Envelope::Provenance,
        ReferenceSemantics::None,
        "fact-A",
        Some("source-A"),
        Some("actor-A"),
        &[],
    );
    let second = map(&s02, 2, 2, first.next_state);
    assert_eq!(second.frame.len(), 842);
    assert_eq!(
        hex(&second.frame[..72]),
        "52444531010003004a0300002a03000002000000000000000000000000000000c57a25cf26e64dbaad56ea7cec2a4865020000000000000000000000000000000000010002030000"
    );
    assert_eq!(
        hex(&sha256(&second.frame)),
        "df4f1358a51683aaf1c8bcd2663c4369d5ed08ae03ac173ae0d6860955e44ff3"
    );
    assert_eq!(decode(&second.frame).unwrap(), second.record);
}

#[test]
fn all_operation_cases_and_payload_boundaries_preserve_the_complete_core() {
    let ordinary = operation(
        0,
        0,
        Envelope::Minimal,
        ReferenceSemantics::None,
        "fact-A",
        None,
        None,
        &[],
    );
    let prior0 = identity_for(0);
    let prior1 = identity_for(1);
    let cases = [
        ordinary,
        operation(
            1,
            1,
            Envelope::Provenance,
            ReferenceSemantics::None,
            "fact-A",
            Some("source-A"),
            Some("actor-A"),
            &[],
        ),
        operation(
            2,
            1,
            Envelope::Causal,
            ReferenceSemantics::Causal,
            "fact-A",
            None,
            None,
            &[prior0],
        ),
        operation(
            3,
            1,
            Envelope::CorrectionRetraction,
            ReferenceSemantics::Correction,
            "correction-A",
            None,
            None,
            &[prior1],
        ),
        operation(
            4,
            5,
            Envelope::CorrectionRetraction,
            ReferenceSemantics::Retraction,
            "retraction-A",
            None,
            None,
            &[prior1],
        ),
    ];
    let mut state = MappingState::initial();
    for (index, sop) in cases.iter().enumerate() {
        let value = u64::try_from(index + 1).unwrap();
        let mapped = map(sop, value * 10, value, state);
        match &mapped.record.body {
            Body::Provisional { stable_core, .. } => assert_eq!(stable_core, sop),
            _ => panic!("R20 always constructs type 3"),
        }
        state = mapped.next_state;
    }
}

fn identity_for(ordinal: u64) -> [u8; 16] {
    let input = OperationInput {
        segment: Segment::WarmUp,
        seed: 0,
        ordinal,
        size_class: 1,
        content: Content::High,
        envelope: Envelope::Minimal,
        temporal: Temporal::Monotonic,
        stream_namespace: parse_uuid("00112233-4455-4677-8899-aabbccddeeff").unwrap(),
        producer_id: parse_uuid("10213243-5465-4768-899a-abbccddeef00").unwrap(),
        producer_ordinal: ordinal,
        controlled_schedule: None,
    };
    identity(&input, IdentityKind::Event).unwrap()
}

#[test]
fn state_is_distinct_checked_and_gaps_are_legal() {
    let sop = operation(
        0,
        0,
        Envelope::Minimal,
        ReferenceSemantics::None,
        "fact-A",
        None,
        None,
        &[],
    );
    let first = map(&sop, 7, 1, MappingState::initial());
    let second = map(&sop, 99, 2, first.next_state);
    assert_eq!(
        second.next_state,
        MappingState {
            previous_sequence: 99,
            previous_physical_ordinal: 2
        }
    );

    for (sequence, ordinal, state, expected) in [
        (0, 1, MappingState::initial(), StateError::ZeroSequence),
        (
            1,
            0,
            MappingState::initial(),
            StateError::ZeroPhysicalOrdinal,
        ),
        (
            1,
            2,
            MappingState::initial(),
            StateError::NonconsecutivePhysicalOrdinal,
        ),
        (7, 2, first.next_state, StateError::DuplicateSequence),
        (6, 2, first.next_state, StateError::DecreasingSequence),
        (
            1,
            1,
            MappingState {
                previous_sequence: u64::MAX,
                previous_physical_ordinal: 0,
            },
            StateError::SequenceExhausted,
        ),
        (
            u64::MAX,
            1,
            MappingState {
                previous_sequence: 0,
                previous_physical_ordinal: u64::MAX,
            },
            StateError::PhysicalOrdinalExhausted,
        ),
    ] {
        assert_eq!(
            map_semantic_operation(&sop, sequence, ordinal, state),
            Err(MappingError::State(expected))
        );
    }
    let final_sequence = map(&sop, u64::MAX, 1, MappingState::initial());
    assert_eq!(
        map_semantic_operation(&sop, u64::MAX, 2, final_sequence.next_state),
        Err(MappingError::State(StateError::SequenceExhausted))
    );
}

#[test]
fn semantic_failures_return_no_value_and_cannot_advance_owned_state() {
    let sop = operation(
        0,
        0,
        Envelope::Minimal,
        ReferenceSemantics::None,
        "fact-A",
        None,
        None,
        &[],
    );
    let state = MappingState::initial();
    for bad in [&sop[..sop.len() - 1], &[0_u8][..]] {
        assert!(matches!(
            map_semantic_operation(bad, 1, 1, state),
            Err(MappingError::SemanticValidation(_))
        ));
        assert_eq!(state, MappingState::initial());
    }
    let mut invalid_uuid = sop.clone();
    let mapped = map(&sop, 1, 1, state);
    let Body::Provisional {
        event_id: Uuid(event_id),
        ..
    } = mapped.record.body
    else {
        unreachable!()
    };
    let event = invalid_uuid
        .windows(16)
        .position(|w| w == event_id)
        .unwrap();
    invalid_uuid[event + 6] &= 0x0f;
    assert!(matches!(
        map_semantic_operation(&invalid_uuid, 1, 1, state),
        Err(MappingError::SemanticValidation(_))
    ));
}
