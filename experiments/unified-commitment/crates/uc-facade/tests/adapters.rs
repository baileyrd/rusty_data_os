mod support;
use support::*;
use uc_protocol::*;

#[test]
fn exact_fields_capabilities_validation_and_soft_outcomes() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    let names: [&[&str]; 3] = [
        &[
            "content",
            "category",
            "tags",
            "source",
            "metadata_json",
            "created_at_unix_ms",
            "updated_at_unix_ms",
            "memory_type",
            "status",
            "sensitive",
            "access_count",
            "deleted_at_unix_ms",
            "node_id",
        ],
        &["label", "kind", "mention_count", "aliases"],
        &[
            "subject",
            "relation",
            "object",
            "created_at_unix_ms",
            "updated_at_unix_ms",
            "node_id",
            "deleted_at_unix_ms",
        ],
    ];
    for (d, s) in stores.iter().enumerate() {
        let schema = s.describe();
        assert_eq!(
            schema
                .fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            names[d]
        );
        let filter_tags: &[FieldRef] = match d {
            0 => &[1],
            1 => &[0, 1],
            _ => &[0],
        };
        for field in &schema.fields {
            assert_eq!(
                field.capabilities.filter_eq,
                filter_tags.contains(&field.tag)
            );
            assert_eq!(field.capabilities.scan, field.tag == UPDATE[d]);
            assert_eq!(field.capabilities.update, field.tag == UPDATE[d]);
            let expected_kind = match &fields(d, 1)[usize::from(field.tag)].1 {
                ScanValue::Str(_) => ValueKind::Str,
                ScanValue::I64(_) => ValueKind::I64,
                ScanValue::Bool(_) => ValueKind::Bool,
                ScanValue::StrList(_) => ValueKind::StrList,
                _ => unreachable!(),
            };
            assert_eq!(field.value_kind, expected_kind);
        }
        assert!(!schema.relations.parent_children);
        assert_eq!(schema.relations.neighbors, d != 2);
        assert!(
            s.describe_relations()
                .iter()
                .all(|r| r.target_table.is_none())
        );
        assert_eq!(s.parent(id(1)), Err(ErrorCode::Unsupported));
        assert_eq!(s.children(id(1)), Err(ErrorCode::Unsupported));
        assert_eq!(s.detach_record("label", id(1)), Err(ErrorCode::Unsupported));
        assert_eq!(s.compact(), Err(ErrorCode::Unsupported));
        for i in 0..fields(d, 1).len() {
            let mut missing = fields(d, 1);
            missing.remove(i);
            let mut duplicate = fields(d, 1);
            duplicate[i].0 = duplicate[(i + 1) % duplicate.len()].0;
            let mut wrong_type = fields(d, 1);
            wrong_type[i].1 = ScanValue::U32(1);
            let mut float = fields(d, 1);
            float[i].1 = ScanValue::F64(1.0);
            for invalid in [missing, duplicate, wrong_type, float] {
                assert_eq!(
                    s.insert_record(id(1), invalid.clone()),
                    Err(ErrorCode::Malformed)
                );
                assert_eq!(s.replace_record(id(1), invalid), Err(ErrorCode::Malformed));
            }
        }
        let mut extra = fields(d, 1);
        extra.push((99, ScanValue::I64(1)));
        assert_eq!(s.insert_record(id(1), extra), Err(ErrorCode::Malformed));
        let mut unknown = fields(d, 1);
        unknown[0].0 = 99;
        assert_eq!(s.insert_record(id(1), unknown), Err(ErrorCode::Malformed));
        assert!(s.scan_all().is_empty());
        assert_eq!(
            s.replace_record(id(1), fields(d, 1)),
            Ok(ReplaceOutcome::NotFound)
        );
        assert_eq!(s.delete_record(id(1)), Ok(DeleteOutcome::NotFound));
        assert_eq!(
            s.update_field(id(1), UPDATE[d], ScanValue::I64(1)),
            Ok(false)
        );
        let mut shuffled = fields(d, 1);
        shuffled.reverse();
        assert_eq!(
            s.insert_record(id(1), shuffled),
            Ok(InsertOutcome::Inserted)
        );
        assert_eq!(s.get(id(1)), Some(fields(d, 1)));
        assert_eq!(
            s.insert_record(id(1), fields(d, 99)),
            Ok(InsertOutcome::Duplicate)
        );
        for &tag in filter_tags {
            assert_eq!(
                s.filter_eq(tag, &fields(d, 1)[usize::from(tag)].1),
                Ok(vec![id(1)])
            );
            assert_eq!(
                s.filter_eq(tag, &ScanValue::Str("absent".into())),
                Ok(vec![])
            );
            assert_eq!(
                s.filter_eq(tag, &ScanValue::I64(1)),
                Err(ErrorCode::Malformed)
            );
        }
        assert_eq!(
            s.filter_eq(99, &ScanValue::Str("x".into())),
            Err(ErrorCode::Unsupported)
        );
        assert_eq!(s.scan_field(99), Err(ErrorCode::Unsupported));
        assert_eq!(
            s.update_field(id(1), 99, ScanValue::I64(1)),
            Err(ErrorCode::Unsupported)
        );
        assert_eq!(
            s.update_field(id(1), UPDATE[d], ScanValue::Bool(false)),
            Err(ErrorCode::Malformed)
        );
        assert_eq!(s.scan_field(UPDATE[d]), Ok(vec![ScanValue::I64(1)]));
        assert_eq!(s.scan_all(), vec![(id(1), fields(d, 1))]);
        assert_eq!(s.page(UPDATE[d], None, 10), Ok(vec![(id(1), fields(d, 1))]));
    }
    assert_eq!(
        stores[0].update_field(id(1), 10, ScanValue::I64(-1)),
        Err(ErrorCode::Malformed)
    );
    for tag in [10, 11] {
        let mut invalid = fields(0, 1);
        invalid[tag].1 = ScanValue::I64(-1);
        assert_eq!(
            stores[0].replace_record(id(1), invalid),
            Err(ErrorCode::Malformed)
        );
    }
    for tag in [0, 1, 2, 6] {
        let mut invalid = fields(2, 1);
        invalid[tag].1 = if tag == 6 {
            ScanValue::I64(-1)
        } else {
            ScanValue::Str(String::new())
        };
        assert_eq!(
            stores[2].insert_record(id(2), invalid.clone()),
            Err(ErrorCode::Malformed)
        );
        assert_eq!(
            stores[2].replace_record(id(1), invalid),
            Err(ErrorCode::Malformed)
        );
    }
    assert_eq!(stores[2].neighbors(id(1)), Err(ErrorCode::Unsupported));
    assert_eq!(
        stores[2].neighbors_by_relation(id(1), "x"),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(
        stores[2].link_records(id(1), id(2), "x"),
        Err(ErrorCode::Unsupported)
    );
    assert_eq!(stores[2].count_edges("x"), Err(ErrorCode::Unsupported));
    assert!(stores[2].list_relation_kinds().is_empty());
    assert!(stores[2].describe_relations().is_empty());
}

