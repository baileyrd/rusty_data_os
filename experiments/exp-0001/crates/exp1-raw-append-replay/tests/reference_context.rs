use exp1_raw_append_replay::reference_context::*;
use exp1_workload_conformance::{hex, manifest_digest_v2, sha256, workload_digest_v2};

const MANIFEST: &[u8] =
    include_bytes!("../../exp1-workload-conformance/tests/data/r26-v2/manifest.jcs");
const WS_HEX: &str = include_str!("../../exp1-workload-conformance/tests/data/r26-v2/ws.hex");
const NS: [u8; 16] = [0x25, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 1];

fn decode_hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn metadata(id: &str, bytes: &[u8], role: &str, media: &str) -> Vec<u8> {
    format!(concat!(
        "{{\"body\":{{\"artifacts\":[{{\"artifact_id\":\"{}\",\"byte_length\":\"{}\",",
        "\"created_by_record_id\":\"16000000-0000-4000-8000-000000000006\",",
        "\"logical_path\":\"exp-0001/artifacts/{}\",\"media_type\":\"{}\",\"retention_state\":\"published\",",
        "\"role\":\"{}\",\"sensitivity\":\"public\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/{}\",\"validation_report_ids\":[]}}],",
        "\"provenance_edges\":[],\"publication_state\":\"published\",\"scope\":\"run\",\"series_freeze\":{{\"state\":\"not_applicable\"}}}},",
        "\"correction_reason\":{{\"state\":\"not_applicable\"}},\"created_at_utc_ns\":\"1788134400000000000\",",
        "\"record_id\":\"16000000-0000-4000-8000-000000000010\",\"record_kind\":\"artifact_manifest\",",
        "\"run_id\":{{\"state\":\"present\",\"value\":\"16000000-0000-4000-8000-000000000008\"}},",
        "\"schema_version\":\"EXP1-R7-JSON-JCS-1\",\"series_id\":\"16000000-0000-4000-8000-000000000007\",",
        "\"supersedes_record_id\":{{\"state\":\"not_applicable\"}}}}"),
        id, bytes.len(), id, media, role, hex(&sha256(bytes)), id).into_bytes()
}

struct Fixture {
    ws: Vec<u8>,
    descriptor: Vec<u8>,
    scope_digest: Vec<u8>,
    manifest_digest: Vec<u8>,
    manifest_meta: Vec<u8>,
    stream_meta: Vec<u8>,
}

fn fixture() -> Fixture {
    let ws = decode_hex(WS_HEX);
    let descriptor = format!(
        concat!(
            "{{\"cell_id\":\"PC-D1-raw-v2\",\"members\":[{{",
            "\"manifest_digest\":\"{}\",\"manifest_id\":\"16000000-0000-4000-8000-000000000001\",",
            "\"stream_artifact_sha256\":\"{}\",\"stream_byte_length\":\"{}\",",
            "\"stream_digest\":\"{}\",\"stream_namespace\":\"25000000-0000-4000-8000-000000000001\",",
            "\"workload_id\":\"16000000-0000-4000-8000-000000000003\"}}],",
            "\"record_kind\":\"closed_stream_scope\",\"schema_version\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v2\",",
            "\"scope_id\":\"27000000-0000-4000-8000-000000000001\"}}"
        ),
        hex(&manifest_digest_v2(MANIFEST)), hex(&sha256(&ws)), ws.len(),
        hex(&workload_digest_v2(&ws)),
    ).into_bytes();
    let scope_value = {
        let mut p = b"rusty-data-os/exp1/closed-stream-scope/v2\0".to_vec();
        p.extend(&descriptor);
        hex(&sha256(&p))
    };
    let scope_digest = format!(concat!(
        "{{\"algorithm\":\"SHA-256/FIPS-180-4\",\"domain\":\"rusty-data-os/exp1/closed-stream-scope/v2\",",
        "\"profile\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v2\",\"scope_ref\":{{\"artifact_id\":\"27000000-0000-4000-8000-000000000002\",\"byte_length\":\"{}\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/scope.jcs\"}},\"value\":\"{}\"}}"
    ), descriptor.len(), hex(&sha256(&descriptor)), scope_value).into_bytes();
    let manifest_digest = format!(
        "{{\"algorithm\":\"SHA-256/FIPS-180-4\",\"domain\":\"rusty-data-os/exp1/workload-manifest/v2\",\"manifest_ref\":{{\"artifact_id\":\"16000000-0000-4000-8000-000000000001\",\"byte_length\":\"{}\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/16000000-0000-4000-8000-000000000001\"}},\"profile\":\"EXP-0001-WORKLOAD-MANIFEST-DIGEST-v2\",\"value\":\"{}\"}}",
        MANIFEST.len(), hex(&sha256(MANIFEST)), hex(&manifest_digest_v2(MANIFEST))
    ).into_bytes();
    let manifest_meta = metadata(
        "16000000-0000-4000-8000-000000000001",
        MANIFEST,
        "workload_manifest",
        "application/vnd.rusty-data-os.exp1-workload-manifest+jcs",
    );
    let stream_meta = metadata(
        "16000000-0000-4000-8000-000000000002",
        &ws,
        "configuration",
        "application/vnd.rusty-data-os.exp1-workload-stream",
    );
    Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    }
}

