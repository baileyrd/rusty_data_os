//! Bounded exploratory history-to-row/column reconstruction experiment.
//!
//! The payload here is `EXPLORATORY-HV1`, not a frozen SOP1/SOP2 workload.

use exp1_raw_append_replay::{RawAppender, ReplayTermination, reopen_and_replay};
use exp1_record_format::{Body, IntegrityProfile, Record, ScanLimits, Uuid, encode};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const PAYLOAD_LABEL: &str = "EXPLORATORY-HV1/entity-time-value-v1";
const PAYLOAD_MAGIC: &[u8; 4] = b"EHV1";
pub const ENTITY_COUNT: u64 = 17;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub entity_id: u64,
    pub logical_time: i64,
    pub value: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Row {
    pub entity_id: u64,
    pub logical_time: i64,
    pub value: i64,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnView {
    pub entity_ids: Vec<u64>,
    pub logical_times: Vec<i64>,
    pub values: Vec<i64>,
    pub sequences: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct Trial {
    pub event_count: usize,
    pub payload_bytes: usize,
    pub physical_bytes: usize,
    pub encode: Duration,
    pub append: Duration,
    pub replay: Duration,
    pub row_rebuild: Duration,
    pub column_rebuild: Duration,
    pub direct_row: Duration,
    pub direct_column: Duration,
    pub correctness: bool,
}

pub fn generate(count: usize) -> Vec<Event> {
    (0..count)
        .map(|index| {
            let entity_id = (index as u64 * 7 + 3) % ENTITY_COUNT;
            Event {
                entity_id,
                // Deliberately differs from sequence order every eleventh event.
                logical_time: 1_800_000_000_000_000_000_i64 + index as i64 * 10
                    - if index % 11 == 0 { 25 } else { 0 },
                value: (index as i64 * 31) - (entity_id as i64 * 13),
            }
        })
        .collect()
}

fn payload(event: Event) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(28);
    bytes.extend_from_slice(PAYLOAD_MAGIC);
    bytes.extend_from_slice(&event.entity_id.to_le_bytes());
    bytes.extend_from_slice(&event.logical_time.to_le_bytes());
    bytes.extend_from_slice(&event.value.to_le_bytes());
    bytes
}

fn decode_payload(bytes: &[u8]) -> Result<Event, String> {
    if bytes.len() != 28 || &bytes[..4] != PAYLOAD_MAGIC {
        return Err("malformed EXPLORATORY-HV1 payload".into());
    }
    Ok(Event {
        entity_id: u64::from_le_bytes(bytes[4..12].try_into().map_err(|_| "entity")?),
        logical_time: i64::from_le_bytes(bytes[12..20].try_into().map_err(|_| "time")?),
        value: i64::from_le_bytes(bytes[20..28].try_into().map_err(|_| "value")?),
    })
}

fn event_uuid(sequence: u64) -> Uuid {
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&sequence.to_be_bytes());
    bytes[8..].copy_from_slice(&(sequence ^ 0xa5a5_5a5a_f0f0_0f0f).to_be_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid(bytes)
}

fn frame(event: Event, sequence: u64) -> Result<Vec<u8>, String> {
    encode(&Record {
        physical_ordinal: sequence,
        integrity: IntegrityProfile::Structural,
        body: Body::Provisional {
            event_id: event_uuid(sequence),
            sequence,
            group_id: 0,
            member_index: 0,
            member_count: 1,
            stable_core: payload(event),
        },
    })
    .map_err(|error| format!("encode failed: {error:?}"))
}

fn decoded_events(
    records: &[exp1_raw_append_replay::PhysicalRecord],
) -> Result<Vec<(u64, Event)>, String> {
    records
        .iter()
        .map(|physical| match &physical.record.body {
            Body::Provisional {
                sequence,
                stable_core,
                ..
            } => decode_payload(stable_core).map(|event| (*sequence, event)),
            _ => Err("unexpected RF1 record kind".into()),
        })
        .collect()
}

fn rows_from_records(
    records: &[exp1_raw_append_replay::PhysicalRecord],
    cutoff: u64,
) -> Result<Vec<Row>, String> {
    let events = decoded_events(records)?;
    Ok(rows_from_events(&events, cutoff))
}

fn columns_from_records(
    records: &[exp1_raw_append_replay::PhysicalRecord],
    cutoff: u64,
) -> Result<ColumnView, String> {
    let events = decoded_events(records)?;
    Ok(columns_from_events(&events, cutoff))
}

pub fn rows_from_events(events: &[(u64, Event)], cutoff: u64) -> Vec<Row> {
    let mut rows = BTreeMap::new();
    for &(sequence, event) in events.iter().filter(|(sequence, _)| *sequence <= cutoff) {
        rows.insert(
            event.entity_id,
            Row {
                entity_id: event.entity_id,
                logical_time: event.logical_time,
                value: event.value,
                sequence,
            },
        );
    }
    rows.into_values().collect()
}

pub fn columns_from_events(events: &[(u64, Event)], cutoff: u64) -> ColumnView {
    let mut entity_to_index = BTreeMap::new();
    let mut view = ColumnView {
        entity_ids: Vec::new(),
        logical_times: Vec::new(),
        values: Vec::new(),
        sequences: Vec::new(),
    };
    for &(sequence, event) in events.iter().filter(|(sequence, _)| *sequence <= cutoff) {
        if let Some(&index) = entity_to_index.get(&event.entity_id) {
            view.logical_times[index] = event.logical_time;
            view.values[index] = event.value;
            view.sequences[index] = sequence;
        } else {
            let index = view.entity_ids.len();
            entity_to_index.insert(event.entity_id, index);
            view.entity_ids.push(event.entity_id);
            view.logical_times.push(event.logical_time);
            view.values.push(event.value);
            view.sequences.push(sequence);
        }
    }

    // Canonical view order is entity order, independent of first-observation order.
    let indices: Vec<_> = entity_to_index.into_values().collect();
    ColumnView {
        entity_ids: indices
            .iter()
            .map(|&index| view.entity_ids[index])
            .collect(),
        logical_times: indices
            .iter()
            .map(|&index| view.logical_times[index])
            .collect(),
        values: indices.iter().map(|&index| view.values[index]).collect(),
        sequences: indices.iter().map(|&index| view.sequences[index]).collect(),
    }
}

fn expected(events: &[Event], cutoff: usize) -> Vec<Row> {
    let mut state = BTreeMap::new();
    for (index, event) in events.iter().take(cutoff).enumerate() {
        state.insert(
            event.entity_id,
            Row {
                entity_id: event.entity_id,
                logical_time: event.logical_time,
                value: event.value,
                sequence: index as u64 + 1,
            },
        );
    }
    state.into_values().collect()
}

fn columns_for_rows(rows: &[Row]) -> ColumnView {
    ColumnView {
        entity_ids: rows.iter().map(|row| row.entity_id).collect(),
        logical_times: rows.iter().map(|row| row.logical_time).collect(),
        values: rows.iter().map(|row| row.value).collect(),
        sequences: rows.iter().map(|row| row.sequence).collect(),
    }
}

pub fn create_output_directory(path: &Path) -> Result<(), String> {
    fs::create_dir(path)
        .map_err(|error| format!("output directory must be newly and exclusively created: {error}"))
}

pub fn unique_default_output(parent: &Path) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!("history-views-{nanos}-{}", std::process::id()))
}

