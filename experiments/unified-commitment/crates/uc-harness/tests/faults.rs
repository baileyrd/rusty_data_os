//! Injected process termination. The parent captures the exact placement marker.
//! These tests do not claim OS-crash or power-loss survival.
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use uc_core::{Durability, Outcome, Rejection, identity};
use uc_memory::{Change, MemoryEngine};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = cm_trace::run::test_directory("uc-process");
        fs::create_dir(&p).unwrap();
        Self(p)
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
                .starts_with("cm-uc-process-")
        );
        fs::remove_dir_all(p).unwrap();
    }
}
fn changes() -> Vec<Change> {
    vec![
        Change::Put {
            record: Box::new(cm_trace::generate::record(
                1,
                1,
                &mut cm_trace::generate::SplitMix64(7),
                false,
            )),
            incarnation: 1,
            insert: true,
        },
        Change::Put {
            record: Box::new(cm_trace::generate::record(
                2,
                1,
                &mut cm_trace::generate::SplitMix64(8),
                false,
            )),
            incarnation: 1,
            insert: true,
        },
    ]
}
#[test]
fn injected_process_termination_matrix_both_levels_not_os_crash_or_power_loss() {
    for (name, mode) in [("d1", Durability::D1), ("d2", Durability::D2)] {
        for point in [
            "binding",
            "reservation",
            "provisional",
            "final",
            "commit",
            "synced",
            "checkpoint",
        ] {
            let t = Temp::new();
            let output = Command::new(env!("CARGO_BIN_EXE_uc-harness"))
                .args(["fault", name, t.0.join("store").to_str().unwrap(), point])
                .output()
                .unwrap();
            assert!(!output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.contains("injected process termination at"),
                "{name} {point}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                stdout.to_lowercase().contains(point),
                "wrong placement: {stdout}"
            );
            let (mut engine, report) = MemoryEngine::open(&t.0.join("store"), mode).unwrap();
            let selected = matches!(point, "commit" | "synced" | "checkpoint");
            assert_eq!(
                engine.log().snapshot().records().len(),
                if selected { 2 } else { 0 },
                "{name} {point}"
            );
            let before = engine.log().history_bytes();
            let outcome = engine.transact(identity(1, 0x52), &changes()).unwrap();
            if selected {
                assert!(matches!(outcome,Outcome::Committed {achieved,..} if achieved==mode));
            } else {
                assert_eq!(
                    outcome,
                    Outcome::Rejected {
                        reason: Rejection::Uncommitted
                    }
                );
                assert_eq!(report.unresolved_bindings.len(), 1);
            }
            assert_eq!(engine.log().history_bytes(), before);
            if point == "final" {
                assert_eq!(report.uncommitted_finals, vec![4]);
            }
            if point == "checkpoint" {
                assert!(report.checkpoint.is_some());
            }
        }
    }
}
#[test]
fn injected_owner_abort_releases_os_lock_without_stale_file_deletion() {
    let t = Temp::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_uc-harness"))
        .args(["fault", "lock", t.0.join("store").to_str().unwrap(), "hold"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert_eq!(line.trim(), "locked");
    assert!(matches!(
        uc_core::Directory::acquire(&t.0.join("store")),
        Err(uc_core::LogError::Owned)
    ));
    assert!(MemoryEngine::open(&t.0.join("store"), Durability::D2).is_err());
    child.stdin.take().unwrap().write_all(b"abort\n").unwrap();
    assert!(!child.wait().unwrap().success());
    assert!(t.0.join("store/owner.lock").exists());
    MemoryEngine::open(&t.0.join("store"), Durability::D2).unwrap();
}
