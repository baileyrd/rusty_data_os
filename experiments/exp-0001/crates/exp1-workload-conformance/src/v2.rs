//! R26's side-by-side v2 causal-reference wire conformance.

use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub const WORKLOAD_CONTRACT_V2: &str = "EXP-0000-WORKLOADS-v2-causal-reference";
pub const ENVELOPE_PROFILE_V2: &str = "envelope-causal-reference-v2";
pub const ENVELOPE_GENERATOR_V2: &str = "EXP-0001-ENVELOPE-INPUT-v2";
pub const REFERENCE_GENERATOR_V2: &str = "EXP-0001-PRIOR-EVENTS-v2";
pub const SEMANTIC_OPERATION_V2: &str = "EXP-0001-SEMANTIC-OP-v2";
pub const WORKLOAD_STREAM_V2: &str = "EXP-0001-WORKLOAD-STREAM-v2";
pub const STREAM_DIGEST_PROFILE_V2: &str = "EXP-0001-WORKLOAD-STREAM-DIGEST-v2";
pub const MANIFEST_PROFILE_V2: &str = "EXP-0001-WORKLOAD-MANIFEST-JCS-v2";
pub const MANIFEST_DIGEST_PROFILE_V2: &str = "EXP-0001-WORKLOAD-MANIFEST-DIGEST-v2";

#[derive(Clone, Debug)]
pub struct EnvelopeInputV2<'a> {
    pub common: EnvelopeInput<'a>,
}

fn ref2(references: &[[u8; 16]]) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    out.extend(
        u32::try_from(references.len())
            .map_err(|_| Error::ResourceLimit)?
            .to_be_bytes(),
    );
    for id in references {
        out.extend(id);
    }
    Ok(out)
}

pub fn envelope_input_v2(input: &EnvelopeInputV2<'_>) -> Result<Vec<u8>, Error> {
    let i = &input.common;
    if i.operation.envelope != Envelope::Causal || i.semantics != ReferenceSemantics::Causal {
        return Err(Error::ProfileMismatch);
    }
    if i.semantic_version.is_empty() || i.fact_type.is_empty() || i.schema_version.is_empty() {
        return Err(Error::Tuple);
    }
    if !valid_uuid_bytes(&i.schema_id)
        || !valid_uuid_bytes(&i.request_id)
        || !valid_uuid_bytes(&i.event_id)
        || !valid_uuid_bytes(&i.information_id)
        || i.references.iter().any(|x| !valid_uuid_bytes(x))
    {
        return Err(Error::Type);
    }
    let mut seen = BTreeSet::new();
    if i.references.iter().any(|x| !seen.insert(*x)) {
        return Err(Error::ReferenceDuplicate);
    }
    let option = |x: Option<&str>| {
        let mut v = vec![u8::from(x.is_some())];
        if let Some(x) = x {
            v.extend(x.as_bytes())
        };
        v
    };
    let mut f = Vec::new();
    field(&mut f, 1, &i.operation.encode()?)?;
    field(&mut f, 2, i.semantic_version.as_bytes())?;
    field(&mut f, 3, i.fact_type.as_bytes())?;
    field(&mut f, 4, &i.schema_id)?;
    field(&mut f, 5, i.schema_version.as_bytes())?;
    field(&mut f, 6, &option(i.source))?;
    field(&mut f, 7, &option(i.actor))?;
    field(&mut f, 8, &i.request_id)?;
    field(&mut f, 9, &i.event_id)?;
    field(&mut f, 10, &i.information_id)?;
    field(&mut f, 11, &i.effective_time.to_be_bytes())?;
    field(&mut f, 12, &[1])?;
    field(&mut f, 13, &ref2(i.references)?)?;
    Ok(record(b"RDOS-ENV2", 13, f))
}

