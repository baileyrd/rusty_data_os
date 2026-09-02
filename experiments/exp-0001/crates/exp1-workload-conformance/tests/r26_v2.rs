//! R26 section 7 oracle tests. Expected bytes and hashes are checked-in literals.
use exp1_workload_conformance::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EventKind {
    Ordinary,
    Other,
}
#[derive(Clone, Debug)]
struct ReferenceEntry {
    namespace: [u8; 16],
    segment: Segment,
    ordinal: u64,
    kind: EventKind,
    fact_type: &'static str,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AcceptedPrefix {
    warm_up: u64,
    measured: u64,
}

fn accept_with_oracle(
    state: &mut AcceptedPrefix,
    op: &DecodedV2,
    catalog: &BTreeMap<[u8; 16], ReferenceEntry>,
    complete: bool,
) -> Result<(), Error> {
    let expected = match op.segment {
        Segment::WarmUp => state.warm_up,
        Segment::Measured => state.measured,
    };
    if op.ordinal != expected {
        return Err(Error::Ordering);
    }
    if expected > 0 && op.references.len() != 1 {
        return Err(Error::ReferenceCardinality);
    }
    for id in &op.references {
        if id == &op.event_id {
            return Err(Error::ReferenceSelf);
        }
        match catalog.get(id) {
            Some(entry) if entry.kind != EventKind::Ordinary => {
                return Err(Error::ReferenceWrongKind);
            }
            Some(entry) if entry.fact_type != "fact-A" => return Err(Error::ReferenceWrongFact),
            Some(entry) if entry.namespace != op.namespace => {
                return Err(Error::ReferenceCrossStream);
            }
            Some(entry) if entry.segment != op.segment => return Err(Error::ReferenceCrossSegment),
            Some(entry) if entry.ordinal >= op.ordinal => return Err(Error::ReferenceFuture),
            Some(_) => {}
            None if complete => return Err(Error::ReferenceMissing),
            None => return Err(Error::ContextRequired),
        }
    }
    match op.segment {
        Segment::WarmUp => state.warm_up += 1,
        Segment::Measured => state.measured += 1,
    }
    Ok(())
}

const DATA: &str = "data/r26-v2/";
const REF2_BOOTSTRAP_HEX: &str = include_str!("data/r26-v2/ref-bootstrap.hex");
const REF2_W1_HEX: &str = include_str!("data/r26-v2/ref-w1.hex");
const REF2_M1_HEX: &str = include_str!("data/r26-v2/ref-m1.hex");
const ENV2_W0_HEX: &str = include_str!("data/r26-v2/env-w0.hex");
const ENV2_W1_HEX: &str = include_str!("data/r26-v2/env-w1.hex");
const ENV2_M0_HEX: &str = include_str!("data/r26-v2/env-m0.hex");
const ENV2_M1_HEX: &str = include_str!("data/r26-v2/env-m1.hex");
const SOP2_W0_HEX: &str = include_str!("data/r26-v2/sop-w0.hex");
const SOP2_W1_HEX: &str = include_str!("data/r26-v2/sop-w1.hex");
const SOP2_M0_HEX: &str = include_str!("data/r26-v2/sop-m0.hex");
const SOP2_M1_HEX: &str = include_str!("data/r26-v2/sop-m1.hex");
const WS2_HEX: &str = include_str!("data/r26-v2/ws.hex");
const MANIFEST_JCS: &[u8] = include_bytes!("data/r26-v2/manifest.jcs");
const WS2_BYTE_LENGTH: usize = 3139;
const WS2_RAW_SHA256: &str = "7f2942ff8e4719688c23ea6ff3507496ce397a8ef767d52d0500cf8a928ac91a";
const WS2_DIGEST_V2: &str = "f1d0d28189680504617bd22c581ba12dab29bb6858909768c2f21180133845f7";
const MANIFEST_BYTE_LENGTH: usize = 3603;
const MANIFEST_RAW_SHA256: &str =
    "c92a230c17bbcbf3dd6bb35637232e757619f87d8628c82d5af372910ed7cd33";
const MANIFEST_DIGEST_V2: &str = "41207781b774ad8e8543b24f7228262cb16c94d9905eba30af72feb116a2328d";

