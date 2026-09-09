//! Common runner: engine timing excludes oracle, digests and output I/O.
use crate::{
    Answer, D1, Memory, Model, Op, Trace, answer_signature, column_digests, format, hex,
    record_digests, sha256,
};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
pub trait Engine {
    fn label(&self) -> String;
    fn durability(&self) -> String;
    fn execute(&mut self, op: &Op) -> Result<Answer, String>;
    fn records(&self) -> Result<Vec<Memory>, String>;
    /// Additional post-execution correctness checks; each is compared with oracle.
    fn finish(&mut self, expected: &[Memory]) -> Result<Vec<(String, u128)>, String>;
}
pub fn file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))
}
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    file(path)?
        .write_all(text.as_bytes())
        .map_err(|e| e.to_string())
}
fn command(program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_owned(),
        Ok(o) => format!(
            "unavailable: {program} status {}; {}",
            o.status,
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => format!("unavailable: {program}: {e}"),
    }
}
fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| format!("unavailable: {path}: {e}"))
}
fn powershell_command(args: &[&str]) -> String {
    let first = command("pwsh", args);
    if first.starts_with("unavailable:") {
        command("powershell", args)
    } else {
        first
    }
}
fn positive_rss(value: &str) -> String {
    match value.trim().parse::<u64>() {
        Ok(n) if n > 0 => n.to_string(),
        _ if value.starts_with("unavailable:") => value.into(),
        _ => format!("unavailable: invalid peak RSS: {value}"),
    }
}
pub fn peak_rss() -> String {
    if cfg!(target_os = "linux") {
        let status = read("/proc/self/status");
        let bytes = status.lines().find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()?
                .checked_mul(1024)
        });
        positive_rss(&bytes.map_or_else(
            || "unavailable: VmHWM absent or invalid".into(),
            |n| n.to_string(),
        ))
    } else if cfg!(target_os = "windows") {
        positive_rss(&powershell_command(&[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("(Get-Process -Id {}).PeakWorkingSet64", std::process::id()),
        ]))
    } else {
        "unavailable: no peak RSS probe for this OS".into()
    }
}
/// Cached per process for identical source labels throughout a series.
pub fn revision() -> &'static str {
    static REVISION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    REVISION.get_or_init(|| command("git", &["rev-parse", "HEAD"]))
}
/// The decoded trace, its original byte identity, and a pre-trial RSS baseline.
/// Fields are private to prevent mutation from silently invalidating that identity.
pub struct LoadedTrace {
    trace: Trace,
    input_sha256: String,
    rss_baseline_after_load: String,
}
impl std::ops::Deref for LoadedTrace {
    type Target = Trace;
    fn deref(&self) -> &Trace {
        &self.trace
    }
}
impl LoadedTrace {
    /// Generated in-memory correctness inputs are encoded/hashed once as well.
    pub fn from_trace(trace: Trace) -> Self {
        let input_sha256 = hex(&sha256(format::encode(&trace).as_bytes()));
        Self {
            trace,
            input_sha256,
            rss_baseline_after_load: peak_rss(),
        }
    }
    pub fn input_sha256(&self) -> &str {
        &self.input_sha256
    }
}
fn size(path: &Path) -> Result<u64, String> {
    let mut total = 0;
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        total += if p.is_dir() {
            size(&p)?
        } else {
            p.metadata().map_err(|e| e.to_string())?.len()
        };
    }
    Ok(total)
}
fn metadata(
    output: &Path,
    label: &str,
    durability: &str,
    trace: &LoadedTrace,
    retain_store: bool,
) -> String {
    let os = if cfg!(windows) {
        powershell_command(&[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,TotalVisibleMemorySize | Format-List; Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | Format-List; Get-Volume | Select-Object DriveLetter,FileSystem,AllocationUnitSize | Format-Table; Get-PhysicalDisk | Select-Object FriendlyName,MediaType,Size | Format-Table",
        ])
    } else {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            command("uname", &["-a"]),
            read("/proc/cpuinfo"),
            read("/proc/meminfo"),
            command("df", &["-T", output.to_str().unwrap_or(".")]),
            command("lsblk", &["-o", "NAME,MODEL,ROTA,SIZE,FSTYPE,MOUNTPOINTS"])
        )
    };
    let mut environment = format!(
        "CMT1-results\t1\nexperiment=EXP-0002\nhypothesis=HYP-0002\nengine={label}\ncandidate_durability={D1}\nengine_durability={durability}\nequivalence=non-equivalent native durability; no winner claim\ncreated_unix_ns={}\nrevision={}\nporcelain={:?}\ncommand={:?}\nrustc={}\ncargo={}\nos={}\narch={}\ndebug_assertions={}\nRUSTFLAGS={:?}\nCARGO_ENCODED_RUSTFLAGS={:?}\ncache=uncontrolled OS/device caches; fresh file per trial; one warm-up then five measured\nconcurrency=1; queue_depth=1; batch_width=1\nclock=std::time::Instant, nanoseconds; probe overhead uncalibrated\nrss_method=bytes; Linux VmHWM converted from KiB or Windows PeakWorkingSet64 via pwsh then powershell, process-lifetime high water including harness/oracle\nfile_size_method=sum logical lengths of store files, not allocated extents\nhardware_and_filesystem={}\nuncollected=CPU microcode/topology beyond probes, firmware, RAM speed, power/thermal policy, affinity, NUMA, swap/limits, virtualization configuration, mount options, device caches/PLP, compiler linker flags beyond recorded environment; not configured or portable probes unavailable\n",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        revision(),
        command(
            "git",
            &["status", "--porcelain=v1", "--untracked-files=all"]
        ),
        std::env::args().collect::<Vec<_>>(),
        command("rustc", &["-vV"]),
        command("cargo", &["-V"]),
        std::env::consts::OS,
        std::env::consts::ARCH,
        cfg!(debug_assertions),
        std::env::var("RUSTFLAGS"),
        std::env::var("CARGO_ENCODED_RUSTFLAGS"),
        os
    );
    let generator_size = match (trace.name.as_str(), trace.seed) {
        ("small-deterministic", 7) => Some("small"),
        ("repeated-mutations-1k", 7) => Some("1k"),
        ("repeated-mutations-10k", 7) => Some("10k"),
        ("repeated-mutations-100k", 7) => Some("100k"),
        _ => None,
    };
    let regeneration = generator_size.map_or_else(
        || "unavailable: custom trace; preserve the operator's original input".into(),
        |size| {
            let toolchain = if cfg!(windows) { "1.89.0-x86_64-pc-windows-gnu" } else { "1.89.0" };
            // A fixed relative destination avoids shell interpretation of caller paths.
            format!("cargo +{toolchain} run --release --manifest-path experiments/convergence-memory/Cargo.toml -p cm-harness --locked --offline -- generate {size} regenerated-{size}.cmt")
        },
    );
    environment.push_str(&format!(
        "generator_seed={}\ntrace_id={}\nrecord_count={}\noperation_count={}\nshape={:?}\ninput_sha256={}\nregeneration_command={regeneration}\nregeneration_note=run from repository root; destination must not exist; verify regenerated SHA-256 against input_sha256, including for edited named traces\nretain_store={retain_store}\nsource_retention=source.sha256 and source.patch; no source tree copy\n",
        trace.seed, trace.name, trace.records, trace.ops.len(), trace.shape,
        trace.input_sha256.clone()
    ));
    environment.push_str(&format!(
        "rss_baseline_after_load={}\n",
        trace.rss_baseline_after_load
    ));
    environment
}
fn snapshot(output: &Path) -> Result<(), String> {
    let output_absolute = output.canonicalize().map_err(|e| e.to_string())?;
    let source_root = std::path::PathBuf::from(command("git", &["rev-parse", "--show-toplevel"]));
    write(
        &output.join("source.patch"),
        &command("git", &["diff", "HEAD", "--binary"]),
    )?;
    let paths = command(
        "git",
        &[
            "-C",
            source_root.to_str().ok_or("source root is not UTF-8")?,
            "ls-files",
            "--full-name",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    );
    let mut manifest = String::new();
    for name in paths.lines() {
        let relative = Path::new(name);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }
        let p = source_root.join(relative);
        if !(name.starts_with("experiments/convergence-memory/")
            || name.starts_with("experiments/convergence-memory-legacy/"))
            || name.contains("/target/")
        {
            continue;
        }
        if !p.is_file() {
            continue;
        }
        if p.canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&output_absolute)
        {
            continue;
        }
        let bytes = fs::read(&p).map_err(|e| e.to_string())?;
        manifest.push_str(&format!("{}  {name}\n", hex(&sha256(&bytes))));
    }
    write(&output.join("source.sha256"), &manifest)
}
pub struct TrialReport {
    pub mismatches: usize,
    pub samples: Vec<(String, u128)>,
}
pub fn trial<E: Engine>(
    trace: &LoadedTrace,
    engine: &mut E,
    output: &Path,
) -> Result<TrialReport, String> {
    let durability = engine.durability();
    let mut observations = crate::results::StreamingResults::create(
        &output.join("observations.cmt"),
        trace.input_sha256.clone(),
        engine.label(),
        durability.clone(),
        trace.ops.len(),
    )?;
    let mut observed_records = vec![];
    let mut observed_columns = vec![];
    let mut model = Model::default();
    let mut mismatch = 0;
    let mut samples = Vec::new();
    let mut results = file(&output.join("results.cmt"))?;
    writeln!(
        results,
        "CMT1-results\t1\ntrace={}\ninput_sha256={}\ncandidate_durability={D1}\nengine_durability={durability}",
        trace.name,
        trace.input_sha256.clone()
    )
    .map_err(|e| e.to_string())?;
    for (id, op) in &trace.ops {
        let expected = model.apply(op);
        let start = Instant::now();
        let actual = engine.execute(op);
        let elapsed = start.elapsed().as_nanos();
        samples.push((op.kind().into(), elapsed));
        let expected = answer_signature(&expected);
        let actual = actual
            .map(|a| answer_signature(&a))
            .unwrap_or_else(|e| format!("ERROR:{e}"));
        observations.operation(&actual)?;
        let valid = actual == expected;
        if !valid {
            mismatch += 1;
            eprintln!(
                "op {id} ({}) mismatch: actual {actual}; expected {expected}",
                op.kind()
            );
        }
        writeln!(
            results,
            "op\t{id}\t{}\t{elapsed}\t{valid}\t{actual}\texpected\t{expected}",
            op.kind()
        )
        .map_err(|e| e.to_string())?;
    }
    let expected = model.records();
    match engine.records() {
        Ok(actual) => {
            observed_records = record_digests(&actual);
            observed_columns = column_digests(&actual);
            let valid = record_digests(&actual) == record_digests(&expected)
                && column_digests(&actual) == column_digests(&expected);
            if !valid {
                mismatch += 1;
            }
            writeln!(results,"final_valid={valid}\nfinal_records={:?}\nfinal_columns={:?}\nexpected_records={:?}\nexpected_columns={:?}",record_digests(&actual),column_digests(&actual),record_digests(&expected),column_digests(&expected)).map_err(|e|e.to_string())?;
        }
        Err(e) => {
            mismatch += 1;
            writeln!(results, "final_error={e}").map_err(|e| e.to_string())?;
        }
    }
    match engine.finish(&expected) {
        Ok(stages) => samples.extend(stages),
        Err(e) => {
            mismatch += 1;
            writeln!(results, "reconstruction_error={e}").map_err(|e| e.to_string())?;
        }
    }
    writeln!(results, "engine={}\nmismatches={mismatch}", engine.label())
        .map_err(|e| e.to_string())?;
    observations.finish(&observed_records, &observed_columns)?;
    Ok(TrialReport {
        mismatches: mismatch,
        samples,
    })
}
pub fn series<E: Engine>(
    trace: &LoadedTrace,
    output: &Path,
    label: &str,
    durability: &str,
    create: impl Fn(&Path) -> Result<E, String>,
) -> Result<(), String> {
    series_with_retention(trace, output, label, durability, false, create)
}