#[test]
fn whole_record_guards_reuse_shared_predicates_and_preserve_records_on_refusal() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    for (d, s) in stores.iter().enumerate() {
        s.insert_record(id(1), fields(d, 10)).unwrap();
        for (op, rhs, passes) in [
            (CompareOp::Eq, 10, true),
            (CompareOp::Ne, 10, false),
            (CompareOp::Lt, 11, true),
            (CompareOp::Le, 9, false),
            (CompareOp::Gt, 9, true),
            (CompareOp::Ge, 11, false),
        ] {
            let guard = Predicate {
                field: UPDATE[d],
                op,
                value: ScanValue::I64(rhs),
            };
            assert_eq!(
                s.replace_record_if(id(1), fields(d, 30), &guard),
                Ok(if passes {
                    ReplaceIfOutcome::Replaced
                } else {
                    ReplaceIfOutcome::GuardFailed
                })
            );
            assert_eq!(s.get(id(1)), Some(fields(d, if passes { 30 } else { 10 })));
            s.replace_record(id(1), fields(d, 10)).unwrap();
        }
        let string_guard = Predicate {
            field: 0,
            op: CompareOp::Eq,
            value: fields(d, 10)[0].1.clone(),
        };
        assert_eq!(
            s.replace_record_if(id(1), fields(d, 20), &string_guard),
            Ok(ReplaceIfOutcome::Replaced)
        );
        assert_eq!(
            s.replace_record_if(id(2), fields(d, 20), &string_guard),
            Ok(ReplaceIfOutcome::NotFound)
        );
        let invalid = Predicate {
            op: CompareOp::Lt,
            ..string_guard
        };
        assert_eq!(
            s.replace_record_if(id(1), fields(d, 99), &invalid),
            Err(ErrorCode::Malformed)
        );
        let invalid = Predicate {
            field: 99,
            op: CompareOp::Eq,
            value: ScanValue::I64(1),
        };
        assert_eq!(
            s.replace_record_if(id(1), fields(d, 99), &invalid),
            Err(ErrorCode::UnknownField)
        );
        assert_eq!(s.get(id(1)), Some(fields(d, 20)));
    }
}

