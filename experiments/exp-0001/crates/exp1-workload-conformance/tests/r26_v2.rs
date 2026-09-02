//! Literal R26 section 7 oracle corpus.  The constants are test data, not runtime generation.
use exp1_workload_conformance::*;
use std::collections::BTreeMap;

const REF2_BOOTSTRAP_HEX: &str = "00000000";
const STREAM_DOMAIN_DIGEST_EMPTY: &str =
    "f713960372d63c833548ca5a81e14b9371751286846c0344cb372e6ef955127c";

fn operation(segment: Segment, ordinal: u64) -> OperationInput {
    OperationInput {
        segment,
        seed: 25,
        ordinal,
        size_class: 1,
        content: Content::High,
        envelope: Envelope::Causal,
        temporal: Temporal::Monotonic,
        stream_namespace: parse_uuid("25000000-0000-4000-8000-000000000001").unwrap(),
        producer_id: parse_uuid("25000000-0000-4000-8000-000000000002").unwrap(),
        producer_ordinal: ordinal,
        controlled_schedule: None,
    }
}
fn sop(segment: Segment, ordinal: u64, references: &[[u8; 16]]) -> Vec<u8> {
    let op = operation(segment, ordinal);
    let request = identity(&op, IdentityKind::Request).unwrap();
    let event = identity(&op, IdentityKind::Event).unwrap();
    let information = identity(&op, IdentityKind::Information).unwrap();
    let env = envelope_input_v2(&EnvelopeInputV2 {
        common: EnvelopeInput {
            operation: &op,
            semantic_version: "2",
            fact_type: "fact-A",
            schema_id: parse_uuid("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
            schema_version: "1",
            source: None,
            actor: None,
            request_id: request,
            event_id: event,
            information_id: information,
            effective_time: 1000 + 10 * ordinal as i64,
            semantics: ReferenceSemantics::Causal,
            references,
        },
    })
    .unwrap();
    SemanticOperationV2 {
        op1: op.encode().unwrap(),
        payload_profile: "EXP-0001-SHA256-CTR-v1",
        payload: payload(&op).unwrap(),
        request_id: request,
        event_id: event,
        information_id: information,
        env2: env,
        base_ns: 1000,
        unit_ns: 10,
    }
    .encode()
    .unwrap()
}

#[test]
fn v25_01_through_04_bootstraps_and_prior_references() {
    assert_eq!(
        hex(&decode_hex(REF2_BOOTSTRAP_HEX).unwrap()),
        REF2_BOOTSTRAP_HEX
    );
    let w0 = sop(Segment::WarmUp, 0, &[]);
    let id0 = validate_semantic_operation_v2(&w0).unwrap().event_id;
    let w1 = sop(Segment::WarmUp, 1, &[id0]);
    let m0 = sop(Segment::Measured, 0, &[]);
    let mid0 = validate_semantic_operation_v2(&m0).unwrap().event_id;
    let m1 = sop(Segment::Measured, 1, &[mid0]);
    let ws = workload_stream_v2(&[w0, w1, m0, m1], 2, 2).unwrap();
    assert_eq!(validate_stream_v2(&ws, 1, 1), Ok((4, 2, 2)));
    assert_ne!(workload_digest_v2(&ws), sha256(&ws));
    assert_eq!(hex(&workload_digest_v2(b"")), STREAM_DOMAIN_DIGEST_EMPTY);
}

#[test]
fn v25_05_06_precedence_and_transactionality() {
    let current = DecodedV2 {
        segment: Segment::WarmUp,
        ordinal: 1,
        namespace: parse_uuid("25000000-0000-4000-8000-000000000001").unwrap(),
        event_id: parse_uuid("25000000-0000-4000-8000-000000000010").unwrap(),
        references: vec![parse_uuid("25000000-0000-4000-8000-000000000020").unwrap()],
    };
    let mut catalog = BTreeMap::new();
    catalog.insert(
        current.references[0],
        ReferenceEntry {
            id: current.references[0],
            namespace: current.namespace,
            segment: Segment::Measured,
            ordinal: 0,
            kind: EventKind::Ordinary,
            fact_type: "fact-A".into(),
        },
    );
    assert_eq!(
        classify_references(&current, "fact-A", &catalog, true),
        Err(Error::ReferenceCrossSegment)
    );
    catalog.get_mut(&current.references[0]).unwrap().namespace =
        parse_uuid("25000000-0000-4000-8000-000000000099").unwrap();
    assert_eq!(
        classify_references(&current, "fact-A", &catalog, true),
        Err(Error::ReferenceCrossStream)
    );
    let mut state = AcceptedPrefix {
        warm_up: 1,
        measured: 0,
    };
    let before = state.clone();
    assert!(accept_transactionally(&mut state, &current, "fact-A", &catalog, true, 1).is_err());
    assert_eq!(state, before);
}

#[test]
fn malformed_duplicate_self_future_wrong_kind_wrong_fact_missing_and_context() {
    let boot = sop(Segment::WarmUp, 0, &[]);
    let mut bad = boot.clone();
    bad.pop();
    assert_eq!(validate_semantic_operation_v2(&bad), Err(Error::Encoding));
    let id = parse_uuid("25000000-0000-4000-8000-000000000020").unwrap();
    let op = DecodedV2 {
        segment: Segment::WarmUp,
        ordinal: 1,
        namespace: parse_uuid("25000000-0000-4000-8000-000000000001").unwrap(),
        event_id: id,
        references: vec![id],
    };
    let mut c = BTreeMap::new();
    assert_eq!(
        classify_references(&op, "fact-A", &c, true),
        Err(Error::ReferenceSelf)
    );
    let target = parse_uuid("25000000-0000-4000-8000-000000000021").unwrap();
    let op = DecodedV2 {
        references: vec![target],
        ..op
    };
    c.insert(
        target,
        ReferenceEntry {
            id: target,
            namespace: op.namespace,
            segment: op.segment,
            ordinal: 2,
            kind: EventKind::Other,
            fact_type: "fact-B".into(),
        },
    );
    assert_eq!(
        classify_references(&op, "fact-A", &c, true),
        Err(Error::ReferenceWrongKind)
    );
    c.get_mut(&target).unwrap().kind = EventKind::Ordinary;
    assert_eq!(
        classify_references(&op, "fact-A", &c, true),
        Err(Error::ReferenceWrongFact)
    );
    c.get_mut(&target).unwrap().fact_type = "fact-A".into();
    assert_eq!(
        classify_references(&op, "fact-A", &c, true),
        Err(Error::ReferenceFuture)
    );
    c.clear();
    assert_eq!(
        classify_references(&op, "fact-A", &c, false),
        Err(Error::ContextRequired)
    );
    assert_eq!(
        classify_references(&op, "fact-A", &c, true),
        Err(Error::ReferenceMissing)
    );
}

#[test]
fn v1_and_v2_are_strictly_incompatible() {
    let op = sop(Segment::WarmUp, 0, &[]);
    assert_eq!(validate_semantic_operation(&op), Err(Error::Encoding));
    let mut mixed = op.clone();
    mixed[5] = b'1';
    assert_eq!(validate_semantic_operation_v2(&mixed), Err(Error::Encoding));
}
