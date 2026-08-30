//! Experimental `EXP1-B1-RF1` physical-record codec and local validator.
//! This API has no stability promise and performs no persistence or I/O.

#![forbid(unsafe_code)]

pub const HEADER_LEN: usize = 32;
pub const MAX_RECORD_LEN: usize = 16_777_216;
pub const MAX_RECORDS: usize = 1_000_000;
pub const MAX_SCAN_BYTES: u64 = 1_073_741_824;
pub const MAX_DIAGNOSTIC_BYTES: usize = 67_108_864;
const MAGIC: &[u8; 4] = b"RDE1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum IntegrityProfile {
    Structural = 0,
    Crc32c = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Uuid(pub [u8; 16]);

impl Uuid {
    pub fn validate_v4(self) -> Result<(), Error> {
        if self.0 == [0; 16] {
            return Err(Error::NilUuid);
        }
        if self.0[6] >> 4 != 4 {
            return Err(Error::UuidVersion);
        }
        if self.0[8] >> 6 != 2 {
            return Err(Error::UuidVariant);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Body {
    Binding {
        request_id: Uuid,
        event_id: Uuid,
        normalized_request: Vec<u8>,
    },
    Reservation {
        request_id: Uuid,
        event_id: Uuid,
        sequence: u64,
        high_water: u64,
    },
    Provisional {
        event_id: Uuid,
        sequence: u64,
        group_id: u64,
        member_index: u16,
        member_count: u16,
        stable_core: Vec<u8>,
    },
    Membership {
        group_id: u64,
        members: Vec<(Uuid, u64)>,
    },
    Final {
        event_id: Uuid,
        request_id: Uuid,
        sequence: u64,
        durability_time: i64,
        complete_envelope: Vec<u8>,
    },
    Commit {
        event_id: Uuid,
        sequence: u64,
        final_ordinal: u64,
        final_crc32c: u32,
        group_id: u64,
        member_index: u16,
        member_count: u16,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub physical_ordinal: u64,
    pub integrity: IntegrityProfile,
    pub body: Body,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    TruncatedHeader,
    TruncatedRecord,
    TrailingBytes,
    BadMagic,
    UnsupportedVersion,
    UnknownKind,
    UnsupportedIntegrity,
    InvalidProfileForKind,
    NonzeroReserved,
    NonzeroStructuralIntegrity,
    InvalidLength,
    LengthOverflow,
    Oversize,
    CrcMismatch,
    NilUuid,
    UuidVersion,
    UuidVariant,
    ZeroOrdinal,
    ZeroSequence,
    InvalidHighWater,
    InvalidGroup,
    InvalidMember,
    DuplicateIdentity,
    DuplicateSequence,
    OrdinalOrder,
    SequenceOrder,
    MissingBinding,
    MissingReservation,
    FinalNotAdjacent,
    FinalIdentityMismatch,
    FinalCrcMismatch,
    DuplicateFinal,
    DuplicateCommit,
    RecordLimit,
    ScanByteLimit,
    DiagnosticLimit,
    InteriorDamage,
}

/// Deterministic resource bounds for one artifact scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanLimits {
    pub max_record_len: usize,
    pub max_records: usize,
    pub max_scan_bytes: u64,
    pub max_diagnostic_bytes: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_record_len: MAX_RECORD_LEN,
            max_records: MAX_RECORDS,
            max_scan_bytes: MAX_SCAN_BYTES,
            max_diagnostic_bytes: MAX_DIAGNOSTIC_BYTES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanTermination {
    CleanEof,
    TerminalTruncation { offset: u64 },
    Failure { offset: u64, error: Error },
}

/// Artifact-level result. `records` contains only the fully validated prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanOutcome {
    pub records: Vec<Record>,
    pub scanned_bytes: u64,
    pub termination: ScanTermination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorDisposition {
    Valid,
    TerminalTruncation,
    MalformedLength,
    Unsupported,
    CoveredCorruption,
    DuplicateOrOrderConflict,
    AmbiguousOrInteriorDamage,
}

/// Executable coverage registry for the authoritative R5 V1--V10 vectors.
pub const R5_VECTOR_DISPOSITIONS: [(&str, VectorDisposition); 10] = [
    ("V1", VectorDisposition::Valid),
    ("V2", VectorDisposition::Valid),
    ("V3", VectorDisposition::Valid),
    ("V4", VectorDisposition::Valid),
    ("V5", VectorDisposition::TerminalTruncation),
    ("V6", VectorDisposition::MalformedLength),
    ("V7", VectorDisposition::Unsupported),
    ("V8", VectorDisposition::CoveredCorruption),
    ("V9", VectorDisposition::DuplicateOrOrderConflict),
    ("V10", VectorDisposition::AmbiguousOrInteriorDamage),
];

pub fn scan(bytes: &[u8]) -> ScanOutcome {
    scan_with_limits(bytes, ScanLimits::default())
}

/// Performs the R1/R5 required checked offset/length arithmetic.
pub fn checked_extent(offset: u64, total_length: u64) -> Result<u64, Error> {
    offset
        .checked_add(total_length)
        .ok_or(Error::LengthOverflow)
}

/// Scans concatenated records without seeking a new record boundary after failure.
pub fn scan_with_limits(bytes: &[u8], limits: ScanLimits) -> ScanOutcome {
    let mut records = Vec::new();
    let mut offset = 0usize;
    let mut retained = 0usize;
    loop {
        if offset == bytes.len() {
            return scan_outcome(records, offset, ScanTermination::CleanEof);
        }
        if records.len() >= limits.max_records {
            return scan_failure(records, offset, Error::RecordLimit);
        }
        let remaining = &bytes[offset..];
        if remaining.len() < HEADER_LEN {
            return scan_outcome(
                records,
                offset,
                ScanTermination::TerminalTruncation {
                    offset: offset as u64,
                },
            );
        }

        // Header identity is checked before either declared length is trusted.
        if &remaining[..4] != MAGIC {
            return scan_failure(records, offset, Error::BadMagic);
        }
        if le_u16(remaining, 4) != 1 {
            return scan_failure(records, offset, Error::UnsupportedVersion);
        }
        if !(1..=6).contains(&remaining[6]) {
            return scan_failure(records, offset, Error::UnknownKind);
        }
        if remaining[7] > 1 {
            return scan_failure(records, offset, Error::UnsupportedIntegrity);
        }
        let total = le_u32(remaining, 8) as usize;
        let body = le_u32(remaining, 12) as usize;
        if total < HEADER_LEN {
            return scan_failure(records, offset, Error::InvalidLength);
        }
        if total > MAX_RECORD_LEN || total > limits.max_record_len {
            return scan_failure(records, offset, Error::Oversize);
        }
        if body.checked_add(HEADER_LEN) != Some(total) {
            return scan_failure(records, offset, Error::InvalidLength);
        }
        let Ok(offset64) = u64::try_from(offset) else {
            return scan_failure(records, offset, Error::LengthOverflow);
        };
        let Ok(total64) = u64::try_from(total) else {
            return scan_failure(records, offset, Error::LengthOverflow);
        };
        let Ok(end64) = checked_extent(offset64, total64) else {
            return scan_failure(records, offset, Error::LengthOverflow);
        };
        let Ok(end) = usize::try_from(end64) else {
            return scan_failure(records, offset, Error::LengthOverflow);
        };
        if end64 > limits.max_scan_bytes {
            return scan_failure(records, offset, Error::ScanByteLimit);
        }
        if end > bytes.len() {
            let later_magic = remaining[HEADER_LEN..]
                .windows(MAGIC.len())
                .any(|window| window == MAGIC);
            let termination = if later_magic {
                ScanTermination::Failure {
                    offset: offset as u64,
                    error: Error::InteriorDamage,
                }
            } else {
                ScanTermination::TerminalTruncation {
                    offset: offset as u64,
                }
            };
            return scan_outcome(records, offset, termination);
        }
        let record = match decode(&bytes[offset..end]) {
            Ok(record) => record,
            Err(error) => return scan_failure(records, offset, error),
        };
        let Some(next_retained) = retained.checked_add(total) else {
            return scan_failure(records, offset, Error::LengthOverflow);
        };
        if next_retained > limits.max_diagnostic_bytes {
            return scan_failure(records, offset, Error::DiagnosticLimit);
        }
        records.push(record);
        if let Err(error) = validate_lifecycle(&records) {
            records.pop();
            return scan_failure(records, offset, error);
        }
        retained = next_retained;
        offset = end;
    }
}

fn scan_failure(records: Vec<Record>, offset: usize, error: Error) -> ScanOutcome {
    scan_outcome(
        records,
        offset,
        ScanTermination::Failure {
            offset: offset as u64,
            error,
        },
    )
}

fn scan_outcome(records: Vec<Record>, offset: usize, termination: ScanTermination) -> ScanOutcome {
    ScanOutcome {
        records,
        scanned_bytes: offset as u64,
        termination,
    }
}

pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    crc ^ 0xffff_ffff
}

pub fn encode(record: &Record) -> Result<Vec<u8>, Error> {
    validate_record(record)?;
    let (kind, body) = encode_body(&record.body)?;
    let total = HEADER_LEN
        .checked_add(body.len())
        .ok_or(Error::LengthOverflow)?;
    if total > MAX_RECORD_LEN || total > u32::MAX as usize {
        return Err(Error::Oversize);
    }
    let body_len = u32::try_from(body.len()).map_err(|_| Error::LengthOverflow)?;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.push(kind);
    out.push(record.integrity as u8);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&body_len.to_le_bytes());
    out.extend_from_slice(&record.physical_ordinal.to_le_bytes());
    out.extend_from_slice(&0_u32.to_le_bytes());
    out.extend_from_slice(&0_u32.to_le_bytes());
    out.extend_from_slice(&body);
    if record.integrity == IntegrityProfile::Crc32c {
        let crc = crc32c(&out);
        out[28..32].copy_from_slice(&crc.to_le_bytes());
    }
    Ok(out)
}

pub fn decode(bytes: &[u8]) -> Result<Record, Error> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::TruncatedHeader);
    }
    if &bytes[0..4] != MAGIC {
        return Err(Error::BadMagic);
    }
    if le_u16(bytes, 4) != 1 {
        return Err(Error::UnsupportedVersion);
    }
    let kind = bytes[6];
    if !(1..=6).contains(&kind) {
        return Err(Error::UnknownKind);
    }
    let integrity = match bytes[7] {
        0 => IntegrityProfile::Structural,
        1 => IntegrityProfile::Crc32c,
        _ => return Err(Error::UnsupportedIntegrity),
    };
    let total = le_u32(bytes, 8) as usize;
    let body_len = le_u32(bytes, 12) as usize;
    if !(HEADER_LEN..=MAX_RECORD_LEN).contains(&total) {
        return Err(if total > MAX_RECORD_LEN {
            Error::Oversize
        } else {
            Error::InvalidLength
        });
    }
    if body_len
        .checked_add(HEADER_LEN)
        .ok_or(Error::LengthOverflow)?
        != total
    {
        return Err(Error::InvalidLength);
    }
    if total > bytes.len() {
        return Err(Error::TruncatedRecord);
    }
    if total != bytes.len() {
        return Err(Error::TrailingBytes);
    }
    if le_u32(bytes, 24) != 0 {
        return Err(Error::NonzeroReserved);
    }
    let stored_crc = le_u32(bytes, 28);
    if integrity == IntegrityProfile::Structural {
        if stored_crc != 0 {
            return Err(Error::NonzeroStructuralIntegrity);
        }
    } else {
        let mut covered = bytes.to_vec();
        covered[28..32].fill(0);
        if crc32c(&covered) != stored_crc {
            return Err(Error::CrcMismatch);
        }
    }
    if (matches!(kind, 2 | 5 | 6) && integrity != IntegrityProfile::Crc32c)
        || (matches!(kind, 4) && integrity != IntegrityProfile::Structural)
    {
        return Err(Error::InvalidProfileForKind);
    }
    let record = Record {
        physical_ordinal: le_u64(bytes, 16),
        integrity,
        body: decode_body(kind, &bytes[32..])?,
    };
    validate_record(&record)?;
    Ok(record)
}

