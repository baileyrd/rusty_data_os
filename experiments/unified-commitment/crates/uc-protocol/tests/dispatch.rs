mod support;
use support::*;
use uc_protocol::*;

#[test]
fn defaults_refuse_atomic_batches_before_any_single_shot_write_or_callback() {
    let table = TestStore::new("table", None, &[]);
    let store = InsertOnly(table.clone());
    let ops = vec![
        WriteOp::Insert {
            id: id(1),
            fields: fields(1),
        },
        WriteOp::Delete { id: id(1) },
    ];
    assert_eq!(
        store.write_batch_checked(&[], &|_| panic!("empty callback")),
        Ok(vec![])
    );
    assert_eq!(
        store.write_batch_checked(&ops, &|_| panic!("refusal callback")),
        Err((0, ErrorCode::Unsupported))
    );
    assert_eq!(
        store.write_batch(&ops, true),
        Err((0, ErrorCode::Unsupported))
    );
    assert_failed(
        dispatch(
            &store,
            Request::WriteBatch {
                ops: ops.clone(),
                atomic: true,
            },
        ),
        0,
        ErrorCode::Unsupported,
    );
    assert_eq!(table.snapshot().writes, 0);
    assert_eq!(
        store.write_batch(&ops, false),
        Ok(vec![
            WriteResult::Inserted,
            WriteResult::Failed(ErrorCode::Unsupported)
        ])
    );
    assert_eq!(table.get(id(1)), Some(fields(1)));
}
#[test]
fn optional_methods_and_conservative_relation_defaults_match_legacy() {
    let table = TestStore::new("table", None, &[(1, 1)]);
    let store = DefaultStore(table);
    assert_eq!(store.table_name(), "table");
    assert_eq!(
        store.describe_relations(),
        vec![
            RelationDescriptor {
                name: "neighbors".into(),
                kind: JoinRelation::Neighbors(None),
                target_table: None
            },
            RelationDescriptor {
                name: "edge".into(),
                kind: JoinRelation::Neighbors(Some("edge".into())),
                target_table: None
            }
        ]
    );
    let mut schema = schema();
    schema.relations.parent_children = true;
    schema.relations.neighbors = false;
    assert_eq!(
        default_relation_descriptors(&schema, vec!["edge".into()]),
        vec![]
    );
    assert_eq!(
        store.insert_record(id(2), fields(2)),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(
        store.replace_record(id(1), fields(2)),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(
        store.replace_record_if(id(1), fields(2), &guard(1)),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(
        store.link_records(id(1), id(2), "edge"),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(store.delete_record(id(1)), Err(ErrorCode::Unsupported));
    assert_eq!(
        store.detach_record("edge", id(1)),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(store.compact(), Err(ErrorCode::Unsupported));
    assert_eq!(store.count_edges("edge"), Err(ErrorCode::Unsupported));
}
#[test]
fn apply_write_op_maps_every_outcome_and_pipelined_failures_continue() {
    let s = TestStore::new("table", None, &[(1, 1), (2, 2)]);
    let ops = vec![
        WriteOp::Insert {
            id: id(3),
            fields: fields(3),
        },
        WriteOp::Insert {
            id: id(3),
            fields: fields(3),
        },
        WriteOp::Replace {
            id: id(3),
            fields: fields(4),
        },
        WriteOp::Replace {
            id: id(9),
            fields: fields(4),
        },
        WriteOp::ReplaceIf {
            id: id(3),
            fields: fields(5),
            guard: guard(4),
        },
        WriteOp::ReplaceIf {
            id: id(3),
            fields: fields(6),
            guard: guard(0),
        },
        WriteOp::ReplaceIf {
            id: id(9),
            fields: fields(6),
            guard: guard(0),
        },
        batch_link(1, 2),
        batch_link(1, 2),
        WriteOp::Delete { id: id(3) },
        WriteOp::Delete { id: id(3) },
        WriteOp::Insert {
            id: id(4),
            fields: vec![],
        },
        WriteOp::Insert {
            id: id(4),
            fields: fields(4),
        },
    ];
    assert_eq!(
        s.write_batch(&ops, false),
        Ok(vec![
            WriteResult::Inserted,
            WriteResult::Duplicate,
            WriteResult::Replaced,
            WriteResult::NotFound,
            WriteResult::Replaced,
            WriteResult::GuardFailed,
            WriteResult::NotFound,
            WriteResult::Linked,
            WriteResult::AlreadyLinked,
            WriteResult::Deleted,
            WriteResult::NotFound,
            WriteResult::Failed(ErrorCode::Malformed),
            WriteResult::Inserted
        ])
    );
    assert_eq!(
        s.apply_write_op(&WriteOp::ReplaceIf {
            id: id(1),
            fields: fields(3),
            guard: Predicate {
                field: 99,
                ..guard(1)
            }
        }),
        WriteResult::Failed(ErrorCode::UnknownField)
    );
}
#[test]
fn only_connection_state_variants_have_unsupported_fallbacks() {
    let s = TestStore::new("table", None, &[]);
    for request in [
        Request::Authenticate { token: "x".into() },
        Request::Hello {
            protocol_version: 22,
        },
        Request::Begin,
        Request::BeginWith { flags: 7 },
        Request::Commit,
        Request::Rollback,
    ] {
        assert_error(dispatch(s.as_ref(), request), ErrorCode::Unsupported);
    }
    assert_eq!(
        dispatch(s.as_ref(), Request::Transaction { updates: vec![] }),
        Response::Ok
    );
    assert_eq!(
        dispatch(
            s.as_ref(),
            Request::Use {
                table: "table".into()
            }
        ),
        Response::Ok
    );
    assert_error(
        dispatch(
            s.as_ref(),
            Request::Use {
                table: "other".into(),
            },
        ),
        ErrorCode::Malformed,
    );
    assert_eq!(
        dispatch(s.as_ref(), Request::ListTables),
        Response::Tables {
            names: vec!["table".into()],
            primary: "table".into()
        }
    );
}
#[test]
fn dispatch_single_shot_read_write_schema_and_relation_outcomes() {
    let s = TestStore::new("table", None, &[(1, 1), (2, 2)]);
    let d = |r| dispatch(s.as_ref(), r);
    assert_eq!(d(Request::DescribeSchema), Response::Schema(schema()));
    assert_eq!(
        d(Request::DescribeRelations),
        Response::Relations {
            relations: s.describe_relations()
        }
    );
    assert_eq!(
        d(Request::ListRelationKinds),
        Response::RelationKinds {
            kinds: vec!["edge".into()]
        }
    );
    assert_eq!(
        d(Request::FilterEq {
            field: 1,
            value: ScanValue::I64(2)
        }),
        Response::RecordList {
            records: vec![id(2)]
        }
    );
    assert_eq!(
        d(Request::ScanField { field: 1 }),
        Response::ScanValues {
            values: vec![ScanValue::I64(1), ScanValue::I64(2)]
        }
    );
    assert_eq!(d(Request::Parent { id: id(1) }), Response::NoParent);
    assert_eq!(d(Request::Parent { id: id(99) }), Response::NotFound);
    assert_eq!(
        d(Request::Children { id: id(1) }),
        Response::RecordList { records: vec![] }
    );
    assert_eq!(d(link(1, 2)), Response::Ok);
    assert_eq!(
        d(Request::Neighbors { id: id(1) }),
        Response::RecordList {
            records: vec![id(2)]
        }
    );
    assert_eq!(
        d(Request::Insert {
            id: id(3),
            fields: fields(3)
        }),
        Response::Ok
    );
    assert_error(
        d(Request::Insert {
            id: id(3),
            fields: fields(3),
        }),
        ErrorCode::Duplicate,
    );
    assert_eq!(
        d(Request::Replace {
            id: id(3),
            fields: fields(4)
        }),
        Response::Ok
    );
    assert_eq!(
        d(Request::Replace {
            id: id(99),
            fields: fields(4)
        }),
        Response::NotFound
    );
    assert_error(
        d(Request::ReplaceIf {
            id: id(3),
            fields: fields(5),
            guard: guard(0),
        }),
        ErrorCode::GuardFailed,
    );
    assert_eq!(
        d(Request::ReplaceIf {
            id: id(3),
            fields: fields(5),
            guard: guard(4)
        }),
        Response::Ok
    );
    assert_eq!(d(update(3, 6)), Response::Ok);
    assert_eq!(d(update(99, 6)), Response::NotFound);
    assert_eq!(
        d(Request::GetById { id: id(3) }),
        Response::Record {
            id: id(3),
            fields: fields(6)
        }
    );
    assert_eq!(d(Request::Delete { id: id(3) }), Response::Ok);
    assert_eq!(d(Request::Delete { id: id(3) }), Response::NotFound);
    assert_eq!(
        d(Request::Compact),
        Response::Compacted {
            records: 2,
            slots_reclaimed: 0,
            log_entries_folded: 0,
            edge_logs_folded: 0
        }
    );
}
#[test]
fn query_validation_projection_filter_order_and_limit() {
    let s = TestStore::new("table", None, &[(1, 3), (2, 1), (3, 3)]);
    let query = |select, filter, limit| {
        dispatch(
            s.as_ref(),
            Request::Query {
                select,
                filter,
                limit,
            },
        )
    };
    assert_eq!(
        query(Selection::All, vec![guard(3)], Some(1)),
        Response::Rows {
            rows: vec![(id(1), fields(3))]
        }
    );
    assert_eq!(
        query(Selection::Fields(vec![]), vec![], Some(1)),
        Response::Rows {
            rows: vec![(id(1), vec![])]
        }
    );
    assert_eq!(
        query(Selection::All, vec![], Some(0)),
        Response::Rows { rows: vec![] }
    );
    assert_error(
        query(Selection::Fields(vec![99]), vec![], None),
        ErrorCode::UnknownField,
    );
    assert_error(
        query(
            Selection::All,
            vec![Predicate {
                value: ScanValue::Bool(true),
                ..guard(1)
            }],
            None,
        ),
        ErrorCode::Malformed,
    );
    for op in [
        CompareOp::Eq,
        CompareOp::Ne,
        CompareOp::Lt,
        CompareOp::Le,
        CompareOp::Gt,
        CompareOp::Ge,
    ] {
        let expected = matches!(op, CompareOp::Eq | CompareOp::Le | CompareOp::Ge);
        assert_eq!(
            predicate_matches(&fields(1), &Predicate { op, ..guard(1) }),
            expected
        );
    }
    let mut schema = schema();
    for kind in [ValueKind::Str, ValueKind::Bool, ValueKind::StrList] {
        schema.fields[0].value_kind = kind;
        assert_eq!(
            validate_predicate(
                &schema,
                &Predicate {
                    field: 1,
                    op: CompareOp::Lt,
                    value: match kind {
                        ValueKind::Str => ScanValue::Str("x".into()),
                        ValueKind::Bool => ScanValue::Bool(true),
                        _ => ScanValue::StrList(vec![]),
                    }
                }
            ),
            Err(ErrorCode::Malformed)
        );
    }
}
#[test]
fn aggregate_count_sum_avg_extremes_groups_and_empty_identity() {
    let s = TestStore::new("table", None, &[(1, 3), (2, -1), (3, 3)]);
    let specs = vec![
        AggregateSpec {
            func: AggregateFn::Count,
            field: None,
        },
        AggregateSpec {
            func: AggregateFn::Sum,
            field: Some(1),
        },
        AggregateSpec {
            func: AggregateFn::Avg,
            field: Some(1),
        },
        AggregateSpec {
            func: AggregateFn::Min,
            field: Some(1),
        },
        AggregateSpec {
            func: AggregateFn::Max,
            field: Some(1),
        },
    ];
    let aggregate = |group_by, filter, limit| {
        dispatch(
            s.as_ref(),
            Request::Aggregate {
                group_by,
                filter,
                aggregates: specs.clone(),
                limit,
            },
        )
    };
    assert_eq!(
        aggregate(vec![], vec![], None),
        Response::Groups {
            groups: vec![AggregateGroup {
                key: vec![],
                values: vec![
                    ScanValue::I64(3),
                    ScanValue::I64(5),
                    ScanValue::F64(5.0 / 3.0),
                    ScanValue::I64(-1),
                    ScanValue::I64(3)
                ]
            }]
        }
    );
    assert_eq!(
        aggregate(vec![], vec![guard(99)], None),
        Response::Groups {
            groups: vec![AggregateGroup {
                key: vec![],
                values: vec![
                    ScanValue::I64(0),
                    ScanValue::I64(0),
                    ScanValue::F64(0.0),
                    ScanValue::I64(0),
                    ScanValue::I64(0)
                ]
            }]
        }
    );
    assert_eq!(
        aggregate(vec![1], vec![guard(99)], None),
        Response::Groups { groups: vec![] }
    );
    assert_eq!(
        aggregate(vec![1], vec![], Some(1)),
        Response::Groups {
            groups: vec![AggregateGroup {
                key: fields(3),
                values: vec![
                    ScanValue::I64(2),
                    ScanValue::I64(6),
                    ScanValue::F64(3.0),
                    ScanValue::I64(3),
                    ScanValue::I64(3)
                ]
            }]
        }
    );
    assert_error(aggregate(vec![99], vec![], None), ErrorCode::UnknownField);
    for spec in [
        AggregateSpec {
            func: AggregateFn::Count,
            field: Some(1),
        },
        AggregateSpec {
            func: AggregateFn::Sum,
            field: None,
        },
    ] {
        assert_error(
            dispatch(
                s.as_ref(),
                Request::Aggregate {
                    group_by: vec![],
                    filter: vec![],
                    aggregates: vec![spec],
                    limit: None,
                },
            ),
            ErrorCode::Malformed,
        );
    }
}
#[test]
fn numeric_pages_use_uuid_tie_break_exclusive_cursor_and_validate_before_scan() {
    let s = DefaultStore(TestStore::new("table", None, &[(1, 3), (2, -1), (3, 3)]));
    assert_eq!(
        dispatch(
            &s,
            Request::Page {
                order_by: 1,
                after: None,
                limit: 2
            }
        ),
        Response::Rows {
            rows: vec![(id(2), fields(-1)), (id(1), fields(3))]
        }
    );
    assert_eq!(
        dispatch(
            &s,
            Request::Page {
                order_by: 1,
                after: Some((ScanValue::I64(3), id(1))),
                limit: 10
            }
        ),
        Response::Rows {
            rows: vec![(id(3), fields(3))]
        }
    );
    assert_error(
        dispatch(
            &s,
            Request::Page {
                order_by: 99,
                after: None,
                limit: 1,
            },
        ),
        ErrorCode::UnknownField,
    );
    assert_error(
        dispatch(
            &s,
            Request::Page {
                order_by: 1,
                after: None,
                limit: 0,
            },
        ),
        ErrorCode::Malformed,
    );
    assert_error(
        dispatch(
            &s,
            Request::Page {
                order_by: 1,
                after: Some((ScanValue::U32(3), id(1))),
                limit: 1,
            },
        ),
        ErrorCode::Malformed,
    );
}