fn refresh_scope_digest(f: &mut Fixture) {
    let mut preimage = b"rusty-data-os/exp1/closed-stream-scope/v2\0".to_vec();
    preimage.extend(&f.descriptor);
    f.scope_digest = format!(concat!(
        "{{\"algorithm\":\"SHA-256/FIPS-180-4\",\"domain\":\"rusty-data-os/exp1/closed-stream-scope/v2\",",
        "\"profile\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v2\",\"scope_ref\":{{\"artifact_id\":\"27000000-0000-4000-8000-000000000002\",\"byte_length\":\"{}\",\"sha256\":\"{}\",\"uri\":\"https://example.invalid/scope.jcs\"}},\"value\":\"{}\"}}"
    ), f.descriptor.len(), hex(&sha256(&f.descriptor)), hex(&sha256(&preimage))).into_bytes();
}

fn construct(
    f: &Fixture,
    selected: [u8; 16],
) -> Result<ReferenceContextV2, ContextConstructionError> {
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &f.manifest_digest,
        manifest_artifact_metadata: &f.manifest_meta,
        stream: &f.ws,
        stream_artifact_metadata: &f.stream_meta,
    };
    construct_reference_context_v2(
        ClosedScopeInputV2 {
            scope: ScopeDigestInput {
                descriptor: &f.descriptor,
                artifact_metadata: &f.scope_digest,
            },
            members: &[binding],
        },
        selected,
    )
}

fn context() -> (ReferenceContextV2, Vec<Vec<u8>>) {
    let Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    let context = construct_reference_context_v2(
        ClosedScopeInputV2 {
            scope: ScopeDigestInput {
                descriptor: &descriptor,
                artifact_metadata: &scope_digest,
            },
            members: &[binding],
        },
        NS,
    )
    .unwrap();
    let mut position = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2".len() + 24;
    let mut operations = Vec::new();
    for _ in 0..4 {
        let length = u64::from_be_bytes(ws[position..position + 8].try_into().unwrap()) as usize;
        position += 8;
        operations.push(ws[position..position + length].to_vec());
        position += length;
    }
    (context, operations)
}

fn replace_once(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).unwrap();
    assert_eq!(text.matches(from).count(), 1, "mutation must be singular");
    text.replacen(from, to, 1).into_bytes()
}

