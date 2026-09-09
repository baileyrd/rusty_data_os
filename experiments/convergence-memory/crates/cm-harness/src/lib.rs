use cm_candidate::{
    Candidate, column_records, decode_history, rebuild_columns, rebuild_rows, scan_history,
};
use cm_trace::{Answer, Memory, Op, column_digests, record_digests, run::Engine};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
pub struct CandidateEngine {
    candidate: Candidate,
    path: PathBuf,
    label: String,
}
pub fn label() -> String {
    format!(
        "candidate RF1 full after-image; IntegrityProfile::Structural; rev={}",
        cm_trace::run::revision()
    )
}
impl CandidateEngine {
    pub fn create(dir: &Path) -> Result<Self, String> {
        let path = dir.join("history.rf1");
        Ok(Self {
            candidate: Candidate::create(&path)?,
            path,
            label: label(),
        })
    }
}
impl Engine for CandidateEngine {
    fn label(&self) -> String {
        self.label.clone()
    }
    fn durability(&self) -> String {
        cm_trace::D1.into()
    }
    fn execute(&mut self, op: &Op) -> Result<Answer, String> {
        self.candidate.execute(op)
    }
    fn records(&self) -> Result<Vec<Memory>, String> {
        Ok(self.candidate.records())
    }
    fn finish(&mut self, expected: &[Memory]) -> Result<Vec<(String, u128)>, String> {
        let (physical, replay_time) = scan_history(&self.path)?;
        let replay_ns = replay_time.as_nanos();
        let start = Instant::now();
        let history = decode_history(&physical)?;
        let decode_ns = start.elapsed().as_nanos();
        drop(physical);
        let start = Instant::now();
        let rows = rebuild_rows(&history);
        let rows_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let columns = rebuild_columns(&history);
        let columns_ns = start.elapsed().as_nanos();
        let column_rows = column_records(&columns)?;
        let rows_valid = record_digests(&rows) == record_digests(expected)
            && column_digests(&rows) == column_digests(expected);
        let columns_valid = record_digests(&column_rows) == record_digests(expected)
            && column_digests(&column_rows) == column_digests(expected);
        if !rows_valid || !columns_valid {
            return Err(format!(
                "oracle comparison: rows={rows_valid}, columns={columns_valid}"
            ));
        }
        Ok(vec![
            ("append".into(), self.candidate.append.as_nanos()),
            ("replay".into(), replay_ns),
            ("decode".into(), decode_ns),
            ("rows".into(), rows_ns),
            ("columns".into(), columns_ns),
        ])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn series_metadata_and_summary_keep_candidate_identity_and_split_stages() {
        let dir = cm_trace::run::test_directory("candidate-series");
        let trace = cm_trace::run::LoadedTrace::from_trace(cm_trace::generate::small());
        cm_trace::run::series(
            &trace,
            &dir,
            &label(),
            cm_trace::D1,
            CandidateEngine::create,
        )
        .unwrap();
        let summary = std::fs::read_to_string(dir.join("summary.txt")).unwrap();
        for stage in ["append", "replay", "decode", "rows", "columns"] {
            assert!(
                summary
                    .lines()
                    .any(|line| line.starts_with(&format!("{stage},5,")))
            );
        }
        let environment = std::fs::read_to_string(dir.join("environment.txt")).unwrap();
        assert!(environment.contains(&format!("input_sha256={}\n", trace.input_sha256())));
        assert!(
            environment
                .lines()
                .any(|line| line.starts_with("rss_baseline_after_load="))
        );
        for i in 0..6 {
            let trial = dir.join(format!(
                "{}-{i}",
                if i == 0 { "warmup" } else { "measured" }
            ));
            for path in [
                dir.join("environment.txt"),
                dir.join("summary.txt"),
                trial.join("environment.txt"),
                trial.join("throughput.txt"),
                trial.join("results.cmt"),
            ] {
                let text = std::fs::read_to_string(path).unwrap();
                assert!(
                    text.lines()
                        .any(|line| line == format!("engine={}", label()))
                );
            }
            let observed = cm_trace::results::Results::decode(
                &std::fs::read_to_string(trial.join("observations.cmt")).unwrap(),
            )
            .unwrap();
            assert_eq!(observed.engine, label());
            assert_eq!(observed.input_sha256, trace.input_sha256());
        }
        let absolute = dir.canonicalize().unwrap();
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
                .starts_with("cm-candidate-series-")
        );
        std::fs::remove_dir_all(absolute).unwrap();
    }
    #[test]
    fn small_and_1k_match_oracle_and_both_reconstructions() {
        for trace in [
            cm_trace::generate::small(),
            cm_trace::generate::repeated(1000, 7),
        ] {
            let trace = cm_trace::run::LoadedTrace::from_trace(trace);
            let dir = cm_trace::run::test_directory("candidate");
            std::fs::create_dir(&dir).unwrap();
            let mut engine = CandidateEngine::create(&dir).unwrap();
            let report = cm_trace::run::trial(&trace, &mut engine, &dir).unwrap();
            assert_eq!(report.mismatches, 0, "retained {}", dir.display());
            let observed = cm_trace::results::Results::decode(
                &std::fs::read_to_string(dir.join("observations.cmt")).unwrap(),
            )
            .unwrap();
            assert_eq!(observed.durability, cm_trace::D1);
            assert!(
                observed.engine.starts_with(
                    "candidate RF1 full after-image; IntegrityProfile::Structural; rev="
                )
            );
            assert_eq!(observed.engine, label());
            for stage in ["append", "replay", "decode", "rows", "columns"] {
                assert_eq!(
                    report
                        .samples
                        .iter()
                        .filter(|(name, _)| name == stage)
                        .count(),
                    1
                );
            }
            drop(engine);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}
