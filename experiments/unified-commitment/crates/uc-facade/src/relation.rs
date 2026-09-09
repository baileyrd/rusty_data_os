use crate::*;
use std::sync::Mutex;
use uc_relation::Relation;
use uc_relation::{Change, RelationEngine, State};

/// Owns one independent engine and holds exclusive access across checks and commit.
pub struct RelationStore(Mutex<RelationEngine>);
impl RelationStore {
    pub fn new(engine: RelationEngine) -> Self {
        Self(Mutex::new(engine))
    }
    fn schema() -> DomainSchema {
        schema(
            &[
                ("subject", ValueKind::Str),
                ("relation", ValueKind::Str),
                ("object", ValueKind::Str),
                ("created_at_unix_ms", ValueKind::I64),
                ("updated_at_unix_ms", ValueKind::I64),
                ("node_id", ValueKind::Str),
                ("deleted_at_unix_ms", ValueKind::I64),
            ],
            &[0],
            4,
            false,
        )
    }
    fn decode(id: RecordId, fields: Fields) -> Result<Relation, ErrorCode> {
        let values = decode_fields(fields, &Self::schema())?;
        Ok(Relation {
            id: uc_core::Uuid(id.0),
            subject: text(&values[0]),
            relation: text(&values[1]),
            object: text(&values[2]),
            created_at_unix_ms: int(&values[3]),
            updated_at_unix_ms: int(&values[4]),
            node_id: text(&values[5]),
            deleted_at_unix_ms: int(&values[6]),
        })
    }
    fn fields(record: &Relation) -> Fields {
        vec![
            (0, ScanValue::Str(record.subject.clone())),
            (1, ScanValue::Str(record.relation.clone())),
            (2, ScanValue::Str(record.object.clone())),
            (3, ScanValue::I64(record.created_at_unix_ms)),
            (4, ScanValue::I64(record.updated_at_unix_ms)),
            (5, ScanValue::Str(record.node_id.clone())),
            (6, ScanValue::I64(record.deleted_at_unix_ms)),
        ]
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
        Change::UpdateTimestamp {
            id: id.0,
            incarnation,
            value,
        }
    }
    fn commit(engine: &mut RelationEngine, changes: &[Change]) -> Result<(), ErrorCode> {
        let request = engine.log().next_request_id();
        committed(engine.transact(request, changes))
    }
}
impl Store for RelationStore {
    fn table_name(&self) -> &str {
        "relation"
    }
    fn describe(&self) -> DomainSchema {
        Self::schema()
    }
    fn get(&self, id: RecordId) -> Option<Fields> {
        let engine = self.0.lock().expect("relation engine mutex poisoned");
        Self::get_from(&engine.log().snapshot(), id)
    }
    fn scan_all(&self) -> Vec<PageRow> {
        let engine = self.0.lock().expect("relation engine mutex poisoned");
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
        if ![0].contains(&field) {
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
        if field != 4 {
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
        let value = integer_update(field, &value, 4, false)?;
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
        integer_update(op.field, &op.value, 4, false)?;
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
                let value = integer_update(op.field, &op.value, 4, false).map_err(|e| (i, e))?;
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
    fn neighbors(&self, _id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn neighbors_by_relation(
        &self,
        _id: RecordId,
        _relation: &str,
    ) -> Result<Vec<RecordId>, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        vec![]
    }
}
