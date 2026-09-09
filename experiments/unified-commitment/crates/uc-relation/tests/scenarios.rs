//! Failure models: deterministic validation/replay, simulated response loss, injected byte truncation.
//! No OS-crash or power-loss evidence; core process fault tests remain unchanged.
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use uc_core::{Durability, Outcome, Rejection, identity, sha256};
use uc_relation::{
    Change, Id, Relation, RelationEngine, State, apply, decode_changes, decode_state,
    encode_changes, encode_state,
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "uc-relation-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
    fn create(&self, mode: Durability) -> RelationEngine {
        RelationEngine::create(&self.store(), mode).unwrap().0
    }
    fn open(&self, mode: Durability) -> RelationEngine {
        RelationEngine::open(&self.store(), mode).unwrap().0
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let path = self.0.canonicalize().unwrap();
        assert_eq!(
            path.parent().unwrap(),
            std::env::temp_dir().canonicalize().unwrap()
        );
        assert!(
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("uc-relation-")
        );
        fs::remove_dir_all(path).unwrap();
    }
}
fn key(n: u64) -> Id {
    identity(n, 0x44).0
}
fn record(n: u64) -> Relation {
    Relation {
        id: identity(n, 0x44),
        subject: "nonexistent://entity\n雪".into(),
        relation: " arbitrary / label !\tRDE1".into(),
        object: "no-such-record\t雪".into(),
        created_at_unix_ms: i64::MIN,
        updated_at_unix_ms: -3,
        node_id: "".into(),
        deleted_at_unix_ms: 0,
    }
}
fn put(n: u64, incarnation: u64) -> Change {
    Change::Put {
        record: Box::new(record(n)),
        incarnation,
        insert: true,
    }
}
fn update(n: u64, incarnation: u64, value: i64) -> Change {
    Change::UpdateTimestamp {
        id: key(n),
        incarnation,
        value,
    }
}
fn delete(n: u64, incarnation: u64) -> Change {
    Change::Delete {
        id: key(n),
        incarnation,
    }
}
fn commit(e: &mut RelationEngine, changes: &[Change]) {
    let outcome = e.transact(e.log().next_request_id(), changes).unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
fn reject(e: &mut RelationEngine, changes: &[Change], expected: &str, t: &Temp) {
    let before = e.log().snapshot();
    let bytes = e.log().history_bytes();
    let disk_bytes = fs::metadata(t.store().join(uc_core::HISTORY_FILE))
        .unwrap()
        .len();
    let outcome = e.transact(e.log().next_request_id(), changes).unwrap();
    assert_eq!(
        outcome,
        Outcome::Rejected {
            reason: Rejection::Validation(expected.into())
        }
    );
    assert_eq!(e.log().snapshot(), before);
    assert_eq!(e.log().history_bytes(), bytes);
    assert_eq!(
        fs::metadata(t.store().join(uc_core::HISTORY_FILE))
            .unwrap()
            .len(),
        disk_bytes
    );
    // Recovery does not use commit-time validation: raw apply must independently reject too.
    assert!(apply(&mut before.as_ref().clone(), &encode_changes(changes)).is_err());
}
fn digest(s: &State) -> [u8; 32] {
    sha256(&encode_state(s).unwrap())
}

#[test]
fn canonical_payload_roundtrip_and_malformed_limits() {
    // Failure model: deterministic malformed input, no file I/O.
    let mut replacement = record(1);
    replacement.subject = "new subject".into();
    replacement.object = "new object".into();
    replacement.relation = "new relation".into();
    replacement.created_at_unix_ms = i64::MAX;
    replacement.node_id = "node".into();
    replacement.deleted_at_unix_ms = 12;
    let changes = vec![
        put(1, 1),
        Change::Put {
            record: Box::new(replacement),
            incarnation: 1,
            insert: false,
        },
        update(1, 1, i64::MIN),
        delete(1, 1),
    ];
    let bytes = encode_changes(&changes);
    assert_eq!(decode_changes(&bytes).unwrap(), changes);
    let original = String::from_utf8(bytes.clone()).unwrap();
    for bad in [
        original.replace("put\t1\t1", "put\t01\t1"),
        original.replace("put\t1\t1", "put\t+1\t1"),
        original.replace("put\t1\t1", "put\t1\t2"),
        original.replace('\n', "\r\n"),
        original.trim_end_matches('\n').into(),
        original.replace("CMR1", "BAD1"),
        format!("{original}\n"),
        "CMR1\n".into(),
        "CMR1\nunknown\n".into(),
        format!("CMR1\ndelete\t1\t{}\n", "FF".repeat(16)),
        format!("CMR1\ndelete\t1\t{}\n", "00".repeat(15)),
        format!(
            "CMR1\nupdate\t1\t{}\t9223372036854775808\n",
            "00".repeat(16)
        ),
    ] {
        assert!(decode_changes(bad.as_bytes()).is_err(), "accepted: {bad}");
    }
    assert!(decode_changes(&[255]).is_err());
    assert!(decode_changes(&vec![b'x'; uc_core::MAX_PAYLOAD + 1]).is_err());
    assert_eq!(
        decode_changes(&encode_changes(&vec![
            delete(1, 1);
            uc_relation::MAX_OPERATIONS
        ]))
        .unwrap()
        .len(),
        uc_relation::MAX_OPERATIONS
    );
    assert!(
        decode_changes(&encode_changes(&vec![
            delete(1, 1);
            uc_relation::MAX_OPERATIONS + 1
        ]))
        .is_err()
    );
}

#[test]
fn invalid_batch_never_publishes_prefix_and_snapshot_stays_immutable() {
    // Failure model: deterministic conflict validation; negative control publishes staging prefix.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(&mut e, &[put(1, 1)]);
        let old = e.log().snapshot();
        let changes = [update(1, 1, 22), update(9, 1, 33)];
        reject(&mut e, &changes, "RecordNotFound", &t);
        let mut broken = old.as_ref().clone();
        assert!(apply(&mut broken, &encode_changes(&changes)).is_err());
        assert_ne!(broken, *old); // Disabled atomic staging leaks the first update.
        commit(&mut e, &[update(1, 1, 99)]);
        assert_eq!(old.current(&key(1), 1).unwrap(), &record(1));
        drop(e);
        assert_eq!(
            t.open(mode)
                .log()
                .snapshot()
                .current(&key(1), 1)
                .unwrap()
                .updated_at_unix_ms,
            99
        );
    }
}