// R27 section 10 executable checklist (one row per requirement sentence/clause):
// - valid bootstraps/references, byte-exact RF1, three watermarks:
//   `both_segment_bootstraps_and_prior_references_map_transactionally`.
// - manifest/WS2/artifact/manifest-digest/scope-digest binding and mixed-version positions:
//   `every_external_binding_and_mixed_version_position_is_rejected`.
// - unchanged v1 behavior: the legacy `tests/mapping.rs` suite (also run by R9).
// - omitted/extra/duplicate/substituted/foreign/digest/noncanonical scope:
//   `closed_scope_set_classification_rows_are_direct` plus
//   `duplicate_supplied_member_precedes_typed_identity_collision`.
// - construction-error adjacency/precedence:
//   `construction_precedence_adjacency_table` and
//   `construction_errors_are_distinct_at_their_precedence_boundaries`.
// - Request/Event/Information typed collisions: source unit test
//   `completion_matrix::request_event_information_typed_collisions_are_actual_insertions`.
// - inclusive/one-over/overflow limits: `resource_limit_boundary_table`; authority-dependent
//   unreachable maxima are explicitly represented by the nearest reachable checked invariant.
// - every ReferenceError, both cross-segment directions, and failure immutability: source unit test
//   `completion_matrix::every_reference_error_and_cross_segment_direction_is_directly_asserted`.
// - duplicate-before-lookup, target-bearing bootstrap, and first-invalid encoded ordering:
//   source unit test `completion_matrix::duplicate_before_lookup_target_bootstrap_and_first_invalid_encoded_order`.
// - mapping precedence, discontinuity, exhaustion, and transactional failures:
//   `mapping_precedence_and_failures_are_transactional` and source unit test
//   `completion_matrix::mapping_precedence_adjacencies_and_failure_immutability_are_direct`.

#[test]
fn r26_literals_construct_an_immutable_catalog_and_initial_state() {
    let Fixture {
        ws,
        descriptor,
        scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    let context = construct_reference_context_v2(
        ClosedScopeInputV2 {
            scope: ScopeDigestInput {
                descriptor: &descriptor,
                artifact_metadata: &scope_digest,
            },
            members: &[binding],
        },
        NS,
    )
    .unwrap();
    assert_eq!(context.catalog().stream_count(), 1);
    assert_eq!(context.catalog().operation_count(), 4);
    assert_eq!(context.catalog().identity_entry_count(), 12);
    assert_eq!(context.catalog().source_bytes(), 3139);
    assert_eq!(context.initial_state().accepted_operations(), 0);
}

#[test]
fn construction_is_closed_and_digest_bound() {
    let Fixture {
        ws,
        descriptor,
        mut scope_digest,
        manifest_digest,
        manifest_meta,
        stream_meta,
    } = fixture();
    *scope_digest.last_mut().unwrap() = b'!';
    let binding = ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &manifest_digest,
        manifest_artifact_metadata: &manifest_meta,
        stream: &ws,
        stream_artifact_metadata: &stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &descriptor,
                    artifact_metadata: &scope_digest
                },
                members: &[binding]
            },
            NS
        )
        .unwrap_err(),
        ContextConstructionError::InvalidScopeDigest
    );
}

#[test]
fn every_external_binding_and_mixed_version_position_is_rejected() {
    type MutationCase = (&'static str, fn(&mut Fixture), ContextConstructionError);
    let cases: &[MutationCase] = &[
        (
            "manifest metadata bytes",
            |f| f.manifest_meta.push(b' '),
            ContextConstructionError::InvalidMemberBinding,
        ),
        (
            "WS2 bytes",
            |f| f.ws[0] ^= 1,
            ContextConstructionError::InvalidMemberBinding,
        ),
        (
            "manifest artifact",
            |f| f.manifest_meta[0] ^= 1,
            ContextConstructionError::InvalidMemberBinding,
        ),
        (
            "stream artifact",
            |f| f.stream_meta[0] ^= 1,
            ContextConstructionError::InvalidMemberBinding,
        ),
        (
            "manifest digest",
            |f| f.manifest_digest[0] ^= 1,
            ContextConstructionError::InvalidMemberBinding,
        ),
        (
            "scope digest",
            |f| f.scope_digest[0] ^= 1,
            ContextConstructionError::InvalidScopeDigest,
        ),
        (
            "scope v1 profile",
            |f| {
                f.descriptor = replace_once(&f.descriptor, "JCS-v2", "JCS-v1");
                refresh_scope_digest(f);
            },
            ContextConstructionError::UnsupportedScopeProfile,
        ),
        (
            "manifest v1 profile",
            |f| f.manifest_digest = replace_once(&f.manifest_digest, "DIGEST-v2", "DIGEST-v1"),
            ContextConstructionError::UnsupportedScopeProfile,
        ),
        (
            "WS1 in WS2 position",
            |f| f.ws[7] = b'1',
            ContextConstructionError::InvalidMemberBinding,
        ),
    ];
    for (name, mutate, expected) in cases {
        let mut f = fixture();
        mutate(&mut f);
        assert_eq!(construct(&f, NS).unwrap_err(), *expected, "{name}");
    }

    let f = fixture();
    let mut changed_manifest = MANIFEST.to_vec();
    *changed_manifest.last_mut().unwrap() = b'!';
    let binding = ManifestBindingInput {
        manifest: &changed_manifest,
        manifest_digest_descriptor: &f.manifest_digest,
        manifest_artifact_metadata: &f.manifest_meta,
        stream: &f.ws,
        stream_artifact_metadata: &f.stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &f.descriptor,
                    artifact_metadata: &f.scope_digest,
                },
                members: &[binding],
            },
            NS,
        )
        .unwrap_err(),
        ContextConstructionError::InvalidMemberBinding,
        "exact manifest bytes are bound"
    );
}