fn literal(s: &str) -> Vec<u8> {
    decode_hex(s.trim()).unwrap()
}
fn validate_manifest_candidate(candidate: &[u8], stream: &[u8]) -> Result<(), Error> {
    let raw = hex(&sha256(candidate));
    let descriptor = hex(&manifest_digest_v2(candidate));
    validate_manifest_v2(
        candidate,
        &ValidationContextV2 {
            stream,
            warm_up_subsequent: 1,
            measured_subsequent: 1,
            manifest_artifact_sha256: &raw,
            manifest_artifact_length: candidate.len() as u64,
            descriptor_profile: MANIFEST_DIGEST_PROFILE_V2,
            descriptor_domain: MANIFEST_DOMAIN_V2,
            descriptor_value: &descriptor,
        },
    )
}
fn replace_once(source: &[u8], from: &str, to: &str) -> Vec<u8> {
    let text = std::str::from_utf8(source).unwrap();
    assert_eq!(text.matches(from).count(), 1, "mutation must be singular");
    text.replacen(from, to, 1).into_bytes()
}
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
fn generated(segment: Segment, ordinal: u64, references: &[[u8; 16]]) -> (Vec<u8>, Vec<u8>) {
    let op = operation(segment, ordinal);
    let request_id = identity(&op, IdentityKind::Request).unwrap();
    let event_id = identity(&op, IdentityKind::Event).unwrap();
    let information_id = identity(&op, IdentityKind::Information).unwrap();
    let env2 = envelope_input_v2(&EnvelopeInputV2 {
        common: EnvelopeInput {
            operation: &op,
            semantic_version: "2",
            fact_type: "fact-A",
            schema_id: parse_uuid("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
            schema_version: "1",
            source: None,
            actor: None,
            request_id,
            event_id,
            information_id,
            effective_time: 1000 + 10 * ordinal as i64,
            semantics: ReferenceSemantics::Causal,
            references,
        },
    })
    .unwrap();
    let sop2 = SemanticOperationV2 {
        op1: op.encode().unwrap(),
        payload_profile: "EXP-0001-SHA256-CTR-v1",
        payload: payload(&op).unwrap(),
        request_id,
        event_id,
        information_id,
        env2: env2.clone(),
        base_ns: 1000,
        unit_ns: 10,
    }
    .encode()
    .unwrap();
    (env2, sop2)
}

#[test]
fn positive_literal_oracles_realize_v25_01_through_v25_04() {
    assert_eq!(DATA, "data/r26-v2/");
    let (w0_env, w0) = generated(Segment::WarmUp, 0, &[]);
    let w0_id = validate_semantic_operation_v2(&literal(SOP2_W0_HEX))
        .unwrap()
        .event_id;
    let (w1_env, w1) = generated(Segment::WarmUp, 1, &[w0_id]);
    let (m0_env, m0) = generated(Segment::Measured, 0, &[]);
    let m0_id = validate_semantic_operation_v2(&literal(SOP2_M0_HEX))
        .unwrap()
        .event_id;
    let (m1_env, m1) = generated(Segment::Measured, 1, &[m0_id]);
    for (actual, expected) in [
        (w0_env, literal(ENV2_W0_HEX)),
        (w1_env, literal(ENV2_W1_HEX)),
        (m0_env, literal(ENV2_M0_HEX)),
        (m1_env, literal(ENV2_M1_HEX)),
        (w0.clone(), literal(SOP2_W0_HEX)),
        (w1.clone(), literal(SOP2_W1_HEX)),
        (m0.clone(), literal(SOP2_M0_HEX)),
        (m1.clone(), literal(SOP2_M1_HEX)),
    ] {
        assert_eq!(actual, expected);
    }
    assert_eq!(literal(REF2_BOOTSTRAP_HEX), [0, 0, 0, 0]);
    assert_eq!(literal(REF2_W1_HEX), [&[0, 0, 0, 1][..], &w0_id].concat());
    assert_eq!(literal(REF2_M1_HEX), [&[0, 0, 0, 1][..], &m0_id].concat());
    let generated_ws = workload_stream_v2(&[w0, w1, m0, m1], 2, 2).unwrap();
    let ws = literal(WS2_HEX);
    assert_eq!(generated_ws, ws);
    assert_eq!(ws.len(), WS2_BYTE_LENGTH);
    assert_eq!(hex(&sha256(&ws)), WS2_RAW_SHA256);
    assert_eq!(hex(&workload_digest_v2(&ws)), WS2_DIGEST_V2);
    assert_eq!(validate_stream_v2(&ws, 1, 1), Ok((4, 2, 2)));
}

#[test]
fn literal_manifest_and_external_digest_oracles_validate() {
    assert_eq!(MANIFEST_JCS.len(), MANIFEST_BYTE_LENGTH);
    assert_eq!(hex(&sha256(MANIFEST_JCS)), MANIFEST_RAW_SHA256);
    assert_eq!(hex(&manifest_digest_v2(MANIFEST_JCS)), MANIFEST_DIGEST_V2);
    let ws = literal(WS2_HEX);
    let context = ValidationContextV2 {
        stream: &ws,
        warm_up_subsequent: 1,
        measured_subsequent: 1,
        manifest_artifact_sha256: MANIFEST_RAW_SHA256,
        manifest_artifact_length: MANIFEST_BYTE_LENGTH as u64,
        descriptor_profile: MANIFEST_DIGEST_PROFILE_V2,
        descriptor_domain: MANIFEST_DOMAIN_V2,
        descriptor_value: MANIFEST_DIGEST_V2,
    };
    assert_eq!(validate_manifest_v2(MANIFEST_JCS, &context), Ok(()));
}

fn entry(id: [u8; 16], namespace: [u8; 16], segment: Segment) -> ReferenceEntry {
    let _ = id;
    ReferenceEntry {
        namespace,
        segment,
        ordinal: 0,
        kind: EventKind::Ordinary,
        fact_type: "fact-A",
    }
}
fn assert_transactional_error(
    op: &DecodedV2,
    catalog: &BTreeMap<[u8; 16], ReferenceEntry>,
    state: AcceptedPrefix,
    expected: Error,
) {
    let mut actual = state.clone();
    assert_eq!(
        accept_with_oracle(&mut actual, op, catalog, true),
        Err(expected)
    );
    assert_eq!(actual, state);
}

#[test]
fn v25_05_zero_targets_at_ordinal_one_is_cardinality() {
    let op = DecodedV2 {
        segment: Segment::WarmUp,
        ordinal: 1,
        namespace: operation(Segment::WarmUp, 1).stream_namespace,
        event_id: validate_semantic_operation_v2(&literal(SOP2_W1_HEX))
            .unwrap()
            .event_id,
        references: vec![],
    };
    assert_transactional_error(
        &op,
        &BTreeMap::new(),
        AcceptedPrefix {
            warm_up: 1,
            measured: 0,
        },
        Error::ReferenceCardinality,
    );
}

#[test]
fn v25_06a_measured_bootstrap_targeting_warm_up_is_cross_segment() {
    let warm = validate_semantic_operation_v2(&literal(SOP2_W0_HEX)).unwrap();
    let measured = validate_semantic_operation_v2(&literal(SOP2_M0_HEX)).unwrap();
    let op = DecodedV2 {
        references: vec![warm.event_id],
        ..measured
    };
    let catalog = BTreeMap::from([(
        warm.event_id,
        entry(warm.event_id, warm.namespace, Segment::WarmUp),
    )]);
    assert_transactional_error(
        &op,
        &catalog,
        AcceptedPrefix::default(),
        Error::ReferenceCrossSegment,
    );
}

#[test]
fn v25_06b_bootstrap_targeting_another_stream_is_cross_stream() {
    let boot = validate_semantic_operation_v2(&literal(SOP2_W0_HEX)).unwrap();
    let target = parse_uuid("25000000-0000-4000-8000-000000000099").unwrap();
    let op = DecodedV2 {
        references: vec![target],
        ..boot
    };
    let foreign = parse_uuid("26000000-0000-4000-8000-000000000001").unwrap();
    let catalog = BTreeMap::from([(target, entry(target, foreign, Segment::WarmUp))]);
    assert_transactional_error(
        &op,
        &catalog,
        AcceptedPrefix::default(),
        Error::ReferenceCrossStream,
    );
}

#[test]
fn malformed_duplicate_and_ordered_precedence_fail_without_state_advance() {
    let mut malformed = literal(SOP2_W0_HEX);
    malformed.pop();
    assert_eq!(
        validate_semantic_operation_v2(&malformed),
        Err(Error::Encoding)
    );
    let id = parse_uuid("25000000-0000-4000-8000-000000000020").unwrap();
    let namespace = operation(Segment::WarmUp, 1).stream_namespace;
    let self_op = DecodedV2 {
        segment: Segment::WarmUp,
        ordinal: 1,
        namespace,
        event_id: id,
        references: vec![id],
    };
    assert_transactional_error(
        &self_op,
        &BTreeMap::new(),
        AcceptedPrefix {
            warm_up: 1,
            measured: 0,
        },
        Error::ReferenceSelf,
    );
    let target = parse_uuid("25000000-0000-4000-8000-000000000021").unwrap();
    let op = DecodedV2 {
        references: vec![target],
        ..self_op
    };
    let mut catalog = BTreeMap::from([(
        target,
        ReferenceEntry {
            kind: EventKind::Other,
            fact_type: "fact-B",
            ordinal: 2,
            ..entry(target, namespace, Segment::WarmUp)
        },
    )]);
    let state = AcceptedPrefix {
        warm_up: 1,
        measured: 0,
    };
    assert_transactional_error(&op, &catalog, state.clone(), Error::ReferenceWrongKind);
    catalog.get_mut(&target).unwrap().kind = EventKind::Ordinary;
    assert_transactional_error(&op, &catalog, state.clone(), Error::ReferenceWrongFact);
    catalog.get_mut(&target).unwrap().fact_type = "fact-A";
    assert_transactional_error(&op, &catalog, state.clone(), Error::ReferenceFuture);
    catalog.clear();
    let mut incomplete = state.clone();
    assert_eq!(
        accept_with_oracle(&mut incomplete, &op, &catalog, false),
        Err(Error::ContextRequired)
    );
    assert_eq!(incomplete, state);
    assert_transactional_error(&op, &catalog, state, Error::ReferenceMissing);
}

#[test]
fn duplicate_target_bytes_precede_lookup() {
    let op = operation(Segment::WarmUp, 1);
    let target = parse_uuid("25000000-0000-4000-8000-000000000021").unwrap();
    let result = envelope_input_v2(&EnvelopeInputV2 {
        common: EnvelopeInput {
            operation: &op,
            semantic_version: "2",
            fact_type: "fact-A",
            schema_id: parse_uuid("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap(),
            schema_version: "1",
            source: None,
            actor: None,
            request_id: identity(&op, IdentityKind::Request).unwrap(),
            event_id: identity(&op, IdentityKind::Event).unwrap(),
            information_id: identity(&op, IdentityKind::Information).unwrap(),
            effective_time: 1010,
            semantics: ReferenceSemantics::Causal,
            references: &[target, target],
        },
    });
    assert_eq!(result, Err(Error::ReferenceDuplicate));
}

#[test]
fn v1_and_v2_are_strictly_incompatible() {
    let op = literal(SOP2_W0_HEX);
    assert_eq!(validate_semantic_operation(&op), Err(Error::Encoding));
    let mut mixed = op;
    mixed[5] = b'1';
    assert_eq!(validate_semantic_operation_v2(&mixed), Err(Error::Encoding));
}

#[test]
fn manifest_is_closed_and_number_free_at_every_depth() {
    let ws = literal(WS2_HEX);
    for candidate in [
        replace_once(
            MANIFEST_JCS,
            "\"authority\":\"EXP-0000-WORKLOADS\",\"revision\":{\"kind\"",
            "\"authority\":\"EXP-0000-WORKLOADS\",\"revision\":{\"aaa\":\"x\",\"kind\"",
        ),
        replace_once(
            MANIFEST_JCS,
            "\"artifact_manifest_ref\":{\"artifact_id\"",
            "\"artifact_manifest_ref\":{\"aaa\":\"x\",\"artifact_id\"",
        ),
    ] {
        assert_eq!(
            validate_manifest_candidate(&candidate, &ws),
            Err(Error::UnknownField)
        );
    }
    for candidate in [
        replace_once(
            MANIFEST_JCS,
            "\"measured\":{\"bootstrap\":\"0\"",
            "\"measured\":{\"bootstrap\":0",
        ),
        replace_once(
            MANIFEST_JCS,
            "\"count\":\"4\",\"profile\":\"envelope-causal-reference-v2\"",
            "\"count\":4,\"profile\":\"envelope-causal-reference-v2\"",
        ),
    ] {
        assert_eq!(
            validate_manifest_candidate(&candidate, &ws),
            Err(Error::Type)
        );
    }
}

#[test]
fn every_manifest_v2_profile_position_rejects_v1() {
    let ws = literal(WS2_HEX);
    for (key, v2, v1) in [
        (
            "workload_contract",
            WORKLOAD_CONTRACT_V2,
            "EXP-0000-WORKLOADS-v1",
        ),
        ("envelope", ENVELOPE_PROFILE_V2, "envelope-causal-reference"),
        (
            "envelope_generator",
            ENVELOPE_GENERATOR_V2,
            "EXP-0001-ENVELOPE-INPUT-v1",
        ),
        (
            "reference_generator",
            REFERENCE_GENERATOR_V2,
            "EXP-0001-PRIOR-EVENTS-v1",
        ),
        (
            "semantic_operation",
            SEMANTIC_OPERATION_V2,
            "EXP-0001-SEMANTIC-OP-v1",
        ),
        (
            "workload_stream",
            WORKLOAD_STREAM_V2,
            "EXP-0001-WORKLOAD-STREAM-v1",
        ),
        (
            "manifest",
            MANIFEST_PROFILE_V2,
            "EXP-0001-WORKLOAD-MANIFEST-JCS-v1",
        ),
    ] {
        let candidate = replace_once(
            MANIFEST_JCS,
            &format!("\"{key}\":\"{v2}\""),
            &format!("\"{key}\":\"{v1}\""),
        );
        assert_eq!(
            validate_manifest_candidate(&candidate, &ws),
            Err(Error::ProfileMismatch)
        );
    }
    let raw = hex(&sha256(MANIFEST_JCS));
    let digest = hex(&manifest_digest_v2(MANIFEST_JCS));
    for (profile, domain) in [
        ("EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1", MANIFEST_DOMAIN_V2),
        (MANIFEST_DIGEST_PROFILE_V2, MANIFEST_DOMAIN),
    ] {
        assert_eq!(
            validate_manifest_v2(
                MANIFEST_JCS,
                &ValidationContextV2 {
                    stream: &ws,
                    warm_up_subsequent: 1,
                    measured_subsequent: 1,
                    manifest_artifact_sha256: &raw,
                    manifest_artifact_length: MANIFEST_JCS.len() as u64,
                    descriptor_profile: profile,
                    descriptor_domain: domain,
                    descriptor_value: &digest,
                }
            ),
            Err(Error::ProfileMismatch)
        );
    }
}

#[test]
fn manifest_counts_stream_bindings_and_digests_are_exact() {
    let ws = literal(WS2_HEX);
    for (from, to, error) in [
        (
            "\"operation_count\":\"4\"",
            "\"operation_count\":\"5\"",
            Error::CountMismatch,
        ),
        (
            "\"stream_namespace\":\"25000000-0000-4000-8000-000000000001\"",
            "\"stream_namespace\":\"25000000-0000-4000-8000-000000000099\"",
            Error::ProfileMismatch,
        ),
        (
            WS2_RAW_SHA256,
            "0f2942ff8e4719688c23ea6ff3507496ce397a8ef767d52d0500cf8a928ac91a",
            Error::Digest,
        ),
        (
            WS2_DIGEST_V2,
            "01d0d28189680504617bd22c581ba12dab29bb6858909768c2f21180133845f7",
            Error::Digest,
        ),
    ] {
        let candidate = replace_once(MANIFEST_JCS, from, to);
        assert_eq!(validate_manifest_candidate(&candidate, &ws), Err(error));
    }
    for (value, expected) in [
        ("18446744073709551615", Error::CountMismatch),
        ("18446744073709551616", Error::Range),
        ("01", Error::Range),
    ] {
        let candidate = replace_once(
            MANIFEST_JCS,
            "\"measured\":{\"bootstrap\":\"0\",\"subsequent\":\"1\"}",
            &format!("\"measured\":{{\"bootstrap\":\"0\",\"subsequent\":\"{value}\"}}"),
        );
        assert_eq!(validate_manifest_candidate(&candidate, &ws), Err(expected));
    }
    let mut bad_length = ws.clone();
    let header = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2".len();
    bad_length[header..header + 8].copy_from_slice(&u64::MAX.to_be_bytes());
    assert_eq!(
        validate_manifest_candidate(MANIFEST_JCS, &bad_length),
        Err(Error::CountMismatch)
    );
}

#[test]
fn binary_v1_substitutions_are_rejected_at_each_versioned_layer() {
    let mut sop = literal(SOP2_W0_HEX);
    sop[8] = b'1';
    let mut nested_env = literal(SOP2_W0_HEX);
    let env_magic = nested_env
        .windows(9)
        .position(|part| part == b"RDOS-ENV2")
        .unwrap();
    nested_env[env_magic + 8] = b'1';
    let mut ws = literal(WS2_HEX);
    ws[7] = b'1';
    assert_eq!(validate_semantic_operation_v2(&sop), Err(Error::Encoding));
    assert_eq!(
        validate_semantic_operation_v2(&nested_env),
        Err(Error::Encoding)
    );
    assert_eq!(validate_stream_v2(&ws, 1, 1), Err(Error::Encoding));
}
