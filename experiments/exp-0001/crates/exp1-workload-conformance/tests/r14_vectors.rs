mod vectors;
use exp1_workload_conformance::*;
use vectors::*;
#[test]
fn operation_vectors() {
    let a = decode_hex(S01).unwrap();
    let b = decode_hex(S02).unwrap();
    assert_eq!((a.len(), b.len()), (755, 770));
    validate_semantic_operation(&a).unwrap();
    validate_semantic_operation(&b).unwrap();
    assert_eq!(
        hex(&sha256(&a)),
        "efa80d1b021e590b8ac02b49a9bb0e68277cf39f32f3849aceabb33e2ec9b83c"
    );
    assert_eq!(
        hex(&sha256(&b)),
        "85a917fe5d4ef24e1904cb6b8ac2554fa60f99ae6f0c69db5e72cf6d81628ddf"
    );
}

fn operation(envelope: Envelope, semantics: ReferenceSemantics, fact: &str) -> SemanticOperation {
    let input = OperationInput {
        segment: Segment::WarmUp,
        seed: 0,
        ordinal: 1,
        size_class: 1,
        content: Content::High,
        envelope,
        temporal: Temporal::Monotonic,
        stream_namespace: parse_uuid("00112233-4455-4677-8899-aabbccddeeff").unwrap(),
        producer_id: parse_uuid("10213243-5465-4768-899a-abbccddeef00").unwrap(),
        producer_ordinal: 1,
        controlled_schedule: None,
    };
    let request = identity(&input, IdentityKind::Request).unwrap();
    let event = identity(&input, IdentityKind::Event).unwrap();
    let information = identity(&input, IdentityKind::Information).unwrap();
    let references = [parse_uuid("330f201a-ea7c-4335-a8ec-e6fe23266a1c").unwrap()];
    let env = envelope_input(&EnvelopeInput {
        operation: &input,
        semantic_version: "1",
        fact_type: fact,
        schema_id: parse_uuid("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
        schema_version: "1",
        source: None,
        actor: None,
        request_id: request,
        event_id: event,
        information_id: information,
        effective_time: 1010,
        semantics,
        references: &references,
    })
    .unwrap();
    let op1 = input.encode().unwrap();
    let payload = payload(&input).unwrap();
    SemanticOperation {
        op1,
        payload_profile: "EXP-0001-SHA256-CTR-v1",
        payload,
        request_id: request,
        event_id: event,
        information_id: information,
        env1: env,
        base_ns: 1000,
        unit_ns: 10,
    }
}

#[test]
fn reference_profiles_and_encoder_fail_closed() {
    for (profile, semantics, fact) in [
        (Envelope::Causal, ReferenceSemantics::Causal, "fact-A"),
        (
            Envelope::CorrectionRetraction,
            ReferenceSemantics::Correction,
            "correction-A",
        ),
        (
            Envelope::CorrectionRetraction,
            ReferenceSemantics::Retraction,
            "retraction-A",
        ),
    ] {
        let encoded = operation(profile, semantics, fact).encode().unwrap();
        validate_semantic_operation(&encoded).unwrap();
    }

    let valid = operation(Envelope::Causal, ReferenceSemantics::Causal, "fact-A");
    let mut bad = valid.clone();
    bad.payload_profile = "EXP-0001-ZERO-v1";
    assert_eq!(bad.encode(), Err(Error::ProfileMismatch));
    let mut bad = valid.clone();
    bad.payload[0] ^= 1;
    assert_eq!(bad.encode(), Err(Error::ProfileMismatch));
    let mut bad = valid.clone();
    bad.request_id[0] ^= 1;
    assert_eq!(bad.encode(), Err(Error::ProfileMismatch));
    let mut bad = valid;
    bad.base_ns += 1;
    assert_eq!(bad.encode(), Err(Error::ProfileMismatch));
}
#[test]
fn stream_vectors() {
    let empty = decode_hex(W00).unwrap();
    assert_eq!(empty.len(), 55);
    assert_eq!(workload_stream(&[], 0, 0).unwrap(), empty);
    assert_eq!(
        hex(&workload_digest(&empty)),
        "6ed7e39756dab1b00e5860365288a35b7b8d40f92bc8d219de50eb633144d387"
    );
    let w = decode_hex(W01).unwrap();
    assert_eq!(w.len(), 1596);
    assert_eq!(validate_stream(&w), Ok((2, 2, 0)));
    assert_eq!(
        hex(&workload_digest(&w)),
        "81dbc6b6e33ee775d4b36aeaa0aca45b9649c987f180e378b5d5fbcf1bc3b024"
    );
    assert_eq!(
        workload_stream(&[decode_hex(S01).unwrap(), decode_hex(S02).unwrap()], 2, 0).unwrap(),
        w
    );
}
