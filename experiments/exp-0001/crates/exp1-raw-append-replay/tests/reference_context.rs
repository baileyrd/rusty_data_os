//! Synthetic constructor rejection checks. These do not materialize or execute a workload.

use exp1_raw_append_replay::reference_context::*;

fn scope_input<'a>(descriptor: &'a [u8]) -> ClosedScopeInput<'a> {
    let artifact_sha =
        exp1_workload_conformance::hex(&exp1_workload_conformance::artifact_digest(descriptor));
    // These strings are deliberately leaked only inside this short-lived correctness test so the
    // borrowed public input can exercise stage-one precedence without an authority graph.
    let artifact_sha = Box::leak(artifact_sha.into_boxed_str());
    ClosedScopeInput {
        descriptor,
        scope_digest: ScopeDigestDescriptor {
            algorithm: "SHA-256/FIPS-180-4",
            domain: "rusty-data-os/exp1/closed-stream-scope/v1",
            profile: "EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v1",
            value: "00",
            scope_ref: ScopeReference {
                artifact_id: "scope",
                byte_length: descriptor.len() as u64,
                sha256: artifact_sha,
                uri: "https://example.invalid/scope",
            },
        },
        scope_artifact: ScopeArtifactMetadata {
            artifact_id: "scope",
            byte_length: descriptor.len() as u64,
            sha256: artifact_sha,
            uri: "https://example.invalid/scope",
            role: "configuration",
            media_type: "application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs",
            created_by_record_id: "record",
            metadata_bytes: &[],
        },
        authorized_cell_ids: &[],
        members: &[],
    }
}

#[test]
fn descriptor_encoding_precedes_digest_and_authority_resolution() {
    let input = scope_input(b"not-json");
    assert_eq!(
        construct_reference_context(input, [0; 16]),
        Err(ContextConstructionError::InvalidScopeEncoding)
    );
}

#[test]
fn descriptor_resource_limit_precedes_parsing_and_allocation() {
    let bytes = vec![b'x'; 262_145];
    assert_eq!(
        construct_reference_context(scope_input(&bytes), [0; 16]),
        Err(ContextConstructionError::ResourceLimit)
    );
}

#[test]
fn public_error_taxonomy_keeps_all_r24_outcomes_distinct() {
    let references = [
        ReferenceError::Missing,
        ReferenceError::Future,
        ReferenceError::WrongKind,
        ReferenceError::WrongFact,
        ReferenceError::SelfReference,
        ReferenceError::CrossStream,
        ReferenceError::CrossSegment,
    ];
    for (index, left) in references.iter().enumerate() {
        for (other, right) in references.iter().enumerate() {
            assert_eq!(left == right, index == other);
        }
    }
    assert_ne!(
        ContextualMappingError::Discontinuity,
        ContextualMappingError::Exhaustion
    );
}