fn validate_record(record: &Record) -> Result<(), Error> {
    if record.physical_ordinal == 0 {
        return Err(Error::ZeroOrdinal);
    }
    let must_crc = matches!(
        record.body,
        Body::Reservation { .. } | Body::Final { .. } | Body::Commit { .. }
    );
    let structural_only = matches!(record.body, Body::Membership { .. });
    if (must_crc && record.integrity != IntegrityProfile::Crc32c)
        || (structural_only && record.integrity != IntegrityProfile::Structural)
    {
        return Err(Error::InvalidProfileForKind);
    }
    match &record.body {
        Body::Binding {
            request_id,
            event_id,
            ..
        } => {
            request_id.validate_v4()?;
            event_id.validate_v4()?;
        }
        Body::Reservation {
            request_id,
            event_id,
            sequence,
            high_water,
        } => {
            request_id.validate_v4()?;
            event_id.validate_v4()?;
            if *sequence == 0 {
                return Err(Error::ZeroSequence);
            }
            if *high_water == 0 || sequence > high_water {
                return Err(Error::InvalidHighWater);
            }
        }
        Body::Provisional {
            event_id,
            sequence,
            group_id,
            member_index,
            member_count,
            ..
        } => {
            event_id.validate_v4()?;
            if *sequence == 0 {
                return Err(Error::ZeroSequence);
            }
            if (*group_id == 0 && (*member_index != 0 || *member_count != 1))
                || (*group_id != 0 && (*member_count == 0 || *member_index >= *member_count))
            {
                return Err(Error::InvalidGroup);
            }
        }
        Body::Membership { group_id, members } => {
            if *group_id == 0 || members.is_empty() || members.len() > u16::MAX as usize {
                return Err(Error::InvalidGroup);
            }
            let mut previous = 0;
            for (index, (event, sequence)) in members.iter().enumerate() {
                event.validate_v4()?;
                if *sequence == 0 {
                    return Err(Error::ZeroSequence);
                }
                if index != 0 && *sequence <= previous {
                    return Err(Error::SequenceOrder);
                }
                if members[..index].iter().any(|(id, _)| id == event) {
                    return Err(Error::DuplicateIdentity);
                }
                previous = *sequence;
            }
        }
        Body::Final {
            event_id,
            request_id,
            sequence,
            ..
        } => {
            event_id.validate_v4()?;
            request_id.validate_v4()?;
            if *sequence == 0 {
                return Err(Error::ZeroSequence);
            }
        }
        Body::Commit {
            event_id,
            sequence,
            group_id,
            member_index,
            member_count,
            ..
        } => {
            event_id.validate_v4()?;
            if *sequence == 0 {
                return Err(Error::ZeroSequence);
            }
            if (*group_id == 0 && (*member_index != 0 || *member_count != 1))
                || (*group_id != 0 && (*member_count == 0 || *member_index >= *member_count))
            {
                return Err(Error::InvalidGroup);
            }
        }
    }
    Ok(())
}