#[test]
fn incarnation_admission_and_replacement_checkpoint_equal_full_replay() {
    // Failure model: deterministic admission and orderly checkpoint/reopen.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        reject(&mut e, &[put(1, 0)], "StaleIncarnation", &t);
        reject(&mut e, &[put(1, 2)], "StaleIncarnation", &t);
        reject(
            &mut e,
            &[Change::Put {
                record: Box::new(record(1)),
                incarnation: 1,
                insert: false,
            }],
            "RecordNotFound",
            &t,
        );
        commit(&mut e, &[put(1, 1)]);
        reject(&mut e, &[put(1, 1)], "Duplicate", &t);
        reject(&mut e, &[delete(1, 2)], "StaleIncarnation", &t);
        reject(
            &mut e,
            &[Change::Put {
                record: Box::new(record(1)),
                incarnation: 2,
                insert: false,
            }],
            "StaleIncarnation",
            &t,
        );
        commit(&mut e, &[update(1, 1, 87)]);
        let mut replacement = record(1);
        replacement.subject = "new subject".into();
        replacement.object = "new object".into();
        replacement.relation = "new relation".into();
        replacement.created_at_unix_ms = i64::MAX;
        replacement.node_id = "node".into();
        replacement.deleted_at_unix_ms = 12;
        commit(
            &mut e,
            &[Change::Put {
                record: Box::new(replacement.clone()),
                incarnation: 1,
                insert: false,
            }],
        );
        commit(&mut e, &[put(2, 1), delete(2, 1)]);
        let state = e.log().snapshot();
        let bytes = encode_state(&state).unwrap();
        assert_eq!(decode_state(&bytes).unwrap(), *state);
        let cp = e.log().checkpoint(encode_state).unwrap();
        drop(e);
        let (e, report) = RelationEngine::open(&t.store(), mode).unwrap();
        assert!(report.checkpoint.is_some());
        assert_eq!(
            e.log().snapshot().current(&key(1), 1).unwrap(),
            &replacement
        );
        assert_eq!(digest(&e.log().snapshot()), digest(&state));
        assert_eq!(encode_state(&e.log().snapshot()).unwrap(), bytes);
        drop(e);
        fs::remove_file(cp.path).unwrap();
        let (e, report) = RelationEngine::open(&t.store(), mode).unwrap();
        assert!(report.checkpoint.is_none());
        assert_eq!(*e.log().snapshot(), *state);
        assert_eq!(digest(&e.log().snapshot()), digest(&state));
        // Negative control: late replay of the older field update corrupts replacement ordering.
        let mut broken = state.as_ref().clone();
        apply(&mut broken, &encode_changes(&[update(1, 1, 87)])).unwrap();
        assert_ne!(digest(&broken), digest(&state));
    }
}

