use super::*;
#[path = "mod.rs"]
mod support;
use std::{
    sync::{
        TryLockError,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use support::*;

struct ReadNotice {
    pipe: Pipe,
    read: usize,
    notify_at: usize,
    notice: mpsc::Sender<()>,
}
impl Read for ReadNotice {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.pipe.read(bytes)?;
        self.read += count;
        if self.read == self.notify_at {
            self.notice.send(()).unwrap();
        }
        Ok(count)
    }
}
impl Write for ReadNotice {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pipe.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.pipe.flush()
    }
}

/// Production path plus a test-only negative control removing the mutex.
/// A captures a found far endpoint and pauses before returning from get.
/// B consumes Delete on a separate duplex pair. With the shared mutex, B
/// must wait through A's link, then deletes/detaches it. Without it, force B
/// to finish first: A then creates a dangling edge from the stale observation.
#[test]
fn shared_registry_two_connection_interleaving_prevents_dangling_edge() {
    for protected in [true, false] {
        let a = TestStore::new("a", Some("b"), &[(1, 1)]);
        let b = TestStore::new("b", None, &[(2, 2)]);
        let mut registration = (*registry(vec![a.clone(), b.clone()], 0)).clone();
        if !protected {
            registration.relationship_lock = None;
        } // explicit test-only negative control
        let registry = Arc::new(registration);
        let cloned = Arc::new((*registry).clone());
        if protected {
            assert!(Arc::ptr_eq(
                registry.relationship_lock.as_ref().unwrap(),
                cloned.relationship_lock.as_ref().unwrap()
            ));
        }
        let mut ca = Client::new(registry.clone());
        let switch = Request::Use { table: "b".into() };
        let delete = Request::Delete { id: id(2) };
        let (mut cb, server) = duplex();
        let (notice_tx, notice_rx) = mpsc::channel();
        let mut server = ReadNotice {
            pipe: server,
            read: 0,
            notify_at: 8 + encode_request(&switch).len() + encode_request(&delete).len(),
            notice: notice_tx,
        };
        let tb = thread::spawn(move || handle_connection(&mut server, &cloned));
        write_message(&mut cb, &encode_request(&switch)).unwrap();
        assert_eq!(
            decode_response(&read_message(&mut cb).unwrap()).unwrap(),
            Response::Ok
        );
        let (checked_tx, checked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let first_get = AtomicBool::new(true);
        let events = Arc::new(Mutex::new(Vec::new()));
        let weak = Arc::downgrade(&registry);
        let observed = events.clone();
        *b.observer.lock().unwrap() = Some(Arc::new(move |event| {
            if (event == "get" && first_get.swap(false, Ordering::SeqCst)) || event == "delete" {
                if protected {
                    let reg = weak.upgrade().unwrap();
                    assert!(matches!(
                        reg.relationship_lock.as_ref().unwrap().try_lock(),
                        Err(TryLockError::WouldBlock)
                    ));
                }
                observed.lock().unwrap().push(event.to_string());
                if event == "get" {
                    checked_tx.send(()).unwrap();
                    release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(10))
                        .unwrap();
                }
            }
        }));
        let weak = Arc::downgrade(&registry);
        let observed = events.clone();
        *a.observer.lock().unwrap() = Some(Arc::new(move |event| {
            if matches!(event, "link" | "detach") {
                if protected {
                    let reg = weak.upgrade().unwrap();
                    assert!(matches!(
                        reg.relationship_lock.as_ref().unwrap().try_lock(),
                        Err(TryLockError::WouldBlock)
                    ));
                }
                observed.lock().unwrap().push(event.to_string());
            }
        }));
        ca.send(link(1, 2));
        checked_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        write_message(&mut cb, &encode_request(&delete)).unwrap();
        notice_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        if protected {
            assert!(matches!(
                registry.relationship_lock.as_ref().unwrap().try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            assert_eq!(*events.lock().unwrap(), ["get"]);
            release_tx.send(()).unwrap();
            assert_eq!(ca.receive(), Response::Ok);
            assert_eq!(
                decode_response(&read_message(&mut cb).unwrap()).unwrap(),
                Response::Ok
            );
            assert_eq!(*events.lock().unwrap(), ["get", "link", "delete", "detach"]);
            assert!(a.snapshot().edges.is_empty());
        } else {
            assert_eq!(
                decode_response(&read_message(&mut cb).unwrap()).unwrap(),
                Response::Ok
            );
            release_tx.send(()).unwrap();
            assert_eq!(ca.receive(), Response::Ok);
            assert_eq!(*events.lock().unwrap(), ["get", "delete", "detach", "link"]);
            assert_eq!(
                a.snapshot().edges.iter().copied().collect::<Vec<_>>(),
                vec![(id(1), id(2))]
            );
        }
        assert!(!b.snapshot().rows.contains_key(&id(2)));
        ca.close();
        drop(cb);
        tb.join().unwrap().unwrap();
    }
}
#[test]
fn reads_other_writes_sessions_and_compact_do_not_acquire_relationship_mutex() {
    let table = TestStore::new("a", Some("b"), &[(1, 1)]);
    let registry = registry(vec![table], 0);
    let mut client = Client::new(registry.clone());
    let held = registry.relationship_lock.as_ref().unwrap().lock().unwrap();
    assert_eq!(
        client.request(Request::GetById { id: id(1) }),
        Response::Record {
            id: id(1),
            fields: fields(1)
        }
    );
    assert_eq!(client.request(update(1, 2)), Response::Ok);
    assert_eq!(
        client.request(Request::Insert {
            id: id(2),
            fields: fields(2)
        }),
        Response::Ok
    );
    assert_eq!(client.request(Request::Begin), Response::Ok);
    assert_eq!(client.request(update(1, 3)), Response::Staged { index: 0 });
    assert_eq!(client.request(Request::Commit), Response::Ok);
    assert!(matches!(
        client.request(Request::Compact),
        Response::Compacted { .. }
    ));
    drop(held);
    client.close();
}

#[test]
fn join_holds_relationship_lock_until_row_fetch_finishes_before_delete_reinsert() {
    let a = TestStore::new("a", Some("b"), &[(1, 1)]);
    let b = TestStore::new("b", None, &[(2, 2)]);
    a.state.lock().unwrap().edges.insert((id(1), id(2)));
    let registry = registry(vec![a.clone(), b.clone()], 0);
    let mut ca = Client::new(registry.clone());
    let switch = Request::Use { table: "b".into() };
    let delete = Request::Delete { id: id(2) };
    let (mut cb, pipe) = duplex();
    let (notice_tx, notice_rx) = mpsc::channel();
    let mut server = ReadNotice {
        pipe,
        read: 0,
        notify_at: 8 + encode_request(&switch).len() + encode_request(&delete).len(),
        notice: notice_tx,
    };
    let cloned = registry.clone();
    let worker = thread::spawn(move || handle_connection(&mut server, &cloned));
    write_message(&mut cb, &encode_request(&switch)).unwrap();
    assert_eq!(
        decode_response(&read_message(&mut cb).unwrap()).unwrap(),
        Response::Ok
    );
    let (paused_tx, paused_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let events = Arc::new(Mutex::new(Vec::new()));
    let observed = events.clone();
    let weak = Arc::downgrade(&registry);
    *b.observer.lock().unwrap() = Some(Arc::new(move |event| {
        if event == "get" || event == "delete" {
            let reg = weak.upgrade().unwrap();
            assert!(matches!(
                reg.relationship_lock.as_ref().unwrap().try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            observed.lock().unwrap().push(event.to_owned());
            if event == "get" {
                paused_tx.send(()).unwrap();
                release_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .unwrap();
                observed.lock().unwrap().push("join-fetch-released".into());
            }
        }
    }));
    ca.send(Request::Join(JoinSpec {
        relation: JoinRelation::Neighbors(Some("edge".into())),
        right_table: Some("b".into()),
        left: Selection::All,
        right: Selection::All,
        left_filter: vec![],
        right_filter: vec![],
        limit: None,
    }));
    paused_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    write_message(&mut cb, &encode_request(&delete)).unwrap();
    write_message(
        &mut cb,
        &encode_request(&Request::Insert {
            id: id(2),
            fields: fields(9),
        }),
    )
    .unwrap();
    notice_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    // B has consumed Delete while A is paused inside get with the actual mutex held.
    assert!(matches!(
        registry.relationship_lock.as_ref().unwrap().try_lock(),
        Err(TryLockError::WouldBlock)
    ));
    assert_eq!(*events.lock().unwrap(), ["get"]);
    release_tx.send(()).unwrap();
    assert_eq!(
        ca.receive(),
        Response::JoinedRows {
            rows: vec![JoinedRow {
                left_id: id(1),
                left: fields(1),
                right_id: id(2),
                right: fields(2),
            }]
        }
    );
    assert_eq!(
        decode_response(&read_message(&mut cb).unwrap()).unwrap(),
        Response::Ok
    );
    assert_eq!(
        decode_response(&read_message(&mut cb).unwrap()).unwrap(),
        Response::Ok
    );
    assert_eq!(
        *events.lock().unwrap(),
        ["get", "join-fetch-released", "delete"]
    );
    assert_eq!(b.snapshot().rows.get(&id(2)), Some(&fields(9)));
    ca.close();
    drop(cb);
    worker.join().unwrap().unwrap();
}
