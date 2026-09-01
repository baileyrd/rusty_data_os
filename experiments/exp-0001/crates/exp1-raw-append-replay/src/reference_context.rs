//! Pure R21--R24 closed-scope reference validation context.

use crate::mapping::{MappedRecord, MappingError, MappingState, map_semantic_operation};
use exp1_record_format::Record;
use exp1_workload_conformance::{
    Error as SemanticError, ValidationContext, artifact_digest, hex, manifest_digest, parse_uuid,
    sha256, validate_manifest, validate_semantic_operation, validate_stream, workload_digest,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_STREAMS: usize = 256;
const MAX_STREAM_BYTES: usize = 16_777_216;
const MAX_OPERATIONS: usize = 65_536;
const MAX_BINDINGS: usize = 196_608;
const MAX_REFERENCES: usize = 65_536;
const MAX_DESCRIPTOR_BYTES: usize = 262_144;
const MAX_MANIFEST_BYTES: usize = 1_048_576;
const MAX_METADATA_BYTES: usize = 4_194_304;
const SCOPE_DOMAIN: &str = "rusty-data-os/exp1/closed-stream-scope/v1";

/// The external R23 scope digest descriptor.
#[derive(Clone, Debug)]
pub struct ScopeDigestDescriptor<'a> {
    pub algorithm: &'a str,
    pub domain: &'a str,
    pub profile: &'a str,
    pub value: &'a str,
    pub scope_ref: ScopeReference<'a>,
}

#[derive(Clone, Debug)]
pub struct ScopeReference<'a> {
    pub artifact_id: &'a str,
    pub byte_length: u64,
    pub sha256: &'a str,
    pub uri: &'a str,
}

/// Resolved immutable R7 metadata for the scope artifact.
#[derive(Clone, Debug)]
pub struct ScopeArtifactMetadata<'a> {
    pub artifact_id: &'a str,
    pub byte_length: u64,
    pub sha256: &'a str,
    pub uri: &'a str,
    pub role: &'a str,
    pub media_type: &'a str,
    pub created_by_record_id: &'a str,
    /// Exact canonical R7 artifact-manifest record which proves this entry.
    pub metadata_bytes: &'a [u8],
}

/// One unvalidated, borrowed resolution of an R23 member.
#[derive(Clone, Debug)]
pub struct ClosedScopeMemberInput<'a> {
    pub stream_namespace: [u8; 16],
    pub workload_id: &'a str,
    pub manifest_id: &'a str,
    pub cell_id: &'a str,
    pub stream: &'a [u8],
    pub manifest: &'a [u8],
    pub manifest_validation: &'a ValidationContext<'a>,
    /// Exact concatenation of the two R7 records validated by `manifest_validation`, each
    /// preceded by its big-endian u64 byte length. This prevents parallel asserted fields from
    /// being treated as provenance.
    pub resolved_metadata_bytes: &'a [u8],
}