fn encode_body(body: &Body) -> Result<(u8, Vec<u8>), Error> {
    let mut out = Vec::new();
    let kind = match body {
        Body::Binding {
            request_id,
            event_id,
            normalized_request,
        } => {
            out.extend(request_id.0);
            out.extend(event_id.0);
            put_len(&mut out, normalized_request)?;
            out.extend(normalized_request);
            1
        }
        Body::Reservation {
            request_id,
            event_id,
            sequence,
            high_water,
        } => {
            out.extend(request_id.0);
            out.extend(event_id.0);
            out.extend(sequence.to_le_bytes());
            out.extend(high_water.to_le_bytes());
            2
        }
        Body::Provisional {
            event_id,
            sequence,
            group_id,
            member_index,
            member_count,
            stable_core,
        } => {
            out.extend(event_id.0);
            out.extend(sequence.to_le_bytes());
            out.extend(group_id.to_le_bytes());
            out.extend(member_index.to_le_bytes());
            out.extend(member_count.to_le_bytes());
            put_len(&mut out, stable_core)?;
            out.extend(stable_core);
            3
        }
        Body::Membership { group_id, members } => {
            out.extend(group_id.to_le_bytes());
            out.extend((members.len() as u16).to_le_bytes());
            out.extend(0_u16.to_le_bytes());
            for (id, sequence) in members {
                out.extend(id.0);
                out.extend(sequence.to_le_bytes());
            }
            4
        }
        Body::Final {
            event_id,
            request_id,
            sequence,
            durability_time,
            complete_envelope,
        } => {
            out.extend(event_id.0);
            out.extend(request_id.0);
            out.extend(sequence.to_le_bytes());
            out.extend(durability_time.to_le_bytes());
            put_len(&mut out, complete_envelope)?;
            out.extend(complete_envelope);
            5
        }
        Body::Commit {
            event_id,
            sequence,
            final_ordinal,
            final_crc32c,
            group_id,
            member_index,
            member_count,
        } => {
            out.extend(event_id.0);
            out.extend(sequence.to_le_bytes());
            out.extend(final_ordinal.to_le_bytes());
            out.extend(final_crc32c.to_le_bytes());
            out.extend(group_id.to_le_bytes());
            out.extend(member_index.to_le_bytes());
            out.extend(member_count.to_le_bytes());
            6
        }
    };
    Ok((kind, out))
}

