//! Plan scenarios: injected byte faults and deterministic validation, not host-failure evidence.
use cm_trace::{
    Memory, Op, Value,
    generate::{SplitMix64, record},
    run::{Engine, LoadedTrace},
};
use std::{fs, path::PathBuf};
use uc_core::{Durability, Outcome, Rejection, identity};
use uc_memory::{Change, MemoryEngine, encode_state};
struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        let p = cm_trace::run::test_directory(label);
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
    fn create(&self) -> MemoryEngine {
        MemoryEngine::create(&self.store(), Durability::D2)
            .unwrap()
            .0
    }
    fn open(&self) -> MemoryEngine {
        MemoryEngine::open(&self.store(), Durability::D2).unwrap().0
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
                .starts_with("cm-uc-")
        );
        fs::remove_dir_all(p).unwrap();
    }
}
fn memory(n: u64, rev: i64) -> Memory {
    record(n, rev, &mut SplitMix64(7), false)
}
fn insert(m: Memory, incarnation: u64) -> Change {
    Change::Put {
        record: Box::new(m),
        incarnation,
        insert: true,
    }
}
fn update(m: &Memory, incarnation: u64, value: i64) -> Change {
    Change::Update {
        id: m.id,
        incarnation,
        field: 10,
        value: Value::Int(value),
        equals: None,
    }
}
fn committed(outcome: Outcome) {
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}

#[test]
fn automatic_requests_use_a_separate_domain_from_explicit_requests() {
    let t = Temp::new("uc-request-domains");
    let mut engine = t.create();
    committed(
        engine
            .transact(identity(6, 0x52), &[insert(memory(1, 1), 1)])
            .unwrap(),
    );
    // The next physical ordinal is 6; the former auto namespace reused the explicit ID.
    assert_eq!(engine.log().next_request_id(), identity(6, 0x41));
    engine.execute(&Op::Insert(memory(2, 1))).unwrap();
    assert_eq!(engine.records().unwrap().len(), 2);
    drop(engine);
    assert_eq!(t.open().records().unwrap().len(), 2);
}

