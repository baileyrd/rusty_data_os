#[cfg(test)]
mod tests {
    use rusty_multimodal_db::generic::entity::{
        create_entity_production_stack, open_entity_production_stack_portable, Entity,
    };
    use rusty_multimodal_db::generic::production::GenericProductionStore;
    use rusty_multimodal_db::server::entity::{EntityConnectionStore, FIELD_MENTION_COUNT};
    use rusty_multimodal_db::server::framing::{read_message, write_message};
    use rusty_multimodal_db::server::protocol::{
        ErrorCode, Request, Response, ScanValue, TransactionOp, PROTOCOL_VERSION,
        SESSION_SNAPSHOT_ISOLATION,
    };
    use rusty_multimodal_db::server::{serve, ConnectionStore, ServeOptions};
    use std::net::{TcpListener, TcpStream};
    use std::path::{Path, PathBuf};
    use std::sync::{atomic::{AtomicU64, Ordering}, Arc};
    use uuid::Uuid;

    fn fresh(name: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "multimodal-review-{name}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample() -> Entity {
        Entity { id: Uuid::from_u128(1), label: "A".into(), kind: "person".into(),
                 mention_count: 1, aliases: vec![] }
    }

    fn adapter(dir: &Path, journaled: bool, reopen: bool) -> EntityConnectionStore {
        let path = dir.join("entities.mmap");
        let stack = if reopen {
            open_entity_production_stack_portable(&path).unwrap()
        } else {
            create_entity_production_stack(vec![sample()], &[], &[], &path).unwrap()
        };
        let store = GenericProductionStore::new(stack);
        if journaled {
            EntityConnectionStore::with_journal(store, &dir.join("txn.journal")).unwrap()
        } else { EntityConnectionStore::new(store) }
    }

    fn op(value: i64) -> TransactionOp {
        TransactionOp { id: sample().id, field: FIELD_MENTION_COUNT, value: ScanValue::I64(value) }
    }

    fn count(store: &EntityConnectionStore) -> i64 {
        match store.get(sample().id).unwrap().into_iter().find(|(f, _)| *f == FIELD_MENTION_COUNT).unwrap().1 {
            ScanValue::I64(v) => v, other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn rejected_conflicting_transaction_must_stay_rejected_after_reopen() {
        let dir = fresh("rejected");
        let store = adapter(&dir, true, false);
        let result = store.apply_transaction(&[op(99)], &[(sample().id, FIELD_MENTION_COUNT, ScanValue::I64(0))]);
        assert_eq!(result, Err((0, ErrorCode::Conflict)));
        assert_eq!(count(&store), 1);
        drop(store);
        let reopened = adapter(&dir, true, true);
        assert_eq!(count(&reopened), 1, "a rejected transaction became visible after reopen");
    }

    #[test]
    fn acknowledged_delete_after_transaction_must_allow_reopen() {
        let dir = fresh("delete");
        let store = adapter(&dir, true, false);
        store.apply_transaction(&[op(10)], &[]).unwrap();
        store.delete_record(sample().id).unwrap();
        assert!(store.get(sample().id).is_none());
        drop(store);
        let reopened = adapter(&dir, true, true);
        assert!(reopened.get(sample().id).is_none());
    }

    #[test]
    fn acknowledged_replacement_must_not_be_rewound_by_older_journal() {
        let dir = fresh("replace");
        let store = adapter(&dir, true, false);
        store.apply_transaction(&[op(10)], &[]).unwrap();
        store.replace_record(sample().id, vec![
            (0, ScanValue::Str("New A".into())), (1, ScanValue::Str("person".into())),
            (2, ScanValue::I64(20)), (3, ScanValue::StrList(vec![])),
        ]).unwrap();
        assert_eq!(count(&store), 20);
        // Compact flushes the current slot image too, removing dependence
        // on whether an unflushed mmap write survives a system crash.
        store.compact().unwrap();
        drop(store);
        let reopened = adapter(&dir, true, true);
        assert_eq!(count(&reopened), 20, "older journal rewound a later durable replacement");
    }

    fn exchange(stream: &mut TcpStream, req: Request) -> Response {
        write_message(stream, &req).unwrap();
        read_message(stream).unwrap()
    }

    #[test]
    fn snapshot_session_must_repeat_its_read() {
        let dir = fresh("snapshot");
        let store = Arc::new(adapter(&dir, false, false));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let serving = Arc::clone(&store);
        std::thread::spawn(move || serve(listener, serving, ServeOptions::default()));
        let mut client = TcpStream::connect(addr).unwrap();
        client.set_nodelay(true).unwrap();
        client.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        assert_eq!(exchange(&mut client, Request::Hello { protocol_version: PROTOCOL_VERSION }),
                   Response::Hello { protocol_version: PROTOCOL_VERSION });
        assert_eq!(exchange(&mut client, Request::BeginWith { flags: SESSION_SNAPSHOT_ISOLATION }), Response::Ok);
        let first = exchange(&mut client, Request::GetById { id: sample().id });
        store.update_field(sample().id, FIELD_MENTION_COUNT, ScanValue::I64(2)).unwrap();
        let second = exchange(&mut client, Request::GetById { id: sample().id });
        let commit = exchange(&mut client, Request::Commit);
        eprintln!("first={first:?}; second={second:?}; commit={commit:?}");
        assert_eq!(second, first, "snapshot session observed a later commit");
    }
}
