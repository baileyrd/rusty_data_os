//! EXP-0004 Entity adapter; one independent Log, no cross-domain commitment.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
pub use uc_core::Uuid;
use uc_core::{Durability, Log, LogError, OpenReport, Outcome, Rejection, Transaction};

pub type Id = [u8; 16];
pub const MAX_OPERATIONS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: Uuid,
    pub label: String,
    pub kind: String,
    pub mention_count: i64,
    pub aliases: Vec<String>,
}
/// Legacy open-label rule, measured in bytes (not Unicode characters).
pub fn valid_relation_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && !label.starts_with('-')
        && label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub incarnation: u64,
    pub record: Option<Arc<Entity>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub slots: BTreeMap<Id, Slot>,
    /// Canonical endpoint order stores each symmetric edge once.
    pub edges: BTreeSet<(Id, u64, String, Id, u64)>,
    pub known_labels: BTreeSet<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            slots: BTreeMap::new(),
            edges: BTreeSet::new(),
            known_labels: ["relates_to".into(), "mentioned_with".into()].into(),
        }
    }
}
impl State {
    pub fn records(&self) -> Vec<Entity> {
        self.slots
            .values()
            .filter_map(|s| s.record.as_deref().cloned())
            .collect()
    }
    pub fn record(&self, id: &Id) -> Option<&Entity> {
        self.slots.get(id)?.record.as_deref()
    }
    pub fn incarnation(&self, id: &Id) -> Option<u64> {
        self.slots.get(id).map(|s| s.incarnation)
    }
    pub fn current(&self, id: &Id, incarnation: u64) -> Result<&Entity, String> {
        let slot = self.slots.get(id).ok_or("RecordNotFound")?;
        if slot.incarnation != incarnation {
            return Err("StaleIncarnation".into());
        }
        slot.record
            .as_deref()
            .ok_or_else(|| "RecordNotFound".into())
    }
    pub fn neighbors_by_relation(&self, label: &str, id: &Id) -> Result<Vec<Id>, String> {
        if !self.known_labels.contains(label) {
            return Err("Malformed".into());
        }
        Ok(self
            .edges
            .iter()
            .filter(|(_, _, relation, _, _)| relation == label)
            .filter_map(|(a, _, _, b, _)| {
                if a == id {
                    Some(*b)
                } else if b == id {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }
    pub fn neighbors(&self, id: &Id) -> Vec<Id> {
        self.edges
            .iter()
            .filter_map(|(a, _, _, b, _)| {
                if a == id {
                    Some(*b)
                } else if b == id {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn list_relation_kinds(&self) -> Vec<String> {
        self.known_labels.iter().cloned().collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Put {
        record: Box<Entity>,
        incarnation: u64,
        insert: bool,
    },
    UpdateMentionCount {
        id: Id,
        incarnation: u64,
        value: i64,
    },
    Delete {
        id: Id,
        incarnation: u64,
    },
    Link {
        relation: String,
        left: Id,
        left_incarnation: u64,
        right: Id,
        right_incarnation: u64,
    },
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}
fn unhex(s: &str) -> Result<Vec<u8>, String> {
    fn digit(b: u8) -> Result<u8, String> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err("invalid hex".into()),
        }
    }
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| Ok(digit(p[0])? * 16 + digit(p[1])?))
        .collect()
}
fn text(s: &str) -> Result<String, String> {
    String::from_utf8(unhex(s)?).map_err(|e| e.to_string())
}
fn id(s: &str) -> Result<Id, String> {
    unhex(s)?.try_into().map_err(|_| "UUID width".into())
}
fn number<T: std::str::FromStr>(s: &str) -> Result<T, String> {
    s.parse().map_err(|_| "invalid number".into())
}

fn encode_record(r: &Entity) -> String {
    let mut out = format!(
        "{}\t{}\t{}\t{}\t{}",
        hex(&r.id.0),
        hex(r.label.as_bytes()),
        hex(r.kind.as_bytes()),
        r.mention_count,
        r.aliases.len()
    );
    for alias in &r.aliases {
        out.push('\t');
        out.push_str(&hex(alias.as_bytes()));
    }
    out
}
fn decode_record(fields: &[&str]) -> Result<Entity, String> {
    match fields {
        [key, label, kind, count, len, aliases @ ..] if number::<usize>(len)? == aliases.len() => {
            Ok(Entity {
                id: Uuid(id(key)?),
                label: text(label)?,
                kind: text(kind)?,
                mention_count: number(count)?,
                aliases: aliases.iter().map(|a| text(a)).collect::<Result<_, _>>()?,
            })
        }
        _ => Err("Entity fields".into()),
    }
}

/// Canonical operation payload. Admission enforces byte and operation limits in decode/apply.
pub fn encode_changes(changes: &[Change]) -> Vec<u8> {
    let mut out = String::from("CME1\n");
    for change in changes {
        match change {
            Change::Put {
                record,
                incarnation,
                insert,
            } => out.push_str(&format!(
                "put\t{incarnation}\t{}\t{}\n",
                u8::from(*insert),
                encode_record(record)
            )),
            Change::UpdateMentionCount {
                id,
                incarnation,
                value,
            } => out.push_str(&format!("update\t{incarnation}\t{}\t{value}\n", hex(id))),
            Change::Delete { id, incarnation } => {
                out.push_str(&format!("delete\t{incarnation}\t{}\n", hex(id)))
            }
            Change::Link {
                relation,
                left,
                left_incarnation,
                right,
                right_incarnation,
            } => out.push_str(&format!(
                "link\t{}\t{left_incarnation}\t{}\t{right_incarnation}\t{}\n",
                hex(relation.as_bytes()),
                hex(left),
                hex(right)
            )),
        }
    }
    out.into_bytes()
}
pub fn decode_changes(bytes: &[u8]) -> Result<Vec<Change>, String> {
    if bytes.len() > uc_core::MAX_PAYLOAD {
        return Err("payload byte limit".into());
    }
    let body = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .strip_prefix("CME1\n")
        .ok_or("CME1 magic")?;
    if body.is_empty() || !body.ends_with('\n') {
        return Err("empty/truncated CME1".into());
    }
    let mut changes = Vec::new();
    for line in body.lines() {
        if changes.len() == MAX_OPERATIONS {
            return Err("transaction operation limit".into());
        }
        let fields: Vec<_> = line.split('\t').collect();
        changes.push(match fields.as_slice() {
            ["put", inc, insert, rest @ ..] => Change::Put {
                record: Box::new(decode_record(rest)?),
                incarnation: number(inc)?,
                insert: match *insert {
                    "0" => false,
                    "1" => true,
                    _ => return Err("put mode".into()),
                },
            },
            ["update", inc, key, value] => Change::UpdateMentionCount {
                id: id(key)?,
                incarnation: number(inc)?,
                value: number(value)?,
            },
            ["delete", inc, key] => Change::Delete {
                id: id(key)?,
                incarnation: number(inc)?,
            },
            ["link", relation, li, left, ri, right] => Change::Link {
                relation: text(relation)?,
                left: id(left)?,
                left_incarnation: number(li)?,
                right: id(right)?,
                right_incarnation: number(ri)?,
            },
            _ => return Err("CME1 operation".into()),
        });
    }
    if encode_changes(&changes) != bytes {
        return Err("noncanonical CME1".into());
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
                if *insert {
                    if state.record(&record.id.0).is_some() {
                        return Err("Duplicate".into());
                    }
                    let next = state
                        .incarnation(&record.id.0)
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or("incarnation overflow")?;
                    if *incarnation != next {
                        return Err("StaleIncarnation".into());
                    }
                } else {
                    state.current(&record.id.0, *incarnation)?;
                }
                state.slots.insert(
                    record.id.0,
                    Slot {
                        incarnation: *incarnation,
                        record: Some(Arc::new(record.as_ref().clone())),
                    },
                );
            }
            Change::UpdateMentionCount {
                id,
                incarnation,
                value,
            } => {
                let mut record = state.current(id, *incarnation)?.clone();
                record.mention_count = *value;
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
                state.edges.retain(|(a, _, _, b, _)| a != id && b != id);
            }
            Change::Link {
                relation,
                left,
                left_incarnation,
                right,
                right_incarnation,
            } => {
                if !valid_relation_label(relation) {
                    return Err("Malformed".into());
                }
                if left == right {
                    return Err("SelfLoop".into());
                }
                state.current(left, *left_incarnation)?;
                state.current(right, *right_incarnation)?;
                let edge = if left < right {
                    (
                        *left,
                        *left_incarnation,
                        relation.clone(),
                        *right,
                        *right_incarnation,
                    )
                } else {
                    (
                        *right,
                        *right_incarnation,
                        relation.clone(),
                        *left,
                        *left_incarnation,
                    )
                };
                state.edges.insert(edge);
                state.known_labels.insert(relation.clone());
            }
        }
    }
    Ok(())
}
/// Apply to a caller-owned staging state. Log discards the entire staged state on error.
/// Every invariant is checked here, including recovery, which never calls the extra validator.
pub fn apply(state: &mut State, bytes: &[u8]) -> Result<(), String> {
    apply_changes(state, &decode_changes(bytes)?)
}
pub fn encode_state(state: &State) -> Result<Vec<u8>, String> {
    let mut out = String::from("CES1\n");
    for (id, slot) in &state.slots {
        out.push_str(&format!(
            "slot\t{}\t{}\t{}\n",
            slot.incarnation,
            hex(id),
            slot.record
                .as_ref()
                .map_or_else(|| "-".into(), |r| encode_record(r))
        ));
    }
    for label in &state.known_labels {
        out.push_str(&format!("label\t{}\n", hex(label.as_bytes())));
    }
    for (left, li, relation, right, ri) in &state.edges {
        out.push_str(&format!(
            "edge\t{}\t{li}\t{}\t{ri}\t{}\n",
            hex(relation.as_bytes()),
            hex(left),
            hex(right)
        ));
    }
    Ok(out.into_bytes())
}
pub fn decode_state(bytes: &[u8]) -> Result<State, String> {
    let body = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .strip_prefix("CES1\n")
        .ok_or("CES1 magic")?;
    let mut state = State::default();
    for line in body.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["slot", inc, key, rest @ ..] => {
                let key = id(key)?;
                let incarnation = number::<u64>(inc)?;
                if incarnation == 0 {
                    return Err("zero incarnation".into());
                }
                let record = if rest == ["-"] {
                    None
                } else {
                    let record = decode_record(rest)?;

                    if record.id.0 != key {
                        return Err("slot identity".into());
                    }
                    Some(Arc::new(record))
                };
                if state
                    .slots
                    .insert(
                        key,
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
            ["label", label] => {
                let label = text(label)?;
                if !valid_relation_label(&label) {
                    return Err("Malformed".into());
                }
                state.known_labels.insert(label);
            }
            ["edge", relation, li, left, ri, right] => {
                apply_changes(
                    &mut state,
                    &[Change::Link {
                        relation: text(relation)?,
                        left: id(left)?,
                        left_incarnation: number(li)?,
                        right: id(right)?,
                        right_incarnation: number(ri)?,
                    }],
                )?;
            }
            _ => return Err("CES1 entry".into()),
        }
    }
    if encode_state(&state)? != bytes {
        return Err("noncanonical CES1".into());
    }
    Ok(state)
}

pub struct EntityEngine {
    log: Log<State>,
    durability: Durability,
}
impl EntityEngine {
    pub fn create(directory: &Path, durability: Durability) -> Result<(Self, OpenReport), String> {
        let (log, report) =
            Log::create(directory, apply, decode_state).map_err(|e| e.to_string())?;
        Ok((Self { log, durability }, report))
    }
    pub fn open(directory: &Path, durability: Durability) -> Result<(Self, OpenReport), String> {
        let (log, report) = Log::open(directory, apply, decode_state).map_err(|e| e.to_string())?;
        Ok((Self { log, durability }, report))
    }
    pub fn log(&self) -> &Log<State> {
        &self.log
    }
    pub fn log_mut(&mut self) -> &mut Log<State> {
        &mut self.log
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
        // Core stages apply from raw bytes before append; the optional guard is commit-only.
        self.log.commit(
            Transaction {
                request_id,
                payload: encode_changes(changes),
                durability: self.durability,
            },
            validate,
        )
    }
    pub fn neighbors_by_relation(&self, label: &str, id: &Id) -> Result<Vec<Id>, String> {
        self.log.snapshot().neighbors_by_relation(label, id)
    }
    pub fn neighbors(&self, id: &Id) -> Vec<Id> {
        self.log.snapshot().neighbors(id)
    }
    pub fn list_relation_kinds(&self) -> Vec<String> {
        self.log.snapshot().list_relation_kinds()
    }
}
