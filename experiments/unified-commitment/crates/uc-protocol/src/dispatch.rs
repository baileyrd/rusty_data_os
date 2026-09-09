use crate::query::*;
use crate::*;

/// Frozen protocol error text; describes legacy wire outcomes, not a durability claim.
pub fn error_message(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::UnknownField => "unrecognized field tag for this domain",
        ErrorCode::Unsupported => "this operation is not available for this field/domain",
        ErrorCode::Malformed => "the supplied value does not match this field's type",
        ErrorCode::Unauthenticated => "this connection has not presented a recognized token",
        ErrorCode::Unauthorized => "this connection's token does not permit this operation",
        ErrorCode::RecordNotFound => "this operation's id has no record",
        ErrorCode::NoSession => "no transaction session is open on this connection",
        ErrorCode::SessionOpen => "a transaction session is already open on this connection",
        ErrorCode::SessionFull => "this session already holds the maximum number of staged writes",
        ErrorCode::Journal => {
            "journal I/O failed; pre-apply refusals apply nothing, but failed durable cleanup leaves the transaction outcome indeterminate and writes are refused until reopen"
        }
        ErrorCode::Conflict => {
            "this session's read set no longer matches current state; nothing was applied"
        }
        ErrorCode::Duplicate => "a record with this id already exists; nothing was written",
        ErrorCode::Storage => "the record could not be made durable; nothing was written",
        ErrorCode::GuardFailed => {
            "the guard did not hold against the stored record; nothing was written"
        }
    }
}
pub fn err_response(code: ErrorCode) -> Response {
    Response::Err {
        code,
        message: error_message(code).into(),
    }
}
pub(crate) fn transaction_failed((index, code): (usize, ErrorCode)) -> Response {
    Response::TransactionFailed {
        index,
        code,
        message: error_message(code).into(),
    }
}
fn result<T>(outcome: Result<T, ErrorCode>, success: impl FnOnce(T) -> Response) -> Response {
    outcome.map(success).unwrap_or_else(err_response)
}

/// Table-local dispatch. Connection state and registry-aware effects belong to
/// handle_connection; calling this directly cannot protect foreign relationships.
pub fn dispatch<S: Store + ?Sized>(store: &S, req: Request) -> Response {
    match req {
        Request::GetById { id } => store
            .get(id)
            .map(|fields| Response::Record { id, fields })
            .unwrap_or(Response::NotFound),
        Request::FilterEq { field, value } => result(store.filter_eq(field, &value), |records| {
            Response::RecordList { records }
        }),
        Request::ScanField { field } => result(store.scan_field(field), |values| {
            Response::ScanValues { values }
        }),
        Request::UpdateField { id, field, value } => {
            result(store.update_field(id, field, value), |found| {
                if found {
                    Response::Ok
                } else {
                    Response::NotFound
                }
            })
        }
        Request::Parent { id } => result(store.parent(id), |v| match v {
            ParentLookup::Parent(id) => Response::Id { id },
            ParentLookup::NoParent => Response::NoParent,
            ParentLookup::ChildNotFound => Response::NotFound,
        }),
        Request::Children { id } => result(store.children(id), |records| Response::RecordList {
            records,
        }),
        Request::Neighbors { id } => result(store.neighbors(id), |records| Response::RecordList {
            records,
        }),
        Request::NeighborsByRelation { id, relation } => {
            result(store.neighbors_by_relation(id, &relation), |records| {
                Response::RecordList { records }
            })
        }
        Request::ListRelationKinds => Response::RelationKinds {
            kinds: store.list_relation_kinds(),
        },
        Request::DescribeRelations => Response::Relations {
            relations: store.describe_relations(),
        },
        Request::DescribeSchema => Response::Schema(store.describe()),
        Request::Transaction { updates } => store
            .apply_transaction(&updates, &[])
            .map(|()| Response::Ok)
            .unwrap_or_else(transaction_failed),
        Request::Query {
            select,
            filter,
            limit,
        } => result(validate_query(&store.describe(), &select, &filter), |()| {
            Response::Rows {
                rows: evaluate_query(store.scan_all(), &select, &filter, limit),
            }
        }),
        Request::Aggregate {
            group_by,
            filter,
            aggregates,
            limit,
        } => {
            let schema = store.describe();
            result(
                validate_aggregate(&schema, &group_by, &filter, &aggregates),
                |()| {
                    result(
                        evaluate_aggregate(
                            store.scan_all(),
                            &group_by,
                            &filter,
                            &aggregates,
                            limit,
                            &schema,
                        ),
                        |groups| Response::Groups { groups },
                    )
                },
            )
        }
        Request::Join(spec) => result(
            validate_join(&store.describe(), &store.describe_relations(), None, &spec),
            |()| Response::JoinedRows {
                rows: evaluate_join(store, store, &spec),
            },
        ),
        Request::Insert { id, fields } => result(store.insert_record(id, fields), |v| match v {
            InsertOutcome::Inserted => Response::Ok,
            InsertOutcome::Duplicate => err_response(ErrorCode::Duplicate),
        }),
        Request::Replace { id, fields } => result(store.replace_record(id, fields), |v| match v {
            ReplaceOutcome::Replaced => Response::Ok,
            ReplaceOutcome::NotFound => Response::NotFound,
        }),
        Request::ReplaceIf { id, fields, guard } => {
            if let Err(code) = validate_predicate(&store.describe(), &guard) {
                return err_response(code);
            }
            result(store.replace_record_if(id, fields, &guard), |v| match v {
                ReplaceIfOutcome::Replaced => Response::Ok,
                ReplaceIfOutcome::NotFound => Response::NotFound,
                ReplaceIfOutcome::GuardFailed => err_response(ErrorCode::GuardFailed),
            })
        }
        Request::Link {
            left,
            right,
            relation,
        } => result(store.link_records(left, right, &relation), |_| Response::Ok),
        Request::Delete { id } => result(store.delete_record(id), |v| match v {
            DeleteOutcome::Deleted => Response::Ok,
            DeleteOutcome::NotFound => Response::NotFound,
        }),
        Request::Compact => result(store.compact(), |v| Response::Compacted {
            records: v.records as u64,
            slots_reclaimed: v.slots_reclaimed as u64,
            log_entries_folded: v.log_entries_folded as u64,
            edge_logs_folded: v.edge_logs_folded as u64,
        }),
        Request::Page {
            order_by,
            after,
            limit,
        } => {
            if let Err(code) = validate_page(&store.describe(), order_by, after.as_ref(), limit) {
                return err_response(code);
            }
            let Ok(limit) = usize::try_from(limit) else {
                return err_response(ErrorCode::Malformed);
            };
            result(store.page(order_by, after, limit), |rows| Response::Rows {
                rows,
            })
        }
        Request::CountEdges { relation } => result(store.count_edges(&relation), |count| {
            Response::Count { count }
        }),
        Request::WriteBatch { ops, atomic } => store
            .write_batch(&ops, atomic)
            .map(|results| Response::BatchResults { results })
            .unwrap_or_else(transaction_failed),
        Request::Use { table } => {
            if table == store.table_name() {
                Response::Ok
            } else {
                err_response(ErrorCode::Malformed)
            }
        }
        Request::ListTables => Response::Tables {
            names: vec![store.table_name().into()],
            primary: store.table_name().into(),
        },
        // Exhaustiveness fallback only: the served loop intercepts these six variants.
        Request::Authenticate { .. }
        | Request::Hello { .. }
        | Request::Begin
        | Request::BeginWith { .. }
        | Request::Commit
        | Request::Rollback => err_response(ErrorCode::Unsupported),
    }
}
