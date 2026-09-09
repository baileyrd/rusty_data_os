use crate::*;
use std::sync::Mutex;
use uc_entity::Entity;
use uc_entity::{Change, EntityEngine, State};

/// Owns one independent engine and holds exclusive access across checks and commit.
pub struct EntityStore(Mutex<EntityEngine>);
impl EntityStore {
    pub fn new(engine: EntityEngine) -> Self {
        Self(Mutex::new(engine))
    }
    fn schema() -> DomainSchema {
        schema(
            &[
                ("label", ValueKind::Str),
                ("kind", ValueKind::Str),
                ("mention_count", ValueKind::I64),
                ("aliases", ValueKind::StrList),
            ],
            &[0, 1],
            2,
            true,
        )
    }
    fn decode(id: RecordId, fields: Fields) -> Result<Entity, ErrorCode> {
        let values = decode_fields(fields, &Self::schema())?;
        Ok(Entity {
            id: uc_core::Uuid(id.0),
            label: text(&values[0]),
            kind: text(&values[1]),
            mention_count: int(&values[2]),
            aliases: strings(&values[3]),
        })
    }
    fn fields(record: &Entity) -> Fields {
        vec![
            (0, ScanValue::Str(record.label.clone())),
            (1, ScanValue::Str(record.kind.clone())),
            (2, ScanValue::I64(record.mention_count)),
            (3, ScanValue::StrList(record.aliases.clone())),
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
        Change::UpdateMentionCount {
            id: id.0,
            incarnation,
            value,
        }
    }
    fn commit(engine: &mut EntityEngine, changes: &[Change]) -> Result<(), ErrorCode> {
        let request = engine.log().next_request_id();
        committed(engine.transact(request, changes))
    }
}
impl Store for EntityStore {
    fn table_name(&self) -> &str {
        "entity"
    }
    fn describe(&self) -> DomainSchema {
        Self::schema()
    }
    fn get(&self, id: RecordId) -> Option<Fields> {
        let engine = self.0.lock().expect("entity engine mutex poisoned");
        Self::get_from(&engine.log().snapshot(), id)
    }
    fn scan_all(&self) -> Vec<PageRow> {
        let engine = self.0.lock().expect("entity engine mutex poisoned");
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
        if ![0, 1].contains(&field) {
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
        if field != 2 {
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
        let value = integer_update(field, &value, 2, false)?;
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
        integer_update(op.field, &op.value, 2, false)?;
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
                let value = integer_update(op.field, &op.value, 2, false).map_err(|e| (i, e))?;
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
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        Ok(engine
            .log()
            .snapshot()
            .neighbors(&id.0)
            .into_iter()
            .map(RecordId)
            .collect())
    }
    fn neighbors_by_relation(
        &self,
        id: RecordId,
        relation: &str,
    ) -> Result<Vec<RecordId>, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        engine
            .log()
            .snapshot()
            .neighbors_by_relation(relation, &id.0)
            .map(|v| v.into_iter().map(RecordId).collect())
            .map_err(|_| ErrorCode::Malformed)
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        let engine = self.0.lock().expect("entity engine mutex poisoned");
        engine.log().snapshot().list_relation_kinds()
    }
    fn count_edges(&self, relation: &str) -> Result<u64, ErrorCode> {
        let engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let state = engine.log().snapshot();
        if !state.known_labels.contains(relation) {
            return Err(ErrorCode::Malformed);
        }
        Ok(state
            .edges
            .iter()
            .filter(|(_, _, label, _, _)| label == relation)
            .count() as u64)
    }
    fn link_records(
        &self,
        left: RecordId,
        right: RecordId,
        relation: &str,
    ) -> Result<LinkOutcome, ErrorCode> {
        let mut engine = self.0.lock().map_err(|_| ErrorCode::Storage)?;
        let state = engine.log().snapshot();
        // The engine alone validates open-label syntax and rejects self loops.
        let left_incarnation = state.incarnation(&left.0).unwrap_or(0);
        let right_incarnation = state.incarnation(&right.0).unwrap_or(0);
        if state
            .neighbors_by_relation(relation, &left.0)
            .is_ok_and(|ids| ids.contains(&right.0))
        {
            return Ok(LinkOutcome::AlreadyLinked);
        }
        Self::commit(
            &mut engine,
            &[Change::Link {
                relation: relation.into(),
                left: left.0,
                left_incarnation,
                right: right.0,
                right_incarnation,
            }],
        )?;
        Ok(LinkOutcome::Linked)
    }
}
