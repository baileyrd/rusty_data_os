mod support;
use std::sync::{Arc, Mutex};
use support::*;
use uc_protocol::*;
fn join(right: Option<&str>) -> Request {
    Request::Join(JoinSpec {
        relation: JoinRelation::Neighbors(Some("edge".into())),
        right_table: right.map(str::to_owned),
        left: Selection::All,
        right: Selection::All,
        left_filter: vec![],
        right_filter: vec![],
        limit: None,
    })
}
#[test]
fn registry_validates_names_primary_and_foreign_lock_opt_out() {
    assert!(Registry::new(vec![], 0).is_err());
    let a = TestStore::new("a", None, &[]);
    let b = TestStore::new("b", None, &[]);
    assert!(Registry::new(vec![("wrong".into(), a.clone())], 0).is_err());
    assert!(Registry::new(vec![("a".into(), a.clone()), ("a".into(), a.clone())], 0).is_err());
    assert!(Registry::new(vec![("a".into(), a.clone())], 1).is_err());
    let reg = registry(vec![a, b], 1);
    assert!(!reg.has_relationship_lock());
    let mut c = Client::new(reg);
    assert_eq!(
        c.request(Request::ListTables),
        Response::Tables {
            names: vec!["a".into(), "b".into()],
            primary: "b".into()
        }
    );
    assert_error(
        c.request(Request::Use {
            table: "missing".into(),
        }),
        ErrorCode::Malformed,
    );
    assert_eq!(c.request(Request::Use { table: "a".into() }), Response::Ok);
    c.close();
    assert!(registry(vec![TestStore::new("a", Some("b"), &[])], 0).has_relationship_lock());
}
#[test]
fn foreign_link_checks_missing_endpoint_and_registration_before_apply() {
    let a = TestStore::new("a", Some("b"), &[(1, 1)]);
    let b = TestStore::new("b", None, &[(2, 2)]);
    let mut c = Client::new(registry(vec![a.clone(), b], 0));
    let before = a.snapshot();
    assert_error(c.request(link(1, 99)), ErrorCode::RecordNotFound);
    assert_eq!(a.snapshot(), before);
    assert_eq!(c.request(link(1, 2)), Response::Ok);
    assert_eq!(c.request(link(1, 2)), Response::Ok);
    assert_eq!(a.count_edges("edge"), Ok(1));
    c.close();
    let mut c = Client::new(registry(vec![a.clone()], 0));
    let before = a.snapshot();
    assert_error(c.request(link(1, 2)), ErrorCode::Unsupported);
    assert_eq!(a.snapshot(), before);
    c.close();
}
#[test]
fn delete_detaches_foreign_adjacency_and_count_only_after_success() {
    let a = TestStore::new("a", Some("b"), &[(1, 1)]);
    let b = TestStore::new("b", None, &[(2, 2)]);
    let mut c = Client::new(registry(vec![a.clone(), b.clone()], 0));
    assert_eq!(c.request(link(1, 2)), Response::Ok);
    assert_eq!(
        c.request(Request::CountEdges {
            relation: "edge".into()
        }),
        Response::Count { count: 1 }
    );
    assert_eq!(
        c.request(Request::NeighborsByRelation {
            id: id(1),
            relation: "edge".into()
        }),
        Response::RecordList {
            records: vec![id(2)]
        }
    );
    assert_eq!(c.request(Request::Use { table: "b".into() }), Response::Ok);
    assert_eq!(
        c.request(Request::Delete { id: id(99) }),
        Response::NotFound
    );
    assert!(a.snapshot().detaches.is_empty());
    assert_eq!(c.request(Request::Delete { id: id(2) }), Response::Ok);
    assert_eq!(b.get(id(2)), None);
    assert_eq!(c.request(Request::Use { table: "a".into() }), Response::Ok);
    assert_eq!(
        c.request(Request::CountEdges {
            relation: "edge".into()
        }),
        Response::Count { count: 0 }
    );
    assert_eq!(
        c.request(Request::NeighborsByRelation {
            id: id(1),
            relation: "edge".into()
        }),
        Response::RecordList { records: vec![] }
    );
    assert_eq!(a.snapshot().detaches, vec![id(2)]);
    c.close();
}
#[test]
fn detach_registration_order_skips_unsupported_malformed_and_surfaces_storage_partial_state() {
    for code in [
        ErrorCode::Unsupported,
        ErrorCode::Malformed,
        ErrorCode::Storage,
    ] {
        let a = TestStore::new("a", Some("b"), &[(1, 1)]);
        let b = TestStore::new("b", None, &[(2, 2)]);
        let c = TestStore::new("c", Some("b"), &[(3, 3)]);
        let d = TestStore::new("d", Some("b"), &[(4, 4)]);
        for (s, n) in [(&a, 1), (&c, 3), (&d, 4)] {
            s.link_records(id(n), id(2), "edge").unwrap();
        }
        *c.detach_error.lock().unwrap() = Some(code);
        let order = Arc::new(Mutex::new(Vec::new()));
        for s in [&a, &c, &d] {
            let order = order.clone();
            let name = s.name.clone();
            *s.observer.lock().unwrap() = Some(Arc::new(move |event| {
                if event == "detach" {
                    order.lock().unwrap().push(name.clone());
                }
            }));
        }
        let mut client = Client::new(registry(
            vec![a.clone(), b.clone(), c.clone(), d.clone()],
            1,
        ));
        let response = client.request(Request::Delete { id: id(2) });
        if code == ErrorCode::Storage {
            assert_error(response, code);
            assert_eq!(*order.lock().unwrap(), ["a", "c"]);
            assert_eq!(d.count_edges("edge"), Ok(1));
        } else {
            assert_eq!(response, Response::Ok);
            assert_eq!(*order.lock().unwrap(), ["a", "c", "d"]);
            assert_eq!(d.count_edges("edge"), Ok(0));
        }
        assert!(b.get(id(2)).is_none());
        assert_eq!(a.count_edges("edge"), Ok(0));
        client.close();
    }
}
#[test]
fn pipelined_and_atomic_batches_check_links_and_cascade_deletes() {
    for atomic in [false, true] {
        let a = TestStore::new("a", Some("b"), &[(1, 1)]);
        let b = TestStore::new("b", None, &[(2, 2)]);
        let c = TestStore::new("c", Some("a"), &[(3, 3)]);
        c.link_records(id(3), id(1), "edge").unwrap();
        let mut client = Client::new(registry(vec![a.clone(), b, c.clone()], 0));
        let ops = vec![batch_link(1, 2), WriteOp::Delete { id: id(1) }];
        assert_eq!(
            client.request(Request::WriteBatch { ops, atomic }),
            Response::BatchResults {
                results: vec![WriteResult::Linked, WriteResult::Deleted]
            }
        );
        assert!(a.get(id(1)).is_none());
        assert_eq!(a.count_edges("edge"), Ok(0));
        assert_eq!(c.count_edges("edge"), Ok(0));
        assert_eq!(c.snapshot().detaches, vec![id(1)]);
        client.close();
    }
}
#[test]
fn batch_preconditions_preserve_operation_order_and_atomic_refusal_applies_nothing() {
    for atomic in [false, true] {
        let a = TestStore::new("a", Some("b"), &[(1, 1)]);
        let b = TestStore::new("b", None, &[(2, 2)]);
        let mut client = Client::new(registry(vec![a.clone(), b], 0));
        let ops = vec![
            WriteOp::Insert {
                id: id(4),
                fields: fields(4),
            },
            batch_link(4, 99),
            WriteOp::Insert {
                id: id(5),
                fields: fields(5),
            },
        ];
        let before = a.snapshot();
        let response = client.request(Request::WriteBatch { ops, atomic });
        if atomic {
            assert_failed(response, 1, ErrorCode::RecordNotFound);
            assert_eq!(a.snapshot(), before);
        } else {
            assert_eq!(
                response,
                Response::BatchResults {
                    results: vec![
                        WriteResult::Inserted,
                        WriteResult::Failed(ErrorCode::RecordNotFound),
                        WriteResult::Inserted
                    ]
                }
            );
            assert!(a.get(id(4)).is_some());
            assert!(a.get(id(5)).is_some());
        }
        client.close();
    }
    // Local failure at op 0 precedes the precomputed foreign failure at op 1.
    let a = TestStore::new("a", Some("b"), &[(1, 1)]);
    let b = TestStore::new("b", None, &[]);
    let mut c = Client::new(registry(vec![a.clone(), b], 0));
    assert_failed(
        c.request(Request::WriteBatch {
            ops: vec![
                WriteOp::Insert {
                    id: id(2),
                    fields: vec![],
                },
                batch_link(1, 99),
            ],
            atomic: true,
        }),
        0,
        ErrorCode::Malformed,
    );
    assert_eq!(a.snapshot().writes, 0);
    c.close();
}
#[test]
fn batch_detach_failure_reports_affected_op_after_local_apply() {
    for atomic in [false, true] {
        let a = TestStore::new("a", None, &[(1, 1), (2, 2)]);
        let b = TestStore::new("b", Some("a"), &[(3, 3)]);
        b.link_records(id(3), id(1), "edge").unwrap();
        *b.detach_error.lock().unwrap() = Some(ErrorCode::Storage);
        let mut c = Client::new(registry(vec![a.clone(), b], 0));
        assert_eq!(
            c.request(Request::WriteBatch {
                ops: vec![
                    WriteOp::Delete { id: id(1) },
                    WriteOp::Replace {
                        id: id(2),
                        fields: fields(20)
                    }
                ],
                atomic
            }),
            Response::BatchResults {
                results: vec![
                    WriteResult::Failed(ErrorCode::Storage),
                    WriteResult::Replaced
                ]
            }
        );
        assert!(a.get(id(1)).is_none());
        assert_eq!(a.get(id(2)), Some(fields(20)));
        c.close();
    }
}
#[test]
fn batch_4096_accepted_and_4097_refused_without_effect_in_both_modes() {
    for atomic in [false, true] {
        let table = TestStore::new("table", None, &[]);
        let mut c = Client::new(registry(vec![table.clone()], 0));
        let ops: Vec<_> = (0..MAX_BATCH_OPS)
            .map(|i| WriteOp::Insert {
                id: id(i as u128),
                fields: fields(i as i64),
            })
            .collect();
        assert_eq!(
            c.request(Request::WriteBatch { ops, atomic }),
            Response::BatchResults {
                results: vec![WriteResult::Inserted; 4096]
            }
        );
        let before = table.snapshot();
        assert_eq!(before.writes, 4096);
        let ops = vec![WriteOp::Delete { id: id(1) }; MAX_BATCH_OPS + 1];
        assert_error(
            c.request(Request::WriteBatch { ops, atomic }),
            ErrorCode::Malformed,
        );
        assert_eq!(table.snapshot(), before);
        c.close();
    }
}
#[test]
fn joins_preserve_all_precise_cross_table_error_distinctions() {
    let a = TestStore::new("a", Some("b"), &[(1, 1)]);
    let mut b = TestStore::new("b", None, &[(2, 2)]);
    // Right schema differs: selection tag 9 can only resolve against b.
    Arc::get_mut(&mut b).unwrap().schema.fields[0].tag = 9;
    b.state
        .lock()
        .unwrap()
        .rows
        .insert(id(2), vec![(9, ScanValue::I64(2))]);
    a.link_records(id(1), id(2), "edge").unwrap();
    let mut client = Client::new(registry(vec![a.clone(), b], 0));
    let Request::Join(mut spec) = join(Some("b")) else {
        unreachable!()
    };
    spec.right = Selection::Fields(vec![9]);
    assert_eq!(
        client.request(Request::Join(spec)),
        Response::JoinedRows {
            rows: vec![JoinedRow {
                left_id: id(1),
                left: fields(1),
                right_id: id(2),
                right: vec![(9, ScanValue::I64(2))]
            }]
        }
    );
    assert_error(client.request(join(Some("wrong"))), ErrorCode::Malformed);
    assert_error(client.request(join(None)), ErrorCode::Unsupported);
    client.close();
    let mut client = Client::new(registry(vec![a.clone()], 0));
    assert_error(client.request(join(Some("b"))), ErrorCode::Malformed);
    client.close();
    assert_error(dispatch(a.as_ref(), join(Some("b"))), ErrorCode::Malformed);
    assert_error(
        dispatch(a.as_ref(), join(Some("wrong"))),
        ErrorCode::Malformed,
    );
    let a = TestStore::new("a", None, &[(1, 1), (2, 2)]);
    a.link_records(id(1), id(2), "edge").unwrap();
    assert_eq!(
        dispatch(a.as_ref(), join(None)),
        Response::JoinedRows {
            rows: vec![JoinedRow {
                left_id: id(1),
                left: fields(1),
                right_id: id(2),
                right: fields(2)
            }]
        }
    );
    assert_error(dispatch(a.as_ref(), join(Some("a"))), ErrorCode::Malformed);
    let Request::Join(mut unknown) = join(None) else {
        unreachable!()
    };
    unknown.relation = JoinRelation::Neighbors(Some("unknown".into()));
    assert_error(
        dispatch(a.as_ref(), Request::Join(unknown)),
        ErrorCode::Malformed,
    );
}
#[test]
fn atomic_self_target_existence_tracks_prior_insert_and_delete_without_reentrant_lock() {
    let a = TestStore::new("a", Some("a"), &[(1, 1)]);
    let mut c = Client::new(registry(vec![a.clone()], 0));
    assert_eq!(
        c.request(Request::WriteBatch {
            ops: vec![
                WriteOp::Insert {
                    id: id(2),
                    fields: fields(2)
                },
                batch_link(1, 2)
            ],
            atomic: true
        }),
        Response::BatchResults {
            results: vec![WriteResult::Inserted, WriteResult::Linked]
        }
    );
    let before = a.snapshot();
    assert_failed(
        c.request(Request::WriteBatch {
            ops: vec![WriteOp::Delete { id: id(2) }, batch_link(1, 2)],
            atomic: true,
        }),
        1,
        ErrorCode::RecordNotFound,
    );
    assert_eq!(a.snapshot(), before);
    c.close();
}