#[test]
fn closed_scope_set_classification_rows_are_direct() {
    let f = fixture();
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &f.descriptor,
                    artifact_metadata: &f.scope_digest
                },
                members: &[],
            },
            NS
        )
        .unwrap_err(),
        ContextConstructionError::OmittedStream
    );

    let mut substituted = fixture();
    substituted.descriptor = replace_once(
        &substituted.descriptor,
        "25000000-0000-4000-8000-000000000001",
        "25000000-0000-4000-8000-000000000002",
    );
    refresh_scope_digest(&mut substituted);
    assert_eq!(
        construct(&substituted, NS).unwrap_err(),
        ContextConstructionError::SubstitutedStream
    );

    let mut foreign = fixture();
    foreign.descriptor = replace_once(
        &foreign.descriptor,
        "16000000-0000-4000-8000-000000000003",
        "16000000-0000-4000-8000-000000000004",
    );
    refresh_scope_digest(&mut foreign);
    assert_eq!(
        construct(&foreign, NS).unwrap_err(),
        ContextConstructionError::ForeignWorkloadOrCell
    );

    let mut digest = fixture();
    digest.descriptor = replace_once(
        &digest.descriptor,
        "f1d0d28189680504617bd22c581ba12dab29bb6858909768c2f21180133845f7",
        "f1d0d28189680504617bd22c581ba12dab29bb6858909768c2f21180133845f0",
    );
    refresh_scope_digest(&mut digest);
    assert_eq!(
        construct(&digest, NS).unwrap_err(),
        ContextConstructionError::InvalidMemberBinding
    );

    let mut noncanonical = fixture();
    noncanonical.descriptor = replace_once(
        &noncanonical.descriptor,
        "{\"cell_id\":\"PC-D1-raw-v2\",\"members\":",
        "{\"members\":",
    );
    assert_eq!(
        construct(&noncanonical, NS).unwrap_err(),
        ContextConstructionError::InvalidScopeEncoding
    );
}

#[test]
fn resource_limit_boundary_table() {
    let mut descriptor = fixture();
    descriptor.descriptor.resize(262_145, b' ');
    assert_eq!(
        construct(&descriptor, NS).unwrap_err(),
        ContextConstructionError::ResourceLimit
    );

    let mut metadata = fixture();
    metadata.scope_digest.resize(4_194_305, b' ');
    assert_eq!(
        construct(&metadata, NS).unwrap_err(),
        ContextConstructionError::ResourceLimit
    );

    let f = fixture();
    let oversized_manifest = vec![b' '; 1_048_577];
    let binding = ManifestBindingInput {
        manifest: &oversized_manifest,
        manifest_digest_descriptor: &f.manifest_digest,
        manifest_artifact_metadata: &f.manifest_meta,
        stream: &f.ws,
        stream_artifact_metadata: &f.stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &f.descriptor,
                    artifact_metadata: &f.scope_digest
                },
                members: &[binding]
            },
            NS
        )
        .unwrap_err(),
        ContextConstructionError::ResourceLimit
    );

    let mut ws = fixture();
    ws.ws.resize(16_777_217, 0);
    assert_eq!(
        construct(&ws, NS).unwrap_err(),
        ContextConstructionError::ResourceLimit
    );
    // Exact inclusive arithmetic, member/operation/identity/reference maxima, and checked
    // overflow are white-box asserted by `completion_matrix::every_inclusive_limit...`.
}