#[test]
fn transactions_check_reads_first_reject_atomically_and_commit_4096_once_with_replay() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    for (d, s) in stores.iter().enumerate() {
        s.insert_record(id(1), fields(d, 1)).unwrap();
        let good = update(d, id(1), 2);
        let missing = update(d, id(2), 3);
        let invalid = TransactionOp {
            field: 99,
            ..good.clone()
        };
        let reads = [(id(1), UPDATE[d], ScanValue::I64(1))];
        assert_eq!(s.validate_op(&good), Ok(()));
        assert_eq!(s.validate_op(&missing), Err(ErrorCode::RecordNotFound));
        assert_eq!(s.validate_op(&invalid), Err(ErrorCode::Unsupported));
        assert_eq!(
            s.apply_transaction(&[good.clone(), missing], &reads),
            Err((1, ErrorCode::RecordNotFound))
        );
        assert_eq!(
            s.apply_transaction(&[good.clone(), invalid.clone()], &reads),
            Err((1, ErrorCode::Unsupported))
        );
        assert_eq!(
            s.apply_transaction(&[invalid], &[(id(1), UPDATE[d], ScanValue::I64(0))]),
            Err((0, ErrorCode::Conflict))
        );
        assert_eq!(s.get(id(1)), Some(fields(d, 1)));
        let oversized = vec![good; MAX_STAGED_OPS + 1];
        assert_eq!(
            s.apply_transaction(&oversized, &reads),
            Err((4096, ErrorCode::SessionFull))
        );
        let updates: Vec<_> = (1..=4096).map(|v| update(d, id(1), v)).collect();
        s.apply_transaction(&updates, &reads).unwrap();
        s.apply_transaction(&[], &[]).unwrap();
        assert_eq!(s.get(id(1)), Some(fields(d, 4096)));
        let history =
            uc_core::read_history(&temp.0.join(s.table_name()).join(uc_core::HISTORY_FILE))
                .unwrap();
        assert_eq!(
            history.events.len(),
            2,
            "one insert plus one unsplit 4096-op transaction"
        );
        let payload = &history.events[1].payload;
        let count = match d {
            0 => uc_memory::decode_changes(payload).unwrap().len(),
            1 => uc_entity::decode_changes(payload).unwrap().len(),
            _ => uc_relation::decode_changes(payload).unwrap().len(),
        };
        assert_eq!(count, 4096);
    }
    drop(stores);
    let reopened = temp.stores(true);
    for (d, s) in reopened.iter().enumerate() {
        assert_eq!(s.get(id(1)), Some(fields(d, 4096)));
    }
}