pub fn run_trial(root: &Path, event_count: usize, trial_name: &str) -> Result<Trial, String> {
    let events = generate(event_count); // outside every timed region
    let payload_bytes = events.len() * 28;
    let start = Instant::now();
    let frames: Vec<Vec<u8>> = events
        .iter()
        .copied()
        .enumerate()
        .map(|(i, event)| frame(event, i as u64 + 1))
        .collect::<Result<_, _>>()?;
    let encode_time = start.elapsed();
    let physical_bytes = frames.iter().map(Vec::len).sum();
    let path = root.join(format!("{trial_name}.rf1"));
    let start = Instant::now();
    let mut appender = RawAppender::open(&path).map_err(|error| error.to_string())?;
    for frame in &frames {
        appender
            .append(frame)
            .map_err(|error| format!("append failed: {error:?}"))?;
    }
    drop(appender);
    let append = start.elapsed();
    let start = Instant::now();
    let replay = reopen_and_replay(&path, ScanLimits::default());
    let replay_time = start.elapsed();
    if replay.termination != ReplayTermination::CleanEof
        || replay.records.len() != event_count
        || replay.accepted_prefix.len() != physical_bytes
        || replay.scanned_bytes != physical_bytes as u64
    {
        return Err("record count/byte count/clean termination mismatch".into());
    }
    let mut offset = 0_u64;
    for (index, (record, retained)) in replay.records.iter().zip(frames.iter()).enumerate() {
        if record.offset != offset
            || record.extent != retained.len()
            || record.bytes != *retained
            || record.record.physical_ordinal != index as u64 + 1
        {
            return Err(format!("record order or retained-byte mismatch at {index}"));
        }
        offset += retained.len() as u64;
    }
    let midpoint = event_count / 2;
    let final_expected = expected(&events, event_count);
    let midpoint_expected = expected(&events, midpoint);
    let start = Instant::now();
    let rows = rows_from_records(&replay.records, event_count as u64)?;
    let row_rebuild = start.elapsed();
    let start = Instant::now();
    let columns = columns_from_records(&replay.records, event_count as u64)?;
    let column_rebuild = start.elapsed();
    let direct_input: Vec<_> = events
        .iter()
        .copied()
        .enumerate()
        .map(|(i, event)| (i as u64 + 1, event))
        .collect();
    let start = Instant::now();
    let direct_rows = rows_from_events(&direct_input, event_count as u64);
    let direct_row = start.elapsed();
    let start = Instant::now();
    let direct_columns = columns_from_events(&direct_input, event_count as u64);
    let direct_column = start.elapsed();
    let correctness = rows == final_expected
        && columns == columns_for_rows(&final_expected)
        && rows_from_records(&replay.records, midpoint as u64)? == midpoint_expected
        && columns_from_records(&replay.records, midpoint as u64)?
            == columns_for_rows(&midpoint_expected)
        && direct_rows == final_expected
        && direct_columns == columns_for_rows(&final_expected);
    if !correctness {
        return Err("row, column, historical, or baseline correctness mismatch".into());
    }
    Ok(Trial {
        event_count,
        payload_bytes,
        physical_bytes,
        encode: encode_time,
        append,
        replay: replay_time,
        row_rebuild,
        column_rebuild,
        direct_row,
        direct_column,
        correctness,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_oracle_reconstructs_final_and_historical_views() {
        let events = [
            (
                1,
                Event {
                    entity_id: 2,
                    logical_time: 100,
                    value: 7,
                },
            ),
            (
                2,
                Event {
                    entity_id: 1,
                    logical_time: 90,
                    value: 3,
                },
            ),
            (
                3,
                Event {
                    entity_id: 2,
                    logical_time: 80,
                    value: 9,
                },
            ),
            (
                4,
                Event {
                    entity_id: 1,
                    logical_time: 110,
                    value: 5,
                },
            ),
        ];
        let midpoint = vec![
            Row {
                entity_id: 1,
                logical_time: 90,
                value: 3,
                sequence: 2,
            },
            Row {
                entity_id: 2,
                logical_time: 100,
                value: 7,
                sequence: 1,
            },
        ];
        let final_rows = vec![
            Row {
                entity_id: 1,
                logical_time: 110,
                value: 5,
                sequence: 4,
            },
            Row {
                entity_id: 2,
                logical_time: 80,
                value: 9,
                sequence: 3,
            },
        ];
        assert_eq!(rows_from_events(&events, 2), midpoint);
        assert_eq!(rows_from_events(&events, 4), final_rows);
        assert_eq!(
            columns_from_events(&events, 2),
            ColumnView {
                entity_ids: vec![1, 2],
                logical_times: vec![90, 100],
                values: vec![3, 7],
                sequences: vec![2, 1],
            }
        );
        assert_eq!(
            columns_from_events(&events, 4),
            ColumnView {
                entity_ids: vec![1, 2],
                logical_times: vec![110, 80],
                values: vec![5, 9],
                sequences: vec![4, 3],
            }
        );
    }

    #[test]
    fn malformed_payload_is_rejected() {
        assert!(decode_payload(b"EHV1-too-short").is_err());
        let mut wrong_magic = [0_u8; 28];
        wrong_magic[..4].copy_from_slice(b"NOPE");
        assert!(decode_payload(&wrong_magic).is_err());
    }

    #[test]
    fn output_directory_collision_fails_closed() {
        let path = unique_default_output(&std::env::temp_dir());
        create_output_directory(&path).expect("first exclusive creation");
        assert!(create_output_directory(&path).is_err());
        fs::remove_dir(&path).expect("test-owned cleanup");
    }

    #[test]
    fn reopened_history_is_the_only_reconstruction_source() {
        let root = unique_default_output(&std::env::temp_dir());
        create_output_directory(&root).expect("test output");
        let result = run_trial(&root, 100, "focused").expect("valid experiment");
        assert!(result.correctness);
        assert_eq!(result.payload_bytes, 2_800);
        fs::remove_dir_all(root).expect("test-owned cleanup");
    }
}