#[test]
fn construction_precedence_adjacency_table() {
    let mut early_and_late = fixture();
    early_and_late.scope_digest[0] ^= 1;
    early_and_late.manifest_meta[0] ^= 1;
    assert_eq!(
        construct(&early_and_late, NS).unwrap_err(),
        ContextConstructionError::InvalidScopeDigest
    );

    let mut authority_and_member = fixture();
    authority_and_member.descriptor = replace_once(
        &authority_and_member.descriptor,
        "PC-D1-raw-v2",
        "PC-D1-raw-XX",
    );
    refresh_scope_digest(&mut authority_and_member);
    authority_and_member.manifest_meta[0] ^= 1;
    assert_eq!(
        construct(&authority_and_member, NS).unwrap_err(),
        ContextConstructionError::InvalidCellAuthority
    );

    // Member semantic resolution precedes supplied-set classification.
    let mut bad_member = fixture();
    bad_member.manifest_meta[0] ^= 1;
    let binding = || ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &bad_member.manifest_digest,
        manifest_artifact_metadata: &bad_member.manifest_meta,
        stream: &bad_member.ws,
        stream_artifact_metadata: &bad_member.stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &bad_member.descriptor,
                    artifact_metadata: &bad_member.scope_digest
                },
                members: &[binding(), binding()]
            },
            NS
        )
        .unwrap_err(),
        ContextConstructionError::InvalidMemberBinding
    );
}

#[test]
fn construction_errors_are_distinct_at_their_precedence_boundaries() {
    let mut bad_reference = fixture();
    bad_reference.scope_digest = String::from_utf8(bad_reference.scope_digest)
        .unwrap()
        .replace(
            "https://example.invalid/scope.jcs",
            "http://example.invalid/scope.jcs",
        )
        .into_bytes();
    assert_eq!(
        construct(&bad_reference, NS).unwrap_err(),
        ContextConstructionError::ScopeReferenceFailure
    );

    let mut bad_authority = fixture();
    bad_authority.descriptor = String::from_utf8(bad_authority.descriptor)
        .unwrap()
        .replace("PC-D1-raw-v2", "PC-D1-raw-XX")
        .into_bytes();
    refresh_scope_digest(&mut bad_authority);
    // A validly bound descriptor naming an unreviewed cell is not a member failure.
    assert_eq!(
        construct(&bad_authority, NS).unwrap_err(),
        ContextConstructionError::InvalidCellAuthority
    );

    let mut bad_member = fixture();
    bad_member.manifest_meta = String::from_utf8(bad_member.manifest_meta)
        .unwrap()
        .replace(
            "\"publication_state\":\"published\"",
            "\"publication_state\":\"staged\"",
        )
        .into_bytes();
    assert_eq!(
        construct(&bad_member, NS).unwrap_err(),
        ContextConstructionError::InvalidMemberBinding
    );
}

#[test]
fn r7_metadata_rejects_invalid_supersession_provenance_identity_and_closed_fields() {
    let cases = [
        (
            "\"supersedes_record_id\":{\"state\":\"not_applicable\"}",
            "\"supersedes_record_id\":{\"state\":\"present\"}",
        ),
        ("\"provenance_edges\":[]", "\"provenance_edges\":[{}]"),
        (
            "\"record_id\":\"16000000-0000-4000-8000-000000000010\"",
            "\"record_id\":\"not-a-uuid\"",
        ),
        (
            "\"series_freeze\":{\"state\":\"not_applicable\"}",
            "\"series_freeze\":{\"extra\":\"x\",\"state\":\"not_applicable\"}",
        ),
        (
            "\"created_by_record_id\":\"16000000-0000-4000-8000-000000000006\"",
            "\"created_by_record_id\":\"not-a-uuid\"",
        ),
    ];
    for (valid, invalid) in cases {
        let mut f = fixture();
        f.manifest_meta = String::from_utf8(f.manifest_meta)
            .unwrap()
            .replace(valid, invalid)
            .into_bytes();
        assert_eq!(
            construct(&f, NS).unwrap_err(),
            ContextConstructionError::InvalidMemberBinding,
            "mutation {invalid} must be rejected"
        );
    }
}

