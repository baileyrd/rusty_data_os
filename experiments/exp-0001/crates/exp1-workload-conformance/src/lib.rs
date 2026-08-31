//! Dependency-free executable conformance for the frozen EXP-0001 R12/R14/R16 contracts.
//! This crate only constructs and validates semantic bytes; it executes no workload.

use std::collections::BTreeSet;

pub const MAX_PAYLOAD: usize = 1_048_576;
pub const STREAM_DOMAIN: &str = "rusty-data-os/exp1/workload-stream/v1";
pub const MANIFEST_DOMAIN: &str = "rusty-data-os/exp1/workload-manifest/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    SeedSyntax,
    SeedRange,
    Unsupported,
    Tuple,
    SizeClass,
    ResourceLimit,
    OrdinalOverflow,
    LogicalTimeParameter,
    LogicalTimeOverflow,
    Encoding,
    ProfileMismatch,
    CountMismatch,
    Noncanonical,
    JsonSyntax,
    DuplicateMember,
    UnknownField,
    MissingField,
    Type,
    Range,
    Digest,
    Reference,
    SupersessionCycle,
    ImmutableState,
    Ordering,
    DuplicateOrConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Segment {
    WarmUp,
    Measured,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Content {
    High,
    Low,
    Zero,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Envelope {
    Minimal,
    Provenance,
    Causal,
    CorrectionRetraction,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Temporal {
    Monotonic,
    EqualBurst,
    Late,
    OutOfOrder,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityKind {
    Request,
    Event,
    Information,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationInput {
    pub segment: Segment,
    pub seed: u64,
    pub ordinal: u64,
    pub size_class: u8,
    pub content: Content,
    pub envelope: Envelope,
    pub temporal: Temporal,
    pub stream_namespace: [u8; 16],
    pub producer_id: [u8; 16],
    pub producer_ordinal: u64,
    pub controlled_schedule: Option<[u8; 16]>,
}
impl OperationInput {
    pub fn payload_len(&self) -> Result<usize, Error> {
        [0, 32, 256, 4096, 65536, 1048576]
            .get(self.size_class as usize)
            .copied()
            .ok_or(Error::Unsupported)
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        if self.producer_ordinal != self.ordinal {
            return Err(Error::Tuple);
        }
        if !valid_uuid_bytes(&self.stream_namespace)
            || !valid_uuid_bytes(&self.producer_id)
            || self
                .controlled_schedule
                .is_some_and(|id| !valid_uuid_bytes(&id))
        {
            return Err(Error::Type);
        }
        let len = self.payload_len()?;
        let mut fields = Vec::new();
        field(&mut fields, 1, &1u16.to_be_bytes())?;
        field(&mut fields, 2, &1u16.to_be_bytes())?;
        field(
            &mut fields,
            3,
            &[match self.segment {
                Segment::WarmUp => 0,
                Segment::Measured => 1,
            }],
        )?;
        field(&mut fields, 4, &self.seed.to_be_bytes())?;
        field(&mut fields, 5, &self.ordinal.to_be_bytes())?;
        field(&mut fields, 6, &[self.size_class])?;
        field(&mut fields, 7, &(len as u32).to_be_bytes())?;
        field(
            &mut fields,
            8,
            &[match self.content {
                Content::High => 1,
                Content::Low => 2,
                Content::Zero => 3,
            }],
        )?;
        field(
            &mut fields,
            9,
            &[match self.envelope {
                Envelope::Minimal => 1,
                Envelope::Provenance => 2,
                Envelope::Causal => 3,
                Envelope::CorrectionRetraction => 4,
            }],
        )?;
        field(
            &mut fields,
            10,
            &[match self.temporal {
                Temporal::Monotonic => 1,
                Temporal::EqualBurst => 2,
                Temporal::Late => 3,
                Temporal::OutOfOrder => 4,
            }],
        )?;
        field(&mut fields, 11, &self.stream_namespace)?;
        field(&mut fields, 12, &self.producer_id)?;
        field(&mut fields, 13, &self.producer_ordinal.to_be_bytes())?;
        let schedule = match self.controlled_schedule {
            None => vec![0],
            Some(v) => {
                let mut x = vec![1];
                x.extend(v);
                x
            }
        };
        field(&mut fields, 14, &schedule)?;
        Ok(record(b"RDOS-OP1", 14, fields))
    }
}

pub fn parse_seed(text: &str) -> Result<u64, Error> {
    let b = text.as_bytes();
    if b.is_empty() || (b.len() > 1 && b[0] == b'0') || !b.iter().all(u8::is_ascii_digit) {
        return Err(Error::SeedSyntax);
    }
    text.parse().map_err(|_| Error::SeedRange)
}
pub fn next_ordinal(value: u64) -> Result<u64, Error> {
    value.checked_add(1).ok_or(Error::OrdinalOverflow)
}

fn field(out: &mut Vec<u8>, tag: u8, value: &[u8]) -> Result<(), Error> {
    let n = u32::try_from(value.len()).map_err(|_| Error::ResourceLimit)?;
    out.push(tag);
    out.extend(n.to_be_bytes());
    out.extend(value);
    Ok(())
}
fn record(magic: &[u8], count: u16, fields: Vec<u8>) -> Vec<u8> {
    let mut v = magic.to_vec();
    v.extend(count.to_be_bytes());
    v.extend(fields);
    v
}

pub fn payload(input: &OperationInput) -> Result<Vec<u8>, Error> {
    let op = input.encode()?;
    let len = input.payload_len()?;
    if len > MAX_PAYLOAD {
        return Err(Error::ResourceLimit);
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    match input.content {
        Content::Zero => Ok(vec![0; len]),
        Content::Low => {
            let d = pay_block(&op, 0)?;
            Ok((0..len).map(|i| d[i % 8]).collect())
        }
        Content::High => {
            let mut out = Vec::with_capacity(len);
            let mut i = 0u32;
            while out.len() < len {
                out.extend(pay_block(&op, i)?);
                i = i.checked_add(1).ok_or(Error::ResourceLimit)?;
            }
            out.truncate(len);
            Ok(out)
        }
    }
}
fn pay_block(op: &[u8], index: u32) -> Result<[u8; 32], Error> {
    let mut f = Vec::new();
    field(&mut f, 1, op)?;
    field(&mut f, 2, &index.to_be_bytes())?;
    Ok(sha256(&record(b"RDOS-PAY1", 2, f)))
}

pub fn identity(input: &OperationInput, kind: IdentityKind) -> Result<[u8; 16], Error> {
    let op = input.encode()?;
    let (tag, ns) = match kind {
        IdentityKind::Request => (1, parse_uuid("a1111111-1111-4111-8111-111111111111")?),
        IdentityKind::Event => (2, parse_uuid("b2222222-2222-4222-8222-222222222222")?),
        IdentityKind::Information => (3, parse_uuid("c3333333-3333-4333-8333-333333333333")?),
    };
    let mut f = Vec::new();
    field(&mut f, 1, &op)?;
    field(&mut f, 2, &[tag])?;
    field(&mut f, 3, &1u16.to_be_bytes())?;
    field(&mut f, 4, &ns)?;
    let d = sha256(&record(b"RDOS-ID1", 4, f));
    let mut id = [0; 16];
    id.copy_from_slice(&d[..16]);
    id[6] = (id[6] & 15) | 64;
    id[8] = (id[8] & 63) | 128;
    Ok(id)
}
pub fn parse_uuid(s: &str) -> Result<[u8; 16], Error> {
    if s.len() != 36
        || [8, 13, 18, 23].iter().any(|&i| s.as_bytes()[i] != b'-')
        || s.bytes().any(|b| b.is_ascii_uppercase())
    {
        return Err(Error::Type);
    }
    let h: String = s.chars().filter(|&c| c != '-').collect();
    let v = decode_hex(&h)?;
    let mut a = [0; 16];
    a.copy_from_slice(&v);
    if a.iter().all(|&x| x == 0) || (a[8] & 0xc0) != 0x80 {
        return Err(Error::Range);
    }
    Ok(a)
}
pub fn format_uuid(v: [u8; 16]) -> String {
    let h = hex(&v);
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}

pub fn logical_time(profile: Temporal, ordinal: u64, base: i64, unit: i64) -> Result<i64, Error> {
    if unit <= 0 {
        return Err(Error::LogicalTimeParameter);
    }
    let i = i128::from(ordinal);
    let u = i128::from(unit);
    let b = i128::from(base);
    let n = match profile {
        Temporal::Monotonic => i,
        Temporal::EqualBurst => i / 100,
        Temporal::Late => i - if ordinal % 10 == 9 { 100 } else { 0 },
        Temporal::OutOfOrder => 4 * (i / 4) + [0, 2, 1, 3][ordinal as usize % 4],
    };
    i64::try_from(b + n * u).map_err(|_| Error::LogicalTimeOverflow)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceSemantics {
    None,
    Causal,
    Correction,
    Retraction,
}

#[derive(Clone, Debug)]
pub struct EnvelopeInput<'a> {
    pub operation: &'a OperationInput,
    pub semantic_version: &'a str,
    pub fact_type: &'a str,
    pub schema_id: [u8; 16],
    pub schema_version: &'a str,
    pub source: Option<&'a str>,
    pub actor: Option<&'a str>,
    pub request_id: [u8; 16],
    pub event_id: [u8; 16],
    pub information_id: [u8; 16],
    pub effective_time: i64,
    pub semantics: ReferenceSemantics,
    pub references: &'a [[u8; 16]],
}

pub fn envelope_input(input: &EnvelopeInput<'_>) -> Result<Vec<u8>, Error> {
    if input.semantic_version.is_empty()
        || input.fact_type.is_empty()
        || input.schema_version.is_empty()
    {
        return Err(Error::Tuple);
    }
    if !valid_uuid_bytes(&input.schema_id)
        || !valid_uuid_bytes(&input.request_id)
        || !valid_uuid_bytes(&input.event_id)
        || !valid_uuid_bytes(&input.information_id)
        || input.references.iter().any(|id| !valid_uuid_bytes(id))
    {
        return Err(Error::Type);
    }
    let mut seen = BTreeSet::new();
    for id in input.references {
        if !seen.insert(id) || id == &input.event_id {
            return Err(Error::DuplicateOrConflict);
        }
    }
    let applicable = match input.operation.envelope {
        Envelope::Minimal => {
            input.source.is_none()
                && input.actor.is_none()
                && input.semantics == ReferenceSemantics::None
                && input.references.is_empty()
        }
        Envelope::Provenance => {
            input.source.is_some_and(|x| !x.is_empty())
                && input.actor.is_some_and(|x| !x.is_empty())
                && input.semantics == ReferenceSemantics::None
                && input.references.is_empty()
        }
        Envelope::Causal => {
            input.semantics == ReferenceSemantics::Causal && !input.references.is_empty()
        }
        Envelope::CorrectionRetraction => {
            matches!(
                input.semantics,
                ReferenceSemantics::Correction | ReferenceSemantics::Retraction
            ) && !input.references.is_empty()
        }
    };
    if !applicable {
        return Err(Error::ProfileMismatch);
    }
    let option = |x: Option<&str>| {
        let mut v = vec![u8::from(x.is_some())];
        if let Some(x) = x {
            v.extend(x.as_bytes())
        }
        v
    };
    let mut refs = Vec::new();
    refs.extend(
        u32::try_from(input.references.len())
            .map_err(|_| Error::ResourceLimit)?
            .to_be_bytes(),
    );
    for id in input.references {
        refs.extend(id)
    }
    let mut f = Vec::new();
    field(&mut f, 1, &input.operation.encode()?)?;
    field(&mut f, 2, input.semantic_version.as_bytes())?;
    field(&mut f, 3, input.fact_type.as_bytes())?;
    field(&mut f, 4, &input.schema_id)?;
    field(&mut f, 5, input.schema_version.as_bytes())?;
    field(&mut f, 6, &option(input.source))?;
    field(&mut f, 7, &option(input.actor))?;
    field(&mut f, 8, &input.request_id)?;
    field(&mut f, 9, &input.event_id)?;
    field(&mut f, 10, &input.information_id)?;
    field(&mut f, 11, &input.effective_time.to_be_bytes())?;
    field(
        &mut f,
        12,
        &[match input.semantics {
            ReferenceSemantics::None => 0,
            ReferenceSemantics::Causal => 1,
            ReferenceSemantics::Correction => 2,
            ReferenceSemantics::Retraction => 3,
        }],
    )?;
    field(&mut f, 13, &refs)?;
    Ok(record(b"RDOS-ENV1", 13, f))
}

/// Selects the frozen ascending suffix of ordinary prior EventIds.
pub fn prior_events(
    prior_ordinary: &[[u8; 16]],
    ordinal: u64,
    cardinality: u64,
) -> Result<Vec<[u8; 16]>, Error> {
    if cardinality == 0
        || cardinality > ordinal
        || usize::try_from(ordinal).ok() != Some(prior_ordinary.len())
    {
        return Err(Error::Reference);
    }
    let k = usize::try_from(cardinality).map_err(|_| Error::ResourceLimit)?;
    let selected = prior_ordinary[prior_ordinary.len() - k..].to_vec();
    let mut seen = BTreeSet::new();
    if selected.iter().any(|x| !seen.insert(*x)) {
        return Err(Error::DuplicateOrConflict);
    }
    Ok(selected)
}

#[derive(Clone, Debug)]
pub struct SemanticOperation {
    pub op1: Vec<u8>,
    pub payload_profile: &'static str,
    pub payload: Vec<u8>,
    pub request_id: [u8; 16],
    pub event_id: [u8; 16],
    pub information_id: [u8; 16],
    pub env1: Vec<u8>,
    pub base_ns: i64,
    pub unit_ns: i64,
}
impl SemanticOperation {
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        validate_op1(&self.op1)?;
        validate_env1(&self.env1, &self.op1)?;
        if self.unit_ns <= 0 {
            return Err(Error::LogicalTimeParameter);
        }
        let mut f = Vec::new();
        field(&mut f, 1, &self.op1)?;
        field(&mut f, 2, self.payload_profile.as_bytes())?;
        field(&mut f, 3, &self.payload)?;
        field(&mut f, 4, b"EXP-0001-UUID4-SHA256-v1")?;
        field(&mut f, 5, &self.request_id)?;
        field(&mut f, 6, &self.event_id)?;
        field(&mut f, 7, &self.information_id)?;
        field(&mut f, 8, b"EXP-0001-ENVELOPE-INPUT-v1")?;
        field(&mut f, 9, &self.env1)?;
        field(&mut f, 10, b"EXP-0001-PRIOR-EVENTS-v1")?;
        field(&mut f, 11, b"EXP-0001-LOGICAL-TIME-v1")?;
        field(&mut f, 12, &self.base_ns.to_be_bytes())?;
        field(&mut f, 13, &self.unit_ns.to_be_bytes())?;
        let candidate = record(b"RDOS-SOP1", 13, f);
        validate_semantic_operation(&candidate)?;
        Ok(candidate)
    }
}

fn one(f: &[u8], allowed: &[u8]) -> Result<u8, Error> {
    if f.len() != 1 || !allowed.contains(&f[0]) {
        Err(Error::Unsupported)
    } else {
        Ok(f[0])
    }
}

fn validate_op1(bytes: &[u8]) -> Result<Vec<&[u8]>, Error> {
    let f = fields(bytes, b"RDOS-OP1", 14)?;
    if f[0] != 1u16.to_be_bytes() || f[1] != 1u16.to_be_bytes() {
        return Err(Error::Unsupported);
    }
    one(f[2], &[0, 1])?;
    if f[3].len() != 8
        || f[4].len() != 8
        || f[6].len() != 4
        || f[10].len() != 16
        || f[11].len() != 16
        || f[12].len() != 8
    {
        return Err(Error::Encoding);
    }
    let size = one(f[5], &[0, 1, 2, 3, 4, 5])? as usize;
    let declared = u32::from_be_bytes(f[6].try_into().map_err(|_| Error::Encoding)?) as usize;
    if [0, 32, 256, 4096, 65536, 1048576][size] != declared {
        return Err(Error::SizeClass);
    }
    one(f[7], &[1, 2, 3])?;
    one(f[8], &[1, 2, 3, 4])?;
    one(f[9], &[1, 2, 3, 4])?;
    if f[12] != f[4] {
        return Err(Error::Tuple);
    }
    match f[13] {
        [0] => {}
        [1, rest @ ..] if rest.len() == 16 => {}
        _ => return Err(Error::Encoding),
    }
    Ok(f)
}

fn valid_uuid_bytes(v: &[u8]) -> bool {
    v.len() == 16 && v.iter().any(|&x| x != 0) && (v[6] & 0xf0) == 0x40 && (v[8] & 0xc0) == 0x80
}

fn option_text(v: &[u8]) -> Result<Option<&[u8]>, Error> {
    match v {
        [0] => Ok(None),
        [1, rest @ ..] if !rest.is_empty() && std::str::from_utf8(rest).is_ok() => Ok(Some(rest)),
        _ => Err(Error::Encoding),
    }
}

fn validate_env1<'a>(bytes: &'a [u8], op_bytes: &[u8]) -> Result<Vec<&'a [u8]>, Error> {
    let env = fields(bytes, b"RDOS-ENV1", 13)?;
    if env[0] != op_bytes
        || env[1].is_empty()
        || env[2].is_empty()
        || env[4].is_empty()
        || !valid_uuid_bytes(env[3])
        || !env[7..=9].iter().all(|x| valid_uuid_bytes(x))
        || std::str::from_utf8(env[1]).is_err()
        || std::str::from_utf8(env[2]).is_err()
        || std::str::from_utf8(env[4]).is_err()
        || env[10].len() != 8
        || env[11].len() != 1
        || env[12].len() < 4
    {
        return Err(Error::Encoding);
    }
    let source = option_text(env[5])?;
    let actor = option_text(env[6])?;
    let count = u32::from_be_bytes(env[12][..4].try_into().map_err(|_| Error::Encoding)?) as usize;
    if env[12].len()
        != 4usize
            .checked_add(16usize.checked_mul(count).ok_or(Error::ResourceLimit)?)
            .ok_or(Error::ResourceLimit)?
    {
        return Err(Error::Encoding);
    }
    let mut seen = BTreeSet::new();
    for id in env[12][4..].chunks_exact(16) {
        if !valid_uuid_bytes(id) || !seen.insert(id) {
            return Err(Error::DuplicateOrConflict);
        }
    }
    let op = validate_op1(op_bytes)?;
    let profile = op[8][0];
    let semantics = env[11][0];
    let applicable = match profile {
        1 => source.is_none() && actor.is_none() && semantics == 0 && count == 0,
        2 => source.is_some() && actor.is_some() && semantics == 0 && count == 0,
        3 => semantics == 1 && count > 0,
        4 => (semantics == 2 || semantics == 3) && count > 0,
        _ => false,
    };
    if !applicable {
        return Err(Error::ProfileMismatch);
    }
    Ok(env)
}

pub fn validate_record(bytes: &[u8], magic: &[u8], count: u16) -> Result<(), Error> {
    let header = magic.len() + 2;
    if bytes.len() < header
        || &bytes[..magic.len()] != magic
        || u16::from_be_bytes([bytes[magic.len()], bytes[magic.len() + 1]]) != count
    {
        return Err(Error::Encoding);
    }
    let mut p = header;
    let mut last = 0;
    for _ in 0..count {
        if p + 5 > bytes.len() {
            return Err(Error::Encoding);
        }
        let tag = bytes[p];
        if tag <= last {
            return Err(Error::Encoding);
        }
        last = tag;
        let n = u32::from_be_bytes(
            bytes[p + 1..p + 5]
                .try_into()
                .map_err(|_| Error::Encoding)?,
        ) as usize;
        p = p.checked_add(5 + n).ok_or(Error::Encoding)?;
        if p > bytes.len() {
            return Err(Error::Encoding);
        }
    }
    if p != bytes.len() {
        return Err(Error::Encoding);
    }
    Ok(())
}

fn fields<'a>(bytes: &'a [u8], magic: &[u8], count: u16) -> Result<Vec<&'a [u8]>, Error> {
    validate_record(bytes, magic, count)?;
    let mut p = magic.len() + 2;
    let mut out = Vec::with_capacity(count as usize);
    for expected in 1..=count {
        if bytes[p] != expected as u8 {
            return Err(Error::Encoding);
        }
        let n = u32::from_be_bytes(
            bytes[p + 1..p + 5]
                .try_into()
                .map_err(|_| Error::Encoding)?,
        ) as usize;
        p += 5;
        out.push(&bytes[p..p + n]);
        p += n;
    }
    Ok(out)
}

/// Parses and semantically validates the complete frozen SOP1 profile.
pub fn validate_semantic_operation(bytes: &[u8]) -> Result<(), Error> {
    let f = fields(bytes, b"RDOS-SOP1", 13)?;
    let op = validate_op1(f[0])?;
    if f[1] != b"EXP-0001-SHA256-CTR-v1"
        && f[1] != b"EXP-0001-SHA256-MOTIF-v1"
        && f[1] != b"EXP-0001-ZERO-v1"
    {
        return Err(Error::ProfileMismatch);
    }
    if f[3] != b"EXP-0001-UUID4-SHA256-v1"
        || f[7] != b"EXP-0001-ENVELOPE-INPUT-v1"
        || f[9] != b"EXP-0001-PRIOR-EVENTS-v1"
        || f[10] != b"EXP-0001-LOGICAL-TIME-v1"
    {
        return Err(Error::ProfileMismatch);
    }
    if f[4].len() != 16
        || f[5].len() != 16
        || f[6].len() != 16
        || f[11].len() != 8
        || f[12].len() != 8
    {
        return Err(Error::Encoding);
    }
    let size = *op[5].first().ok_or(Error::Encoding)? as usize;
    if op[5].len() != 1
        || op[6].len() != 4
        || usize::try_from(u32::from_be_bytes(
            op[6].try_into().map_err(|_| Error::Encoding)?,
        ))
        .ok()
            != Some(f[2].len())
        || [0, 32, 256, 4096, 65536, 1048576].get(size).copied() != Some(f[2].len())
    {
        return Err(Error::ProfileMismatch);
    }
    let content = op[7].first().copied().ok_or(Error::Encoding)?;
    let expected = match content {
        1 => b"EXP-0001-SHA256-CTR-v1".as_slice(),
        2 => b"EXP-0001-SHA256-MOTIF-v1".as_slice(),
        3 => b"EXP-0001-ZERO-v1".as_slice(),
        _ => return Err(Error::Encoding),
    };
    if f[1] != expected {
        return Err(Error::ProfileMismatch);
    }
    let env = validate_env1(f[8], f[0])?;
    if env[0] != f[0] || env[7] != f[4] || env[8] != f[5] || env[9] != f[6] {
        return Err(Error::ProfileMismatch);
    }
    if env[10].len() != 8 {
        return Err(Error::Encoding);
    }
    let unit = i64::from_be_bytes(f[12].try_into().map_err(|_| Error::Encoding)?);
    if unit <= 0 {
        return Err(Error::LogicalTimeParameter);
    }
    let mut input = OperationInput {
        segment: if op[2][0] == 0 {
            Segment::WarmUp
        } else {
            Segment::Measured
        },
        seed: u64::from_be_bytes(op[3].try_into().map_err(|_| Error::Encoding)?),
        ordinal: u64::from_be_bytes(op[4].try_into().map_err(|_| Error::Encoding)?),
        size_class: op[5][0],
        content: match op[7][0] {
            1 => Content::High,
            2 => Content::Low,
            _ => Content::Zero,
        },
        envelope: match op[8][0] {
            1 => Envelope::Minimal,
            2 => Envelope::Provenance,
            3 => Envelope::Causal,
            _ => Envelope::CorrectionRetraction,
        },
        temporal: match op[9][0] {
            1 => Temporal::Monotonic,
            2 => Temporal::EqualBurst,
            3 => Temporal::Late,
            _ => Temporal::OutOfOrder,
        },
        stream_namespace: op[10].try_into().map_err(|_| Error::Encoding)?,
        producer_id: op[11].try_into().map_err(|_| Error::Encoding)?,
        producer_ordinal: u64::from_be_bytes(op[12].try_into().map_err(|_| Error::Encoding)?),
        controlled_schedule: None,
    };
    if op[13][0] == 1 {
        input.controlled_schedule = Some(op[13][1..].try_into().map_err(|_| Error::Encoding)?);
    }
    if f[2] != payload(&input)?
        || f[4] != identity(&input, IdentityKind::Request)?
        || f[5] != identity(&input, IdentityKind::Event)?
        || f[6] != identity(&input, IdentityKind::Information)?
    {
        return Err(Error::ProfileMismatch);
    }
    let base = i64::from_be_bytes(f[11].try_into().map_err(|_| Error::Encoding)?);
    if env[10] != logical_time(input.temporal, input.ordinal, base, unit)?.to_be_bytes() {
        return Err(Error::ProfileMismatch);
    }
    Ok(())
}

pub fn workload_stream(
    operations: &[Vec<u8>],
    warm_up: u64,
    measured: u64,
) -> Result<Vec<u8>, Error> {
    let total = warm_up.checked_add(measured).ok_or(Error::CountMismatch)?;
    if usize::try_from(total).ok() != Some(operations.len()) {
        return Err(Error::CountMismatch);
    }
    let mut v = b"RDOS-WS1EXP-0001-SEMANTIC-OP-v1".to_vec();
    v.extend(total.to_be_bytes());
    v.extend(warm_up.to_be_bytes());
    v.extend(measured.to_be_bytes());
    for op in operations {
        validate_semantic_operation(op)?;
        v.extend(
            u64::try_from(op.len())
                .map_err(|_| Error::ResourceLimit)?
                .to_be_bytes(),
        );
        v.extend(op)
    }
    validate_stream(&v)?;
    Ok(v)
}

#[derive(Default)]
pub(crate) struct StreamBindings {
    pub envelope: std::collections::BTreeMap<&'static str, u64>,
    pub temporal: std::collections::BTreeMap<&'static str, u64>,
    pub size: std::collections::BTreeMap<String, u64>,
    pub schedule: Option<Option<[u8; 16]>>,
}

pub(crate) fn stream_bindings(bytes: &[u8]) -> Result<StreamBindings, Error> {
    validate_stream(bytes)?;
    let mut p = 55;
    let mut result = StreamBindings::default();
    while p < bytes.len() {
        let z = usize::try_from(u64::from_be_bytes(
            bytes[p..p + 8].try_into().map_err(|_| Error::Encoding)?,
        ))
        .map_err(|_| Error::ResourceLimit)?;
        p += 8;
        let sop = fields(&bytes[p..p + z], b"RDOS-SOP1", 13)?;
        let op = validate_op1(sop[0])?;
        p += z;
        let envelope = [
            "",
            "envelope-minimal",
            "envelope-provenance",
            "envelope-causal-reference",
            "envelope-correction-retraction-reference",
        ][op[8][0] as usize];
        let temporal = [
            "",
            "time-monotonic-effective",
            "time-equal-burst-v1",
            "time-late-arriving-v1",
            "time-out-of-effective-order-v1",
        ][op[9][0] as usize];
        *result.envelope.entry(envelope).or_default() += 1;
        *result.temporal.entry(temporal).or_default() += 1;
        *result.size.entry(format!("P{}", op[5][0])).or_default() += 1;
        let schedule = if op[13][0] == 0 {
            None
        } else {
            Some(op[13][1..].try_into().map_err(|_| Error::Encoding)?)
        };
        match result.schedule {
            None => result.schedule = Some(schedule),
            Some(x) if x == schedule => {}
            _ => return Err(Error::ProfileMismatch),
        }
    }
    Ok(result)
}
pub fn validate_stream(bytes: &[u8]) -> Result<(u64, u64, u64), Error> {
    if bytes.len() < 55 || &bytes[..31] != b"RDOS-WS1EXP-0001-SEMANTIC-OP-v1" {
        return Err(Error::Encoding);
    }
    let u = |p| u64::from_be_bytes(bytes[p..p + 8].try_into().unwrap());
    let (n, w, m) = (u(31), u(39), u(47));
    if w.checked_add(m) != Some(n) {
        return Err(Error::CountMismatch);
    }
    let mut p = 55;
    for index in 0..n {
        if p + 8 > bytes.len() {
            return Err(Error::Encoding);
        }
        let z = usize::try_from(u64::from_be_bytes(bytes[p..p + 8].try_into().unwrap()))
            .map_err(|_| Error::ResourceLimit)?;
        p = p.checked_add(8 + z).ok_or(Error::Encoding)?;
        if p > bytes.len() {
            return Err(Error::Encoding);
        }
        let sop = &bytes[p - z..p];
        validate_semantic_operation(sop)?;
        let sf = fields(sop, b"RDOS-SOP1", 13)?;
        let op = validate_op1(sf[0])?;
        let expected_segment = if index < w { 0 } else { 1 };
        let expected_ordinal = if index < w { index } else { index - w };
        if op[2][0] != expected_segment
            || u64::from_be_bytes(op[4].try_into().map_err(|_| Error::Encoding)?)
                != expected_ordinal
        {
            return Err(Error::Ordering);
        }
    }
    if p != bytes.len() {
        return Err(Error::Encoding);
    }
    Ok((n, w, m))
}
pub fn workload_digest(bytes: &[u8]) -> [u8; 32] {
    domain_digest(STREAM_DOMAIN, bytes)
}
pub fn manifest_digest(bytes: &[u8]) -> [u8; 32] {
    domain_digest(MANIFEST_DOMAIN, bytes)
}
pub fn artifact_digest(bytes: &[u8]) -> [u8; 32] {
    domain_digest("rusty-data-os/exp1/r7/artifact/v1", bytes)
}
fn domain_digest(domain: &str, bytes: &[u8]) -> [u8; 32] {
    let mut v = domain.as_bytes().to_vec();
    v.push(0);
    v.extend(bytes);
    sha256(&v)
}

mod manifest;
pub use manifest::{
    ArtifactMetadata, ArtifactReference, AuthorityRevision, DigestValue, Distribution,
    GeneratorInputs, InputState, Manifest, ManifestCounts, ManifestDigestDescriptor,
    ManifestProfiles, ManifestReference, ProvenanceEdge, RevisionKind, StreamReference,
    Supersession, SupersessionTarget, TypedManifest, ValidationContext, validate_manifest,
};

pub fn validate_supersession(
    id: &str,
    workload: &str,
    targets: &[(String, String)],
    reason: Option<&str>,
) -> Result<(), Error> {
    if targets.is_empty() {
        return if reason.is_none() {
            Ok(())
        } else {
            Err(Error::ImmutableState)
        };
    }
    if reason.is_none() {
        return Err(Error::ImmutableState);
    }
    let mut seen = BTreeSet::new();
    let mut previous = "";
    for (target, w) in targets {
        if target == id || w != workload {
            return Err(Error::ImmutableState);
        }
        if target.as_str() <= previous || !seen.insert(target) {
            return Err(Error::SupersessionCycle);
        }
        previous = target;
    }
    Ok(())
}

pub fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 15) as usize] as char)
    }
    s
}
pub fn decode_hex(s: &str) -> Result<Vec<u8>, Error> {
    if s.len() % 2 != 0 {
        return Err(Error::Type);
    }
    s.as_bytes()
        .chunks(2)
        .map(|x| {
            let n = |b| match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err(Error::Type),
            };
            Ok(n(x[0])? * 16 + n(x[1])?)
        })
        .collect()
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut m = data.to_vec();
    let bits = (m.len() as u64).wrapping_mul(8);
    m.push(0x80);
    while m.len() % 64 != 56 {
        m.push(0)
    }
    m.extend(bits.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for c in m.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, x) in c.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(x.try_into().unwrap())
        }
        for i in 16..64 {
            let a = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let b = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(a)
                .wrapping_add(w[i - 7])
                .wrapping_add(b)
        }
        let (mut a, mut b, mut c0, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c0) ^ (b & c0);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c0;
            c0 = b;
            b = a;
            a = t1.wrapping_add(t2)
        }
        for (i, x) in [a, b, c0, d, e, f, g, hh].iter().enumerate() {
            h[i] = h[i].wrapping_add(*x)
        }
    }
    let mut out = [0; 32];
    for (i, x) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes())
    }
    out
}