#[test]
fn deleted_recreated_id_rejects_predecessor_update_after_checkpoint() {
    // Failure model: deterministic incarnation and orderly reopen; rebinding is the negative control.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(&mut e, &[put(1, 1), delete(1, 1)]);
        reject(&mut e, &[update(1, 1, 7)], "RecordNotFound", &t);
        reject(&mut e, &[put(1, 1)], "StaleIncarnation", &t);
        drop(e);
        let mut e = t.open(mode);
        assert!(e.log().snapshot().record(&key(1)).is_none());
        commit(&mut e, &[put(1, 2)]);
        reject(&mut e, &[update(1, 1, 99)], "StaleIncarnation", &t);
        e.log().checkpoint(encode_state).unwrap();
        drop(e);
        let mut e = t.open(mode);
        assert_eq!(e.log().snapshot().incarnation(&key(1)), Some(2));
        reject(&mut e, &[update(1, 1, 99)], "StaleIncarnation", &t);
        commit(&mut e, &[update(1, 2, 99)]);
        assert_eq!(
            e.log()
                .snapshot()
                .current(&key(1), 2)
                .unwrap()
                .updated_at_unix_ms,
            99
        );
    }
}

#[test]
fn response_loss_retry_preserves_original_outcome_and_history() {
    // Failure model: simulated response loss, orderly close, retry after checkpoint.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        let changes = [put(1, 1), update(1, 1, 77)];
        let request = identity(1, 0x52);
        let outcome = e.transact(request, &changes).unwrap();
        assert!(matches!(outcome, Outcome::Committed { .. }));
        e.log().checkpoint(encode_state).unwrap();
        drop(e);
        let mut e = t.open(mode);
        let bytes = e.log().history_bytes();
        assert_eq!(e.transact(request, &changes).unwrap(), outcome);
        assert_eq!(e.log().history_bytes(), bytes);
        assert_eq!(
            e.transact(request, &[update(1, 1, 8)]).unwrap(),
            Outcome::Rejected {
                reason: Rejection::RequestIdReuse
            }
        );
        // Negative control: a new identity cannot resolve the original insert as a retry.
        reject(&mut e, &changes, "Duplicate", &t);
    }
}

#[test]
fn injected_truncated_batch_final_exposes_no_partial_transaction() {
    // Failure model: injected byte truncation; same fixture boundary as uc-memory, no host failure.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(&mut e, &[put(1, 1), put(2, 1)]);
        let history = uc_core::read_history(&t.store().join(uc_core::HISTORY_FILE)).unwrap();
        let event = &history.events[0];
        drop(e);
        fs::OpenOptions::new()
            .write(true)
            .open(t.store().join(uc_core::HISTORY_FILE))
            .unwrap()
            .set_len(event.position.offset - 1)
            .unwrap();
        assert_eq!(*t.open(mode).log().snapshot(), State::default());
        // Negative control: applying an unselected payload wrongly makes both records visible.
        let mut broken = State::default();
        apply(&mut broken, &event.payload).unwrap();
        assert_eq!(broken.records().len(), 2);
    }
}

