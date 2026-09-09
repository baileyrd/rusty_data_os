//! CMT1 wire types, independent oracle, and shared measurement/retention support.
pub mod format;
pub mod generate;
pub mod results;
pub mod run;
mod sha;
pub use sha::sha256;
use std::collections::BTreeMap;

pub type Id = [u8; 16];
pub const FIELDS: [&str; 13] = [
    "content",
    "category",
    "tags",
    "source",
    "metadata_json",
    "created_at_unix_ms",
    "updated_at_unix_ms",
    "memory_type",
    "status",
    "sensitive",
    "access_count",
    "deleted_at_unix_ms",
    "node_id",
];
pub const D1: &str = "D1 ordinary writes, no fsync, no crash-survival claim";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Text(String),
    Tags(Vec<String>),
    Int(i64),
    Bool(bool),
}
impl Value {
    pub fn integer(&self) -> i64 {
        if let Self::Int(v) = self {
            *v
        } else {
            panic!("validated integer field")
        }
    }
}

/// Wire-equivalent Memory@2 fields in the fixed FIELDS order; UUID is separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memory {
    pub id: Id,
    pub fields: [Value; 13],
}
impl Memory {
    pub fn validate(&self) -> Result<(), String> {
        for (i, value) in self.fields.iter().enumerate() {
            let valid = match i {
                2 => matches!(value, Value::Tags(_)),
                9 => matches!(value, Value::Bool(_)),
                5 | 6 | 10 | 11 => matches!(value, Value::Int(_)),
                _ => matches!(value, Value::Text(_)),
            };
            if !valid {
                return Err(format!("wrong type for {}", FIELDS[i]));
            }
        }
        if self.fields[10].integer() < 0 || self.fields[11].integer() < 0 {
            return Err("negative count/deleted timestamp".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Insert(Memory),
    Get(Id),
    Update(Id, i64),
    Replace(Memory),
    Guard {
        record: Memory,
        field: usize,
        equals: Value,
    },
    Delete(Id),
    Equal(String),
    Aggregate,
    Page {
        after: Option<(i64, Id)>,
        limit: usize,
    },
}
impl Op {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Insert(_) => "insert",
            Self::Get(_) => "get",
            Self::Update(..) => "update",
            Self::Replace(_) => "replace",
            Self::Guard { .. } => "guard",
            Self::Delete(_) => "delete",
            Self::Equal(_) => "equal",
            Self::Aggregate => "aggregate",
            Self::Page { .. } => "page",
        }
    }
    pub fn query(&self) -> bool {
        matches!(
            self,
            Self::Get(_) | Self::Equal(_) | Self::Aggregate | Self::Page { .. }
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub name: String,
    pub seed: u64,
    pub records: usize,
    /// content min/max, metadata min/max, maximum tag count.
    pub shape: [usize; 5],
    pub ops: Vec<(u64, Op)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub outcome: String,
    pub rows: Vec<Memory>,
    pub aggregate: Option<(u64, i128, Option<i64>, Option<i64>)>,
}
impl Answer {
    pub fn outcome(s: &str) -> Self {
        Self {
            outcome: s.into(),
            rows: vec![],
            aggregate: None,
        }
    }
    pub fn rows(rows: Vec<Memory>) -> Self {
        Self {
            rows,
            ..Self::outcome("Rows")
        }
    }
}

/// Deliberately per-field reference state, not an engine or replay implementation.
#[derive(Default)]
pub struct Model {
    pub state: BTreeMap<Id, [Value; 13]>,
}
impl Model {
    pub fn records(&self) -> Vec<Memory> {
        self.state
            .iter()
            .map(|(id, fields)| Memory {
                id: *id,
                fields: fields.clone(),
            })
            .collect()
    }
    pub fn apply(&mut self, op: &Op) -> Answer {
        let status = match op {
            Op::Insert(m) => {
                if let std::collections::btree_map::Entry::Vacant(e) = self.state.entry(m.id) {
                    e.insert(m.fields.clone());
                    "Inserted"
                } else {
                    "Duplicate"
                }
            }
            Op::Replace(m) => {
                if let Some(fields) = self.state.get_mut(&m.id) {
                    *fields = m.fields.clone();
                    "Replaced"
                } else {
                    "NotFound"
                }
            }
            Op::Guard {
                record,
                field,
                equals,
            } => match self.state.get_mut(&record.id) {
                None => "NotFound",
                Some(fields) if fields[*field] != *equals => "GuardFailed",
                Some(fields) => {
                    *fields = record.fields.clone();
                    "Replaced"
                }
            },
            Op::Update(id, value) => {
                if let Some(fields) = self.state.get_mut(id) {
                    fields[10] = Value::Int(*value);
                    "Updated"
                } else {
                    "NotFound"
                }
            }
            Op::Delete(id) => {
                if self.state.remove(id).is_some() {
                    "Deleted"
                } else {
                    "NotFound"
                }
            }
            Op::Get(id) => {
                return match self.state.get(id) {
                    Some(fields) => Answer::rows(vec![Memory {
                        id: *id,
                        fields: fields.clone(),
                    }]),
                    None => Answer::outcome("NotFound"),
                };
            }
            Op::Equal(s) => {
                return Answer::rows(
                    self.records()
                        .into_iter()
                        .filter(|m| m.fields[1] == Value::Text(s.clone()))
                        .collect(),
                );
            }
            Op::Aggregate => {
                let counts: Vec<_> = self.state.values().map(|f| f[10].integer()).collect();
                return Answer {
                    aggregate: Some((
                        counts.len() as u64,
                        counts.iter().map(|v| i128::from(*v)).sum(),
                        counts.iter().min().copied(),
                        counts.iter().max().copied(),
                    )),
                    ..Answer::outcome("Aggregate")
                };
            }
            Op::Page { after, limit } => {
                let mut rows = self.records();
                rows.sort_by_key(|m| (m.fields[6].integer(), m.id));
                rows.retain(|m| after.is_none_or(|cursor| (m.fields[6].integer(), m.id) > cursor));
                rows.truncate(*limit);
                return Answer::rows(rows);
            }
        };
        Answer::outcome(status)
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn digest(m: &Memory) -> String {
    hex(&sha256(
        format!("CMT1/record\t{}", format::memory(m)).as_bytes(),
    ))
}
/// Records sorted by UUID; ordered query sequence is retained separately.
pub fn record_digests(rows: &[Memory]) -> Vec<(Id, String)> {
    let mut result: Vec<_> = rows.iter().map(|m| (m.id, digest(m))).collect();
    result.sort();
    result
}
pub fn column_digests(rows: &[Memory]) -> Vec<String> {
    let mut sorted: Vec<_> = rows.iter().collect();
    sorted.sort_by_key(|m| m.id);
    (0..13)
        .map(|i| {
            let mut bytes = format!("CMT1/column/{i}\n");
            for m in &sorted {
                bytes.push_str(&format!(
                    "{}\t{}\n",
                    hex(&m.id),
                    format::value(&m.fields[i])
                ));
            }
            hex(&sha256(bytes.as_bytes()))
        })
        .collect()
}
pub fn answer_signature(a: &Answer) -> String {
    format!(
        "{}\t{:?}\t{}\t{}",
        a.outcome,
        a.aggregate,
        record_digests(&a.rows)
            .iter()
            .map(|(id, d)| format!("{}:{d}", hex(id)))
            .collect::<Vec<_>>()
            .join(","),
        a.rows
            .iter()
            .map(|m| hex(&m.id))
            .collect::<Vec<_>>()
            .join(",")
    )
}
