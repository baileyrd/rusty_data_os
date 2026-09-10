use crate::*;
use crate::{codec::*, dispatch::transaction_failed, framing::*, query::evaluate_join};
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

/// Immutable registration order and shared synchronization for all its connections.
/// Construct once, then borrow or clone this registry; clones share the same mutex.
/// Foreign relation declarations and table names must remain stable (Store contract).
#[derive(Clone)]
pub struct Registry {
    tables: Vec<(String, Arc<dyn Store>)>,
    relationship_lock: Option<Arc<Mutex<()>>>,
    primary: usize,
}
impl Registry {
    /// Names must uniquely match Store::table_name, as required by detach routing.
    pub fn new(tables: Vec<(String, Arc<dyn Store>)>, primary: usize) -> Result<Self, String> {
        if primary >= tables.len() {
            return Err("registry needs a valid primary table".into());
        }
        for (i, (name, store)) in tables.iter().enumerate() {
            if name.is_empty()
                || name != store.table_name()
                || tables[..i].iter().any(|(n, _)| n == name)
            {
                return Err(
                    "registry names must be unique, nonempty and match Store::table_name".into(),
                );
            }
        }
        let foreign = tables.iter().any(|(_, s)| {
            s.describe_relations()
                .iter()
                .any(|r| r.target_table.is_some())
        });
        Ok(Self {
            tables,
            relationship_lock: foreign.then(|| Arc::new(Mutex::new(()))),
            primary,
        })
    }
    pub fn has_relationship_lock(&self) -> bool {
        self.relationship_lock.is_some()
    }
    fn find(&self, name: &str) -> Option<&dyn Store> {
        self.tables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, s)| s.as_ref())
    }
    fn check_link(
        &self,
        store: &dyn Store,
        right: RecordId,
        relation: &str,
        existence: &BTreeMap<RecordId, bool>,
    ) -> Result<(), ErrorCode> {
        if let Some(target) = store
            .describe_relations()
            .into_iter()
            .find(|r| r.name == relation)
            .and_then(|r| r.target_table)
        {
            let other = self.find(&target).ok_or(ErrorCode::Unsupported)?;
            let exists = existence
                .get(&right)
                .copied()
                .filter(|_| target == store.table_name())
                .unwrap_or_else(|| other.get(right).is_some());
            if !exists {
                return Err(ErrorCode::RecordNotFound);
            }
        }
        Ok(())
    }
    fn detach(&self, store: &dyn Store, id: RecordId) -> Result<(), ErrorCode> {
        for (name, other) in &self.tables {
            if name == store.table_name() {
                continue;
            }
            for relation in other.describe_relations() {
                if relation.target_table.as_deref() == Some(store.table_name()) {
                    match other.detach_record(&relation.name, id) {
                        Ok(_) | Err(ErrorCode::Unsupported | ErrorCode::Malformed) => {}
                        Err(code) => return Err(code),
                    }
                }
            }
        }
        Ok(())
    }
    fn batch(&self, store: &dyn Store, ops: &[WriteOp], atomic: bool) -> Response {
        if ops.len() > MAX_BATCH_OPS {
            return err_response(ErrorCode::Malformed);
        }
        if !atomic {
            let results = ops
                .iter()
                .map(|op| {
                    if let WriteOp::Link {
                        right, relation, ..
                    } = op
                        && let Err(code) =
                            self.check_link(store, *right, relation, &BTreeMap::new())
                    {
                        return WriteResult::Failed(code);
                    }
                    let result = store.apply_write_op(op);
                    if let (WriteOp::Delete { id }, WriteResult::Deleted) = (op, &result)
                        && let Err(code) = self.detach(store, *id)
                    {
                        return WriteResult::Failed(code);
                    }
                    result
                })
                .collect();
            return Response::BatchResults { results };
        }
        // Resolve cross-table observations before the adapter's exclusive section.
        // Predict same-table existence in operation order to avoid recursive locks.
        let mut existence = BTreeMap::new();
        let checks: Vec<_> = ops
            .iter()
            .map(|op| {
                match op {
                    WriteOp::Insert { id, .. } => {
                        existence.insert(*id, true);
                    }
                    WriteOp::Delete { id } => {
                        existence.insert(*id, false);
                    }
                    WriteOp::Link {
                        right, relation, ..
                    } => return self.check_link(store, *right, relation, &existence),
                    _ => {}
                }
                Ok(())
            })
            .collect();
        match store.write_batch_checked(ops, &|i| checks[i]) {
            Err(e) => transaction_failed(e),
            Ok(mut results) => {
                for (op, result) in ops.iter().zip(&mut results) {
                    if let (WriteOp::Delete { id }, WriteResult::Deleted) = (op, &*result)
                        && let Err(code) = self.detach(store, *id)
                    {
                        *result = WriteResult::Failed(code);
                    }
                }
                Response::BatchResults { results }
            }
        }
    }
}
struct Session {
    flags: u32,
    staged: Vec<TransactionOp>,
    reads: BTreeMap<(RecordId, FieldRef), ScanValue>,
    updatable: Vec<FieldRef>,
}
impl Session {
    fn read(&mut self, store: &dyn Store, id: RecordId) -> Response {
        let mut fields = store.get(id);
        if self.flags & SESSION_SNAPSHOT_ISOLATION != 0 {
            if let Some(fields) = &mut fields {
                for (tag, value) in fields {
                    if let Some(saved) = self.reads.get(&(id, *tag)) {
                        *value = saved.clone();
                    } else if self.reads.len() < MAX_TRACKED_READS {
                        self.reads.insert((id, *tag), value.clone());
                    }
                }
            } else {
                let tracked: Fields = self
                    .reads
                    .iter()
                    .filter(|((record, _), _)| *record == id)
                    .map(|((_, tag), value)| (*tag, value.clone()))
                    .collect();
                if !tracked.is_empty() {
                    fields = Some(tracked);
                }
            }
        }
        match fields {
            None => Response::NotFound,
            Some(mut fields) => {
                if self.flags & SESSION_READ_YOUR_WRITES != 0 {
                    for (tag, value) in &mut fields {
                        if self.updatable.contains(tag)
                            && let Some(op) = self
                                .staged
                                .iter()
                                .rev()
                                .find(|op| op.id == id && op.field == *tag)
                            && std::mem::discriminant(value) == std::mem::discriminant(&op.value)
                        {
                            *value = op.value.clone();
                        }
                    }
                }
                Response::Record { id, fields }
            }
        }
    }
}
struct Connection<'a> {
    registry: &'a Registry,
    table: usize,
    first: bool,
    version: u32,
    session: Option<Session>,
}
impl Connection<'_> {
    fn request(&mut self, req: Request) -> Response {
        let first = std::mem::replace(&mut self.first, false);
        if let Request::Hello { protocol_version } = req {
            if !first || protocol_version == 0 {
                return err_response(ErrorCode::Malformed);
            }
            self.version = protocol_version.min(PROTOCOL_VERSION);
            return Response::Hello {
                protocol_version: self.version,
            };
        }
        // One acquisition per relationship request, before any adapter access;
        // the guard outlives checks, apply and every detach, but not response I/O.
        let _relationship = if matches!(
            req,
            Request::Link { .. }
                | Request::Delete { .. }
                | Request::WriteBatch { .. }
                | Request::Join(_)
        ) {
            self.registry
                .relationship_lock
                .as_ref()
                .map(|l| l.lock().unwrap_or_else(|p| p.into_inner()))
        } else {
            None
        };
        if self.session.is_some()
            && matches!(
                req,
                Request::Begin
                    | Request::BeginWith { .. }
                    | Request::Transaction { .. }
                    | Request::Insert { .. }
                    | Request::Link { .. }
                    | Request::Replace { .. }
                    | Request::Delete { .. }
                    | Request::Compact
                    | Request::ReplaceIf { .. }
                    | Request::WriteBatch { .. }
                    | Request::Use { .. }
            )
        {
            return err_response(ErrorCode::SessionOpen);
        }
        let store = self.registry.tables[self.table].1.as_ref();
        match req {
            Request::Authenticate { .. } => Response::Ok,
            Request::Begin | Request::BeginWith { .. } => {
                let flags = match req {
                    Request::BeginWith { flags } => flags,
                    _ => 0,
                };
                let known = if self.version >= 5 {
                    SESSION_READ_YOUR_WRITES
                } else {
                    0
                } | if self.version >= 6 {
                    SESSION_VALIDATE_ON_STAGE
                } else {
                    0
                } | if self.version >= 7 {
                    SESSION_SNAPSHOT_ISOLATION
                } else {
                    0
                };
                if flags & !known != 0 {
                    return err_response(ErrorCode::Malformed);
                }
                self.session = Some(Session {
                    flags,
                    staged: Vec::new(),
                    reads: BTreeMap::new(),
                    updatable: store
                        .describe()
                        .fields
                        .into_iter()
                        .filter(|f| f.capabilities.update)
                        .map(|f| f.tag)
                        .collect(),
                });
                Response::Ok
            }
            Request::Commit => match self.session.take() {
                None => err_response(ErrorCode::NoSession),
                Some(session) => {
                    let reads: ReadSet = session
                        .reads
                        .into_iter()
                        .map(|((id, tag), value)| (id, tag, value))
                        .collect();
                    store
                        .apply_transaction(&session.staged, &reads)
                        .map(|()| Response::Ok)
                        .unwrap_or_else(transaction_failed)
                }
            },
            Request::Rollback => {
                if self.session.take().is_some() {
                    Response::Ok
                } else {
                    err_response(ErrorCode::NoSession)
                }
            }
            Request::GetById { id } if self.session.is_some() => self
                .session
                .as_mut()
                .expect("guarded session")
                .read(store, id),
            Request::UpdateField { id, field, value } if self.session.is_some() => {
                let session = self.session.as_mut().expect("guarded session");
                let op = TransactionOp { id, field, value };
                if session.flags & SESSION_VALIDATE_ON_STAGE != 0
                    && let Err(code) = store.validate_op(&op)
                {
                    return err_response(code);
                }
                if session.staged.len() == MAX_STAGED_OPS {
                    return err_response(ErrorCode::SessionFull);
                }
                let index = session.staged.len() as u32;
                session.staged.push(op);
                Response::Staged { index }
            }
            Request::Use { table } => match self
                .registry
                .tables
                .iter()
                .position(|(name, _)| name == &table)
            {
                None => err_response(ErrorCode::Malformed),
                Some(index) => {
                    self.table = index;
                    Response::Ok
                }
            },
            Request::ListTables => Response::Tables {
                names: self
                    .registry
                    .tables
                    .iter()
                    .map(|(n, _)| n.clone())
                    .collect(),
                primary: self.registry.tables[self.registry.primary].0.clone(),
            },
            Request::Link {
                left,
                right,
                relation,
            } => match self
                .registry
                .check_link(store, right, &relation, &BTreeMap::new())
            {
                Err(code) => err_response(code),
                Ok(()) => dispatch(
                    store,
                    Request::Link {
                        left,
                        right,
                        relation,
                    },
                ),
            },
            Request::Delete { id } => {
                let response = dispatch(store, Request::Delete { id });
                if response != Response::Ok {
                    return response;
                }
                self.registry
                    .detach(store, id)
                    .map(|()| Response::Ok)
                    .unwrap_or_else(err_response)
            }
            Request::WriteBatch { ops, atomic } => self.registry.batch(store, &ops, atomic),
            Request::Join(spec) if spec.right_table.is_some() => {
                let right = self
                    .registry
                    .find(spec.right_table.as_deref().expect("named join"));
                let right_schema = right.map(Store::describe);
                match validate_join(
                    &store.describe(),
                    &store.describe_relations(),
                    right_schema.as_ref(),
                    &spec,
                ) {
                    Err(code) => err_response(code),
                    Ok(()) => Response::JoinedRows {
                        rows: evaluate_join(store, right.expect("validated right schema"), &spec),
                    },
                }
            }
            other => dispatch(store, other),
        }
    }
}
/// Serve framed requests until clean EOF. A truncated frame returns an I/O error;
/// a malformed payload produces Malformed and leaves framing synchronized/open.
/// Disconnect discards staged writes. No authentication enforcement or version
/// content rewriting is performed (D4/D5); only BeginWith flag availability varies.
pub fn handle_connection<S: Read + Write>(stream: &mut S, registry: &Registry) -> io::Result<()> {
    let mut connection = Connection {
        registry,
        table: registry.primary,
        first: true,
        version: 1,
        session: None,
    };
    loop {
        // Distinguish clean EOF from a partially read prefix or payload.
        let mut first = [0u8; 1];
        let count = loop {
            match stream.read(&mut first) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                other => break other?,
            }
        };
        if count == 0 {
            return Ok(());
        }
        let payload = read_message(&mut first.as_slice().chain(&mut *stream))?;
        let response = match decode_request(&payload) {
            Ok(request) => connection.request(request),
            Err(_) => {
                connection.first = false;
                err_response(ErrorCode::Malformed)
            }
        };
        write_message(stream, &encode_response(&response))?;
        stream.flush()?;
    }
}

#[cfg(test)]
#[path = "../tests/support/interleaving.rs"]
mod interleaving;