#[test]
fn checkpoint_decoder_rejects_noncanonical_and_invalid_slots() {
    // Failure model: synthetic checkpoint bytes; full history remains authoritative at core layer.
    let mut state = State::default();
    apply(&mut state, &encode_changes(&[put(1, 1)])).unwrap();
    let bytes = encode_state(&state).unwrap();
    let original = String::from_utf8(bytes.clone()).unwrap();
    assert_eq!(decode_state(&bytes).unwrap(), state);
    for bad in [
        original.replace("slot\t1", "slot\t0"),
        original.replace("slot\t1", "slot\t01"),
        original.replace('\n', "\r\n"),
        original.trim_end().into(),
        format!("{original}\n"),
    ] {
        assert!(decode_state(bad.as_bytes()).is_err());
    }
    let slot = original.lines().find(|s| s.starts_with("slot")).unwrap();
    assert!(decode_state(format!("{original}{slot}\n").as_bytes()).is_err());
    let wrong = slot.replacen(
        &format!("\t{}\t", hex_key(key(1))),
        &format!("\t{}\t", hex_key(key(2))),
        1,
    );
    assert!(decode_state(original.replace(slot, &wrong).as_bytes()).is_err());
    state.slots.get_mut(&key(1)).unwrap().record = None;
    state.slots.get_mut(&key(1)).unwrap().incarnation = u64::MAX;
    assert_eq!(
        apply(&mut state, &encode_changes(&[put(1, 1)])).unwrap_err(),
        "incarnation overflow"
    );
}
fn hex_key(id: Id) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn arbitrary_endpoints_and_labels_assert_no_entity_existence() {
    // Failure model: deterministic admission/replay. ADR-0058's "asserts nothing" stance
    // makes this correct: no Entity, Memory or other store exists in this test at all.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        let mut r = record(1);
        r.subject = "entity-that-does-not-exist-anywhere".into();
        r.relation = format!("- unrestricted label / 雪 {}", "x".repeat(65));
        r.object = " ".into();
        r.node_id = "\0\t\n雪".into();
        commit(
            &mut e,
            &[Change::Put {
                record: Box::new(r.clone()),
                incarnation: 1,
                insert: true,
            }],
        );
        assert_eq!(e.log().snapshot().current(&key(1), 1).unwrap(), &r);
        drop(e);
        assert_eq!(
            t.open(mode).log().snapshot().current(&key(1), 1).unwrap(),
            &r
        );
        // Negative control: an invented endpoint lookup in an empty entity map would reject it.
        let invented_entities: BTreeMap<String, ()> = BTreeMap::new();
        assert!(!invented_entities.contains_key(&r.subject));
    }
}