#[derive(Clone, Debug)]
pub struct SemanticOperationV2 {
    pub op1: Vec<u8>,
    pub payload_profile: &'static str,
    pub payload: Vec<u8>,
    pub request_id: [u8; 16],
    pub event_id: [u8; 16],
    pub information_id: [u8; 16],
    pub env2: Vec<u8>,
    pub base_ns: i64,
    pub unit_ns: i64,
}
impl SemanticOperationV2 {
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut f = Vec::new();
        field(&mut f, 1, &self.op1)?;
        field(&mut f, 2, self.payload_profile.as_bytes())?;
        field(&mut f, 3, &self.payload)?;
        field(&mut f, 4, b"EXP-0001-UUID4-SHA256-v1")?;
        field(&mut f, 5, &self.request_id)?;
        field(&mut f, 6, &self.event_id)?;
        field(&mut f, 7, &self.information_id)?;
        field(&mut f, 8, ENVELOPE_GENERATOR_V2.as_bytes())?;
        field(&mut f, 9, &self.env2)?;
        field(&mut f, 10, REFERENCE_GENERATOR_V2.as_bytes())?;
        field(&mut f, 11, b"EXP-0001-LOGICAL-TIME-v1")?;
        field(&mut f, 12, &self.base_ns.to_be_bytes())?;
        field(&mut f, 13, &self.unit_ns.to_be_bytes())?;
        let out = record(b"RDOS-SOP2", 13, f);
        validate_semantic_operation_v2(&out)?;
        Ok(out)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedV2 {
    pub segment: Segment,
    pub ordinal: u64,
    pub namespace: [u8; 16],
    pub event_id: [u8; 16],
    pub references: Vec<[u8; 16]>,
}

pub fn validate_semantic_operation_v2(bytes: &[u8]) -> Result<DecodedV2, Error> {
    let f = fields(bytes, b"RDOS-SOP2", 13)?;
    let op = validate_op1(f[0])?;
    if f[1] != b"EXP-0001-SHA256-CTR-v1"
        && f[1] != b"EXP-0001-SHA256-MOTIF-v1"
        && f[1] != b"EXP-0001-ZERO-v1"
    {
        return Err(Error::ProfileMismatch);
    }
    if f[3] != b"EXP-0001-UUID4-SHA256-v1"
        || f[7] != ENVELOPE_GENERATOR_V2.as_bytes()
        || f[9] != REFERENCE_GENERATOR_V2.as_bytes()
        || f[10] != b"EXP-0001-LOGICAL-TIME-v1"
    {
        return Err(Error::ProfileMismatch);
    }
    if op[8] != [3]
        || f[4..=6].iter().any(|x| !valid_uuid_bytes(x))
        || f[11].len() != 8
        || f[12].len() != 8
    {
        return Err(Error::Encoding);
    }
    let env = fields(f[8], b"RDOS-ENV2", 13)?;
    if env[0] != f[0] || env[7] != f[4] || env[8] != f[5] || env[9] != f[6] {
        return Err(Error::ProfileMismatch);
    }
    if env[1].is_empty()
        || env[2].is_empty()
        || env[4].is_empty()
        || !valid_uuid_bytes(env[3])
        || option_text(env[5]).is_err()
        || option_text(env[6]).is_err()
        || env[10].len() != 8
        || env[11] != [1]
        || env[12].len() < 4
    {
        return Err(Error::ProfileMismatch);
    }
    let count = u32::from_be_bytes(env[12][..4].try_into().map_err(|_| Error::Encoding)?) as usize;
    if env[12].len()
        != 4usize
            .checked_add(count.checked_mul(16).ok_or(Error::ResourceLimit)?)
            .ok_or(Error::ResourceLimit)?
    {
        return Err(Error::Encoding);
    }
    let mut references = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();
    for x in env[12][4..].chunks_exact(16) {
        if !valid_uuid_bytes(x) {
            return Err(Error::Encoding);
        }
        let id = x.try_into().map_err(|_| Error::Encoding)?;
        if !seen.insert(id) {
            return Err(Error::ReferenceDuplicate);
        }
        references.push(id);
    }
    let size = op[5][0] as usize;
    if [0, 32, 256, 4096, 65536, 1048576].get(size).copied() != Some(f[2].len()) {
        return Err(Error::ProfileMismatch);
    }
    let expected = match op[7][0] {
        1 => b"EXP-0001-SHA256-CTR-v1".as_slice(),
        2 => b"EXP-0001-SHA256-MOTIF-v1".as_slice(),
        3 => b"EXP-0001-ZERO-v1".as_slice(),
        _ => return Err(Error::Encoding),
    };
    if f[1] != expected {
        return Err(Error::ProfileMismatch);
    }
    let segment = if op[2] == [0] {
        Segment::WarmUp
    } else {
        Segment::Measured
    };
    let input = OperationInput {
        segment,
        seed: u64::from_be_bytes(op[3].try_into().map_err(|_| Error::Encoding)?),
        ordinal: u64::from_be_bytes(op[4].try_into().map_err(|_| Error::Encoding)?),
        size_class: op[5][0],
        content: match op[7][0] {
            1 => Content::High,
            2 => Content::Low,
            _ => Content::Zero,
        },
        envelope: Envelope::Causal,
        temporal: match op[9][0] {
            1 => Temporal::Monotonic,
            2 => Temporal::EqualBurst,
            3 => Temporal::Late,
            _ => Temporal::OutOfOrder,
        },
        stream_namespace: op[10].try_into().map_err(|_| Error::Encoding)?,
        producer_id: op[11].try_into().map_err(|_| Error::Encoding)?,
        producer_ordinal: u64::from_be_bytes(op[12].try_into().map_err(|_| Error::Encoding)?),
        controlled_schedule: if op[13] == [0] {
            None
        } else {
            Some(
                op[13]
                    .get(1..)
                    .ok_or(Error::Encoding)?
                    .try_into()
                    .map_err(|_| Error::Encoding)?,
            )
        },
    };
    let base = i64::from_be_bytes(f[11].try_into().map_err(|_| Error::Encoding)?);
    let unit = i64::from_be_bytes(f[12].try_into().map_err(|_| Error::Encoding)?);
    if unit <= 0 {
        return Err(Error::LogicalTimeParameter);
    }
    if f[2] != payload(&input)?
        || f[4] != identity(&input, IdentityKind::Request)?
        || f[5] != identity(&input, IdentityKind::Event)?
        || f[6] != identity(&input, IdentityKind::Information)?
        || env[10] != logical_time(input.temporal, input.ordinal, base, unit)?.to_be_bytes()
    {
        return Err(Error::ProfileMismatch);
    }
    Ok(DecodedV2 {
        segment,
        ordinal: u64::from_be_bytes(op[4].try_into().map_err(|_| Error::Encoding)?),
        namespace: op[10].try_into().map_err(|_| Error::Encoding)?,
        event_id: f[5].try_into().map_err(|_| Error::Encoding)?,
        references,
    })
}

pub fn workload_stream_v2(ops: &[Vec<u8>], warm_up: u64, measured: u64) -> Result<Vec<u8>, Error> {
    let n = warm_up.checked_add(measured).ok_or(Error::CountMismatch)?;
    if usize::try_from(n).ok() != Some(ops.len()) {
        return Err(Error::CountMismatch);
    }
    let mut out = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2".to_vec();
    out.extend(n.to_be_bytes());
    out.extend(warm_up.to_be_bytes());
    out.extend(measured.to_be_bytes());
    for op in ops {
        validate_semantic_operation_v2(op)?;
        out.extend(
            u64::try_from(op.len())
                .map_err(|_| Error::ResourceLimit)?
                .to_be_bytes(),
        );
        out.extend(op);
    }
    validate_stream_v2(&out, 1, 1)?;
    Ok(out)
}

pub fn validate_stream_v2(
    bytes: &[u8],
    warm_subsequent: u64,
    measured_subsequent: u64,
) -> Result<(u64, u64, u64), Error> {
    const H: &[u8] = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2";
    if bytes.len() < H.len() + 24 || &bytes[..H.len()] != H {
        return Err(Error::Encoding);
    }
    if warm_subsequent == 0 || measured_subsequent == 0 {
        return Err(Error::Range);
    }
    let u = |p| u64::from_be_bytes(bytes[p..p + 8].try_into().unwrap());
    let (n, w, m) = (u(H.len()), u(H.len() + 8), u(H.len() + 16));
    if w.checked_add(m) != Some(n) {
        return Err(Error::CountMismatch);
    }
    let mut p = H.len() + 24;
    let mut namespace = None;
    for index in 0..n {
        if p + 8 > bytes.len() {
            return Err(Error::Encoding);
        }
        let z = usize::try_from(u(p)).map_err(|_| Error::ResourceLimit)?;
        p += 8;
        let end = p.checked_add(z).ok_or(Error::Encoding)?;
        if end > bytes.len() {
            return Err(Error::Encoding);
        }
        let d = validate_semantic_operation_v2(&bytes[p..end])?;
        p = end;
        let (seg, ord, k) = if index < w {
            (Segment::WarmUp, index, warm_subsequent)
        } else {
            (Segment::Measured, index - w, measured_subsequent)
        };
        if d.segment != seg || d.ordinal != ord {
            return Err(Error::Ordering);
        }
        if namespace.is_some_and(|x| x != d.namespace) {
            return Err(Error::ProfileMismatch);
        }
        namespace = Some(d.namespace);
        if (ord == 0 && !d.references.is_empty())
            || (ord > 0
                && d.references.len() != usize::try_from(k).map_err(|_| Error::ResourceLimit)?)
        {
            return Err(Error::ReferenceCardinality);
        }
        if ord > 0 && k > ord {
            return Err(Error::CountMismatch);
        }
    }
    if p != bytes.len() {
        return Err(Error::Encoding);
    }
    Ok((n, w, m))
}

pub fn workload_digest_v2(bytes: &[u8]) -> [u8; 32] {
    domain_digest(STREAM_DOMAIN_V2, bytes)
}
pub fn manifest_digest_v2(bytes: &[u8]) -> [u8; 32] {
    domain_digest(MANIFEST_DOMAIN_V2, bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    Ordinary,
    Other,
}
#[derive(Clone, Debug)]
pub struct ReferenceEntry {
    pub id: [u8; 16],
    pub namespace: [u8; 16],
    pub segment: Segment,
    pub ordinal: u64,
    pub kind: EventKind,
    pub fact_type: String,
}
pub fn classify_references(
    op: &DecodedV2,
    fact_type: &str,
    catalog: &BTreeMap<[u8; 16], ReferenceEntry>,
    complete_scope: bool,
) -> Result<(), Error> {
    for id in &op.references {
        if id == &op.event_id {
            return Err(Error::ReferenceSelf);
        };
        if let Some(x) = catalog.get(id) {
            if x.kind != EventKind::Ordinary {
                return Err(Error::ReferenceWrongKind);
            }
            if x.fact_type != fact_type {
                return Err(Error::ReferenceWrongFact);
            }
            if x.namespace != op.namespace {
                return Err(Error::ReferenceCrossStream);
            }
            if x.segment != op.segment {
                return Err(Error::ReferenceCrossSegment);
            }
            if x.ordinal >= op.ordinal {
                return Err(Error::ReferenceFuture);
            }
        } else if complete_scope {
            return Err(Error::ReferenceMissing);
        } else {
            return Err(Error::ContextRequired);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AcceptedPrefix {
    pub warm_up: u64,
    pub measured: u64,
}
pub fn accept_transactionally(
    state: &mut AcceptedPrefix,
    op: &DecodedV2,
    fact_type: &str,
    catalog: &BTreeMap<[u8; 16], ReferenceEntry>,
    complete_scope: bool,
    subsequent: u64,
) -> Result<(), Error> {
    let expected = match op.segment {
        Segment::WarmUp => state.warm_up,
        Segment::Measured => state.measured,
    };
    if op.ordinal != expected {
        return Err(Error::Ordering);
    }
    if (expected == 0 && !op.references.is_empty())
        || (expected > 0
            && op.references.len()
                != usize::try_from(subsequent).map_err(|_| Error::ResourceLimit)?)
    {
        return Err(Error::ReferenceCardinality);
    }
    classify_references(op, fact_type, catalog, complete_scope)?;
    match op.segment {
        Segment::WarmUp => state.warm_up = next_ordinal(state.warm_up)?,
        Segment::Measured => state.measured = next_ordinal(state.measured)?,
    };
    Ok(())
}
