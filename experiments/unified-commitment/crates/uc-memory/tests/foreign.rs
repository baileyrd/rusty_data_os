//! Step 4c single-log correctness: no Entity engine or far-side lookup exists here.
use cm_trace::generate::{SplitMix64, record};
use uc_core::{Durability, Outcome};
use uc_memory::*;

#[test]
fn foreign_checkpoint_full_replay_cascades_and_local_incarnation_validation() {
    let root = cm_trace::run::test_directory("uc-foreign");
    std::fs::create_dir(&root).unwrap();
    let path = root.join("store");
    let mut engine = MemoryEngine::create_with_label(&path, Durability::D1, "test".into())
        .unwrap()
        .0;
    let m = record(1, 1, &mut SplitMix64(7), false);
    let other = [42; 16];
    let link = Change::LinkForeign {
        from: m.id,
        from_incarnation: 1,
        to: other,
        to_incarnation: 99,
    };
    let changes = [
        Change::Put {
            record: Box::new(m.clone()),
            incarnation: 1,
            insert: true,
        },
        link.clone(),
        Change::LinkForeign {
            from: m.id,
            from_incarnation: 1,
            to: [43; 16],
            to_incarnation: 7,
        },
        Change::DetachForeign { to: [43; 16] },
        Change::DetachForeign { to: [43; 16] },
    ];
    assert!(matches!(
        engine
            .transact(engine.log().next_request_id(), &changes)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    let expected = engine.log().snapshot();
    assert_eq!(expected.foreign_edges, [(m.id, 1, other, 99)].into());
    assert!(!expected.slots.contains_key(&other));
    engine.log().checkpoint(encode_state).unwrap();
    drop(engine);
    let (mut engine, report) = MemoryEngine::open(&path, Durability::D1).unwrap();
    assert!(report.checkpoint.is_some());
    assert_eq!(engine.log().snapshot(), expected);
    let history = uc_core::read_history(&path.join(uc_core::HISTORY_FILE)).unwrap();
    let mut replay = State::default();
    for event in history.events {
        apply(&mut replay, &event.payload).unwrap();
    }
    assert_eq!(replay, *expected);
    for bad in [
        Change::LinkForeign {
            from: [88; 16],
            from_incarnation: 1,
            to: other,
            to_incarnation: 99,
        },
        Change::LinkForeign {
            from: m.id,
            from_incarnation: 2,
            to: other,
            to_incarnation: 99,
        },
    ] {
        assert!(matches!(
            engine
                .transact(engine.log().next_request_id(), &[bad])
                .unwrap(),
            Outcome::Rejected { .. }
        ));
        assert_eq!(engine.log().snapshot(), expected);
    }
    assert!(matches!(
        engine
            .transact(
                engine.log().next_request_id(),
                &[Change::Delete {
                    id: m.id,
                    incarnation: 1
                }]
            )
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert!(engine.log().snapshot().foreign_edges.is_empty());
    assert!(matches!(
        engine
            .transact(
                engine.log().next_request_id(),
                &[Change::Put {
                    record: Box::new(m),
                    incarnation: 2,
                    insert: true
                }]
            )
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        engine
            .transact(engine.log().next_request_id(), &[link])
            .unwrap(),
        Outcome::Rejected { .. }
    ));
    drop(engine);
    let checked = root.canonicalize().unwrap();
    assert_eq!(
        checked.parent().unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
    assert!(
        checked
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("cm-uc-")
    );
    std::fs::remove_dir_all(checked).unwrap();
}

#[test]
fn cmm3_cms3_magic_limits_and_canonical_foreign_checkpoint() {
    let m = record(1, 1, &mut SplitMix64(7), false);
    let ops = [
        Change::Put {
            record: Box::new(m.clone()),
            incarnation: 1,
            insert: true,
        },
        Change::LinkForeign {
            from: m.id,
            from_incarnation: 1,
            to: [42; 16],
            to_incarnation: 7,
        },
        Change::DetachForeign { to: [43; 16] },
    ];
    let encoded = encode_changes(&ops);
    assert!(encoded.starts_with(b"CMM3\n"));
    assert_eq!(decode_changes(&encoded).unwrap(), ops);
    let old = String::from_utf8(encoded.clone())
        .unwrap()
        .replacen("CMM3", "CMM2", 1);
    assert_eq!(decode_changes(old.as_bytes()).unwrap_err(), "CMM3 magic");
    // Frozen old decoder's first gate, sufficient to prove the new payload fails closed.
    assert!(
        std::str::from_utf8(&encoded)
            .unwrap()
            .strip_prefix("CMM2\n")
            .is_none()
    );
    let detach = Change::DetachForeign { to: [42; 16] };
    assert_eq!(
        decode_changes(&encode_changes(&vec![detach.clone(); MAX_OPERATIONS]))
            .unwrap()
            .len(),
        MAX_OPERATIONS
    );
    assert_eq!(
        decode_changes(&encode_changes(&vec![detach; MAX_OPERATIONS + 1])).unwrap_err(),
        "transaction operation limit"
    );
    for bad in [
        b"CMM3\nlinkforeign\t1\tbad\t7\tbad\n".as_slice(),
        b"CMM3\ndetachforeign\n",
        b"CMM3\nunknown\n",
    ] {
        assert!(decode_changes(bad).is_err());
    }
    let mut state = State::default();
    apply(&mut state, &encoded).unwrap();
    let checkpoint = encode_state(&state).unwrap();
    assert_eq!(decode_state(&checkpoint).unwrap(), state);
    let text = String::from_utf8(checkpoint).unwrap();
    assert!(text.starts_with("CMS3\n"));
    assert_eq!(
        decode_state(text.replacen("CMS3", "CMS2", 1).as_bytes()).unwrap_err(),
        "CMS3 magic"
    );
    let edge = text
        .lines()
        .find(|line| line.starts_with("foreign_edge"))
        .unwrap();
    assert!(decode_state(format!("{text}{edge}\n").as_bytes()).is_err());
    assert!(
        decode_state(
            text.replace("foreign_edge\t1", "foreign_edge\t2")
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        decode_state(
            text.replace("foreign_edge\t1", "foreign_edge\t01")
                .as_bytes()
        )
        .is_err()
    );
    assert!(decode_state(text.trim_end().as_bytes()).is_err());
}
