use cm_candidate::{Candidate, Change, column_records, rebuild_columns, rebuild_rows, replay};
use cm_trace::{
    Op, Value,
    generate::{self, SplitMix64},
};
#[test]
fn independent_columns_remove_deleted_incarnation_and_replace_all_fields() {
    let mut rng = SplitMix64(7);
    let a = generate::record(1, 0, &mut rng, false);
    let mut b = generate::record(1, 2, &mut rng, false);
    b.fields[10] = Value::Int(42);
    let history = vec![
        Change::Put(Box::new(a.clone())),
        Change::Delete(a.id),
        Change::Put(Box::new(b.clone())),
    ];
    assert_eq!(rebuild_rows(&history), vec![b.clone()]);
    assert_eq!(column_records(&rebuild_columns(&history)).unwrap(), vec![b]);
    assert!(rebuild_columns(&history[..2]).iter().all(|c| c.is_empty()));
}
#[test]
fn physical_replay_rejects_truncation_and_corrupt_magic() {
    let dir = cm_trace::run::test_directory("replay");
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("history.rf1");
    let mut c = Candidate::create(&path).unwrap();
    c.execute(&Op::Insert(generate::record(
        1,
        0,
        &mut SplitMix64(7),
        false,
    )))
    .unwrap();
    drop(c);
    assert_eq!(replay(&path).unwrap().len(), 1);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..bytes.len() - 1]).unwrap();
    assert!(replay(&path).is_err());
    let mut corrupt = bytes;
    corrupt[0] ^= 1;
    std::fs::write(&path, corrupt).unwrap();
    assert!(replay(&path).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}