/// Forensic opt-in retains trial stores and a trace copy, never a source tree.
pub fn series_with_retention<E: Engine>(
    trace: &LoadedTrace,
    output: &Path,
    label: &str,
    durability: &str,
    retain_store: bool,
    create: impl Fn(&Path) -> Result<E, String>,
) -> Result<(), String> {
    fs::create_dir(output).map_err(|e| format!("exclusive output {}: {e}", output.display()))?;
    let env = metadata(output, label, durability, trace, retain_store);
    write(&output.join("environment.txt"), &env)?;
    snapshot(output)?;
    if retain_store {
        write(&output.join("trace.cmt"), &format::encode(trace))?;
    }
    let mut all: BTreeMap<String, Vec<u128>> = BTreeMap::new();
    let mut failed = false;
    let mut trials = file(&output.join("trials.csv"))?;
    writeln!(
        trials,
        "phase,index,stage,ns,store_logical_bytes,peak_rss,valid"
    )
    .map_err(|e| e.to_string())?;
    for index in 0..6 {
        let phase = if index == 0 { "warmup" } else { "measured" };
        let dir = output.join(format!("{phase}-{index}"));
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        write(&dir.join("environment.txt"), &env)?;
        let store = dir.join("store");
        fs::create_dir(&store).map_err(|e| e.to_string())?;
        let start = Instant::now();
        let mut engine = match create(&store) {
            Ok(e) => e,
            Err(e) => {
                failed = true;
                write(&dir.join("failure.txt"), &e)?;
                if !retain_store {
                    remove_trial_store(&dir)?;
                }
                continue;
            }
        };
        let setup = start.elapsed().as_nanos();
        let report = match trial(trace, &mut engine, &dir) {
            Ok(r) => r,
            Err(e) => {
                failed = true;
                drop(engine);
                write(&dir.join("failure.txt"), &e)?;
                if !retain_store {
                    remove_trial_store(&dir)?;
                }
                continue;
            }
        };
        failed |= report.mismatches > 0;
        let mut per_stage: BTreeMap<String, u128> = BTreeMap::new();
        per_stage.insert("setup".into(), setup);
        for (stage, ns) in &report.samples {
            *per_stage.entry(stage.clone()).or_default() += ns;
            if index > 0 {
                all.entry(stage.clone()).or_default().push(*ns);
            }
        }
        let total: u128 = report
            .samples
            .iter()
            .filter(|(s, _)| {
                [
                    "insert",
                    "get",
                    "update",
                    "replace",
                    "guard",
                    "delete",
                    "equal",
                    "aggregate",
                    "page",
                ]
                .contains(&s.as_str())
            })
            .map(|(_, n)| *n)
            .sum();
        per_stage.insert("complete_operations".into(), total);
        let rss = peak_rss();
        let bytes = size(&store)?;
        // Replay/digest validation and size observation precede cleanup. Release
        // the mmap/appender before deleting its files, including on Windows.
        drop(engine);
        if !retain_store {
            remove_trial_store(&dir)?;
        }
        for (stage, ns) in per_stage {
            writeln!(
                trials,
                "{phase},{index},{stage},{ns},{bytes},{rss:?},{}",
                report.mismatches == 0
            )
            .map_err(|e| e.to_string())?;
        }
        write(
            &dir.join("throughput.txt"),
            &format!(
                "candidate_durability={D1}\nengine_durability={durability}\nengine={label}\nphase={phase}\nvalid={}\noperations={}\noperation_time_ns={total}\noperations_per_second={}\n{}",
                report.mismatches == 0,
                trace.ops.len(),
                trace.ops.len() as f64 * 1e9 / total.max(1) as f64,
                raw_references(&dir)?
            ),
        )?;
    }
    let mut summary = format!(
        "CMT1-results\t1\ncandidate_durability={D1}\nengine_durability={durability}\nengine={label}\nvalid={}\nlegacy_internal_stages=unavailable through public adapter; no isolated append/replay/decode/row/column timing\nquantiles=nearest-rank; warmup excluded; raw samples in results.cmt and trials.csv\nstage,count,min,p50,p90,p95,p99,max\n",
        !failed
    );
    for (stage, mut values) in all {
        values.sort();
        let q = |p: usize| values[(values.len() * p).div_ceil(100).saturating_sub(1)];
        summary.push_str(&format!(
            "{stage},{},{},{},{},{},{},{}\n",
            values.len(),
            values[0],
            q(50),
            q(90),
            q(95),
            q(99),
            values[values.len() - 1]
        ));
    }
    write(&output.join("summary.txt"), &summary)?;
    if failed {
        Err("invalid trial(s); all available failures/results retained".into())
    } else {
        Ok(())
    }
}