#[test]
fn same_table_edges_incarnations_and_known_labels_survive_reopen() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    for (d, s) in stores.iter().take(2).enumerate() {
        let label = if d == 0 { "mentions" } else { "custom_label-2" };
        s.insert_record(id(1), fields(d, 1)).unwrap();
        s.insert_record(id(2), fields(d, 2)).unwrap();
        assert_eq!(
            s.link_records(id(1), id(2), "bad label"),
            Err(ErrorCode::Malformed)
        );
        assert_eq!(
            s.link_records(id(1), id(3), label),
            Err(ErrorCode::RecordNotFound)
        );
        assert_eq!(s.link_records(id(1), id(2), label), Ok(LinkOutcome::Linked));
        assert_eq!(
            s.link_records(id(2), id(1), label),
            Ok(LinkOutcome::AlreadyLinked)
        );
        assert_eq!(s.neighbors(id(2)), Ok(vec![id(1)]));
        assert_eq!(s.neighbors_by_relation(id(1), label), Ok(vec![id(2)]));
        assert_eq!(s.count_edges(label), Ok(1));
        assert_eq!(
            s.neighbors_by_relation(id(1), "unknown"),
            Err(ErrorCode::Malformed)
        );
        assert!(s.list_relation_kinds().contains(&label.into()));
        assert!(s.describe_relations().contains(&RelationDescriptor {
            name: label.into(),
            kind: JoinRelation::Neighbors(Some(label.into())),
            target_table: None
        }));
    }
    drop(stores);
    let stores = temp.stores(true);
    for (d, s) in stores.iter().enumerate() {
        if d == 2 {
            s.insert_record(id(1), fields(d, 1)).unwrap();
        } else {
            assert_eq!(s.neighbors(id(1)), Ok(vec![id(2)]));
        }
        s.delete_record(id(1)).unwrap();
        s.insert_record(id(1), fields(d, 3)).unwrap();
        s.update_field(id(1), UPDATE[d], ScanValue::I64(4)).unwrap();
        assert_eq!(s.get(id(1)), Some(fields(d, 4)));
        if d < 2 {
            assert_eq!(s.neighbors(id(2)), Ok(vec![]));
        }
    }
    drop(stores);
    let stores = temp.stores(true);
    for (d, s) in stores.iter().enumerate() {
        assert_eq!(s.get(id(1)), Some(fields(d, 4)));
        if d < 2 {
            assert_eq!(s.neighbors(id(2)), Ok(vec![]));
        }
    }
    assert!(
        stores[1]
            .list_relation_kinds()
            .contains(&"custom_label-2".into())
    );
}

#[test]
fn shared_predicate_admission_refuses_string_list_guards() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    for (domain, tag) in [(0, 2), (1, 3)] {
        let store = &stores[domain];
        store.insert_record(id(1), fields(domain, 1)).unwrap();
        for op in [CompareOp::Eq, CompareOp::Ne] {
            let guard = Predicate {
                field: tag,
                op,
                value: fields(domain, 1)[usize::from(tag)].1.clone(),
            };
            // The frozen shared validator omits StrList even for Eq/Ne. Preserve
            // that admission rule; do not implement a second predicate language.
            assert_eq!(
                validate_predicate(&store.describe(), &guard),
                Err(ErrorCode::Malformed)
            );
            assert_eq!(
                store.replace_record_if(id(1), fields(domain, 2), &guard),
                Err(ErrorCode::Malformed)
            );
            assert_eq!(store.get(id(1)), Some(fields(domain, 1)));
        }
    }
}

#[test]
fn concurrent_guarded_replacements_have_one_winner_per_table() {
    let temp = Temp::new();
    let stores = temp.stores(false);
    for (d, store) in stores.iter().enumerate() {
        store.insert_record(id(1), fields(d, 1)).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let workers: Vec<_> = (2..=3)
            .map(|v| {
                let store = store.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    store
                        .replace_record_if(
                            id(1),
                            fields(d, v),
                            &Predicate {
                                field: UPDATE[d],
                                op: CompareOp::Eq,
                                value: ScanValue::I64(1),
                            },
                        )
                        .unwrap()
                })
            })
            .collect();
        barrier.wait();
        let outcomes: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
        assert_eq!(
            outcomes
                .iter()
                .filter(|o| **o == ReplaceIfOutcome::Replaced)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|o| **o == ReplaceIfOutcome::GuardFailed)
                .count(),
            1
        );
    }
}
