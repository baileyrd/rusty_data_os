use exp1_descriptive_d1_harness::history_views::{
    Trial, create_output_directory, run_trial, unique_default_output,
};
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SIZES: [usize; 3] = [100, 1_000, 10_000];
const REPRODUCTION_COMMAND: &str = "cargo +1.89.0 run --release --manifest-path experiments/exp-0001/Cargo.toml --locked --offline --package exp1-descriptive-d1-harness --example history_views -- NEW_OUTPUT_DIRECTORY";

fn command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("unavailable ({error})"))?;
    if !output.status.success() {
        return Err(format!("unavailable (exit status {})", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace('\n', " | "))
}

fn observed_command(program: &str, args: &[&str]) -> String {
    command(program, args).unwrap_or_else(|reason| reason)
}

fn git_status() -> String {
    match command("git", &["status", "--porcelain"]) {
        Ok(status) if status.is_empty() => "clean".into(),
        Ok(status) => format!("dirty ({})", status.replace(',', ";")),
        Err(reason) => reason,
    }
}

fn ns(duration: Duration) -> u128 {
    duration.as_nanos()
}
fn rate(count: usize, duration: Duration) -> f64 {
    count as f64 * 1_000_000_000.0 / ns(duration).max(1) as f64
}

fn csv_line(size: usize, kind: &str, index: usize, trial: &Trial) -> String {
    format!(
        "{size},{kind},{index},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{}\n",
        trial.payload_bytes,
        trial.physical_bytes,
        ns(trial.encode),
        ns(trial.append),
        ns(trial.replay),
        ns(trial.row_rebuild),
        ns(trial.column_rebuild),
        ns(trial.direct_row),
        ns(trial.direct_column),
        rate(size, trial.encode),
        rate(size, trial.append),
        rate(size, trial.replay),
        rate(size, trial.row_rebuild),
        rate(size, trial.column_rebuild),
        rate(size, trial.direct_row),
        rate(size, trial.direct_column),
        trial.correctness
    )
}

fn median_range(samples: &[Duration]) -> (u128, u128, u128) {
    let mut values: Vec<_> = samples.iter().map(|value| value.as_nanos()).collect();
    values.sort_unstable();
    (
        values[values.len() / 2],
        values[0],
        values[values.len() - 1],
    )
}

struct Outputs {
    trials: File,
    failures: File,
    summary: File,
}

fn create_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "exclusive output creation failed for {}: {error}",
                path.display()
            )
        })
}