#[test]
fn conflict_rejects_entire_field_transaction_before_append_and_reopen() {
    let t = Temp::new("uc-conflict");
    let mut e = t.create();
    let m = memory(1, 1);
    committed(
        e.transact(identity(1, 0x52), &[insert(m.clone(), 1)])
            .unwrap(),
    );
    let before = e.log().history_bytes();
    let changes = [
        update(&m, 1, 22),
        Change::Update {
            id: m.id,
            incarnation: 1,
            field: 10,
            value: Value::Int(33),
            equals: Some(Value::Int(1)),
        },
    ];
    assert!(matches!(
        e.transact(identity(2, 0x52), &changes).unwrap(),
        Outcome::Rejected { .. }
    ));
    assert_eq!(e.log().history_bytes(), before);
    drop(e);
    assert_eq!(t.open().records().unwrap(), vec![m.clone()]);
    // Negative control: sequential per-operation publication would leak the first value.
    let mut broken = uc_memory::State::default();
    uc_memory::apply(
        &mut broken,
        &uc_memory::encode_changes(&[insert(m.clone(), 1)]),
    )
    .unwrap();
    assert!(uc_memory::apply(&mut broken, &uc_memory::encode_changes(&changes)).is_err());
    assert_ne!(broken.records(), vec![m]);
}
#[test]
fn committed_field_update_then_delete_stays_absent_after_reopen() {
    let t = Temp::new("uc-delete");
    let mut e = t.create();
    let m = memory(1, 1);
    for op in [
        Op::Insert(m.clone()),
        Op::Update(m.id, 44),
        Op::Delete(m.id),
    ] {
        e.execute(&op).unwrap();
    }
    drop(e);
    assert!(t.open().records().unwrap().is_empty());
    // Negative control: replay the actual history with deletion handling disabled.
    let history = uc_core::read_history(&t.store().join(uc_core::HISTORY_FILE)).unwrap();
    let mut broken = uc_memory::State::default();
    for event in history.events {
        let changes: Vec<_> = uc_memory::decode_changes(&event.payload)
            .unwrap()
            .into_iter()
            .filter(|c| !matches!(c, Change::Delete { .. }))
            .collect();
        if !changes.is_empty() {
            uc_memory::apply(&mut broken, &uc_memory::encode_changes(&changes)).unwrap();
        }
    }
    assert_eq!(broken.records().len(), 1);
    assert_eq!(broken.records()[0].fields[10], Value::Int(44));
}
#[test]
fn replacement_supersedes_older_field_transaction_with_checkpoint_and_full_replay() {
    let t = Temp::new("uc-replacement");
    let mut e = t.create();
    let old = memory(1, 1);
    let replacement = memory(1, 9);
    for op in [
        Op::Insert(old.clone()),
        Op::Update(old.id, 44),
        Op::Replace(replacement.clone()),
    ] {
        e.execute(&op).unwrap();
    }
    let cp = e.log().checkpoint(encode_state).unwrap();
    drop(e);
    let (e, report) = MemoryEngine::open(&t.store(), Durability::D2).unwrap();
    assert!(report.checkpoint.is_some());
    assert_eq!(e.records().unwrap(), vec![replacement.clone()]);
    drop(e);
    fs::remove_file(cp.path).unwrap();
    assert_eq!(t.open().records().unwrap(), vec![replacement.clone()]);
    // Negative control: replay the actual old field change after the checkpoint replacement.
    let history = uc_core::read_history(&t.store().join(uc_core::HISTORY_FILE)).unwrap();
    let mut broken = uc_memory::State::default();
    for event in &history.events {
        uc_memory::apply(&mut broken, &event.payload).unwrap();
    }
    uc_memory::apply(&mut broken, &history.events[1].payload).unwrap();
    assert_ne!(broken.records(), vec![replacement]);
}
#[test]
fn deleted_recreated_uuid_rejects_predecessor_updates_and_edges_including_after_checkpoint() {
    let t = Temp::new("uc-incarnation");
    let mut e = t.create();
    let a = memory(1, 1);
    let b = memory(2, 1);
    let edge = Change::Link {
        from: a.id,
        from_incarnation: 1,
        to: b.id,
        to_incarnation: 1,
    };
    committed(
        e.transact(
            identity(1, 0x52),
            &[insert(a.clone(), 1), insert(b.clone(), 1), edge.clone()],
        )
        .unwrap(),
    );
    assert_eq!(e.log().snapshot().edges.len(), 1);
    e.execute(&Op::Delete(a.id)).unwrap();
    e.execute(&Op::Insert(memory(1, 9))).unwrap();
    assert!(e.log().snapshot().edges.is_empty());
    assert_eq!(e.log().snapshot().incarnation(&a.id), Some(2));
    for change in [update(&a, 1, 99), edge] {
        assert!(matches!(
            e.transact(e.log().next_request_id(), &[change]).unwrap(),
            Outcome::Rejected {
                reason: Rejection::Validation(_)
            }
        ));
    }
    e.log().checkpoint(encode_state).unwrap();
    drop(e);
    let mut e = t.open();
    assert_eq!(e.log().snapshot().incarnation(&a.id), Some(2));
    assert!(matches!(
        e.transact(e.log().next_request_id(), &[update(&a, 1, 99)])
            .unwrap(),
        Outcome::Rejected { .. }
    ));
    assert!(e.log().snapshot().edges.is_empty());
    assert_eq!(e.log().snapshot().current(&a.id, 2).unwrap(), &memory(1, 9));
    // Negative control: rebinding the stale request to current incarnation demonstrates the oracle's sensitivity.
    committed(
        e.transact(e.log().next_request_id(), &[update(&a, 2, 99)])
            .unwrap(),
    );
    assert_ne!(e.log().snapshot().current(&a.id, 2).unwrap(), &memory(1, 9));
}
#[test]
fn injected_truncated_batch_final_never_exposes_partial_transaction() {
    let t = Temp::new("uc-batch-tail");
    let mut e = t.create();
    committed(
        e.transact(
            identity(1, 0x52),
            &[insert(memory(1, 1), 1), insert(memory(2, 1), 1)],
        )
        .unwrap(),
    );
    let history = uc_core::read_history(&t.store().join(uc_core::HISTORY_FILE)).unwrap();
    assert_eq!(history.events.len(), 1);
    let offset = history.events[0].position.offset;
    drop(e);
    fs::OpenOptions::new()
        .write(true)
        .open(t.store().join(uc_core::HISTORY_FILE))
        .unwrap()
        .set_len(offset - 1)
        .unwrap();
    assert!(t.open().records().unwrap().is_empty());
    // Negative control: applying the unselected event payload would expose both uncommitted changes.
    let mut broken = uc_memory::State::default();
    uc_memory::apply(&mut broken, &history.events[0].payload).unwrap();
    assert_eq!(broken.records().len(), 2);
}
#[test]
fn response_loss_resolves_to_same_transaction_and_stable_state() {
    let t = Temp::new("uc-retry");
    let mut e = t.create();
    let changes = [insert(memory(1, 1), 1), update(&memory(1, 1), 1, 2)];
    let request = identity(1, 0x52);
    let original = e.transact(request, &changes).unwrap();
    drop(e);
    let mut e = t.open();
    let before = e.log().history_bytes();
    assert_eq!(e.transact(request, &changes).unwrap(), original);
    assert_eq!(e.log().history_bytes(), before);
    // Without retry resolution validation of the repeated insert returns Duplicate, not the original outcome.
    assert!(matches!(
        e.transact(identity(2, 0x52), &changes).unwrap(),
        Outcome::Rejected { .. }
    ));
}
#[test]
fn small_and_1k_cmt1_traces_match_independent_oracle_in_both_durability_settings() {
    for mode in [Durability::D1, Durability::D2] {
        for trace in [
            cm_trace::generate::small(),
            cm_trace::generate::repeated(1000, 7),
        ] {
            let t = Temp::new("uc-traces");
            let trace = LoadedTrace::from_trace(trace);
            let (mut engine, _) = MemoryEngine::create(&t.store(), mode).unwrap();
            let report = cm_trace::run::trial(&trace, &mut engine, &t.0).unwrap();
            assert_eq!(report.mismatches, 0, "{} {mode:?}", trace.name);
            for stage in [
                "replay",
                "decode",
                "rows",
                "columns",
                "checkpoint",
                "open_from_checkpoint",
            ] {
                assert!(report.samples.iter().any(|(s, _)| s == stage));
            }
        }
    }
}
