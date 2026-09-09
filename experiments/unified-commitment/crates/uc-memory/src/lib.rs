//! EXP-0003 CMM2 adapter. Engine mutations never call the independent CMT1 model.
use cm_trace::{Answer, Id, Memory, Op, Value, format, hex, run::Engine};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use uc_core::{Durability, Log, LogError, OpenReport, Outcome, Rejection, Transaction, Uuid};

pub const MAX_OPERATIONS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub incarnation: u64,
    pub record: Option<Arc<Memory>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub slots: BTreeMap<Id, Slot>,
    pub edges: BTreeSet<(Id, u64, Id, u64)>,
}
impl State {
    pub fn records(&self) -> Vec<Memory> {
        self.slots
            .values()
            .filter_map(|s| s.record.as_deref().cloned())
            .collect()
    }
    pub fn current(&self, id: &Id, incarnation: u64) -> Result<&Memory, String> {
        let s = self.slots.get(id).ok_or("NotFound")?;
        if s.incarnation != incarnation {
            return Err("StaleIncarnation".into());
        }
        s.record.as_deref().ok_or_else(|| "NotFound".into())
    }
    pub fn incarnation(&self, id: &Id) -> Option<u64> {
        self.slots.get(id).map(|s| s.incarnation)
    }
    fn record(&self, id: &Id) -> Option<&Memory> {
        self.slots.get(id)?.record.as_deref()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Put {
        record: Box<Memory>,
        incarnation: u64,
        insert: bool,
    },
    Update {
        id: Id,
        incarnation: u64,
        field: usize,
        value: Value,
        equals: Option<Value>,
    },
    Delete {
        id: Id,
        incarnation: u64,
    },
    Link {
        from: Id,
        from_incarnation: u64,
        to: Id,
        to_incarnation: u64,
    },
}
pub fn encode_changes(changes: &[Change]) -> Vec<u8> {
    let mut out = String::from("CMM2\n");
    for change in changes {
        match change {
            Change::Put {
                record,
                incarnation,
                insert,
            } => out.push_str(&format!(
                "put\t{incarnation}\t{}\t{}\n",
                u8::from(*insert),
                format::memory(record)
            )),
            Change::Update {
                id,
                incarnation,
                field,
                value,
                equals,
            } => out.push_str(&format!(
                "update\t{incarnation}\t{}\t{field}\t{}\t{}\n",
                hex(id),
                format::value(value),
                equals.as_ref().map_or_else(|| "-".into(), format::value)
            )),
            Change::Delete { id, incarnation } => {
                out.push_str(&format!("delete\t{incarnation}\t{}\n", hex(id)))
            }
            Change::Link {
                from,
                from_incarnation,
                to,
                to_incarnation,
            } => out.push_str(&format!(
                "link\t{from_incarnation}\t{}\t{to_incarnation}\t{}\n",
                hex(from),
                hex(to)
            )),
        }
    }
    out.into_bytes()
}
fn number<T: std::str::FromStr>(s: &str) -> Result<T, String> {
    s.parse().map_err(|_| "invalid CMM2 number".into())
}
pub fn decode_changes(bytes: &[u8]) -> Result<Vec<Change>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let text = text.strip_prefix("CMM2\n").ok_or("CMM2 magic")?;
    if !text.ends_with('\n') {
        return Err("empty/truncated CMM2".into());
    }
    let mut changes = Vec::new();
    for line in text.lines() {
        let f: Vec<_> = line.split('\t').collect();
        let change = match f.as_slice() {
            ["put", inc, insert, rest @ ..] if !rest.is_empty() => Change::Put {
                record: Box::new(format::parse_memory(&rest.join("\t"))?),
                incarnation: number(inc)?,
                insert: match *insert {
                    "0" => false,
                    "1" => true,
                    _ => return Err("put mode".into()),
                },
            },
            ["update", inc, id, field, value, equals] => Change::Update {
                id: format::id(id)?,
                incarnation: number(inc)?,
                field: number(field)?,
                value: format::parse_value(value)?,
                equals: if *equals == "-" {
                    None
                } else {
                    Some(format::parse_value(equals)?)
                },
            },
            ["delete", inc, id] => Change::Delete {
                id: format::id(id)?,
                incarnation: number(inc)?,
            },
            ["link", from_inc, from, to_inc, to] => Change::Link {
                from: format::id(from)?,
                from_incarnation: number(from_inc)?,
                to: format::id(to)?,
                to_incarnation: number(to_inc)?,
            },
            _ => return Err("CMM2 operation".into()),
        };
        changes.push(change);
        if changes.len() > MAX_OPERATIONS {
            return Err("transaction operation limit".into());
        }
    }
    Ok(changes)
}
fn apply_changes(state: &mut State, changes: &[Change]) -> Result<(), String> {
    for change in changes {
        match change {
            Change::Put {
                record,
                incarnation,
                insert,
            } => {
                record.validate()?;
                if *insert {
                    if state.record(&record.id).is_some() {
                        return Err("Duplicate".into());
                    }
                    let next = state
                        .incarnation(&record.id)
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or("incarnation overflow")?;
                    if *incarnation != next {
                        return Err("StaleIncarnation".into());
                    }
                } else {
                    state.current(&record.id, *incarnation)?;
                }
                state.slots.insert(
                    record.id,
                    Slot {
                        incarnation: *incarnation,
                        record: Some(Arc::new(record.as_ref().clone())),
                    },
                );
            }
            Change::Update {
                id,
                incarnation,
                field,
                value,
                equals,
            } => {
                let mut record = state.current(id, *incarnation)?.clone();
                let prior = record.fields.get(*field).ok_or("invalid field")?;
                if equals.as_ref().is_some_and(|v| v != prior) {
                    return Err("GuardFailed".into());
                }
                record.fields[*field] = value.clone();
                record.validate()?;
                state.slots.insert(
                    *id,
                    Slot {
                        incarnation: *incarnation,
                        record: Some(Arc::new(record)),
                    },
                );
            }
            Change::Delete { id, incarnation } => {
                state.current(id, *incarnation)?;
                state.slots.get_mut(id).unwrap().record = None;
                state.edges.retain(|(a, _, b, _)| a != id && b != id);
            }
            Change::Link {
                from,
                from_incarnation,
                to,
                to_incarnation,
            } => {
                state.current(from, *from_incarnation)?;
                state.current(to, *to_incarnation)?;
                state
                    .edges
                    .insert((*from, *from_incarnation, *to, *to_incarnation));
            }
        }
    }
    Ok(())
}
pub fn apply(state: &mut State, bytes: &[u8]) -> Result<(), String> {
    apply_changes(state, &decode_changes(bytes)?)
}
pub fn encode_state(state: &State) -> Result<Vec<u8>, String> {
    let mut text = String::from("CMS2\n");
    for (id, slot) in &state.slots {
        text.push_str(&format!(
            "slot\t{}\t{}\t{}\n",
            slot.incarnation,
            hex(id),
            slot.record
                .as_ref()
                .map_or_else(|| "-".into(), |m| format::memory(m))
        ));
    }
    for (from, fi, to, ti) in &state.edges {
        text.push_str(&format!("edge\t{fi}\t{}\t{ti}\t{}\n", hex(from), hex(to)));
    }
    Ok(text.into_bytes())
}
pub fn decode_state(bytes: &[u8]) -> Result<State, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .strip_prefix("CMS2\n")
        .ok_or("CMS2 magic")?;
    let mut state = State::default();
    for line in text.lines() {
        let f: Vec<_> = line.split('\t').collect();
        match f.as_slice() {
            ["slot", inc, id, rest @ ..] if !rest.is_empty() => {
                let id = format::id(id)?;
                let incarnation = number::<u64>(inc)?;
                if incarnation == 0 {
                    return Err("zero incarnation".into());
                }
                let record = if rest == ["-"] {
                    None
                } else {
                    let m = format::parse_memory(&rest.join("\t"))?;
                    if m.id != id {
                        return Err("slot identity".into());
                    }
                    Some(Arc::new(m))
                };
                if state
                    .slots
                    .insert(
                        id,
                        Slot {
                            incarnation,
                            record,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate slot".into());
                }
            }
            ["edge", fi, from, ti, to] => {
                let change = Change::Link {
                    from: format::id(from)?,
                    from_incarnation: number(fi)?,
                    to: format::id(to)?,
                    to_incarnation: number(ti)?,
                };
                apply_changes(&mut state, &[change])?;
            }
            _ => return Err("CMS2 entry".into()),
        }
    }
    if encode_state(&state)? != bytes {
        return Err("noncanonical CMS2".into());
    }
    Ok(state)
}

