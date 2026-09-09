use std::path::Path;
use uc_core::{Durability, Point, identity};
use uc_memory::{Change, MemoryEngine, encode_state};

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if cm_trace::run::generate_cli(&args)? {
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "fault") {
        if args.len() != 4 {
            return Err("fault d1|d2|lock DIRECTORY POINT".into());
        }
        return fault(&args[1], Path::new(&args[2]), &args[3]);
    }
    let retain = cm_trace::run::take_retain_store(&mut args);
    if args.len() != 4 || args[0] != "run" {
        return Err("run TRACE.cmt OUT_DIR d1|d2 [--retain-store]".into());
    }
    let mode = Durability::parse(&args[3])?;
    let trace = cm_trace::run::load(Path::new(&args[1]))?;
    run_series(&trace, Path::new(&args[2]), mode, retain)
}
fn run_series(
    trace: &cm_trace::run::LoadedTrace,
    output: &Path,
    mode: Durability,
    retain: bool,
) -> Result<(), String> {
    let label = MemoryEngine::series_label(mode);
    cm_trace::run::series_with_meta(
        trace,
        output,
        &label,
        mode.label(),
        retain,
        cm_trace::run::SeriesMeta {
            experiment: "EXP-0003",
            hypothesis: "HYP-0003",
            source_prefixes: &[
                "experiments/unified-commitment/",
                "experiments/convergence-memory/crates/cm-trace/",
                "experiments/exp-0001/crates/exp1-record-format/",
                "experiments/exp-0001/crates/exp1-raw-append-replay/",
            ],
        },
        |dir| MemoryEngine::create_with_label(dir, mode, label.clone()).map(|(engine, _)| engine),
    )
}
fn fault(scenario: &str, directory: &Path, point: &str) -> Result<(), String> {
    if scenario == "lock" {
        let (_engine, _) = MemoryEngine::create(directory, Durability::D2)?;
        println!("locked");
        use std::io::Write;
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        std::process::abort();
    }
    let mode = Durability::parse(scenario)?;
    let point = Point::parse(point)?;
    let (mut engine, _) = MemoryEngine::create(directory, mode)?;
    engine.log_mut().set_hook(move |p| {
        if p == point {
            // Out-of-band placement marker in parent-captured stdout, never recovery input.
            println!("injected process termination at {p:?}");
            use std::io::Write;
            std::io::stdout().flush().expect("placement evidence");
            std::process::abort();
        }
    });
    let a = cm_trace::generate::record(1, 1, &mut cm_trace::generate::SplitMix64(7), false);
    let b = cm_trace::generate::record(2, 1, &mut cm_trace::generate::SplitMix64(8), false);
    let changes = [
        Change::Put {
            record: Box::new(a),
            incarnation: 1,
            insert: true,
        },
        Change::Put {
            record: Box::new(b),
            incarnation: 1,
            insert: true,
        },
    ];
    let result = engine
        .transact(identity(1, 0x52), &changes)
        .map_err(|e| e.to_string())?;
    if !matches!(result, uc_core::Outcome::Committed { .. }) {
        return Err(format!("fault setup: {result:?}"));
    }
    if point == Point::Checkpoint {
        engine
            .log()
            .checkpoint(encode_state)
            .map_err(|e| e.to_string())?;
    }
    Err("injection point was not reached".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_keeps_one_revision_bound_identity_in_every_evidence_file() {
        for mode in [Durability::D1, Durability::D2] {
            let dir = cm_trace::run::test_directory("uc-series-identity");
            let trace = cm_trace::run::LoadedTrace::from_trace(cm_trace::generate::small());
            run_series(&trace, &dir, mode, false).unwrap();
            let environment = std::fs::read_to_string(dir.join("environment.txt")).unwrap();
            let identity = environment
                .lines()
                .find(|s| s.starts_with("engine="))
                .unwrap();
            assert!(identity.contains("; rev="));
            assert!(identity.starts_with(&format!("engine={}", mode.label())));
            for i in 0..6 {
                let trial = dir.join(format!(
                    "{}-{i}",
                    if i == 0 { "warmup" } else { "measured" }
                ));
                for path in [
                    dir.join("summary.txt"),
                    trial.join("environment.txt"),
                    trial.join("throughput.txt"),
                    trial.join("results.cmt"),
                ] {
                    let text = std::fs::read_to_string(&path).unwrap();
                    assert!(
                        text.lines().any(|line| line == identity),
                        "{}",
                        path.display()
                    );
                }
                let observed = cm_trace::results::Results::decode(
                    &std::fs::read_to_string(trial.join("observations.cmt")).unwrap(),
                )
                .unwrap();
                assert_eq!(format!("engine={}", observed.engine), identity);
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
                    .starts_with("cm-uc-series-identity-")
            );
            std::fs::remove_dir_all(absolute).unwrap();
        }
    }
}
