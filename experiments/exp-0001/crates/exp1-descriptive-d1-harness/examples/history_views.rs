use exp1_descriptive_d1_harness::history_views::{
    Trial, create_output_directory, run_trial, unique_default_output,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn command(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .replace('\n', " | ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unavailable (command failed or absent)".into())
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
    let mut csv = String::from(
        "event_count,trial_kind,trial_index,payload_bytes,physical_bytes,encode_ns,append_ns,replay_ns,row_rebuild_ns,column_rebuild_ns,direct_row_ns,direct_column_ns,encode_events_s,append_events_s,replay_events_s,row_rebuild_events_s,column_rebuild_events_s,direct_row_events_s,direct_column_events_s,correctness\n",
    );
    let mut summary = String::from(
        "# Exploratory history/materialization run summary\n\nAll measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.\n\n| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |\n|---:|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for size in SIZES {
        let warmup = run_trial(&output, size, &format!("events-{size}-warmup"))?;
        csv.push_str(&csv_line(size, "warmup", 0, &warmup));
        let mut measured = Vec::new();
        for index in 1..=5 {
            let trial = run_trial(&output, size, &format!("events-{size}-trial-{index}"))?;
            csv.push_str(&csv_line(size, "measured", index, &trial));
            measured.push(trial);
        }
        let field = |get: fn(&Trial) -> Duration| {
            median_range(&measured.iter().map(get).collect::<Vec<_>>())
        };
        let fmt = |v: (u128, u128, u128)| format!("{} ({}–{})", v.0, v.1, v.2);
        summary.push_str(&format!(
            "| {size} | {} | {} | {} | {} | {} | {} | {} |\n",
            fmt(field(|t| t.encode)),
            fmt(field(|t| t.append)),
            fmt(field(|t| t.replay)),
            fmt(field(|t| t.row_rebuild)),
            fmt(field(|t| t.column_rebuild)),
            fmt(field(|t| t.direct_row)),
            fmt(field(|t| t.direct_column))
        ));
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let revision = command("git", &["rev-parse", "HEAD"]);
    let dirty = !command("git", &["status", "--porcelain"]).starts_with("unavailable")
        && !command("git", &["status", "--porcelain"]).is_empty();
    let cpu = fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unavailable (/proc/cpuinfo field absent)".into());
    let mem = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unavailable (/proc/meminfo field absent)".into());
    let metadata = format!(
        "schema=EXPLORATORY-HV1/run-metadata-v1\nunix_timestamp={timestamp}\nrepository_revision={revision}\nrepository_dirty={dirty}\ntoolchain={}\ntarget={}\nbuild_profile=release; overflow-checks=true; locked=true; offline=true\nworkload=deterministic single-thread; sizes=100,1000,10000; entities=17; warmups=1; measured_trials=5; batching=none; queue_depth=1\npayload_label=EXPLORATORY-HV1/entity-time-value-v1 (separate from SOP1/SOP2)\ndurability=D1 ordinary writes only; no fsync; no crash-survival or durable-canonical-commit claim\ncache_state=uncontrolled; warm-up and prior trials may affect page cache\nos_kernel={}\ncpu={}\nmemory={}\nfilesystem={}\nstorage_model_interface=unavailable (not reliably exposed to unprivileged container)\nmount_options=unavailable (output-path mount mapping not resolved)\npower_performance_mode=unavailable (not reliably exposed)\nvirtualization_container={}\nram_speed=unavailable (not exposed)\nperf_counters=not collected (optional)\n",
        command("rustc", &["--version", "--verbose"]),
        command("rustc", &["-vV"]),
        command("uname", &["-a"]),
        cpu,
        mem,
        command("df", &["-T", output.to_str().unwrap_or(".")]),
        if fs::read_to_string("/proc/1/cgroup")
            .map(|s| s.contains("docker") || s.contains("kubepods") || s.contains("containerd"))
            .unwrap_or(false)
        {
            "container indicators present"
        } else {
            "unavailable/undetected from /proc/1/cgroup"
        }
    );
    fs::write(output.join("trials.csv"), csv).map_err(|error| error.to_string())?;
    fs::write(output.join("summary.md"), summary).map_err(|error| error.to_string())?;
    fs::write(output.join("environment.txt"), metadata).map_err(|error| error.to_string())?;
    println!("results retained in {}", output.display());
    Ok(())
}
