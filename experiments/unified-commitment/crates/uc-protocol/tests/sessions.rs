mod support;
use support::*;
use uc_protocol::framing::*;
use uc_protocol::*;

#[test]
fn hello_negotiation_errors_keep_connection_open_and_authentication_is_unconditional() {
    for (offered, negotiated) in [(22, 22), (99, 22), (2, 2), (1, 1)] {
        let table = TestStore::new("table", None, &[(1, 10)]);
        let mut c = Client::raw(registry(vec![table], 0));
        assert_eq!(
            c.request(Request::Hello {
                protocol_version: offered
            }),
            Response::Hello {
                protocol_version: negotiated
            }
        );
        assert_error(
            c.request(Request::Hello {
                protocol_version: 22,
            }),
            ErrorCode::Malformed,
        );
        assert_eq!(
            c.request(Request::Authenticate {
                token: "anything".into()
            }),
            Response::Ok
        );
        assert_eq!(
            c.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(10)
            }
        );
        c.close();
    }
    let table = TestStore::new("table", None, &[]);
    let reg = registry(vec![table], 0);
    let mut c = Client::raw(reg.clone());
    assert_error(
        c.request(Request::Hello {
            protocol_version: 0,
        }),
        ErrorCode::Malformed,
    );
    assert_error(
        c.request(Request::Hello {
            protocol_version: 22,
        }),
        ErrorCode::Malformed,
    );
    assert_eq!(
        c.request(Request::Authenticate {
            token: String::new()
        }),
        Response::Ok
    );
    c.close();
    let mut c = Client::raw(reg);
    assert_eq!(
        c.request(Request::ListTables),
        Response::Tables {
            names: vec!["table".into()],
            primary: "table".into()
        }
    );
    assert_error(
        c.request(Request::BeginWith { flags: 1 }),
        ErrorCode::Malformed,
    ); // implicit version 1
    assert_eq!(c.request(Request::Begin), Response::Ok); // D5: no general version rewriting/gating
    assert_eq!(c.request(Request::Rollback), Response::Ok);
    c.close();
}
#[test]
fn malformed_payload_preserves_frame_boundary_and_consumes_first_frame() {
    let reg = registry(vec![TestStore::new("table", None, &[])], 0);
    let mut c = Client::raw(reg);
    write_message(&mut c.pipe, &[255, 255, 255, 255]).unwrap();
    assert_error(c.receive(), ErrorCode::Malformed);
    assert_error(
        c.request(Request::Hello {
            protocol_version: 22,
        }),
        ErrorCode::Malformed,
    );
    assert_eq!(
        c.request(Request::Authenticate { token: "x".into() }),
        Response::Ok
    );
    c.close();
}
#[test]
fn truncated_frame_errors_and_clean_disconnect_discards_session() {
    struct Buffer {
        read: std::io::Cursor<Vec<u8>>,
        written: Vec<u8>,
    }
    impl std::io::Read for Buffer {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            std::io::Read::read(&mut self.read, b)
        }
    }
    impl std::io::Write for Buffer {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.written.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let table = TestStore::new("table", None, &[(1, 1)]);
    let reg = registry(vec![table.clone()], 0);
    for bytes in [vec![1], vec![8, 0, 0, 0, 1]] {
        let mut stream = Buffer {
            read: std::io::Cursor::new(bytes),
            written: vec![],
        };
        assert_eq!(
            handle_connection(&mut stream, &reg).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }
    let mut c = Client::new(reg);
    assert_eq!(c.request(Request::Begin), Response::Ok);
    assert_eq!(c.request(update(1, 5)), Response::Staged { index: 0 });
    c.close();
    assert_eq!(table.get(id(1)), Some(fields(1)));
    assert_eq!(table.snapshot().writes, 0);
}
#[test]
fn one_shot_transaction_validates_all_updates_before_apply_and_uses_empty_read_set() {
    let table = TestStore::new("table", None, &[(1, 1), (2, 2)]);
    let mut c = Client::new(registry(vec![table.clone()], 0));
    let before = table.snapshot();
    assert_failed(
        c.request(Request::Transaction {
            updates: vec![op(1, 10), op(3, 30)],
        }),
        1,
        ErrorCode::RecordNotFound,
    );
    assert_eq!(table.snapshot(), before);
    assert_eq!(
        c.request(Request::Transaction {
            updates: vec![op(1, 10), op(2, 20)]
        }),
        Response::Ok
    );
    assert_eq!(table.get(id(1)), Some(fields(10)));
    assert_eq!(table.get(id(2)), Some(fields(20)));
    assert_eq!(table.snapshot().read_sets, vec![ReadSet::new()]);
    c.close();
}
#[test]
fn each_flag_and_combination_controls_its_own_behavior() {
    for flags in 0..=7 {
        let table = TestStore::new("table", None, &[(1, 1)]);
        let mut c = Client::new(registry(vec![table.clone()], 0));
        assert_eq!(c.request(Request::BeginWith { flags }), Response::Ok);
        assert_eq!(
            c.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(1)
            }
        );
        assert_eq!(c.request(update(1, 10)), Response::Staged { index: 0 });
        let expected = if flags & 1 != 0 { 10 } else { 1 };
        assert_eq!(
            c.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(expected)
            }
        );
        if flags & 2 != 0 {
            assert_error(c.request(update(99, 9)), ErrorCode::RecordNotFound);
        }
        assert_eq!(c.request(Request::Commit), Response::Ok);
        assert_eq!(table.get(id(1)), Some(fields(10)));
        let reads = if flags & 4 != 0 {
            vec![(id(1), 1, ScanValue::I64(1))]
        } else {
            vec![]
        };
        assert_eq!(table.snapshot().read_sets, vec![reads]);
        assert_eq!(table.snapshot().transaction_sizes, vec![1]);
        c.close();
    }
}
#[test]
fn begin_with_flag_versions_unknown_bits_and_session_survival() {
    for version in [1, 4, 5, 6, 7, 22] {
        for flags in [1, 2, 4, 7, 8, u32::MAX] {
            let mut c = Client::raw(registry(vec![TestStore::new("table", None, &[])], 0));
            assert_eq!(
                c.request(Request::Hello {
                    protocol_version: version
                }),
                Response::Hello {
                    protocol_version: version
                }
            );
            let valid = flags & !7 == 0
                && (flags & 1 == 0 || version >= 5)
                && (flags & 2 == 0 || version >= 6)
                && (flags & 4 == 0 || version >= 7);
            if valid {
                assert_eq!(c.request(Request::BeginWith { flags }), Response::Ok);
                assert_eq!(c.request(Request::Rollback), Response::Ok);
            } else {
                assert_error(
                    c.request(Request::BeginWith { flags }),
                    ErrorCode::Malformed,
                );
                assert_error(c.request(Request::Commit), ErrorCode::NoSession);
            }
            c.close();
        }
    }
}
#[test]
fn snapshot_repeatable_after_concurrent_commit_or_deletion_and_conflict_applies_nothing() {
    for delete in [false, true] {
        let table = TestStore::new("table", None, &[(1, 1), (2, 2)]);
        let reg = registry(vec![table.clone()], 0);
        let mut a = Client::new(reg.clone());
        let mut b = Client::new(reg);
        assert_eq!(a.request(Request::BeginWith { flags: 7 }), Response::Ok);
        assert_eq!(
            a.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(1)
            }
        );
        assert_eq!(a.request(update(2, 20)), Response::Staged { index: 0 });
        let mutation = if delete {
            Request::Delete { id: id(1) }
        } else {
            Request::Transaction {
                updates: vec![op(1, 100)],
            }
        };
        assert_eq!(b.request(mutation), Response::Ok);
        assert_eq!(
            a.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(1)
            }
        );
        let before = table.snapshot();
        assert_failed(a.request(Request::Commit), 0, ErrorCode::Conflict);
        assert_eq!(table.snapshot(), before);
        assert_error(a.request(Request::Commit), ErrorCode::NoSession);
        a.close();
        b.close();
    }
}
#[test]
fn snapshot_does_not_freeze_absent_records_scans_or_untracked_keys_at_cap() {
    let rows: Vec<_> = (1..=4097).map(|n| (n, n as i64)).collect();
    let table = TestStore::new("table", None, &rows);
    let reg = registry(vec![table.clone()], 0);
    let mut a = Client::new(reg.clone());
    let mut b = Client::new(reg);
    assert_eq!(a.request(Request::BeginWith { flags: 4 }), Response::Ok);
    assert_eq!(
        a.request(Request::GetById { id: id(5000) }),
        Response::NotFound
    );
    assert_eq!(
        b.request(Request::Insert {
            id: id(5000),
            fields: fields(50)
        }),
        Response::Ok
    );
    for n in 1..=4096 {
        assert_eq!(
            a.request(Request::GetById { id: id(n) }),
            Response::Record {
                id: id(n),
                fields: fields(n as i64)
            }
        );
    }
    assert_eq!(
        a.request(Request::GetById { id: id(4097) }),
        Response::Record {
            id: id(4097),
            fields: fields(4097)
        }
    );
    assert_eq!(b.request(update(4097, 999)), Response::Ok);
    assert_eq!(
        a.request(Request::GetById { id: id(4097) }),
        Response::Record {
            id: id(4097),
            fields: fields(999)
        }
    );
    assert_eq!(
        a.request(Request::GetById { id: id(5000) }),
        Response::Record {
            id: id(5000),
            fields: fields(50)
        }
    );
    assert_eq!(
        a.request(Request::Query {
            select: Selection::All,
            filter: vec![guard(999)],
            limit: None
        }),
        Response::Rows {
            rows: vec![(id(999), fields(999)), (id(4097), fields(999))]
        }
    );
    assert_eq!(a.request(Request::Commit), Response::Ok);
    assert_eq!(
        table.snapshot().read_sets.last().unwrap().len(),
        MAX_TRACKED_READS
    );
    a.close();
    b.close();
}
#[test]
fn read_your_writes_ignores_bad_kind_unknown_field_read_only_and_missing_record() {
    let mut table = TestStore::new("table", None, &[(1, 1)]);
    std::sync::Arc::get_mut(&mut table).unwrap().schema.fields[0]
        .capabilities
        .update = false;
    let mut c = Client::new(registry(vec![table], 0));
    assert_eq!(c.request(Request::BeginWith { flags: 1 }), Response::Ok);
    assert_eq!(c.request(update(1, 9)), Response::Staged { index: 0 });
    assert_eq!(
        c.request(Request::GetById { id: id(1) }),
        Response::Record {
            id: id(1),
            fields: fields(1)
        }
    );
    c.close();
    let mut c = Client::new(registry(vec![TestStore::new("table", None, &[(1, 1)])], 0));
    assert_eq!(c.request(Request::BeginWith { flags: 1 }), Response::Ok);
    for (i, req) in [
        update(99, 9),
        Request::UpdateField {
            id: id(1),
            field: 99,
            value: ScanValue::I64(9),
        },
        Request::UpdateField {
            id: id(1),
            field: 1,
            value: ScanValue::Bool(true),
        },
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(c.request(req), Response::Staged { index: i as u32 });
    }
    assert_eq!(
        c.request(Request::GetById { id: id(99) }),
        Response::NotFound
    );
    assert_eq!(
        c.request(Request::GetById { id: id(1) }),
        Response::Record {
            id: id(1),
            fields: fields(1)
        }
    );
    c.close();
}
#[test]
fn all_eleven_session_guards_have_zero_effect_and_use_keeps_active_table() {
    let left = TestStore::new("left", Some("right"), &[(1, 1)]);
    let right = TestStore::new("right", None, &[(2, 2)]);
    let mut c = Client::new(registry(vec![left.clone(), right.clone()], 0));
    assert_eq!(c.request(link(1, 2)), Response::Ok);
    let before = (left.snapshot(), right.snapshot());
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for table in [&left, &right] {
        let calls = calls.clone();
        *table.observer.lock().unwrap() = Some(std::sync::Arc::new(move |event| {
            if event != "get" {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }));
    }
    assert_eq!(c.request(Request::Begin), Response::Ok);
    let guarded = vec![
        Request::Begin,
        Request::BeginWith { flags: 7 },
        Request::Transaction {
            updates: vec![op(1, 9)],
        },
        Request::Insert {
            id: id(9),
            fields: fields(9),
        },
        link(1, 2),
        Request::Replace {
            id: id(1),
            fields: fields(9),
        },
        Request::Delete { id: id(1) },
        Request::Compact,
        Request::ReplaceIf {
            id: id(1),
            fields: fields(9),
            guard: guard(1),
        },
        Request::WriteBatch {
            ops: vec![WriteOp::Delete { id: id(1) }],
            atomic: false,
        },
        Request::Use {
            table: "right".into(),
        },
    ];
    assert_eq!(guarded.len(), 11);
    for request in guarded {
        assert_error(c.request(request), ErrorCode::SessionOpen);
        assert_eq!((left.snapshot(), right.snapshot()), before);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(
            c.request(Request::GetById { id: id(1) }),
            Response::Record {
                id: id(1),
                fields: fields(1)
            }
        );
    }
    assert_eq!(c.request(Request::Rollback), Response::Ok);
    // Frozen wording correction: the refused deletion's effect is absent; the record remains.
    assert_eq!(
        c.request(Request::GetById { id: id(1) }),
        Response::Record {
            id: id(1),
            fields: fields(1)
        }
    );
    assert_eq!((left.snapshot(), right.snapshot()), before);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    c.close();
}
#[test]
fn staging_4096_then_session_full_preserves_exact_successful_batch() {
    let table = TestStore::new("table", None, &[(1, 0)]);
    let mut c = Client::new(registry(vec![table.clone()], 0));
    assert_eq!(c.request(Request::Begin), Response::Ok);
    for i in 0..MAX_STAGED_OPS {
        assert_eq!(
            c.request(update(1, i as i64)),
            Response::Staged { index: i as u32 }
        );
    }
    assert_error(c.request(update(1, -999)), ErrorCode::SessionFull);
    assert_eq!(table.snapshot().writes, 0);
    assert_eq!(c.request(Request::Commit), Response::Ok);
    let state = table.snapshot();
    assert_eq!(state.writes, 4096);
    assert_eq!(state.transaction_sizes, vec![4096]);
    assert_eq!(state.rows.get(&id(1)), Some(&fields(4095)));
    c.close();
}
#[test]
fn session_without_stage_validation_fails_at_commit_and_closes() {
    let table = TestStore::new("table", None, &[(1, 1)]);
    let mut c = Client::new(registry(vec![table.clone()], 0));
    assert_error(c.request(Request::Rollback), ErrorCode::NoSession);
    assert_eq!(c.request(Request::Begin), Response::Ok);
    assert_eq!(c.request(update(1, 10)), Response::Staged { index: 0 });
    assert_eq!(c.request(update(99, 10)), Response::Staged { index: 1 });
    assert_failed(c.request(Request::Commit), 1, ErrorCode::RecordNotFound);
    assert_eq!(table.snapshot().writes, 0);
    assert_error(c.request(Request::Rollback), ErrorCode::NoSession);
    c.close();
}
