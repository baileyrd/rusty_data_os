//! Pure R20 mapping from one validated SOP1 operation to one provisional RF1 frame.

use exp1_record_format::{
    Body, Error as FormatError, IntegrityProfile, Record, Uuid, decode, encode,
};
use exp1_workload_conformance::{Error as SemanticError, validate_semantic_operation};

/// The successfully consumed sequence and physical ordinal at the mapping boundary.
///
/// `initial()` represents an empty destination. Callers retain ownership and replace
/// their state only with [`MappedRecord::next_state`] after success.
/// Inconsistent state cannot be fabricated across the public boundary:
///
/// ```compile_fail
/// use exp1_raw_append_replay::mapping::MappingState;
/// let _ = MappingState { previous_sequence: 0, previous_physical_ordinal: 5 };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappingState {
    previous_sequence: u64,
    previous_physical_ordinal: u64,
}

impl MappingState {
    pub const fn initial() -> Self {
        Self {
            previous_sequence: 0,
            previous_physical_ordinal: 0,
        }
    }

    pub const fn previous_sequence(self) -> u64 {
        self.previous_sequence
    }

    pub const fn previous_physical_ordinal(self) -> u64 {
        self.previous_physical_ordinal
    }

    pub(crate) const fn from_validated(
        previous_sequence: u64,
        previous_physical_ordinal: u64,
    ) -> Self {
        Self {
            previous_sequence,
            previous_physical_ordinal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappedRecord {
    pub frame: Vec<u8>,
    pub record: Record,
    next_state: MappingState,
}

impl MappedRecord {
    /// Returns the only noninitial state that callers can supply to a later mapping.
    pub const fn next_state(&self) -> MappingState {
        self.next_state
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateError {
    ZeroSequence,
    ZeroPhysicalOrdinal,
    NonconsecutivePhysicalOrdinal,
    DuplicateSequence,
    DecreasingSequence,
    SequenceExhausted,
    PhysicalOrdinalExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtractionError {
    MalformedValidatedOperation,
    MalformedValidatedEnvelope,
    InvalidEventUuid(FormatError),
    EventIdentityMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingError {
    SemanticValidation(SemanticError),
    Extraction(ExtractionError),
    State(StateError),
    ResourceOrLength(FormatError),
    Encode(FormatError),
    Decode(FormatError),
    RoundTripMismatch,
}

// Extraction, encode/decode, and round-trip errors are defensive composition-boundary errors.
// With the currently frozen validators and limits, semantic validation bounds SOP1 below RF1's
// limit and establishes the fields extracted below, while RF1 encode followed immediately by RF1
// decode cannot ordinarily reach those variants. They remain explicit so a future change in either
// unchanged authority crate fails closed instead of becoming a panic or an unchecked assumption.

/// Maps complete SOP1 bytes without I/O or mutation of caller-owned state.
pub fn map_semantic_operation(
    semantic_operation: &[u8],
    assigned_sequence: u64,
    physical_ordinal: u64,
    state: MappingState,
) -> Result<MappedRecord, MappingError> {
    validate_semantic_operation(semantic_operation).map_err(MappingError::SemanticValidation)?;
    validate_state(assigned_sequence, physical_ordinal, state)?;
    let event_id = extract_event_id(semantic_operation)?;
    let record = Record {
        physical_ordinal,
        integrity: IntegrityProfile::Structural,
        body: Body::Provisional {
            event_id,
            sequence: assigned_sequence,
            group_id: 0,
            member_index: 0,
            member_count: 1,
            stable_core: semantic_operation.to_vec(),
        },
    };
    let frame = encode(&record).map_err(classify_encode)?;
    let decoded = decode(&frame).map_err(MappingError::Decode)?;
    if decoded != record {
        return Err(MappingError::RoundTripMismatch);
    }
    Ok(MappedRecord {
        frame,
        record,
        next_state: MappingState {
            previous_sequence: assigned_sequence,
            previous_physical_ordinal: physical_ordinal,
        },
    })
}

fn validate_state(sequence: u64, ordinal: u64, state: MappingState) -> Result<(), MappingError> {
    if sequence == 0 {
        return Err(MappingError::State(StateError::ZeroSequence));
    }
    if ordinal == 0 {
        return Err(MappingError::State(StateError::ZeroPhysicalOrdinal));
    }
    if state.previous_sequence == u64::MAX {
        return Err(MappingError::State(StateError::SequenceExhausted));
    }
    if state.previous_physical_ordinal == u64::MAX {
        return Err(MappingError::State(StateError::PhysicalOrdinalExhausted));
    }
    let expected = state
        .previous_physical_ordinal
        .checked_add(1)
        .ok_or(MappingError::State(StateError::PhysicalOrdinalExhausted))?;
    if ordinal != expected {
        return Err(MappingError::State(
            StateError::NonconsecutivePhysicalOrdinal,
        ));
    }
    if sequence == state.previous_sequence {
        return Err(MappingError::State(StateError::DuplicateSequence));
    }
    if sequence < state.previous_sequence {
        return Err(MappingError::State(StateError::DecreasingSequence));
    }
    Ok(())
}

fn classify_encode(error: FormatError) -> MappingError {
    if matches!(error, FormatError::LengthOverflow | FormatError::Oversize) {
        MappingError::ResourceOrLength(error)
    } else {
        MappingError::Encode(error)
    }
}

fn extract_event_id(bytes: &[u8]) -> Result<Uuid, MappingError> {
    let sop = validated_fields(bytes, b"RDOS-SOP1", 13).ok_or(MappingError::Extraction(
        ExtractionError::MalformedValidatedOperation,
    ))?;
    let outer: [u8; 16] = sop[5]
        .try_into()
        .map_err(|_| MappingError::Extraction(ExtractionError::MalformedValidatedOperation))?;
    let env = validated_fields(sop[8], b"RDOS-ENV1", 13).ok_or(MappingError::Extraction(
        ExtractionError::MalformedValidatedEnvelope,
    ))?;
    let inner: [u8; 16] = env[8]
        .try_into()
        .map_err(|_| MappingError::Extraction(ExtractionError::MalformedValidatedEnvelope))?;
    if outer != inner {
        return Err(MappingError::Extraction(
            ExtractionError::EventIdentityMismatch,
        ));
    }
    let id = Uuid(outer);
    id.validate_v4()
        .map_err(|error| MappingError::Extraction(ExtractionError::InvalidEventUuid(error)))?;
    Ok(id)
}

// This is deliberately only post-validation field extraction, not a semantic validator.
fn validated_fields<'a>(bytes: &'a [u8], magic: &[u8], count: u16) -> Option<Vec<&'a [u8]>> {
    if bytes.get(..magic.len())? != magic || bytes.len() < magic.len() + 2 {
        return None;
    }
    if u16::from_be_bytes(bytes[magic.len()..magic.len() + 2].try_into().ok()?) != count {
        return None;
    }
    let mut position = magic.len() + 2;
    let mut fields = Vec::with_capacity(count as usize);
    for tag in 1..=count {
        if *bytes.get(position)? != tag as u8 {
            return None;
        }
        let length =
            u32::from_be_bytes(bytes.get(position + 1..position + 5)?.try_into().ok()?) as usize;
        position = position.checked_add(5)?;
        let end = position.checked_add(length)?;
        fields.push(bytes.get(position..end)?);
        position = end;
    }
    (position == bytes.len()).then_some(fields)
}