fn metadata() -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let cpu = fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("model name"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unavailable (/proc/cpuinfo field absent)".into());
    let mem = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("MemTotal:"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unavailable (/proc/meminfo field absent)".into());
    Ok(format!(
        "schema=EXPLORATORY-HV1/run-metadata-v2\nunix_timestamp={timestamp}\nrepository_revision={}\nrepository_status={}\nbuild_profile=unknown (not embedded by Cargo)\nbuild_debug_assertions={}\nbuild_overflow_checks=unknown (not embedded by Cargo)\nbuild_locked=unknown (not embedded in binary)\nbuild_offline=unknown (not embedded in binary)\nbuild_target_triple=unknown (not embedded by Cargo)\nbuild_target_arch={}\nbuild_target_os={}\nruntime_toolchain_observation={}\nreproduction_command={}\nworkload=deterministic single-thread; sizes=100,1000,10000; entities=17; warmups=1; measured_trials=5; batching=none; queue_depth=1\npayload_label=EXPLORATORY-HV1/entity-time-value-v1 (separate from SOP1/SOP2)\ndurability=D1 ordinary writes only; no fsync; no crash-survival or durable-canonical-commit claim\ncache_state=uncontrolled; warm-up and prior trials may affect page cache\nos_kernel={}\ncpu={}\nmemory={}\nfilesystem=recorded after output creation in environment.txt companion context only\nstorage_model_interface=unavailable (not reliably exposed to unprivileged container)\nmount_options=unavailable (output-path mount mapping not resolved)\npower_performance_mode=unavailable (not reliably exposed)\nvirtualization_container={}\nram_speed=unavailable (not exposed)\nperf_counters=not collected (optional)\n",
        observed_command("git", &["rev-parse", "HEAD"]),
        git_status(),
        cfg!(debug_assertions),
        std::env::consts::ARCH,
        std::env::consts::OS,
        observed_command("rustc", &["-vV"]),
        REPRODUCTION_COMMAND,
        observed_command("uname", &["-a"]),
        cpu,
        mem,
        if fs::read_to_string("/proc/1/cgroup")
            .map(|text| text.contains("docker")
                || text.contains("kubepods")
                || text.contains("containerd"))
            .unwrap_or(false)
        {
            "container indicators present"
        } else {
            "unavailable/undetected from /proc/1/cgroup"
        }
    ))
}

fn open_outputs(output: &Path) -> Result<Outputs, String> {
    let mut environment = create_file(&output.join("environment.txt"))?;
    let mut trials = create_file(&output.join("trials.csv"))?;
    let mut failures = create_file(&output.join("failures.csv"))?;
    let mut summary = create_file(&output.join("summary.md"))?;
    environment
        .write_all(metadata()?.as_bytes())
        .map_err(|error| error.to_string())?;
    writeln!(
        environment,
        "filesystem={}",
        observed_command("df", &["-T", output.to_str().unwrap_or(".")])
    )
    .map_err(|error| error.to_string())?;
    trials.write_all(b"event_count,trial_kind,trial_index,payload_bytes,physical_bytes,encode_ns,append_ns,replay_ns,row_rebuild_ns,column_rebuild_ns,direct_row_ns,direct_column_ns,encode_events_s,append_events_s,replay_events_s,row_rebuild_events_s,column_rebuild_events_s,direct_row_events_s,direct_column_events_s,correctness\n").map_err(|error| error.to_string())?;
    failures
        .write_all(b"event_count,trial_kind,trial_index,reason\n")
        .map_err(|error| error.to_string())?;
    summary.write_all(b"# Exploratory history/materialization run summary\n\nRun incomplete unless replaced by an all-passed summary after every configured trial completes.\n").map_err(|error| error.to_string())?;
    environment.flush().map_err(|error| error.to_string())?;
    trials.flush().map_err(|error| error.to_string())?;
    failures.flush().map_err(|error| error.to_string())?;
    summary.flush().map_err(|error| error.to_string())?;
    Ok(Outputs {
        trials,
        failures,
        summary,
    })
}

fn record_failure(
    outputs: &mut Outputs,
    size: usize,
    kind: &str,
    index: usize,
    reason: &str,
) -> Result<(), String> {
    let escaped = reason.replace('"', "\"\"").replace(['\r', '\n'], " ");
    writeln!(outputs.failures, "{size},{kind},{index},\"{escaped}\"")
        .map_err(|error| error.to_string())?;
    outputs.failures.flush().map_err(|error| error.to_string())
}

fn execute<F>(output: &Path, mut runner: F) -> Result<(), String>
where
    F: FnMut(&Path, usize, &str) -> Result<Trial, String>,
{
    let mut outputs = open_outputs(output)?;
    let mut summary = String::from(
        "# Exploratory history/materialization run summary\n\nAll measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.\n\n| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |\n|---:|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for size in SIZES {
        let warmup_name = format!("events-{size}-warmup");
        match runner(output, size, &warmup_name) {
            Ok(trial) => {
                outputs
                    .trials
                    .write_all(csv_line(size, "warmup", 0, &trial).as_bytes())
                    .map_err(|error| error.to_string())?;
                outputs.trials.flush().map_err(|error| error.to_string())?;
            }
            Err(reason) => {
                record_failure(&mut outputs, size, "warmup", 0, &reason)?;
                return Err(format!("{warmup_name} failed: {reason}"));
            }
        }
        let mut measured = Vec::new();
        for index in 1..=5 {
            let trial_name = format!("events-{size}-trial-{index}");
            match runner(output, size, &trial_name) {
                Ok(trial) => {
                    outputs
                        .trials
                        .write_all(csv_line(size, "measured", index, &trial).as_bytes())
                        .map_err(|error| error.to_string())?;
                    outputs.trials.flush().map_err(|error| error.to_string())?;
                    measured.push(trial);
                }
                Err(reason) => {
                    record_failure(&mut outputs, size, "measured", index, &reason)?;
                    return Err(format!("{trial_name} failed: {reason}"));
                }
            }
        }
        let field = |get: fn(&Trial) -> Duration| {
            median_range(&measured.iter().map(get).collect::<Vec<_>>())
        };
        let fmt = |value: (u128, u128, u128)| format!("{} ({}–{})", value.0, value.1, value.2);
        summary.push_str(&format!(
            "| {size} | {} | {} | {} | {} | {} | {} | {} |\n",
            fmt(field(|trial| trial.encode)),
            fmt(field(|trial| trial.append)),
            fmt(field(|trial| trial.replay)),
            fmt(field(|trial| trial.row_rebuild)),
            fmt(field(|trial| trial.column_rebuild)),
            fmt(field(|trial| trial.direct_row)),
            fmt(field(|trial| trial.direct_column))
        ));
    }
    outputs
        .summary
        .set_len(0)
        .map_err(|error| error.to_string())?;
    outputs
        .summary
        .seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    outputs
        .summary
        .write_all(summary.as_bytes())
        .map_err(|error| error.to_string())?;
    outputs.summary.flush().map_err(|error| error.to_string())
}

fn main() -> Result<(), String> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            unique_default_output(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        });
    if std::env::args_os().nth(2).is_some() {
        return Err("usage: history_views [NEW_OUTPUT_DIRECTORY]".into());
    }
    create_output_directory(&output)?;
    execute(&output, run_trial)?;
    println!("results retained in {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_observation_and_failure_are_retained() {
        let output = unique_default_output(&std::env::temp_dir());
        create_output_directory(&output).expect("exclusive test output");
        let mut calls = 0;
        let result = execute(&output, |root, size, name| {
            calls += 1;
            if calls == 2 {
                Err("injected failure, with detail".into())
            } else {
                run_trial(root, size, name)
            }
        });
        assert!(result.is_err());
        let trials = fs::read_to_string(output.join("trials.csv")).expect("retained trials");
        assert!(trials.contains("100,warmup,0,"));
        let failures = fs::read_to_string(output.join("failures.csv")).expect("retained failures");
        assert!(failures.contains("100,measured,1,\"injected failure, with detail\""));
        let summary = fs::read_to_string(output.join("summary.md")).expect("incomplete summary");
        assert!(summary.contains("Run incomplete"));
        assert!(!summary.contains("All measured trials passed"));
        fs::remove_dir_all(output).expect("test-owned cleanup");
    }
}