fn decode_body(kind: u8, b: &[u8]) -> Result<Body, Error> {
    Ok(match kind {
        1 => {
            exact_min(b, 36)?;
            let n = le_u32(b, 32) as usize;
            exact_variable(b, 36, n)?;
            Body::Binding {
                request_id: uuid(b, 0),
                event_id: uuid(b, 16),
                normalized_request: b[36..].to_vec(),
            }
        }
        2 => {
            exact(b, 48)?;
            Body::Reservation {
                request_id: uuid(b, 0),
                event_id: uuid(b, 16),
                sequence: le_u64(b, 32),
                high_water: le_u64(b, 40),
            }
        }
        3 => {
            exact_min(b, 40)?;
            let n = le_u32(b, 36) as usize;
            exact_variable(b, 40, n)?;
            Body::Provisional {
                event_id: uuid(b, 0),
                sequence: le_u64(b, 16),
                group_id: le_u64(b, 24),
                member_index: le_u16(b, 32),
                member_count: le_u16(b, 34),
                stable_core: b[40..].to_vec(),
            }
        }
        4 => {
            exact_min(b, 12)?;
            if le_u16(b, 10) != 0 {
                return Err(Error::NonzeroReserved);
            }
            let count = le_u16(b, 8) as usize;
            let n = count.checked_mul(24).ok_or(Error::LengthOverflow)?;
            exact_variable(b, 12, n)?;
            let mut members = Vec::with_capacity(count);
            for i in 0..count {
                let p = 12 + i * 24;
                members.push((uuid(b, p), le_u64(b, p + 16)));
            }
            Body::Membership {
                group_id: le_u64(b, 0),
                members,
            }
        }
        5 => {
            exact_min(b, 52)?;
            let n = le_u32(b, 48) as usize;
            exact_variable(b, 52, n)?;
            Body::Final {
                event_id: uuid(b, 0),
                request_id: uuid(b, 16),
                sequence: le_u64(b, 32),
                durability_time: le_i64(b, 40),
                complete_envelope: b[52..].to_vec(),
            }
        }
        6 => {
            exact(b, 48)?;
            Body::Commit {
                event_id: uuid(b, 0),
                sequence: le_u64(b, 16),
                final_ordinal: le_u64(b, 24),
                final_crc32c: le_u32(b, 32),
                group_id: le_u64(b, 36),
                member_index: le_u16(b, 44),
                member_count: le_u16(b, 46),
            }
        }
        _ => return Err(Error::UnknownKind),
    })
}

