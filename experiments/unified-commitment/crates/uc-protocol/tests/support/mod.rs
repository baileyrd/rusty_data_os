#![allow(dead_code)]
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{self, Read, Write},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::Duration,
};
use uc_protocol::{codec::*, framing::*, *};
pub fn id(n: u128) -> RecordId {
    RecordId::from_u128(n)
}
pub fn fields(n: i64) -> Fields {
    vec![(1, ScanValue::I64(n))]
}
pub fn op(n: u128, v: i64) -> TransactionOp {
    TransactionOp {
        id: id(n),
        field: 1,
        value: ScanValue::I64(v),
    }
}
pub fn update(n: u128, v: i64) -> Request {
    Request::UpdateField {
        id: id(n),
        field: 1,
        value: ScanValue::I64(v),
    }
}
pub fn guard(n: i64) -> Predicate {
    Predicate {
        field: 1,
        op: CompareOp::Eq,
        value: ScanValue::I64(n),
    }
}
pub fn link(a: u128, b: u128) -> Request {
    Request::Link {
        left: id(a),
        right: id(b),
        relation: "edge".into(),
    }
}
pub fn batch_link(a: u128, b: u128) -> WriteOp {
    WriteOp::Link {
        left: id(a),
        right: id(b),
        relation: "edge".into(),
    }
}
pub fn assert_error(response: Response, code: ErrorCode) {
    assert_eq!(response, err_response(code));
}
pub fn assert_failed(response: Response, index: usize, code: ErrorCode) {
    assert_eq!(
        response,
        Response::TransactionFailed {
            index,
            code,
            message: error_message(code).into()
        }
    );
}
pub fn schema() -> DomainSchema {
    DomainSchema {
        fields: vec![FieldDescriptor {
            tag: 1,
            name: "number".into(),
            value_kind: ValueKind::I64,
            capabilities: FieldCapabilities {
                filter_eq: true,
                scan: true,
                update: true,
            },
        }],
        relations: RelationCapabilities {
            parent_children: false,
            neighbors: true,
        },
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    pub rows: BTreeMap<RecordId, Fields>,
    pub edges: BTreeSet<(RecordId, RecordId)>,
    pub writes: usize,
    pub transaction_sizes: Vec<usize>,
    pub read_sets: Vec<ReadSet>,
    pub batch_checks: Vec<usize>,
    pub detaches: Vec<RecordId>,
}
type Observer = Arc<dyn Fn(&str) + Send + Sync>;
/// Deliberately test-only store. Mutex-protected map snapshots implement commit
/// independently of protocol dispatch; no core/domain adapter is used here.
pub struct TestStore {
    pub name: String,
    pub target: Option<String>,
    pub state: Mutex<State>,
    pub detach_error: Mutex<Option<ErrorCode>>,
    pub observer: Mutex<Option<Observer>>,
    pub schema: DomainSchema,
}
impl TestStore {
    pub fn new(name: &str, target: Option<&str>, rows: &[(u128, i64)]) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            target: target.map(str::to_owned),
            state: Mutex::new(State {
                rows: rows.iter().map(|(n, v)| (id(*n), fields(*v))).collect(),
                ..State::default()
            }),
            detach_error: Mutex::new(None),
            observer: Mutex::new(None),
            schema: schema(),
        })
    }
    pub fn snapshot(&self) -> State {
        self.state.lock().unwrap().clone()
    }
    fn observe(&self, event: &str) {
        let callback = self.observer.lock().unwrap().clone();
        if let Some(callback) = callback {
            callback(event);
        }
    }
    fn validate(state: &State, op: &TransactionOp) -> Result<(), ErrorCode> {
        if !state.rows.contains_key(&op.id) {
            return Err(ErrorCode::RecordNotFound);
        }
        if op.field != 1 {
            return Err(ErrorCode::UnknownField);
        }
        if !matches!(op.value, ScanValue::I64(_)) {
            return Err(ErrorCode::Malformed);
        }
        Ok(())
    }
    fn validate_fields(fields: &Fields) -> Result<(), ErrorCode> {
        if fields.iter().any(|(f, _)| *f != 1) {
            return Err(ErrorCode::UnknownField);
        }
        if fields.len() != 1 || !matches!(fields[0].1, ScanValue::I64(_)) {
            return Err(ErrorCode::Malformed);
        }
        Ok(())
    }
    fn apply(state: &mut State, op: &WriteOp, foreign: bool) -> WriteResult {
        let result = match op {
            WriteOp::Insert { id, fields } => {
                if let Err(e) = Self::validate_fields(fields) {
                    return WriteResult::Failed(e);
                }
                if state.rows.contains_key(id) {
                    return WriteResult::Duplicate;
                }
                state.rows.insert(*id, fields.clone());
                WriteResult::Inserted
            }
            WriteOp::Replace { id, fields } | WriteOp::ReplaceIf { id, fields, .. } => {
                if let Err(e) = Self::validate_fields(fields) {
                    return WriteResult::Failed(e);
                }
                let Some(old) = state.rows.get_mut(id) else {
                    return WriteResult::NotFound;
                };
                if let WriteOp::ReplaceIf { guard, .. } = op {
                    if let Err(e) = validate_predicate(&schema(), guard) {
                        return WriteResult::Failed(e);
                    }
                    if !predicate_matches(old, guard) {
                        return WriteResult::GuardFailed;
                    }
                }
                *old = fields.clone();
                WriteResult::Replaced
            }
            WriteOp::Delete { id } => {
                if state.rows.remove(id).is_none() {
                    return WriteResult::NotFound;
                }
                state.edges.retain(|(a, b)| a != id && b != id);
                WriteResult::Deleted
            }
            WriteOp::Link {
                left,
                right,
                relation,
            } => {
                if relation != "edge" {
                    return WriteResult::Failed(ErrorCode::Malformed);
                }
                if !state.rows.contains_key(left) || (!foreign && !state.rows.contains_key(right)) {
                    return WriteResult::Failed(ErrorCode::RecordNotFound);
                }
                if !state.edges.insert((*left, *right)) {
                    return WriteResult::AlreadyLinked;
                }
                WriteResult::Linked
            }
        };
        state.writes += 1;
        result
    }
    fn single(&self, op: WriteOp) -> WriteResult {
        self.observe(match op {
            WriteOp::Link { .. } => "link",
            WriteOp::Delete { .. } => "delete",
            _ => "write",
        });
        Self::apply(&mut self.state.lock().unwrap(), &op, self.target.is_some())
    }
}
impl Store for TestStore {
    fn table_name(&self) -> &str {
        &self.name
    }
    fn get(&self, id: RecordId) -> Option<Fields> {
        let fields = self.state.lock().unwrap().rows.get(&id).cloned();
        self.observe("get");
        fields
    }
    fn filter_eq(&self, field: FieldRef, value: &ScanValue) -> Result<Vec<RecordId>, ErrorCode> {
        if field != 1 {
            return Err(ErrorCode::UnknownField);
        }
        if !matches!(value, ScanValue::I64(_)) {
            return Err(ErrorCode::Malformed);
        }
        Ok(self
            .state
            .lock()
            .unwrap()
            .rows
            .iter()
            .filter(|(_, f)| f.contains(&(field, value.clone())))
            .map(|(id, _)| *id)
            .collect())
    }
    fn scan_field(&self, field: FieldRef) -> Result<Vec<ScanValue>, ErrorCode> {
        if field != 1 {
            return Err(ErrorCode::UnknownField);
        }
        Ok(self
            .state
            .lock()
            .unwrap()
            .rows
            .values()
            .flat_map(|f| {
                f.iter()
                    .filter(|(tag, _)| *tag == field)
                    .map(|(_, v)| v.clone())
            })
            .collect())
    }
    fn update_field(
        &self,
        id: RecordId,
        field: FieldRef,
        value: ScanValue,
    ) -> Result<bool, ErrorCode> {
        let mut state = self.state.lock().unwrap();
        if !state.rows.contains_key(&id) {
            return Ok(false);
        }
        Self::validate(
            &state,
            &TransactionOp {
                id,
                field,
                value: value.clone(),
            },
        )?;
        state.rows.insert(id, vec![(field, value)]);
        state.writes += 1;
        Ok(true)
    }
    fn parent(&self, id: RecordId) -> Result<ParentLookup, ErrorCode> {
        Ok(if self.state.lock().unwrap().rows.contains_key(&id) {
            ParentLookup::NoParent
        } else {
            ParentLookup::ChildNotFound
        })
    }
    fn children(&self, _id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        Ok(vec![])
    }
    fn neighbors(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.neighbors_by_relation(id, "edge")
    }
    fn neighbors_by_relation(
        &self,
        id: RecordId,
        relation: &str,
    ) -> Result<Vec<RecordId>, ErrorCode> {
        if relation != "edge" {
            return Err(ErrorCode::Malformed);
        }
        Ok(self
            .state
            .lock()
            .unwrap()
            .edges
            .iter()
            .filter(|(a, _)| *a == id)
            .map(|(_, b)| *b)
            .collect())
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        vec!["edge".into()]
    }
    fn describe(&self) -> DomainSchema {
        self.schema.clone()
    }
    fn describe_relations(&self) -> Vec<RelationDescriptor> {
        vec![RelationDescriptor {
            name: "edge".into(),
            kind: JoinRelation::Neighbors(Some("edge".into())),
            target_table: self.target.clone(),
        }]
    }
    fn scan_all(&self) -> Vec<PageRow> {
        self.state
            .lock()
            .unwrap()
            .rows
            .iter()
            .map(|(id, f)| (*id, f.clone()))
            .collect()
    }
    fn validate_op(&self, op: &TransactionOp) -> Result<(), ErrorCode> {
        Self::validate(&self.state.lock().unwrap(), op)
    }
    fn apply_transaction(
        &self,
        updates: &[TransactionOp],
        read_set: &[(RecordId, FieldRef, ScanValue)],
    ) -> Result<(), (usize, ErrorCode)> {
        let mut state = self.state.lock().unwrap();
        for (id, field, value) in read_set {
            if !state
                .rows
                .get(id)
                .is_some_and(|f| f.contains(&(*field, value.clone())))
            {
                return Err((0, ErrorCode::Conflict));
            }
        }
        for (i, op) in updates.iter().enumerate() {
            Self::validate(&state, op).map_err(|e| (i, e))?;
        }
        for op in updates {
            state.rows.insert(op.id, vec![(op.field, op.value.clone())]);
            state.writes += 1;
        }
        state.transaction_sizes.push(updates.len());
        state.read_sets.push(read_set.to_vec());
        Ok(())
    }
    fn insert_record(&self, id: RecordId, fields: Fields) -> Result<InsertOutcome, ErrorCode> {
        match self.single(WriteOp::Insert { id, fields }) {
            WriteResult::Inserted => Ok(InsertOutcome::Inserted),
            WriteResult::Duplicate => Ok(InsertOutcome::Duplicate),
            WriteResult::Failed(e) => Err(e),
            other => panic!("{other:?}"),
        }
    }
    fn replace_record(&self, id: RecordId, fields: Fields) -> Result<ReplaceOutcome, ErrorCode> {
        match self.single(WriteOp::Replace { id, fields }) {
            WriteResult::Replaced => Ok(ReplaceOutcome::Replaced),
            WriteResult::NotFound => Ok(ReplaceOutcome::NotFound),
            WriteResult::Failed(e) => Err(e),
            other => panic!("{other:?}"),
        }
    }
    fn replace_record_if(
        &self,
        id: RecordId,
        fields: Fields,
        guard: &Predicate,
    ) -> Result<ReplaceIfOutcome, ErrorCode> {
        match self.single(WriteOp::ReplaceIf {
            id,
            fields,
            guard: guard.clone(),
        }) {
            WriteResult::Replaced => Ok(ReplaceIfOutcome::Replaced),
            WriteResult::NotFound => Ok(ReplaceIfOutcome::NotFound),
            WriteResult::GuardFailed => Ok(ReplaceIfOutcome::GuardFailed),
            WriteResult::Failed(e) => Err(e),
            other => panic!("{other:?}"),
        }
    }
    fn link_records(
        &self,
        left: RecordId,
        right: RecordId,
        relation: &str,
    ) -> Result<LinkOutcome, ErrorCode> {
        match self.single(WriteOp::Link {
            left,
            right,
            relation: relation.into(),
        }) {
            WriteResult::Linked => Ok(LinkOutcome::Linked),
            WriteResult::AlreadyLinked => Ok(LinkOutcome::AlreadyLinked),
            WriteResult::Failed(e) => Err(e),
            other => panic!("{other:?}"),
        }
    }
    fn delete_record(&self, id: RecordId) -> Result<DeleteOutcome, ErrorCode> {
        match self.single(WriteOp::Delete { id }) {
            WriteResult::Deleted => Ok(DeleteOutcome::Deleted),
            WriteResult::NotFound => Ok(DeleteOutcome::NotFound),
            WriteResult::Failed(e) => Err(e),
            other => panic!("{other:?}"),
        }
    }
    fn detach_record(&self, _relation: &str, id: RecordId) -> Result<usize, ErrorCode> {
        self.observe("detach");
        if let Some(e) = *self.detach_error.lock().unwrap() {
            return Err(e);
        }
        let mut state = self.state.lock().unwrap();
        let before = state.edges.len();
        state.edges.retain(|(_, b)| *b != id);
        state.detaches.push(id);
        Ok(before - state.edges.len())
    }
    fn compact(&self) -> Result<CompactionReport, ErrorCode> {
        self.observe("compact");
        Ok(CompactionReport {
            records: self.state.lock().unwrap().rows.len(),
            slots_reclaimed: 0,
            log_entries_folded: 0,
            edge_logs_folded: 0,
        })
    }
    fn count_edges(&self, relation: &str) -> Result<u64, ErrorCode> {
        if relation != "edge" {
            return Err(ErrorCode::Malformed);
        }
        Ok(self.state.lock().unwrap().edges.len() as u64)
    }
    fn write_batch_checked(
        &self,
        ops: &[WriteOp],
        check: &dyn Fn(usize) -> Result<(), ErrorCode>,
    ) -> Result<Vec<WriteResult>, (usize, ErrorCode)> {
        self.observe("batch");
        let mut state = self.state.lock().unwrap();
        let mut candidate = state.clone();
        let mut results = Vec::new();
        for (i, op) in ops.iter().enumerate() {
            check(i).map_err(|e| (i, e))?;
            candidate.batch_checks.push(i);
            let result = Self::apply(&mut candidate, op, self.target.is_some());
            if let WriteResult::Failed(e) = result {
                return Err((i, e));
            }
            results.push(result);
        }
        *state = candidate;
        Ok(results)
    }
}
/// Only required methods are forwarded; all optional methods exercise actual defaults.
pub struct DefaultStore(pub Arc<TestStore>);
impl Store for DefaultStore {
    fn get(&self, id: RecordId) -> Option<Fields> {
        self.0.get(id)
    }
    fn filter_eq(&self, f: FieldRef, v: &ScanValue) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.filter_eq(f, v)
    }
    fn scan_field(&self, f: FieldRef) -> Result<Vec<ScanValue>, ErrorCode> {
        self.0.scan_field(f)
    }
    fn update_field(&self, id: RecordId, f: FieldRef, v: ScanValue) -> Result<bool, ErrorCode> {
        self.0.update_field(id, f, v)
    }
    fn parent(&self, id: RecordId) -> Result<ParentLookup, ErrorCode> {
        self.0.parent(id)
    }
    fn children(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.children(id)
    }
    fn neighbors(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.neighbors(id)
    }
    fn neighbors_by_relation(&self, id: RecordId, r: &str) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.neighbors_by_relation(id, r)
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        self.0.list_relation_kinds()
    }
    fn describe(&self) -> DomainSchema {
        self.0.describe()
    }
    fn validate_op(&self, op: &TransactionOp) -> Result<(), ErrorCode> {
        self.0.validate_op(op)
    }
    fn scan_all(&self) -> Vec<PageRow> {
        self.0.scan_all()
    }
    fn apply_transaction(
        &self,
        ops: &[TransactionOp],
        reads: &[(RecordId, FieldRef, ScanValue)],
    ) -> Result<(), (usize, ErrorCode)> {
        self.0.apply_transaction(ops, reads)
    }
}
pub fn registry(stores: Vec<Arc<TestStore>>, primary: usize) -> Arc<Registry> {
    Arc::new(
        Registry::new(
            stores
                .into_iter()
                .map(|s| (s.name.clone(), s as Arc<dyn Store>))
                .collect(),
            primary,
        )
        .unwrap(),
    )
}
/// Two independent byte directions, no socket or shared read/write cursor.
pub struct Pipe {
    rx: mpsc::Receiver<Vec<u8>>,
    tx: mpsc::Sender<Vec<u8>>,
    pending: VecDeque<u8>,
}
pub fn duplex() -> (Pipe, Pipe) {
    let (at, ar) = mpsc::channel();
    let (bt, br) = mpsc::channel();
    (
        Pipe {
            rx: ar,
            tx: bt,
            pending: VecDeque::new(),
        },
        Pipe {
            rx: br,
            tx: at,
            pending: VecDeque::new(),
        },
    )
}
impl Read for Pipe {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        if b.is_empty() {
            return Ok(0);
        }
        while self.pending.is_empty() {
            match self.rx.recv_timeout(Duration::from_secs(10)) {
                Ok(bytes) => self.pending.extend(bytes),
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(0),
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "test duplex stalled",
                    ));
                }
            }
        }
        let count = b.len().min(self.pending.len());
        for byte in &mut b[..count] {
            *byte = self.pending.pop_front().unwrap();
        }
        Ok(count)
    }
}
impl Write for Pipe {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.tx
            .send(b.to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "test peer closed"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub struct Client {
    pub pipe: Pipe,
    task: JoinHandle<io::Result<()>>,
}
impl Client {
    pub fn raw(registry: Arc<Registry>) -> Self {
        let (client, mut server) = duplex();
        let task = thread::spawn(move || handle_connection(&mut server, &registry));
        Self { pipe: client, task }
    }
    pub fn new(registry: Arc<Registry>) -> Self {
        let mut c = Self::raw(registry);
        assert_eq!(
            c.request(Request::Hello {
                protocol_version: 22
            }),
            Response::Hello {
                protocol_version: 22
            }
        );
        c
    }
    pub fn request(&mut self, req: Request) -> Response {
        self.send(req);
        self.receive()
    }
    pub fn send(&mut self, req: Request) {
        write_message(&mut self.pipe, &encode_request(&req)).unwrap();
    }
    pub fn receive(&mut self) -> Response {
        decode_response(&read_message(&mut self.pipe).unwrap()).unwrap()
    }
    pub fn close(self) {
        drop(self.pipe);
        self.task.join().unwrap().unwrap();
    }
}

pub struct InsertOnly(pub Arc<TestStore>);
impl Store for InsertOnly {
    fn get(&self, id: RecordId) -> Option<Fields> {
        self.0.get(id)
    }
    fn filter_eq(&self, f: FieldRef, v: &ScanValue) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.filter_eq(f, v)
    }
    fn scan_field(&self, f: FieldRef) -> Result<Vec<ScanValue>, ErrorCode> {
        self.0.scan_field(f)
    }
    fn update_field(&self, id: RecordId, f: FieldRef, v: ScanValue) -> Result<bool, ErrorCode> {
        self.0.update_field(id, f, v)
    }
    fn parent(&self, id: RecordId) -> Result<ParentLookup, ErrorCode> {
        self.0.parent(id)
    }
    fn children(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.children(id)
    }
    fn neighbors(&self, id: RecordId) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.neighbors(id)
    }
    fn neighbors_by_relation(&self, id: RecordId, r: &str) -> Result<Vec<RecordId>, ErrorCode> {
        self.0.neighbors_by_relation(id, r)
    }
    fn list_relation_kinds(&self) -> Vec<String> {
        self.0.list_relation_kinds()
    }
    fn describe(&self) -> DomainSchema {
        self.0.describe()
    }
    fn validate_op(&self, op: &TransactionOp) -> Result<(), ErrorCode> {
        self.0.validate_op(op)
    }
    fn scan_all(&self) -> Vec<PageRow> {
        self.0.scan_all()
    }
    fn apply_transaction(
        &self,
        ops: &[TransactionOp],
        reads: &[(RecordId, FieldRef, ScanValue)],
    ) -> Result<(), (usize, ErrorCode)> {
        self.0.apply_transaction(ops, reads)
    }
    fn insert_record(&self, id: RecordId, fields: Fields) -> Result<InsertOutcome, ErrorCode> {
        self.0.insert_record(id, fields)
    }
}
