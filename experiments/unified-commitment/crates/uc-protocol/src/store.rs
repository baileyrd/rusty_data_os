use crate::*;

/// Adapter contract. Validation and commit/read-set checking must share an exclusive
/// adapter section. The facade supplies registry coordination, not adapter atomicity.
/// Metadata (table name, schema, foreign relation targets) must remain stable while
/// registered. Call mutations through the shared Registry to preserve foreign edges.
pub trait Store: Send + Sync {
    fn get(&self, id: RecordId) -> Option<Fields>;
    fn filter_eq(&self, field: FieldRef, value: &ScanValue) -> Result<Vec<RecordId>, ErrorCode>;
    fn scan_field(&self, field: FieldRef) -> Result<Vec<ScanValue>, ErrorCode>;
    fn update_field(
        &self,
        id: RecordId,
        field: FieldRef,
        value: ScanValue,
    ) -> Result<bool, ErrorCode>;
    fn parent(&self, id: RecordId) -> Result<ParentLookup, ErrorCode>;
    fn children(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode>;
    fn neighbors(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode>;
    fn neighbors_by_relation(
        &self,
        id: RecordId,
        relation: &str,
    ) -> Result<Vec<RecordId>, ErrorCode>;
    fn list_relation_kinds(&self) -> Vec<String>;
    fn describe(&self) -> DomainSchema;
    fn validate_op(&self, op: &TransactionOp) -> Result<(), ErrorCode>;
    fn scan_all(&self) -> Vec<PageRow>;
    /// Validate the original tracked values before write preconditions; mismatch is
    /// (0, Conflict). A precondition error names its first update index. Apply nothing
    /// on either refusal. All checks and writes must be atomic under adapter locking.
    fn apply_transaction(
        &self,
        updates: &[TransactionOp],
        read_set: &[(RecordId, FieldRef, ScanValue)],
    ) -> Result<(), (usize, ErrorCode)>;

    fn describe_relations(&self) -> Vec<RelationDescriptor> {
        default_relation_descriptors(&self.describe(), self.list_relation_kinds())
    }
    fn table_name(&self) -> &str {
        "table"
    }
    fn insert_record(&self, _id: RecordId, _fields: Fields) -> Result<InsertOutcome, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn link_records(
        &self,
        _left: RecordId,
        _right: RecordId,
        _relation: &str,
    ) -> Result<LinkOutcome, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn replace_record(&self, _id: RecordId, _fields: Fields) -> Result<ReplaceOutcome, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    /// Guard evaluation and replacement must be one atomic adapter operation.
    fn replace_record_if(
        &self,
        _id: RecordId,
        _fields: Fields,
        _guard: &Predicate,
    ) -> Result<ReplaceIfOutcome, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn delete_record(&self, _id: RecordId) -> Result<DeleteOutcome, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn detach_record(&self, _relation: &str, _id: RecordId) -> Result<usize, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    /// Current live incarnation, or None when absent or unsupported by this table.
    fn incarnation(&self, _id: RecordId) -> Option<u64> {
        None
    }
    fn compact(&self) -> Result<CompactionReport, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn count_edges(&self, _relation: &str) -> Result<u64, ErrorCode> {
        Err(ErrorCode::Unsupported)
    }
    fn page_keys(&self, order_by: FieldRef) -> Vec<(RecordId, i128)> {
        self.scan_all()
            .into_iter()
            .map(|(id, fields)| (id, page_key(&fields, order_by, id).0))
            .collect()
    }
    fn page(
        &self,
        order_by: FieldRef,
        after: Option<(ScanValue, RecordId)>,
        limit: usize,
    ) -> Result<Vec<PageRow>, ErrorCode> {
        Ok(page_by_scan(self, order_by, after, limit))
    }
    fn apply_write_op(&self, op: &WriteOp) -> WriteResult {
        let result = match op {
            WriteOp::Insert { id, fields } => {
                self.insert_record(*id, fields.clone()).map(|v| match v {
                    InsertOutcome::Inserted => WriteResult::Inserted,
                    InsertOutcome::Duplicate => WriteResult::Duplicate,
                })
            }
            WriteOp::Replace { id, fields } => {
                self.replace_record(*id, fields.clone()).map(|v| match v {
                    ReplaceOutcome::Replaced => WriteResult::Replaced,
                    ReplaceOutcome::NotFound => WriteResult::NotFound,
                })
            }
            WriteOp::ReplaceIf { id, fields, guard } => {
                if let Err(code) = validate_predicate(&self.describe(), guard) {
                    return WriteResult::Failed(code);
                }
                self.replace_record_if(*id, fields.clone(), guard)
                    .map(|v| match v {
                        ReplaceIfOutcome::Replaced => WriteResult::Replaced,
                        ReplaceIfOutcome::NotFound => WriteResult::NotFound,
                        ReplaceIfOutcome::GuardFailed => WriteResult::GuardFailed,
                    })
            }
            WriteOp::Delete { id } => self.delete_record(*id).map(|v| match v {
                DeleteOutcome::Deleted => WriteResult::Deleted,
                DeleteOutcome::NotFound => WriteResult::NotFound,
            }),
            WriteOp::Link {
                left,
                right,
                relation,
            } => self.link_records(*left, *right, relation).map(|v| match v {
                LinkOutcome::Linked => WriteResult::Linked,
                LinkOutcome::AlreadyLinked => WriteResult::AlreadyLinked,
            }),
        };
        result.unwrap_or_else(WriteResult::Failed)
    }
    /// Table-local entry point. Served writes additionally check/detach across Registry.
    fn write_batch(
        &self,
        ops: &[WriteOp],
        atomic: bool,
    ) -> Result<Vec<WriteResult>, (usize, ErrorCode)> {
        if atomic {
            self.write_batch_checked(ops, &|_| Ok(()))
        } else {
            Ok(ops.iter().map(|op| self.apply_write_op(op)).collect())
        }
    }
    /// Run check(i) before local validation of op i, in operation order, before
    /// publishing any writes. Caller holds other tables stable until return.
    /// Default refuses nonempty atomic batches without writing; empty succeeds.
    fn write_batch_checked(
        &self,
        ops: &[WriteOp],
        _check: &dyn Fn(usize) -> Result<(), ErrorCode>,
    ) -> Result<Vec<WriteResult>, (usize, ErrorCode)> {
        if ops.is_empty() {
            Ok(Vec::new())
        } else {
            Err((0, ErrorCode::Unsupported))
        }
    }
}