/// Validates only relationships decidable from this complete supplied record slice.
pub fn validate_lifecycle(records: &[Record]) -> Result<(), Error> {
    let mut previous_ordinal = 0_u64;
    let mut previous_commit_sequence = 0_u64;
    let mut bindings = Vec::new();
    let mut reservations = Vec::new();
    let mut finals = Vec::new();
    let mut commits = Vec::new();
    for (i, record) in records.iter().enumerate() {
        validate_record(record)?;
        if i != 0
            && record.physical_ordinal
                != previous_ordinal
                    .checked_add(1)
                    .ok_or(Error::LengthOverflow)?
        {
            return Err(Error::OrdinalOrder);
        }
        previous_ordinal = record.physical_ordinal;
        match &record.body {
            Body::Binding {
                request_id,
                event_id,
                normalized_request,
            } => {
                if bindings
                    .iter()
                    .any(|(request, event, _): &(Uuid, Uuid, Vec<u8>)| {
                        request == request_id || event == event_id
                    })
                {
                    return Err(Error::DuplicateIdentity);
                }
                bindings.push((*request_id, *event_id, normalized_request.clone()));
            }
            Body::Reservation {
                request_id,
                event_id,
                sequence,
                ..
            } => {
                if !bindings
                    .iter()
                    .any(|(request, event, _)| request == request_id && event == event_id)
                {
                    return Err(Error::MissingBinding);
                }
                if reservations
                    .iter()
                    .any(|(_, _, used): &(Uuid, Uuid, u64)| used == sequence)
                {
                    return Err(Error::DuplicateSequence);
                }
                reservations.push((*request_id, *event_id, *sequence));
            }
            Body::Final {
                event_id,
                request_id,
                sequence,
                ..
            } => {
                if !bindings
                    .iter()
                    .any(|(request, event, _)| request == request_id && event == event_id)
                {
                    return Err(Error::MissingBinding);
                }
                if !reservations.iter().any(|(request, event, reserved)| {
                    request == request_id && event == event_id && reserved == sequence
                }) {
                    return Err(Error::MissingReservation);
                }
                if finals.contains(event_id) {
                    return Err(Error::DuplicateFinal);
                }
                finals.push(*event_id);
            }
            _ => {}
        }
        if let Body::Commit {
            event_id,
            sequence,
            final_ordinal,
            final_crc32c,
            ..
        } = &record.body
        {
            let Some(final_record) = i.checked_sub(1).and_then(|p| records.get(p)) else {
                return Err(Error::FinalNotAdjacent);
            };
            let Body::Final {
                event_id: final_id,
                sequence: final_sequence,
                ..
            } = &final_record.body
            else {
                return Err(Error::FinalNotAdjacent);
            };
            if final_record.physical_ordinal != *final_ordinal {
                return Err(Error::FinalNotAdjacent);
            }
            if final_id != event_id || final_sequence != sequence {
                return Err(Error::FinalIdentityMismatch);
            }
            if commits.contains(event_id) {
                return Err(Error::DuplicateCommit);
            }
            let encoded = encode(final_record)?;
            if le_u32(&encoded, 28) != *final_crc32c {
                return Err(Error::FinalCrcMismatch);
            }
            if *sequence <= previous_commit_sequence {
                return Err(Error::SequenceOrder);
            }
            previous_commit_sequence = *sequence;
            commits.push(*event_id);
        }
    }
    Ok(())
}

