//! Injected I/O and byte-level faults only: not OS-crash or power-loss evidence.
use exp1_record_format::{Body, ScanLimits, encode, scan_with_limits};
use std::{
    cell::Cell,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use uc_core::{
    Durability, HISTORY_FILE, Log, LogError, Outcome, Rejection, Transaction, Writer, identity,
    sha256,
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "uc-recovery-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Log::<State>::create(&p, apply, decode).unwrap();
        Self(p)
    }
    fn history(&self) -> PathBuf {
        self.0.join(HISTORY_FILE)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let p = self.0.canonicalize().unwrap();
        assert_eq!(
            p.parent().unwrap(),
            std::env::temp_dir().canonicalize().unwrap()
        );
        assert!(
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("uc-recovery-")
        );
        fs::remove_dir_all(p).unwrap();
    }
}
type State = Vec<u8>;
fn apply(s: &mut State, payload: &[u8]) -> Result<(), String> {
    s.extend(payload);
    Ok(())
}
fn decode(s: &[u8]) -> Result<State, String> {
    Ok(s.to_vec())
}
fn open(p: &Path) -> Result<(Log<State>, uc_core::OpenReport), LogError> {
    Log::open(p, apply, decode)
}
fn txn(n: u64, mode: Durability) -> Transaction {
    Transaction {
        request_id: identity(n, 0x52),
        payload: vec![n as u8],
        durability: mode,
    }
}
fn commit(log: &mut Log<State>, n: u64, mode: Durability) -> Outcome {
    log.commit(txn(n, mode), |_| Ok(())).unwrap()
}
fn frames(bytes: &[u8]) -> Vec<exp1_record_format::Record> {
    scan_with_limits(
        bytes,
        ScanLimits {
            max_diagnostic_bytes: bytes.len(),
            ..ScanLimits::default()
        },
    )
    .records
}
fn write_frames(path: &Path, records: &[exp1_record_format::Record]) {
    fs::write(
        path,
        records
            .iter()
            .flat_map(|r| encode(r).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap();
}

#[test]
fn rejected_conflict_has_no_bytes_and_never_recovers() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D2);
    let before = fs::read(t.history()).unwrap();
    assert_eq!(
        log.commit(txn(2, Durability::D2), |_| Err(Rejection::Validation(
            "conflict".into()
        )))
        .unwrap(),
        Outcome::Rejected {
            reason: Rejection::Validation("conflict".into())
        }
    );
    assert_eq!(fs::read(t.history()).unwrap(), before);
    drop(log);
    assert_eq!(&*open(&t.0).unwrap().0.snapshot(), &[1]);
}
#[test]
fn lost_response_retry_and_changed_request_or_durability_are_resolved_without_append() {
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        let outcome = commit(&mut log, 1, mode);
        drop(log);
        let (mut log, _) = open(&t.0).unwrap();
        let before = fs::read(t.history()).unwrap();
        assert_eq!(
            log.commit(txn(1, mode), |_| panic!("retry must precede validator"))
                .unwrap(),
            outcome
        );
        let mut changed = txn(1, mode);
        changed.payload.push(99);
        assert_eq!(
            log.commit(changed, |_| panic!()).unwrap(),
            Outcome::Rejected {
                reason: Rejection::RequestIdReuse
            }
        );
        let mut oversized_reuse = txn(1, mode);
        oversized_reuse.payload = vec![0; uc_core::MAX_PAYLOAD + 1];
        assert_eq!(
            log.commit(oversized_reuse, |_| panic!(
                "retry precedes size/validation"
            ))
            .unwrap(),
            Outcome::Rejected {
                reason: Rejection::RequestIdReuse
            }
        );
        let other = if mode == Durability::D1 {
            Durability::D2
        } else {
            Durability::D1
        };
        assert_eq!(
            commit(&mut log, 1, other),
            Outcome::Rejected {
                reason: Rejection::RequestIdReuse
            }
        );
        assert_eq!(fs::read(t.history()).unwrap(), before);
    }
}
#[test]
fn publication_is_one_immutable_snapshot_and_timestamp_is_persisted() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    let old = log.snapshot();
    let observed = Arc::clone(&old);
    log.set_hook(move |_| assert!(observed.is_empty()));
    let outcome = commit(&mut log, 1, Durability::D2);
    assert!(old.is_empty());
    assert_eq!(&*log.snapshot(), &[1]);
    let records = frames(&fs::read(t.history()).unwrap());
    let Body::Final {
        durability_time, ..
    } = records[3].body
    else {
        panic!()
    };
    assert!(
        matches!(outcome,Outcome::Committed { durability_time: time,.. } if time==durability_time)
    );
}
#[derive(Clone, Copy, Debug)]
enum Fault {
    Short,
    Zero,
    Error,
    SubmittedError,
    Sync,
}
struct Script {
    inner: Box<dyn Writer + Send>,
    fault: Fault,
    at: usize,
    writes: usize,
    syncs: Cell<usize>,
}
impl Write for Script {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.writes == self.at {
            match self.fault {
                Fault::Short => return self.inner.write(&b[..b.len() / 2]),
                Fault::Zero => return Ok(0),
                Fault::Error => return Err(io::Error::other("injected write")),
                Fault::SubmittedError => {
                    self.inner.write_all(b)?;
                    return Err(io::Error::other("injected after submission"));
                }
                Fault::Sync => {}
            }
        }
        self.inner.write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
impl Writer for Script {
    fn synchronize(&self) -> io::Result<()> {
        self.syncs.set(self.syncs.get() + 1);
        if matches!(self.fault, Fault::Sync) && self.syncs.get() == self.at {
            return Err(io::Error::other("injected sync"));
        }
        self.inner.synchronize()
    }
}
#[test]
fn injected_short_zero_write_and_sync_errors_at_every_step_poison_until_reopen() {
    for mode in [Durability::D1, Durability::D2] {
        for fault in [
            Fault::Short,
            Fault::Zero,
            Fault::Error,
            Fault::SubmittedError,
            Fault::Sync,
        ] {
            if matches!(fault, Fault::Sync) && mode == Durability::D1 {
                continue;
            }
            let steps = if matches!(fault, Fault::Sync) { 4 } else { 5 };
            for at in 1..=steps {
                let t = Temp::new();
                let (mut log, _) = open(&t.0).unwrap();
                log.wrap_writer(|inner| {
                    Box::new(Script {
                        inner,
                        fault,
                        at,
                        writes: 0,
                        syncs: Cell::new(0),
                    })
                });
                let outcome = commit(&mut log, 1, mode);
                let boundary = at >= 4;
                assert_eq!(
                    matches!(outcome, Outcome::Indeterminate { .. }),
                    boundary,
                    "{mode:?} {fault:?} {at}: {outcome:?}"
                );
                assert!(log.is_poisoned());
                assert!(log.snapshot().is_empty());
                let before = fs::read(t.history()).unwrap();
                assert!(matches!(
                    log.commit(txn(2, mode), |_| panic!()),
                    Err(LogError::Poisoned)
                ));
                assert_eq!(fs::read(t.history()).unwrap(), before);
                drop(log);
                let (mut log, report) = open(&t.0).unwrap();
                let committed = (matches!(fault, Fault::Sync) && at == 4)
                    || (matches!(fault, Fault::SubmittedError) && at == 5);
                assert_eq!(
                    !log.snapshot().is_empty(),
                    committed,
                    "{mode:?} {fault:?} {at}"
                );
                if !report.unresolved_bindings.is_empty() {
                    assert_eq!(
                        commit(&mut log, 1, mode),
                        Outcome::Rejected {
                            reason: Rejection::Uncommitted
                        }
                    );
                }
                let next = commit(&mut log, 2, mode);
                assert!(matches!(next, Outcome::Committed { .. }));
                drop(log);
                open(&t.0).unwrap();
            }
        }
    }
}
#[test]
fn injected_byte_tails_at_every_boundary_and_inside_commit_only_replay_selected_events() {
    for mode in [Durability::D1, Durability::D2] {
        let source = Temp::new();
        let (mut log, _) = open(&source.0).unwrap();
        commit(&mut log, 1, mode);
        commit(&mut log, 2, mode);
        drop(log);
        let bytes = fs::read(source.history()).unwrap();
        let records = frames(&bytes);
        let mut offsets = vec![0];
        for record in &records {
            offsets.push(offsets.last().unwrap() + encode(record).unwrap().len());
        }
        for (i, &offset) in offsets.iter().enumerate() {
            let t = Temp::new();
            fs::write(t.history(), &bytes[..offset]).unwrap();
            let (mut log, report) = open(&t.0).unwrap();
            assert_eq!(log.snapshot().len(), i / 5);
            assert!(report.dropped_tail.is_none());
            if i % 5 == 4 {
                assert_eq!(report.uncommitted_finals.len(), 1);
            }
            if i % 5 >= 2 {
                assert_eq!(report.gaps.len(), 1);
            }
            assert!(matches!(
                commit(&mut log, 100, mode),
                Outcome::Committed { .. }
            ));
            drop(log);
            open(&t.0).unwrap();
        }
        // A Final followed by a Commit truncated one byte inside its frame is valid residue.
        for end in [
            offsets[4] + 1,
            offsets[5] - 1,
            offsets[9] + 1,
            offsets[10] - 1,
        ] {
            let t = Temp::new();
            fs::write(t.history(), &bytes[..end]).unwrap();
            let (log, report) = open(&t.0).unwrap();
            let accepted = if end < offsets[5] {
                offsets[4]
            } else {
                offsets[9]
            };
            assert_eq!(report.dropped_tail, Some(accepted as u64));
            assert!(report.tail_truncated_and_synced);
            assert_eq!(report.uncommitted_finals.len(), 1);
            assert_eq!(fs::metadata(t.history()).unwrap().len(), accepted as u64);
            assert_eq!(log.snapshot().len(), usize::from(end > offsets[5]));
        }
    }
}
#[test]
fn injected_interior_final_and_provisional_damage_fail_closed_at_exact_offset() {
    for index in [2, 3] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        commit(&mut log, 1, Durability::D2);
        commit(&mut log, 2, Durability::D2);
        drop(log);
        let mut bytes = fs::read(t.history()).unwrap();
        let records = frames(&bytes);
        let offset: usize = records[..index]
            .iter()
            .map(|r| encode(r).unwrap().len())
            .sum();
        bytes[offset + 40] ^= 1;
        fs::write(t.history(), &bytes).unwrap();
        assert!(matches!(open(&t.0),Err(LogError::Corrupt {offset:o,..}) if o==offset as u64));
        assert_eq!(fs::read(t.history()).unwrap(), bytes);
    }
}
#[test]
fn every_final_envelope_mismatch_with_valid_rf1_crcs_fails_closed() {
    // magic, request, event, sequence, realtime, mode, both explicit-empty counts,
    // normalized magic/mode/length/payload and trailing byte are independently mutated.
    for index in [0, 5, 21, 37, 45, 53, 54, 56, 58, 63, 64, 72, usize::MAX] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        commit(&mut log, 1, Durability::D2);
        drop(log);
        let mut records = frames(&fs::read(t.history()).unwrap());
        let Body::Final {
            complete_envelope, ..
        } = &mut records[3].body
        else {
            panic!()
        };
        if index == usize::MAX {
            complete_envelope.push(0);
        } else {
            complete_envelope[index] ^= 0x04;
        }
        let final_bytes = encode(&records[3]).unwrap();
        let Body::Commit { final_crc32c, .. } = &mut records[4].body else {
            panic!()
        };
        *final_crc32c = u32::from_le_bytes(final_bytes[28..32].try_into().unwrap());
        write_frames(&t.history(), &records);
        assert!(
            matches!(
                open(&t.0),
                Err(LogError::EnvelopeMismatch {
                    physical_ordinal: 4
                })
            ),
            "field {index}"
        );
        // Envelope validation also applies to valid unselected Final residue.
        write_frames(&t.history(), &records[..4]);
        assert!(matches!(
            open(&t.0),
            Err(LogError::EnvelopeMismatch {
                physical_ordinal: 4
            })
        ));
    }
}

