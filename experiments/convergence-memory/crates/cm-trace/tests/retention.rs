use cm_trace::{
    Answer, Memory, Model, Op, generate,
    results::Results,
    run::{self, Engine},
};
struct Broken {
    calls: usize,
}

struct FileBackedModel {
    model: Model,
    file: std::fs::File,
}
impl FileBackedModel {
    fn create(store: &std::path::Path) -> Result<Self, String> {
        Ok(Self {
            model: Model::default(),
            file: run::file(&store.join("test-store"))?,
        })
    }
}
impl Engine for FileBackedModel {
    fn label(&self) -> String {
        "synthetic retention test".into()
    }
    fn durability(&self) -> String {
        "synthetic test; no durability claim".into()
    }
    fn execute(&mut self, op: &Op) -> Result<Answer, String> {
        Ok(self.model.apply(op))
    }
    fn records(&self) -> Result<Vec<Memory>, String> {
        Ok(self.model.records())
    }
    fn finish(&mut self, expected: &[Memory]) -> Result<Vec<(String, u128)>, String> {
        use std::io::Write;
        assert_eq!(self.model.records(), expected);
        self.file
            .write_all(b"validated")
            .map_err(|e| e.to_string())?;
        Ok(vec![])
    }
}

fn remove_test_series(path: &std::path::Path) {
    let absolute = path.canonicalize().unwrap();
    assert_eq!(
        absolute.parent().unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
    assert!(
        absolute
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("cm-")
    );
    std::fs::remove_dir_all(absolute).unwrap();
}

#[test]
fn series_defaults_to_minimal_retention_with_forensic_opt_in() {
    let trace = run::LoadedTrace::from_trace(generate::small());
    for retain in [false, true] {
        let dir = run::test_directory("series-retention");
        if retain {
            run::series_with_retention(
                &trace,
                &dir,
                "synthetic",
                "synthetic test; no durability claim",
                true,
                FileBackedModel::create,
            )
            .unwrap();
        } else {
            run::series(
                &trace,
                &dir,
                "synthetic",
                "synthetic test; no durability claim",
                FileBackedModel::create,
            )
            .unwrap();
        }
        assert_eq!(dir.join("trace.cmt").exists(), retain);
        assert!(!dir.join("source").exists());
        for file in [
            "environment.txt",
            "source.sha256",
            "source.patch",
            "trials.csv",
            "summary.txt",
        ] {
            assert!(dir.join(file).is_file(), "missing {file}");
        }
        let env = std::fs::read_to_string(dir.join("environment.txt")).unwrap();
        assert!(env.contains("generator_seed=7\n"));
        assert!(
            env.lines()
                .any(|l| l.starts_with("rss_baseline_after_load="))
        );
        let durability_lines = format!(
            "candidate_durability={}\nengine_durability=synthetic test; no durability claim\n",
            cm_trace::D1
        );
        assert!(env.contains(&durability_lines));
        assert!(
            std::fs::read_to_string(dir.join("summary.txt"))
                .unwrap()
                .contains(&durability_lines)
        );
        assert!(env.contains("-- generate small regenerated-small.cmt\n"));
        assert!(env.contains(&format!(
            "input_sha256={}\n",
            cm_trace::hex(&cm_trace::sha256(
                cm_trace::format::encode(&trace).as_bytes()
            ))
        )));
        assert!(
            std::fs::read_to_string(dir.join("source.sha256"))
                .unwrap()
                .contains("crates/cm-trace/src/run.rs")
        );
        for i in 0..6 {
            let trial = dir.join(format!(
                "{}-{i}",
                if i == 0 { "warmup" } else { "measured" }
            ));
            assert_eq!(trial.join("store").exists(), retain);
            for file in [
                "environment.txt",
                "throughput.txt",
                "observations.cmt",
                "results.cmt",
            ] {
                assert!(trial.join(file).is_file(), "missing {file}");
            }
            let throughput = std::fs::read_to_string(trial.join("throughput.txt")).unwrap();
            assert!(throughput.contains(&durability_lines));
            assert!(
                std::fs::read_to_string(trial.join("results.cmt"))
                    .unwrap()
                    .contains(&durability_lines)
            );
            for file in ["results.cmt", "observations.cmt"] {
                let path = trial.join(file).canonicalize().unwrap();
                assert!(throughput.contains(&format!("raw_path={path:?}\n")));
                assert!(throughput.contains(&format!(
                    "raw_sha256={}\n",
                    cm_trace::hex(&cm_trace::sha256(&std::fs::read(path).unwrap()))
                )));
            }
            let observed =
                Results::decode(&std::fs::read_to_string(trial.join("observations.cmt")).unwrap())
                    .unwrap();
            assert_eq!(observed.operations.len(), 59);
            assert_eq!(observed.durability, "synthetic test; no durability claim");
            if retain {
                assert_eq!(
                    std::fs::read(trial.join("store/test-store")).unwrap(),
                    b"validated"
                );
            }
        }
        let csv = std::fs::read_to_string(dir.join("trials.csv")).unwrap();
        assert!(
            csv.lines()
                .skip(1)
                .all(|line| line.split(',').nth(4) == Some("9")),
            "size must be recorded before cleanup"
        );
        assert!(
            run::series(
                &trace,
                &dir,
                "synthetic",
                "synthetic test; no durability claim",
                FileBackedModel::create
            )
            .is_err()
        );
        remove_test_series(&dir);
    }
}

#[test]
fn invalid_series_keeps_diagnostics_but_removes_stores() {
    let dir = run::test_directory("invalid-series");
    let mut trace = generate::small();
    trace.ops = vec![(1, Op::Get(generate::uuid(99)))];
    let trace = run::LoadedTrace::from_trace(trace);
    assert!(
        run::series(
            &trace,
            &dir,
            "synthetic failures",
            "synthetic test; no durability claim",
            |store| {
                run::write(&store.join("partial-store"), "partial")?;
                Ok(Broken { calls: 0 })
            }
        )
        .is_err()
    );
    for i in 0..6 {
        let trial = dir.join(format!(
            "{}-{i}",
            if i == 0 { "warmup" } else { "measured" }
        ));
        assert!(!trial.join("store").exists());
        assert!(
            std::fs::read_to_string(trial.join("results.cmt"))
                .unwrap()
                .contains("injected failure 1")
        );
        assert!(
            std::fs::read_to_string(trial.join("throughput.txt"))
                .unwrap()
                .contains("valid=false")
        );
    }
    assert!(
        std::fs::read_to_string(dir.join("summary.txt"))
            .unwrap()
            .contains("valid=false")
    );
    remove_test_series(&dir);
}

#[test]
fn forensic_flag_is_only_accepted_in_trailing_position() {
    let mut args = vec!["run".into(), "input.cmt".into(), "output".into()];
    assert!(!run::take_retain_store(&mut args));
    let original = args.clone();
    args.push("--retain-store".into());
    assert!(run::take_retain_store(&mut args));
    assert_eq!(args, original);
    args.insert(0, "--retain-store".into());
    assert!(!run::take_retain_store(&mut args));
}
impl Engine for Broken {
    fn label(&self) -> String {
        "synthetic failure injection; no engine measurements".into()
    }
    fn durability(&self) -> String {
        "synthetic test; no durability claim".into()
    }
    fn execute(&mut self, _op: &Op) -> Result<Answer, String> {
        self.calls += 1;
        Err(format!("injected failure {}", self.calls))
    }
    fn records(&self) -> Result<Vec<Memory>, String> {
        Ok(vec![])
    }
    fn finish(&mut self, _expected: &[Memory]) -> Result<Vec<(String, u128)>, String> {
        Err("independent reconstruction mismatch".into())
    }
}
#[test]
fn retains_every_operation_and_final_failure_without_overwriting() {
    let dir = run::test_directory("retention");
    std::fs::create_dir(&dir).unwrap();
    let mut broken = Broken { calls: 0 };
    let trace = run::LoadedTrace::from_trace(generate::small());
    let report = run::trial(&trace, &mut broken, &dir).unwrap();
    assert_eq!(broken.calls, 59);
    assert_eq!(report.mismatches, 61);
    let path = dir.join("observations.cmt");
    let bytes = std::fs::read_to_string(&path).unwrap();
    let results = Results::decode(&bytes).unwrap();
    assert_eq!(results.operations.len(), 59);
    assert!(results.operations[58].contains("injected failure 59"));
    assert!(run::write(&path, "overwrite").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    assert!(
        std::fs::read_to_string(dir.join("results.cmt"))
            .unwrap()
            .contains("independent reconstruction mismatch")
    );
    std::fs::remove_dir_all(dir).unwrap();
}
