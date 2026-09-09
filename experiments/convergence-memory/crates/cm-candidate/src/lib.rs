//! EXP-0002 provisional full-after-image RF1 candidate. No model mutation calls.
use cm_trace::{Answer, Id, Memory, Op, Value, format};
use exp1_raw_append_replay::{RawAppender, ReplayReport, ReplayTermination, reopen_and_replay};
use exp1_record_format::{Body, IntegrityProfile, Record, ScanLimits, Uuid, encode};
use std::{
    collections::BTreeMap,
    path::Path,
    time::{Duration, Instant},
};

pub struct Candidate {
    rows: BTreeMap<Id, Memory>,
    appender: RawAppender,
    sequence: u64,
    pub append: Duration,
}
impl Candidate {
    pub fn create(path: &Path) -> Result<Self, String> {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            rows: BTreeMap::new(),
            appender: RawAppender::open(path).map_err(|e| e.to_string())?,
            sequence: 0,
            append: Duration::ZERO,
        })
    }
    pub fn records(&self) -> Vec<Memory> {
        self.rows.values().cloned().collect()
    }
    fn persist(&mut self, id: Id, record: Option<Memory>) -> Result<(), String> {
        let sequence = self.sequence + 1;
        let payload = record.as_ref().map_or_else(
            || format!("CMM1\tdelete\t{}", cm_trace::hex(&id)),
            |m| format!("CMM1\tput\t{}", format::memory(m)),
        );
        let frame = encode(&Record {
            physical_ordinal: sequence,
            integrity: IntegrityProfile::Structural,
            body: Body::Provisional {
                event_id: Uuid(cm_trace::generate::uuid(sequence)),
                sequence,
                group_id: 0,
                member_index: 0,
                member_count: 1,
                stable_core: payload.into_bytes(),
            },
        })
        .map_err(|e| format!("RF1 encode: {e:?}"))?;
        let start = Instant::now();
        let result = self.appender.append(&frame);
        self.append += start.elapsed();
        result.map_err(|e| format!("D1 append: {e:?}"))?;
        self.sequence = sequence;
        match record {
            Some(m) => {
                self.rows.insert(id, m);
            }
            None => {
                self.rows.remove(&id);
            }
        }
        Ok(())
    }
    pub fn execute(&mut self, op: &Op) -> Result<Answer, String> {
        let status = match op {
            Op::Insert(m) => {
                if self.rows.contains_key(&m.id) {
                    "Duplicate"
                } else {
                    self.persist(m.id, Some(m.clone()))?;
                    "Inserted"
                }
            }
            Op::Replace(m) => {
                if self.rows.contains_key(&m.id) {
                    self.persist(m.id, Some(m.clone()))?;
                    "Replaced"
                } else {
                    "NotFound"
                }
            }
            Op::Guard {
                record,
                field,
                equals,
            } => match self.rows.get(&record.id) {
                None => "NotFound",
                Some(m) if &m.fields[*field] != equals => "GuardFailed",
                Some(_) => {
                    self.persist(record.id, Some(record.clone()))?;
                    "Replaced"
                }
            },
            Op::Update(id, value) => {
                if let Some(m) = self.rows.get(id) {
                    let mut m = m.clone();
                    m.fields[10] = Value::Int(*value);
                    self.persist(*id, Some(m))?;
                    "Updated"
                } else {
                    "NotFound"
                }
            }
            Op::Delete(id) => {
                if self.rows.contains_key(id) {
                    self.persist(*id, None)?;
                    "Deleted"
                } else {
                    "NotFound"
                }
            }
            Op::Get(id) => {
                return Ok(self.rows.get(id).map_or_else(
                    || Answer::outcome("NotFound"),
                    |m| Answer::rows(vec![m.clone()]),
                ));
            }
            Op::Equal(s) => {
                return Ok(Answer::rows(
                    self.rows
                        .values()
                        .filter(|m| matches!(&m.fields[1],Value::Text(v) if v==s))
                        .cloned()
                        .collect(),
                ));
            }
            Op::Aggregate => {
                let (mut count, mut sum, mut min, mut max) = (0, 0i128, None, None);
                for m in self.rows.values() {
                    let v = m.fields[10].integer();
                    count += 1;
                    sum += i128::from(v);
                    min = Some(min.map_or(v, |x: i64| x.min(v)));
                    max = Some(max.map_or(v, |x: i64| x.max(v)));
                }
                return Ok(Answer {
                    aggregate: Some((count, sum, min, max)),
                    ..Answer::outcome("Aggregate")
                });
            }
            Op::Page { after, limit } => {
                let mut keys: Vec<_> = self
                    .rows
                    .values()
                    .map(|m| (m.fields[6].integer(), m.id))
                    .filter(|key| after.is_none_or(|c| *key > c))
                    .collect();
                keys.sort();
                return Ok(Answer::rows(
                    keys.into_iter()
                        .take(*limit)
                        .map(|(_, id)| self.rows[&id].clone())
                        .collect(),
                ));
            }
        };
        Ok(Answer::outcome(status))
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    Put(Box<Memory>),
    Delete(Id),
}
/// Time only the physical reopen/scan call; prepare scan limits beforehand.
pub fn scan_history(path: &Path) -> Result<(ReplayReport, Duration), String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    let limits = ScanLimits {
        max_records: 2_000_000,
        max_scan_bytes: size,
        max_diagnostic_bytes: usize::try_from(size).map_err(|_| "history exceeds address space")?,
        ..ScanLimits::default()
    };
    let start = Instant::now();
    let report = reopen_and_replay(path, limits);
    let elapsed = start.elapsed();
    if report.termination != ReplayTermination::CleanEof {
        return Err(format!("replay: {:?}", report.termination));
    }
    Ok((report, elapsed))
}
/// Interpret experiment-local CMM1 payloads separately from physical replay.
pub fn decode_history(report: &ReplayReport) -> Result<Vec<Change>, String> {
    report
        .records
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let Body::Provisional {
                sequence,
                stable_core,
                ..
            } = &p.record.body
            else {
                return Err("unexpected RF1 kind".into());
            };
            if *sequence != i as u64 + 1 || p.record.physical_ordinal != *sequence {
                return Err("RF1 ordering".into());
            }
            let s = std::str::from_utf8(stable_core).map_err(|e| e.to_string())?;
            if let Some(s) = s.strip_prefix("CMM1\tput\t") {
                Ok(Change::Put(Box::new(format::parse_memory(s)?)))
            } else if let Some(s) = s.strip_prefix("CMM1\tdelete\t") {
                Ok(Change::Delete(format::id(s)?))
            } else {
                Err("unknown CMM1 payload".into())
            }
        })
        .collect()
}
pub fn replay(path: &Path) -> Result<Vec<Change>, String> {
    let (report, _) = scan_history(path)?;
    decode_history(&report)
}
pub fn rebuild_rows(history: &[Change]) -> Vec<Memory> {
    let mut rows = BTreeMap::new();
    for change in history {
        match change {
            Change::Put(m) => {
                rows.insert(m.id, m.as_ref().clone());
            }
            Change::Delete(id) => {
                rows.remove(id);
            }
        }
    }
    rows.into_values().collect()
}
/// Each field has its own ID/value map, fed directly from history, never from rows.
pub fn rebuild_columns(history: &[Change]) -> [BTreeMap<Id, Value>; 13] {
    let mut columns: [BTreeMap<Id, Value>; 13] = std::array::from_fn(|_| BTreeMap::new());
    for change in history {
        match change {
            Change::Put(m) => {
                for (column, value) in columns.iter_mut().zip(&m.fields) {
                    column.insert(m.id, value.clone());
                }
            }
            Change::Delete(id) => {
                for column in &mut columns {
                    column.remove(id);
                }
            }
        }
    }
    columns
}
pub fn column_records(columns: &[BTreeMap<Id, Value>; 13]) -> Result<Vec<Memory>, String> {
    let ids: Vec<_> = columns[0].keys().copied().collect();
    if columns
        .iter()
        .any(|c| c.keys().copied().collect::<Vec<_>>() != ids)
    {
        return Err("column identity divergence".into());
    }
    Ok(ids
        .into_iter()
        .map(|id| Memory {
            id,
            fields: std::array::from_fn(|i| columns[i][&id].clone()),
        })
        .collect())
}