#[test]
fn valid_but_different_envelope_durability_is_not_an_upgrade() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D1);
    drop(log);
    let mut records = frames(&fs::read(t.history()).unwrap());
    let Body::Final {
        complete_envelope, ..
    } = &mut records[3].body
    else {
        panic!()
    };
    complete_envelope[53] = 2;
    let encoded = encode(&records[3]).unwrap();
    let Body::Commit { final_crc32c, .. } = &mut records[4].body else {
        panic!()
    };
    *final_crc32c = u32::from_le_bytes(encoded[28..32].try_into().unwrap());
    write_frames(&t.history(), &records);
    assert!(matches!(
        open(&t.0),
        Err(LogError::EnvelopeMismatch {
            physical_ordinal: 4
        })
    ));
}

#[test]
fn successful_writer_records_exact_r5_sync_placements_and_three_payload_copies() {
    use std::sync::Mutex;
    struct Observe {
        inner: Box<dyn Writer + Send>,
        calls: Arc<Mutex<Vec<u8>>>,
    }
    impl Write for Observe {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.calls.lock().unwrap().push(b[6]);
            self.inner.write(b)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }
    impl Writer for Observe {
        fn synchronize(&self) -> io::Result<()> {
            self.calls.lock().unwrap().push(0);
            self.inner.synchronize()
        }
    }
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        log.wrap_writer(|inner| {
            Box::new(Observe {
                inner,
                calls: Arc::clone(&calls),
            })
        });
        let observed = Arc::clone(&calls);
        log.set_hook(move |point| {
            let kind = match point {
                uc_core::Point::Binding => 1,
                uc_core::Point::Reservation => 2,
                uc_core::Point::Provisional => 3,
                _ => return,
            };
            assert_eq!(
                observed.lock().unwrap().last(),
                Some(&if mode == Durability::D2 { 0 } else { kind })
            );
        });
        let payload = b"unique complete opaque payload".to_vec();
        let transaction = Transaction {
            request_id: identity(1, 0x52),
            payload: payload.clone(),
            durability: mode,
        };
        assert!(matches!(
            log.commit(transaction, |_| Ok(())).unwrap(),
            Outcome::Committed { .. }
        ));
        assert_eq!(
            *calls.lock().unwrap(),
            if mode == Durability::D1 {
                vec![1, 2, 3, 5, 6]
            } else {
                vec![1, 0, 2, 0, 3, 0, 5, 6, 0]
            }
        );
        calls.lock().unwrap().clear();
        log.checkpoint(|s| Ok(s.clone())).unwrap();
        assert_eq!(
            *calls.lock().unwrap(),
            vec![0],
            "history sync before checkpoint at {mode:?}"
        );
        let bytes = fs::read(t.history()).unwrap();
        assert_eq!(
            bytes
                .windows(payload.len())
                .filter(|w| *w == payload)
                .count(),
            3
        );
    }
}