fn raw_references(trial: &Path) -> Result<String, String> {
    let mut references = String::new();
    for name in ["results.cmt", "observations.cmt"] {
        let path = trial.join(name).canonicalize().map_err(|e| e.to_string())?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        references.push_str(&format!(
            "raw_path={:?}\nraw_sha256={}\n",
            path,
            hex(&sha256(&bytes))
        ));
    }
    Ok(references)
}

fn remove_trial_store(trial: &Path) -> Result<(), String> {
    let trial = trial.canonicalize().map_err(|e| e.to_string())?;
    let store = trial.join("store");
    if fs::symlink_metadata(&store)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
        || store.canonicalize().map_err(|e| e.to_string())? != store
    {
        return Err("refusing cleanup outside the newly created trial/store directory".into());
    }
    fs::remove_dir_all(&store).map_err(|e| format!("store cleanup {}: {e}", store.display()))
}

/// Consume only the documented trailing forensic flag; arity checks reject others.
pub fn take_retain_store(args: &mut Vec<String>) -> bool {
    if args.last().is_some_and(|arg| arg == "--retain-store") {
        args.pop();
        true
    } else {
        false
    }
}
pub fn load(path: &Path) -> Result<LoadedTrace, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let input_sha256 = hex(&sha256(&bytes));
    let trace = format::decode(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)?;
    drop(bytes);
    let rss_baseline_after_load = peak_rss();
    Ok(LoadedTrace {
        trace,
        input_sha256,
        rss_baseline_after_load,
    })
}
pub fn generate_cli(args: &[String]) -> Result<bool, String> {
    if args.first().is_none_or(|s| s != "generate") {
        return Ok(false);
    }
    if args.len() != 3 {
        return Err("generate small|1k|10k|100k OUTPUT.cmt".into());
    }
    let trace = match args[1].as_str() {
        "small" => crate::generate::small(),
        "1k" => crate::generate::repeated(1000, 7),
        "10k" => crate::generate::repeated(10000, 7),
        "100k" => crate::generate::repeated(100000, 7),
        _ => return Err("unknown size".into()),
    };
    write(Path::new(&args[2]), &format::encode(&trace))?;
    Ok(true)
}
pub fn test_directory(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "cm-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rss_probe_is_positive_bytes_or_explicitly_unavailable() {
        let value = peak_rss();
        assert!(
            value.parse::<u64>().is_ok_and(|n| n > 0) || value.starts_with("unavailable:"),
            "{value}"
        );
        assert!(positive_rss("0").starts_with("unavailable:"));
        assert!(positive_rss("unexpected").starts_with("unavailable:"));
    }
    #[test]
    fn load_captures_original_bytes_and_baseline_before_trials() {
        let path = test_directory("load-identity");
        let bytes = format::encode(&crate::generate::small());
        write(&path, &bytes).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.input_sha256(), hex(&sha256(bytes.as_bytes())));
        assert_eq!(loaded.trace, crate::generate::small());
        assert!(
            loaded
                .rss_baseline_after_load
                .parse::<u64>()
                .is_ok_and(|n| n > 0)
                || loaded.rss_baseline_after_load.starts_with("unavailable:")
        );
        fs::remove_file(path).unwrap();
    }
}
