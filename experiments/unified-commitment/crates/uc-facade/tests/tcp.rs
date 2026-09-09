mod support;
use std::{
    io,
    net::{Shutdown, SocketAddr, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
use support::*;
use uc_facade::{LoopbackListener, serve};
use uc_protocol::{codec::*, framing::*, *};

struct Server {
    worker: Option<JoinHandle<io::Result<()>>>,
    stop: Arc<AtomicBool>,
    addr: SocketAddr,
    stores: Vec<Arc<dyn Store>>,
    temp: Temp,
}
impl Server {
    fn new() -> Self {
        let temp = Temp::new();
        let stores = temp.stores(false);
        let registry = registry(&stores);
        assert!(!registry.has_relationship_lock());
        let listener = LoopbackListener::bind().unwrap();
        let addr = listener.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_ne!(addr.port(), 0);
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let worker = std::thread::spawn(move || serve(listener, registry, &signal));
        Self {
            worker: Some(worker),
            stop,
            addr,
            stores,
            temp,
        }
    }
    fn client(&self) -> Client {
        let stream = TcpStream::connect(self.addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        stream.set_nodelay(true).unwrap();
        let mut client = Client(stream);
        assert_eq!(
            client.request(Request::Hello {
                protocol_version: 22
            }),
            Response::Hello {
                protocol_version: 22
            }
        );
        client
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap().unwrap();
        }
        self.stores.clear();
        // Stores/connection workers release directory ownership before Temp cleanup.
    }
}
struct Client(TcpStream);
impl Client {
    fn pipeline(&mut self, requests: &[Request]) {
        let mut bytes = Vec::new();
        for request in requests {
            write_message(&mut bytes, &encode_request(request)).unwrap();
        }
        std::io::Write::write_all(&mut self.0, &bytes).unwrap();
    }
    fn response(&mut self) -> Response {
        decode_response(&read_message(&mut self.0).unwrap()).unwrap()
    }
    fn request(&mut self, request: Request) -> Response {
        self.pipeline(&[request]);
        self.response()
    }
    fn ok(&mut self, request: Request) {
        assert_eq!(self.request(request), Response::Ok);
    }
    fn record(&mut self, domain: usize, key: RecordId, value: i64) {
        assert_eq!(
            self.request(Request::GetById { id: key }),
            Response::Record {
                id: key,
                fields: fields(domain, value)
            }
        );
    }
    fn use_table(&mut self, table: &str) {
        self.ok(Request::Use {
            table: table.into(),
        });
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Shutdown::Both);
    }
}

#[test]
fn real_tcp_unlabeled_memory_join() {
    unlabeled_join(0, "memory", ["mentions", "mentions"]);
}

#[test]
fn real_tcp_unlabeled_entity_join() {
    unlabeled_join(1, "entity", ["works_with", "studied_with"]);
}

fn unlabeled_join(domain: usize, table: &str, labels: [&str; 2]) {
    let server = Server::new();
    let mut client = server.client();
    client.use_table(table);
    for key in 1..=4 {
        client.ok(Request::Insert {
            id: id(key),
            fields: fields(domain, i64::from(key)),
        });
    }
    for (right, label) in [(2, labels[0]), (3, labels[1])] {
        client.ok(Request::Link {
            left: id(1),
            right: id(right),
            relation: label.into(),
        });
    }
    // An unlabeled Join follows all same-table edges (both Entity labels), in
    // both directions, and excludes the unlinked fourth record.
    assert_eq!(
        client.request(Request::Join(JoinSpec {
            relation: JoinRelation::Neighbors(None),
            right_table: None,
            left: Selection::All,
            right: Selection::All,
            left_filter: vec![],
            right_filter: vec![],
            limit: None,
        })),
        Response::JoinedRows {
            rows: [(1, 2), (1, 3), (2, 1), (3, 1)]
                .into_iter()
                .map(|(left, right)| JoinedRow {
                    left_id: id(left),
                    left: fields(domain, i64::from(left)),
                    right_id: id(right),
                    right: fields(domain, i64::from(right)),
                })
                .collect(),
        }
    );
    let Response::Relations { relations } = client.request(Request::DescribeRelations) else {
        panic!("expected relation descriptors");
    };
    assert!(relations.contains(&RelationDescriptor {
        name: "neighbors".into(),
        kind: JoinRelation::Neighbors(None),
        target_table: None,
    }));
    assert!(
        relations
            .iter()
            .all(|relation| relation.target_table.is_none())
    );
    for label in labels {
        assert!(relations.contains(&RelationDescriptor {
            name: label.into(),
            kind: JoinRelation::Neighbors(Some(label.into())),
            target_table: None,
        }));
    }
}

#[test]
fn real_tcp_three_domains_crud_entity_links_batches_and_all_flags_sessions() {
    let server = Server::new();
    let mut client = server.client();
    assert_eq!(
        client.request(Request::ListTables),
        Response::Tables {
            names: vec!["memory".into(), "entity".into(), "relation".into()],
            primary: "memory".into()
        }
    );
    for (d, table) in ["memory", "entity", "relation"].iter().enumerate() {
        client.use_table(table);
        client.ok(Request::Insert {
            id: id(1),
            fields: fields(d, 1),
        });
        client.record(d, id(1), 1);
        client.ok(Request::Replace {
            id: id(1),
            fields: fields(d, 2),
        });
        client.record(d, id(1), 2);
        client.ok(Request::BeginWith {
            flags: SESSION_READ_YOUR_WRITES
                | SESSION_VALIDATE_ON_STAGE
                | SESSION_SNAPSHOT_ISOLATION,
        });
        client.record(d, id(1), 2);
        assert_eq!(
            client.request(Request::UpdateField {
                id: id(1),
                field: UPDATE[d],
                value: ScanValue::I64(3)
            }),
            Response::Staged { index: 0 }
        );
        client.record(d, id(1), 3);
        client.ok(Request::Commit);
        client.record(d, id(1), 3);
        client.ok(Request::ReplaceIf {
            id: id(1),
            fields: fields(d, 4),
            guard: Predicate {
                field: UPDATE[d],
                op: CompareOp::Lt,
                value: ScanValue::I64(4),
            },
        });
        client.record(d, id(1), 4);
        if d == 1 {
            client.ok(Request::Insert {
                id: id(2),
                fields: fields(d, 2),
            });
            client.ok(Request::Link {
                left: id(1),
                right: id(2),
                relation: "works_with".into(),
            });
            assert_eq!(
                client.request(Request::Neighbors { id: id(2) }),
                Response::RecordList {
                    records: vec![id(1)]
                }
            );
            assert_eq!(
                client.request(Request::NeighborsByRelation {
                    id: id(1),
                    relation: "works_with".into()
                }),
                Response::RecordList {
                    records: vec![id(2)]
                }
            );
            assert_eq!(
                client.request(Request::ListRelationKinds),
                Response::RelationKinds {
                    kinds: vec![
                        "mentioned_with".into(),
                        "relates_to".into(),
                        "works_with".into()
                    ]
                }
            );
            assert_eq!(
                client.request(Request::CountEdges {
                    relation: "works_with".into()
                }),
                Response::Count { count: 1 }
            );
        }
        client.ok(Request::Delete { id: id(1) });
        assert_eq!(
            client.request(Request::GetById { id: id(1) }),
            Response::NotFound
        );
        if d == 1 {
            assert_eq!(
                client.request(Request::Neighbors { id: id(2) }),
                Response::RecordList { records: vec![] }
            );
            client.ok(Request::Delete { id: id(2) });
        }
    }
    client.use_table("memory");
    // Both non-atomic batch semantics and actual wire pipelining: send the batch
    // and following Get before reading either response.
    client.pipeline(&[
        Request::WriteBatch {
            atomic: false,
            ops: vec![
                WriteOp::Insert {
                    id: id(3),
                    fields: fields(0, 10),
                },
                WriteOp::Replace {
                    id: id(3),
                    fields: fields(0, 11),
                },
                WriteOp::Insert {
                    id: id(4),
                    fields: fields(0, 12),
                },
                WriteOp::Delete { id: id(4) },
            ],
        },
        Request::GetById { id: id(3) },
    ]);
    assert_eq!(
        client.response(),
        Response::BatchResults {
            results: vec![
                WriteResult::Inserted,
                WriteResult::Replaced,
                WriteResult::Inserted,
                WriteResult::Deleted
            ]
        }
    );
    assert_eq!(
        client.response(),
        Response::Record {
            id: id(3),
            fields: fields(0, 11)
        }
    );
    assert_eq!(
        client.request(Request::GetById { id: id(4) }),
        Response::NotFound
    );
}

#[test]
fn real_tcp_nonempty_atomic_batches_refuse_without_any_mutation_on_every_domain() {
    let server = Server::new();
    let mut client = server.client();
    for (d, table) in ["memory", "entity", "relation"].iter().enumerate() {
        client.use_table(table);
        client.ok(Request::Insert {
            id: id(1),
            fields: fields(d, 1),
        });
        assert!(matches!(
            client.request(Request::WriteBatch {
                atomic: true,
                ops: vec![
                    WriteOp::Replace {
                        id: id(1),
                        fields: fields(d, 2)
                    },
                    WriteOp::Insert {
                        id: id(2),
                        fields: fields(d, 2)
                    }
                ]
            }),
            Response::TransactionFailed {
                index: 0,
                code: ErrorCode::Unsupported,
                ..
            }
        ));
        client.record(d, id(1), 1);
        assert_eq!(
            client.request(Request::GetById { id: id(2) }),
            Response::NotFound
        );
    }
}

#[test]
fn real_tcp_concurrent_sessions_conflict_before_writes_on_every_domain() {
    let server = Server::new();
    let mut first = server.client();
    let mut second = server.client();
    for (d, table) in ["memory", "entity", "relation"].iter().enumerate() {
        first.use_table(table);
        second.use_table(table);
        first.ok(Request::Insert {
            id: id(1),
            fields: fields(d, 1),
        });
        first.ok(Request::BeginWith { flags: 7 });
        first.record(d, id(1), 1);
        assert_eq!(
            first.request(Request::UpdateField {
                id: id(1),
                field: UPDATE[d],
                value: ScanValue::I64(3)
            }),
            Response::Staged { index: 0 }
        );
        second.ok(Request::UpdateField {
            id: id(1),
            field: UPDATE[d],
            value: ScanValue::I64(2),
        });
        first.record(d, id(1), 3);
        assert!(matches!(
            first.request(Request::Commit),
            Response::TransactionFailed {
                index: 0,
                code: ErrorCode::Conflict,
                ..
            }
        ));
        second.record(d, id(1), 2);
    }
}

#[test]
fn real_tcp_4096_staged_updates_commit_once_per_domain_and_reopen() {
    let server = Server::new();
    let mut client = server.client();
    for (d, table) in ["memory", "entity", "relation"].iter().enumerate() {
        client.use_table(table);
        client.ok(Request::Insert {
            id: id(1),
            fields: fields(d, 0),
        });
        client.ok(Request::BeginWith { flags: 7 });
        let requests: Vec<_> = (1..=4096)
            .map(|value| Request::UpdateField {
                id: id(1),
                field: UPDATE[d],
                value: ScanValue::I64(value),
            })
            .collect();
        client.pipeline(&requests);
        for index in 0..4096 {
            assert_eq!(client.response(), Response::Staged { index });
        }
        assert!(matches!(
            client.request(Request::UpdateField {
                id: id(1),
                field: UPDATE[d],
                value: ScanValue::I64(4097)
            }),
            Response::Err {
                code: ErrorCode::SessionFull,
                ..
            }
        ));
        client.ok(Request::Commit);
        client.record(d, id(1), 4096);
        let history =
            uc_core::read_history(&server.temp.0.join(table).join(uc_core::HISTORY_FILE)).unwrap();
        assert_eq!(history.events.len(), 2);
    }
    drop(client);
    // Join workers and release engines, then use the retained directories for reopen.
    let mut server = server;
    server.stop.store(true, Ordering::Release);
    server.worker.take().unwrap().join().unwrap().unwrap();
    server.stores.clear();
    let reopened = server.temp.stores(true);
    for (d, store) in reopened.iter().enumerate() {
        assert_eq!(store.get(id(1)), Some(fields(d, 4096)));
    }
    drop(reopened);
    // Drop normally handles a previously joined worker too.
}

#[test]
fn real_tcp_aggregate_overflow_is_a_response_and_connection_remains_usable() {
    let server = Server::new();
    let mut client = server.client();
    client.use_table("entity");
    for key in [id(1), id(2)] {
        client.ok(Request::Insert {
            id: key,
            fields: fields(1, i64::MAX),
        });
    }
    for func in [AggregateFn::Sum, AggregateFn::Avg] {
        let response = client.request(Request::Aggregate {
            group_by: vec![],
            filter: vec![],
            aggregates: vec![AggregateSpec {
                func,
                field: Some(2),
            }],
            limit: None,
        });
        if func == AggregateFn::Sum {
            assert!(matches!(
                response,
                Response::Err {
                    code: ErrorCode::Malformed,
                    ..
                }
            ));
        } else {
            assert_eq!(
                response,
                Response::Groups {
                    groups: vec![AggregateGroup {
                        key: vec![],
                        values: vec![ScanValue::F64(i64::MAX as f64)]
                    }]
                }
            );
        }
    }
    client.record(1, id(1), i64::MAX);
}