#[test]
fn duplicate_supplied_member_precedes_typed_identity_collision() {
    let f = fixture();
    let binding = || ManifestBindingInput {
        manifest: MANIFEST,
        manifest_digest_descriptor: &f.manifest_digest,
        manifest_artifact_metadata: &f.manifest_meta,
        stream: &f.ws,
        stream_artifact_metadata: &f.stream_meta,
    };
    assert_eq!(
        construct_reference_context_v2(
            ClosedScopeInputV2 {
                scope: ScopeDigestInput {
                    descriptor: &f.descriptor,
                    artifact_metadata: &f.scope_digest,
                },
                members: &[binding(), binding()],
            },
            NS,
        )
        .unwrap_err(),
        ContextConstructionError::DuplicateStreamNamespace
    );
}

#[test]
fn missing_selected_stream_is_checked_before_catalog_publication() {
    let mut absent = NS;
    absent[15] = 2;
    assert_eq!(
        construct(&fixture(), absent).unwrap_err(),
        ContextConstructionError::SelectedStreamMissing
    );
}

#[test]
fn both_segment_bootstraps_and_prior_references_map_transactionally() {
    let (context, operations) = context();
    let mut state = context.initial_state().clone();
    let frame_digests = [
        "5822e65071f5ed4a17865d96c60247639bf11fe7fb6421efc139181510a1333a",
        "2a293b9f09924711c476d0892abfb296a04e8b31a1b6649b1e2c6906d714dd87",
        "cc1293aed2c195cb047b0bbe607979074369993946b757914b14d7c0b9f44f1a",
        "fb5d0a896bb841aae4045e6dfd2a6369d0d2095e74aae18c8234f39011cd83f3",
    ];
    for (index, operation) in operations.iter().enumerate() {
        let before = state.clone();
        let mapped = map_semantic_operation_v2_with_context(
            operation,
            (index + 1) as u64,
            (index + 1) as u64,
            context.catalog(),
            &state,
        )
        .unwrap();
        assert_eq!(state, before);
        assert_eq!(
            mapped.next_state().accepted_operations(),
            (index + 1) as u64
        );
        assert_eq!(mapped.next_state().previous_sequence(), (index + 1) as u64);
        assert_eq!(
            mapped.next_state().previous_physical_ordinal(),
            (index + 1) as u64
        );
        assert_eq!(
            exp1_record_format::decode(mapped.frame()).unwrap(),
            *mapped.record()
        );
        // Checked-in hashes make the RF1 assertion independent of the decoder
        // and cover both segment bootstraps and both ordinal-one references.
        assert_eq!(hex(&sha256(mapped.frame())), frame_digests[index]);
        state = mapped.next_state().clone();
    }
    assert_eq!(
        map_semantic_operation_v2_with_context(&operations[3], 5, 5, context.catalog(), &state)
            .unwrap_err(),
        ContextualMappingError::Exhaustion
    );
}

#[test]
fn mapping_precedence_and_failures_are_transactional() {
    let (context, operations) = context();
    let initial = context.initial_state().clone();

    assert_eq!(
        map_semantic_operation_v2_with_context(&operations[1], 1, 1, context.catalog(), &initial,)
            .unwrap_err(),
        ContextualMappingError::Discontinuity
    );
    assert_eq!(initial, *context.initial_state());

    assert!(matches!(
        map_semantic_operation_v2_with_context(&operations[0], 0, 1, context.catalog(), &initial,),
        Err(ContextualMappingError::Mapping(_))
    ));
    assert_eq!(initial, *context.initial_state());

    let mut state = initial;
    for (index, operation) in operations.iter().enumerate() {
        state = map_semantic_operation_v2_with_context(
            operation,
            (index + 1) as u64,
            (index + 1) as u64,
            context.catalog(),
            &state,
        )
        .unwrap()
        .next_state()
        .clone();
    }
    let exhausted = state.clone();
    // Semantic validation is stage one and therefore wins even at exhaustion.
    assert!(matches!(
        map_semantic_operation_v2_with_context(
            b"not-an-operation",
            5,
            5,
            context.catalog(),
            &state,
        ),
        Err(ContextualMappingError::SemanticValidation(_))
    ));
    assert_eq!(state, exhausted);
}
