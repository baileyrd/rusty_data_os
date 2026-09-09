use crate::{Binding, LogError, MAX_HISTORY, Outcome, Position, Rejection, Uuid, envelope, sha256};
use exp1_raw_append_replay::{ReplayTermination, reopen_and_replay};
use exp1_record_format::{Body, IntegrityProfile, ScanLimits};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub struct OpenReport {
    pub dropped_tail: Option<u64>,
    pub tail_truncated_and_synced: bool,
    pub accepted_bytes: u64,
    pub gaps: Vec<u64>,
    pub unresolved_bindings: Vec<Uuid>,
    pub uncommitted_finals: Vec<u64>,
    pub rejected_checkpoints: Vec<(PathBuf, String)>,
    pub checkpoint: Option<Position>,
    pub replay_time: Duration,
}
#[derive(Clone, Debug)]
pub struct CommittedEvent {
    pub request_id: Uuid,
    pub event_id: Uuid,
    pub position: Position,
    pub outcome: Outcome,
    pub payload: Vec<u8>,
}
pub struct History {
    pub events: Vec<CommittedEvent>,
    pub report: OpenReport,
    pub(crate) prefix: Vec<u8>,
    pub(crate) bindings: BTreeMap<[u8; 16], Binding>,
    pub(crate) ordinal: u64,
    pub(crate) high_water: u64,
}
/// Read-only scan used by open and independent Memory reconstruction.
pub fn read_history(path: &Path) -> Result<History, LogError> {
    let size = std::fs::metadata(path)?.len();
    if size > MAX_HISTORY {
        return Err(LogError::Limit);
    }
    let limits = ScanLimits {
        max_scan_bytes: MAX_HISTORY,
        max_diagnostic_bytes: usize::try_from(size).map_err(|_| LogError::Limit)?,
        ..ScanLimits::default()
    };
    let start = Instant::now();
    let scan = reopen_and_replay(path, limits);
    let replay_time = start.elapsed();
    let mut report = OpenReport {
        replay_time,
        accepted_bytes: scan.accepted_prefix.len() as u64,
        ..OpenReport::default()
    };
    match scan.termination {
        ReplayTermination::CleanEof => {}
        ReplayTermination::TerminalTruncation { offset } => report.dropped_tail = Some(offset),
        ReplayTermination::Failure { offset, error } => {
            return Err(LogError::Corrupt {
                offset,
                error: format!("{error:?}"),
            });
        }
        ReplayTermination::IoFailure { offset, error } => {
            return Err(LogError::Io {
                offset,
                kind: error,
            });
        }
    }
    // reopen_and_replay already validates the accepted prefix lifecycle.
    let mut bindings = BTreeMap::<[u8; 16], Binding>::new();
    let mut events = Vec::new();
    let mut reservations = BTreeMap::new();
    let mut provisionals = BTreeMap::<[u8; 16], (u64, Vec<u8>)>::new();
    let mut finals = BTreeMap::<[u8; 16], u64>::new();
    let mut high_water = 0;
    for physical in &scan.records {
        let r = &physical.record;
        let corrupt = |error: &str| LogError::Corrupt {
            offset: physical.offset,
            error: error.into(),
        };
        if r.integrity != IntegrityProfile::Crc32c
            || (physical.offset == 0 && r.physical_ordinal != 1)
        {
            return Err(corrupt(
                "unified history requires CRC32C and first ordinal 1",
            ));
        }
        match &r.body {
            Body::Binding {
                request_id,
                event_id,
                normalized_request,
            } => {
                envelope::request(normalized_request).map_err(|e| corrupt(&e))?;
                bindings.insert(
                    request_id.0,
                    Binding {
                        event: *event_id,
                        normalized: normalized_request.clone(),
                        hash: sha256(normalized_request),
                        outcome: Outcome::Rejected {
                            reason: Rejection::Uncommitted,
                        },
                        offset: physical.offset,
                    },
                );
            }
            Body::Reservation {
                request_id,
                event_id,
                sequence,
                high_water: water,
            } => {
                if *water < high_water || *sequence <= high_water {
                    return Err(corrupt("reservation high-water/order"));
                }
                high_water = *water;
                reservations.insert(event_id.0, (*sequence, *request_id));
            }
            Body::Provisional {
                event_id,
                sequence,
                group_id,
                member_index,
                member_count,
                stable_core,
            } => {
                let Some((reserved, request_id)) = reservations.get(&event_id.0) else {
                    return Err(corrupt("provisional without reservation"));
                };
                if reserved != sequence
                    || (*group_id, *member_index, *member_count) != (0, 0, 1)
                    || envelope::request(&bindings[&request_id.0].normalized)
                        .map_err(|e| corrupt(&e))?
                        .1
                        != stable_core
                    || provisionals
                        .insert(event_id.0, (*sequence, stable_core.clone()))
                        .is_some()
                {
                    return Err(corrupt("provisional mismatch"));
                }
            }
            Body::Final {
                event_id,
                request_id,
                sequence,
                durability_time,
                complete_envelope,
            } => {
                let mismatch = || LogError::EnvelopeMismatch {
                    physical_ordinal: r.physical_ordinal,
                };
                let binding = bindings.get(&request_id.0).ok_or_else(mismatch)?;
                envelope::validate(
                    complete_envelope,
                    *request_id,
                    *event_id,
                    *sequence,
                    *durability_time,
                    &binding.normalized,
                )
                .map_err(|_| mismatch())?;
                if !provisionals.contains_key(&event_id.0) {
                    return Err(corrupt("Final without Provisional"));
                }
                finals.insert(event_id.0, r.physical_ordinal);
            }
            Body::Commit {
                event_id,
                sequence,
                group_id,
                member_index,
                member_count,
                ..
            } => {
                if (*group_id, *member_index, *member_count) != (0, 0, 1) {
                    return Err(corrupt("group outside bounded core"));
                }
                let (_, request_id) = reservations[&event_id.0];
                let binding = bindings.get_mut(&request_id.0).expect("validated binding");
                let previous = &scan.records[(r.physical_ordinal - 2) as usize].record;
                let Body::Final {
                    durability_time, ..
                } = previous.body
                else {
                    unreachable!("frozen adjacent Commit validation")
                };
                let (achieved, payload) =
                    envelope::request(&binding.normalized).map_err(|e| corrupt(&e))?;
                let outcome = Outcome::Committed {
                    sequence: *sequence,
                    physical_ordinal: r.physical_ordinal,
                    durability_time,
                    achieved,
                };
                let position = Position {
                    sequence: *sequence,
                    physical_ordinal: r.physical_ordinal,
                    offset: physical.offset + physical.extent as u64,
                };
                events.push(CommittedEvent {
                    request_id,
                    event_id: *event_id,
                    position,
                    outcome: outcome.clone(),
                    payload: payload.to_vec(),
                });
                binding.outcome = outcome;
                finals.remove(&event_id.0);
            }
            Body::Membership { .. } => return Err(corrupt("D3 outside bounded core")),
        }
    }
    for (sequence, request) in reservations.values() {
        if !matches!(bindings[&request.0].outcome, Outcome::Committed { .. }) {
            report.gaps.push(*sequence);
        }
    }
    report.gaps.sort();
    report.unresolved_bindings = bindings
        .iter()
        .filter(|(_, b)| !matches!(b.outcome, Outcome::Committed { .. }))
        .map(|(id, _)| Uuid(*id))
        .collect();
    report.uncommitted_finals = finals.into_values().collect();
    report.uncommitted_finals.sort();
    Ok(History {
        events,
        report,
        prefix: scan.accepted_prefix,
        bindings,
        ordinal: scan.records.last().map_or(0, |r| r.record.physical_ordinal),
        high_water,
    })
}