#[test]
fn checkpoint_is_create_new_and_newest_valid_checkpoint_falls_back_after_injected_partial_write() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D2);
    let first = log.checkpoint(|s| Ok(s.clone())).unwrap();
    assert!(matches!(
        log.checkpoint(|s| Ok(s.clone())),
        Err(LogError::Io {
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    commit(&mut log, 2, Durability::D2);
    let second = log.checkpoint(|s| Ok(s.clone())).unwrap();
    drop(log);
    fs::write(&second.path, b"UC1\0partial").unwrap();
    let (log, report) = open(&t.0).unwrap();
    assert_eq!(report.checkpoint, Some(first.position));
    assert_eq!(report.rejected_checkpoints.len(), 1);
    assert_eq!(&*log.snapshot(), &[1, 2]);
}
#[test]
fn checkpoint_replays_only_later_commits_and_full_replays_have_identical_digests() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D2);
    let cp = log.checkpoint(|s| Ok(s.clone())).unwrap();
    commit(&mut log, 2, Durability::D2);
    drop(log);
    let (log, report) = open(&t.0).unwrap();
    assert_eq!(report.checkpoint, Some(cp.position));
    let expected = sha256(&log.snapshot());
    assert_eq!(&*log.snapshot(), &[1, 2]);
    drop(log);
    fs::remove_file(cp.path).unwrap();
    for _ in 0..2 {
        let (log, report) = open(&t.0).unwrap();
        assert!(report.checkpoint.is_none());
        assert_eq!(sha256(&log.snapshot()), expected);
    }
}
fn fix_cp_crc(bytes: &mut [u8]) {
    let n = bytes.len() - 4;
    let crc = uc_core::crc32c(&bytes[..n]);
    bytes[n..].copy_from_slice(&crc.to_le_bytes());
}
#[test]
fn injected_checkpoint_corruption_hash_mismatch_and_cache_disagreement_have_distinct_results() {
    for kind in 0..3 {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        commit(&mut log, 1, Durability::D2);
        let cp = log.checkpoint(|s| Ok(s.clone())).unwrap();
        drop(log);
        let mut bytes = fs::read(&cp.path).unwrap();
        match kind {
            0 => {
                bytes.truncate(20);
            } // interrupted checkpoint body: rejected, full replay
            1 => {
                bytes[28] ^= 1;
                fix_cp_crc(&mut bytes);
            } // valid checkpoint CRC, wrong prefix hash: fallback
            _ => {
                let n = bytes.len();
                bytes[n - 5] ^= 1;
                fix_cp_crc(&mut bytes);
            } // valid cache framing, disagreement
        }
        fs::write(cp.path, bytes).unwrap();
        if kind == 2 {
            assert!(matches!(open(&t.0), Err(LogError::CheckpointCacheMismatch)));
        } else {
            let (log, report) = open(&t.0).unwrap();
            assert_eq!(report.rejected_checkpoints.len(), 1);
            assert_eq!(&*log.snapshot(), &[1]);
        }
    }
}
#[test]
fn injected_lost_committed_suffix_below_valid_checkpoint_fails_closed() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D2);
    let offset = log.history_bytes();
    commit(&mut log, 2, Durability::D2);
    let cp = log.checkpoint(|s| Ok(s.clone())).unwrap();
    drop(log);
    fs::OpenOptions::new()
        .write(true)
        .open(t.history())
        .unwrap()
        .set_len(offset)
        .unwrap();
    let error = match open(&t.0) {
        Err(e) => e,
        Ok(_) => panic!("lost suffix accepted"),
    };
    assert!(
        error
            .to_string()
            .contains("history shorter than checkpoint")
    );
    // Negative control: without the independent checkpoint the loss is bytewise undecidable.
    fs::remove_file(cp.path).unwrap();
    assert_eq!(&*open(&t.0).unwrap().0.snapshot(), &[1]);
}
#[test]
fn reservation_high_water_gap_survives_reopen_and_binding_never_expires() {
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    commit(&mut log, 1, Durability::D2);
    drop(log);
    let mut records = frames(&fs::read(t.history()).unwrap());
    records.truncate(2);
    let Body::Reservation { high_water, .. } = &mut records[1].body else {
        panic!()
    };
    *high_water = 99;
    write_frames(&t.history(), &records);
    let (mut log, report) = open(&t.0).unwrap();
    assert_eq!(report.gaps, vec![1]);
    assert_eq!(
        commit(&mut log, 1, Durability::D2),
        Outcome::Rejected {
            reason: Rejection::Uncommitted
        }
    );
    assert!(matches!(
        commit(&mut log, 2, Durability::D2),
        Outcome::Committed { sequence: 100, .. }
    ));
    log.checkpoint(|s| Ok(s.clone())).unwrap();
    drop(log);
    let (mut log, _) = open(&t.0).unwrap();
    assert_eq!(
        commit(&mut log, 1, Durability::D2),
        Outcome::Rejected {
            reason: Rejection::Uncommitted
        }
    );
}
#[test]
fn sha256_known_vector_and_oversize_transaction_rejected_before_mutation() {
    assert_eq!(
        sha256(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad
        ]
    );
    let t = Temp::new();
    let (mut log, _) = open(&t.0).unwrap();
    let mut too_big = txn(1, Durability::D1);
    too_big.payload = vec![0; uc_core::MAX_PAYLOAD + 1];
    assert!(matches!(
        log.commit(too_big, |_| panic!()),
        Err(LogError::Limit)
    ));
    assert_eq!(log.history_bytes(), 0);
    assert!(!log.is_poisoned());
}