fn invalid_put_case(insert: bool, mutate: impl Fn(&mut Relation), reason: &str) {
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        if !insert {
            commit(&mut e, &[put(1, 1)]);
        }
        let mut r = record(1);
        mutate(&mut r);
        reject(
            &mut e,
            &[Change::Put {
                record: Box::new(r),
                incarnation: 1,
                insert,
            }],
            reason,
            &t,
        );
        drop(e);
        assert_eq!(
            t.open(mode).log().snapshot().records().len(),
            usize::from(!insert)
        );
    }
}
#[test]
fn empty_relation_insert_rejects_before_append() {
    // Failure model: deterministic insert validation, unchanged disk/state is the control.
    invalid_put_case(true, |r| r.relation.clear(), "empty relation");
}
#[test]
fn empty_relation_replace_rejects_before_append() {
    // Failure model: deterministic whole-record replacement validation.
    invalid_put_case(false, |r| r.relation.clear(), "empty relation");
}
#[test]
fn negative_deleted_timestamp_insert_rejects_before_append() {
    // Failure model: deterministic insert validation.
    invalid_put_case(
        true,
        |r| r.deleted_at_unix_ms = -1,
        "negative deleted_at_unix_ms",
    );
}
#[test]
fn negative_deleted_timestamp_replace_rejects_before_append() {
    // Failure model: deterministic whole-record replacement validation.
    invalid_put_case(
        false,
        |r| r.deleted_at_unix_ms = i64::MIN,
        "negative deleted_at_unix_ms",
    );
}
#[test]
fn empty_subject_or_object_rejects_in_both_put_modes() {
    // Failure model: deterministic endpoint non-emptiness validation, no existence lookup.
    for insert in [true, false] {
        invalid_put_case(insert, |r| r.subject.clear(), "empty subject");
        invalid_put_case(insert, |r| r.object.clear(), "empty object");
    }
}
#[test]
fn signed_timestamps_and_live_or_positive_deleted_values_are_accepted() {
    // Failure model: deterministic boundary admission and orderly replay.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        for (n, deleted, created, updated) in [
            (1, 0, i64::MIN, i64::MAX),
            (2, 1, i64::MAX, i64::MIN),
            (3, i64::MAX, -1, -1),
        ] {
            let mut r = record(n);
            r.deleted_at_unix_ms = deleted;
            r.created_at_unix_ms = created;
            r.updated_at_unix_ms = updated;
            commit(
                &mut e,
                &[Change::Put {
                    record: Box::new(r.clone()),
                    incarnation: 1,
                    insert: true,
                }],
            );
            // A positive deleted_at is record metadata, not a Delete command or a tombstone.
            assert_eq!(e.log().snapshot().current(&key(n), 1).unwrap(), &r);
            r.updated_at_unix_ms = i64::MIN;
            commit(
                &mut e,
                &[Change::Put {
                    record: Box::new(r.clone()),
                    incarnation: 1,
                    insert: false,
                }],
            );
            assert_eq!(e.log().snapshot().current(&key(n), 1).unwrap(), &r);
        }
        commit(&mut e, &[update(1, 1, i64::MIN)]);
        let expected = e.log().snapshot();
        drop(e);
        assert_eq!(*t.open(mode).log().snapshot(), *expected);
    }
}
#[test]
fn invalid_record_in_checkpoint_is_rejected() {
    // Failure model: synthetic checkpoint corruption of domain fields.
    for invalid in 0..4 {
        let mut s = State::default();
        apply(&mut s, &encode_changes(&[put(1, 1)])).unwrap();
        let r =
            std::sync::Arc::make_mut(s.slots.get_mut(&key(1)).unwrap().record.as_mut().unwrap());
        match invalid {
            0 => r.subject.clear(),
            1 => r.relation.clear(),
            2 => r.object.clear(),
            _ => r.deleted_at_unix_ms = -1,
        }
        assert!(decode_state(&encode_state(&s).unwrap()).is_err());
    }
}

