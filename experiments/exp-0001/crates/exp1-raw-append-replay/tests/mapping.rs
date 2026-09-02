use exp1_raw_append_replay::mapping::*;
use exp1_record_format::{Body, IntegrityProfile, Record, Uuid, decode};
use exp1_workload_conformance::{
    Content, Envelope, EnvelopeInput, IdentityKind, OperationInput, ReferenceSemantics, Segment,
    SemanticOperation, Temporal, envelope_input, hex, identity, parse_uuid, payload, sha256,
};

// R27 contextual mapping is exercised with its independently checked-in literal oracle in
// `reference_context.rs`; these legacy imports and cases intentionally remain unchanged.

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
    let second = map(&s02, 2, 2, first.next_state());
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
        state = mapped.next_state();
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

fn field_range(bytes: &[u8], magic: &[u8], wanted: u8) -> std::ops::Range<usize> {
    let mut position = magic.len() + 2;
    loop {
        let tag = bytes[position];
        let length = u32::from_be_bytes(bytes[position + 1..position + 5].try_into().unwrap());
        let start = position + 5;
        let end = start + usize::try_from(length).unwrap();
        if tag == wanted {
            return start..end;
        }
        position = end;
    }
}

fn replace_field(bytes: &[u8], magic: &[u8], tag: u8, replacement: &[u8]) -> Vec<u8> {
    let range = field_range(bytes, magic, tag);
    let mut out = bytes.to_vec();
    let length_start = range.start - 4;
    out[length_start..range.start]
        .copy_from_slice(&u32::try_from(replacement.len()).unwrap().to_be_bytes());
    out.splice(range, replacement.iter().copied());
    out
}

#[test]
fn authoritative_profiles_versions_and_kinds_reject_before_mapping() {
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
    let op_range = field_range(&sop, b"RDOS-SOP1", 1);
    let env_range = field_range(&sop, b"RDOS-SOP1", 9);
    let mut cases = Vec::new();

    // Every SOP1 profile selector validated by the authoritative one-operation validator.
    for tag in [2, 4, 8, 10, 11] {
        let mut bad = sop.clone();
        let range = field_range(&bad, b"RDOS-SOP1", tag);
        bad[range.start] ^= 1;
        cases.push(bad);
    }
    // Both OP1 versions and each one-byte OP1 enum/kind selector.
    for tag in [1, 2, 3, 8, 9, 10] {
        let mut bad = sop.clone();
        let relative = field_range(&bad[op_range.clone()], b"RDOS-OP1", tag);
        bad[op_range.start + relative.start] = 0xff;
        cases.push(bad);
    }
    // ENV1 semantic version and reference-semantics kind.
    for tag in [2, 12] {
        let mut bad = sop.clone();
        let relative = field_range(&bad[env_range.clone()], b"RDOS-ENV1", tag);
        bad[env_range.start + relative.start] = 0xff;
        cases.push(bad);
    }

    for bad in cases {
        assert!(matches!(
            map_semantic_operation(&bad, 1, 1, MappingState::initial()),
            Err(MappingError::SemanticValidation(_))
        ));
    }
}

#[test]
fn duplicate_reference_bytes_reject_but_membership_requires_unfrozen_context() {
    let causal = operation(
        2,
        1,
        Envelope::Causal,
        ReferenceSemantics::Causal,
        "fact-A",
        None,
        None,
        &[identity_for(0)],
    );
    let env_range = field_range(&causal, b"RDOS-SOP1", 9);
    let env = &causal[env_range];
    let references = &env[field_range(env, b"RDOS-ENV1", 13)];
    let mut duplicate = Vec::from(2_u32.to_be_bytes());
    duplicate.extend_from_slice(&references[4..20]);
    duplicate.extend_from_slice(&references[4..20]);
    let invalid_env = replace_field(env, b"RDOS-ENV1", 13, &duplicate);
    let invalid_sop = replace_field(&causal, b"RDOS-SOP1", 9, &invalid_env);

    assert!(matches!(
        map_semantic_operation(&invalid_sop, 1, 1, MappingState::initial()),
        Err(MappingError::SemanticValidation(_))
    ));
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
    let second = map(&sop, 99, 2, first.next_state());
    assert_eq!(second.next_state().previous_sequence(), 99);
    assert_eq!(second.next_state().previous_physical_ordinal(), 2);

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
        (7, 2, first.next_state(), StateError::DuplicateSequence),
        (6, 2, first.next_state(), StateError::DecreasingSequence),
    ] {
        assert_eq!(
            map_semantic_operation(&sop, sequence, ordinal, state),
            Err(MappingError::State(expected))
        );
    }
    let final_sequence = map(&sop, u64::MAX, 1, MappingState::initial());
    assert_eq!(
        map_semantic_operation(&sop, u64::MAX, 2, final_sequence.next_state()),
        Err(MappingError::State(StateError::SequenceExhausted))
    );
}

#[test]
fn failed_mapping_cannot_produce_or_advance_state() {
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
    let accepted = map(&sop, 7, 1, MappingState::initial());
    let retained = accepted.next_state();
    assert_eq!(
        map_semantic_operation(&sop, 7, 2, retained),
        Err(MappingError::State(StateError::DuplicateSequence))
    );
    assert_eq!(retained.previous_sequence(), 7);
    assert_eq!(retained.previous_physical_ordinal(), 1);
    let resumed = map(&sop, 8, 2, retained);
    assert_eq!(resumed.next_state().previous_sequence(), 8);
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