#[test]
fn rf1_magic_payload_is_rejected_before_write_and_old_binding_is_corrupt() {
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        let mut request = txn(1, mode);
        request.payload = b"before RDE1 after".to_vec();
        assert!(
            matches!(log.commit(request, |_| panic!("before validation")),
            Err(LogError::Invalid(reason)) if reason == "payload contains the RF1 magic")
        );
        assert_eq!(log.history_bytes(), 0);
        assert!(fs::read(t.history()).unwrap().is_empty());
        assert!(!log.is_poisoned());
        commit(&mut log, 1, mode);
        drop(log);
        let mut records = frames(&fs::read(t.history()).unwrap());
        let Body::Binding {
            normalized_request, ..
        } = &mut records[0].body
        else {
            panic!()
        };
        normalized_request[6..14].copy_from_slice(&4u64.to_le_bytes());
        normalized_request.truncate(14);
        normalized_request.extend(b"RDE1");
        // Legitimate RF1 encoding and CRC; rejected by the normalized-request decoder.
        write_frames(&t.history(), &records[..1]);
        assert!(
            matches!(open(&t.0), Err(LogError::Corrupt { offset: 0, error })
            if error == "payload contains the RF1 magic")
        );
    }
}

#[test]
fn injected_torn_binding_with_magic_in_valid_uuid_remains_corrupt_undecidable() {
    // Payload exclusion cannot constrain opaque UUID bytes in the RF1 body.
    // HEADER_LEN is 32; the scanner searches for magic after that header.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let (mut log, _) = open(&t.0).unwrap();
        let mut request = txn(1, mode);
        request.request_id.0[..4].copy_from_slice(b"RDE1");
        request.request_id.validate_v4().unwrap();
        assert!(matches!(
            log.commit(request, |_| Ok(())).unwrap(),
            Outcome::Committed { .. }
        ));
        drop(log);
        let records = frames(&fs::read(t.history()).unwrap());
        let binding = encode(&records[0]).unwrap();
        assert_eq!(&binding[32..36], b"RDE1");
        fs::write(t.history(), &binding[..binding.len() - 1]).unwrap();
        assert!(
            matches!(open(&t.0), Err(LogError::Corrupt { offset: 0, error })
            if error.contains("InteriorDamage"))
        );
    }
}

