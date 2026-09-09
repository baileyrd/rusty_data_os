//! EXP-0004 Relation adapter; one independent Log, no cross-domain commitment.
use std::{collections::BTreeMap, path::Path, sync::Arc};
pub use uc_core::Uuid;
use uc_core::{Durability, Log, LogError, OpenReport, Outcome, Rejection, Transaction};

pub type Id = [u8; 16];
pub const MAX_OPERATIONS: usize = 4096;

/// Standalone legacy ADR-0058 record: "nothing checks that an endpoint names an entity".
/// Subject/object strings assert no cross-table existence; only non-emptiness is checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    pub id: Uuid,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub node_id: String,
    pub deleted_at_unix_ms: i64,
}
impl Relation {
    fn validate(&self) -> Result<(), String> {
        if self.subject.is_empty() {
            return Err("empty subject".into());
        }
        if self.relation.is_empty() {
            return Err("empty relation".into());
        }
        if self.object.is_empty() {
            return Err("empty object".into());
        }
        if self.deleted_at_unix_ms < 0 {
            return Err("negative deleted_at_unix_ms".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub incarnation: u64,
    pub record: Option<Arc<Relation>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub slots: BTreeMap<Id, Slot>,
}
impl State {
    pub fn records(&self) -> Vec<Relation> {
        self.slots
            .values()
            .filter_map(|s| s.record.as_deref().cloned())
            .collect()
    }
    pub fn record(&self, id: &Id) -> Option<&Relation> {
        self.slots.get(id)?.record.as_deref()
    }
    pub fn incarnation(&self, id: &Id) -> Option<u64> {
        self.slots.get(id).map(|s| s.incarnation)
    }
    pub fn current(&self, id: &Id, incarnation: u64) -> Result<&Relation, String> {
        let slot = self.slots.get(id).ok_or("RecordNotFound")?;
        if slot.incarnation != incarnation {
            return Err("StaleIncarnation".into());
        }
        slot.record
            .as_deref()
            .ok_or_else(|| "RecordNotFound".into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Put {
        record: Box<Relation>,
        incarnation: u64,
        insert: bool,
    },
    UpdateTimestamp {
        id: Id,
        incarnation: u64,
        value: i64,
    },
    Delete {
        id: Id,
        incarnation: u64,
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

fn encode_record(r: &Relation) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        hex(&r.id.0),
        hex(r.subject.as_bytes()),
        hex(r.relation.as_bytes()),
        hex(r.object.as_bytes()),
        r.created_at_unix_ms,
        r.updated_at_unix_ms,
        hex(r.node_id.as_bytes()),
        r.deleted_at_unix_ms
    )
}
fn decode_record(fields: &[&str]) -> Result<Relation, String> {
    match fields {
        [
            key,
            subject,
            relation,
            object,
            created,
            updated,
            node,
            deleted,
        ] => Ok(Relation {
            id: Uuid(id(key)?),
            subject: text(subject)?,
            relation: text(relation)?,
            object: text(object)?,
            created_at_unix_ms: number(created)?,
            updated_at_unix_ms: number(updated)?,
            node_id: text(node)?,
            deleted_at_unix_ms: number(deleted)?,
        }),
        _ => Err("Relation fields".into()),
    }
}

/// Canonical operation payload. Admission enforces byte and operation limits in decode/apply.
pub fn encode_changes(changes: &[Change]) -> Vec<u8> {
    let mut out = String::from("CMR1\n");
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
            Change::UpdateTimestamp {
                id,
                incarnation,
                value,
            } => out.push_str(&format!("update\t{incarnation}\t{}\t{value}\n", hex(id))),
            Change::Delete { id, incarnation } => {
                out.push_str(&format!("delete\t{incarnation}\t{}\n", hex(id)))
            }
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
        .strip_prefix("CMR1\n")
        .ok_or("CMR1 magic")?;
    if body.is_empty() || !body.ends_with('\n') {
        return Err("empty/truncated CMR1".into());
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
            ["update", inc, key, value] => Change::UpdateTimestamp {
                id: id(key)?,
                incarnation: number(inc)?,
                value: number(value)?,
            },
            ["delete", inc, key] => Change::Delete {
                id: id(key)?,
                incarnation: number(inc)?,
            },

            _ => return Err("CMR1 operation".into()),
        });
    }
    if encode_changes(&changes) != bytes {
        return Err("noncanonical CMR1".into());
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
            Change::UpdateTimestamp {
                id,
                incarnation,
                value,
            } => {
                let mut record = state.current(id, *incarnation)?.clone();
                record.updated_at_unix_ms = *value;
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
    let mut out = String::from("CRS1\n");
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

    Ok(out.into_bytes())
}
pub fn decode_state(bytes: &[u8]) -> Result<State, String> {
    let body = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .strip_prefix("CRS1\n")
        .ok_or("CRS1 magic")?;
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
                    record.validate()?;
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

            _ => return Err("CRS1 entry".into()),
        }
    }
    if encode_state(&state)? != bytes {
        return Err("noncanonical CRS1".into());
    }
    Ok(state)
}

pub struct RelationEngine {
    log: Log<State>,
    durability: Durability,
}
impl RelationEngine {
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
}