pub struct MemoryEngine {
    log: Option<Log<State>>,
    directory: PathBuf,
    durability: Durability,
    label: String,
}
impl MemoryEngine {
    pub fn open(directory: &Path, durability: Durability) -> Result<(Self, OpenReport), String> {
        let (log, report) = Log::open(directory, apply, decode_state).map_err(|e| e.to_string())?;
        Ok((
            Self {
                log: Some(log),
                directory: directory.to_owned(),
                durability,
                label: Self::series_label(durability),
            },
            report,
        ))
    }
    pub fn series_label(durability: Durability) -> String {
        format!("{}; rev={}", durability.label(), cm_trace::run::revision())
    }
    pub fn create(directory: &Path, durability: Durability) -> Result<(Self, OpenReport), String> {
        Self::create_with_label(directory, durability, Self::series_label(durability))
    }
    pub fn create_with_label(
        directory: &Path,
        durability: Durability,
        label: String,
    ) -> Result<(Self, OpenReport), String> {
        let (log, report) =
            Log::create(directory, apply, decode_state).map_err(|e| e.to_string())?;
        Ok((
            Self {
                log: Some(log),
                directory: directory.to_owned(),
                durability,
                label,
            },
            report,
        ))
    }
    pub fn log(&self) -> &Log<State> {
        self.log.as_ref().expect("open engine")
    }
    pub fn log_mut(&mut self) -> &mut Log<State> {
        self.log.as_mut().expect("open engine")
    }
    pub fn transact(&mut self, request_id: Uuid, changes: &[Change]) -> Result<Outcome, LogError> {
        self.transact_checked(request_id, changes, |_| Ok(()))
    }
    pub fn transact_checked(
        &mut self,
        request_id: Uuid,
        changes: &[Change],
        validate: impl FnOnce(&State) -> Result<(), Rejection>,
    ) -> Result<Outcome, LogError> {
        let payload = encode_changes(changes);
        let txn = Transaction {
            request_id,
            payload,
            durability: self.durability,
        };
        self.log_mut().commit(txn, |state| {
            validate(state)?;
            let mut staged = state.clone();
            apply_changes(&mut staged, changes).map_err(Rejection::Validation)
        })
    }
    fn persist(
        &mut self,
        changes: &[Change],
        guard: impl FnOnce(&State) -> Result<(), Rejection>,
    ) -> Result<(), String> {
        let request = self.log().next_request_id();
        match self
            .transact_checked(request, changes, guard)
            .map_err(|e| e.to_string())?
        {
            Outcome::Committed { .. } => Ok(()),
            other => Err(format!("transaction: {other:?}")),
        }
    }
}
impl Engine for MemoryEngine {
    fn creates_store_directory() -> bool {
        true
    }
    fn label(&self) -> String {
        self.label.clone()
    }
    fn durability(&self) -> String {
        self.durability.label().into()
    }
    fn records(&self) -> Result<Vec<Memory>, String> {
        Ok(self.log().snapshot().records())
    }
    fn execute(&mut self, op: &Op) -> Result<Answer, String> {
        let state = self.log().snapshot();
        let status = match op {
            Op::Insert(m) => {
                if state.record(&m.id).is_some() {
                    "Duplicate"
                } else {
                    let incarnation = state
                        .incarnation(&m.id)
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or("incarnation overflow")?;
                    self.persist(
                        &[Change::Put {
                            record: Box::new(m.clone()),
                            incarnation,
                            insert: true,
                        }],
                        |_| Ok(()),
                    )?;
                    "Inserted"
                }
            }
            Op::Replace(m) => {
                if state.record(&m.id).is_none() {
                    "NotFound"
                } else {
                    self.persist(
                        &[Change::Put {
                            record: Box::new(m.clone()),
                            incarnation: state.incarnation(&m.id).unwrap(),
                            insert: false,
                        }],
                        |_| Ok(()),
                    )?;
                    "Replaced"
                }
            }
            Op::Guard {
                record,
                field,
                equals,
            } => match state.record(&record.id) {
                None => "NotFound",
                Some(m) if m.fields.get(*field) != Some(equals) => "GuardFailed",
                Some(_) => {
                    self.persist(
                        &[Change::Put {
                            record: Box::new(record.clone()),
                            incarnation: state.incarnation(&record.id).unwrap(),
                            insert: false,
                        }],
                        |current| {
                            if current
                                .record(&record.id)
                                .and_then(|m| m.fields.get(*field))
                                == Some(equals)
                            {
                                Ok(())
                            } else {
                                Err(Rejection::Validation("GuardFailed".into()))
                            }
                        },
                    )?;
                    "Replaced"
                }
            },
            Op::Update(id, value) => {
                if state.record(id).is_none() {
                    "NotFound"
                } else {
                    self.persist(
                        &[Change::Update {
                            id: *id,
                            incarnation: state.incarnation(id).unwrap(),
                            field: 10,
                            value: Value::Int(*value),
                            equals: None,
                        }],
                        |_| Ok(()),
                    )?;
                    "Updated"
                }
            }
            Op::Delete(id) => {
                if state.record(id).is_none() {
                    "NotFound"
                } else {
                    self.persist(
                        &[Change::Delete {
                            id: *id,
                            incarnation: state.incarnation(id).unwrap(),
                        }],
                        |_| Ok(()),
                    )?;
                    "Deleted"
                }
            }
            Op::Get(id) => {
                return Ok(state.record(id).map_or_else(
                    || Answer::outcome("NotFound"),
                    |m| Answer::rows(vec![m.clone()]),
                ));
            }
            Op::Equal(s) => {
                return Ok(Answer::rows(
                    state
                        .records()
                        .into_iter()
                        .filter(|m| matches!(&m.fields[1],Value::Text(v) if v == s))
                        .collect(),
                ));
            }
            Op::Aggregate => {
                let (mut count, mut sum, mut min, mut max) = (0, 0i128, None, None);
                for slot in state.slots.values() {
                    if let Some(m) = &slot.record {
                        let v = m.fields[10].integer();
                        count += 1;
                        sum += i128::from(v);
                        min = Some(min.map_or(v, |x: i64| x.min(v)));
                        max = Some(max.map_or(v, |x: i64| x.max(v)));
                    }
                }
                return Ok(Answer {
                    aggregate: Some((count, sum, min, max)),
                    ..Answer::outcome("Aggregate")
                });
            }
            Op::Page { after, limit } => {
                let mut rows: Vec<_> = state
                    .records()
                    .into_iter()
                    .filter(|m| after.is_none_or(|a| (m.fields[6].integer(), m.id) > a))
                    .collect();
                rows.sort_by_key(|m| (m.fields[6].integer(), m.id));
                rows.truncate(*limit);
                return Ok(Answer::rows(rows));
            }
        };
        Ok(Answer::outcome(status))
    }
    fn finish(&mut self, expected: &[Memory]) -> Result<Vec<(String, u128)>, String> {
        let history = uc_core::read_history(&self.directory.join(uc_core::HISTORY_FILE))
            .map_err(|e| e.to_string())?;
        let replay_ns = history.report.replay_time.as_nanos();
        let start = Instant::now();
        let changes: Vec<_> = history
            .events
            .iter()
            .map(|e| decode_changes(&e.payload))
            .collect::<Result<_, _>>()?;
        let decode_ns = start.elapsed().as_nanos();
        drop(history);
        let start = Instant::now();
        let mut state = State::default();
        for transaction in &changes {
            apply_changes(&mut state, transaction)?;
        }
        let rows_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let columns = rebuild_columns(&changes);
        let columns_ns = start.elapsed().as_nanos();
        let rows = state.records();
        let ids: Vec<_> = columns[0].keys().copied().collect();
        if columns
            .iter()
            .any(|c| c.keys().copied().collect::<Vec<_>>() != ids)
        {
            return Err("column identities".into());
        }
        let column_rows: Vec<_> = ids
            .into_iter()
            .map(|id| Memory {
                id,
                fields: std::array::from_fn(|i| columns[i][&id].clone()),
            })
            .collect();
        if rows != expected || column_rows != expected {
            return Err("independent row/column oracle mismatch".into());
        }
        let append_ns = self.log().append_time.as_nanos();
        let payload_bytes = self.log().payload_bytes;
        let physical_bytes = self.log().history_bytes();
        // Counts are retained separately from latency samples; never labelled nanoseconds.
        std::fs::write(self.directory.with_extension("amplification.txt"), format!("payload_bytes={payload_bytes}\nphysical_history_bytes={physical_bytes}\npayload_copies=3\n")).map_err(|e| e.to_string())?;
        let start = Instant::now();
        self.log()
            .checkpoint(encode_state)
            .map_err(|e| e.to_string())?;
        let checkpoint_ns = start.elapsed().as_nanos();
        drop(self.log.take());
        let start = Instant::now();
        let (log, report) =
            Log::open(&self.directory, apply, decode_state).map_err(|e| e.to_string())?;
        let open_ns = start.elapsed().as_nanos();
        self.log = Some(log);
        if report.checkpoint.is_none() || self.log().snapshot().as_ref() != &state {
            return Err("checkpoint reconstruction mismatch".into());
        }
        Ok(vec![
            ("append".into(), append_ns),
            ("replay".into(), replay_ns),
            ("decode".into(), decode_ns),
            ("rows".into(), rows_ns),
            ("columns".into(), columns_ns),
            ("checkpoint".into(), checkpoint_ns),
            ("open_from_checkpoint".into(), open_ns),
        ])
    }
}
fn rebuild_columns(transactions: &[Vec<Change>]) -> [BTreeMap<Id, Value>; 13] {
    let mut columns: [BTreeMap<Id, Value>; 13] = std::array::from_fn(|_| BTreeMap::new());
    for change in transactions.iter().flatten() {
        match change {
            Change::Put { record, .. } => {
                for (column, value) in columns.iter_mut().zip(&record.fields) {
                    column.insert(record.id, value.clone());
                }
            }
            Change::Update {
                id, field, value, ..
            } => {
                columns[*field].insert(*id, value.clone());
            }
            Change::Delete { id, .. } => {
                for column in &mut columns {
                    column.remove(id);
                }
            }
            Change::Link { .. } => {}
        }
    }
    columns
}