#[test]
fn replay_io_failure_is_distinct_from_byte_corruption() {
    let t = Temp::new();
    // Metadata succeeds; reopening/reading a directory as a history fails at byte zero.
    assert!(matches!(
        uc_core::read_history(&t.0),
        Err(LogError::Io { offset: 0, .. })
    ));
}

#[test]
fn open_never_creates_missing_directory_or_history_and_create_is_exclusive() {
    let t = Temp::new();
    let missing = t.0.join("missing");
    assert!(matches!(open(&missing), Err(LogError::NotFound)));
    assert!(!missing.exists());
    assert!(matches!(
        uc_core::Directory::acquire(&missing),
        Err(LogError::NotFound)
    ));
    assert!(!missing.exists());
    fs::create_dir(&missing).unwrap();
    assert!(matches!(open(&missing), Err(LogError::NotFound)));
    assert_eq!(fs::read_dir(&missing).unwrap().count(), 0);
    assert!(matches!(
        Log::create(&missing, apply, decode),
        Err(LogError::Io {
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    assert_eq!(fs::read_dir(&missing).unwrap().count(), 0);
    let existing = fs::read(t.history()).unwrap();
    assert!(matches!(
        Log::create(&t.0, apply, decode),
        Err(LogError::Io {
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    assert_eq!(fs::read(t.history()).unwrap(), existing);
    let fresh = missing.join("fresh");
    let (log, _) = Log::create(&fresh, apply, decode).unwrap();
    assert!(log.snapshot().is_empty());
    drop(log);
    open(&fresh).unwrap();
}