fn put_len(out: &mut Vec<u8>, value: &[u8]) -> Result<(), Error> {
    out.extend(
        u32::try_from(value.len())
            .map_err(|_| Error::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}
fn exact(b: &[u8], n: usize) -> Result<(), Error> {
    if b.len() == n {
        Ok(())
    } else {
        Err(Error::InvalidLength)
    }
}
fn exact_min(b: &[u8], n: usize) -> Result<(), Error> {
    if b.len() >= n {
        Ok(())
    } else {
        Err(Error::InvalidLength)
    }
}
fn exact_variable(b: &[u8], fixed: usize, n: usize) -> Result<(), Error> {
    if fixed.checked_add(n).ok_or(Error::LengthOverflow)? == b.len() {
        Ok(())
    } else {
        Err(Error::InvalidLength)
    }
}
fn uuid(b: &[u8], p: usize) -> Uuid {
    let mut value = [0; 16];
    value.copy_from_slice(&b[p..p + 16]);
    Uuid(value)
}
fn le_u16(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([b[p], b[p + 1]])
}
fn le_u32(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]])
}
fn le_u64(b: &[u8], p: usize) -> u64 {
    u64::from_le_bytes(b[p..p + 8].try_into().expect("fixed checked slice"))
}
fn le_i64(b: &[u8], p: usize) -> i64 {
    i64::from_le_bytes(b[p..p + 8].try_into().expect("fixed checked slice"))
}
