use crate::*;
use cm_trace::{Memory, Value};
use std::sync::{Arc, Mutex};
use uc_memory::{Change, MemoryEngine, State};

/// Owns one independent engine and holds exclusive access across checks and commit.
pub struct MemoryStore(Mutex<MemoryEngine>, Arc<dyn Store>);
impl MemoryStore {
    /// The Entity handle must be the same store registered as the foreign target.
    pub fn new(engine: MemoryEngine, entity: Arc<dyn Store>) -> Self {
        Self(Mutex::new(engine), entity)
    }
    fn schema() -> DomainSchema {
        schema(
            &[
                ("content", ValueKind::Str),
                ("category", ValueKind::Str),
                ("tags", ValueKind::StrList),
                ("source", ValueKind::Str),
                ("metadata_json", ValueKind::Str),
                ("created_at_unix_ms", ValueKind::I64),
                ("updated_at_unix_ms", ValueKind::I64),
                ("memory_type", ValueKind::Str),
                ("status", ValueKind::Str),
                ("sensitive", ValueKind::Bool),
                ("access_count", ValueKind::I64),
                ("deleted_at_unix_ms", ValueKind::I64),
                ("node_id", ValueKind::Str),
            ],
            &[1],
            10,
            true,
        )
    }
    fn decode(id: RecordId, fields: Fields) -> Result<Memory, ErrorCode> {
        let values = decode_fields(fields, &Self::schema())?;
        Ok(Memory {
            id: id.0,
            fields: values
                .into_iter()
                .map(|v| match v {
                    ScanValue::Str(s) => Value::Text(s),
                    ScanValue::StrList(v) => Value::Tags(v),
                    ScanValue::I64(i) => Value::Int(i),
                    ScanValue::Bool(b) => Value::Bool(b),
                    _ => unreachable!("decoded Memory field"),
                })
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| ErrorCode::Malformed)?,
        })
    }
    fn fields(record: &Memory) -> Fields {
        record
            .fields
            .iter()
            .map(|v| match v {
                Value::Text(s) => ScanValue::Str(s.clone()),
                Value::Tags(v) => ScanValue::StrList(v.clone()),
                Value::Int(i) => ScanValue::I64(*i),
                Value::Bool(b) => ScanValue::Bool(*b),
            })
            .enumerate()
            .map(|(i, v)| (i as FieldRef, v))
            .collect()
    }
    fn get_from(state: &State, id: RecordId) -> Option<Fields> {
        state.slots.get(&id.0)?.record.as_deref().map(Self::fields)
    }
    fn current(state: &State, id: RecordId) -> Result<u64, ErrorCode> {
        let slot = state
            .slots
            .get(&id.0)
            .filter(|s| s.record.is_some())
            .ok_or(ErrorCode::RecordNotFound)?;
        Ok(slot.incarnation)
    }
    fn update(id: RecordId, incarnation: u64, value: i64) -> Change {
        Change::Update {
            id: id.0,
            incarnation,
            field: 10,
            value: Value::Int(value),
            equals: None,
        }
    }
    fn commit(engine: &mut MemoryEngine, changes: &[Change]) -> Result<(), ErrorCode> {
        let request = engine.log().next_request_id();
        committed(engine.transact(request, changes))
    }
}
impl Store for MemoryStore {
    fn describe_relations(&self) -> Vec<RelationDescriptor> {
        vec![RelationDescriptor {
            name: "mentions".into(),
            kind: JoinRelation::Neighbors(Some("mentions".into())),
            target_table: Some("entity".into()),
        }]
    }
    fn table_name(&self) -> &str {
        "memory"
    }
    fn describe(&self) -> DomainSchema {
        Self::schema()
    }
    fn get(&self, id: RecordId) -> Option<Fields> {
        let engine = self.0.lock().expect("memory engine mutex poisoned");
        Self::get_from(&engine.log().snapshot(), id)
    }
    fn scan_all(&self) -> Vec<PageRow> {
        let engine = self.0.lock().expect("memory engine mutex poisoned");
        engine
            .log()
            .snapshot()
            .slots
            .iter()
            .filter_map(|(id, s)| {
                s.record
                    .as_deref()
                    .map(|r| (RecordId(*id), Self::fields(r)))
            })
            .collect()
    }
    fn filter_eq(&self, field: FieldRef, value: &ScanValue) -> Result<Vec<RecordId>, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        if ![1].contains(&field) {
            return Err(ErrorCode::Unsupported);
        }
        if !matches!(value, ScanValue::Str(_)) {
            return Err(ErrorCode::Malformed);
        }
        Ok(engine
            .log()
            .snapshot()
            .slots
            .iter()
            .filter_map(|(id, slot)| {
                let fields = Self::fields(slot.record.as_deref()?);
                (fields[usize::from(field)].1 == *value).then_some(RecordId(*id))
            })
            .collect())
    }
    fn scan_field(&self, field: FieldRef) -> Result<Vec<ScanValue>, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        if field != 10 {
            return Err(ErrorCode::Unsupported);
        }
        Ok(engine
            .log()
            .snapshot()
            .slots
            .values()
            .filter_map(|s| {
                s.record
                    .as_deref()
                    .map(|r| Self::fields(r)[usize::from(field)].1.clone())
            })
            .collect())
    }
    fn update_field(
        &self,
        id: RecordId,
        field: FieldRef,
        value: ScanValue,
    ) -> Result<bool, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let value = integer_update(field, &value, 10, true)?;
        let state = engine.log().snapshot();
        let Ok(incarnation) = Self::current(&state, id) else {
            return Ok(false);
        };
        Self::commit(&mut engine, &[Self::update(id, incarnation, value)])?;
        Ok(true)
    }
    fn parent(&self, _id: RecordId) -> Result<ParentLookup, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn children(&self, _id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn validate_op(&self, op: &TransactionOp) -> Result<(), ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        integer_update(op.field, &op.value, 10, true)?;
        Self::current(&engine.log().snapshot(), op.id)?;
        Ok(())
    }
    fn apply_transaction(
        &self,
        updates: &[TransactionOp],
        read_set: &[(RecordId, FieldRef, ScanValue)],
    ) -> Result<(), (usize, ErrorCode)> {
        let mut engine = self.0.lock().map_err(|_| (0, ErrorCode::Storage))?;
        let state = engine.log().snapshot();
        check_reads(read_set, |id| Self::get_from(&state, id))?;
        if updates.len() > MAX_STAGED_OPS {
            return Err((MAX_STAGED_OPS, ErrorCode::SessionFull));
        }
        let changes = updates
            .iter()
            .enumerate()
            .map(|(i, op)| {
                let value = integer_update(op.field, &op.value, 10, true).map_err(|e| (i, e))?;
                let incarnation = Self::current(&state, op.id).map_err(|e| (i, e))?;
                Ok(Self::update(op.id, incarnation, value))
            })
            .collect::<Result<Vec<_>, (usize, ErrorCode)>>()?;
        if changes.is_empty() {
            return Ok(());
        }
        Self::commit(&mut engine, &changes).map_err(|e| (0, e))
    }
    fn insert_record(&self, id: RecordId, fields: Fields) -> Result<InsertOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let record = Box::new(Self::decode(id, fields)?);
        let state = engine.log().snapshot();
        if Self::current(&state, id).is_ok() {
            return Ok(InsertOutcome::Duplicate);
        }
        let incarnation = state
            .incarnation(&id.0)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ErrorCode::Storage)?;
        Self::commit(
            &mut engine,
            &[Change::Put {
                record,
                incarnation,
                insert: true,
            }],
        )?;
        Ok(InsertOutcome::Inserted)
    }
    fn replace_record(&self, id: RecordId, fields: Fields) -> Result<ReplaceOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let record = Box::new(Self::decode(id, fields)?);
        let Ok(incarnation) = Self::current(&engine.log().snapshot(), id) else {
            return Ok(ReplaceOutcome::NotFound);
        };
        Self::commit(
            &mut engine,
            &[Change::Put {
                record,
                incarnation,
                insert: false,
            }],
        )?;
        Ok(ReplaceOutcome::Replaced)
    }
    fn replace_record_if(
        &self,
        id: RecordId,
        fields: Fields,
        guard: &Predicate,
    ) -> Result<ReplaceIfOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        validate_predicate(&Self::schema(), guard)?;
        let record = Box::new(Self::decode(id, fields)?);
        let state = engine.log().snapshot();
        let Some(current) = Self::get_from(&state, id) else {
            return Ok(ReplaceIfOutcome::NotFound);
        };
        if !predicate_matches(&current, guard) {
            return Ok(ReplaceIfOutcome::GuardFailed);
        }
        let incarnation = Self::current(&state, id)?;
        Self::commit(
            &mut engine,
            &[Change::Put {
                record,
                incarnation,
                insert: false,
            }],
        )?;
        Ok(ReplaceIfOutcome::Replaced)
    }
    fn delete_record(&self, id: RecordId) -> Result<DeleteOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let Ok(incarnation) = Self::current(&engine.log().snapshot(), id) else {
            return Ok(DeleteOutcome::NotFound);
        };
        Self::commit(
            &mut engine,
            &[Change::Delete {
                id: id.0,
                incarnation,
            }],
        )?;
        Ok(DeleteOutcome::Deleted)
    }
    fn neighbors(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.neighbors_by_relation(id, "mentions")
    }
    fn neighbors_by_relation(
        &self,
        id: RecordId,
        relation: &str,
    ) -> Result<Vec<RecordId>, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        if relation != "mentions" {
            return Err(ErrorCode::Malformed);
        }
        let state = engine.log().snapshot();
        // Raw id collisions resolve to the live local record, never both directions.
        let local = Self::current(&state, id).is_ok();
        Ok(state
            .foreign_edges
            .iter()
            .filter_map(|(a, fi, b, ti)| {
                if state.current(a, *fi).is_err() || self.1.incarnation(RecordId(*b)) != Some(*ti) {
                    return None;
                }
                if local && *a == id.0 {
                    Some(RecordId(*b))
                } else if !local && *b == id.0 {
                    Some(RecordId(*a))
                } else {
                    None
                }
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect())
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        vec!["mentions".into()]
    }
    fn count_edges(&self, relation: &str) -> Result<u64, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        if relation != "mentions" {
            return Err(ErrorCode::Malformed);
        }
        let state = engine.log().snapshot();
        Ok(state
            .foreign_edges
            .iter()
            .filter(|(a, fi, b, ti)| {
                state.current(a, *fi).is_ok() && self.1.incarnation(RecordId(*b)) == Some(*ti)
            })
            .count() as u64)
    }
    fn detach_record(&self, relation: &str, id: RecordId) -> Result<usize, ErrorCode> {
        if relation != "mentions" {
            return Err(ErrorCode::Malformed);
        }
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let count = engine
            .log()
            .snapshot()
            .foreign_edges
            .iter()
            .filter(|(_, _, to, _)| *to == id.0)
            .count();
        Self::commit(&mut engine, &[Change::DetachForeign { to: id.0 }])?;
        Ok(count)
    }
    fn link_records(
        &self,
        left: RecordId,
        right: RecordId,
        relation: &str,
    ) -> Result<LinkOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        if relation != "mentions" {
            return Err(ErrorCode::Malformed);
        }
        let state = engine.log().snapshot();
        let from_incarnation = Self::current(&state, left)?;
        if left.0 == right.0 {
            return Err(ErrorCode::Malformed);
        }
        let to_incarnation = self.1.incarnation(right).ok_or(ErrorCode::RecordNotFound)?;
        if state
            .foreign_edges
            .contains(&(left.0, from_incarnation, right.0, to_incarnation))
        {
            return Ok(LinkOutcome::AlreadyLinked);
        }
        Self::commit(
            &mut engine,
            &[Change::LinkForeign {
                from: left.0,
                from_incarnation,
                to: right.0,
                to_incarnation,
            }],
        )?;
        Ok(LinkOutcome::Linked)
    }
}