/// Borrowed R23 construction input. Public fields are assertions, not proof.
#[derive(Clone, Debug)]
pub struct ClosedScopeInput<'a> {
    pub descriptor: &'a [u8],
    pub scope_digest: ScopeDigestDescriptor<'a>,
    pub scope_artifact: ScopeArtifactMetadata<'a>,
    pub members: &'a [ClosedScopeMemberInput<'a>],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextConstructionError {
    InvalidScopeEncoding,
    UnsupportedScopeProfile,
    InvalidScopeDigest,
    ScopeReferenceFailure,
    InvalidCellAuthority,
    InvalidMemberBinding,
    OmittedStream,
    ExtraStream,
    DuplicateStreamNamespace,
    SubstitutedStream,
    ForeignWorkloadOrCell,
    SemanticValidation(SemanticError),
    IdentityCollision,
    ResourceLimit,
    Extraction,
    SelectedStreamMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceError {
    Missing,
    Future,
    WrongKind,
    WrongFact,
    SelfReference,
    CrossStream,
    CrossSegment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextualMappingError {
    SemanticValidation(SemanticError),
    Discontinuity,
    Exhaustion,
    Reference(ReferenceError),
    Mapping(MappingError),
    Extraction,
    ResourceLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Role {
    Request,
    Event,
    Information,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fact {
    Ordinary,
    Correction,
    Retraction,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StreamSegment {
    WarmUp,
    Measured,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Binding {
    role: Role,
    fact: Option<Fact>,
    namespace: [u8; 16],
    position: u64,
    segment: StreamSegment,
    segment_ordinal: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Operation {
    bytes: Vec<u8>,
    event_id: [u8; 16],
    references: Vec<[u8; 16]>,
    segment: StreamSegment,
    segment_ordinal: u64,
}

/// Immutable successful R21 catalog. Its entries are deliberately opaque.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceCatalog {
    scope_id: [u8; 16],
    streams: usize,
    operations: usize,
    bindings: BTreeMap<[u8; 16], Binding>,
    selected: Vec<Operation>,
    selected_namespace: [u8; 16],
}
impl ReferenceCatalog {
    pub const fn scope_id(&self) -> [u8; 16] {
        self.scope_id
    }
    pub const fn stream_count(&self) -> usize {
        self.streams
    }
    pub const fn operation_count(&self) -> usize {
        self.operations
    }
    pub fn identity_binding_count(&self) -> usize {
        self.bindings.len()
    }
}

/// Opaque, caller-owned accepted-prefix and R20 watermark state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedPrefixState {
    scope_id: [u8; 16],
    namespace: [u8; 16],
    next_position: usize,
    next_segment: Option<StreamSegment>,
    next_segment_ordinal: Option<u64>,
    mapping: MappingState,
}
impl AcceptedPrefixState {
    pub const fn accepted_count(&self) -> usize {
        self.next_position
    }
    pub const fn previous_sequence(&self) -> u64 {
        self.mapping.previous_sequence()
    }
    pub const fn previous_physical_ordinal(&self) -> u64 {
        self.mapping.previous_physical_ordinal()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceContext {
    catalog: ReferenceCatalog,
    initial_state: AcceptedPrefixState,
}
impl ReferenceContext {
    pub const fn catalog(&self) -> &ReferenceCatalog {
        &self.catalog
    }
    pub const fn initial_state(&self) -> &AcceptedPrefixState {
        &self.initial_state
    }
    pub fn into_parts(self) -> (ReferenceCatalog, AcceptedPrefixState) {
        (self.catalog, self.initial_state)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualMappedRecord {
    pub frame: Vec<u8>,
    pub record: Record,
    next_state: AcceptedPrefixState,
}
impl ContextualMappedRecord {
    pub fn next_state(&self) -> &AcceptedPrefixState {
        &self.next_state
    }
    pub fn into_next_state(self) -> AcceptedPrefixState {
        self.next_state
    }
}

#[derive(Clone, Debug)]
struct Descriptor {
    scope_id: [u8; 16],
    cell_id: String,
    members: Vec<Member>,
}
#[derive(Clone, Debug)]
struct Member {
    namespace: [u8; 16],
    workload_id: String,
    manifest_id: String,
    manifest_digest: String,
    stream_digest: String,
    stream_len: u64,
    artifact_digest: String,
}

/// Validates R23's eight construction stages and publishes no partial result.
pub fn construct_reference_context(
    input: ClosedScopeInput<'_>,
    selected_stream_namespace: [u8; 16],
) -> Result<ReferenceContext, ContextConstructionError> {
    // Stage 1: descriptor syntax, canonical closed schema, profiles, ordering, uniqueness.
    if input.descriptor.len() > MAX_DESCRIPTOR_BYTES {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let descriptor = parse_descriptor(input.descriptor)?;
    // Stage 2: external digest and immutable R7 binding.
    validate_scope_digest(&input, &descriptor)?;
    // Bounds whose inputs are available before authority validation/allocation.
    if input.members.len() > MAX_STREAMS {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let manifest_bytes = checked_sum(
        input.members.iter().map(|m| m.manifest.len()),
        MAX_MANIFEST_BYTES,
    )?;
    let metadata_bytes = checked_sum(
        input
            .members
            .iter()
            .map(|m| m.resolved_metadata_bytes.len()),
        MAX_METADATA_BYTES,
    )?
    .checked_add(input.scope_artifact.metadata_bytes.len())
    .ok_or(ContextConstructionError::ResourceLimit)?;
    if manifest_bytes > MAX_MANIFEST_BYTES || metadata_bytes > MAX_METADATA_BYTES {
        return Err(ContextConstructionError::ResourceLimit);
    }
    // Stage 3: exact R16 resolution and declared cell authority, in descriptor order.
    let mut supplied_by_ns = BTreeMap::new();
    for supplied in input.members {
        if supplied_by_ns
            .insert(supplied.stream_namespace, supplied)
            .is_some()
        {
            return Err(ContextConstructionError::DuplicateStreamNamespace);
        }
    }
    for member in &descriptor.members {
        let Some(supplied) = supplied_by_ns.get(&member.namespace) else {
            continue;
        };
        validate_manifest(supplied.manifest, supplied.manifest_validation)
            .map_err(ContextConstructionError::SemanticValidation)?;
        if !r8_cell_id(&descriptor.cell_id) {
            return Err(ContextConstructionError::InvalidCellAuthority);
        }
        validate_member_provenance(supplied)?;
        if supplied.cell_id != descriptor.cell_id || supplied.workload_id != member.workload_id {
            return Err(ContextConstructionError::ForeignWorkloadOrCell);
        }
    }
    // Stage 4: member cross-bindings.
    for member in &descriptor.members {
        let Some(supplied) = supplied_by_ns.get(&member.namespace) else {
            continue;
        };
        let manifest_workload = json_string(supplied.manifest, "workload_id")
            .ok_or(ContextConstructionError::Extraction)?;
        let manifest_id = json_string(supplied.manifest, "manifest_id")
            .ok_or(ContextConstructionError::Extraction)?;
        let manifest_namespace = json_string(supplied.manifest, "stream_namespace")
            .and_then(|s| parse_uuid(s).ok())
            .ok_or(ContextConstructionError::Extraction)?;
        if supplied.workload_id != member.workload_id
            || supplied.manifest_id != member.manifest_id
            || manifest_workload != member.workload_id
            || manifest_id != member.manifest_id
            || manifest_namespace != member.namespace
            || supplied.manifest_validation.descriptor.value != member.manifest_digest
            || hex(&manifest_digest(supplied.manifest)) != member.manifest_digest
            || member.stream_len
                != u64::try_from(supplied.stream.len())
                    .map_err(|_| ContextConstructionError::ResourceLimit)?
            || hex(&workload_digest(supplied.stream)) != member.stream_digest
            || hex(&artifact_digest(supplied.stream)) != member.artifact_digest
        {
            return Err(ContextConstructionError::InvalidMemberBinding);
        }
    }
    // Stage 5: complete WS1 authority validation.
    for member in &descriptor.members {
        if let Some(supplied) = supplied_by_ns.get(&member.namespace) {
            validate_stream(supplied.stream)
                .map_err(ContextConstructionError::SemanticValidation)?;
        }
    }
    // Stage 6: exact supplied-set equality.
    if input.members.len() < descriptor.members.len() {
        return Err(ContextConstructionError::OmittedStream);
    }
    if input.members.len() > descriptor.members.len() {
        return Err(ContextConstructionError::ExtraStream);
    }
    for member in &descriptor.members {
        if !supplied_by_ns.contains_key(&member.namespace) {
            return Err(ContextConstructionError::SubstitutedStream);
        }
    }
    // Stage 7: aggregate bounds, extraction, and global typed collision checks.
    let total_bytes = checked_sum(
        input.members.iter().map(|m| m.stream.len()),
        MAX_STREAM_BYTES,
    )?;
    if total_bytes > MAX_STREAM_BYTES {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let mut bindings = BTreeMap::new();
    let mut selected = None;
    let mut operation_count = 0usize;
    for member in &descriptor.members {
        let supplied = supplied_by_ns[&member.namespace];
        let operations = extract_stream(supplied.stream, member.namespace)?;
        operation_count = operation_count
            .checked_add(operations.len())
            .ok_or(ContextConstructionError::ResourceLimit)?;
        if operation_count > MAX_OPERATIONS {
            return Err(ContextConstructionError::ResourceLimit);
        }
        for (position, op) in operations.iter().enumerate() {
            let f = operation_fields(&op.bytes).map_err(|error| match error {
                FieldsError::Encoding => ContextConstructionError::Extraction,
                FieldsError::ReferenceLimit => ContextConstructionError::ResourceLimit,
            })?;
            let ids = [
                (f.request_id, Role::Request, None),
                (f.event_id, Role::Event, Some(f.fact)),
                (f.information_id, Role::Information, None),
            ];
            for (id, role, fact) in ids {
                let binding = Binding {
                    role,
                    fact,
                    namespace: member.namespace,
                    position: position as u64,
                    segment: op.segment,
                    segment_ordinal: op.segment_ordinal,
                };
                if bindings.insert(id, binding).is_some() {
                    return Err(ContextConstructionError::IdentityCollision);
                }
            }
        }
        if member.namespace == selected_stream_namespace && selected.replace(operations).is_some() {
            return Err(ContextConstructionError::SelectedStreamMissing);
        }
    }
    if bindings.len() > MAX_BINDINGS {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let selected = selected.ok_or(ContextConstructionError::SelectedStreamMissing)?;
    // Stage 8: publish immutable catalog and its only initial state.
    let next_segment = selected.first().map(|o| o.segment);
    let next_segment_ordinal = selected.first().map(|o| o.segment_ordinal);
    let catalog = ReferenceCatalog {
        scope_id: descriptor.scope_id,
        streams: descriptor.members.len(),
        operations: operation_count,
        bindings,
        selected,
        selected_namespace: selected_stream_namespace,
    };
    let initial_state = AcceptedPrefixState {
        scope_id: descriptor.scope_id,
        namespace: selected_stream_namespace,
        next_position: 0,
        next_segment,
        next_segment_ordinal,
        mapping: MappingState::initial(),
    };
    Ok(ReferenceContext {
        catalog,
        initial_state,
    })
}

/// Transactionally validates references and delegates unchanged physical mapping to R20.
pub fn map_semantic_operation_with_context(
    semantic_operation: &[u8],
    assigned_sequence: u64,
    physical_ordinal: u64,
    catalog: &ReferenceCatalog,
    state: &AcceptedPrefixState,
) -> Result<ContextualMappedRecord, ContextualMappingError> {
    validate_semantic_operation(semantic_operation)
        .map_err(ContextualMappingError::SemanticValidation)?;
    let fields = operation_fields(semantic_operation).map_err(|error| match error {
        FieldsError::Encoding => ContextualMappingError::Extraction,
        FieldsError::ReferenceLimit => ContextualMappingError::ResourceLimit,
    })?;
    if state.scope_id != catalog.scope_id || state.namespace != catalog.selected_namespace {
        return Err(ContextualMappingError::Discontinuity);
    }
    let expected = catalog
        .selected
        .get(state.next_position)
        .ok_or(ContextualMappingError::Exhaustion)?;
    if expected.bytes != semantic_operation
        || state.next_segment != Some(expected.segment)
        || state.next_segment_ordinal != Some(expected.segment_ordinal)
    {
        return Err(ContextualMappingError::Discontinuity);
    }
    for target in &fields.references {
        if *target == fields.event_id {
            return Err(ContextualMappingError::Reference(
                ReferenceError::SelfReference,
            ));
        }
        match catalog.bindings.get(target) {
            Some(binding) if binding.role != Role::Event => {
                return Err(ContextualMappingError::Reference(ReferenceError::WrongKind));
            }
            Some(binding) if binding.fact != Some(Fact::Ordinary) => {
                return Err(ContextualMappingError::Reference(ReferenceError::WrongFact));
            }
            Some(binding) if binding.namespace != catalog.selected_namespace => {
                return Err(ContextualMappingError::Reference(
                    ReferenceError::CrossStream,
                ));
            }
            Some(binding) if binding.segment != expected.segment => {
                return Err(ContextualMappingError::Reference(
                    ReferenceError::CrossSegment,
                ));
            }
            Some(binding) if binding.segment_ordinal > expected.segment_ordinal => {
                return Err(ContextualMappingError::Reference(ReferenceError::Future));
            }
            Some(binding) if binding.segment_ordinal == expected.segment_ordinal => {
                return Err(ContextualMappingError::Reference(
                    ReferenceError::SelfReference,
                ));
            }
            Some(_) => {}
            None => return Err(ContextualMappingError::Reference(ReferenceError::Missing)),
        }
    }
    let MappedRecord { frame, record, .. } = map_semantic_operation(
        semantic_operation,
        assigned_sequence,
        physical_ordinal,
        state.mapping,
    )
    .map_err(ContextualMappingError::Mapping)?;
    let next_position = state
        .next_position
        .checked_add(1)
        .ok_or(ContextualMappingError::Exhaustion)?;
    let next = catalog.selected.get(next_position);
    let next_state = AcceptedPrefixState {
        scope_id: state.scope_id,
        namespace: state.namespace,
        next_position,
        next_segment: next.map(|o| o.segment),
        next_segment_ordinal: next.map(|o| o.segment_ordinal),
        mapping: MappingState::from_validated(assigned_sequence, physical_ordinal),
    };
    Ok(ContextualMappedRecord {
        frame,
        record,
        next_state,
    })
}

fn checked_sum(
    iter: impl Iterator<Item = usize>,
    limit: usize,
) -> Result<usize, ContextConstructionError> {
    let mut sum = 0usize;
    for value in iter {
        sum = sum
            .checked_add(value)
            .ok_or(ContextConstructionError::ResourceLimit)?;
        if sum > limit {
            return Err(ContextConstructionError::ResourceLimit);
        }
    }
    Ok(sum)
}

fn validate_scope_digest(
    input: &ClosedScopeInput<'_>,
    descriptor: &Descriptor,
) -> Result<(), ContextConstructionError> {
    let d = &input.scope_digest;
    let r = &d.scope_ref;
    let a = &input.scope_artifact;
    let mut digest_input = SCOPE_DOMAIN.as_bytes().to_vec();
    digest_input.push(0);
    digest_input.extend(input.descriptor);
    if d.algorithm != "SHA-256/FIPS-180-4"
        || d.domain != SCOPE_DOMAIN
        || d.profile != "EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v1"
        || d.value != hex(&sha256(&digest_input))
    {
        return Err(ContextConstructionError::InvalidScopeDigest);
    }
    if r.artifact_id != a.artifact_id
        || r.byte_length != a.byte_length
        || r.sha256 != a.sha256
        || r.uri != a.uri
        || r.byte_length as usize != input.descriptor.len()
        || r.sha256 != hex(&artifact_digest(input.descriptor))
        || a.role != "configuration"
        || a.media_type != "application/vnd.rusty-data-os.exp1-closed-stream-scope+jcs"
        || !valid_uuid(a.created_by_record_id)
        || descriptor.cell_id.is_empty()
    {
        return Err(ContextConstructionError::ScopeReferenceFailure);
    }
    validate_scope_r7_record(a)?;
    Ok(())
}

fn r8_cell_id(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("PC-") else {
        return false;
    };
    let mut fields = rest.split('-');
    let (Some(mode), Some(baseline), Some(profile), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return false;
    };
    matches!(
        (mode, baseline),
        ("D0", "B0") | ("D1", "B1" | "B2" | "B3") | ("D2", "B1" | "B2" | "B3") | ("D3", "B1")
    ) && matches!(profile, "F1" | "F2" | "F3" | "ME" | "MW")
}

fn validate_member_provenance(
    member: &ClosedScopeMemberInput<'_>,
) -> Result<(), ContextConstructionError> {
    let a = member.manifest_validation.artifact_manifest_bytes;
    let w = member.manifest_validation.workload_artifact_manifest_bytes;
    let expected = 16usize
        .checked_add(a.len())
        .and_then(|n| n.checked_add(w.len()))
        .ok_or(ContextConstructionError::ResourceLimit)?;
    if member.resolved_metadata_bytes.len() != expected
        || member.resolved_metadata_bytes.get(..8) != Some(&(a.len() as u64).to_be_bytes())
        || member.resolved_metadata_bytes.get(8..8 + a.len()) != Some(a)
        || member
            .resolved_metadata_bytes
            .get(8 + a.len()..16 + a.len())
            != Some(&(w.len() as u64).to_be_bytes())
        || member.resolved_metadata_bytes.get(16 + a.len()..) != Some(w)
    {
        return Err(ContextConstructionError::InvalidMemberBinding);
    }
    Ok(())
}

fn validate_scope_r7_record(a: &ScopeArtifactMetadata<'_>) -> Result<(), ContextConstructionError> {
    // R24 admits one frozen, run-scoped R7 shape. Comparing the complete canonical serialization
    // is deliberately stricter than searching JSON text: it rejects malformed JSON, duplicate or
    // unknown members, reordered keys, extra/conflicting artifacts and every provenance edge
    // (including unreachable or cyclic graphs).
    let expected = format!(
        "{{\"body\":{{\"artifacts\":[{{\"artifact_id\":\"{}\",\"byte_length\":\"{}\",\"created_by_record_id\":\"{}\",\"logical_path\":\"exp-0001/scopes/{}/configuration\",\"media_type\":\"{}\",\"retention_state\":\"published\",\"role\":\"{}\",\"sensitivity\":\"public\",\"sha256\":\"{}\",\"uri\":\"{}\",\"validation_report_ids\":[]}}],\"provenance_edges\":[],\"publication_state\":\"published\",\"scope\":\"run\",\"series_freeze\":{{\"state\":\"not_applicable\"}}}},\"correction_reason\":{{\"state\":\"not_applicable\"}},\"created_at_utc_ns\":\"1788134400000000000\",\"record_id\":\"{}\",\"record_kind\":\"artifact_manifest\",\"run_id\":{{\"state\":\"present\",\"value\":\"24000000-0000-4000-8000-000000000005\"}},\"schema_version\":\"EXP1-R7-JSON-JCS-1\",\"series_id\":\"24000000-0000-4000-8000-000000000004\",\"supersedes_record_id\":{{\"state\":\"not_applicable\"}}}}",
        a.artifact_id,
        a.byte_length,
        a.created_by_record_id,
        a.artifact_id,
        a.media_type,
        a.role,
        a.sha256,
        a.uri,
        a.created_by_record_id,
    );
    if !valid_uuid(a.artifact_id)
        || !valid_digest(a.sha256)
        || !valid_uuid(a.created_by_record_id)
        || !valid_unescaped_json_string(a.uri)
        || a.metadata_bytes != expected.as_bytes()
    {
        return Err(ContextConstructionError::ScopeReferenceFailure);
    }
    Ok(())
}

// The frozen R7 serialization emits these caller-provided values without JSON escapes.  Requiring
// the exact unescaped I-JSON subset makes interpolation safe: quotes/backslashes cannot terminate
// a value or inject a member, controls cannot create malformed JSON, and UTF-8 is guaranteed by
// the Rust string type.
fn valid_unescaped_json_string(value: &str) -> bool {
    value
        .chars()
        .all(|c| c >= '\u{20}' && c != '"' && c != '\\' && !matches!(c, '\u{7f}'..='\u{9f}'))
}

fn parse_descriptor(bytes: &[u8]) -> Result<Descriptor, ContextConstructionError> {
    let s =
        std::str::from_utf8(bytes).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    let prefix = "{\"cell_id\":\"";
    let mut p = s
        .strip_prefix(prefix)
        .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
    let (cell, rest) = take_string(p)?;
    p = rest
        .strip_prefix(",\"members\":[")
        .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
    if cell.is_empty()
        || cell.len() > 128
        || !cell.bytes().all(|b| b.is_ascii() && !b.is_ascii_control())
    {
        return Err(ContextConstructionError::InvalidCellAuthority);
    }
    let mut members = Vec::new();
    while !p.starts_with(']') {
        if !members.is_empty() {
            p = p
                .strip_prefix(',')
                .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        }
        let pre = "{\"manifest_digest\":\"";
        p = p
            .strip_prefix(pre)
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (md, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"manifest_id\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (mi, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"stream_artifact_sha256\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (ad, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"stream_byte_length\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (sl, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"stream_digest\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (sd, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"stream_namespace\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (ns, r) = take_string(p)?;
        p = r
            .strip_prefix(",\"workload_id\":\"")
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        let (wi, r) = take_string(p)?;
        p = r
            .strip_prefix('}')
            .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
        if ![md, ad, sd].iter().all(|x| valid_digest(x)) {
            return Err(ContextConstructionError::InvalidScopeEncoding);
        }
        let namespace =
            parse_uuid(ns).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        parse_uuid(mi).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        parse_uuid(wi).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        let stream_len = decimal(sl)?;
        members.push(Member {
            namespace,
            workload_id: wi.into(),
            manifest_id: mi.into(),
            manifest_digest: md.into(),
            stream_digest: sd.into(),
            stream_len,
            artifact_digest: ad.into(),
        });
        if members.len() > MAX_STREAMS {
            return Err(ContextConstructionError::ResourceLimit);
        }
    }
    p=p.strip_prefix("],\"record_kind\":\"closed_stream_scope\",\"schema_version\":\"EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v1\",\"scope_id\":\"").ok_or(ContextConstructionError::UnsupportedScopeProfile)?;
    let (scope, r) = take_string(p)?;
    if r != "}" {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    let scope_id = parse_uuid(scope).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    if !valid_uuid(scope) || scope_id == [0; 16] {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    if members.is_empty() {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    let mut prev = None;
    let mut workloads = BTreeSet::new();
    let mut manifests = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for m in &members {
        if prev.is_some_and(|x| x >= m.namespace)
            || !workloads.insert(&m.workload_id)
            || !manifests.insert(&m.manifest_id)
            || !digests.insert(&m.manifest_digest)
        {
            return Err(ContextConstructionError::DuplicateStreamNamespace);
        }
        prev = Some(m.namespace);
    }
    Ok(Descriptor {
        scope_id,
        cell_id: cell.into(),
        members,
    })
}
fn take_string(s: &str) -> Result<(&str, &str), ContextConstructionError> {
    let i = s
        .find('"')
        .ok_or(ContextConstructionError::InvalidScopeEncoding)?;
    let v = &s[..i];
    if v.bytes().any(|b| b < 0x20 || b == b'\\') {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    Ok((v, &s[i + 1..]))
}
fn valid_uuid(s: &str) -> bool {
    parse_uuid(s).is_ok()
        && s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_digit() || matches!(b, b'a'..=b'f')
            }
        })
        && parse_uuid(s).is_ok_and(|v| v[8] & 0xc0 == 0x80)
}
fn valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn decimal(s: &str) -> Result<u64, ContextConstructionError> {
    if s.is_empty() || (s.len() > 1 && s.starts_with('0')) || !s.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    s.parse()
        .map_err(|_| ContextConstructionError::InvalidScopeEncoding)
}

// Post-validation extraction only. R16's validator has already established canonical JSON and
// the closed schema before this helper is used.
fn json_string<'a>(bytes: &'a [u8], name: &str) -> Option<&'a str> {
    let text = std::str::from_utf8(bytes).ok()?;
    let needle = format!("\"{name}\":\"");
    let start = text.find(&needle)?.checked_add(needle.len())?;
    let rest = text.get(start..)?;
    let end = rest.find('"')?;
    rest.get(..end)
}

struct Fields {
    request_id: [u8; 16],
    event_id: [u8; 16],
    information_id: [u8; 16],
    references: Vec<[u8; 16]>,
    fact: Fact,
    segment: StreamSegment,
    ordinal: u64,
}
#[derive(Clone, Copy, Debug)]
enum FieldsError {
    Encoding,
    ReferenceLimit,
}
fn operation_fields(bytes: &[u8]) -> Result<Fields, FieldsError> {
    let sop = fields(bytes, b"RDOS-SOP1", 13).ok_or(FieldsError::Encoding)?;
    let op = fields(sop[0], b"RDOS-OP1", 14).ok_or(FieldsError::Encoding)?;
    let env = fields(sop[8], b"RDOS-ENV1", 13).ok_or(FieldsError::Encoding)?;
    let ids = |v: &[u8]| v.try_into().map_err(|_| FieldsError::Encoding);
    let refs = &env[12];
    let count = u32::from_be_bytes(
        refs.get(..4)
            .ok_or(FieldsError::Encoding)?
            .try_into()
            .map_err(|_| FieldsError::Encoding)?,
    ) as usize;
    // This check intentionally precedes Vec allocation. Encoded inputs cannot turn an untrusted
    // u32 into an attempted multi-gigabyte reservation before the frozen R21 bound is enforced.
    if count > MAX_REFERENCES {
        return Err(FieldsError::ReferenceLimit);
    }
    let mut references = Vec::with_capacity(count);
    for v in refs.get(4..).ok_or(FieldsError::Encoding)?.chunks_exact(16) {
        references.push(v.try_into().map_err(|_| FieldsError::Encoding)?)
    }
    if references.len() != count {
        return Err(FieldsError::Encoding);
    }
    Ok(Fields {
        request_id: ids(sop[4])?,
        event_id: ids(sop[5])?,
        information_id: ids(sop[6])?,
        references,
        fact: match env[11] {
            [0] | [1] => Fact::Ordinary,
            [2] => Fact::Correction,
            [3] => Fact::Retraction,
            _ => return Err(FieldsError::Encoding),
        },
        segment: match op[2] {
            [0] => StreamSegment::WarmUp,
            [1] => StreamSegment::Measured,
            _ => return Err(FieldsError::Encoding),
        },
        ordinal: u64::from_be_bytes(op[4].try_into().map_err(|_| FieldsError::Encoding)?),
    })
}
fn extract_stream(
    bytes: &[u8],
    namespace: [u8; 16],
) -> Result<Vec<Operation>, ContextConstructionError> {
    let n = u64::from_be_bytes(
        bytes
            .get(31..39)
            .ok_or(ContextConstructionError::Extraction)?
            .try_into()
            .map_err(|_| ContextConstructionError::Extraction)?,
    );
    let capacity = usize::try_from(n).map_err(|_| ContextConstructionError::ResourceLimit)?;
    if capacity > MAX_OPERATIONS {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let mut out = Vec::with_capacity(capacity);
    let mut p = 55usize;
    for _ in 0..n {
        let z = usize::try_from(u64::from_be_bytes(
            bytes
                .get(p..p + 8)
                .ok_or(ContextConstructionError::Extraction)?
                .try_into()
                .map_err(|_| ContextConstructionError::Extraction)?,
        ))
        .map_err(|_| ContextConstructionError::ResourceLimit)?;
        p = p
            .checked_add(8)
            .ok_or(ContextConstructionError::ResourceLimit)?;
        let end = p
            .checked_add(z)
            .ok_or(ContextConstructionError::ResourceLimit)?;
        let b = bytes
            .get(p..end)
            .ok_or(ContextConstructionError::Extraction)?;
        let f = operation_fields(b).map_err(|error| match error {
            FieldsError::Encoding => ContextConstructionError::Extraction,
            FieldsError::ReferenceLimit => ContextConstructionError::ResourceLimit,
        })?;
        let sop = fields(b, b"RDOS-SOP1", 13).ok_or(ContextConstructionError::Extraction)?;
        let op = fields(sop[0], b"RDOS-OP1", 14).ok_or(ContextConstructionError::Extraction)?;
        if op[10] != namespace {
            return Err(ContextConstructionError::SubstitutedStream);
        }
        out.push(Operation {
            bytes: b.to_vec(),
            event_id: f.event_id,
            references: f.references,
            segment: f.segment,
            segment_ordinal: f.ordinal,
        });
        p = end
    }
    Ok(out)
}
fn fields<'a>(bytes: &'a [u8], magic: &[u8], count: u16) -> Option<Vec<&'a [u8]>> {
    if bytes.get(..magic.len())? != magic
        || u16::from_be_bytes(bytes.get(magic.len()..magic.len() + 2)?.try_into().ok()?) != count
    {
        return None;
    }
    let mut p = magic.len() + 2;
    let mut out = Vec::with_capacity(count as usize);
    for tag in 1..=count {
        if *bytes.get(p)? != tag as u8 {
            return None;
        }
        let n = u32::from_be_bytes(bytes.get(p + 1..p + 5)?.try_into().ok()?) as usize;
        p = p.checked_add(5)?;
        let e = p.checked_add(n)?;
        out.push(bytes.get(p..e)?);
        p = e
    }
    (p == bytes.len()).then_some(out)
}

#[cfg(test)]
mod allocation_bound_tests {
    use super::*;

    fn record(magic: &[u8], fields: &[Vec<u8>]) -> Vec<u8> {
        let mut out = magic.to_vec();
        out.extend(u16::try_from(fields.len()).unwrap().to_be_bytes());
        for (index, value) in fields.iter().enumerate() {
            out.push(u8::try_from(index + 1).unwrap());
            out.extend(u32::try_from(value.len()).unwrap().to_be_bytes());
            out.extend(value);
        }
        out
    }

    fn operation_with_encoded_reference_count(count: usize, include_members: bool) -> Vec<u8> {
        let mut op = vec![Vec::new(); 14];
        op[2] = vec![0];
        op[4] = 0_u64.to_be_bytes().to_vec();
        let op = record(b"RDOS-OP1", &op);

        let mut refs = u32::try_from(count).unwrap().to_be_bytes().to_vec();
        if include_members {
            refs.resize(4 + count * 16, 0);
        }
        let mut env = vec![Vec::new(); 13];
        env[11] = vec![1];
        env[12] = refs;
        let env = record(b"RDOS-ENV1", &env);

        let mut sop = vec![Vec::new(); 13];
        sop[0] = op;
        sop[4] = vec![0; 16];
        sop[5] = vec![0; 16];
        sop[6] = vec![0; 16];
        sop[8] = env;
        record(b"RDOS-SOP1", &sop)
    }

    #[test]
    fn reference_limit_is_inclusive_and_one_over_is_rejected_before_allocation() {
        let exact = operation_with_encoded_reference_count(MAX_REFERENCES, true);
        assert_eq!(
            operation_fields(&exact).unwrap().references.len(),
            MAX_REFERENCES
        );

        // Deliberately omit the 1,048,592 member bytes. ReferenceLimit must be returned from the
        // four-byte count alone, rather than attempting Vec::with_capacity(65_537) or reporting
        // the later encoding-length error.
        let over = operation_with_encoded_reference_count(MAX_REFERENCES + 1, false);
        assert!(matches!(
            operation_fields(&over),
            Err(FieldsError::ReferenceLimit)
        ));
    }
}
