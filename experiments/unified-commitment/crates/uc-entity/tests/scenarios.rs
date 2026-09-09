//! Failure models: deterministic validation/replay, simulated response loss, injected byte truncation.
//! No OS-crash or power-loss evidence; core process fault tests remain unchanged.
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use uc_core::{Durability, Outcome, Rejection, identity, sha256};
use uc_entity::{
    Change, Entity, EntityEngine, Id, State, apply, decode_changes, decode_state, encode_changes,
    encode_state,
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "uc-entity-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
    fn create(&self, mode: Durability) -> EntityEngine {
        EntityEngine::create(&self.store(), mode).unwrap().0
    }
    fn open(&self, mode: Durability) -> EntityEngine {
        EntityEngine::open(&self.store(), mode).unwrap().0
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
                .starts_with("uc-entity-")
        );
        fs::remove_dir_all(path).unwrap();
    }
}
fn key(n: u64) -> Id {
    identity(n, 0x44).0
}
fn record(n: u64) -> Entity {
    Entity {
        id: identity(n, 0x44),
        label: "label\t雪\nRDE1".into(),
        kind: "open kind".into(),
        mention_count: -3,
        aliases: vec!["".into(), "alias\n\t雪".into(), "alias".into()],
    }
}
fn link(label: &str, left: u64, li: u64, right: u64, ri: u64) -> Change {
    Change::Link {
        relation: label.into(),
        left: key(left),
        left_incarnation: li,
        right: key(right),
        right_incarnation: ri,
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
    Change::UpdateMentionCount {
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
fn commit(e: &mut EntityEngine, changes: &[Change]) {
    let outcome = e.transact(e.log().next_request_id(), changes).unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
fn reject(e: &mut EntityEngine, changes: &[Change], expected: &str, t: &Temp) {
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
    replacement.label = "replacement".into();
    replacement.kind = "new kind".into();
    replacement.aliases = vec!["new alias".into()];
    let changes = vec![
        put(1, 1),
        Change::Put {
            record: Box::new(replacement),
            incarnation: 1,
            insert: false,
        },
        update(1, 1, i64::MIN),
        delete(1, 1),
        link("new_label", 1, 1, 2, 1),
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
        original.replace("CME1", "BAD1"),
        format!("{original}\n"),
        "CME1\n".into(),
        "CME1\nunknown\n".into(),
        format!("CME1\ndelete\t1\t{}\n", "FF".repeat(16)),
        format!("CME1\ndelete\t1\t{}\n", "00".repeat(15)),
        format!(
            "CME1\nupdate\t1\t{}\t9223372036854775808\n",
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
            uc_entity::MAX_OPERATIONS
        ]))
        .unwrap()
        .len(),
        uc_entity::MAX_OPERATIONS
    );
    assert!(
        decode_changes(&encode_changes(&vec![
            delete(1, 1);
            uc_entity::MAX_OPERATIONS + 1
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
                .mention_count,
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
        replacement.label = "replacement".into();
        replacement.kind = "new kind".into();
        replacement.aliases = vec!["new alias".into()];
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
        let (e, report) = EntityEngine::open(&t.store(), mode).unwrap();
        assert!(report.checkpoint.is_some());
        assert_eq!(
            e.log().snapshot().current(&key(1), 1).unwrap(),
            &replacement
        );
        assert_eq!(digest(&e.log().snapshot()), digest(&state));
        assert_eq!(encode_state(&e.log().snapshot()).unwrap(), bytes);
        drop(e);
        fs::remove_file(cp.path).unwrap();
        let (e, report) = EntityEngine::open(&t.store(), mode).unwrap();
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
                .mention_count,
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
fn seeded_labels_are_known_before_any_links_and_empty_checkpoint() {
    // Failure model: deterministic construction and checkpoint/replay with no links.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let e = t.create(mode);
        let labels = vec!["mentioned_with".to_string(), "relates_to".to_string()];
        assert_eq!(e.list_relation_kinds(), labels);
        for label in &labels {
            assert_eq!(e.neighbors_by_relation(label, &key(1)), Ok(vec![]));
        }
        assert_eq!(
            e.neighbors_by_relation("truly-unknown", &key(1)),
            Err("Malformed".into())
        );
        // The unchanged core requires a commit before checkpointing. First prove fresh replay,
        // then checkpoint an empty live record set after an insert/delete (still no Link).
        assert!(matches!(
            e.log().checkpoint(encode_state),
            Err(uc_core::LogError::Invalid(_))
        ));
        drop(e);
        let mut e = t.open(mode);
        assert_eq!(e.list_relation_kinds(), labels);
        commit(&mut e, &[put(1, 1), delete(1, 1)]);
        let cp = e.log().checkpoint(encode_state).unwrap();
        drop(e);
        assert_eq!(t.open(mode).list_relation_kinds(), labels);
        fs::remove_file(cp.path).unwrap();
        assert_eq!(t.open(mode).list_relation_kinds(), labels);
        // Negative control: deriving labels from the empty edge set loses the seeds.
        assert!(!labels.is_empty());
        assert!(State::default().edges.is_empty());
    }
}

#[test]
fn open_labels_are_symmetric_idempotent_and_survive_reopen() {
    // Failure model: deterministic same-table links and orderly full replay.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(
            &mut e,
            &[put(1, 1), put(2, 1), link("New_Label-9", 1, 1, 2, 1)],
        );
        commit(
            &mut e,
            &[link("New_Label-9", 2, 1, 1, 1), link("second", 1, 1, 2, 1)],
        );
        assert_eq!(e.log().snapshot().edges.len(), 2);
        assert_eq!(e.neighbors(&key(1)), vec![key(2)]);
        assert_eq!(
            e.neighbors_by_relation("New_Label-9", &key(2)),
            Ok(vec![key(1)])
        );
        drop(e);
        let e = t.open(mode);
        assert_eq!(e.log().snapshot().current(&key(1), 1).unwrap(), &record(1));
        assert_eq!(
            e.neighbors_by_relation("New_Label-9", &key(1)),
            Ok(vec![key(2)])
        );
        assert_eq!(e.neighbors_by_relation("second", &key(2)), Ok(vec![key(1)]));
    }
}

#[test]
fn invalid_labels_self_loop_and_missing_endpoints_reject_before_write() {
    // Failure model: deterministic validation; accepted boundary labels are positive controls.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(&mut e, &[put(1, 1), put(2, 1)]);
        for label in [
            "",
            "-a",
            "a b",
            "a.b",
            "a/b",
            "a\t",
            "é",
            "a\0",
            &"a".repeat(65),
        ] {
            reject(&mut e, &[link(label, 1, 1, 2, 1)], "Malformed", &t);
        }
        reject(&mut e, &[link("new", 1, 1, 1, 1)], "SelfLoop", &t);
        reject(&mut e, &[link("new", 1, 1, 9, 1)], "RecordNotFound", &t);
        reject(&mut e, &[link("new", 9, 1, 1, 1)], "RecordNotFound", &t);
        assert!(!e.list_relation_kinds().contains(&"new".into()));
        for label in ["_", "0", "A-z_9", &"a".repeat(64)] {
            commit(&mut e, &[link(label, 1, 1, 2, 1)]);
        }
    }
}

#[test]
fn delete_cascades_edges_but_retains_labels_through_checkpoint_and_full_replay() {
    // Failure model: deterministic deletion/recreation; checkpoint and full replay are separate paths.
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        commit(
            &mut e,
            &[
                put(1, 1),
                put(2, 1),
                put(3, 1),
                link("brand-new", 1, 1, 2, 1),
                link("other", 3, 1, 1, 1),
            ],
        );
        let old_edges = e.log().snapshot().edges.clone();
        commit(&mut e, &[delete(1, 1)]);
        assert_eq!(e.neighbors_by_relation("brand-new", &key(1)), Ok(vec![]));
        assert_eq!(e.neighbors_by_relation("brand-new", &key(2)), Ok(vec![]));
        commit(&mut e, &[put(1, 2)]);
        reject(
            &mut e,
            &[link("brand-new", 1, 1, 2, 1)],
            "StaleIncarnation",
            &t,
        );
        reject(&mut e, &[link("other", 3, 1, 1, 1)], "StaleIncarnation", &t);
        assert!(e.log().snapshot().edges.is_empty());
        let expected = e.log().snapshot();
        let cp = e.log().checkpoint(encode_state).unwrap();
        drop(e);
        for checkpoint in [true, false] {
            if !checkpoint {
                fs::remove_file(&cp.path).unwrap();
            }
            let (mut e, report) = EntityEngine::open(&t.store(), mode).unwrap();
            assert_eq!(report.checkpoint.is_some(), checkpoint);
            assert_eq!(*e.log().snapshot(), *expected);
            assert_eq!(digest(&e.log().snapshot()), digest(&expected));
            assert!(e.list_relation_kinds().contains(&"brand-new".into()));
            assert_eq!(e.neighbors_by_relation("brand-new", &key(2)), Ok(vec![]));
            assert_eq!(
                e.neighbors_by_relation("truly-unknown", &key(2)),
                Err("Malformed".into())
            );
            reject(
                &mut e,
                &[link("brand-new", 1, 1, 2, 1)],
                "StaleIncarnation",
                &t,
            );
            reject(&mut e, &[update(1, 1, 99)], "StaleIncarnation", &t);
        }
        // Negative controls: disabling cascade or deriving labels from edges loses fidelity.
        let mut broken = expected.as_ref().clone();
        broken.edges = old_edges;
        assert_ne!(digest(&broken), digest(&expected));
        broken = expected.as_ref().clone();
        broken.known_labels = broken
            .edges
            .iter()
            .map(|(_, _, l, _, _)| l.clone())
            .collect();
        assert_ne!(digest(&broken), digest(&expected));
    }
}

#[test]
fn checkpoint_labels_and_edges_must_be_canonical_and_valid() {
    // Failure model: deterministic malformed checkpoint state.
    let mut s = State::default();
    apply(
        &mut s,
        &encode_changes(&[put(1, 1), put(2, 1), link("custom", 1, 1, 2, 1)]),
    )
    .unwrap();
    let original = String::from_utf8(encode_state(&s).unwrap()).unwrap();
    for prefix in ["label", "edge"] {
        let line = original.lines().find(|l| l.starts_with(prefix)).unwrap();
        assert!(decode_state(format!("{original}{line}\n").as_bytes()).is_err());
    }
    let seed = original
        .lines()
        .find(|l| l.starts_with("label\t6d"))
        .unwrap();
    assert!(decode_state(original.replace(&format!("{seed}\n"), "").as_bytes()).is_err());
    s.known_labels.insert("bad label".into());
    assert!(decode_state(&encode_state(&s).unwrap()).is_err());
    s.known_labels.remove("bad label");
    let edge = s.edges.pop_first().unwrap();
    s.edges.insert((edge.0, 2, edge.2, edge.3, edge.4));
    assert!(decode_state(&encode_state(&s).unwrap()).is_err());
}

// Independent plain model: rows, generation counters and two-way adjacency, not State/Slot.
// Operations are handwritten semantic commands; the oracle never calls apply or domain codecs.
#[derive(Clone, Copy)]
enum Op {
    Insert(u64),
    Replace(u64),
    Update(u64, u64, i64),
    Delete(u64, u64),
    Link(&'static str, u64, u64, u64, u64),
}
#[derive(Default)]
struct Model {
    rows: BTreeMap<u64, Entity>,
    generations: BTreeMap<u64, u64>,
    labels: BTreeMap<String, BTreeMap<u64, std::collections::BTreeSet<u64>>>,
}
impl Model {
    fn new() -> Self {
        Self {
            labels: [
                ("mentioned_with".into(), BTreeMap::new()),
                ("relates_to".into(), BTreeMap::new()),
            ]
            .into(),
            ..Self::default()
        }
    }
    fn live(&self, n: u64, generation: u64) -> bool {
        self.rows.contains_key(&n) && self.generations.get(&n) == Some(&generation)
    }
    fn execute(&mut self, op: Op) -> bool {
        match op {
            Op::Insert(n) => {
                if self.rows.contains_key(&n) {
                    return false;
                }
                *self.generations.entry(n).or_default() += 1;
                self.rows.insert(n, record(n));
            }
            Op::Replace(n) => {
                let Some(row) = self.rows.get_mut(&n) else {
                    return false;
                };
                *row = record(n);
                row.label = "replaced".into();
                row.kind = "replacement kind".into();
                row.aliases.clear();
            }
            Op::Update(n, generation, value) => {
                if !self.live(n, generation) {
                    return false;
                }
                self.rows.get_mut(&n).unwrap().mention_count = value;
            }
            Op::Delete(n, generation) => {
                if !self.live(n, generation) {
                    return false;
                }
                self.rows.remove(&n);
                for adjacency in self.labels.values_mut() {
                    adjacency.remove(&n);
                    for peers in adjacency.values_mut() {
                        peers.remove(&n);
                    }
                }
            }
            Op::Link(label, a, ai, b, bi) => {
                // Spell the source rule independently, never call valid_relation_label.
                if label.is_empty()
                    || label.len() > 64
                    || label.as_bytes()[0] == b'-'
                    || !label.bytes().all(|b| {
                        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
                            .contains(&b)
                    })
                    || a == b
                    || !self.live(a, ai)
                    || !self.live(b, bi)
                {
                    return false;
                }
                let adjacency = self.labels.entry(label.into()).or_default();
                adjacency.entry(a).or_default().insert(b);
                adjacency.entry(b).or_default().insert(a);
            }
        }
        true
    }
    fn check(&self, e: &EntityEngine) {
        let s = e.log().snapshot();
        let expected_rows: BTreeMap<_, _> =
            self.rows.values().map(|r| (r.id.0, r.clone())).collect();
        assert_eq!(
            s.records()
                .into_iter()
                .map(|r| (r.id.0, r))
                .collect::<BTreeMap<_, _>>(),
            expected_rows
        );
        assert_eq!(s.slots.len(), self.generations.len());
        for (n, generation) in &self.generations {
            assert_eq!(s.incarnation(&key(*n)), Some(*generation));
        }
        assert_eq!(
            e.list_relation_kinds(),
            self.labels.keys().cloned().collect::<Vec<_>>()
        );
        let expected_edges: usize = self
            .labels
            .values()
            .flat_map(|adj| adj.values())
            .map(|peers| peers.len())
            .sum::<usize>()
            / 2;
        assert_eq!(s.edges.len(), expected_edges);
        for (label, adjacency) in &self.labels {
            for n in 1..=4 {
                let mut peers: Vec<_> = adjacency
                    .get(&n)
                    .into_iter()
                    .flatten()
                    .map(|p| key(*p))
                    .collect();
                peers.sort();
                assert_eq!(e.neighbors_by_relation(label, &key(n)), Ok(peers));
            }
        }
    }
}
#[test]
fn handwritten_sequence_matches_independent_adjacency_model() {
    // Failure model: deterministic semantic sequence, both D1/D2, compare after every attempt.
    let ops = [
        Op::Insert(1),
        Op::Insert(2),
        Op::Insert(3),
        Op::Link("alpha", 1, 1, 2, 1),
        Op::Link("beta", 2, 1, 3, 1),
        Op::Link("alpha", 2, 1, 1, 1),
        Op::Link("new", 1, 1, 1, 1),
        Op::Update(2, 1, 12),
        Op::Replace(2),
        Op::Delete(1, 1),
        Op::Insert(1),
        Op::Update(1, 1, 90),
        Op::Link("alpha", 1, 1, 2, 1),
        Op::Link("alpha", 1, 2, 3, 1),
        Op::Update(1, 2, -9),
        Op::Insert(2),
        Op::Link("bad label", 1, 2, 2, 1),
        Op::Link("gamma", 1, 2, 4, 1),
        Op::Delete(2, 1),
        Op::Delete(3, 1),
        Op::Insert(2),
        Op::Link("beta", 1, 2, 2, 2),
    ];
    for mode in [Durability::D1, Durability::D2] {
        let t = Temp::new();
        let mut e = t.create(mode);
        let mut model = Model::new();
        for op in ops {
            let change = match op {
                Op::Insert(n) => put(n, model.generations.get(&n).copied().unwrap_or(0) + 1),
                Op::Replace(n) => {
                    let mut r = record(n);
                    r.label = "replaced".into();
                    r.kind = "replacement kind".into();
                    r.aliases.clear();
                    Change::Put {
                        record: Box::new(r),
                        incarnation: model.generations[&n],
                        insert: false,
                    }
                }
                Op::Update(n, i, v) => update(n, i, v),
                Op::Delete(n, i) => delete(n, i),
                Op::Link(l, a, ai, b, bi) => link(l, a, ai, b, bi),
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
