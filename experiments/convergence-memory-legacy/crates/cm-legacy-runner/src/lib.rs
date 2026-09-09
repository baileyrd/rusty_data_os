//! Pinned MemoryConnectionStore, in-process calls; no network server is started.
use cm_trace::{Answer, Memory, Op, Value, run::Engine};
use rusty_multimodal_db::{
    generic::{memory::create_memory_production_stack, production::GenericProductionStore},
    server::{
        ConnectionStore,
        memory::MemoryConnectionStore,
        protocol::{CompareOp, FieldRef, Predicate, RecordId, ScanValue, WriteOp},
    },
};
use std::path::Path;
pub const REVISION: &str = "abda0a7e94a9727e410001e9724edc741f6e0d31";
pub const DURABILITY: &str = "legacy Memory@2; journal off; insert/replace/delete log sync_data; mmap access_count update without per-op flush; not equivalent to D1; no power-loss claim";
pub struct Legacy {
    store: MemoryConnectionStore,
    mode: Mode,
}
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Single,
    Pipelined,
    Atomic,
}
pub fn label_for(mode: Mode) -> String {
    format!("{DURABILITY}; rev={REVISION}; mode={mode:?}; batch_width=1")
}
impl Mode {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "single" => Ok(Self::Single),
            "pipelined" => Ok(Self::Pipelined),
            "atomic" => Ok(Self::Atomic),
            _ => Err("mode must be single, pipelined or atomic".into()),
        }
    }
}
fn wire(v: &Value) -> ScanValue {
    match v {
        Value::Text(s) => ScanValue::Str(s.clone()),
        Value::Tags(t) => ScanValue::StrList(t.clone()),
        Value::Int(n) => ScanValue::I64(*n),
        Value::Bool(b) => ScanValue::Bool(*b),
    }
}
fn fields(m: &Memory) -> Vec<(FieldRef, ScanValue)> {
    m.fields
        .iter()
        .enumerate()
        .map(|(i, v)| (i as FieldRef, wire(v)))
        .collect()
}
fn memory(id: RecordId, fields: Vec<(FieldRef, ScanValue)>) -> Result<Memory, String> {
    let mut values: [Option<Value>; 13] = std::array::from_fn(|_| None);
    for (field, v) in fields {
        let slot = values
            .get_mut(field as usize)
            .ok_or("unknown legacy field")?;
        if slot.is_some() {
            return Err("duplicate legacy field".into());
        }
        *slot = Some(match v {
            ScanValue::Str(s) => Value::Text(s),
            ScanValue::StrList(t) => Value::Tags(t),
            ScanValue::I64(n) => Value::Int(n),
            ScanValue::Bool(b) => Value::Bool(b),
            _ => return Err("unexpected legacy field type".into()),
        });
    }
    let m = Memory {
        id: *id.as_bytes(),
        fields: values
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or("missing legacy field")?
            .try_into()
            .map_err(|_| "fields")?,
    };
    m.validate()?;
    Ok(m)
}
fn guard(field: usize, equals: &Value) -> Predicate {
    Predicate {
        field: field as FieldRef,
        op: CompareOp::Eq,
        value: wire(equals),
    }
}
fn write_op(op: &Op) -> Option<WriteOp> {
    Some(match op {
        Op::Insert(m) => WriteOp::Insert {
            id: RecordId::from_bytes(m.id),
            fields: fields(m),
        },
        Op::Replace(m) => WriteOp::Replace {
            id: RecordId::from_bytes(m.id),
            fields: fields(m),
        },
        Op::Guard {
            record,
            field,
            equals,
        } => WriteOp::ReplaceIf {
            id: RecordId::from_bytes(record.id),
            fields: fields(record),
            guard: guard(*field, equals),
        },
        Op::Delete(id) => WriteOp::Delete {
            id: RecordId::from_bytes(*id),
        },
        _ => return None,
    })
}
impl Legacy {
    pub fn create(dir: &Path, mode: Mode) -> Result<Self, String> {
        let stack = create_memory_production_stack(vec![], &[], &dir.join("memory.mmap"))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            store: MemoryConnectionStore::new(GenericProductionStore::new(stack)),
            mode,
        })
    }
    /// Actual protocol-22 multi-operation boundary, tested separately from width-one timing.
    pub fn batch(&self, ops: &[Op], atomic: bool) -> Result<Vec<Answer>, String> {
        let writes = ops
            .iter()
            .map(|op| write_op(op).ok_or("unsupported batch operation"))
            .collect::<Result<Vec<_>, _>>()?;
        let results = self
            .store
            .write_batch(&writes, atomic)
            .map_err(|e| format!("legacy batch: {e:?}"))?;
        if results.len() != ops.len() {
            return Err("batch result count".into());
        }
        Ok(results
            .into_iter()
            .map(|r| Answer::outcome(&format!("{r:?}")))
            .collect())
    }
}
impl Engine for Legacy {
    fn label(&self) -> String {
        label_for(self.mode)
    }
    fn durability(&self) -> String {
        DURABILITY.into()
    }
    fn execute(&mut self, op: &Op) -> Result<Answer, String> {
        if !matches!(self.mode, Mode::Single) && write_op(op).is_some() {
            return self
                .batch(std::slice::from_ref(op), matches!(self.mode, Mode::Atomic))?
                .pop()
                .ok_or("empty batch".into());
        }
        let status = match op {
            Op::Insert(m) => format!(
                "{:?}",
                self.store
                    .insert_record(RecordId::from_bytes(m.id), fields(m))
                    .map_err(|e| format!("{e:?}"))?
            ),
            Op::Replace(m) => format!(
                "{:?}",
                self.store
                    .replace_record(RecordId::from_bytes(m.id), fields(m))
                    .map_err(|e| format!("{e:?}"))?
            ),
            Op::Guard {
                record,
                field,
                equals,
            } => format!(
                "{:?}",
                self.store
                    .replace_record_if(
                        RecordId::from_bytes(record.id),
                        fields(record),
                        &guard(*field, equals)
                    )
                    .map_err(|e| format!("{e:?}"))?
            ),
            Op::Delete(id) => format!(
                "{:?}",
                self.store
                    .delete_record(RecordId::from_bytes(*id))
                    .map_err(|e| format!("{e:?}"))?
            ),
            Op::Update(id, n) => {
                if self
                    .store
                    .update_field(RecordId::from_bytes(*id), 10, ScanValue::I64(*n))
                    .map_err(|e| format!("{e:?}"))?
                {
                    "Updated".into()
                } else {
                    "NotFound".into()
                }
            }
            Op::Get(id) => {
                return self.store.get(RecordId::from_bytes(*id)).map_or_else(
                    || Ok(Answer::outcome("NotFound")),
                    |f| Ok(Answer::rows(vec![memory(RecordId::from_bytes(*id), f)?])),
                );
            }
            Op::Equal(s) => {
                let mut ids = self
                    .store
                    .filter_eq(1, &ScanValue::Str(s.clone()))
                    .map_err(|e| format!("{e:?}"))?;
                ids.sort();
                let rows = ids
                    .into_iter()
                    .map(|id| {
                        memory(
                            id,
                            self.store.get(id).ok_or("index points to missing record")?,
                        )
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                return Ok(Answer::rows(rows));
            }
            Op::Aggregate => {
                let counts = self
                    .store
                    .scan_field(10)
                    .map_err(|e| format!("{e:?}"))?
                    .into_iter()
                    .map(|v| match v {
                        ScanValue::I64(n) => Ok(n),
                        _ => Err("scan type"),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Answer {
                    aggregate: Some((
                        counts.len() as u64,
                        counts.iter().map(|n| i128::from(*n)).sum(),
                        counts.iter().min().copied(),
                        counts.iter().max().copied(),
                    )),
                    ..Answer::outcome("Aggregate")
                });
            }
            Op::Page { after, limit } => {
                let rows = self
                    .store
                    .page(
                        6,
                        after.map(|(t, id)| (ScanValue::I64(t), RecordId::from_bytes(id))),
                        *limit,
                    )
                    .map_err(|e| format!("{e:?}"))?;
                return Ok(Answer::rows(
                    rows.into_iter()
                        .map(|(id, f)| memory(id, f))
                        .collect::<Result<Vec<_>, _>>()?,
                ));
            }
        };
        Ok(Answer::outcome(&status))
    }
    fn records(&self) -> Result<Vec<Memory>, String> {
        let mut records = self
            .store
            .scan_all()
            .into_iter()
            .map(|(id, f)| memory(id, f))
            .collect::<Result<Vec<_>, _>>()?;
        records.sort_by_key(|m| m.id);
        Ok(records)
    }
    fn finish(&mut self, _expected: &[Memory]) -> Result<Vec<(String, u128)>, String> {
        Ok(vec![])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_and_1k_all_dispatch_modes() {
        for mode in [Mode::Single, Mode::Pipelined, Mode::Atomic] {
            for trace in [
                cm_trace::generate::small(),
                cm_trace::generate::repeated(1000, 7),
            ] {
                let trace = cm_trace::run::LoadedTrace::from_trace(trace);
                let dir = cm_trace::run::test_directory("legacy");
                std::fs::create_dir(&dir).unwrap();
                let mut engine = Legacy::create(&dir, mode).unwrap();
                let report = cm_trace::run::trial(&trace, &mut engine, &dir).unwrap();
                assert_eq!(report.mismatches, 0, "retained {}", dir.display());
                let observed = cm_trace::results::Results::decode(
                    &std::fs::read_to_string(dir.join("observations.cmt")).unwrap(),
                )
                .unwrap();
                assert_eq!(observed.durability, DURABILITY);
                assert_eq!(observed.engine, label_for(mode));
                drop(engine);
                std::fs::remove_dir_all(dir).unwrap();
            }
        }
    }
    #[test]
    fn multi_write_batches_observe_earlier_writes() {
        for atomic in [false, true] {
            let dir = cm_trace::run::test_directory("batch");
            std::fs::create_dir(&dir).unwrap();
            let engine = Legacy::create(&dir, Mode::Single).unwrap();
            let mut rng = cm_trace::generate::SplitMix64(7);
            let a = cm_trace::generate::record(1, 0, &mut rng, false);
            let b = cm_trace::generate::record(1, 2, &mut rng, false);
            let ops = vec![
                Op::Insert(a.clone()),
                Op::Insert(b.clone()),
                Op::Guard {
                    record: b.clone(),
                    field: 10,
                    equals: Value::Int(99),
                },
                Op::Replace(b.clone()),
                Op::Delete(a.id),
                Op::Insert(a),
            ];
            let mut model = cm_trace::Model::default();
            let expected: Vec<_> = ops.iter().map(|op| model.apply(op)).collect();
            assert_eq!(engine.batch(&ops, atomic).unwrap(), expected);
            assert_eq!(engine.records().unwrap(), model.records());
            drop(engine);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}