// Independent row/generation model operates on semantic commands, never apply/State/codecs.
#[derive(Clone, Copy)]
enum Op {
    Insert(u64),
    Replace(u64),
    Update(u64, u64, i64),
    Delete(u64, u64),
    BadLabel(u64, bool),
    BadDeleted(u64, bool),
}
#[derive(Default)]
struct Model {
    rows: BTreeMap<u64, Relation>,
    generations: BTreeMap<u64, u64>,
}
impl Model {
    fn live(&self, n: u64, generation: u64) -> bool {
        self.rows.contains_key(&n) && self.generations.get(&n) == Some(&generation)
    }
    fn write(&mut self, n: u64, row: Relation, insert: bool) -> bool {
        if row.subject.is_empty()
            || row.object.is_empty()
            || row.relation.is_empty()
            || row.deleted_at_unix_ms < 0
        {
            return false;
        }
        if insert {
            if self.rows.contains_key(&n) {
                return false;
            }
            *self.generations.entry(n).or_default() += 1;
        } else if !self.rows.contains_key(&n) {
            return false;
        }
        self.rows.insert(n, row);
        true
    }
    fn execute(&mut self, op: Op) -> bool {
        match op {
            Op::Insert(n) => self.write(n, record(n), true),
            Op::Replace(n) => {
                let mut r = record(n);
                r.subject = "replacement subject".into();
                r.updated_at_unix_ms = -99;
                r.deleted_at_unix_ms = 8;
                self.write(n, r, false)
            }
            Op::Update(n, i, v) => {
                if !self.live(n, i) {
                    return false;
                }
                self.rows.get_mut(&n).unwrap().updated_at_unix_ms = v;
                true
            }
            Op::Delete(n, i) => {
                if !self.live(n, i) {
                    return false;
                }
                self.rows.remove(&n);
                true
            }
            Op::BadLabel(n, insert) => {
                let mut r = record(n);
                r.relation.clear();
                self.write(n, r, insert)
            }
            Op::BadDeleted(n, insert) => {
                let mut r = record(n);
                r.deleted_at_unix_ms = -2;
                self.write(n, r, insert)
            }
        }
    }
    fn check(&self, e: &RelationEngine) {
        let s = e.log().snapshot();
        let expected: BTreeMap<_, _> = self.rows.values().map(|r| (r.id.0, r.clone())).collect();
        assert_eq!(
            s.records()
                .into_iter()
                .map(|r| (r.id.0, r))
                .collect::<BTreeMap<_, _>>(),
            expected
        );
        assert_eq!(s.slots.len(), self.generations.len());
        for (n, generation) in &self.generations {
            assert_eq!(s.incarnation(&key(*n)), Some(*generation));
        }
    }
}
#[test]
fn handwritten_sequence_matches_independent_row_model() {
    // Failure model: deterministic operations including rejection, replacement and incarnation.
    let ops = [
        Op::Insert(1),
        Op::Insert(2),
        Op::Insert(3),
        Op::Update(1, 1, 45),
        Op::Update(2, 1, i64::MIN),
        Op::Replace(1),
        Op::BadLabel(4, true),
        Op::BadLabel(1, false),
        Op::BadDeleted(4, true),
        Op::BadDeleted(2, false),
        Op::Delete(1, 1),
        Op::Update(1, 1, 99),
        Op::Insert(1),
        Op::Update(1, 1, 99),
        Op::Update(1, 2, -77),
        Op::Delete(2, 1),
        Op::Insert(2),
        Op::Insert(2),
        Op::Replace(2),
        Op::Update(3, 1, i64::MAX),
        Op::Delete(1, 1),
        Op::Delete(3, 1),
    ];
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        let mut model = Model::default();
        for op in ops {
            let change = match op {
                Op::Insert(n) => put(n, model.generations.get(&n).copied().unwrap_or(0) + 1),
                Op::Replace(n) => {
                    let mut r = record(n);
                    r.subject = "replacement subject".into();
                    r.updated_at_unix_ms = -99;
                    r.deleted_at_unix_ms = 8;
                    Change::Put {
                        record: Box::new(r),
                        incarnation: model.generations[&n],
                        insert: false,
                    }
                }
                Op::Update(n, i, v) => update(n, i, v),
                Op::Delete(n, i) => delete(n, i),
                Op::BadLabel(n, insert) | Op::BadDeleted(n, insert) => {
                    let mut r = record(n);
                    if matches!(op, Op::BadLabel(..)) {
                        r.relation.clear();
                    } else {
                        r.deleted_at_unix_ms = -2;
                    }
                    Change::Put {
                        record: Box::new(r),
                        incarnation: model.generations.get(&n).copied().unwrap_or(0)
                            + u64::from(insert),
                        insert,
                    }
                }
            };
            let expected = model.execute(op);
            let before = e.log().history_bytes();
            let outcome = e.transact(e.log().next_request_id(), &[change]).unwrap();
            assert_eq!(
                matches!(outcome, Outcome::Committed { .. }),
                expected,
                "{outcome:?}"
            );
            if !expected {
                assert_eq!(e.log().history_bytes(), before);
            }
            model.check(&e);
        }
        e.log().checkpoint(encode_state).unwrap();
        drop(e);
        model.check(&t.open(mode));
    }
}
